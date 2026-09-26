//! 과학 제29항 — 비례 기호.
//!
//! ∝(U+221D)는 과학 제29항이 `+3`으로 정한 기호다. 수학 제5항의 비 기호
//! ∶(U+2236)와는 다른 조문이므로 단축표에서도 따로 귀속한다.

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
