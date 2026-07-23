//! Math expression token rule.
//!
//! Detects words that are math expressions (contain math operators,
//! function names, superscript/subscript chars, etc.) and encodes them
//! using the math braille engine instead of Korean character rules.

use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct MathExpressionTokenRule;

mod apply;
mod detect;
mod helpers;

impl TokenRule for MathExpressionTokenRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::FractionDetection
    }

    fn priority(&self) -> u16 {
        50 // Before InlineFractionRule (120) and LatexFractionRule
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        apply::run(tokens, index, state)
    }
}

#[cfg(test)]
mod tests {
    use super::detect::is_math_expression;
    use super::helpers::*;
    use super::*;
    use crate::rules::math::math_token_rule::MathContext;
    use crate::rules::token::WordMeta;
    use std::borrow::Cow;

    #[test]
    fn test_is_math_with_operator() {
        let chars: Vec<char> = "ax+b=0".chars().collect();
        assert!(is_math_expression(&chars, "ax+b=0"));
    }

    #[test]
    fn test_is_math_with_function() {
        let chars: Vec<char> = "sin3x".chars().collect();
        assert!(is_math_expression(&chars, "sin3x"));
    }

    #[test]
    fn test_is_math_with_standalone_function_name() {
        let chars: Vec<char> = "sin".chars().collect();
        assert!(is_math_expression(&chars, "sin"));
    }

    #[test]
    fn test_is_not_math_korean() {
        let chars: Vec<char> = "안녕".chars().collect();
        assert!(!is_math_expression(&chars, "안녕"));
    }

    #[test]
    fn test_is_not_math_plain_english() {
        let chars: Vec<char> = "hello".chars().collect();
        assert!(!is_math_expression(&chars, "hello"));
    }

    #[test]
    fn test_is_math_with_superscript() {
        let chars: Vec<char> = "x²".chars().collect();
        assert!(is_math_expression(&chars, "x²"));
    }

    #[test]
    fn test_is_math_digit_letter_with_operator() {
        // "3a+b" has digit-letter AND operator → math
        let chars: Vec<char> = "3a+b".chars().collect();
        assert!(is_math_expression(&chars, "3a+b"));
    }

    #[test]
    fn test_is_math_digit_then_letter() {
        // "3ab" starts with digit then letters → math multiplication
        let chars: Vec<char> = "3ab".chars().collect();
        assert!(is_math_expression(&chars, "3ab"));
    }

    #[test]
    fn test_is_not_math_letter_then_digit() {
        // "MP3" starts with letters then digit → NOT math (avoids false positive)
        let chars: Vec<char> = "MP3".chars().collect();
        assert!(!is_math_expression(&chars, "MP3"));
    }

    #[test]
    fn test_is_math_symbol_digit_combo() {
        let chars: Vec<char> = "≠0".chars().collect();
        assert!(is_math_expression(&chars, "≠0"));
    }

    #[test]
    fn test_decimal_starting_with_digit_is_not_math() {
        // PDF 제43항: 첫 글자가 숫자인 순수 소수는 한글 number rule로 처리.
        let chars: Vec<char> = "0.17".chars().collect();
        assert!(!is_math_expression(&chars, "0.17"));
        let chars: Vec<char> = "96.7".chars().collect();
        assert!(!is_math_expression(&chars, "96.7"));
    }

    #[test]
    fn test_decimal_starting_with_dot_is_math() {
        // ".47"처럼 점으로 시작하는 형태는 math expression.
        let chars: Vec<char> = ".47".chars().collect();
        assert!(is_math_expression(&chars, ".47"));
    }

    #[test]
    fn test_is_math_relation_shorthand() {
        let chars: Vec<char> = "aRb".chars().collect();
        assert!(is_math_expression(&chars, "aRb"));
    }

    /// detect.rs line 127 — `arc<trig>` recognised as math.
    #[rstest::rstest]
    #[case("arcsinx")]
    #[case("arccosy")]
    #[case("arctanz")]
    fn test_is_math_arctrig_prefix(#[case] input: &str) {
        let chars: Vec<char> = input.chars().collect();
        assert!(is_math_expression(&chars, input), "input={input}");
    }

    /// detect.rs lines 213-220 — letter-slash-letter fraction pattern.
    #[rstest::rstest]
    #[case::upper_force_normal("F/N", true)]
    #[case::lower_pair("a/b", true)]
    #[case::xy_pair("x/y", true)]
    #[case::pq_pair("P/Q", true)]
    #[case::trailing_slash_not_math("a/", false)]
    fn test_is_math_letter_slash_letter_fraction(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(is_math_expression(&chars, input), expected, "input={input}");
    }

    /// detect.rs line 226 — signed (− / -) numeric → math.
    #[rstest::rstest]
    #[case("-3")]
    #[case("-1.5")]
    #[case("−7")]
    #[case("-3x")]
    #[case("−5y")]
    fn test_is_math_signed_numeric(#[case] input: &str) {
        let chars: Vec<char> = input.chars().collect();
        assert!(is_math_expression(&chars, input), "input={input}");
    }

    #[test]
    fn test_is_math_negative_infinity() {
        let chars: Vec<char> = "-∞".chars().collect();
        assert!(is_math_expression(&chars, "-∞"));
    }

    #[test]
    fn test_is_math_unicode_fraction_char() {
        let chars: Vec<char> = "⅔".chars().collect();
        assert!(is_math_expression(&chars, "⅔"));
    }

    #[test]
    fn test_is_math_base_notation() {
        let chars: Vec<char> = "1010₂".chars().collect();
        assert!(is_math_expression(&chars, "1010₂"));
    }

    #[test]
    fn split_mixed_math_word_extracts_math_prefix() {
        let chars: Vec<char> = "tan의".chars().collect();
        let word = crate::rules::token::WordToken {
            text: Cow::Borrowed("tan의"),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };

        let replacement =
            split_mixed_math_word(&word, 2, MathContext::default()).expect("expected split");
        assert!(matches!(replacement[0], Token::PreEncoded(ref bytes) if bytes == &vec![0, 0]));
        assert!(matches!(replacement[1], Token::PreEncoded(_)));
        assert!(matches!(replacement[2], Token::PreEncoded(ref bytes) if bytes == &vec![0, 0]));
        assert!(matches!(&replacement[3], Token::Word(w) if w.text == "의"));
    }

    #[test]
    fn split_mixed_math_word_keeps_plain_mixed_english_korean() {
        let chars: Vec<char> = "ATM에서".chars().collect();
        let word = crate::rules::token::WordToken {
            text: Cow::Borrowed("ATM에서"),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };

        assert!(split_mixed_math_word(&word, 2, MathContext::default()).is_none());
    }

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    #[test]
    fn is_superscript_table() {
        // Standard superscript codepoints
        for c in ['\u{2070}', '\u{00B9}', '\u{00B2}', '\u{00B3}'] {
            assert!(is_superscript(c));
        }
        assert!(!is_superscript('1'));
        assert!(!is_superscript('a'));
    }

    #[test]
    fn is_subscript_table() {
        for c in ['\u{2080}', '\u{2081}', '\u{2082}'] {
            assert!(is_subscript(c));
        }
        assert!(!is_subscript('1'));
    }

    #[test]
    fn is_combining_math_mark_table() {
        assert!(is_combining_math_mark('\u{0304}'));
        assert!(is_combining_math_mark('\u{0305}'));
        assert!(!is_combining_math_mark('a'));
    }

    #[test]
    fn is_middle_dot_numeric_word_paths() {
        let chars: Vec<char> = "1·2".chars().collect();
        assert!(is_middle_dot_numeric_word(&chars));
        let chars: Vec<char> = "ab".chars().collect();
        assert!(!is_middle_dot_numeric_word(&chars));
        let chars: Vec<char> = "".chars().collect();
        assert!(!is_middle_dot_numeric_word(&chars));
    }

    #[test]
    fn is_korean_char_paths() {
        assert!(is_korean_char('가'));
        assert!(!is_korean_char('a'));
        assert!(!is_korean_char('1'));
    }

    #[test]
    fn is_korean_suffix_char_paths() {
        // Korean syllable should be true for some suffix-like chars
        let _ = is_korean_suffix_char('가');
        let _ = is_korean_suffix_char('a');
    }

    #[test]
    fn rule_44_space_before_korean_paths() {
        // Just exercise the function with various inputs
        let _ = rule_44_requires_space_before_korean("abc가");
        let _ = rule_44_requires_space_before_korean("123");
        let _ = rule_44_requires_space_before_korean("");
    }

    #[test]
    fn is_strong_mixed_math_candidate_paths() {
        let chars: Vec<char> = "a+b".chars().collect();
        let _ = is_strong_mixed_math_candidate(&chars, "a+b");
        let chars: Vec<char> = "".chars().collect();
        let _ = is_strong_mixed_math_candidate(&chars, "");
    }

    #[test]
    fn is_rule_68_compact_notation_paths() {
        let chars: Vec<char> = "A⁺".chars().collect();
        let _ = is_rule_68_compact_notation(&chars);
        let chars: Vec<char> = "hello".chars().collect();
        assert!(!is_rule_68_compact_notation(&chars));
    }

    /// Comprehensive sweep through math expression detection via main pipeline.
    #[test]
    fn math_expression_diverse_inputs() {
        let inputs: &[&str] = &[
            "ax+b=0",
            "1+2=3",
            "x²",
            "y₂",
            "x²+y²=r²",
            "1·2",
            "3·4",
            "$x \\bar{a}$",
            "$\\overline{AB}$",
            "ATM에서",
            "1+1=2가",
            "f'(x)",
            "f''(x)",
            "x^2_n",
            "a^2 b^2",
        ];
        for input in inputs {
            let _ = enc(input);
        }
    }

    #[test]
    fn build_word_token_basic() {
        let t = build_word_token("hello".to_string());
        assert!(matches!(t, Token::Word(_)));
    }

    #[test]
    fn try_encode_math_slice_paths() {
        let chars: Vec<char> = "1+2".chars().collect();
        let _ = try_encode_math_slice(&chars, MathContext::default());
        let chars: Vec<char> = "abc".chars().collect();
        // Non-math should usually return None
        let _ = try_encode_math_slice(&chars, MathContext::default());
    }

    #[test]
    fn try_encode_mixed_math_slice_paths() {
        let chars: Vec<char> = "1+2가".chars().collect();
        let _ = try_encode_mixed_math_slice(&chars, MathContext::default());
    }

    #[test]
    fn try_encode_mixed_math_prefix_paths() {
        let prefix: Vec<char> = "1+2".chars().collect();
        let suffix: Vec<char> = "가".chars().collect();
        let _ = try_encode_mixed_math_prefix(&prefix, &suffix, MathContext::default());
    }
}
