//! General Korean syllable encoding — the fallback rule.
//!
//! Wraps `korean_char::encode_korean_char()` which handles the full syllable
//! encoding pipeline: abbreviation combination lookups, choseong/jungseong/jongseong
//! decomposition, and all shortcut optimizations from articles 1-7, 13, 15.
//!
//! This rule runs AFTER rules 16 (exception chars), 14 (no-abbreviation),
//! and 13 (single-char abbreviation), serving as the general-purpose fallback
//! for Korean syllables that weren't caught by those specialized rules.

use crate::char_struct::CharType;
use crate::korean_char::encode_korean_char;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "1",
    subsection: Some("general"),
    name: "korean_syllable_encoding",
    standard_ref: "2024 Korean Braille Standard, Ch.1-2 (composite)",
    description: "General Korean syllable encoding via encode_korean_char()",
};

/// Plugin struct for the rule engine.
///
/// Fallback Korean syllable encoding. Calls `encode_korean_char()` which
/// performs multi-level shortcut combination lookups before decomposing
/// into choseong + jungseong + jongseong components.
pub struct RuleKorean;

impl BrailleRule for RuleKorean {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        150 // After Rule16(70), Rule14(80), Rule13(90) — general fallback
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Korean(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let Some(korean) = ctx.as_korean() else {
            return Ok(RuleResult::Skip);
        };
        let encoded = encode_korean_char(korean)?;
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "1");
        assert_eq!(META.subsection, Some("general"));
    }

    /// rule_korean line 45 — `let-else return Skip` for non-Korean ctx.
    #[test]
    fn rule_korean_apply_skip_for_non_korean_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = RuleKorean.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
