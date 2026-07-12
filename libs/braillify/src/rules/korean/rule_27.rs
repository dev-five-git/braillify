use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::EncodingMode;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "27",
    subsection: None,
    name: "middle_korean_tone_marks",
    standard_ref: "2024 Korean Braille Standard, Ch.3 Art.27",
    description: "Middle Korean tone marks: 거성(·), 상성(：)",
};

const GEOSEONG: [u8; 2] = [
    crate::unicode::decode_unicode('⠸'),
    crate::unicode::decode_unicode('⠂'),
];
const SANGSEONG: [u8; 2] = [
    crate::unicode::decode_unicode('⠸'),
    crate::unicode::decode_unicode('⠅'),
];

fn is_historical_context_word(word: &str) -> bool {
    word.chars().any(|c| {
        let code = c as u32;
        (0xE000..=0xF8FF).contains(&code)
            || (0x4E00..=0x9FFF).contains(&code)
            || matches!(c, '：' | '〔' | '〕')
    })
}

fn has_historical_context(ctx: &RuleContext) -> bool {
    if is_historical_context_word(&ctx.word_chars.iter().collect::<String>()) {
        return true;
    }
    if is_historical_context_word(ctx.prev_word) {
        return true;
    }
    ctx.remaining_words
        .iter()
        .take(2)
        .any(|word| is_historical_context_word(word))
}

fn is_middle_korean_geoseong(ctx: &RuleContext) -> bool {
    if !matches!(ctx.char_type, CharType::Symbol('·')) {
        return false;
    }

    // 단독 입력 `·`은 한국어 점자에서 두 가지 의미를 가진다:
    //   - 일반 한국어(가운뎃점, 제49항): ⠐⠆ — rule_49가 처리
    //   - 중세국어(거성, 제27항): ⠸⠂ — 이 규칙이 처리
    // 두 의미는 동일 입력으로 구분할 수 없으므로 EncodingMode::MiddleKorean이
    // 명시된 경우에만 거성으로 해석한다.
    if ctx.word_len() == 1 {
        return ctx.state.current_mode() == EncodingMode::MiddleKorean;
    }

    if ctx.prev_char().is_some_and(|prev| prev.is_ascii_digit())
        && ctx.next_char().is_some_and(|next| next.is_ascii_digit())
    {
        return false;
    }

    (ctx.index == 0 && ctx.next_char().is_some()) || has_historical_context(ctx)
}

fn is_middle_korean_particle_geoseong(ctx: &RuleContext) -> bool {
    matches!(ctx.char_type, CharType::Symbol('·'))
        && ctx.state.current_mode() == EncodingMode::MiddleKorean
        && ctx.next_char() == Some('에')
}

fn is_inline_gloss_separator(ctx: &RuleContext) -> bool {
    matches!(ctx.char_type, CharType::Symbol('·'))
        && ctx.state.current_mode() == EncodingMode::MiddleKorean
        && ctx.prev_char() == Some('字')
        && ctx.next_char() == Some('')
}

fn is_middle_korean_sangseong(ctx: &RuleContext) -> bool {
    matches!(ctx.char_type, CharType::Symbol('：'))
}

pub struct Rule27;

impl BrailleRule for Rule27 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        51
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        let is_potential_tone_mark = matches!(ctx.char_type, CharType::Symbol('·' | '：'));
        if !is_potential_tone_mark {
            return false;
        }

        if ctx.state.current_mode() == EncodingMode::MiddleKorean {
            return true;
        }

        is_middle_korean_geoseong(ctx) || is_middle_korean_sangseong(ctx)
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        match c {
            '·' if is_inline_gloss_separator(ctx) => {}
            '·' if is_middle_korean_particle_geoseong(ctx) => {
                ctx.emit(0);
                ctx.emit_slice(&GEOSEONG);
            }
            '·' if ctx.state.current_mode() == EncodingMode::MiddleKorean => {
                ctx.emit_slice(&GEOSEONG);
            }
            '·' if is_middle_korean_geoseong(ctx) => ctx.emit_slice(&GEOSEONG),
            '：' => ctx.emit_slice(&SANGSEONG),
            _ => return Ok(RuleResult::Skip),
        }

        Ok(RuleResult::Consumed)
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
        let _ = Rule27.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule27.matches(&ctx);
    }

    /// 제27항 — `has_historical_context` returns true when current word contains
    /// a hanja character (CJK Unified). Exercises lines 33-35 (own-word branch).
    #[test]
    fn has_historical_context_via_own_word() {
        // "·君" — first cell is '·', second '君' (CJK).
        let mut owned = crate::test_helpers::CtxOwned::for_text("·君", false);
        let ctx = owned.ctx_at(0);
        assert!(has_historical_context(&ctx));
    }

    #[rstest::rstest]
    #[case::private_use('\u{E000}')]
    #[case::hanja('君')]
    #[case::gloss_open('〔')]
    #[case::gloss_close('〕')]
    fn historical_context_word_detects_marker_chars(#[case] marker: char) {
        assert!(is_historical_context_word(&marker.to_string()));
    }

    /// 제27항 — `has_historical_context` returns true when prev_word contains
    /// a hanja. Exercises lines 36-38 (prev_word branch).
    #[test]
    fn has_historical_context_via_prev_word() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("·", false).with_prev_word("君");
        let ctx = owned.ctx_at(0);
        assert!(has_historical_context(&ctx));
    }

    /// 제27항 — `has_historical_context` returns true when a remaining_word
    /// contains a hanja (within the first two). Exercises lines 40-43.
    #[test]
    fn has_historical_context_via_remaining_word() {
        let mut owned =
            crate::test_helpers::CtxOwned::for_text("·", false).with_remaining_words(["君"]);
        let ctx = owned.ctx_at(0);
        assert!(has_historical_context(&ctx));
    }

    /// 제27항 — `：` (sangseong/상성) in any context emits the SANGSEONG cells.
    #[test]
    fn apply_emits_sangseong_for_full_width_colon() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("：", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule27.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result, SANGSEONG.to_vec());
    }

    /// 제27항 — In MiddleKorean mode, single-cell `·` emits GEOSEONG cells.
    /// Triggers the `current_mode() == MiddleKorean` arm in apply (line 125-127).
    #[test]
    fn apply_emits_geoseong_in_middle_korean_mode() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("·", false);
        owned.state.push_mode(EncodingMode::MiddleKorean);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule27.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result, GEOSEONG.to_vec());
    }

    /// rule_27 line 106 — `_ => return Ok(Skip)` fallback for non-· non-: symbol char.
    #[test]
    fn rule27_apply_skip_for_unrelated_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("!", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule27.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
