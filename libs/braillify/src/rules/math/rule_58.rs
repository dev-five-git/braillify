//! 수학 제58항 — 이중적분 (∬).
//!
//! Double integral ∬ (U+222C).

use crate::math_symbol_shortcut;

pub fn encode_double_integral(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?;
    result.extend_from_slice(encoded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_double_integral_symbol() {
        let mut result = Vec::new();

        encode_double_integral('\u{222C}', &mut result).expect("double integral should encode");

        assert!(!result.is_empty());
    }
}
