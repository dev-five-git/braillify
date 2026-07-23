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
}
