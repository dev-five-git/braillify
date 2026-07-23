//! 수학 제5항 — 비례식 기호.
//!
//! 비례식에서 사용하는 ∝(U+221D) 기호를 단축표 인코딩으로 준비한다.

use crate::math_symbol_shortcut;

pub fn is_proportion_symbol(c: char) -> bool {
    c == '\u{221D}'
}

pub fn encode_proportion_symbol(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    math_symbol_shortcut::encode_char_math_symbol_shortcut(c)
        .map(|encoded| result.extend_from_slice(encoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_proportion_symbol() {
        assert!(is_proportion_symbol('\u{221D}'));
    }
}
