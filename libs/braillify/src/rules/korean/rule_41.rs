//! 제41항 — 숫자 사이에 붙어 나오는 쉼표는 ⠂(2)으로 적는다.
//!
//! When a comma is attached between digits (e.g., "1,000"), it uses the numeric
//! comma ⠂ instead of the standard Korean comma ⠐. A whitespace boundary means
//! the comma is ordinary punctuation under rule 49, not an attached numeric
//! comma under this rule.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 5, Section 11, Article 41

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
pub static META: RuleMeta = RuleMeta {
    section: "41",
    subsection: None,
    name: "numeric_comma",
    standard_ref: "2024 Korean Braille Standard, Ch.5 Sec.11 Art.41",
    description: "Attached comma within a numeric/ASCII sequence uses ⠂ (2)",
};

/// Numeric comma braille code.
const NUMERIC_COMMA: u8 = 2; // ⠂

/// Plugin struct for the rule engine.
///
/// Handles attached comma encoding in numeric/English context.
/// Runs before generic punctuation (rule_49) to intercept commas.
pub struct Rule41;

impl BrailleRule for Rule41 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        400 // Before rule_49 (500) — intercept comma before generic punctuation
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        let CharType::Symbol(c) = ctx.char_type else {
            return false;
        };
        if *c != ',' {
            return false;
        }

        let (has_numeric_prefix, has_ascii_prefix) = scan_prefix(ctx.word_chars, ctx.index);
        // 제41항의 "붙어 나오는" 경계만 본다. `remaining_words`까지
        // 건너뛰면 `1, 2`의 일반 쉼표를 숫자 쉼표로 오분류한다.
        let next_char = ctx.word_chars.get(ctx.index + 1).copied();
        let next_is_digit = next_char.is_some_and(|ch| ch.is_ascii_digit());
        let next_is_ascii = next_char.is_some_and(|ch| ch.is_ascii_alphabetic());
        let next_is_alphanumeric = next_is_digit || next_is_ascii;

        // Comma between numbers, or between ASCII and alphanumeric
        ((ctx.state.is_number || has_numeric_prefix) && next_is_digit)
            || (has_ascii_prefix && next_is_alphanumeric)
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        ctx.emit(NUMERIC_COMMA);
        Ok(RuleResult::Consumed)
    }
}

/// Scan backwards from index to find if preceded by a digit or ASCII letter.
fn scan_prefix(word_chars: &[char], index: usize) -> (bool, bool) {
    match word_chars[..index]
        .iter()
        .rev()
        .copied()
        .find(|prev| *prev != ' ')
    {
        Some(prev) => (prev.is_ascii_digit(), prev.is_ascii_alphabetic()),
        None => (false, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `scan_prefix` — 직전 prefix 가 digit 흐름인지 ASCII 흐름인지 식별.
    #[rstest::rstest]
    #[case::digit_prefix("1,000", 1, true, false)]
    #[case::ascii_prefix("A,B", 1, false, true)]
    fn scan_prefix_paths(
        #[case] input: &str,
        #[case] idx: usize,
        #[case] expect_num: bool,
        #[case] expect_ascii: bool,
    ) {
        let chars: Vec<char> = input.chars().collect();
        let (num, ascii) = scan_prefix(&chars, idx);
        assert_eq!(num, expect_num);
        assert_eq!(ascii, expect_ascii);
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "41");
        assert_eq!(META.name, "numeric_comma");
    }

    /// rule_41 line 39 — `let-else return false` for non-Symbol ctx.
    #[test]
    fn rule41_matches_false_for_non_symbol_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("ab", false);
        let ctx = owned.ctx_at(0);
        assert!(!Rule41.matches(&ctx));
    }

    /// 제41항 숫자 쉼표와 UEB의 같은-token 로마자 쉼표 경로.
    #[rstest::rstest]
    #[case::between_digits("1,000", 1)]
    #[case::between_ascii_letters("A,B", 1)]
    #[case::ascii_before_digit("A,1", 1)]
    fn rule41_matches_numeric_or_ascii_comma_context(#[case] input: &str, #[case] index: usize) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(index);

        assert!(Rule41.matches(&ctx));
    }

    #[rstest::rstest]
    // PDF physical p.209: `제5열 버튼(3, 7 혹은 S)`.
    #[case::music_button_list("3,", "7")]
    // PDF physical p.142: `1/3, 2/3의 길이`.
    #[case::music_fraction_list("1/3,", "2/3의")]
    fn rule41_does_not_cross_whitespace_token_boundaries(
        #[case] current_word: &str,
        #[case] next_word: &str,
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(current_word, false)
            .with_remaining_words([next_word]);
        let comma_index = current_word
            .chars()
            .position(|ch| ch == ',')
            .expect("test word must contain a comma");
        let ctx = owned.ctx_at(comma_index);

        assert!(!Rule41.matches(&ctx));
    }

    /// 제41항/제49항 PDF 예제를 전체 인코더로 통과시켜 붙은 숫자 쉼표와
    /// 일반 한글 쉼표의 서로 다른 셀을 함께 고정한다.
    #[rstest::rstest]
    #[case::rule41_grouped_number("9,375명", '⠂')]
    #[case::rule41_verse_reference("창세기 12,1-9", '⠂')]
    #[case::rule49_korean_list("근면, 검소, 협동은 우리 겨레의 미덕이다.", '⠐')]
    fn full_encoder_preserves_pdf_comma_boundaries(
        #[case] input: &str,
        #[case] expected_comma: char,
    ) {
        let comma_byte = input.find(',').expect("PDF example must contain comma");
        let prefix = crate::encode_to_unicode(&input[..comma_byte]).expect("prefix must encode");
        let actual = crate::encode_to_unicode(input).expect("PDF example must encode");
        let comma_cell = actual.chars().nth(prefix.chars().count());

        assert!(actual.starts_with(&prefix));
        assert_eq!(comma_cell, Some(expected_comma));
    }

    /// rule_41 line 75 — `j -= 1;` when prev char is a space (continues backward scan).
    #[test]
    fn scan_prefix_skips_space_then_finds_digit() {
        let chars: Vec<char> = "1 ,".chars().collect();
        let (num, _) = scan_prefix(&chars, 2);
        // prev=` `, j-=1, prev=`1`→ digit → has_numeric_prefix=true.
        assert!(num);
    }
}
