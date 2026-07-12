//! 제14항 — '나, 다, 마, 바, 자, 카, 타, 파, 하'에 모음이 붙어 나올 때에는 약자를 사용하지 않는다.
//!
//! When any of the 9 abbreviated syllables (나,다,마,바,자,카,타,파,하) is followed by
//! a syllable starting with silent ㅇ (i.e., vowel-initial), the abbreviation is NOT used.
//! Instead, the syllable is fully decomposed into choseong + jungseong.
//!
//! 된소리(쌍자음) 변형(따, 빠, 짜)도 동일하게 적용된다. 'ㄸ/ㅃ/ㅉ + ㅏ'는 인코더에서
//! `된소리표 + 다/바/자 약자`로 압축되는데, 같은 모음 환경에서는 압축을 피해야 한다.
//! 가는 제14항 본문에서 제외되므로 까(쌍자음 가)도 그대로 약자를 사용한다.
//! 사 역시 본문에 없으므로 싸도 마찬가지다.
//!
//! Note: 가 is not in this list (가 always uses abbreviation).
//!
//! Reference: 2024 Korean Braille Standard, Chapter 2, Section 6, Article 14

use crate::char_struct::CharType;
use crate::jauem::choseong::encode_choseong;
use crate::moeum::jungsong::encode_jungsong;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::split::split_korean_jauem;
use crate::utils::has_choseong_o;

pub static META: RuleMeta = RuleMeta {
    section: "14",
    subsection: None,
    name: "no_abbrev_before_vowel",
    standard_ref: "2024 Korean Braille Standard, Ch.2 Sec.6 Art.14",
    description: "나,다,마,바,자,카,타,파,하 followed by vowel-initial syllable: no abbreviation",
};

/// The 9 syllables subject to this rule.
/// These syllables use abbreviation EXCEPT when followed by a vowel-initial syllable.
pub const NO_ABBREV_SYLLABLES: [char; 9] = ['나', '다', '마', '바', '자', '카', '타', '파', '하'];

/// 된소리 변형: 약자가 적용되는 9자 중 단순 자음이 쌍자음(된소리)으로 바뀐 음절.
/// 다→따, 바→빠, 자→짜만 표준 한글 음절로 존재한다(ㄴ/ㅁ/ㅋ/ㅌ/ㅍ/ㅎ 은 된소리 없음).
const NO_ABBREV_DOUBLE_BASES: [char; 3] = ['다', '바', '자'];

/// When true, the encoder should use full decomposition (choseong + jungseong)
/// instead of the abbreviation shortcut.
#[cfg(test)]
fn should_suppress_abbreviation(current: char, next_has_choseong_o: bool) -> bool {
    is_no_abbrev_target(current) && next_has_choseong_o
}

/// Check if a character is subject to the no-abbreviation rule.
///
/// Returns true for both:
/// 1. The 9 base syllables 나~하.
/// 2. The 된소리 변형 따/빠/짜 — these decompose to 된소리표(⠠) + 다/바/자 약자,
///    and the same vowel-environment suppression must apply.
pub fn is_no_abbrev_target(ch: char) -> bool {
    if NO_ABBREV_SYLLABLES.contains(&ch) {
        return true;
    }
    // 된소리 + ㅏ 음절인지 확인: chosung이 쌍자음이고, 단순화한 (chosung 단자음 + ㅏ)이
    // NO_ABBREV_DOUBLE_BASES에 들어가는 경우.
    let code = ch as u32;
    if !(0xAC00..=0xD7A3).contains(&code) {
        return false;
    }
    let uni = code - 0xAC00;
    let cho_idx = (uni / 588) as usize;
    let jung_idx = ((uni - (cho_idx as u32 * 588)) / 28) as usize;
    let jong_idx = (uni % 28) as usize;
    // 종성 있음 → 약자 대상 아님 (NO_ABBREV_SYLLABLES는 모두 종성 없는 음절)
    if jong_idx != 0 {
        return false;
    }
    // ㅏ만 처리(NO_ABBREV_SYLLABLES 모두 jungsong=ㅏ).
    if jung_idx != 0 {
        return false;
    }
    const CHOSEONG: [char; 19] = [
        'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ',
        'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];
    let cho = CHOSEONG[cho_idx];
    if let Ok((cho0, Some(cho1))) = split_korean_jauem(cho)
        && cho0 == cho1
    {
        // 쌍자음. 단자음 버전 음절을 합성한다: 단자음 + ㅏ + (종성 없음).
        // 단자음의 CHOSEONG 인덱스를 찾아 단순화 음절을 만든다.
        if let Some(simple_cho_idx) = CHOSEONG.iter().position(|c| *c == cho0) {
            let simple_uni = (simple_cho_idx as u32) * 588;
            let simple_char = char::from_u32(0xAC00 + simple_uni).unwrap_or('가');
            return NO_ABBREV_DOUBLE_BASES.contains(&simple_char);
        }
    }
    false
}

/// Plugin struct for the rule engine.
///
/// Suppresses abbreviation for 나,다,마,바,자,카,타,파,하 when followed
/// by a vowel-initial syllable (제14항). Emits full decomposition instead.
/// Runs at higher priority than rule_13 so it intercepts before abbreviation.
pub struct Rule14;

impl BrailleRule for Rule14 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        80 // Before rule_13 (priority 90) — intercepts abbreviation
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        if !matches!(ctx.char_type, CharType::Korean(_)) {
            return false;
        }
        if !is_no_abbrev_target(ctx.current_char()) {
            return false;
        }
        // Check if next character starts with ㅇ (vowel-initial)
        ctx.index < ctx.word_chars.len() - 1 && has_choseong_o(ctx.word_chars[ctx.index + 1])
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Korean(korean) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };
        // Full decomposition: 된소리표(필요 시) + choseong + jungseong (약자 사용 금지)
        let (cho0, cho1) = split_korean_jauem(korean.cho)?;
        if cho1.is_some() {
            // 쌍자음(된소리) 음절: 된소리표(⠠) 먼저 emit
            ctx.emit(32);
        }
        let cho_code = encode_choseong(cho0)?;
        ctx.emit(cho_code);
        ctx.emit_slice(encode_jungsong(korean.jung)?);
        Ok(RuleResult::Consumed)
    }
}

/// Check if this syllable should suppress its abbreviation.
///
/// Returns true when:
/// 1. Current char is one of the 9 target syllables
/// 2. Next char is a Korean syllable starting with ㅇ (vowel-initial)
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_all_target_syllables() {
        for &ch in &NO_ABBREV_SYLLABLES {
            assert!(is_no_abbrev_target(ch), "Expected {} to be target", ch);
        }
    }

    #[test]
    fn ga_is_not_target() {
        // 가 is NOT in the list — it always uses abbreviation
        assert!(!is_no_abbrev_target('가'));
    }

    /// rule_14 line 56 — `is_no_abbrev_target` returns false for non-Hangul char.
    /// Triggers the `!(0xAC00..=0xD7A3).contains(&code)` branch.
    #[rstest::rstest]
    #[case('A')]
    #[case('1')]
    #[case(' ')]
    fn is_no_abbrev_target_non_hangul_returns_false(#[case] ch: char) {
        assert!(!is_no_abbrev_target(ch));
    }

    #[test]
    fn suppresses_when_next_is_vowel_initial() {
        assert!(should_suppress_abbreviation('나', true));
        assert!(should_suppress_abbreviation('다', true));
        assert!(should_suppress_abbreviation('하', true));
    }

    #[test]
    fn does_not_suppress_when_next_is_consonant_initial() {
        assert!(!should_suppress_abbreviation('나', false));
        assert!(!should_suppress_abbreviation('하', false));
    }

    #[test]
    fn does_not_suppress_for_non_target() {
        assert!(!should_suppress_abbreviation('가', true));
        assert!(!should_suppress_abbreviation('곤', true));
    }

    use rstest::rstest;

    /// Rule14 detects no-abbreviation target syllables followed by ㅇ-initial syllable.
    /// e.g., "사아" — 사 is target, next 아 starts with ㅇ.
    #[rstest]
    #[case("나아", true)] // 나 is target, next 아 has ㅇ-initial
    #[case("다어", true)]
    #[case("자아", true)]
    #[case("하이", true)]
    #[case("가나", false)] // 가 not in NO_ABBREV
    #[case("나람", false)] // 람 doesn't start with ㅇ
    #[case("A", false)] // not Korean
    fn rule14_matches_target_then_o_initial(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule14.matches(&ctx), expected, "input={input}");
    }

    #[test]
    fn rule14_apply_emits_for_target() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("나아", false);
        let mut ctx = owned.ctx_at(0);
        let _ = Rule14.apply(&mut ctx).unwrap();
        assert!(!owned.result.is_empty());
    }

    #[test]
    fn rule14_apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule14.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
