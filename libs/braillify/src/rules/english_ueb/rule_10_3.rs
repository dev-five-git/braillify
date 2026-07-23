//! §10.3 Strong contractions — `and`, `for`, `of`, `the`, `with`.
//!
//! Each is a one-cell contraction that applies wherever the letter sequence
//! occurs (as a wordsign standing alone, or as a groupsign inside a longer
//! word), per RUEB 2024 §10.3.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule, match_longest};
use crate::unicode::decode_unicode;

static STRONG: phf::Map<&'static str, u8> = phf_map! {
    "and"  => decode_unicode('⠯'),
    "for"  => decode_unicode('⠿'),
    "of"   => decode_unicode('⠷'),
    "the"  => decode_unicode('⠮'),
    "with" => decode_unicode('⠾'),
};

/// §10.3 strong contraction rule.
pub struct StrongContractionRule;

impl ContractionRule for StrongContractionRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        match_longest(word, pos, &STRONG, 50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::the("the", 0, Some((decode_unicode('⠮'), 3)))]
    #[case::and("and", 0, Some((decode_unicode('⠯'), 3)))]
    #[case::with("with", 0, Some((decode_unicode('⠾'), 4)))]
    #[case::no_match("cat", 0, None)]
    fn matches_strong_contractions(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(u8, usize)>,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = StrongContractionRule
            .try_match(&chars, pos)
            .map(|m| (m.cells[0], m.consumed));
        assert_eq!(got, expected);
    }
}
