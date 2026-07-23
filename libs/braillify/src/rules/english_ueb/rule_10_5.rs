//! §10.5 Lower wordsigns — `be, enough, his, in, was, were`.
//!
//! Per RUEB 2024 §10.5, six lower-cell signs represent a whole word when it
//! stands alone (§2.6): `be ⠆, enough ⠢, his ⠦, in ⠔, was ⠴, were ⠶`.
//!
//! Standing-alone is checked by the document engine before calling [`wordsign`];
//! these wordsigns are therefore suppressed inside Korean text (한국 점자 제37항),
//! which the engine signals by passing `standing_alone = false`.

use crate::unicode::decode_unicode;

/// The §10.5 lower wordsign cell for a lowercased whole word, if any.
pub fn wordsign(word: &str) -> Option<u8> {
    match word {
        "be" => Some(decode_unicode('⠆')),
        "enough" => Some(decode_unicode('⠢')),
        "his" => Some(decode_unicode('⠦')),
        "in" => Some(decode_unicode('⠔')),
        "was" => Some(decode_unicode('⠴')),
        "were" => Some(decode_unicode('⠶')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::be("be", Some(decode_unicode('⠆')))]
    #[case::enough("enough", Some(decode_unicode('⠢')))]
    #[case::was("was", Some(decode_unicode('⠴')))]
    #[case::were("were", Some(decode_unicode('⠶')))]
    #[case::not_a_wordsign("cat", None)]
    fn looks_up_lower_wordsigns(#[case] word: &str, #[case] expected: Option<u8>) {
        assert_eq!(wordsign(word), expected);
    }
}
