//! CMUdict-backed pronunciation provider.
//!
//! Embeds the CMU Pronouncing Dictionary (Simplified BSD, © 1993-2014 Carnegie
//! Mellon University) and exposes ARPABET pronunciations to the §10.6/§10.7
//! classifiers. UEB Grade-2 is base behaviour, so the ~3.5 MB table is compiled
//! into every build.

use std::collections::HashMap;
use std::sync::LazyLock;

use super::{Phoneme, PronunciationProvider, parse_phoneme};

/// Raw CMUdict data: `word PH PH …` per line, optional trailing `# comment`,
/// `word(N)` for pronunciation variants.
static CMUDICT: &str = include_str!("../../../../resources/cmudict.dict");

/// word → its pronunciation strings, as zero-copy slices into [`CMUDICT`].
static INDEX: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    for line in CMUDICT.lines() {
        // Drop any trailing `# comment` and surrounding whitespace.
        let line = line.split('#').next().unwrap_or(line).trim();
        let Some((head, phones)) = line.split_once(' ') else {
            continue;
        };
        // Strip a `(2)` variant marker so all variants share one key.
        let word = head.split_once('(').map_or(head, |(w, _)| w);
        let phones = phones.trim();
        if !word.is_empty() && !phones.is_empty() {
            map.entry(word).or_default().push(phones);
        }
    }
    map
});

/// True iff `word` (lowercase) is recorded in CMUdict — a cheap membership test
/// (no phoneme parsing). Used as a free-word oracle for morphological boundary
/// detection: a `micro·film`-style combining-form compound is recognised only when
/// the component after the combining form is itself a recorded word (§10.11).
pub fn is_recorded_word(word: &str) -> bool {
    INDEX.contains_key(word)
}

/// Looks up ARPABET pronunciations from the embedded CMUdict.
pub struct CmuDictProvider;

impl CmuDictProvider {
    /// Construct the provider (the dictionary is parsed lazily on first use).
    pub fn new() -> Self {
        Self
    }
}

impl Default for CmuDictProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PronunciationProvider for CmuDictProvider {
    fn pronunciations(&self, word: &str) -> Vec<Vec<Phoneme>> {
        INDEX
            .get(word)
            .map(|prons| {
                prons
                    .iter()
                    .map(|s| s.split_whitespace().map(parse_phoneme).collect())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded dictionary returns the known ARPABET for common words, with
    /// stress markers intact (the classifier depends on first-vowel stress).
    #[test]
    fn looks_up_known_word_with_stress() {
        let prons = CmuDictProvider::new().pronunciations("become");
        assert!(!prons.is_empty(), "become should be in cmudict");
        let first = &prons[0];
        assert_eq!(first[0], parse_phoneme("B"));
        // become = B IH0 K AH1 M — first vowel is unstressed IH0.
        let first_vowel = first.iter().find(|p| p.is_vowel()).unwrap();
        assert_eq!(first_vowel.stress, Some(0));
    }

    #[test]
    fn unknown_word_yields_empty() {
        assert!(CmuDictProvider::new().pronunciations("zzqxwv").is_empty());
    }

    /// Variant headwords (`word(2)`) collapse onto the base key.
    #[test]
    fn variants_share_one_key() {
        // "aalborg" has a `(2)` variant in cmudict → at least two pronunciations.
        let prons = CmuDictProvider::new().pronunciations("aalborg");
        assert!(prons.len() >= 2, "expected variant pronunciations");
    }

    #[test]
    fn default_provider_matches_new_provider() {
        let default_provider = CmuDictProvider;

        assert_eq!(
            default_provider.pronunciations("become"),
            CmuDictProvider::new().pronunciations("become")
        );
    }
}
