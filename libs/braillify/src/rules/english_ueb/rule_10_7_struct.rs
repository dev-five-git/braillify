//! §10.7 initial-letter contractions gated by morpheme STRUCTURE, not pronunciation
//! (feature `english_ueb_cmudict`).
//!
//! §10.7 is spelling-first — an initial-letter contraction may be used wherever its
//! letters occur — but §10.11 forbids using it where the letters do not begin a real
//! word component (it would bridge into / out of an unrelated morpheme and hinder
//! recognition). This rule applies a contraction at `pos` only when it STARTS A
//! COMPONENT (the word start, just after a hyphen/apostrophe, or right after a real
//! dictionary word) AND does not bridge an internal compound seam.
//!
//! - `lord` (⠐⠇), `work` (⠐⠺), `know` (⠐⠅) use the COMPONENT-START gate:
//!   `Gay·lord`, `stone·work`, `ac·know·ledge` start a real component; `ch·lordane`,
//!   `D·workin` (a non-word prefix) and `Luck·now` (bridges a `luck|now` seam) spell
//!   out. A 1-letter prefix is not a component — CMUdict records `d`/`p` as words.
//! - `part` (⠐⠏) is spelling-first with NO component-start gate (`Spar·tan`,
//!   `im·part·ial`, `part·erre` contract); only a `part`+`h` (`Par·thenon` /θ/ spells
//!   vs `apart·heid` /t/+/h/ contracts) is deferred to the pronunciation gate.
//!
//! Boundaries come from word STRUCTURE (the CMUdict word list), never from braille
//! test outputs. Conservative by design: an unrecognised prefix → spell out, a safe
//! miss rather than a wrong contraction.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule};
use super::pronunciation::PronunciationProvider;
use crate::unicode::decode_unicode;

/// Structure-gated §10.7 initial-letter contractions → (prefix cell, first-letter cell).
static STRUCT_CONTRACTIONS: phf::Map<&'static str, [u8; 2]> = phf_map! {
    "lord" => [decode_unicode('⠐'), decode_unicode('⠇')],
    "work" => [decode_unicode('⠐'), decode_unicode('⠺')],
    "know" => [decode_unicode('⠐'), decode_unicode('⠅')],
};

/// Structurally-gated §10.7 initial-letter contraction rule.
pub struct StructuralInitialContractionRule {
    provider: Box<dyn PronunciationProvider>,
}

impl StructuralInitialContractionRule {
    /// Build the rule with the word source used to detect component boundaries.
    pub fn new(provider: Box<dyn PronunciationProvider>) -> Self {
        Self { provider }
    }

    fn is_word(&self, chars: &[char]) -> bool {
        let s: String = chars.iter().collect();
        !self.provider.pronunciations(&s).is_empty()
    }

    /// The contraction at `pos` begins a real word component: the word start, just
    /// after a hyphen/apostrophe boundary, or right after a recorded dictionary word
    /// (`Gay|lord`, `stone|work`). A non-word prefix (`ch`, `D`) is NOT a component,
    /// so a contraction whose letters merely fall there spells out.
    fn starts_component(&self, word: &[char], pos: usize) -> bool {
        pos == 0
            || matches!(word.get(pos - 1), Some('-' | '\u{2013}' | '\u{2014}' | '\''))
            // A real word prefix of ≥2 letters — CMUdict records single letters
            // (`d`) as words, so a 1-letter prefix is NOT a component (`D·workin`).
            || (pos >= 2 && self.is_word(&word[..pos]))
    }

    /// §10.11: the contraction's letters bridge an internal compound seam — some
    /// split inside `[pos, pos+len)` has a real ≥2-letter word on BOTH sides
    /// (`Luck|now`). The ≥2 floor excludes CMUdict single-letter entries (`p`, `d`)
    /// that would spuriously split a non-compound.
    fn crosses_seam(&self, word: &[char], pos: usize, len: usize) -> bool {
        (1..len).any(|split| {
            let left = &word[..pos + split];
            let right = &word[pos + split..];
            left.len() >= 2 && right.len() >= 2 && self.is_word(left) && self.is_word(right)
        })
    }

    /// §10.7 `part` (⠐⠏): spelling-first — `part` is the morpheme almost wherever
    /// `p·a·r·t` occurs (`Spar·tan`, `im·part·ial`, `part·erre`, `passe-part·out`), so
    /// unlike `lord`/`work` it does NOT need the component-start gate. The ONE
    /// ambiguity is `part`+`h`, where `t·h` may be a /θ/ digraph (`Par·thenon` spells,
    /// `apart·heid` contracts); resolving that needs the word's sound, so a `part`+`h`
    /// is DEFERRED to the §10.7 pronunciation gate ([`super::rule_10_7_pron`]) and not
    /// claimed here. No compound-seam check is applied: the corpus shows no `part`
    /// spell-out from bridging, and a coincidental word|word split (`Spar|tan`,
    /// `par|ty`) must NOT block the genuine `part` morpheme.
    fn part_allowed(&self, word: &[char], pos: usize) -> bool {
        let end = pos + 4;
        end <= word.len()
            && ['p', 'a', 'r', 't']
                .iter()
                .zip(&word[pos..end])
                .all(|(k, w)| k == w)
            && word.get(end) != Some(&'h')
    }
}

impl ContractionRule for StructuralInitialContractionRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        for (key, &cells) in STRUCT_CONTRACTIONS.entries() {
            let klen = key.chars().count();
            let end = pos + klen;
            if end <= word.len()
                && key.chars().zip(&word[pos..end]).all(|(k, w)| k == *w)
                && self.starts_component(word, pos)
                && !self.crosses_seam(word, pos, klen)
            {
                return Some(ContractionMatch {
                    cells: vec![cells[0], cells[1]],
                    consumed: klen,
                    priority: 55,
                    // §10.10.1: structurally validated at a component start, so the
                    // cell-minimiser must not split it with a cheaper generic sign.
                    protect_span: true,
                });
            }
        }
        self.part_allowed(word, pos).then(|| ContractionMatch {
            cells: vec![decode_unicode('⠐'), decode_unicode('⠏')],
            consumed: 4,
            priority: 55,
            protect_span: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::pronunciation::cmudict::CmuDictProvider;
    use super::*;

    fn rule() -> StructuralInitialContractionRule {
        StructuralInitialContractionRule::new(Box::new(CmuDictProvider::new()))
    }

    fn try_at(word: &str, pos: usize) -> Option<(Vec<u8>, usize)> {
        let chars: Vec<char> = word.chars().collect();
        rule().try_match(&chars, pos).map(|m| (m.cells, m.consumed))
    }

    /// The contraction starts a real component → used (the prefix is the word start
    /// or a recorded dictionary word).
    #[rstest::rstest]
    #[case::lordosis("lordosis", 0, '⠇')] // word start
    #[case::landlord("landlord", 4, '⠇')] // land|lord
    #[case::lordship("lordship", 0, '⠇')]
    #[case::stonework("stonework", 5, '⠺')] // stone|work
    #[case::network("network", 3, '⠺')] // net|work
    #[case::homework("homework", 4, '⠺')] // home|work
    #[case::acknowledge("acknowledge", 2, '⠅')] // ac|know
    #[case::knowledge("knowledge", 0, '⠅')] // word start
    fn contracts_at_component_start(#[case] word: &str, #[case] pos: usize, #[case] letter: char) {
        assert_eq!(
            try_at(word, pos),
            Some((vec![decode_unicode('⠐'), decode_unicode(letter)], 4))
        );
    }

    /// The contraction does NOT start a component (a non-word prefix) → spelled out.
    #[rstest::rstest]
    #[case::chlordane("chlordane", 2)] // `ch` is not a word
    #[case::dworkin("dworkin", 1)] // `D` is not a word
    fn spells_out_off_component_start(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    /// §10.11: a contraction whose letters bridge a compound seam spells out, even
    /// when it starts a component (`Luck|now` — both halves ≥2-letter words).
    #[rstest::rstest]
    #[case::lucknow("lucknow", 3)] // luck|now
    fn spells_out_across_compound_seam(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    #[test]
    fn structural_contraction_can_start_component_without_crossing_seam() {
        let chars: Vec<char> = "stonework".chars().collect();
        let rule = rule();

        assert!(rule.starts_component(&chars, 5));
        assert!(!rule.crosses_seam(&chars, 5, 4));
        assert!(rule.try_match(&chars, 5).is_some());
    }

    #[test]
    fn structural_contraction_detects_crossing_compound_seam() {
        let chars: Vec<char> = "lucknow".chars().collect();
        let rule = rule();

        assert!(rule.starts_component(&chars, 3));
        assert!(rule.crosses_seam(&chars, 3, 4));
    }

    /// After a hyphen/apostrophe the contraction starts a component (`m'lord`).
    #[rstest::rstest]
    #[case::mlord("m'lord", 2, '⠇')]
    fn contracts_after_punctuation_boundary(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] letter: char,
    ) {
        assert_eq!(
            try_at(word, pos),
            Some((vec![decode_unicode('⠐'), decode_unicode(letter)], 4))
        );
    }

    /// §10.7 `part`: spelling-first, contracted wherever `part` is the morpheme and
    /// it is NOT followed by `h` (the only sound-sensitive case).
    #[rstest::rstest]
    #[case::impartial("impartial", 2)] // im·part·ial — t not before h
    #[case::parterre("parterre", 0)]
    #[case::spartan("spartan", 1)] // mid-word is fine (no component-start gate)
    #[case::party("party", 0)]
    fn part_contracts(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(
            try_at(word, pos),
            Some((vec![decode_unicode('⠐'), decode_unicode('⠏')], 4))
        );
    }

    /// `part`+`h` is deferred to the pronunciation gate (this rule returns `None`):
    /// `Parthenon` (t·h = /θ/, spells) vs `apartheid` (t·h = /t/+/h/, contracts).
    #[rstest::rstest]
    #[case::parthenon("parthenon", 0)]
    #[case::apartheid("apartheid", 1)]
    fn part_deferred_when_followed_by_h(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }
}
