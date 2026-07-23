use crate::unicode::decode_unicode;

use phf::phf_map;

pub static JUNGSEONG_MAP: phf::Map<char, &'static [u8]> = phf_map! {
    'ㅏ' => &[decode_unicode('⠣')],
    'ㅑ' => &[decode_unicode('⠜')],
    'ㅓ' => &[decode_unicode('⠎')],
    'ㅕ' => &[decode_unicode('⠱')],
    'ㅗ' => &[decode_unicode('⠥')],
    'ㅛ' => &[decode_unicode('⠬')],
    'ㅜ' => &[decode_unicode('⠍')],
    'ㅠ' => &[decode_unicode('⠩')],
    'ㅡ' => &[decode_unicode('⠪')],
    'ㅣ' => &[decode_unicode('⠕')],
    'ㅐ' => &[decode_unicode('⠗')],
    'ㅔ' => &[decode_unicode('⠝')],
    'ㅚ' => &[decode_unicode('⠽')],
    'ㅘ' => &[decode_unicode('⠧')],
    'ㅝ' => &[decode_unicode('⠏')],
    'ㅢ' => &[decode_unicode('⠺')],
    'ㅖ' => &[decode_unicode('⠌')],
    'ㅟ' => &[decode_unicode('⠍'), decode_unicode('⠗')],
    'ㅒ' => &[decode_unicode('⠜'), decode_unicode('⠗')],
    'ㅙ' => &[decode_unicode('⠧'), decode_unicode('⠗')],
    'ㅞ' => &[decode_unicode('⠏'), decode_unicode('⠗')],
};
pub fn encode_jungsong(text: char) -> Result<&'static [u8], String> {
    if let Some(code) = JUNGSEONG_MAP.get(&text) {
        Ok(code)
    } else {
        Err("Invalid Korean jungseong character".to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::unicode::decode_unicode;

    #[rstest::rstest]
    #[case::ah('ㅏ', '⠣')]
    #[case::ya('ㅑ', '⠜')]
    #[case::eo('ㅓ', '⠎')]
    #[case::yeo('ㅕ', '⠱')]
    #[case::o('ㅗ', '⠥')]
    #[case::yo('ㅛ', '⠬')]
    #[case::u('ㅜ', '⠍')]
    pub fn test_encode_jungsong(#[case] jung: char, #[case] expected: char) {
        assert_eq!(
            encode_jungsong(jung).unwrap(),
            vec![decode_unicode(expected)]
        );
    }
}
