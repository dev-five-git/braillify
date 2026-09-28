//! 과학 [부록] 5 — 안개·비·소나기 등 일기 현상 기호.
//!
//! 묵자에 글자로 적히는 기호만 받는다. 눈은 그림뿐이라 입력할 글자가 없다.

use crate::unicode::decode_unicode;

/// 글 전체가 일기 현상 기호 하나이면 그 점형.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    let braille = match text {
        "≡" => "⠶⠶",  // 안개
        "∇˙" => "⠶⠩", // 소나기
        "☈" => "⠶⠗",  // 뇌우
        "●" => "⠶⠤",  // 비
        _ => return None,
    };
    Some(braille.chars().map(decode_unicode).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::fog("≡", "⠶⠶")]
    #[case::shower("∇˙", "⠶⠩")]
    #[case::thunderstorm("☈", "⠶⠗")]
    #[case::rain("●", "⠶⠤")]
    fn writes_the_appendix_sign(#[case] text: &str, #[case] expected: &str) {
        let want: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(encode(text), Some(want));
    }

    #[rstest::rstest]
    #[case::nabla_without_dot("∇")]
    #[case::sign_in_a_sentence("● 비")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text), None);
    }
}
