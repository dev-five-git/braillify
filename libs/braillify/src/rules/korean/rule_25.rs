use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "25",
    subsection: None,
    name: "middle_korean_vowels",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.25",
    description: "Middle Korean standalone vowels such as ㆍ, ㆎ, ㆇ-ㆌ",
};

const MAPPINGS: &[(char, &str)] = &[
    ('轉', "⠊"),
    ('榮', "⠐⠙"),
    ('ㆍ', "⠐⠼"),
    ('ㆎ', "⠐⠼⠗"),
    ('ㆇ', "⠸⠬⠜"),
    ('ㆈ', "⠸⠬⠜⠗"),
    ('ㆉ', "⠸⠬⠕"),
    ('ㆊ', "⠸⠩⠱"),
    ('ㆋ', "⠸⠩⠌"),
    ('ㆌ', "⠸⠩⠕"),
    ('', "⠈⠐⠼"),
    ('', "⠐⠨⠐⠼⠂"),
    ('', "⠐⠼⠗"),
    ('', "⠑⠐⠼⠗"),
    ('', "⠚⠐⠼⠒"),
    ('', "⠠⠸⠬⠕"),
    ('', "⠸⠩⠱⠒"),
    ('', "⠐⠙⠧⠐⠲"),
    ('', "⠸⠩⠱⠐⠲"),
    ('', "⠜⠐⠲"),
    ('', "⠰⠸⠩⠌"),
    ('', "⠸⠩⠕"),
    ('', "⠰⠸⠩⠕"),
    ('', "⠚⠐⠼"),
];

const SILENT_HANJA: &[char] = &['輪', '王', '養', '砌'];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn is_middle_korean_vowel(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c) || SILENT_HANJA.contains(&c)
}

pub fn is_rule_25_symbol(c: char) -> bool {
    is_middle_korean_vowel(c)
}

pub struct Rule25;

impl BrailleRule for Rule25 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        56
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::KoreanPart(c) if is_middle_korean_vowel(*c))
            || matches!(ctx.char_type, CharType::Symbol(c) if is_middle_korean_vowel(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let c = match ctx.char_type {
            CharType::KoreanPart(c) | CharType::Symbol(c) => *c,
            _ => return Ok(RuleResult::Skip),
        };
        if SILENT_HANJA.contains(&c) {
            return Ok(RuleResult::Consumed);
        }
        let Some((_, unicode)) = MAPPINGS.iter().find(|(candidate, _)| *candidate == c) else {
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
    use crate::char_struct::CharType;
    use crate::rules::context::{EncoderState, RuleContext};

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule25.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제25항 — 중세국어 모음 ㆍ (아래아) standalone emits the legacy mapping.
    /// Triggers the MAPPINGS-found branch (line 86-91).
    #[test]
    fn apply_emits_for_middle_korean_vowel() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("ㆍ", false);
        let mut ctx = owned.ctx_at(0);
        assert!(Rule25.matches(&ctx));
        let outcome = Rule25.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!owned.result.is_empty());
    }

    #[test]
    fn matches_middle_korean_vowel_classified_as_korean_part() {
        let word = ['ㆎ'];
        let char_type = CharType::KoreanPart('ㆎ');
        let mut skip_count = 0;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let ctx = RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &char_type,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip_count,
            state: &mut state,
            result: &mut result,
        };

        assert!(Rule25.matches(&ctx));
    }

    /// 제25항 — SILENT_HANJA characters (輪/王/養/砌) are silently consumed
    /// without emission (line 83-85).
    #[test]
    fn apply_silent_hanja_consumed_without_emit() {
        // '砌' is one of the SILENT_HANJA entries. Its CharType is Symbol.
        let mut owned = crate::test_helpers::CtxOwned::for_text("砌", false);
        let mut ctx = owned.ctx_at(0);
        assert!(Rule25.matches(&ctx));
        let outcome = Rule25.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(owned.result.is_empty());
    }

    /// 제25항 — Symbol char that is not in MAPPINGS reaches Skip (line 86-88).
    #[test]
    fn apply_skip_when_not_in_mappings() {
        let mut owned = crate::test_helpers::CtxOwned::for_text(".", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule25.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
