//! 제57항 — 가림표(○, ×, △, ☆, ◇, ◆)가 여러 개 붙어 나올 때에는
//! ⠸과 해당 기호 사이 점형을 묵자 개수만큼 적고 끝에 ⠇을 적는다.

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::utils;

pub static META: RuleMeta = RuleMeta {
    section: "57",
    subsection: None,
    name: "symbol_grouping",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.57",
    description: "Group repeated placeholder symbols with ⠸ ... ⠇",
};

const PREFIX: u8 = 56; // ⠸
const SUFFIX: u8 = 7; // ⠇

fn placeholder_mark(ch: char) -> Option<u8> {
    match ch {
        '○' => Some(52), // ⠴
        '×' => Some(45), // ⠭
        '△' => Some(44), // ⠬
        '☆' => Some(20), // ⠔
        '◇' => Some(34), // ⠢
        '◆' => Some(21), // ⠕
        _ => None,
    }
}

fn is_math_times_context(ctx: &RuleContext) -> bool {
    if ctx.current_char() != '×' {
        return false;
    }

    let prev = ctx.prev_char();
    let next = ctx.next_char();

    // 수식 문맥에서는 기존 수학 기호 규칙(RuleMath)을 유지한다.
    (prev.is_some_and(|c| c.is_ascii_digit()) && next.is_some_and(|c| c.is_ascii_digit()))
        || (prev.is_some_and(utils::is_korean_char) && next.is_some_and(utils::is_korean_char))
}

fn is_placeholder_times_context(ctx: &RuleContext) -> bool {
    if ctx.current_char() != '×' {
        return false;
    }

    if is_math_times_context(ctx) {
        return false;
    }

    // 연속된 ×, 또는 단독 시작(×란) 문맥은 가림표로 본다.
    ctx.prev_char().is_some_and(|c| c == '×')
        || ctx.next_char().is_some_and(|c| c == '×')
        || (ctx.prev_char().is_none() && ctx.next_char().is_some_and(utils::is_korean_char))
}

pub struct Rule57;

impl BrailleRule for Rule57 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        90 // Before rule_math(100), rule_58(400), and rule_49(500)
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        match ctx.char_type {
            CharType::Symbol(c) => placeholder_mark(*c).is_some(),
            CharType::MathSymbol('×') => is_placeholder_times_context(ctx),
            _ => false,
        }
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let current = ctx.current_char();

        // MathSymbol('×')인 경우에도 가림표 문맥이 아니면 RuleMath로 넘긴다.
        if current == '×' && !is_placeholder_times_context(ctx) {
            return Ok(RuleResult::Skip);
        }

        let Some(mark) = placeholder_mark(current) else {
            return Ok(RuleResult::Skip);
        };

        let count = ctx.word_chars[ctx.index..]
            .iter()
            .take_while(|&&c| c == current)
            .count();

        ctx.emit(PREFIX);
        for _ in 0..count {
            ctx.emit(mark);
        }
        ctx.emit(SUFFIX);

        if count > 1 {
            *ctx.skip_count = count - 1;
        }

        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 제57항 — `is_math_times_context` short-circuits when current char is not `×`
    /// (line 33-36).
    #[test]
    fn is_math_times_context_returns_false_for_non_times() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "a".chars().collect();
        let ct = CharType::Symbol('a');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        assert!(!is_math_times_context(&ctx));
    }

    /// 제57항 — `is_placeholder_times_context` short-circuits for non-`×` chars
    /// (line 46-49).
    #[test]
    fn is_placeholder_times_context_returns_false_for_non_times() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "a".chars().collect();
        let ct = CharType::Symbol('a');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        assert!(!is_placeholder_times_context(&ctx));
    }

    /// 제57항 — apply path where MathSymbol(×) is in non-placeholder context
    /// returns `Skip` (line 88-90).
    #[test]
    fn rule57_apply_math_times_context_returns_skip() {
        use crate::char_struct::CharType;
        // "5×3": × at idx 1, prev=5, next=3 → is_math_times_context = true
        let word: Vec<char> = "5×3".chars().collect();
        let ct = CharType::MathSymbol('×');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 1,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule57.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제57항 — apply path falls through to `placeholder_mark` returning None
    /// (line 92-94). Force by giving a Symbol char that isn't in placeholder_mark.
    #[test]
    fn rule57_apply_unknown_symbol_skips() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "a".chars().collect();
        let ct = CharType::Symbol('a');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule57.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
