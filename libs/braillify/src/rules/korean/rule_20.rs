use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "20",
    subsection: None,
    name: "middle_korean_bieup_series",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.20",
    description: "Middle Korean ㅸ-series and legacy syllable glyphs",
};

/// PDF 제20항 — 연서로 만들어진 옛 자음자 (단독 사용 시).
///
/// 단독 사용 시 옛 글자표 ⠐ + 받침형(있으면) 또는 첫소리형(받침형 없을 시) + 연서표 ⠶.
/// 단독 입력은 제8항 온표(⠿)가 앞에 붙어 emit된다.
const OLD_CONSONANT_BODIES_RULE20: &[(char, &str)] = &[
    ('ㅱ', "⠐⠢⠶"),       // 순경음 미음 — 받침형 ⠐⠢ + 연서표 ⠶
    ('ㅸ', "⠐⠃⠶"),       // 순경음 비읍 — 받침형 ⠐⠃ + 연서표 ⠶
    ('ㅹ', "⠐⠘⠘⠶"),      // 순경음 쌍비읍 — 첫소리형(받침 없음) ⠐⠘⠘ + 연서표 ⠶
    ('ㆄ', "⠐⠙⠶"),       // 순경음 피읖 — 첫소리형(받침 없음) ⠐⠙ + 연서표 ⠶
    ('\u{111B}', "⠐⠐⠶"), // 반설경음 ᄛ — 첫소리형(받침 없음) ⠐⠐ + 연서표 ⠶
];

fn old_consonant_body_rule20(c: char) -> Option<&'static [u8]> {
    static CACHE: std::sync::OnceLock<Vec<(char, Vec<u8>)>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        OLD_CONSONANT_BODIES_RULE20
            .iter()
            .map(|(c, s)| (*c, encode_unicode_cells(s)))
            .collect()
    });
    cache
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, bytes)| bytes.as_slice())
}

const LEGACY_MAPPINGS: &[(char, &str)] = &[
    ('', "⠸"),
    ('', "⠐⠘⠶"),
    ('', "⠐⠼⠐⠨⠣"),
    ('', "⠐⠨⠐⠼⠐⠃⠶"),
    ('', "⠐⠘⠘⠶⠣⠐⠲"),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn legacy_symbol_bytes(c: char) -> Option<Vec<u8>> {
    LEGACY_MAPPINGS
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, unicode)| encode_unicode_cells(unicode))
}

pub struct Rule20;

impl BrailleRule for Rule20 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        53
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        // 제20항 옛 자음자(ㅱ, ㅸ, ㅹ, ㆄ, ᄛ) 또는 PUA legacy 기호.
        // ㅸ는 rule_23 MAPPINGS에 등록되어 있어 CharType::Symbol로 분류되므로
        // Symbol form도 함께 매칭한다.
        matches!(ctx.char_type, CharType::KoreanPart(c) | CharType::Symbol(c)
            if old_consonant_body_rule20(*c).is_some())
            || matches!(ctx.char_type, CharType::Symbol(c) if legacy_symbol_bytes(*c).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if let CharType::KoreanPart(c) | CharType::Symbol(c) = ctx.char_type
            && let Some(body) = old_consonant_body_rule20(*c)
        {
            // 한자 동국정운식 표기 `ㅸ字` 컨텍스트는 선행 PUA(⠸)가 prefix를 제공하므로
            // 본 규칙은 body만 emit한다 (이중 prefix 회피).
            if *c == 'ㅸ' && ctx.next_char() == Some('字') {
                ctx.emit_slice(body);
                return Ok(RuleResult::Consumed);
            }
            // 일반 컨텍스트: 제8항에 따른 prefix(온표 ⠿ 또는 word-attached ⠸) + body.
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
            && let Some(encoded) = legacy_symbol_bytes(*c)
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
        let _ = Rule20.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule20.matches(&ctx);
    }

    #[test]
    fn legacy_symbol_lookup_encodes_pua_mapping() {
        assert_eq!(
            legacy_symbol_bytes(''),
            Some(vec![crate::unicode::decode_unicode('⠸')])
        );
    }
}
