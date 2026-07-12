//! 수학 제56항 — 적분 기호 (∫).
//!
//! Integral ∫ (U+222B).

use crate::math_symbol_shortcut;

pub fn is_integral_symbol(c: char) -> bool {
    matches!(c, '\u{222B}')
}

pub fn encode_integral_symbol(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?;
    result.extend_from_slice(encoded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_integral_symbol() {
        assert!(is_integral_symbol('\u{222B}'));
    }

    #[test]
    fn integral_symbol_paths_use_runtime_input() {
        let integral = std::hint::black_box('\u{222B}');
        let plain = std::hint::black_box('x');
        let mut result = Vec::new();

        assert!(is_integral_symbol(integral));
        assert!(!is_integral_symbol(plain));
        encode_integral_symbol(integral, &mut result).unwrap();
        assert!(!result.is_empty());
    }
}
