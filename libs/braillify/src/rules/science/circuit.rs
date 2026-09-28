//! 과학 [부록] 9 — 전기·전자 회로 및 소자 기호.
//!
//! 묵자가 그림이라 `[그림: 명칭]` 으로 받는다. 점형은 모두 ⠯ 과 ⠽ 사이에 소자의
//! 약호를 적은 것이다.

use crate::unicode::decode_unicode;

/// 글 전체가 회로 소자 그림 하나이면 그 점형.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    let abbreviation = match super::picture_name(text)? {
        "전지(직류 전원)" => "⠢⠠⠃⠔",
        "교류 전원" => "⠠⠁⠉",
        "전구" => "⠠⠇⠏",
        "열린 스위치" => "⠕⠠⠎",
        "닫힌 스위치" => "⠉⠠⠎",
        "전류계" => "⠠⠁",
        "전압계" => "⠠⠧",
        "직류 전류계" => "⠙⠠⠁",
        "직류 전압계" => "⠙⠠⠧",
        "교류 전류계" => "⠁⠠⠁",
        "교류 전압계" => "⠁⠠⠧",
        "검류계" => "⠠⠛⠁",
        "변압기" => "⠠⠞",
        "콘덴서" => "⠠⠉",
        "가변 콘덴서" => "⠧⠠⠉",
        "저항" => "⠠⠗",
        "가변 저항" => "⠧⠠⠗",
        "코일" => "⠠⠇",
        "가변 코일" => "⠧⠠⠇",
        "다이오드" => "⠠⠙",
        "발광 다이오드" => "⠠⠇⠑⠙",
        "N-P-N형 트랜지스터" => "⠠⠝⠏⠝",
        "P-N-P형 트랜지스터" => "⠠⠏⠝⠏",
        _ => return None,
    };
    let mut out = vec![decode_unicode('⠯')];
    out.extend(abbreviation.chars().map(decode_unicode));
    out.push(decode_unicode('⠽'));
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::battery("[그림: 전지(직류 전원)]", "⠯⠢⠠⠃⠔⠽")]
    #[case::open_switch("[그림: 열린 스위치]", "⠯⠕⠠⠎⠽")]
    #[case::direct_current_ammeter("[그림: 직류 전류계]", "⠯⠙⠠⠁⠽")]
    #[case::light_emitting_diode("[그림: 발광 다이오드]", "⠯⠠⠇⠑⠙⠽")]
    fn writes_the_element_sign(#[case] text: &str, #[case] expected: &str) {
        let want: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(encode(text), Some(want));
    }

    #[rstest::rstest]
    #[case::name_alone("전구")]
    #[case::unknown_element("[그림: 퓨즈]")]
    #[case::unclosed("[그림: 전구")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text), None);
    }
}
