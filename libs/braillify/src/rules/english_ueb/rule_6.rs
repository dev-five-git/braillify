//! §6 Numeric Mode — numeric indicator `⠼` followed by digits.
//!
//! Per RUEB 2024 §6.1, a number is the numeric indicator `⠼` (dots 3-4-5-6)
//! once, then each digit rendered with the letters a–j: `1→a 2→b … 9→i 0→j`.
//! Numeric mode runs until a space or a non-digit symbol ends it (§6.3).
//!
//! The `⠰` grade-1 indicator needed when a lowercase letter a–j directly
//! follows a number (§6.5) is applied by the document engine, which has the
//! cross-token context.

use crate::english::encode_english;
use crate::unicode::decode_unicode;

/// `⠼` numeric indicator (dots 3-4-5-6).
pub const NUMERIC_INDICATOR: u8 = decode_unicode('⠼');

/// Map a single ASCII digit to its braille cell (the a–j letter cells).
pub fn digit_cell(d: char) -> Option<u8> {
    let letter = match d {
        '1'..='9' => char::from(b'a' + (d as u8 - b'1')),
        '0' => 'j',
        _ => return None,
    };
    encode_english(letter).ok()
}

/// Encode a run of ASCII digits as a UEB number: `⠼` + digit cells.
pub fn encode_number(digits: &[char]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(digits.len() + 1);
    out.push(NUMERIC_INDICATOR);
    for d in digits {
        out.push(digit_cell(*d)?);
    }
    Some(out)
}

/// §6: a precomposed Unicode vulgar fraction (`½`, `⅜`, …) is written
/// numerator-first as `⠼` + numerator digits + `⠌` (fraction line, dots 3-4) +
/// denominator digits — one numeric pass, no second `⠼` before the denominator
/// (`⅜` → `⠼⠉⠌⠓`, `½` → `⠼⠁⠌⠃`, `¾` → `⠼⠉⠌⠙`). This is the UEB §6 order; the
/// Korean/math path writes the same code points denominator-first (`¾` →
/// `⠼⠙⠌⠼⠉`), so only the English engine routes here. The numerator/denominator
/// are read from the glyph's own Unicode NFKD decomposition via the shared
/// [`crate::fraction::parse_unicode_fraction`] (`½` → `("1", "2")`) — the same
/// general decomposer the Korean/math path uses, so every vulgar-fraction code
/// point is covered without a per-glyph table.
pub fn encode_vulgar_fraction(c: char) -> Option<Vec<u8>> {
    let (num, den) = crate::fraction::parse_unicode_fraction(c)?;
    let mut out = Vec::with_capacity(num.len() + den.len() + 2);
    out.push(NUMERIC_INDICATOR);
    for d in num.chars() {
        out.push(digit_cell(d)?);
    }
    out.push(decode_unicode('⠌'));
    for d in den.chars() {
        out.push(digit_cell(d)?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::ninety_five("95", vec![NUMERIC_INDICATOR, decode_unicode('⠊'), decode_unicode('⠑')])]
    #[case::one_zero("10", vec![NUMERIC_INDICATOR, decode_unicode('⠁'), decode_unicode('⠚')])]
    #[case::all_digits("1234567890", vec![
        NUMERIC_INDICATOR,
        decode_unicode('⠁'), decode_unicode('⠃'), decode_unicode('⠉'), decode_unicode('⠙'),
        decode_unicode('⠑'), decode_unicode('⠋'), decode_unicode('⠛'), decode_unicode('⠓'),
        decode_unicode('⠊'), decode_unicode('⠚'),
    ])]
    fn encodes_numbers(#[case] digits: &str, #[case] expected: Vec<u8>) {
        let chars: Vec<char> = digits.chars().collect();
        assert_eq!(encode_number(&chars), Some(expected));
    }

    /// §6 vulgar fractions: numerator-first `⠼num⠌den`, denominator without `⠼`.
    #[rstest::rstest]
    #[case::half('\u{00BD}', vec![NUMERIC_INDICATOR, decode_unicode('⠁'), decode_unicode('⠌'), decode_unicode('⠃')])]
    #[case::three_eighths('\u{215C}', vec![NUMERIC_INDICATOR, decode_unicode('⠉'), decode_unicode('⠌'), decode_unicode('⠓')])]
    #[case::three_quarters('\u{00BE}', vec![NUMERIC_INDICATOR, decode_unicode('⠉'), decode_unicode('⠌'), decode_unicode('⠙')])]
    #[case::one_quarter('\u{00BC}', vec![NUMERIC_INDICATOR, decode_unicode('⠁'), decode_unicode('⠌'), decode_unicode('⠙')])]
    fn encodes_vulgar_fractions(#[case] c: char, #[case] expected: Vec<u8>) {
        assert_eq!(encode_vulgar_fraction(c), Some(expected));
    }

    #[test]
    fn non_fraction_char_is_none() {
        assert_eq!(encode_vulgar_fraction('x'), None);
    }
}
