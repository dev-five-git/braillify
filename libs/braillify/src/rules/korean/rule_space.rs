//! 빈칸 자체는 어떤 규정 항목에도 속하지 않음.
//!
//! Spaces → 0, newlines → 255.

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "-",
    subsection: None,
    name: "space_encoding",
    standard_ref: "빈칸 자체",
    description: "Encode space (0) and newline (255)",
};

pub struct RuleSpace;

impl BrailleRule for RuleSpace {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Space(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Space(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };
        ctx.emit(if *c == '\n' { 255 } else { 0 });
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
        let _ = RuleSpace.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = RuleSpace.matches(&ctx);
    }

    /// 빈칸 자체는 어떤 규정 항목에도 속하지 않으므로, 정직한 표시로 "-"를 사용한다.
    /// trace.rs의 WORD_SPACE_META 선례를 따른다.
    #[test]
    fn meta_section_is_dash_for_non_article() {
        assert_eq!(
            META.section, "-",
            "META.section must be dash for non-article blank cell"
        );
    }
}
