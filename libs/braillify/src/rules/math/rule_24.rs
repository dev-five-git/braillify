//! 수학 제24항 — 수열 표기 `{aₙ}`.
//!
//! 수열은 중괄호로 구간을 감싸고 항 기호(예: `aₙ`)를 내부에 배치한다.
//! 인코딩 파이프라인에서는 중괄호 경계와 첨자 정보를 분리해 후속 규칙에 전달한다.

use crate::math_symbol_shortcut;

pub fn is_sequence_brace(c: char) -> bool {
    matches!(c, '{' | '}')
}

pub fn encode_sequence_brace(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    math_symbol_shortcut::encode_char_math_symbol_shortcut(c)
        .map(|encoded| result.extend_from_slice(encoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_sequence_notation_char(c: char) -> bool {
        is_sequence_brace(c) || c == '\u{2099}'
    }

    #[test]
    fn detects_sequence_braces() {
        assert!(is_sequence_brace('{'));
        assert!(is_sequence_brace('}'));
    }

    #[test]
    fn detects_sequence_notation_chars() {
        assert!(is_sequence_notation_char('{'));
        assert!(is_sequence_notation_char('}'));
        assert!(is_sequence_notation_char('\u{2099}')); // subscript n
        assert!(!is_sequence_notation_char('a'));
    }
}
