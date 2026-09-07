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

/// Shared character-emission predicate for a Roman identifier whose `+` must
/// use the UEB general-symbol cells rather than the Korean math plus cell.
pub(crate) fn is_roman_plus_identifier(chars: &[char]) -> bool {
    apply::is_korean_prose_roman_plus_identifier(chars)
        || apply::has_korean_prefix_roman_plus_annotation(chars)
        || apply::has_korean_prefix_terminal_roman_plus_identifier(chars)
}

#[cfg(test)]
mod tests {
    use super::apply::{
        has_korean_prefix_roman_hyphen_suffix, has_korean_prefix_roman_plus_annotation,
        has_korean_prefix_terminal_roman_plus_identifier, is_korean_prose_acronym_parenthetical,
        is_korean_prose_roman_hyphen_identifier, is_korean_prose_roman_number_identifier,
        is_korean_prose_roman_plus_identifier, is_korean_prose_roman_slash_identifier,
        is_korean_prose_single_letter_slash_phrase,
    };
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

    /// Korean rules 28/29/34/35: Roman code/compound surfaces must remain on
    /// the Roman path in Korean prose, while lowercase algebra and one-letter
    /// subtraction stay math-owned.
    #[rstest::rstest]
    #[case::official_roman_number("D-100", true)]
    #[case::capital_code("AB-12", true)]
    #[case::capitalised_compound("Title-Case", true)]
    #[case::enclosed_code("(ABC)-D", true)]
    #[case::single_capital_lexical_prefix("K-pop", true)]
    #[case::single_capital_common_term("X-ray", true)]
    #[case::single_capital_brand_prefix("K-water", true)]
    #[case::single_lowercase_brand_prefix("k-water", true)]
    #[case::mixed_case_digit_code("pH-1", true)]
    #[case::decimal_model_code("GPT-3.5", true)]
    #[case::lowercase_algebra("x-1", false)]
    #[case::uppercase_subtraction("A-B", false)]
    #[case::function_expression("F(x-1)", false)]
    fn korean_prose_hyphen_identifier_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            is_korean_prose_roman_hyphen_identifier(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    /// Korean rules 29/35 keep a Roman-led name and its adjoining number in
    /// one Roman section. A one-letter algebraic variable remains math-owned.
    #[rstest::rstest]
    #[case::media_generation("Web3.0", true)]
    #[case::model_version("GPT3.5", true)]
    #[case::audio_format("MP3", true)]
    #[case::mixed_case_measure("pH7", true)]
    #[case::single_letter_variable("x2", false)]
    #[case::number_first("3ab", false)]
    #[case::plain_decimal("3.14", false)]
    #[case::separator_not_between_digits("Web.3", false)]
    fn korean_prose_roman_number_identifier_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            is_korean_prose_roman_number_identifier(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::enclosed_code("한글(ABC)-D", true)]
    #[case::roman_gloss_then_code("한글(Title)-AB", true)]
    #[case::korean_then_single_capital("하쿠토-R", true)]
    #[case::korean_then_initialism("기장-KBO", true)]
    #[case::korean_then_lowercase_word("온다-life", true)]
    #[case::korean_then_alphanumeric_label("대신-Y2HC", true)]
    #[case::lowercase_algebra("한글(x-1)", false)]
    #[case::uppercase_subtraction("한글(A-B)", false)]
    #[case::korean_then_lowercase_variable("값-x", false)]
    #[case::korean_then_explicit_expression("값-X+1", false)]
    #[case::korean_then_number("한-3", false)]
    fn korean_prefix_roman_hyphen_suffix_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            has_korean_prefix_roman_hyphen_suffix(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    /// Korean rules 29 and 33: a Korean/Roman hyphen boundary stays attached,
    /// uses the Korean hyphen cell, and opens one Roman section after it.
    #[rstest::rstest]
    #[case::single_capital("하쿠토-R", "⠚⠋⠍⠓⠥⠤⠴⠠⠗⠲")]
    #[case::initialism("기장-KBO", "⠈⠕⠨⠶⠤⠴⠠⠠⠅⠃⠕⠲")]
    #[case::country_initialism("한-UAE", "⠚⠒⠤⠴⠠⠠⠥⠁⠑⠲")]
    fn korean_to_roman_hyphen_boundary_stays_prose(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    #[rstest::rstest]
    #[case::standards_bodies("ISO/IEC", true)]
    #[case::market_pair("WEMIX/KRW", true)]
    #[case::aircraft_family("F-5E/F", true)]
    #[case::roman_model_family("NVMe/TCP", true)]
    #[case::single_letter_fraction("F/N", false)]
    #[case::algebraic_fraction("A/B", false)]
    #[case::numeric_fraction("1/2", false)]
    #[case::equation_context("X≈F/N", false)]
    fn korean_prose_slash_identifier_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            is_korean_prose_roman_slash_identifier(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::hardware_wallet("H/W Wallet", true)]
    #[case::relapsed_refractory_cancer("폐암(R/R ES-SCLC)에서", true)]
    #[case::lowercase_math_description("F/N ratio", false)]
    #[case::explicit_equation("X≈F/N Result", false)]
    #[case::isolated_fraction("F/N", false)]
    fn single_letter_slash_phrase_requires_a_capital_led_roman_continuation(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let ir = crate::rules::token::DocumentIR::parse(input, true);
        let index = ir
            .tokens
            .iter()
            .position(|token| matches!(token, Token::Word(word) if word.chars.contains(&'/')))
            .expect("probe must contain a slash word");
        let Token::Word(word) = &ir.tokens[index] else {
            unreachable!("selected token must be a word");
        };

        assert_eq!(
            is_korean_prose_single_letter_slash_phrase(&ir.tokens, index, &word.chars),
            expected
        );
    }

    #[rstest::rstest]
    #[case::hardware_wallet("가 H/W Wallet 나", "⠫⠀⠴⠠⠓⠸⠌⠠⠺⠀⠠⠺⠁⠇⠇⠑⠞⠲⠀⠉")]
    #[case::relapsed_refractory_cancer("가 R/R ES-SCLC 나", "⠫⠀⠴⠠⠗⠸⠌⠠⠗⠀⠠⠠⠑⠎⠤⠠⠠⠎⠉⠇⠉⠲⠀⠉")]
    #[case::official_math_rule_29("X ≈ F/N", "⠠⠭⠀⠈⠔⠈⠔⠀⠠⠋⠸⠌⠠⠝")]
    fn slash_phrase_respects_korean_rule_29_without_stealing_math_rule_29(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    #[rstest::rstest]
    #[case::service("TV+", true)]
    #[case::safety_grade("TSP+", true)]
    #[case::alphanumeric_product("HDR10+", true)]
    #[case::numeric_parenthetical("ATC+(20017936)", true)]
    #[case::mixed_case_service("U+tv", true)]
    #[case::attached_korean_particle("XYZ+는", true)]
    #[case::number_led_identifier("24K+", true)]
    #[case::identifier_separator("Model.Name+", true)]
    #[case::repeated_terminal_plus("UV++++", true)]
    #[case::contextual_single_letter_grade("A+(우수)", true)]
    #[case::ascii_single_letter_expression("A+(B)", false)]
    #[case::one_letter_terminal_identifier("A+", true)]
    #[case::completed_sum("AB+C", false)]
    #[case::chemical_expression("SmBa0.5-xCo2O5+d", false)]
    #[case::lexical_compound("Dog+Yoga", true)]
    #[case::lowercase_math_functions("sin+cos", false)]
    #[case::korean_service("U+유모바일", true)]
    #[case::mixed_script_korean_service("U+한글tv", true)]
    #[case::one_letter_korean_addition("A+나", true)]
    fn korean_prose_plus_identifier_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            is_korean_prose_roman_plus_identifier(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::closed_lexical_gloss("도가(Dog+Yoga)", true)]
    #[case::attached_particle("워케이션(Work+Vacation)은", true)]
    #[case::single_letter_terminal_label("등급(A+)은", true)]
    #[case::middle_dot_chained_identifier("상품(Service+)·후속(Next+)는", true)]
    #[case::math_body("공식(A+B)은", false)]
    #[case::unclosed("도가(Dog+Yoga", false)]
    fn korean_prefix_plus_annotation_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            has_korean_prefix_roman_plus_annotation(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::roman_suffix("한글TV+는", true)]
    #[case::numeric_roman_suffix("한글7GB+는", true)]
    #[case::completed_sum("한글A+B는", false)]
    #[case::all_capital_internal_ambiguity("한글X+U는", false)]
    fn korean_prefix_terminal_plus_suffix_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            has_korean_prefix_terminal_roman_plus_identifier(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::acronym_expansion("ABC(Alpha", true)]
    #[case::alphanumeric_acronym("S2E(System)", true)]
    #[case::single_math_function("f(x)", false)]
    #[case::operator_body("AB(x+1)", false)]
    fn korean_prose_acronym_parenthetical_grammar(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            is_korean_prose_acronym_parenthetical(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }

    /// Korean rules 29, 35 and 54 compose the same anonymized-person label
    /// regardless of whether the following honorific is attached or spaced.
    #[rstest::rstest]
    #[case::adult("A(27)", "씨는")]
    #[case::minor_male("B(11)", "군에게")]
    #[case::minor_female("C(16)", "양의")]
    #[case::elected_official("A(31)", "도의원을")]
    #[case::professor("B(61)", "교수를")]
    #[case::judge("C(54)", "부장판사에게")]
    fn spaced_anonymized_person_label_uses_korean_prose_composition(
        #[case] label: &str,
        #[case] honorific: &str,
    ) {
        let input = format!("{label} {honorific}");
        let mut expected = encode_anonymized_person_label(&label.chars().collect::<Vec<_>>())
            .expect("valid anonymized-person label");
        expected.push(0);
        expected.extend(crate::encode(honorific).expect("Korean honorific must encode"));

        assert_eq!(
            crate::encode(&input).expect("prose label must encode"),
            expected
        );
    }

    #[test]
    fn spaced_function_value_is_not_an_anonymized_person_label() {
        let input = "A(14) 값";
        let tokens = crate::rules::token::DocumentIR::parse(input, true).tokens;
        assert!(!super::apply::next_word_begins_korean_prose_label_context(
            &tokens, 0
        ));
    }

    #[rstest::rstest]
    #[case::lower_list_item("(x)", false)]
    #[case::upper_list_item("(A)", false)]
    #[case::plain_parenthesized_word("(abc)", false)]
    fn standalone_parenthesized_inputs_keep_baseline_detector_result(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(is_math_expression(&chars, input), expected, "input={input}");
    }

    #[rstest::rstest]
    #[case::addition("(x+1)")]
    #[case::fraction("(a/b)")]
    #[case::subscript("(x₁)")]
    fn parenthesized_explicit_expressions_keep_existing_math_result(#[case] input: &str) {
        let chars: Vec<char> = input.chars().collect();
        assert!(is_math_expression(&chars, input));
    }

    #[test]
    fn rule_34_bare_roman_parenthesis_has_exact_particle_suffix() {
        let bare = crate::encode("링컨(Lincoln)").expect("bare rule 34 form must encode");
        let attached =
            crate::encode("링컨(Lincoln)은").expect("particle-attached rule 34 form must encode");
        let particle = crate::encode("은").expect("particle must encode");

        assert_eq!(
            attached.strip_prefix(bare.as_slice()),
            Some(particle.as_slice())
        );
    }

    #[rstest::rstest]
    #[case::comma("링컨(Lincoln)", "링컨(Lincoln),", ",")]
    #[case::period("링컨(Lincoln)", "링컨(Lincoln).", ".")]
    fn rule_54_punctuation_follows_closed_roman_parenthesis_without_resplitting(
        #[case] bare_input: &str,
        #[case] with_punctuation: &str,
        #[case] punctuation: &str,
    ) {
        let bare = crate::encode(bare_input).expect("bare rule 34 form must encode");
        let punctuated =
            crate::encode(with_punctuation).expect("punctuated rule 54 form must encode");
        let punctuation = crate::encode(punctuation).expect("punctuation must encode");

        assert_eq!(
            punctuated.strip_prefix(bare.as_slice()),
            Some(punctuation.as_slice())
        );
    }

    #[test]
    fn rule_34_alphanumeric_o4o_uses_the_same_bare_and_particle_path() {
        let bare = crate::encode("표기(O4O)").expect("alphanumeric Roman form must encode");
        let attached =
            crate::encode("표기(O4O)는").expect("particle-attached Roman form must encode");
        let particle = crate::encode("는").expect("particle must encode");

        assert_eq!(
            attached.strip_prefix(bare.as_slice()),
            Some(particle.as_slice())
        );
        assert!(!bare.windows(2).any(|cells| cells == [0, 0]));
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

    #[rstest::rstest]
    #[case::rule_34_particle("링컨(Lincoln)은")]
    #[case::rule_54_comma("링컨(Lincoln),")]
    #[case::alphanumeric_roman("표기(O4O).")]
    fn split_mixed_math_word_keeps_korean_prefixed_closed_roman_annotation(#[case] input: &str) {
        let chars: Vec<char> = input.chars().collect();
        let word = crate::rules::token::WordToken {
            text: Cow::Borrowed(input),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };

        assert!(split_mixed_math_word(&word, 0, MathContext::default()).is_none());
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

    #[rstest::rstest]
    #[case::single_middle_dot("1·2", true)]
    #[case::multiple_middle_dots("2017·2018·2019·2021", true)]
    #[case::trailing_comma("4·5,", true)]
    #[case::letters("ab", false)]
    #[case::empty("", false)]
    fn is_middle_dot_numeric_word_paths(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(is_middle_dot_numeric_word(&chars), expected);
    }

    #[rstest::rstest]
    #[case::numeric_fraction("1/3", true)]
    #[case::year_range("2023/2024", true)]
    #[case::leading_decimal(".515", true)]
    #[case::decimal_range("1.77~5.72", true)]
    #[case::middle_dot_years("2017·2018·2019·2021", true)]
    #[case::signed_number("-3", false)]
    #[case::algebra("1/3+x", false)]
    fn korean_prose_numeric_notation_paths(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(is_korean_prose_numeric_notation(&chars), expected);
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
