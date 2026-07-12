//! §10.6 Lower groupsigns — `en` and `in` (the position-unrestricted subset).
//!
//! Per RUEB 2024 §10.6, the lower groupsigns `en` (⠢) and `in` (⠔) may be used
//! **anywhere** in a word (they are exempt from the lower-sign-rule placement
//! restrictions that govern `be con dis ea bb cc ff gg`). Those restricted
//! groupsigns require the lower-sign-rule primitive and are added in a later
//! phase; this file implements only the two unrestricted ones.
//!
//! Note: as a whole word, `in` is also the §10.5 lower wordsign — encoded with
//! the same cell (⠔), so handling it here as a groupsign yields identical output.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule, match_longest};
use crate::unicode::decode_unicode;

static LOWER_GROUPSIGNS: phf::Map<&'static str, u8> = phf_map! {
    "en" => decode_unicode('⠢'),
    "in" => decode_unicode('⠔'),
};

/// §10.6 lower groupsign rule (unrestricted subset: `en`, `in`).
pub struct LowerGroupsignRule;

impl ContractionRule for LowerGroupsignRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        match_longest(word, pos, &LOWER_GROUPSIGNS, 70)
    }
}

/// §10.6.5 middle lower groupsigns `ea bb cc ff gg`. One-cell signs usable only
/// when a letter immediately **precedes and follows** them within the word
/// (the structural lower-sign rule). The pure-English engine defers these
/// because the morphology exceptions (`hideaway`, `react`) need a dictionary, so
/// they are NOT registered in the default contraction engine; this helper lets
/// contexts that apply the structural rule directly reuse one definition
/// (e.g. digital-notation 제74항 URL/email runs).
pub(crate) static MIDDLE_LOWER_GROUPSIGNS: phf::Map<&'static str, u8> = phf_map! {
    "ea" => decode_unicode('⠂'),
    "bb" => decode_unicode('⠆'),
    "cc" => decode_unicode('⠒'),
    "ff" => decode_unicode('⠖'),
    "gg" => decode_unicode('⠶'),
};

/// Match a §10.6.5 middle lower groupsign at `pos`, or `None`. Requires an
/// alphabetic neighbour on both sides (so word-initial/final pairs spell out).
pub(crate) fn middle_lower_groupsign(word: &[char], pos: usize) -> Option<ContractionMatch> {
    // §10.6.5 "preceded and followed by a letter": a letter is any alphabetic
    // char, incl. an accented Latin letter (`abbé`, `réchauffé`) whose accent the
    // §4.2 path renders as indicator + base — still a letter neighbour, so the
    // doubled/`ea` pair beside it stays *medial* and keeps its groupsign.
    if pos == 0 || !word[pos - 1].is_alphabetic() {
        return None;
    }
    let key: String = word.get(pos..pos + 2)?.iter().collect();
    let &cell = MIDDLE_LOWER_GROUPSIGNS.get(key.as_str())?;
    if !word.get(pos + 2).is_some_and(|c| c.is_alphabetic()) {
        return None;
    }
    Some(ContractionMatch {
        cells: vec![cell],
        consumed: 2,
        priority: 70,
        protect_span: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::en("en", 0, Some((decode_unicode('⠢'), 2)))]
    #[case::in_word("find", 1, Some((decode_unicode('⠔'), 2)))]
    #[case::no_match("cat", 0, None)]
    fn matches_lower_groupsigns(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(u8, usize)>,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = LowerGroupsignRule
            .try_match(&chars, pos)
            .map(|m| (m.cells[0], m.consumed));
        assert_eq!(got, expected);
    }

    #[test]
    fn middle_lower_groupsign_rejects_runtime_word_final_pair() {
        let chars: Vec<char> = std::hint::black_box("sea").chars().collect();

        assert!(middle_lower_groupsign(&chars, std::hint::black_box(1)).is_none());
    }
}
