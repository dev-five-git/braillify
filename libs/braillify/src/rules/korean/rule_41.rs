//! 제41항 — 숫자 또는 로마자 구간에서 쉼표는 ⠂(2)으로 적는다.
//!
//! When a comma appears between digits (e.g., "1,000") or between ASCII letters
//! and alphanumeric characters, it uses the numeric comma ⠂ instead of the
//! standard Korean comma ⠐.
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
    description: "Comma between digits/letters uses ⠂ (2) instead of standard comma",
};

/// Numeric comma braille code.
const NUMERIC_COMMA: u8 = 2; // ⠂

/// Plugin struct for the rule engine.
///
/// Handles comma encoding in numeric/English context.
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
        let next_char = get_next_char(ctx);
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

/// Get the next character (within word or from next word).
fn get_next_char(ctx: &RuleContext) -> Option<char> {
    if ctx.index + 1 < ctx.word_chars.len() {
        Some(ctx.word_chars[ctx.index + 1])
    } else {
        ctx.remaining_words.first().and_then(|w| w.chars().next())
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

    /// 제41항 — 숫자/로마자 구간 안의 쉼표는 숫자 쉼표 규칙이 잡는다.
    #[rstest::rstest]
    #[case::between_digits("1,000", 1)]
    #[case::between_ascii_letters("A,B", 1)]
    #[case::ascii_before_digit("A,1", 1)]
    fn rule41_matches_numeric_or_ascii_comma_context(#[case] input: &str, #[case] index: usize) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(index);

        assert!(Rule41.matches(&ctx));
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
