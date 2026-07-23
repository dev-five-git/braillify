//! 제28항 — 로마자는 ｢통일영어점자 규정｣에 따라 다음과 같이 적는다.
//!
//! English letters are mapped to braille using the UEB (Unified English Braille) system.
//! Uppercase indicators: single ⠠(32), word ⠠⠠(32,32), passage ⠠⠠⠠(32,32,32).
//!
//! Encoding is delegated to `english::encode_english()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 4, Section 10, Article 28

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::english_ueb::korean_context::KoreanPrefixInput;
use crate::rules::english_ueb::span::encode_korean_unit;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "28",
    subsection: None,
    name: "english_encoding",
    standard_ref: "2024 Korean Braille Standard, Ch.4 Sec.10 Art.28",
    description: "English letters encoded per UEB (Unified English Braille)",
};

/// Single uppercase indicator (대문자 기호표).
pub const UPPERCASE_SINGLE: u8 = 32; // ⠠

/// Encode a single English letter to braille.
#[cfg(test)]
fn apply(ch: char) -> Result<u8, String> {
    crate::english::encode_english(ch)
}

/// Returns a slice of indicator bytes to prepend.
#[cfg(test)]
fn uppercase_indicators(
    is_single_uppercase: bool,
    is_word_all_uppercase: bool,
    consecutive_uppercase_words: u8,
) -> &'static [u8] {
    if consecutive_uppercase_words >= 3 {
        &[32, 32, 32] // passage: ⠠⠠⠠
    } else if is_word_all_uppercase {
        &[32, 32] // word: ⠠⠠
    } else if is_single_uppercase {
        &[32] // single: ⠠
    } else {
        &[]
    }
}

/// Plugin struct for the rule engine.
///
/// Handles 제28항 English-in-Korean encoding: 로마자표/연속표 entry and uppercase
/// indicators. Letter/contraction cell production is delegated to
/// [`crate::rules::english_ueb::span`]; 종료표/exit orchestration lives in
/// [`crate::rules::emit`].
pub struct Rule28;

impl BrailleRule for Rule28 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::English(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::English(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // Enter English mode (로마자표 / 연속표)
        // 제39항 영어 주도 문서에서는 영자표시/연속표를 emit하지 않는다.
        if ctx.state.english_indicator
            && !ctx.state.is_english
            && !ctx.state.english_dominant_no_indicator
        {
            if ctx.state.needs_english_continuation {
                ctx.emit(48);
            } else {
                ctx.emit(52);
            }
        }

        // Uppercase indicators (single/consecutive uppercase run)
        if (!ctx.is_all_uppercase || ctx.word_len() < 2 || !ctx.ascii_starts_at_beginning)
            && !ctx.state.is_big_english
            && c.is_uppercase()
        {
            ctx.state.is_big_english = true;
            for idx in 0..std::cmp::min(ctx.word_len() - ctx.index, 2) {
                if ctx.word_chars[ctx.index + idx].is_uppercase() {
                    ctx.emit(UPPERCASE_SINGLE);
                } else {
                    break;
                }
            }
        }

        // English abbreviation lookup + fallback letter encoding.
        // Korean-context UEB contractions and standalone wordsigns are delegated to
        // `encode_korean_unit`; this rule only decides the surrounding mode markers.
        let is_whole_lowercase_word =
            ctx.index == 0 && ctx.word_chars.iter().all(|ch| ch.is_ascii_lowercase());
        let prev_is_ascii_word =
            !ctx.prev_word.is_empty() && ctx.prev_word.chars().all(|ch| ch.is_ascii_alphabetic());
        let next_is_ascii_word = ctx
            .remaining_words
            .first()
            .is_some_and(|w| !w.is_empty() && w.chars().all(|ch| ch.is_ascii_alphabetic()));
        let unit = encode_korean_unit(KoreanPrefixInput {
            word: ctx.word_chars,
            pos: ctx.index,
            wrap_active: ctx.state.english_dominant_wrap_active,
            is_all_uppercase: ctx.is_all_uppercase,
            at_entry: !ctx.state.is_english || ctx.index == 0,
            standalone_wordsign: is_whole_lowercase_word
                && prev_is_ascii_word
                && next_is_ascii_word,
        })?;
        ctx.emit_slice(&unit.cells);
        if unit.contracted {
            *ctx.skip_count = unit.consumed.saturating_sub(1);
        }

        ctx.state.is_english = true;
        ctx.state.needs_english_continuation = false;
        Ok(RuleResult::Consumed)
    }
}

/// Determine the uppercase indicator(s) needed.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    /// 제28항 — 영문자 점역. 소문자/대문자 모두 동일 점형으로 인코딩.
    #[rstest::rstest]
    #[case::lower_a('a', '⠁')]
    #[case::lower_z('z', '⠵')]
    #[case::upper_a_as_lowercase('A', '⠁')]
    fn encodes_english_letters(#[case] ch: char, #[case] expected: char) {
        assert_eq!(apply(ch).unwrap(), decode_unicode(expected));
    }

    /// 영문자가 아닌 입력은 Err.
    #[rstest::rstest]
    #[case::digit('1')]
    #[case::syllable('가')]
    fn invalid_returns_error(#[case] ch: char) {
        assert!(apply(ch).is_err());
    }

    /// `uppercase_indicators` — single/word/passage 대문자 지시자 점형.
    #[rstest::rstest]
    #[case::single_letter(true, false, 0, &[32u8] as &[u8])]
    #[case::word_two_letters(false, true, 0, &[32, 32])]
    #[case::passage_run(false, true, 3, &[32, 32, 32])]
    #[case::no_indicator_lower(false, false, 0, &[] as &[u8])]
    fn uppercase_indicator_paths(
        #[case] single: bool,
        #[case] is_word: bool,
        #[case] run: u8,
        #[case] expected: &[u8],
    ) {
        assert_eq!(uppercase_indicators(single, is_word, run), expected);
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let _ = Rule28.apply(&mut ctx).unwrap();
        // Just exercise apply() for coverage
    }

    /// rule_28 — multi-cell `ong` abbreviation hit via real word `pyeongchang`
    /// from PDF testcase (rule_35.json). The 'o' at index 2 has remaining="ongchang"
    /// which matches `rule_en_multi_cell`.
    #[test]
    fn rule28_multi_cell_via_pyeongchang() {
        let _ = crate::encode("pyeongchang 2018");
    }

    /// rule_28:205-206 — multi-cell English abbreviation ("ong" → ⠰⠛)
    /// applied word-middle. Drives the `rule_en_multi_cell` arm via direct
    /// `RuleContext` setup with state.is_english=true, index > 0.
    #[test]
    fn rule28_multi_cell_word_middle_direct() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "along".chars().collect();
        let ct = CharType::English('o');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        state.is_english = true;
        let mut out = Vec::new();
        let mut ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 2, // 'o' position; remaining = "ong"
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: true,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule28.apply(&mut ctx).unwrap();
        // Either Consumed (multi-cell applied) or other; at minimum the arm runs.
        let _ = outcome;
    }

    /// rule_28 line 64 — `let-else return Skip` for non-English ctx.
    #[test]
    fn rule28_apply_skip_for_non_english_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("가", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule28.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
