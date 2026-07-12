//! 제1항 — 기본 자음자 14개가 첫소리로 쓰일 때에는 다음과 같이 적는다.
//!
//! Maps 13 initial consonants (choseong) to braille dot patterns.
//! Note: ㅇ as initial consonant is NOT encoded (제1항 [다만 1]).
//!
//! Encoding is delegated to `jauem::choseong::encode_choseong()` which uses a PHF map.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 1, Section 1, Article 1

use crate::jauem::choseong;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "1",
    subsection: None,
    name: "basic_choseong",
    standard_ref: "2024 Korean Braille Standard, Ch.1 Sec.1 Art.1",
    description: "Encode 13 basic initial consonants (choseong) to braille",
};

/// Encode a choseong character to its braille representation.
/// Re-exports `jauem::choseong::encode_choseong`.
#[cfg(test)]
fn apply(cho: char) -> Result<u8, String> {
    choseong::encode_choseong(cho)
}

/// Check if a choseong is ㅇ (which should be skipped per 제1항 [다만 1]).
pub fn is_silent_ieung(cho: char) -> bool {
    cho == 'ㅇ'
}

/// Plugin struct for the rule engine.
///
/// Sub-component rule: encodes the initial consonant (choseong) of a Korean syllable.
/// In the engine-driven pipeline, this is called as part of syllable encoding — NOT
/// registered as a standalone top-level rule. It emits only the choseong portion
/// and returns Continue so jungseong/jongseong rules can add their parts.
///
/// Note: ㅇ as choseong is silent (제1항 [다만 1]) and emits nothing.
pub struct Rule1;

impl BrailleRule for Rule1 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        200 // Sub-component — runs within syllable encoding pipeline
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        ctx.as_korean().is_some()
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let Some(korean) = ctx.as_korean() else {
            return Ok(RuleResult::Skip);
        };
        // 제1항 [다만 1]: ㅇ as choseong is silent
        if !is_silent_ieung(korean.cho) {
            let code = choseong::encode_choseong(korean.cho)?;
            ctx.emit(code);
        }
        Ok(RuleResult::Continue) // Jungseong/jongseong still need encoding
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    #[rstest::rstest]
    #[case::giyeok('ㄱ', '⠈')]
    #[case::nieun('ㄴ', '⠉')]
    #[case::digeut('ㄷ', '⠊')]
    #[case::rieul('ㄹ', '⠐')]
    #[case::mieum('ㅁ', '⠑')]
    #[case::bieup('ㅂ', '⠘')]
    #[case::siot('ㅅ', '⠠')]
    #[case::jieut('ㅈ', '⠨')]
    #[case::chieut('ㅊ', '⠰')]
    #[case::kieuk('ㅋ', '⠋')]
    #[case::tieut('ㅌ', '⠓')]
    #[case::pieup('ㅍ', '⠙')]
    #[case::hieut('ㅎ', '⠚')]
    fn encodes_all_13_basic_consonants(#[case] cho: char, #[case] expected_braille: char) {
        let result = apply(cho).unwrap();
        assert_eq!(
            result,
            decode_unicode(expected_braille),
            "Failed for choseong: {cho}"
        );
    }

    #[test]
    fn ieung_is_not_in_choseong_map() {
        // ㅇ as choseong returns Err — it should be skipped, not encoded
        assert!(apply('ㅇ').is_err());
    }

    #[test]
    fn silent_ieung_detected() {
        assert!(is_silent_ieung('ㅇ'));
        assert!(!is_silent_ieung('ㄱ'));
    }

    #[test]
    fn invalid_char_returns_error() {
        assert!(apply('A').is_err());
        assert!(apply('가').is_err());
    }

    use rstest::rstest;

    /// matches() must return true iff the current char is a Korean syllable.
    #[rstest]
    #[case("가", true)]
    #[case("힣", true)]
    #[case("A", false)]
    #[case("1", false)]
    #[case("ㄱ", false)] // 자모 단독 — not a syllable
    fn rule1_matches_only_for_korean_syllables(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule1.matches(&ctx), expected, "input={input}");
    }

    /// apply() emits choseong for non-ㅇ initial; emits nothing for ㅇ.
    #[rstest]
    #[case("가", false)] // ㄱ — non-silent
    #[case("나", false)] // ㄴ — non-silent
    #[case("아", true)] // ㅇ — silent (no emit)
    #[case("어", true)] // ㅇ — silent
    fn rule1_apply_handles_silent_ieung(#[case] input: &str, #[case] silent: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule1.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Continue));
        if silent {
            assert!(
                owned.result.is_empty(),
                "ㅇ should emit nothing for {input}"
            );
        } else {
            assert!(!owned.result.is_empty(), "non-ㅇ should emit for {input}");
        }
    }

    /// apply() returns Skip when char_type is not Korean (Variable, English, etc.).
    #[test]
    fn rule1_apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule1.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    #[test]
    fn rule1_meta_phase_priority() {
        let rule: &dyn BrailleRule = std::hint::black_box(&Rule1);

        assert_eq!(rule.meta().section, "1");
        assert!(matches!(rule.phase(), Phase::CoreEncoding));
        assert_eq!(rule.priority(), 200);
    }
}
