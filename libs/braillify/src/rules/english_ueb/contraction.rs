//! Intra-word contraction dispatch (UEB §10).
//!
//! Each [`ContractionRule`] offers a match at a position; the engine picks the
//! **longest** match, breaking ties by **lowest priority number** — this is the
//! §10.10 preference rule encoded structurally. Unmatched positions fall back to
//! single letters (§4.1 alphabet), realised via [`crate::english::encode_english`].

#[cfg(test)]
use crate::english::encode_english;

/// A contraction match starting at a word position.
pub struct ContractionMatch {
    /// Braille cells produced for the matched span.
    pub cells: Vec<u8>,
    /// Number of source characters consumed.
    pub consumed: usize,
    /// Tie-breaker among equal-length matches (lower wins).
    pub priority: u16,
    /// §10.10.1/§10.11: a morpheme- or pronunciation-validated contraction whose
    /// span must NOT be split by a cheaper generic contraction in the §10.10.2
    /// cell-minimiser. Set by the gated rules — restricted `be`/`con`/`dis`
    /// (§10.6.4), the pronunciation-gated initial-letter contractions like `part`
    /// (§10.7), and a `en`/`in` kept against an overlapping coda `ness` (§10.6.8,
    /// e.g. `captain·ess`). Keeps greedy's "consume-to-block" guarantee that the
    /// cell-min would otherwise lose (`apartheid`→`a·part·heid`, not `ar·the·id`).
    pub protect_span: bool,
}

/// One UEB contraction rule (§10.x). `word` is the lowercased letter slice.
pub trait ContractionRule: Send + Sync {
    /// Offer a match starting at `pos`, or `None`.
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch>;
}

/// Longest-prefix match of `word[pos..]` against a PHF map of `&str -> cell`.
pub fn match_longest(
    word: &[char],
    pos: usize,
    map: &phf::Map<&'static str, u8>,
    priority: u16,
) -> Option<ContractionMatch> {
    let mut best: Option<(usize, u8)> = None;
    for (key, &cell) in map.entries() {
        let klen = key.chars().count();
        if pos + klen <= word.len()
            && key
                .chars()
                .zip(&word[pos..pos + klen])
                .all(|(k, w)| k == *w)
            && best.is_none_or(|(bl, _)| klen > bl)
        {
            best = Some((klen, cell));
        }
    }
    best.map(|(klen, cell)| ContractionMatch {
        cells: vec![cell],
        consumed: klen,
        priority,
        protect_span: false,
    })
}

/// Dispatches contraction rules over a single (lowercased) word.
#[derive(Default)]
pub struct ContractionEngine {
    rules: Vec<Box<dyn ContractionRule>>,
}

impl ContractionEngine {
    /// Register a contraction rule.
    pub fn register(&mut self, rule: Box<dyn ContractionRule>) {
        self.rules.push(rule);
    }

    /// Every contraction match offered at `pos`, across all registered rules —
    /// the §10.10 candidate set from which the cell-minimising DP in
    /// [`super::rule_10_9::encode_with_longer_shortforms`] selects the sequence
    /// occupying the fewest cells (§10.10.2), after structural filtering (§10.10.1).
    pub fn matches_at(&self, word: &[char], pos: usize) -> Vec<ContractionMatch> {
        self.rules
            .iter()
            .filter_map(|rule| rule.try_match(word, pos))
            .collect()
    }

    /// Encode a lowercased letter slice to braille cells.
    /// Returns `None` if a character cannot be encoded as an English letter.
    #[cfg(test)]
    pub fn encode_word(&self, word: &[char]) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(word.len());
        let mut pos = 0;
        while pos < word.len() {
            match self.best_match(word, pos) {
                Some(m) => {
                    out.extend(m.cells);
                    pos += m.consumed;
                }
                None => {
                    // §4.2: an accented letter becomes its accent indicator plus
                    // the base letter; otherwise a plain alphabet letter (§4.1).
                    if let Some(cells) = super::rule_4::accent_cells(word[pos]) {
                        out.extend(cells);
                    } else {
                        out.push(encode_english(word[pos]).ok()?);
                    }
                    pos += 1;
                }
            }
        }
        Some(out)
    }

    /// Greedy longest-match (test-only helper for [`Self::encode_word`]); the
    /// production path is the §10.10.2 cell-minimising DP over [`Self::matches_at`].
    #[cfg(test)]
    fn best_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let mut best: Option<ContractionMatch> = None;
        for rule in &self.rules {
            if let Some(m) = rule.try_match(word, pos) {
                let better = best.as_ref().is_none_or(|b| {
                    m.consumed > b.consumed || (m.consumed == b.consumed && m.priority < b.priority)
                });
                if better {
                    best = Some(m);
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    struct StaticRule {
        pattern: &'static [char],
        cell: char,
        priority: u16,
    }

    impl ContractionRule for StaticRule {
        fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
            word.get(pos..pos + self.pattern.len())
                .is_some_and(|slice| slice == self.pattern)
                .then(|| ContractionMatch {
                    cells: vec![decode_unicode(self.cell)],
                    consumed: self.pattern.len(),
                    priority: self.priority,
                    protect_span: false,
                })
        }
    }

    #[test]
    fn plain_letters_fall_back_to_alphabet() {
        let eng = ContractionEngine::default();
        let cells = eng.encode_word(&['c', 'a', 't']).unwrap();
        assert_eq!(
            cells,
            vec![
                decode_unicode('⠉'),
                decode_unicode('⠁'),
                decode_unicode('⠞')
            ]
        );
    }

    #[test]
    fn accented_letters_fall_back_to_accent_cells() {
        let eng = ContractionEngine::default();
        let cells = eng.encode_word(&['é']).unwrap();

        assert_eq!(
            cells,
            vec![
                decode_unicode('⠘'),
                decode_unicode('⠌'),
                decode_unicode('⠑')
            ]
        );
    }

    #[test]
    fn registered_rule_is_used_before_letter_fallback() {
        let mut eng = ContractionEngine::default();
        eng.register(Box::new(StaticRule {
            pattern: &['c', 'h'],
            cell: '⠡',
            priority: 10,
        }));

        let cells = eng.encode_word(&['c', 'h', 'a', 't']).unwrap();

        assert_eq!(
            cells,
            vec![
                decode_unicode('⠡'),
                decode_unicode('⠁'),
                decode_unicode('⠞')
            ]
        );
    }

    #[test]
    fn lower_priority_wins_equal_length_match() {
        let mut eng = ContractionEngine::default();
        eng.register(Box::new(StaticRule {
            pattern: &['a'],
            cell: '⠁',
            priority: 20,
        }));
        eng.register(Box::new(StaticRule {
            pattern: &['a'],
            cell: '⠃',
            priority: 10,
        }));

        let cells = eng.encode_word(&['a']).unwrap();

        assert_eq!(cells, vec![decode_unicode('⠃')]);
    }

    #[test]
    fn match_longest_accepts_runtime_word_slice() {
        static MAP: phf::Map<&'static str, u8> = phf::phf_map! {
            "a" => decode_unicode('⠁'),
            "ab" => decode_unicode('⠃'),
        };
        let word: Vec<char> = std::hint::black_box("abc").chars().collect();

        let matched = match_longest(&word, std::hint::black_box(0), &MAP, 42).unwrap();

        assert_eq!(matched.cells, vec![decode_unicode('⠃')]);
        assert_eq!(matched.consumed, 2);
        assert_eq!(matched.priority, 42);
    }
}
