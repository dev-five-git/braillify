//! 제40항 — 숫자는 수표 ⠼(60)을 앞세워 다음과 같이 적는다.
//!
//! 제43항 — 숫자 사이에 마침표, 쉼표, 연결표가 붙어 나올 때에는 뒤의 숫자에 수표를 적지 않는다.
//!
//! The number indicator ⠼ (code 60) is prepended before the first digit in a number sequence.
//! Within a sequence, if separated by . or , the indicator is NOT repeated.
//!
//! Digit encoding is delegated to `number::encode_number()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 5, Section 11, Articles 40, 43

use crate::char_struct::CharType;
use crate::number;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META_40: RuleMeta = RuleMeta {
    section: "40",
    subsection: None,
    name: "number_prefix",
    standard_ref: "2024 Korean Braille Standard, Ch.5 Sec.11 Art.40",
    description: "Number indicator ⠼ (60) before first digit in number sequence",
};

/// Number indicator (수표).
pub const NUMBER_INDICATOR: u8 = 60; // ⠼

/// Encode a digit character to braille.
#[cfg(test)]
fn encode_digit(ch: char) -> Result<u8, String> {
    number::encode_number(ch)
}

/// Plugin struct for the rule engine.
///
/// Handles number encoding with prefix indicator (제40항, 제43항).
/// Emits 수표 ⠼ before the first digit in a sequence. Subsequent digits
/// after continuation characters (`.`, `,`) do not repeat the prefix.
/// Fraction detection and complex numeric formatting are separate concerns.
pub struct Rule40;

impl BrailleRule for Rule40 {
    fn meta(&self) -> &'static RuleMeta {
        &META_40
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Number(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Number(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // PDF 제40항/제69항 — numeric+unit prefix는 Rule69(priority=90)가
        // Rule40(priority=100, 기본값)보다 먼저 처리한다. 이 분기는 dead code였다.
        // (rule_69.rs:174-181 matches() + 184-196 apply() 참조)

        if !ctx.state.is_number {
            // 제43항: skip prefix after continuation characters (. or ,)
            let needs_prefix = ctx
                .prev_char()
                .is_none_or(|prev| !is_number_continuation(prev));
            if needs_prefix {
                ctx.emit(NUMBER_INDICATOR);
                // 제61항: apostrophe/right single quote before number emits ⠄ after 수표
                if ctx
                    .prev_char()
                    .is_some_and(|prev| prev == '\'' || prev == '\u{2019}')
                {
                    ctx.emit(4);
                }
            }
            ctx.state.is_number = true;
        }
        let digit = number::encode_number(*c)?;
        ctx.emit(digit);
        Ok(RuleResult::Consumed)
    }
}

/// Check if the previous character is a continuation character (. or ,)
/// that should suppress the number indicator on the next digit.
pub fn is_number_continuation(prev: char) -> bool {
    prev == '.' || prev == ','
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    /// 제40항 — 숫자 0-9 점형.
    #[rstest::rstest]
    #[case::one('1', '⠁')]
    #[case::zero('0', '⠚')]
    #[case::nine('9', '⠊')]
    fn encodes_digits(#[case] ch: char, #[case] expected: char) {
        assert_eq!(encode_digit(ch).unwrap(), decode_unicode(expected));
    }

    #[test]
    fn invalid_digit() {
        assert!(encode_digit('a').is_err());
    }

    /// `is_number_continuation` — `.` / `,` 만 숫자 흐름에 포함.
    #[rstest::rstest]
    #[case::period('.', true)]
    #[case::comma(',', true)]
    #[case::space(' ', false)]
    #[case::hyphen('-', false)]
    fn continuation_chars(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(is_number_continuation(ch), expected);
    }

    /// PDF 제40항 + 제69항 — numeric prefix followed by ASCII unit (kg, cm, etc.)
    /// is handled by Rule69 (priority=90) BEFORE Rule40 (priority=100). This test
    /// verifies the integration path works (not Rule40's apply specifically).
    #[test]
    fn number_with_ascii_unit_prefix_handled_by_rule69() {
        let cases = vec!["1kg", "5cm", "10mm", "3m", "2h", "100GB"];
        for input in cases {
            let result = crate::encode(input);
            assert!(
                result.is_ok(),
                "encode({input}) should succeed via Rule69 path"
            );
            let bytes = result.unwrap();
            assert!(!bytes.is_empty(), "non-empty output for {input}");
        }
    }

    /// rule_40 line 52 — `let-else return Skip` for non-Number ctx.
    #[test]
    fn rule40_apply_skip_for_non_number_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("가", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule40.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
