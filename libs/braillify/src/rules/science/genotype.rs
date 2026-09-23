//! 과학 제23항 — 유전자형.
//!
//! 대문자로 표시된 유전자는 1급 점자로 적고, 대문자가 2개 이상 붙어 나오면 대문자
//! 단어표 ⠠⠠ 를 쓴다(`RRyy` → ⠠⠠⠗⠗⠠⠄⠽⠽). 통일영어점자는 두 대문자 뒤에 소문자가
//! 이어지면 글자마다 대문자표를 붙이므로(`MHz` → ⠠⠍⠠⠓⠵) 유전자형은 따로 적는다.

use crate::english::encode_english;
use crate::unicode::decode_unicode;

/// 한 글자의 대립 유전자 두 개가 짝을 이룬 유전자형인가(`RRyy`, `AaBb`).
/// 대문자와 소문자가 모두 있어야 통일영어점자와 적는 법이 갈린다.
fn is_genotype(chars: &[char]) -> bool {
    chars.len() >= 4
        && chars.len().is_multiple_of(2)
        && chars.iter().all(char::is_ascii_alphabetic)
        && chars
            .chunks(2)
            .all(|pair| pair[0].eq_ignore_ascii_case(&pair[1]))
        && chars.iter().any(char::is_ascii_uppercase)
        && chars.iter().any(char::is_ascii_lowercase)
}

pub(crate) fn encode_genotype(text: &str) -> Option<Vec<u8>> {
    let chars: Vec<char> = text.chars().collect();
    if !is_genotype(&chars) {
        return None;
    }
    let capital = decode_unicode('⠠');
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let run = chars[at..]
            .iter()
            .take_while(|c| c.is_ascii_uppercase())
            .count();
        match run {
            0 => {}
            1 => out.push(capital),
            _ => out.extend([capital, capital]),
        }
        for letter in &chars[at..at + run] {
            out.push(encode_english(letter.to_ascii_lowercase()).ok()?);
        }
        at += run;
        let lower = chars[at..]
            .iter()
            .take_while(|c| c.is_ascii_lowercase())
            .count();
        if run >= 2 && lower > 0 {
            out.extend([capital, decode_unicode('⠄')]);
        }
        for letter in &chars[at..at + lower] {
            out.push(encode_english(*letter).ok()?);
        }
        at += lower;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(cells: &[u8]) -> String {
        cells
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect()
    }

    #[rstest::rstest]
    #[case::capital_pair_then_lower("RRyy", "⠠⠠⠗⠗⠠⠄⠽⠽")]
    #[case::heterozygous("AaBb", "⠠⠁⠁⠠⠃⠃")]
    #[case::lower_then_capitals("rrYY", "⠗⠗⠠⠠⠽⠽")]
    fn writes_genotypes(#[case] text: &str, #[case] expected: &str) {
        let cells = encode_genotype(text).expect("is a genotype");
        assert_eq!(braille(&cells), expected);
    }

    #[rstest::rstest]
    #[case::all_capitals("RRYY")]
    #[case::single_pair("Aa")]
    #[case::unit("MHz")]
    #[case::word("Ball")]
    #[case::odd_length("RRy")]
    fn leaves_other_words_alone(#[case] text: &str) {
        assert!(encode_genotype(text).is_none());
    }
}
