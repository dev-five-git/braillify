use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "19",
    subsection: None,
    name: "middle_korean_old_consonants",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.19",
    description: "Old consonants and legacy Middle Korean syllable glyphs",
};

const OLD_ZIYEUT: [u8; 2] = [
    crate::unicode::decode_unicode('⠐'),
    crate::unicode::decode_unicode('⠅'),
];
const OLD_IEUNG: [u8; 2] = [
    crate::unicode::decode_unicode('⠐'),
    crate::unicode::decode_unicode('⠲'),
];
const OLD_HIEUH: [u8; 2] = [
    crate::unicode::decode_unicode('⠐'),
    crate::unicode::decode_unicode('⠴'),
];

const LEGACY_MAPPINGS: &[(char, &str)] = &[
    ('', "⠐⠨⠐⠼"),
    ('', "⠱⠐⠅"),
    ('', "⠐⠙⠎"),
    ('', "⠘⠎⠐⠲"),
    ('', "⠨⠱⠐⠲"),
    ('', "⠐⠚⠪⠢"),
    ('', "⠚⠥⠂⠐⠴"),
    // Historical annotation helper glyph used only as a pronunciation bridge.
    ('', ""),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn old_consonant_body(c: char) -> Option<&'static [u8]> {
    match c {
        'ㅿ' => Some(&OLD_ZIYEUT),
        'ㆁ' => Some(&OLD_IEUNG),
        'ㆆ' => Some(&OLD_HIEUH),
        _ => None,
    }
}

fn is_historical_word(ctx: &RuleContext) -> bool {
    ctx.word_chars.iter().any(|ch| {
        let code = *ch as u32;
        (0x4E00..=0x9FFF).contains(&code) || matches!(*ch, '字' | '')
    })
}

fn has_ja_annotation_markers(ctx: &RuleContext) -> bool {
    ctx.word_chars.contains(&'字') && ctx.word_chars.contains(&'')
}

fn forced_prefix_for_historical_jamo(ctx: &RuleContext) -> u8 {
    if is_historical_word(ctx) {
        return crate::rules::korean::rule_8::WORD_ATTACHED_PREFIX;
    }

    let is_symbol_fn = |ch: char| matches!(CharType::new(ch), Ok(CharType::Symbol(_)));
    crate::rules::korean::rule_8::determine_prefix(
        ctx.word_len(),
        ctx.index,
        ctx.word_chars,
        ctx.has_korean_char,
        is_symbol_fn,
    )
}

fn legacy_symbol_bytes(c: char) -> Option<Vec<u8>> {
    LEGACY_MAPPINGS
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, unicode)| encode_unicode_cells(unicode))
}

pub struct Rule19;

impl BrailleRule for Rule19 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        52
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::KoreanPart(c) if old_consonant_body(*c).is_some())
            || matches!(ctx.char_type, CharType::Symbol(c) if old_consonant_body(*c).is_some())
            || matches!(ctx.char_type, CharType::KoreanPart('ㄱ' | 'ㄷ' | 'ㅂ' | 'ㅅ' | 'ㄹ') if has_ja_annotation_markers(ctx))
            || matches!(ctx.char_type, CharType::Symbol(c) if legacy_symbol_bytes(*c).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if let CharType::KoreanPart(c) | CharType::Symbol(c) = ctx.char_type
            && let Some(body) = old_consonant_body(*c)
        {
            let prefix = forced_prefix_for_historical_jamo(ctx);
            ctx.emit(prefix);
            ctx.emit_slice(body);
            return Ok(RuleResult::Consumed);
        }

        if let CharType::KoreanPart(c) = ctx.char_type
            && matches!(c, 'ㄱ' | 'ㄷ' | 'ㅂ' | 'ㅅ' | 'ㄹ')
            && has_ja_annotation_markers(ctx)
        {
            ctx.emit(crate::rules::korean::rule_8::WORD_ATTACHED_PREFIX);
            ctx.emit_slice(crate::jauem::jongseong::encode_jongseong(*c)?);
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
        let _ = Rule19.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule19.matches(&ctx);
    }
}
