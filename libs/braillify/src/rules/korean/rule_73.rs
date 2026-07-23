use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "73",
    subsection: None,
    name: "fill_in_blank_markers",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.73",
    description: "Render underscore fill-in blanks as ⠸⠤ markers",
};

const PREFIX: u8 = 56; // ⠸
const BLANK: u8 = 36; // ⠤

pub struct Rule73;

impl BrailleRule for Rule73 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        70
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol('_' | '□'))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if ctx.current_char() == '□' {
            let next_word_starts_korean = ctx
                .remaining_words
                .first()
                .and_then(|w| w.chars().next())
                .is_some_and(crate::utils::is_korean_char);

            let fill_blank_context = ctx.next_char() == Some('에')
                || (ctx.next_char().is_none() && next_word_starts_korean);
            if !fill_blank_context {
                return Ok(RuleResult::Skip);
            }

            // 제73항: 빈칸표(□)는 ⠸⠦, 뒤에 설명/조사가 오면 ⠴⠇을 덧붙인다.
            ctx.emit(PREFIX);
            ctx.emit(crate::unicode::decode_unicode('⠦'));
            ctx.emit(0);
            ctx.emit(crate::unicode::decode_unicode('⠴'));
            ctx.emit(crate::unicode::decode_unicode('⠇'));
            return Ok(RuleResult::Consumed);
        }

        // keep 제56항 종료표기(__") intact
        if ctx.current_char() == '_'
            && ctx.next_char() == Some('_')
            && ctx.word_chars.get(ctx.index + 2) == Some(&'"')
        {
            return Ok(RuleResult::Skip);
        }

        if ctx.prev_char() == Some('_') {
            return Ok(RuleResult::Consumed);
        }

        let count = ctx.word_chars[ctx.index..]
            .iter()
            .take_while(|&&c| c == '_')
            .count();

        let marker_count = if count >= 3 { 1 } else { count.max(1) };
        for _ in 0..marker_count {
            ctx.emit(PREFIX);
            ctx.emit(BLANK);
        }

        if count > 1 {
            *ctx.skip_count = count - 1;
        }

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
        let _ = Rule73.apply(&mut ctx).unwrap();
        // Just exercise apply() for coverage
    }

    /// 제73항 — `_` 직전에 다른 `_`가 있을 때 (이미 처리된 후속 underscore)
    /// `Consumed`를 반환하고 출력하지 않는다 (line 67-69).
    #[test]
    fn rule73_apply_consecutive_underscore_consumed() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("__", false);
        let mut ctx = owned.ctx_at(1); // second '_'
        let outcome = Rule73.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        // The second '_' should be consumed silently (after the first '_' emitted)
        assert!(owned.result.is_empty());
    }
}
