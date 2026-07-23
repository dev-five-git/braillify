use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "72",
    subsection: None,
    name: "placeholder_markers",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.72",
    description: "Single list and placeholder markers without grouping suffix",
};

const MAPPINGS: &[(char, &str)] = &[
    ('○', "⠸⠴"),
    ('□', "⠸⠶"),
    ('△', "⠸⠬"),
    ('•', "⠸⠲"),
    ('◎', "⠸⠴⠴"),
    ('▣', "⠸⠶⠶"),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

pub fn is_rule_72_symbol(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c)
}

pub struct Rule72;

impl BrailleRule for Rule72 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        80
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_rule_72_symbol(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let current = ctx.current_char();
        let repeated = ctx.prev_char() == Some(current) || ctx.next_char() == Some(current);
        if repeated && matches!(current, '○' | '△' | '□') {
            return Ok(RuleResult::Skip);
        }

        let contextual_marker = ctx.word_len() == 1
            || ctx
                .next_char()
                .is_some_and(|c| c.is_whitespace() || matches!(c, '(' | '\'' | '"'))
            || matches!(current, '◎' | '▣');
        if !contextual_marker {
            return Ok(RuleResult::Skip);
        }

        let Some((_, unicode)) = MAPPINGS.iter().find(|(candidate, _)| *candidate == current)
        else {
            return Ok(RuleResult::Skip);
        };
        let encoded = encode_unicode_cells(unicode);
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_to_char(input: char) -> (RuleResult, Vec<u8>) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(&input.to_string(), false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule72.apply(&mut ctx).unwrap();
        (outcome, ctx.result.clone())
    }

    #[rstest::rstest]
    #[case::circle('○', true)]
    #[case::square('□', true)]
    #[case::latin('A', false)]
    fn detects_rule_72_symbols(#[case] input: char, #[case] expected: bool) {
        assert_eq!(is_rule_72_symbol(input), expected);
    }

    #[rstest::rstest]
    #[case::circle('○', "⠸⠴")]
    #[case::square('□', "⠸⠶")]
    #[case::triangle('△', "⠸⠬")]
    #[case::bullet('•', "⠸⠲")]
    #[case::double_circle('◎', "⠸⠴⠴")]
    #[case::filled_square('▣', "⠸⠶⠶")]
    fn apply_encodes_placeholder_markers(#[case] input: char, #[case] expected: &str) {
        let (outcome, output) = apply_to_char(input);

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(output, encode_unicode_cells(expected));
    }

    #[test]
    fn detects_double_circle_placeholder_symbol() {
        assert!(is_rule_72_symbol('◎'));
    }

    #[test]
    fn metadata_reports_rule_72_identity() {
        assert_eq!(Rule72.meta().section, "72");
        assert_eq!(Rule72.phase(), Phase::CoreEncoding);
        assert_eq!(Rule72.priority(), 80);
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule72.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
