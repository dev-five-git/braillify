//! 제58항 — 빠짐표(□)가 여러 개 붙어 나올 때에는 ⠸과 ⠶ 사이에
//! ⠶을 묵자의 개수만큼 적어 나타낸다.
//!
//! Blank marks (□) are encoded as: prefix ⠸(56) + count×⠶(54) + suffix ⠇(7).
//! Consecutive □ characters are consumed and encoded as a single group.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 6, Section 13, Article 58

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "58",
    subsection: None,
    name: "blank_marks",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Sec.13 Art.58",
    description: "Blank marks □: prefix ⠸ + count × ⠶ + suffix ⠇",
};

const BLANK_MARK: char = '□';
const PREFIX: u8 = 56; // ⠸
const MARK: u8 = 54; // ⠶
const SUFFIX: u8 = 7; // ⠇

/// Plugin struct for the rule engine.
///
/// Handles blank mark (□) encoding. Counts consecutive □ characters,
/// emits the grouped encoding, and sets skip_count to skip the consumed chars.
pub struct Rule58;

impl BrailleRule for Rule58 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        70 // Before rule_72 (80) and rule_49 (500) — intercept □ before other symbol rules
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        if !matches!(ctx.char_type, CharType::Symbol(c) if *c == BLANK_MARK) {
            return false;
        }

        // 단독 □ (앞뒤에 다른 단어 없음, word 자체도 1글자)는 rule_72/rule_15에 위임.
        let is_lone =
            ctx.word_len() == 1 && ctx.prev_word.is_empty() && ctx.remaining_words.is_empty();
        if is_lone {
            return false;
        }

        // 단어 안에서는 □가 2개 이상 연속될 때 (rule_58 본문) 적용.
        let count = ctx.word_chars[ctx.index..]
            .iter()
            .take_while(|&&c| c == BLANK_MARK)
            .count();
        count >= 2
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        // Count consecutive □ characters
        let count = ctx.word_chars[ctx.index..]
            .iter()
            .take_while(|&&c| c == BLANK_MARK)
            .count();

        ctx.emit(PREFIX);
        for _ in 0..count {
            ctx.emit(MARK);
        }
        ctx.emit(SUFFIX);

        // Skip the remaining □ characters (current one is already processed)
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
    fn multiple_blank_marks() {
        // □□□ → ⠸⠶⠶⠶⠇ (제58항: 2개 이상 연속될 때 묶음 표기)
        let result = crate::encode_to_unicode("□□□").unwrap();
        assert_eq!(result, "⠸⠶⠶⠶⠇");
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "58");
        assert_eq!(META.name, "blank_marks");
    }
}
