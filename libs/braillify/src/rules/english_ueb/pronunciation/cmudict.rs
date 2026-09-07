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

/// Parse one CMUdict line into `(word, phones)`: drop a trailing `# comment` and
/// surrounding whitespace, strip a `(2)` variant marker so all variants share one
/// key, and return `None` for a blank or comment-only line (no space-separated
/// head) or an entry missing a word or phones.
fn parse_cmudict_line(line: &str) -> Option<(&str, &str)> {
    let line = line.split('#').next().unwrap_or(line).trim();
    let (head, phones) = line.split_once(' ')?;
    let word = head.split_once('(').map_or(head, |(w, _)| w);
    let phones = phones.trim();
    (!word.is_empty() && !phones.is_empty()).then_some((word, phones))
}

/// word → its pronunciation strings, as zero-copy slices into [`CMUDICT`].
static INDEX: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    for line in CMUDICT.lines() {
        if let Some((word, phones)) = parse_cmudict_line(line) {
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

/// Whether CMUdict supplies sufficiently specific evidence that an uppercase
/// abbreviation is pronounced as letter names.
///
/// UEB §10.12.1 suppresses contractions when an abbreviation is pronounced as
/// letters.  Case-folded dictionary headwords alone cannot establish that
/// (`LED` otherwise collides with lexical *led*), so compare a pronunciation
/// variant against the concatenated ARPABET names of the printed capitals. An
/// For three or more letters, every recorded pronunciation must be the exact
/// letter-name sequence; this keeps word-pronounced acronyms such as `ASEAN`
/// on UEB's contract-when-uncertain fallback when the dictionary records both
/// readings.  For a two-capital abbreviation, an exact two-letter reading is
/// already specific evidence for the printed abbreviation even when the same
/// case-folded headword also has a one-word homograph (`AI` versus *ai*).
/// Unknown abbreviations and longer mixed-pronunciation entries return false.
pub fn has_unambiguous_letter_name_pronunciation(chars: &[char]) -> bool {
    if chars.len() < 2 || !chars.iter().all(|ch| ch.is_ascii_uppercase()) {
        return false;
    }

    const LETTER_PHONES: &[&[&str]; 26] = &[
        &["EY"],
        &["B", "IY"],
        &["S", "IY"],
        &["D", "IY"],
        &["IY"],
        &["EH", "F"],
        &["JH", "IY"],
        &["EY", "CH"],
        &["AY"],
        &["JH", "EY"],
        &["K", "EY"],
        &["EH", "L"],
        &["EH", "M"],
        &["EH", "N"],
        &["OW"],
        &["P", "IY"],
        &["K", "Y", "UW"],
        &["AA", "R"],
        &["EH", "S"],
        &["T", "IY"],
        &["Y", "UW"],
        &["V", "IY"],
        &["D", "AH", "B", "AH", "L", "Y", "UW"],
        &["EH", "K", "S"],
        &["W", "AY"],
        &["Z", "IY"],
    ];

    let expected: Vec<&str> = chars
        .iter()
        // The guard above proves every character is in `A..=Z`.
        .flat_map(|letter| LETTER_PHONES[*letter as usize - 'A' as usize])
        .copied()
        .collect();
    let key: String = chars.iter().map(|ch| ch.to_ascii_lowercase()).collect();
    INDEX.get(key.as_str()).is_some_and(|variants| {
        let is_letter_name_variant = |variant: &&str| {
            variant
                .split_whitespace()
                .map(|phone| phone.trim_end_matches(['0', '1', '2']))
                .eq(expected.iter().copied())
        };
        !variants.is_empty()
            && (variants.iter().all(is_letter_name_variant)
                || (chars.len() == 2 && variants.iter().any(is_letter_name_variant)))
    })
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
        // Exercise the `Default` impl (delegates to `new`) via the trait method so
        // clippy's unit-struct lint stays happy.
        let default_provider: CmuDictProvider = Default::default();

        assert_eq!(
            default_provider.pronunciations("become"),
            CmuDictProvider::new().pronunciations("become")
        );
    }

    #[test]
    fn parse_cmudict_line_handles_entries_variants_and_blanks() {
        // A normal entry parses to (word, phones).
        assert_eq!(
            parse_cmudict_line("become B IH0 K AH1 M"),
            Some(("become", "B IH0 K AH1 M"))
        );
        // A `(2)` variant collapses onto the base word.
        assert_eq!(
            parse_cmudict_line("become(2) B IY0 K AH1 M"),
            Some(("become", "B IY0 K AH1 M"))
        );
        // Blank, whitespace-only, and comment-only lines have no head → None.
        assert_eq!(parse_cmudict_line(""), None);
        assert_eq!(parse_cmudict_line("   "), None);
        assert_eq!(parse_cmudict_line("# comment only"), None);
        // A head with no phones → None.
        assert_eq!(parse_cmudict_line("word "), None);
    }

    #[rstest::rstest]
    #[case::ged_is_ambiguous("GED", false)]
    #[case::ai_two_letter_abbreviation("AI", true)]
    #[case::cc_two_letter_abbreviation("CC", true)]
    #[case::asean_mixed_pronunciation("ASEAN", false)]
    #[case::ofc_initialism("OFC", true)]
    #[case::lexical_led_only("LED", false)]
    #[case::unknown_mou("MOU", false)]
    #[case::lowercase_is_not_capitals("ged", false)]
    fn detects_unambiguous_letter_name_pronunciations(#[case] text: &str, #[case] expected: bool) {
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(has_unambiguous_letter_name_pronunciation(&chars), expected);
    }
}
