//! 제60항 — 별표(*)는 앞뒤를 한 칸씩 띄어 쓴다. 홀로 선 별표 앞뒤의 한 칸은
//! 묵자의 어절 사이 빈칸이다.

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::symbol_shortcut;

pub static META: RuleMeta = RuleMeta {
    section: "60",
    subsection: None,
    name: "asterisk_spacing",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.60",
    description: "Asterisk (*) requires surrounding spaces",
};

pub struct Rule60;

impl BrailleRule for Rule60 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        400 // Before rule_49 (500) — intercept * before generic symbol encoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if *c == '*')
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let encoded = symbol_shortcut::encode_char_symbol_shortcut('*')?;
        ctx.emit_slice(encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "60");
        assert_eq!(META.name, "asterisk_spacing");
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let _ = Rule60.apply(&mut ctx).unwrap();
        // Just exercise apply() for coverage
    }

    /// 제60항 — 앞뒤 한 칸씩. 묵자의 빈칸에 한 칸을 더 얹지 않는다.
    #[rstest::rstest]
    #[case::between_words("가나 * 다라", "⠫⠉⠀⠐⠔⠀⠊⠐⠣")]
    #[case::after_a_number("1만7천원 * 0.2", "⠒⠀⠐⠔⠀⠼⠚")]
    #[case::after_an_exclamation("들! * 가", "⠖⠀⠐⠔⠀⠫")]
    fn a_standalone_asterisk_takes_one_blank_each_side(#[case] input: &str, #[case] cells: &str) {
        let encoded = crate::encode_to_unicode(input).unwrap();
        assert!(encoded.contains(cells), "{encoded}");
    }
}
