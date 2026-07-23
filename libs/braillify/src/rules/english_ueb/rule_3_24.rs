//! §3.24 superscript and subscript indicators.
//!
//! UEB §3.24: a superscript item is preceded by the indicator `⠔` (dots 3-5) and
//! a subscript item by `⠢` (dots 2-6). In a grade-2 (letter) context the indicator
//! itself needs a grade-1 indicator `⠰` first (`clarion¹` → `⠰⠔⠼⠁`, `B₁₂` →
//! `⠰⠢⠼⠁⠃`); after a number the surrounding numeric grade-1 mode already covers it
//! (`1682.³` → `⠔⠼⠉`, no `⠰`).
//!
//! Only **digit** super/subscripts that *follow a base* are owned here: a math
//! expression carries the same Unicode code points but a different (제18/19항)
//! point shape, so single-token uses (`c²`, `x₂`) are kept on the math path by
//! `is_math_owned`, and a *leading* super/subscript (`¹ clarion`, combinatorics
//! `₇𝑃₂`) makes the whole UEB attempt fail closed.

use crate::unicode::decode_unicode;

/// Whether a super/subscript sits above (`⠔`) or below (`⠢`) the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    /// Superscript — §3.24 indicator `⠔` (dots 3-5).
    Superscript,
    /// Subscript — §3.24 indicator `⠢` (dots 2-6).
    Subscript,
}

impl ScriptKind {
    /// The §3.24 level-indicator cell for this kind.
    pub fn indicator(self) -> u8 {
        match self {
            ScriptKind::Superscript => decode_unicode('⠔'),
            ScriptKind::Subscript => decode_unicode('⠢'),
        }
    }
}

/// Whether `c` is any Unicode super/subscript character — digit, letter, or sign.
///
/// Used to keep a single-token math expression carrying these code points on the
/// math path (`is_math_owned`).
pub fn is_script_char(c: char) -> bool {
    matches!(c, '\u{00B2}' | '\u{00B3}' | '\u{00B9}')      // ² ³ ¹
        || ('\u{2070}'..='\u{209F}').contains(&c)          // superscripts + subscripts
        || ('\u{1D2C}'..='\u{1D6A}').contains(&c)          // modifier-letter superscripts (ᵐ …)
        || ('\u{1D9B}'..='\u{1DBF}').contains(&c) // more modifier letters (ᶜ …)
}

/// A super/subscript **digit** → its kind and the plain ASCII digit, else `None`.
pub fn script_digit(c: char) -> Option<(ScriptKind, char)> {
    use ScriptKind::{Subscript, Superscript};
    Some(match c {
        '\u{2070}' => (Superscript, '0'),
        '\u{00B9}' => (Superscript, '1'),
        '\u{00B2}' => (Superscript, '2'),
        '\u{00B3}' => (Superscript, '3'),
        '\u{2074}' => (Superscript, '4'),
        '\u{2075}' => (Superscript, '5'),
        '\u{2076}' => (Superscript, '6'),
        '\u{2077}' => (Superscript, '7'),
        '\u{2078}' => (Superscript, '8'),
        '\u{2079}' => (Superscript, '9'),
        '\u{2080}' => (Subscript, '0'),
        '\u{2081}' => (Subscript, '1'),
        '\u{2082}' => (Subscript, '2'),
        '\u{2083}' => (Subscript, '3'),
        '\u{2084}' => (Subscript, '4'),
        '\u{2085}' => (Subscript, '5'),
        '\u{2086}' => (Subscript, '6'),
        '\u{2087}' => (Subscript, '7'),
        '\u{2088}' => (Subscript, '8'),
        '\u{2089}' => (Subscript, '9'),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::super_2('\u{00B2}', ScriptKind::Superscript, '2')]
    #[case::super_3('\u{00B3}', ScriptKind::Superscript, '3')]
    #[case::super_1('\u{00B9}', ScriptKind::Superscript, '1')]
    #[case::super_0('\u{2070}', ScriptKind::Superscript, '0')]
    #[case::super_4('\u{2074}', ScriptKind::Superscript, '4')]
    #[case::super_5('\u{2075}', ScriptKind::Superscript, '5')]
    #[case::super_6('\u{2076}', ScriptKind::Superscript, '6')]
    #[case::super_7('\u{2077}', ScriptKind::Superscript, '7')]
    #[case::super_8('\u{2078}', ScriptKind::Superscript, '8')]
    #[case::super_9('\u{2079}', ScriptKind::Superscript, '9')]
    #[case::sub_0('\u{2080}', ScriptKind::Subscript, '0')]
    #[case::sub_1('\u{2081}', ScriptKind::Subscript, '1')]
    #[case::sub_2('\u{2082}', ScriptKind::Subscript, '2')]
    #[case::sub_3('\u{2083}', ScriptKind::Subscript, '3')]
    #[case::sub_4('\u{2084}', ScriptKind::Subscript, '4')]
    #[case::sub_5('\u{2085}', ScriptKind::Subscript, '5')]
    #[case::sub_6('\u{2086}', ScriptKind::Subscript, '6')]
    #[case::sub_7('\u{2087}', ScriptKind::Subscript, '7')]
    #[case::sub_8('\u{2088}', ScriptKind::Subscript, '8')]
    #[case::sub_9('\u{2089}', ScriptKind::Subscript, '9')]
    fn script_digits_decode(#[case] c: char, #[case] kind: ScriptKind, #[case] digit: char) {
        assert_eq!(script_digit(c), Some((kind, digit)));
    }

    #[rstest::rstest]
    #[case::super_letter_m('\u{1D50}')] // ᵐ
    #[case::super_letter_c('\u{1D9C}')] // ᶜ
    #[case::super_minus('\u{207B}')] // ⁻
    #[case::sub_minus('\u{208B}')] // ₋
    #[case::sub_letter_n('\u{2099}')] // ₙ
    fn non_digit_script_chars_have_no_digit(#[case] c: char) {
        assert!(is_script_char(c), "{c:?} should be a script char");
        assert_eq!(script_digit(c), None);
    }

    #[rstest::rstest]
    #[case::plain_digit('2')]
    #[case::letter('m')]
    #[case::space(' ')]
    fn plain_chars_are_not_script(#[case] c: char) {
        assert!(!is_script_char(c));
        assert_eq!(script_digit(c), None);
    }

    #[test]
    fn indicators_are_distinct() {
        assert_eq!(ScriptKind::Superscript.indicator(), decode_unicode('⠔'));
        assert_eq!(ScriptKind::Subscript.indicator(), decode_unicode('⠢'));
    }
}
