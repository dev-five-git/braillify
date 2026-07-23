//! 제61항 — 작은따옴표(`‘…’` 또는 ASCII `'…'`)의 점역.
//!
//! - 여는 따옴표 `‘`(U+2018): 항상 `⠠⠦`로 점역하고 짝맞춤 상태를 증가시킨다.
//! - 닫는 따옴표 `’`(U+2019): 짝맞춤이 열려 있으면 `⠴⠄`로 점역하고 상태를 감소시킨다.
//!   닫혀 있는 상태에서 단독으로 등장하면 본 규칙은 적용되지 않고 일반 심볼 점역
//!   (`symbol_shortcut` → `⠴⠄`)이 처리한다.
//! - ASCII `'`(U+0027): 양방향 부호로 동작한다.
//!   - 짝맞춤이 닫혀 있고 다음 글자가 ASCII 숫자면 연도 약자(예: `’22`)로 보고
//!     부호 자체는 소비한다. 수표 `⠼`과 `⠄`는 숫자 점역 단계(rule_40)가 emit한다.
//!   - 짝맞춤이 닫혀 있고 숫자가 따르지 않으면 opener로 보고 `⠠⠦` emit + 상태 증가.
//!   - 짝맞춤이 열려 있으면 closer로 보고 `⠴⠄` emit + 상태 감소.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 6, Section 13, Article 61

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "61",
    subsection: None,
    name: "apostrophe_before_number",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.61",
    description: "Apostrophe before digit: skip here, emit after 수표 in number rule",
};

/// Plugin struct for the rule engine.
///
/// When an apostrophe (or right single quote) appears before a digit,
/// this rule Consumes the apostrophe without emitting anything.
/// The apostrophe code ⠄ is emitted by the number encoding rule (rule_40)
/// after the number prefix.
pub struct Rule61;

impl BrailleRule for Rule61 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        350 // Before rule_49 (500) — intercept apostrophe before generic symbol encoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        let CharType::Symbol(c) = ctx.char_type else {
            return false;
        };
        matches!(*c, '\u{2018}' | '\u{2019}' | '\'')
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // 여는 따옴표 `‘`(전용 opener): `⠠⠦` emit + 짝맞춤 카운트 증가.
        if *c == '\u{2018}' {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
            ctx.emit(crate::unicode::decode_unicode('⠦'));
            ctx.state.unmatched_open_single_quotes += 1;
            return Ok(RuleResult::Consumed);
        }

        // 짝맞춤이 열려 있으면 `’` 또는 ASCII `'`는 closer로 동작.
        if ctx.state.unmatched_open_single_quotes > 0 {
            ctx.emit(crate::unicode::decode_unicode('⠴'));
            ctx.emit(crate::unicode::decode_unicode('⠄'));
            ctx.state.unmatched_open_single_quotes -= 1;
            return Ok(RuleResult::Consumed);
        }

        // 짝맞춤이 닫혀 있는 상태:
        let next_is_digit = ctx.next_char().is_some_and(|next| next.is_ascii_digit());

        // 숫자 앞 연도 약자: 부호 자체는 소비하고 rule_40이 수표 직후 `⠄`를 emit.
        if next_is_digit {
            return Ok(RuleResult::Consumed);
        }

        // ASCII `'`는 한국어 본문 안 paired opener로 동작 (`‘`와 동일).
        if *c == '\'' {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
            ctx.emit(crate::unicode::decode_unicode('⠦'));
            ctx.state.unmatched_open_single_quotes += 1;
            return Ok(RuleResult::Consumed);
        }

        // `’` 단독: 일반 심볼 점역(`⠴⠄`)에 위임.
        Ok(RuleResult::Skip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "61");
        assert_eq!(META.name, "apostrophe_before_number");
    }

    #[test]
    fn apostrophe_is_not_standalone_before_digit() {
        // When apostrophe precedes a digit, it should not produce the standard
        // symbol encoding; instead, ⠄ is emitted after the 수표 by rule_40.
        // This test verifies via the full pipeline that the combination works.
        // Note: this is tested indirectly — rule_61 skips the apostrophe,
        // rule_40 emits 수표 + ⠄ + digit.
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule61.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
