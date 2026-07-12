//! §10.1 Alphabetic wordsigns.
//!
//! Per RUEB 2024 §10.1, twenty-three single letters represent a whole word when
//! that word stands alone (§2.6): `b but, c can, d do, e every, f from, g go,
//! h have, j just, k knowledge, l like, m more, n not, p people, q quite,
//! r rather, s so, t that, u us, v very, w will, x it, y you, z as`.
//!
//! The wordsign cell is simply the letter's own cell. Standing-alone is checked
//! by the document engine before calling [`wordsign`].

use phf::phf_map;

use crate::unicode::decode_unicode;

static WORDSIGNS: phf::Map<&'static str, u8> = phf_map! {
    "but"       => decode_unicode('⠃'),
    "can"       => decode_unicode('⠉'),
    "do"        => decode_unicode('⠙'),
    "every"     => decode_unicode('⠑'),
    "from"      => decode_unicode('⠋'),
    "go"        => decode_unicode('⠛'),
    "have"      => decode_unicode('⠓'),
    "just"      => decode_unicode('⠚'),
    "knowledge" => decode_unicode('⠅'),
    "like"      => decode_unicode('⠇'),
    "more"      => decode_unicode('⠍'),
    "not"       => decode_unicode('⠝'),
    "people"    => decode_unicode('⠏'),
    "quite"     => decode_unicode('⠟'),
    "rather"    => decode_unicode('⠗'),
    "so"        => decode_unicode('⠎'),
    "that"      => decode_unicode('⠞'),
    "us"        => decode_unicode('⠥'),
    "very"      => decode_unicode('⠧'),
    "will"      => decode_unicode('⠺'),
    "it"        => decode_unicode('⠭'),
    "you"       => decode_unicode('⠽'),
    "as"        => decode_unicode('⠵'),
};

/// The §10.1 wordsign cell for a lowercased whole word, if any.
pub fn wordsign(word: &str) -> Option<u8> {
    WORDSIGNS.get(word).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::but("but", Some(decode_unicode('⠃')))]
    #[case::knowledge("knowledge", Some(decode_unicode('⠅')))]
    #[case::us("us", Some(decode_unicode('⠥')))]
    #[case::not_a_wordsign("cat", None)]
    #[case::a_excluded("a", None)]
    fn looks_up_wordsigns(#[case] word: &str, #[case] expected: Option<u8>) {
        assert_eq!(wordsign(word), expected);
    }
}
