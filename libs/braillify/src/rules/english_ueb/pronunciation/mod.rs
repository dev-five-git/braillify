//! Pronunciation layer for syllable-dependent UEB contractions (§10.6).
//!
//! The restricted lower groupsigns `be` (⠆) and `con` (⠒) may be used only when
//! the prefix forms the *first syllable* of the word — undecidable from spelling
//! alone (`become`/`beckon`, `benefit`/`beneficent`). A [`PronunciationProvider`]
//! supplies ARPABET phoneme data so the [`classifier`] can make this call;
//! without one, every decision is `Unknown` (→ spell out), so the layer is
//! safe-by-default.
//!
//! Source of pronunciation data: the CMU Pronouncing Dictionary (Simplified
//! BSD), embedded by [`cmudict`]. The decision rules derive from RUEB 2024
//! §10.6 (first-syllable) plus phonological facts, never from test outputs.

pub mod aligner;
pub mod classifier;
pub mod cmudict;

/// Decide whether an apostrophe-separated print sequence is one recorded
/// lexical word when the apostrophe is elided.
///
/// UEB §10.12.1 suppresses contractions when capitals are letters pronounced
/// separately, while §10.6.8 retains `en`/`in` inside an ordinarily pronounced
/// word.  A stylised spelling such as `O'PENing` is split into two parser runs,
/// so the case pattern of `PENing` alone cannot distinguish those situations.
/// Requiring the complete adjacent sequence (`opening`) to be in CMUdict gives
/// pronunciation evidence without recognising any particular corpus phrase.
/// The caller supplies the current ASCII run so unrelated quote punctuation is
/// never absorbed into the lookup.
pub(crate) fn apostrophe_elided_recorded_word_at(
    chars: &[char],
    run_start: usize,
    run_end: usize,
) -> bool {
    if run_start >= run_end
        || run_end > chars.len()
        || !chars[run_start..run_end]
            .iter()
            .all(|ch| ch.is_ascii_alphabetic())
    {
        return false;
    }

    let is_apostrophe = |ch: char| matches!(ch, '\'' | '\u{2019}');
    let is_member = |ch: char| ch.is_ascii_alphabetic() || is_apostrophe(ch);

    let mut start = run_start;
    while start > 0 && is_member(chars[start - 1]) {
        start -= 1;
    }
    let mut end = run_end;
    while end < chars.len() && is_member(chars[end]) {
        end += 1;
    }

    let segment = &chars[start..end];
    if !segment.iter().any(|ch| is_apostrophe(*ch)) {
        return false;
    }
    if segment.iter().enumerate().any(|(index, ch)| {
        is_apostrophe(*ch)
            && (index == 0
                || index + 1 == segment.len()
                || !segment[index - 1].is_ascii_alphabetic()
                || !segment[index + 1].is_ascii_alphabetic())
    }) {
        return false;
    }

    let normalized: String = segment
        .iter()
        .filter(|ch| ch.is_ascii_alphabetic())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    cmudict::is_recorded_word(&normalized)
}

/// One ARPABET phoneme: its base symbol (e.g. `B`, `AH`, `N`) and, for vowels,
/// the lexical stress (0 = unstressed, 1 = primary, 2 = secondary). In CMUdict
/// only vowels carry a stress digit, so `stress.is_some()` identifies a vowel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phoneme {
    /// ARPABET base symbol with any trailing stress digit removed.
    pub base: String,
    /// Lexical stress for vowels; `None` for consonants.
    pub stress: Option<u8>,
}

impl Phoneme {
    /// Whether this phoneme is a vowel (CMUdict marks stress only on vowels).
    pub fn is_vowel(&self) -> bool {
        self.stress.is_some()
    }
}

/// Parse a CMUdict phoneme token (`AH0`, `B`, `N`) into base symbol + stress.
pub fn parse_phoneme(tok: &str) -> Phoneme {
    if let Some(d @ b'0'..=b'2') = tok.as_bytes().last().copied() {
        return Phoneme {
            base: tok[..tok.len() - 1].to_string(),
            stress: Some(d - b'0'),
        };
    }
    let base = tok.to_string();
    Phoneme { base, stress: None }
}

/// Supplies ARPABET pronunciations for a lowercase word. Returns an empty vec
/// for unknown words, which the classifier treats as `Unknown` (→ spell out).
pub trait PronunciationProvider: Send + Sync {
    /// Every recorded pronunciation of `word` (CMUdict lists variants).
    fn pronunciations(&self, word: &str) -> Vec<Vec<Phoneme>>;
}

/// A provider with no data, so every word is unknown and the restricted
/// groupsigns are never applied. Currently exercised only by tests (it lets the
/// classifier run without the dictionary); when a no-dictionary production path
/// (e.g. wasm) needs it, drop the `cfg(test)` gate.
#[cfg(test)]
pub struct NoPronunciationProvider;

#[cfg(test)]
impl PronunciationProvider for NoPronunciationProvider {
    fn pronunciations(&self, _word: &str) -> Vec<Vec<Phoneme>> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::vowel_primary("AH1", "AH", Some(1))]
    #[case::vowel_unstressed("IH0", "IH", Some(0))]
    #[case::vowel_secondary("EH2", "EH", Some(2))]
    #[case::digit_not_stress("AH3", "AH3", None)]
    #[case::consonant_b("B", "B", None)]
    #[case::consonant_ng("NG", "NG", None)]
    fn parses_phoneme_base_and_stress(
        #[case] tok: &str,
        #[case] base: &str,
        #[case] stress: Option<u8>,
    ) {
        let ph = parse_phoneme(tok);
        assert_eq!(ph.base, base);
        assert_eq!(ph.stress, stress);
        assert_eq!(ph.is_vowel(), stress.is_some());
    }

    #[test]
    fn no_provider_yields_no_pronunciations() {
        assert!(NoPronunciationProvider.pronunciations("become").is_empty());
    }

    #[test]
    fn parses_runtime_consonant_token_without_stress() {
        let token = std::hint::black_box("NG");
        let ph = parse_phoneme(token);

        assert_eq!(ph.base, "NG");
        assert_eq!(ph.stress, None);
        assert!(!ph.is_vowel());
    }

    #[rstest::rstest]
    #[case::straight_apostrophe("O'PENing", 2, 8, true)]
    #[case::curly_apostrophe("O\u{2019}PENing", 2, 8, true)]
    #[case::no_join("PENing", 0, 6, false)]
    #[case::unknown_elision("rock'n", 5, 6, false)]
    #[case::empty_requested_run("abc", 1, 1, false)]
    #[case::run_end_out_of_bounds("abc", 0, 4, false)]
    #[case::requested_run_contains_nonletter("a1c", 0, 3, false)]
    fn classifies_apostrophe_elided_lexical_words(
        #[case] text: &str,
        #[case] run_start: usize,
        #[case] run_end: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(
            apostrophe_elided_recorded_word_at(&chars, run_start, run_end),
            expected
        );
    }
}
