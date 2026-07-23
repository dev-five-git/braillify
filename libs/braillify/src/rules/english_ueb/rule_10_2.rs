//! §10.2 Strong wordsigns.
//!
//! Per RUEB 2024 §10.2, six strong-groupsign cells also represent a whole word
//! when standing alone (§2.6): `ch child, sh shall, th this, wh which, ou out,
//! st still`. The cell is the same single cell as the §10.4 groupsign.
//!
//! Standing-alone is checked by the document engine before calling [`wordsign`].

use phf::phf_map;

use crate::unicode::decode_unicode;

static WORDSIGNS: phf::Map<&'static str, u8> = phf_map! {
    "child" => decode_unicode('⠡'), // ch
    "shall" => decode_unicode('⠩'), // sh
    "this"  => decode_unicode('⠹'), // th
    "which" => decode_unicode('⠱'), // wh
    "out"   => decode_unicode('⠳'), // ou
    "still" => decode_unicode('⠌'), // st
};

/// The §10.2 strong wordsign cell for a lowercased whole word, if any.
pub fn wordsign(word: &str) -> Option<u8> {
    WORDSIGNS.get(word).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::child("child", Some(decode_unicode('⠡')))]
    #[case::this("this", Some(decode_unicode('⠹')))]
    #[case::still("still", Some(decode_unicode('⠌')))]
    #[case::not_a_wordsign("cat", None)]
    fn looks_up_strong_wordsigns(#[case] word: &str, #[case] expected: Option<u8>) {
        assert_eq!(wordsign(word), expected);
    }
}
