//! Math function name lookup table.
//!
//! Maps function names (sin, cos, tan, log, etc.) to their
//! braille byte encodings per the 2024 Korean Math Braille Standard.

use phf::phf_map;

use crate::unicode::decode_unicode;

/// Function name → braille encoding map.
///
/// Encodings follow the 2024 Korean Math Braille function-name notation:
/// - sin → 6s → ⠖⠎
/// - cos → 6c → ⠖⠉
/// - tan → 6t → ⠖⠞
/// - csc → 6< → ⠖⠣
/// - sec → 6- → ⠖⠤
/// - cot → 6\\ → ⠖⠳
/// - sinh → 6sh → ⠖⠎⠓
/// - cosh → 6ch → ⠖⠉⠓
/// - tanh → 6th → ⠖⠞⠓
/// - log → _ (context-dependent, uses ; or , prefixes)
/// - ln → special encoding
/// - lim → lim (letters, context-dependent subscript)
/// - arc → arc (prefix for inverse trig)
static FUNCTION_MAP: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "sin" => &[decode_unicode('⠖'), decode_unicode('⠎')],     // 6s
    "cos" => &[decode_unicode('⠖'), decode_unicode('⠉')],     // 6c
    "tan" => &[decode_unicode('⠖'), decode_unicode('⠞')],     // 6t
    "csc" => &[decode_unicode('⠖'), decode_unicode('⠣')],     // 6<
    "sec" => &[decode_unicode('⠖'), decode_unicode('⠤')],     // 6-
    "cot" => &[decode_unicode('⠖'), decode_unicode('⠳')],     // 6\\
    "sinh" => &[decode_unicode('⠖'), decode_unicode('⠎'), decode_unicode('⠓')], // 6sh
    "cosh" => &[decode_unicode('⠖'), decode_unicode('⠉'), decode_unicode('⠓')], // 6ch
    "tanh" => &[decode_unicode('⠖'), decode_unicode('⠞'), decode_unicode('⠓')], // 6th
    "arcsin" => &[decode_unicode('⠁'), decode_unicode('⠗'), decode_unicode('⠉'), decode_unicode('⠖'), decode_unicode('⠎')], // arc6s
    "arccos" => &[decode_unicode('⠁'), decode_unicode('⠗'), decode_unicode('⠉'), decode_unicode('⠖'), decode_unicode('⠉')], // arc6c
    "arctan" => &[decode_unicode('⠁'), decode_unicode('⠗'), decode_unicode('⠉'), decode_unicode('⠖'), decode_unicode('⠞')], // arc6t
    "cosec" => &[decode_unicode('⠖'), decode_unicode('⠣')], // 6< (alias for csc)
    "log" => &[], // Special-case encoded in math::encoder
    "lim" => &[],
};

/// Known function names in order of length (longest first for greedy matching).
const FUNCTION_NAMES: &[&str] = &[
    "arcsin", "arccos", "arctan", // 6-letter arc functions
    "cosec",  // 5-letter alias
    "sinh", "cosh", "tanh", // 4-letter
    "lim", "log", // 3-letter (special-case)
    "sin", "cos", "tan", "csc", "sec", "cot", // 3-letter
];

/// Check if the text starts with a known function name.
/// Returns the function name and its braille encoding if matched.
/// Uses longest-match-first strategy.
pub fn match_function_prefix(text: &str) -> Option<(&'static str, &'static [u8])> {
    for &name in FUNCTION_NAMES {
        if text.starts_with(name)
            && let Some(encoding) = FUNCTION_MAP.get(name)
        {
            return Some((name, encoding));
        }
    }
    None
}

/// Get the encoding for a known function name.
pub fn encode_function(name: &str) -> Option<&'static [u8]> {
    FUNCTION_MAP.get(name).copied()
}

/// Check if a string starts with a known function name.
pub fn starts_with_function(text: &str) -> bool {
    match_function_prefix(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin() {
        let result = encode_function("sin");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[22, 14]); // 6s = ⠖⠎
    }

    #[test]
    fn test_cos() {
        let result = encode_function("cos");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[22, 9]); // 6c = ⠖⠉
    }

    #[test]
    fn test_tan() {
        let result = encode_function("tan");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[22, 30]); // 6t = ⠖⠞
    }

    #[test]
    fn test_sinh() {
        let result = encode_function("sinh");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[22, 14, 19]); // 6sh = ⠖⠎⠓
    }

    #[test]
    fn test_match_prefix_sin3x() {
        let result = match_function_prefix("sin3x");
        assert!(result.is_some());
        let (name, _) = result.unwrap();
        assert_eq!(name, "sin");
    }

    #[test]
    fn test_match_prefix_sinh() {
        // sinh should match before sin
        let result = match_function_prefix("sinhx");
        assert!(result.is_some());
        let (name, _) = result.unwrap();
        assert_eq!(name, "sinh");
    }

    #[test]
    fn test_unknown_function() {
        assert!(encode_function("xyz").is_none());
    }
}
