//! 수학 제27항 — 약수/배수 관계 기호.
//!
//! `|`(나눔 관계)와 `∤`(나누지 않음)를 처리한다.
//! `|`는 rule_21(절댓값 막대)가 항상 먼저 처리하고, `∤`는 math_symbol_shortcut
//! 테이블을 통해 직접 인코딩된다.

pub fn is_divisibility_symbol(c: char) -> bool {
    matches!(c, '|' | '\u{2224}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_divisibility_symbols() {
        assert!(is_divisibility_symbol('|'));
        assert!(is_divisibility_symbol('\u{2224}'));
        assert!(!is_divisibility_symbol('a'));
    }
}
