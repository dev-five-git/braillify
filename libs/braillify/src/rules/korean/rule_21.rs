use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "21",
    subsection: None,
    name: "middle_korean_hieuh_series",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.21",
    description: "Middle Korean aspirated old-consonant composites",
};

/// PDF 제21항 — 각자 병서로 만들어진 옛 자음자 (단독 사용 시).
///
/// 단독 사용 시 옛 글자표 ⠐ + 각자 병서 form. 단독 입력은 제8항 온표(⠿)가 prefix.
/// (제20항과 달리 연서표 ⠶이 붙지 않는다.)
const OLD_CONSONANT_BODIES_RULE21: &[(char, &str)] = &[
    ('ㅥ', "⠐⠉⠉"), // 쌍니은 — 옛글자표 ⠐ + ⠉⠉
    ('ㆀ', "⠐⠛⠛"), // 쌍이응 — 옛글자표 ⠐ + ⠛⠛
    ('ㆅ', "⠐⠚⠚"), // 쌍히읗 — 옛글자표 ⠐ + ⠚⠚
];

fn old_consonant_body_rule21(c: char) -> Option<&'static [u8]> {
    static CACHE: std::sync::OnceLock<Vec<(char, Vec<u8>)>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        OLD_CONSONANT_BODIES_RULE21
            .iter()
            .map(|(c, s)| (*c, encode_unicode_cells(s)))
            .collect()
    });
    cache
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, bytes)| bytes.as_slice())
}

const MAPPINGS: &[(char, &str)] = &[
    ('', "⠐⠉⠉⠐⠼"),
    ('', "⠚⠐⠼⠗"),
    ('', "⠐⠛⠛⠱"),
    ('', "⠐⠐⠼"),
    ('', "⠐⠚⠚⠱"),
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

pub struct Rule21;

impl BrailleRule for Rule21 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        54
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::KoreanPart(c) | CharType::Symbol(c)
            if old_consonant_body_rule21(*c).is_some())
            || matches!(ctx.char_type, CharType::Symbol(c) if encode_legacy(*c).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        // 제21항 옛 자음자 (ㅥ, ㆀ, ㆅ): 제8항 prefix(온표 또는 word-attached) + body.
        if let CharType::KoreanPart(c) | CharType::Symbol(c) = ctx.char_type
            && let Some(body) = old_consonant_body_rule21(*c)
        {
            let is_symbol_fn = |ch: char| matches!(CharType::new(ch), Ok(CharType::Symbol(_)));
            let prefix = crate::rules::korean::rule_8::determine_prefix(
                ctx.word_len(),
                ctx.index,
                ctx.word_chars,
                ctx.has_korean_char,
                is_symbol_fn,
            );
            ctx.emit(prefix);
            ctx.emit_slice(body);
            return Ok(RuleResult::Consumed);
        }

        if let CharType::Symbol(c) = ctx.char_type
            && let Some(encoded) = encode_legacy(*c)
        {
            ctx.emit_slice(&encoded);
            return Ok(RuleResult::Consumed);
        }

        Ok(RuleResult::Skip)
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
        let _ = Rule21.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule21.matches(&ctx);
    }

    #[test]
    fn matches_old_consonant_body_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("ㅥ", false);
        let ctx = owned.ctx_at(0);

        assert!(Rule21.matches(&ctx));
    }
}
