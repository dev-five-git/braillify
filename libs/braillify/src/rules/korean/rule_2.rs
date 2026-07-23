//! 제2항 — 된소리 글자 'ㄲ, ㄸ, ㅃ, ㅆ, ㅉ'이 첫소리로 쓰일 때에는
//! 'ㄱ, ㄷ, ㅂ, ㅅ, ㅈ' 앞에 된소리표 ⠠을 적어 나타낸다.
//!
//! Double consonants as initial (choseong) are written as 된소리표 (⠠, code 32)
//! followed by the base consonant.
//!
//! The decomposition is handled by `split::split_korean_jauem()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 1, Section 1, Article 2

use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::split;

pub static META: RuleMeta = RuleMeta {
    section: "2",
    subsection: None,
    name: "double_choseong",
    standard_ref: "2024 Korean Braille Standard, Ch.1 Sec.1 Art.2",
    description: "Double consonants (ㄲ,ㄸ,ㅃ,ㅆ,ㅉ) as choseong: 된소리표 ⠠ + base consonant",
};

const DOUBLE_CONSONANT_INDICATOR: u8 = 32; // ⠠ (된소리표)

/// The 5 double consonants that trigger this rule.
pub const DOUBLE_CHOSEONG: [char; 5] = ['ㄲ', 'ㄸ', 'ㅃ', 'ㅆ', 'ㅉ'];

/// Check if a choseong is a double consonant.
pub fn is_double_choseong(cho: char) -> bool {
    DOUBLE_CHOSEONG.contains(&cho)
}

/// Decompose a double choseong into (된소리표, base consonant).
/// Returns None if not a double consonant.
pub fn decompose(cho: char) -> Option<(u8, char)> {
    if !is_double_choseong(cho) {
        return None;
    }
    let (base, _) = split::split_korean_jauem(cho).ok()?;
    Some((DOUBLE_CONSONANT_INDICATOR, base))
}

/// Plugin struct for the rule engine.
///
/// Sub-component rule: handles double consonant (된소리) choseong encoding.
/// When a Korean syllable has a double initial consonant (ㄲ,ㄸ,ㅃ,ㅆ,ㅉ),
/// this rule emits 된소리표 ⠠ followed by the base consonant code.
/// Returns Continue for jungseong/jongseong processing.
pub struct Rule2;

impl BrailleRule for Rule2 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        195 // Sub-component — runs before Rule1 (200) for double consonant check
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        ctx.as_korean().is_some_and(|k| is_double_choseong(k.cho))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let Some(korean) = ctx.as_korean() else {
            return Ok(RuleResult::Skip);
        };
        if let Some((indicator, _base)) = decompose(korean.cho) {
            ctx.emit(indicator);
        }
        Ok(RuleResult::Continue) // Continue to Rule1 for base consonant + jungseong/jongseong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `is_double_choseong` — 된소리(ㄲㄸㅃㅆㅉ)는 true, 평음/격음은 false.
    #[rstest::rstest]
    #[case::ssang_giyeok('ㄲ', true)]
    #[case::ssang_digeut('ㄸ', true)]
    #[case::ssang_bieup('ㅃ', true)]
    #[case::ssang_siot('ㅆ', true)]
    #[case::ssang_jieut('ㅉ', true)]
    #[case::single_giyeok('ㄱ', false)]
    #[case::single_digeut('ㄷ', false)]
    #[case::single_bieup('ㅂ', false)]
    #[case::single_siot('ㅅ', false)]
    #[case::single_jieut('ㅈ', false)]
    fn identifies_all_double_consonants(#[case] cho: char, #[case] expected: bool) {
        assert_eq!(is_double_choseong(cho), expected);
    }

    /// `decompose` — 된소리는 (된소리표 32, 평음) 으로 분해, 단일자는 None.
    #[rstest::rstest]
    #[case::ssang_giyeok('ㄲ', Some((32, 'ㄱ')))]
    #[case::ssang_digeut('ㄸ', Some((32, 'ㄷ')))]
    #[case::ssang_bieup('ㅃ', Some((32, 'ㅂ')))]
    #[case::ssang_siot('ㅆ', Some((32, 'ㅅ')))]
    #[case::ssang_jieut('ㅉ', Some((32, 'ㅈ')))]
    #[case::single_giyeok_none('ㄱ', None)]
    #[case::single_hieut_none('ㅎ', None)]
    fn decompose_double_consonant(#[case] cho: char, #[case] expected: Option<(u8, char)>) {
        assert_eq!(decompose(cho), expected);
    }

    #[test]
    fn rejects_single_consonants() {
        assert!(!is_double_choseong('ㄱ'));
        assert!(!is_double_choseong('ㄷ'));
        assert!(!is_double_choseong('ㅂ'));
        assert!(!is_double_choseong('ㅅ'));
        assert!(!is_double_choseong('ㅈ'));
    }

    #[test]
    fn decomposes_correctly() {
        assert_eq!(decompose('ㄲ'), Some((32, 'ㄱ')));
        assert_eq!(decompose('ㄸ'), Some((32, 'ㄷ')));
        assert_eq!(decompose('ㅃ'), Some((32, 'ㅂ')));
        assert_eq!(decompose('ㅆ'), Some((32, 'ㅅ')));
        assert_eq!(decompose('ㅉ'), Some((32, 'ㅈ')));
    }

    #[test]
    fn decompose_returns_none_for_single() {
        assert_eq!(decompose('ㄱ'), None);
        assert_eq!(decompose('ㅎ'), None);
    }

    use rstest::rstest;

    #[rstest]
    #[case("까", true)] // ㄲ
    #[case("따", true)] // ㄸ
    #[case("빠", true)] // ㅃ
    #[case("싸", true)] // ㅆ
    #[case("짜", true)] // ㅉ
    #[case("가", false)] // ㄱ — single
    #[case("A", false)] // not Korean
    fn rule2_matches_double_choseong(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule2.matches(&ctx), expected, "input={input}");
    }

    #[rstest]
    #[case("까")]
    #[case("따")]
    #[case("빠")]
    #[case("싸")]
    #[case("짜")]
    fn rule2_apply_emits_double_indicator(#[case] input: &str) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule2.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Continue));
        assert_eq!(owned.result, vec![32u8]); // ⠠ 된소리표
    }

    #[test]
    fn rule2_apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule2.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    #[test]
    fn rule2_apply_no_emit_for_single_choseong() {
        // 가 is Korean, but ㄱ is not a double consonant — apply matches=false path
        // however apply() doesn't check matches, just calls decompose
        let mut owned = crate::test_helpers::CtxOwned::for_text("가", false);
        let mut ctx = owned.ctx_at(0);
        Rule2.apply(&mut ctx).unwrap();
        // decompose('ㄱ') returns None → no emit
        assert!(owned.result.is_empty());
    }

    #[test]
    fn rule2_meta_phase_priority() {
        let rule: &dyn BrailleRule = std::hint::black_box(&Rule2);

        assert_eq!(rule.meta().section, "2");
        assert!(matches!(rule.phase(), Phase::CoreEncoding));
        assert_eq!(rule.priority(), 195);
    }
}
