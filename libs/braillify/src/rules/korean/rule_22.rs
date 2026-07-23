use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "22",
    subsection: None,
    name: "middle_korean_fortis_series",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.22",
    description: "Middle Korean fortis/cluster legacy syllable glyphs",
};

/// PDF 제22항 — 합용 병서로 만들어진 옛 자음자가 첫소리로 쓰일 때 (단독 사용 시).
///
/// 단독 사용 시 옛 글자표 ⠐ + 어울러 적은 형태. 단독 입력은 제8항 온표(⠿)가 prefix.
/// ㅄ(U+3144)는 modern 한국어에서 받침(ㅂㅅ)으로 사용되므로 본 규칙에서 제외 — 별도
/// 모던 처리(rule_8 + korean_part)가 담당한다. 옛 합용 병서 ㅄ가 필요하면 Old Hangul
/// 코드포인트(ᄡ U+1121)를 사용한다 (향후 별도 지원 검토).
const OLD_CONSONANT_BODIES_RULE22: &[(char, &str)] = &[
    ('ㅲ', "⠐⠘⠈"),  // ㅲ 비읍기역
    ('ㅳ', "⠐⠘⠊"),  // ㅳ 비읍디귿
    ('ᄡ', "⠐⠘⠠"),  // ᄡ 비읍시옷 (Old Hangul U+1121; Compat ㅄ는 모던 받침으로 별도 처리)
    ('ㅶ', "⠐⠘⠨"),  // ㅶ 비읍지읒
    ('ㅷ', "⠐⠘⠓"),  // ㅷ 비읍티읕
    ('ㅴ', "⠐⠘⠠⠈"), // ㅴ 비읍시옷기역
    ('ㅵ', "⠐⠘⠠⠊"), // ㅵ 비읍시옷디귿
    ('ㅺ', "⠐⠠⠈"),  // ㅺ 시옷기역
    ('ㅻ', "⠐⠠⠉"),  // ㅻ 시옷니은
    ('ㅼ', "⠐⠠⠊"),  // ㅼ 시옷디귿
    ('ㅽ', "⠐⠠⠘"),  // ㅽ 시옷비읍
    ('ㅾ', "⠐⠠⠨"),  // ㅾ 시옷지읒
];

fn old_consonant_body_rule22(c: char) -> Option<&'static [u8]> {
    static CACHE: std::sync::OnceLock<Vec<(char, Vec<u8>)>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        OLD_CONSONANT_BODIES_RULE22
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
    ('', "⠐⠘⠈⠪"),
    ('', "⠐⠘⠊⠪"),
    ('', "⠐⠘⠠⠣"),
    ('', "⠉⠐⠼⠒"),
    ('', "⠐⠘⠨⠣⠁"),
    ('', "⠐⠘⠓⠎"),
    ('', "⠐⠘⠠⠈⠪⠢"),
    ('', "⠐⠘⠠⠊⠗"),
    ('', "⠐⠠⠈⠎"),
    ('', "⠐⠠⠉⠣"),
    ('', "⠐⠠⠊⠐⠼⠂"),
    ('', "⠐⠠⠘⠥⠐⠲"),
    ('', "⠐⠠⠨⠺"),
    ('', "⠊⠐⠼⠂⠁⠄"),
    ('禽', "⠈⠪⠢⠵"),
    ('', "⠉⠐⠼⠂"),
    ('', "⠐⠴⠨⠩⠐⠲"),
    ('', "⠠⠐⠼⠗⠐⠲"),
    ('', "⠨⠕⠢⠄"),
    ('', "⠠⠜⠐⠲⠄"),
    ('', "⠊⠐⠼⠂"),
    ('', "⠚⠐⠼⠂"),
    ('', "⠠⠠⠐⠼"),
    ('', "⠠⠠⠐⠼⠗"),
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

pub struct Rule22;

impl BrailleRule for Rule22 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        58
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::KoreanPart(c) | CharType::Symbol(c)
            if old_consonant_body_rule22(*c).is_some())
            || matches!(ctx.char_type, CharType::Symbol(c) if encode_legacy(*c).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        // 제22항 합용 병서 옛 자음자 (ㅲ, ㅳ, ㅶ, ㅷ, ㅴ, ㅵ, ㅺ, ㅻ, ㅼ, ㅽ, ㅾ):
        // 제8항 prefix(온표 또는 word-attached) + body.
        if let CharType::KoreanPart(c) | CharType::Symbol(c) = ctx.char_type
            && let Some(body) = old_consonant_body_rule22(*c)
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

        if ctx.current_char() == '禽' && ctx.next_char() == Some('은') {
            ctx.emit_slice(&encode_unicode_cells("⠈⠪⠢⠵"));
            *ctx.skip_count = 1;
            return Ok(RuleResult::Consumed);
        }

        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

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
        let outcome = Rule22.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// PDF 제22항 — `apply` reaches the `encode_legacy` fallthrough and returns
    /// `Skip` when the Symbol char is recognised by neither
    /// `old_consonant_body_rule22` nor `encode_legacy` (line 140-142).
    /// Exercised by a plain symbol like `.`.
    #[test]
    fn apply_returns_skip_for_unknown_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text(".", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule22.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// PDF 제22항 — 옛 합용 병서 ㅺ (시옷기역) standalone input.
    /// Triggers the `old_consonant_body_rule22` consumed branch.
    #[test]
    fn apply_emits_for_old_consonant_body() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("ㅺ", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule22.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!owned.result.is_empty());
    }
}
