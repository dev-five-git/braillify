//! 제16항 — '까, 싸, 껏' 등 된소리 글자의 약자 처리.
//! 제17항 — '성, 썽, 정, 쩡, 청' 등 특정 종성 결합 글자.
//! 제14항 [붙임] — '팠'을 적을 때에는 'ㅏ'를 생략하지 않고 적는다.
//!
//! Exception characters that must be fully decomposed into choseong + jungseong + jongseong
//! rather than using abbreviation shortcuts. Handles: 팠, 껐, 셩, 쎵, 졍, 쪙, 쳥, 겄.
//!
//! Reference: 2024 Korean Braille Standard, Ch.2 Sec.6 Art.14 [붙임], Art.16 [붙임], Art.17

use crate::char_struct::CharType;
use crate::jauem::choseong::encode_choseong;
use crate::jauem::jongseong::encode_jongseong;
use crate::moeum::jungsong::encode_jungsong;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::split::split_korean_jauem;

pub static META: RuleMeta = RuleMeta {
    section: "16",
    subsection: None,
    name: "korean_exception_decomposition",
    standard_ref: "2024 Korean Braille Standard, Ch.2 Sec.6 Art.14[붙임]/16[붙임]/17",
    description: "Exception syllables (팠,껐,셩,쎵,졍,쪙,쳥,겄) fully decomposed",
};

/// The exception characters requiring full cho+jung+jong decomposition.
pub const EXCEPTION_CHARS: [char; 8] = ['팠', '껐', '셩', '쎵', '졍', '쪙', '쳥', '겄'];

/// Check if a character is in the exception list.
pub fn is_exception(ch: char) -> bool {
    EXCEPTION_CHARS.contains(&ch)
}

/// Plugin struct for the rule engine.
///
/// Intercepts exception Korean characters BEFORE abbreviation lookup (rule_13).
/// These characters must be fully decomposed: 된소리표 (if double cho) + base cho + jung + jong.
pub struct Rule16;

impl BrailleRule for Rule16 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        70 // Before rule_14 (80) and rule_13 (90) — intercepts exception chars first
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Korean(_)) && is_exception(ctx.current_char())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Korean(korean) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };
        let (cho0, cho1) = split_korean_jauem(korean.cho)?;
        if cho1.is_some() {
            // 된소리표 for double initial consonant
            ctx.emit(32); // ⠠
        }
        ctx.emit(encode_choseong(cho0)?);
        ctx.emit_slice(encode_jungsong(korean.jung)?);
        if let Some(jong) = korean.jong {
            ctx.emit_slice(encode_jongseong(jong)?);
        }
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_all_exception_chars() {
        for &ch in &EXCEPTION_CHARS {
            assert!(is_exception(ch), "Expected {} to be exception", ch);
        }
    }

    #[rstest::rstest]
    #[case::ga('가')]
    #[case::na('나')]
    #[case::seong_not_syeong('성')] // 성 is NOT exception — 셩 is
    #[case::jeong_not_jyeong('정')] // 정 is NOT exception — 졍 is
    fn rejects_non_exception_chars(#[case] ch: char) {
        assert!(!is_exception(ch));
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "16");
        assert_eq!(META.name, "korean_exception_decomposition");
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule16.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
