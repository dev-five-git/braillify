//! 과학 [부록] 5~8 — 일기 현상·운량·전선 기호와 일기 관련 그 밖의 기호.
//!
//! 묵자가 그림뿐인 기호는 `[그림: 명칭]` 으로 받는다. 고기압 H·저기압 L 은 로마자
//! 대문자 그대로 적으므로 여기서 다루지 않는다.

use crate::unicode::decode_unicode;

/// 글 전체가 일기 기호 하나이면 그 점형.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    if let Some(name) = super::picture_name(text) {
        return picture(name);
    }
    let braille = match text {
        "≡" => "⠶⠶",  // 안개
        "∇˙" => "⠶⠩", // 소나기
        "☈" => "⠶⠗",  // 뇌우
        "●" => "⠶⠤",  // 비
        _ => return None,
    };
    Some(braille.chars().map(decode_unicode).collect())
}

/// 부록 6 — 운량 0~10 은 ⠶ 뒤에 운량을 수표 없이 적는다.
fn cloud_amount(tenths: &str) -> Option<Vec<u8>> {
    let valid = tenths
        .parse::<u8>()
        .is_ok_and(|amount| amount <= 10 && amount.to_string() == tenths);
    if !valid {
        return None;
    }
    let mut out = vec![decode_unicode('⠶')];
    for digit in tenths.chars() {
        out.push(crate::number::encode_number(digit).ok()?);
    }
    Some(out)
}

/// 그림뿐인 기호 — 눈(부록 5), 운량(부록 6), 전선(부록 7), 태풍·열대성 저기압(부록 8).
fn picture(name: &str) -> Option<Vec<u8>> {
    if let Some(tenths) = name.strip_prefix("운량 ") {
        return cloud_amount(tenths);
    }
    let braille = match name {
        "눈" => "⠶⠔",
        "한랭 전선" => "⠼⠤⠼⠤⠼⠤⠼⠤",
        "온난 전선" => "⠶⠤⠶⠤⠶⠤⠶⠤",
        "폐색 전선" => "⠶⠼⠶⠼⠶⠼⠶⠼",
        "정체 전선" => "⠛⠲⠛⠲⠛⠲⠛⠲",
        "태풍" => "⠶⠞",
        "열대성 저기압" => "⠶⠳",
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
    #[case::snow("[그림: 눈]", "⠶⠔")]
    #[case::clear_sky("[그림: 운량 0]", "⠶⠚")]
    #[case::overcast("[그림: 운량 10]", "⠶⠁⠚")]
    #[case::cold_front("[그림: 한랭 전선]", "⠼⠤⠼⠤⠼⠤⠼⠤")]
    #[case::tropical_low("[그림: 열대성 저기압]", "⠶⠳")]
    fn writes_the_appendix_sign(#[case] text: &str, #[case] expected: &str) {
        let want: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(encode(text), Some(want));
    }

    #[rstest::rstest]
    #[case::nabla_without_dot("∇")]
    #[case::sign_in_a_sentence("● 비")]
    #[case::cloud_amount_over_ten("[그림: 운량 11]")]
    #[case::cloud_amount_with_leading_zero("[그림: 운량 05]")]
    #[case::unknown_picture("[그림: 무지개]")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text), None);
    }
}
