//! English-context symbol handling.
//!
//! Handles symbol behavior that depends on English mode state:
//! - English symbol rendering for (, ), , when context requires
//! - Parenthesis stack push/pop for matching English parentheses
//! - Comma before Korean fallback preservation

use crate::char_struct::CharType;
use crate::english_logic;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::symbol_shortcut;
use crate::utils;

pub static META: RuleMeta = RuleMeta {
    section: "49",
    subsection: Some("eng"),
    name: "english_symbol_context",
    standard_ref: "2024 Korean Braille Standard, Ch.4 Sec.10 + Ch.6 Sec.13",
    description: "English-context punctuation rendering with parenthesis tracking",
};

pub struct RuleEnglishSymbol;

impl BrailleRule for RuleEnglishSymbol {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        300
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(sym) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        let mut use_english_symbol = english_logic::should_render_symbol_as_english(
            ctx.state.english_indicator,
            ctx.state.is_english,
            &ctx.state.parenthesis_stack,
            *sym,
            ctx.word_chars,
            ctx.index,
            ctx.remaining_words,
        );

        // 제39항 영-한 wrap context: 단어 끝의 영어 모드 유지 가능 기호(. , : ;)
        // 다음에 한글 어절(wrap 대상)이 이어지면 그 기호를 영어 점자로 처리한다.
        // 예) "(Korean:" 끝의 ':'은 다음 wrap된 "반찬" 직전이므로 영어 점자 ⠒.
        if !use_english_symbol
            && ctx.state.english_dominant_wrap_active
            && ctx.state.is_english
            && ctx.index == ctx.word_chars.len() - 1
            && matches!(*sym, '.' | ',' | ':' | ';')
            && let Some(next_word) = ctx.remaining_words.first()
            && next_word.chars().next().is_some_and(utils::is_korean_char)
        {
            use_english_symbol = true;
        }

        if *sym == '(' {
            ctx.state.parenthesis_stack.push(use_english_symbol);
        } else if *sym == ')' {
            use_english_symbol = ctx
                .state
                .parenthesis_stack
                .pop()
                .unwrap_or(use_english_symbol);
        }

        let has_ascii_alphabetic = ctx.word_chars.iter().any(|ch| ch.is_ascii_alphabetic());
        let can_use_english_symbol = ctx.state.is_english || has_ascii_alphabetic;

        if ctx.state.english_indicator && can_use_english_symbol && use_english_symbol {
            if !ctx.state.is_english && !ctx.state.needs_english_continuation {
                ctx.emit(52);
                ctx.state.is_english = true;
                ctx.state.needs_english_continuation = false;
            }
            if let Some(encoded) = symbol_shortcut::encode_english_char_symbol_shortcut(*sym) {
                ctx.emit_slice(&encoded);
                if *sym == '-' && ctx.state.is_english {
                    // 다음 글자가 숫자이면 수표(⠼)가 emit되므로 연속표(⠰)는
                    // 불필요하다 (제35항 D-100 같은 영문-숫자 인접 패턴).
                    let next_is_digit = ctx
                        .word_chars
                        .get(ctx.index + 1)
                        .is_some_and(|c| c.is_ascii_digit());
                    if !next_is_digit {
                        ctx.emit(crate::rules::korean::rule_29::ENGLISH_CONTINUATION);
                    }
                }
                return Ok(RuleResult::Consumed);
            }
        }

        if *sym == ',' {
            let next_char = ctx
                .next_char()
                .or_else(|| ctx.remaining_words.first().and_then(|w| w.chars().next()));
            if next_char.is_some_and(utils::is_korean_char) {
                ctx.emit_slice(symbol_shortcut::encode_char_symbol_shortcut(*sym)?);
                return Ok(RuleResult::Consumed);
            }
        }

        Ok(RuleResult::Continue)
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
        let _ = RuleEnglishSymbol.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = RuleEnglishSymbol.matches(&ctx);
    }

    #[test]
    fn opening_parenthesis_pushes_symbol_mode() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("(", false);
        let mut ctx = owned.ctx_at(0);

        let _ = RuleEnglishSymbol.apply(&mut ctx);

        assert!(!ctx.state.parenthesis_stack.is_empty());
    }

    #[test]
    fn closing_parenthesis_reuses_opening_parenthesis_symbol_mode() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("()", true);
        {
            let mut ctx = owned.ctx_at(0);
            ctx.state.is_english = true;

            let _ = RuleEnglishSymbol.apply(&mut ctx);
            assert_eq!(ctx.state.parenthesis_stack.len(), 1);
        }

        let mut ctx = owned.ctx_at(1);
        ctx.state.is_english = true;

        let _ = RuleEnglishSymbol.apply(&mut ctx);

        assert!(ctx.state.parenthesis_stack.is_empty());
    }
}
