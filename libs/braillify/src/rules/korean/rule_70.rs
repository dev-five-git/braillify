use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "70",
    subsection: None,
    name: "arrows",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.70",
    description: "Directional arrow symbols",
};

const MAPPINGS: &[(char, &str)] = &[
    ('→', "⠒⠕"),
    ('←', "⠪⠒"),
    ('↔', "⠪⠒⠕"),
    ('↓', "⠘⠒⠕"),
    ('↑', "⠰⠒⠕"),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

pub fn is_arrow_symbol(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c)
}

pub struct Rule70;

impl BrailleRule for Rule70 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        170
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_arrow_symbol(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let Some((_, unicode)) = MAPPINGS
            .iter()
            .find(|(candidate, _)| *candidate == ctx.current_char())
        else {
            return Ok(RuleResult::Skip);
        };
        // 제70항은 화살표의 앞뒤를 한 칸씩 띄도록 명시한다. 묵자에
        // 공백이 이미 있으면 별도 Space token이 담당하므로, 같은 token
        // 안에 인접 문자가 있을 때만 누락된 한 칸을 보충한다.
        if ctx.index > 0 && ctx.result.last() != Some(&0) {
            ctx.emit(0);
        }
        let encoded = encode_unicode_cells(unicode);
        ctx.emit_slice(&encoded);
        if ctx.index + 1 < ctx.word_len() {
            ctx.emit(0);
        }
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::right_arrow('→', "⠒⠕")]
    #[case::left_arrow('←', "⠪⠒")]
    #[case::left_right_arrow('↔', "⠪⠒⠕")]
    #[case::down_arrow('↓', "⠘⠒⠕")]
    #[case::up_arrow('↑', "⠰⠒⠕")]
    fn encode_arrow_symbol_table(#[case] input: char, #[case] expected: &str) {
        assert_eq!(encode_unicode_cells(expected), encode_enclosed_arrow(input));
    }

    /// 제70항 — 화살표 앞뒤 한 칸은 묵자 공백 유무와 무관하게 보장하며,
    /// 이미 띄어 쓴 공식 예제에는 공백을 중복하지 않는다.
    #[rstest::rstest]
    #[case::tight_both_sides("부산→서울", "⠘⠍⠇⠒⠀⠒⠕⠀⠠⠎⠯")]
    #[case::tight_right_side("←행주대교", "⠪⠒⠀⠚⠗⠶⠨⠍⠊⠗⠈⠬")]
    #[case::tight_left_side("거래량↓", "⠈⠎⠐⠗⠐⠜⠶⠀⠘⠒⠕")]
    #[case::already_spaced("부산 → 서울", "⠘⠍⠇⠒⠀⠒⠕⠀⠠⠎⠯")]
    fn enforces_one_blank_around_arrow(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule70.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    fn encode_enclosed_arrow(input: char) -> Vec<u8> {
        let mut owned = crate::test_helpers::CtxOwned::for_text(&input.to_string(), false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule70.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        ctx.result.clone()
    }
}
