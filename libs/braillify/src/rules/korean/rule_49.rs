//! 제49항 — 문장 부호는 다음과 같이 적는다.
//!
//! Symbol/punctuation encoding via `symbol_shortcut::encode_char_symbol_shortcut()` (PHF).
//! Includes: period, comma, question mark, exclamation, quotes, brackets, etc.
//!
//! English-specific symbol variants via `symbol_shortcut::encode_english_char_symbol_shortcut()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 6, Section 13, Article 49

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::symbol_shortcut;
use crate::unicode::decode_unicode;

pub static META: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "punctuation_encoding",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.49",
    description: "Punctuation marks encoded to braille dot patterns",
};

/// Encode a punctuation/symbol character to braille (Korean context).
#[cfg(test)]
fn apply(ch: char) -> Result<&'static [u8], String> {
    symbol_shortcut::encode_char_symbol_shortcut(ch)
}

/// Encode a punctuation/symbol in English context (different dot patterns for (, ), ,).
#[cfg(test)]
fn apply_english(ch: char) -> Option<Vec<u8>> {
    symbol_shortcut::encode_english_char_symbol_shortcut(ch)
}

/// Check if a character is a recognized symbol.
#[cfg(test)]
fn is_symbol(ch: char) -> bool {
    symbol_shortcut::is_symbol_char(ch)
}

/// Plugin struct for the rule engine.
///
/// Handles the base case of symbol/punctuation encoding (제49항).
/// Special cases (comma in numbers, blank marks, asterisks) are handled
/// by dedicated rules (rule_41, rule_58, rule_60) which run at higher priority.
pub struct Rule49;

impl BrailleRule for Rule49 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        500 // Low priority — fallback after special-case symbol rules
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // 제49항 [붙임]: 문장 부호를 낱말처럼 설명할 때
        // 물음표는 기호표(⠸) + 해당 기호 + 점역자 주표(⠠⠄ ... ⠠⠄)를 사용한다.
        if *c == '?' && ctx.index == 0 {
            let prev_word_is_korean = ctx.prev_word.chars().any(crate::utils::is_korean_char);
            let next_word_is_korean = ctx
                .remaining_words
                .first()
                .is_some_and(|w| w.chars().any(crate::utils::is_korean_char));

            if !ctx.has_korean_char && !prev_word_is_korean && !next_word_is_korean {
                let encoded = symbol_shortcut::encode_char_symbol_shortcut(*c)?;
                ctx.emit_slice(encoded);
                return Ok(RuleResult::Consumed);
            }

            let next_is_korean_or_end = ctx.next_char().is_none_or(crate::utils::is_korean_char);
            if next_is_korean_or_end {
                ctx.emit(decode_unicode('⠸'));
                let encoded = symbol_shortcut::encode_char_symbol_shortcut(*c)?;
                ctx.emit_slice(encoded);
                ctx.emit(0);
                ctx.emit(decode_unicode('⠠'));
                ctx.emit(decode_unicode('⠄'));
                // "물음표"
                ctx.emit_slice(&[
                    decode_unicode('⠑'),
                    decode_unicode('⠯'),
                    decode_unicode('⠪'),
                    decode_unicode('⠢'),
                    decode_unicode('⠙'),
                    decode_unicode('⠬'),
                ]);
                ctx.emit(decode_unicode('⠠'));
                ctx.emit(decode_unicode('⠄'));
                return Ok(RuleResult::Consumed);
            }
        }

        // ASCII apostrophe context-sensitive open/close rendering.
        // open:  ⠠⠦, close: ⠴⠄
        if *c == '\'' {
            let is_close = ctx.prev_char().is_some();
            if is_close {
                ctx.emit_slice(&[decode_unicode('⠴'), decode_unicode('⠄')]);
            } else {
                ctx.emit_slice(&[decode_unicode('⠠'), decode_unicode('⠦')]);
            }
            return Ok(RuleResult::Consumed);
        }

        // ASCII double quote context-sensitive open/close rendering.
        // open: ⠦, close: ⠴
        if *c == '"' && ctx.next_char() != Some('˙') {
            let is_close = ctx.prev_char().is_some();
            if is_close {
                ctx.emit(decode_unicode('⠴'));
            } else {
                ctx.emit(decode_unicode('⠦'));
            }
            return Ok(RuleResult::Consumed);
        }

        // 제56항 입력 표기(인쇄 부호 잔존) 호환:
        // "˙,  __" 형태를 강조 시작/종결 표기로 해석한다.
        if *c == '"' && ctx.next_char() == Some('˙') {
            ctx.emit_slice(&[decode_unicode('⠠'), decode_unicode('⠤')]);
            *ctx.skip_count = 2; // skip ˙,
            return Ok(RuleResult::Consumed);
        }
        if *c == '_'
            && ctx.next_char() == Some('_')
            && ctx.word_chars.get(ctx.index + 2) == Some(&'"')
        {
            ctx.emit_slice(&[decode_unicode('⠤'), decode_unicode('⠄')]);
            *ctx.skip_count = 2; // skip _"
            return Ok(RuleResult::Consumed);
        }

        if *c == '×'
            && ctx.word_len() == 1
            && ctx.prev_word.is_empty()
            && ctx.remaining_words.is_empty()
        {
            ctx.emit_slice(&[
                decode_unicode('⠸'),
                decode_unicode('⠭'),
                decode_unicode('⠇'),
            ]);
            return Ok(RuleResult::Consumed);
        }

        let encoded = symbol_shortcut::encode_char_symbol_shortcut(*c)?;
        ctx.emit_slice(encoded);

        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    /// 제49항 — 기본 문장부호 점형.
    #[rstest::rstest]
    #[case::period('.', vec!['⠲'])]
    #[case::comma(',', vec!['⠐'])]
    #[case::question('?', vec!['⠦'])]
    #[case::exclamation('!', vec!['⠖'])]
    #[case::open_paren('(', vec!['⠦', '⠄'])]
    #[case::close_paren(')', vec!['⠠', '⠴'])]
    fn encodes_basic_punctuation(#[case] ch: char, #[case] expected_unicode: Vec<char>) {
        let expected: Vec<u8> = expected_unicode.into_iter().map(decode_unicode).collect();
        assert_eq!(apply(ch).unwrap(), expected.as_slice());
    }

    #[test]
    fn english_parentheses_different() {
        let eng = apply_english('(').unwrap();
        let kor = apply('(').unwrap();
        assert_ne!(
            eng.as_slice(),
            kor,
            "English and Korean parentheses should differ"
        );
    }

    /// `is_symbol` — 문장부호 / 괄호는 true, 영문/한글은 false.
    #[rstest::rstest]
    #[case::period('.', true)]
    #[case::question('?', true)]
    #[case::open_paren('(', true)]
    #[case::ascii_letter('A', false)]
    #[case::korean_syllable('가', false)]
    fn is_symbol_detection(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(is_symbol(ch), expected);
    }

    #[test]
    fn unknown_symbol_returns_error() {
        assert!(apply('@').is_err());
    }

    use rstest::rstest;

    #[rstest]
    #[case("?", true)] // Symbol
    #[case("'", true)] // Symbol (apostrophe)
    #[case("\"", true)] // Symbol (double quote)
    #[case("A", false)] // English
    #[case("가", false)] // Korean syllable
    fn rule49_matches_symbols(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule49.matches(&ctx), expected, "input={input}");
    }

    /// Single '×' in isolation — × is MathSymbol, so Rule49 (Symbol matcher) skips.
    /// Just exercise the apply path for coverage.
    #[test]
    fn rule49_x_apply_exercises_path() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("×", false);
        let mut ctx = owned.ctx_at(0);
        let _ = Rule49.apply(&mut ctx);
    }

    /// Opening apostrophe at start of word.
    #[test]
    fn rule49_apostrophe_open() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("'", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result, vec![decode_unicode('⠠'), decode_unicode('⠦')]);
    }

    /// Closing apostrophe (preceded by another char).
    #[test]
    fn rule49_apostrophe_close() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A'", false);
        let mut ctx = owned.ctx_at(1); // ' at index 1 (preceded by A)
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result, vec![decode_unicode('⠴'), decode_unicode('⠄')]);
    }

    /// Opening double quote at start.
    #[test]
    fn rule49_doublequote_open() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("\"", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result, vec![decode_unicode('⠦')]);
    }

    /// Apply skips non-symbol char_type.
    #[test]
    fn rule49_apply_skips_non_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제49항 [붙임] — 물음표(?)가 어절 처음에 한국어 컨텍스트로 나오면
    /// 기호 설명 점역 ⠸⠦…⠠⠄물음표⠠⠄ 형태를 emit (lines 87-107).
    /// `next_is_korean_or_end` 분기.
    #[test]
    fn rule49_question_mark_in_korean_context_descriptive() {
        let word_chars = ['?'];
        let char_type = CharType::Symbol('?');
        let mut skip_count = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word_chars,
            index: 0,
            char_type: &char_type,
            prev_word: "가",
            remaining_words: &["가"],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip_count,
            state: &mut state,
            result: &mut result,
        };
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        // Descriptive output is multi-cell ⠸⠦ ... 물음표 ... ⠠⠄
        assert!(result.len() > 5);
    }

    /// 제49항 — 단독 `×` (한 글자 단어, 인접 단어 없음)는 ⠸⠭⠇로 점역
    /// (lines 150-161).
    #[test]
    fn rule49_standalone_times_emits_object_symbol_form() {
        let word_chars = ['×'];
        let char_type = CharType::Symbol('×');
        let mut skip_count = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
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
        let outcome = Rule49.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(
            result,
            vec![
                decode_unicode('⠸'),
                decode_unicode('⠭'),
                decode_unicode('⠇'),
            ]
        );
    }
}
