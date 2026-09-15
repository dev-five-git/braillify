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

/// 한글 중성 자모를 대응하는 하나 이상의 점자 셀로 변환한다.
/// 중성 매핑에 없는 자모나 문자는 오류로 반환한다.
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

    /// 중성 인코더가 자음처럼 `JUNGSEONG_MAP`에 없는 입력을 오류로 거부하는지 검증한다.
    #[test]
    fn rejects_invalid_jungseong() {
        assert_eq!(
            encode_jungsong('ㄱ').unwrap_err(),
            "Invalid Korean jungseong character"
        );
    }
}
