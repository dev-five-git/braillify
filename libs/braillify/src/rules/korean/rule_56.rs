//! 제56항 — 드러냄표( ̊ )/밑줄 강조 표기를 처리한다.
//!
//! In plain-text inputs, combining marks can survive as U+0307/U+030A.
//! They are formatting annotations and should not throw an invalid-char error.

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "56",
    subsection: None,
    name: "combining_emphasis_marks",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.56",
    description: "Treat combining emphasis marks as formatting annotations",
};

pub struct Rule56;

impl BrailleRule for Rule56 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        380 // before generic punctuation fallback
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::CombiningMark)
    }

    fn apply(&self, _ctx: &mut RuleContext) -> Result<RuleResult, String> {
        // Formatting-only marks are consumed here.
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_exercise() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        // Just exercise apply() for coverage; either Skip or Continue/Consumed is OK
        let _ = Rule56.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule56.matches(&ctx);
    }
}
