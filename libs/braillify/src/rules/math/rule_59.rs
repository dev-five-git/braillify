//! 수학 제59항 — 선적분/경로적분 (∮).
//!
//! Contour integral ∮ (U+222E).

use crate::math_symbol_shortcut;

pub fn is_contour_integral(c: char) -> bool {
    c == '\u{222E}'
}

pub fn encode_contour_integral(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    if !is_contour_integral(c) {
        return Err(format!("not a contour integral: {c}"));
    }
    let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?;
    result.extend_from_slice(encoded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_contour_integral_symbol() {
        assert!(std::hint::black_box(is_contour_integral('\u{222E}')));
        assert!(!std::hint::black_box(is_contour_integral('∫')));
    }

    #[test]
    fn encodes_contour_integral_symbol() {
        let mut result = Vec::new();

        encode_contour_integral('\u{222E}', &mut result).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn rejects_non_contour_integral_symbol() {
        let mut result = Vec::new();

        assert!(encode_contour_integral('x', &mut result).is_err());
        assert!(result.is_empty());
    }
}
