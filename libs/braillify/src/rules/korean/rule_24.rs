use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "24",
    subsection: None,
    name: "middle_korean_legacy_syllables",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.24",
    description: "Middle Korean legacy syllable glyph mappings",
};

const MAPPINGS: &[(char, &str)] = &[
    ('', "⠐⠨⠣⠢"),
    ('', "⠊⠐⠼"),
    ('', "⠐⠲"),
    ('', "⠐⠘⠶⠕"),
    ('', "⠉⠣⠐⠅"),
    ('', "⠣⠐⠅"),
    ('', "⠐⠲"),
    ('', "⠐⠲"),
    ('', "⠫⠐⠲⠄"),
    ('', "⠐⠘⠶⠣"),
    ('', "⠐⠘⠶⠣⠔"),
    ('', "⠐⠘⠠⠣⠶"),
    ('', "⠠⠐⠼⠗⠶"),
    ('', "⠫⠢⠄"),
    ('', "⠉⠐⠼⠂"),
    ('', "⠐⠘⠶"),
    ('', "⠐⠼⠐⠨⠣"),
    ('', "⠠⠐⠼"),
    ('', "⠑⠐⠼⠄"),
    ('', "⠠⠠⠐⠼"),
    ('', "⠠⠠⠐⠼⠗"),
    ('', "⠚⠐⠼"),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn encode_legacy(c: char) -> Option<Vec<u8>> {
    MAPPINGS
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, unicode)| encode_unicode_cells(unicode))
}

pub struct Rule24;

impl BrailleRule for Rule24 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        59
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if encode_legacy(*c).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        if *c == '' && ctx.prev_char() == Some('바') {
            ctx.emit_slice(&encode_unicode_cells("⠊⠣⠐⠲"));
            return Ok(RuleResult::Consumed);
        }

        if *c == '' {
            ctx.emit_slice(&encode_unicode_cells("⠓⠣⠐⠲"));
            return Ok(RuleResult::Consumed);
        }

        let Some(encoded) = encode_legacy(*c) else {
            return Ok(RuleResult::Skip);
        };

        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule24.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제24항 — Symbol char that is not in MAPPINGS reaches the
    /// `encode_legacy` early-return (line 87-89). Use a plain ASCII symbol.
    #[test]
    fn apply_skip_when_symbol_not_in_mappings() {
        let mut owned = crate::test_helpers::CtxOwned::for_text(".", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule24.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제24항 — meta and phase exercise.
    #[test]
    fn rule24_meta_phase_priority() {
        let r = Rule24;
        assert_eq!(r.meta().section, "24");
        assert!(matches!(r.phase(), Phase::CoreEncoding));
        assert_eq!(r.priority(), 59);
    }
}
