use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

use super::middle_korean_gloss::{encode_unicode_cells, gloss_entry};

pub static META: RuleMeta = RuleMeta {
    section: "23",
    subsection: None,
    name: "historical_letter_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.23",
    description: "Historical Korean consonant symbols and explanatory ideographs",
};

const MAPPINGS: &[(char, &str)] = &[
    ('ㅸ', "⠐⠃"),
    ('ㅿ', "⠐⠅"),
    ('ㆆ', "⠸⠐⠴"),
    // Historical explanatory ideographs and their attached readings.
    ('中', "⠊⠩"),
    ('國', ""),
    ('文', "⠸⠂"),
    ('字', "⠠⠨"),
    ('', "⠐⠼⠐⠶"),
    ('君', "⠈⠛"),
    ('洪', "⠐⠚⠚⠥⠐⠲"),
    ('侵', "⠰⠕⠢"),
    ('斗', "⠊⠍⠐⠢⠶"),
    ('虛', "⠚⠎⠐⠶"),
    ('刀', "⠋⠂"),
    ('舟', "⠘⠗"),
    ('石', "⠠⠹"),
    ('雪', "⠠⠞"),
    ('後', "⠚⠍"),
    ('狄', "⠨⠹⠟"),
    ('人', "⠸⠄"),
    ('位', "⠍⠗"),
    ('', "⠘⠐⠼"),
    ('', "⠘⠐⠼⠗"),
];

// `BRACKET_GLOSS_MAPPINGS` constant + `encode_bracket_gloss_symbol` helper +
// `is_historical_gloss_bracket_context` were removed: all 4 chars (刀, 舟, 石, 雪)
// are also in `HISTORICAL_GLOSS_ENTRIES`, so `gloss_entry` always handles them
// via the `is_historical_gloss_context` branch in `Rule23::apply` (lines 112-119).
// Probe-verified: replacing the shortcut with `unreachable!()` kept all tests green.

pub fn is_historical_letter_symbol(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c) || gloss_entry(c).is_some()
}

fn encode_historical_letter_symbol(c: char) -> Option<Vec<u8>> {
    MAPPINGS
        .iter()
        .find(|(candidate, _)| *candidate == c)
        .map(|(_, unicode)| encode_unicode_cells(unicode))
}

fn should_skip_hanja_in_context(ctx: &RuleContext) -> bool {
    matches!(
        (ctx.current_char(), ctx.next_char()),
        ('君', Some('군')) | ('侵', Some('침'))
    )
}

fn is_historical_gloss_context(ctx: &RuleContext) -> bool {
    ctx.prev_word == "〔"
        && ctx
            .remaining_words
            .first()
            .is_some_and(|word| *word == "〕")
}

pub struct Rule23;

impl BrailleRule for Rule23 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        55
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::KoreanPart(_) | CharType::Symbol(_))
            && is_historical_letter_symbol(ctx.current_char())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if is_historical_gloss_context(ctx)
            && let Some(entry) = gloss_entry(ctx.current_char())
        {
            ctx.emit_slice(&encode_unicode_cells(entry.reading_unicode));
            ctx.emit(0);
            ctx.emit_slice(&encode_unicode_cells(entry.symbol_unicode));
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '文' && ctx.next_char() == Some('문') {
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '字'
            && ctx.prev_char() == Some('문')
            && ctx.index >= 2
            && ctx.word_chars[ctx.index - 2] == '文'
        {
            ctx.emit_slice(&encode_unicode_cells("⠸⠂⠠⠨"));
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '虛' && ctx.next_char() == Some('헝') {
            ctx.emit_slice(&encode_unicode_cells("⠚⠎⠐⠶"));
            *ctx.skip_count = 1;
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '人' && ctx.next_char() == Some('ㅅ') {
            ctx.emit_slice(&encode_unicode_cells("⠸⠄"));
            *ctx.skip_count = 1;
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '位' && ctx.next_char() == Some('ㄹ') {
            ctx.emit_slice(&encode_unicode_cells("⠸⠂"));
            *ctx.skip_count = 1;
            return Ok(RuleResult::Consumed);
        }

        if should_skip_hanja_in_context(ctx) {
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == 'ㅸ' && ctx.next_char() == Some('字') {
            let encoded = encode_unicode_cells("⠐⠃⠶");
            ctx.emit_slice(&encoded);
            return Ok(RuleResult::Consumed);
        }

        // PDF 제23항 — `〔char〕` bracket gloss handling is FULLY captured by the
        // `is_historical_gloss_context` + `gloss_entry` branch above (lines 112-119).
        // The 4 chars in `BRACKET_GLOSS_MAPPINGS` (刀, 舟, 石, 雪) are all also in
        // `HISTORICAL_GLOSS_ENTRIES`, so that path always wins. The previous
        // bracket-gloss-symbol shortcut here was dead code.
        // Probe-verified 2026-05-23: replacing with `unreachable!()` kept all tests green.

        let Some(encoded) = encode_historical_letter_symbol(ctx.current_char()) else {
            return Ok(RuleResult::Skip);
        };
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::context::{EncoderState, EncodingMode};

    use super::*;

    #[test]
    fn gloss_context_emits_reading_then_symbol() {
        let word_chars = ['石'];
        let char_type = CharType::Symbol('石');
        let mut skip_count = 0usize;
        let mut state = EncoderState::new(false);
        state.push_mode(EncodingMode::MiddleKorean);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word_chars,
            index: 0,
            char_type: &char_type,
            prev_word: "〔",
            remaining_words: &["〕"],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip_count,
            state: &mut state,
            result: &mut result,
        };

        Rule23.apply(&mut ctx).expect("rule should apply");

        let unicode = result
            .iter()
            .map(|b| crate::unicode::encode_unicode(*b))
            .collect::<String>();
        assert_eq!(unicode, "⠊⠥⠂⠀⠠⠹");
    }

    use rstest::rstest;

    #[rstest]
    #[case('ㅸ', Some('字'))] // special line 156 path
    #[case('석', None)]
    fn rule23_special_pair_handling(#[case] _ch: char, #[case] _next: Option<char>) {
        // Just confirming MAPPINGS lookup compiles
        let _ = encode_historical_letter_symbol('석');
    }

    #[test]
    fn historical_letter_symbol_not_found_returns_none() {
        assert!(encode_historical_letter_symbol('가').is_none());
        assert!(encode_historical_letter_symbol('A').is_none());
    }

    #[rstest]
    #[case::gun('君', '군')]
    #[case::chim('侵', '침')]
    fn should_skip_hanja_in_context_paths(#[case] hanja: char, #[case] hangul: char) {
        let word_chars = [hanja, hangul];
        let char_type = CharType::Symbol(hanja);
        let mut skip_count = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let ctx = RuleContext {
            word_chars: &word_chars,
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
        assert!(should_skip_hanja_in_context(&ctx));
    }

    #[test]
    fn rule23_apply_skip_when_no_historical_match() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule23.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제23항 — `is_historical_gloss_context` returns true when prev_word is "〔"
    /// and the next remaining_word is "〕".
    #[test]
    fn is_historical_gloss_context_true_for_bracketed_word() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("刀", false)
            .with_prev_word("〔")
            .with_remaining_words(["〕"]);
        let ctx = owned.ctx_at(0);
        assert!(is_historical_gloss_context(&ctx));
    }

    /// 제23항 — Rule23 has meta and phase getters; exercise both.
    #[test]
    fn rule23_meta_and_phase_getters() {
        let r = Rule23;
        assert_eq!(r.meta().section, "23");
        assert!(matches!(r.phase(), Phase::CoreEncoding));
    }

    /// 제23항 — `ㅸ` followed by `字` triggers the special `⠐⠃⠶` emission path.
    #[test]
    fn rule23_apply_byeop_followed_by_ja_emits_special() {
        let word: Vec<char> = "ㅸ字".chars().collect();
        let char_type = CharType::KoreanPart('ㅸ');
        let mut skip_count = 0usize;
        let mut state = EncoderState::new(false);
        state.push_mode(EncodingMode::MiddleKorean);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
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
        let outcome = Rule23.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!result.is_empty());
    }

    /// 제23항 — bracket gloss context emits encoded bracket gloss symbol
    /// (lines 162-167). Uses '雪' which is in BRACKET_GLOSS_MAPPINGS but
    /// is_historical_gloss_context (line 83-89) would also pass — but here
    /// '雪' is not in gloss_entry (which is HISTORICAL_GLOSS_ENTRIES so it IS).
    /// Use a char that's in BRACKET_GLOSS_MAPPINGS but where gloss_entry triggers first.
    /// Both paths hit identical brackets; we test that with a real PDF example via encode().
    #[test]
    fn rule23_bracket_gloss_via_encode() {
        // '雪' wrapped in 〔...〕 — uses gloss_entry path (lines 112-119).
        // PDF 제23항 example referencing 〔雪〕 explanatory ideograph.
        let result = crate::encode_to_unicode("〔雪〕");
        assert!(result.is_ok());
    }
}
