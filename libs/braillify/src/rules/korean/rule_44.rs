//! 제44항 [다만] — 숫자와 혼동되는 'ㄴ, ㄷ, ㅁ, ㅋ, ㅌ, ㅍ, ㅎ'의 첫소리 글자와
//! '운'의 약자는 숫자 뒤에 붙어 나오더라도 숫자와 한글을 띄어 쓴다.
//!
//! When a Korean syllable starting with a "confusable" choseong (ㄴ,ㄷ,ㅁ,ㅋ,ㅌ,ㅍ,ㅎ)
//! or the syllable '운' follows a number, insert a space to prevent confusion.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 5, Section 11, Article 44 [다만]

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "44",
    subsection: Some("b1"),
    name: "number_korean_spacing",
    standard_ref: "2024 Korean Braille Standard, Ch.5 Sec.11 Art.44 [다만]",
    description: "Insert space between number and confusable Korean choseong",
};

/// Choseong characters that could be confused with digit braille patterns.
const CONFUSABLE_CHOSEONG: [char; 7] = ['ㄴ', 'ㄷ', 'ㅁ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];

/// Plugin struct for the rule engine.
///
/// Inserts a space (code 0) before Korean syllables with confusable choseong
/// when preceded by a number sequence. Runs in CoreEncoding at high priority
/// to insert the space BEFORE the Korean character is encoded.
pub struct Rule44;

impl BrailleRule for Rule44 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        50 // Very high — inserts space before any encoding of the Korean char
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        if !ctx.state.is_number {
            return false;
        }
        let CharType::Korean(korean) = ctx.char_type else {
            return false;
        };
        CONFUSABLE_CHOSEONG.contains(&korean.cho) || ctx.current_char() == '운'
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        // 한글 바로 앞 문자가 가운뎃점(`·`)인 경우에만 부착 분리자 ⠈(8)을 쓰고,
        // 그 외 (가운뎃점 열거 내부라도 한글이 숫자 다음에 나오는 경우 등)에는
        // 통상의 공백 ⠀(0)으로 분리한다.
        // 근거: 제44항 [다만] — 숫자와 혼동되는 한글은 띄어 쓴다. (제50항 가운뎃점
        // 열거의 부착 분리자는 `·` 바로 뒤에 한글이 붙은 형태에만 적용)
        let middle_dot_adjacent = ctx.prev_char() == Some('·');
        if middle_dot_adjacent {
            ctx.emit(8); // Attached separator
        } else {
            ctx.emit(0); // Space separator
        }
        Ok(RuleResult::Continue) // Continue to Korean encoding rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_confusable_choseong() {
        for &cho in &CONFUSABLE_CHOSEONG {
            assert!(
                CONFUSABLE_CHOSEONG.contains(&cho),
                "Missing confusable: {}",
                cho
            );
        }
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "44");
        assert_eq!(META.name, "number_korean_spacing");
    }

    /// 제44항 [다만] — 가운뎃점(·) 바로 뒤에 confusable 한글이 오면 부착 분리자
    /// ⠈(8)을 emit한다 (line 62-63). 가운뎃점 열거 컨텍스트.
    #[test]
    fn rule44_apply_emits_attached_separator_after_middle_dot() {
        // "·" + "ㅎ어" pattern — confusable choseong ㅎ after middle dot
        let word: Vec<char> = "·하".chars().collect();
        let ct = CharType::new(word[1]).unwrap();
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        state.is_number = true;
        let mut out = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word,
            index: 1,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: true,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule44.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Continue));
        assert_eq!(out, vec![8]); // attached separator
    }
}
