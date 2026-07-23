//! 수학 제41항 — 수직 기호.
//!
//! 수직 기호 ⊥(U+22A5)는 내부표기 0'에 대응하며 점자 코드 [52, 4]로 인코딩한다.

pub fn is_perpendicular_symbol(c: char) -> bool {
    c == '\u{22A5}'
}

pub fn encode_perpendicular(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    if !is_perpendicular_symbol(c) {
        return Err("Not a perpendicular symbol".to_string());
    }
    result.push(52);
    result.push(4);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_perpendicular() {
        let mut result = Vec::new();
        let encoded = encode_perpendicular('⊥', &mut result);
        assert!(encoded.is_ok());
        assert_eq!(result, vec![52, 4]);
    }

    #[test]
    fn rejects_non_perpendicular() {
        let mut result = Vec::new();
        assert!(encode_perpendicular('a', &mut result).is_err());
        assert!(!is_perpendicular_symbol('a'));
    }
}
