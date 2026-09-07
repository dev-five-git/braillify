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
            // 제43항: 마침표/쉼표가 *숫자 사이*에 있을 때에만 뒤 수표를
            // 생략한다. `M.2`, `No.1`, `2만,4142`처럼 문장 부호의 왼쪽이
            // 숫자가 아닌 경우에는 제40항에 따라 새 수표를 적는다.
            let needs_prefix =
                !is_number_continuation(ctx.word_chars, ctx.index, ctx.state.english_indicator);
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

/// Return whether the digit at `index` follows `digit + (. or ,)`.
///
/// 제43항의 적용 조건은 문장 부호 자체가 아니라 그 문장 부호가 두 숫자
/// 사이에 놓였는지이다. 따라서 로마자나 한글 뒤의 마침표/쉼표는 새 숫자
/// 묶음의 수표를 생략하지 않는다.
pub fn is_number_continuation(word_chars: &[char], index: usize, in_korean_document: bool) -> bool {
    if index == 0 || !matches!(word_chars[index - 1], '.' | ',') {
        return false;
    }

    if in_korean_document {
        return index >= 2 && word_chars[index - 2].is_numeric();
    }

    // UEB 6.3.1: numeric mode continues through a sequence of full stops or
    // commas. It can therefore span `4..7`, but it was never established in
    // an identifier such as `M.2`.
    word_chars[..index]
        .iter()
        .rev()
        .find(|ch| !matches!(ch, '.' | ','))
        .is_some_and(|ch| ch.is_numeric())
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

    /// 제43항 — `.` / `,`가 실제로 숫자 사이에 있을 때만 숫자 흐름에 포함.
    #[rstest::rstest]
    #[case::korean_decimal("3.9", 2, true, true)]
    #[case::korean_grouped("1,000", 2, true, true)]
    #[case::korean_repeated_period("4..7", 3, true, false)]
    #[case::ueb_repeated_period("4..7", 3, false, true)]
    #[case::roman_period("M.2", 2, false, false)]
    #[case::roman_period_in_korean("M.2", 2, true, false)]
    #[case::roman_comma("X,1", 2, true, false)]
    #[case::korean_comma("2만,4142", 3, true, false)]
    #[case::leading_period(".47", 1, false, false)]
    #[case::hyphen("3-4", 2, false, false)]
    #[case::first_digit("7", 0, false, false)]
    fn continuation_chars(
        #[case] input: &str,
        #[case] index: usize,
        #[case] in_korean_document: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            is_number_continuation(
                &input.chars().collect::<Vec<_>>(),
                index,
                in_korean_document,
            ),
            expected
        );
    }

    /// 제35항/제40항/제43항 — 로마자 뒤 마침표는 숫자 사이의 소수점이
    /// 아니므로 뒤 숫자에는 수표를 새로 적는다.
    #[rstest::rstest]
    #[case::capital_identifier("가 M.2 나", "⠍⠲⠼⠃")]
    #[case::all_caps_identifier("가 NO.1 나", "⠕⠲⠼⠁")]
    #[case::korean_before_comma("가 2만,4142명 나", "⠑⠒⠐⠼⠙")]
    #[case::ueb_multiple_periods("4..7", "⠼⠙⠲⠲⠛")]
    fn non_numeric_left_side_does_not_suppress_number_indicator(
        #[case] input: &str,
        #[case] expected_fragment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains(expected_fragment),
            "missing rule-40 number indicator in {actual}"
        );
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
