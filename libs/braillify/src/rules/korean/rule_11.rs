//! 제11항 — 모음자에 '예'가 붙어 나올 때에는 그 사이에 구분표 ⠤을 적어 나타낸다.
//!
//! When a vowel is followed by '예' (ㅇ+ㅖ), insert separator ⠤ (code 36) between them.
//! Condition: current syllable has no final consonant (jongseong).
//!
//! Reference: 2024 Korean Braille Standard, Chapter 1, Section 5, Article 11

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "11",
    subsection: None,
    name: "vowel_ye_separator",
    standard_ref: "2024 Korean Braille Standard, Ch.1 Sec.5 Art.11",
    description: "Insert separator ⠤ between vowel-ending syllable and 예 (ㅇ+ㅖ)",
};

const SEPARATOR: u8 = 36; // ⠤

/// Apply rule 11: insert ⠤ separator before 예 when preceded by a vowel-ending syllable.
///
/// # Arguments
/// * `current` - The current Korean syllable (already decomposed)
/// * `next` - The next raw character in the word
/// * `result` - The braille output buffer to append to
#[cfg(test)]
fn apply(
    current: &crate::char_struct::KoreanChar,
    next: char,
    result: &mut Vec<u8>,
) -> Result<(), String> {
    if let CharType::Korean(korean) = CharType::new(next)?
        && current.jong.is_none()
        && korean.cho == 'ㅇ'
        && korean.jung == 'ㅖ'
    {
        result.push(SEPARATOR);
    }
    Ok(())
}

/// Plugin struct for the rule engine.
pub struct Rule11;

impl BrailleRule for Rule11 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::InterCharacter
    }

    fn priority(&self) -> u16 {
        100
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        let Some(korean) = ctx.as_korean() else {
            return false;
        };
        if korean.jong.is_some() {
            return false;
        }
        ctx.next_char().is_some_and(|next| {
            matches!(
                CharType::new(next),
                Ok(CharType::Korean(next_k)) if next_k.cho == 'ㅇ' && next_k.jung == 'ㅖ'
            )
        })
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        ctx.emit(SEPARATOR);
        Ok(RuleResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::char_struct::KoreanChar;

    fn make_korean(ch: char) -> KoreanChar {
        match CharType::new(ch).unwrap() {
            CharType::Korean(k) => k,
            _ => panic!("Expected Korean character: {}", ch),
        }
    }

    #[test]
    fn inserts_separator_for_a_ye() {
        // 아예: 아 (ㅇ+ㅏ, no jong) + 예 (ㅇ+ㅖ) → should insert 36
        let current = make_korean('아');
        let mut result = Vec::new();
        apply(&current, '예', &mut result).unwrap();
        assert_eq!(result, vec![SEPARATOR]);
    }

    #[test]
    fn inserts_separator_for_do_ye() {
        // 도예: 도 (ㄷ+ㅗ, no jong) + 예 (ㅇ+ㅖ)
        let current = make_korean('도');
        let mut result = Vec::new();
        apply(&current, '예', &mut result).unwrap();
        assert_eq!(result, vec![SEPARATOR]);
    }

    #[test]
    fn inserts_separator_for_seo_ye() {
        // 서예: 서 (ㅅ+ㅓ, no jong) + 예 (ㅇ+ㅖ)
        let current = make_korean('서');
        let mut result = Vec::new();
        apply(&current, '예', &mut result).unwrap();
        assert_eq!(result, vec![SEPARATOR]);
    }

    #[test]
    fn skips_when_current_has_jongseong() {
        // 본예: 본 (ㅂ+ㅗ+ㄴ) has jong → no separator
        let current = make_korean('본');
        assert!(current.jong.is_some());
        let mut result = Vec::new();
        apply(&current, '예', &mut result).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn skips_when_next_is_not_ye() {
        // 아이: next is 이, not 예
        let current = make_korean('아');
        let mut result = Vec::new();
        apply(&current, '이', &mut result).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn skips_when_next_is_non_korean() {
        let current = make_korean('아');
        let mut result = Vec::new();
        apply(&current, 'A', &mut result).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "11");
        assert_eq!(META.name, "vowel_ye_separator");
    }

    use rstest::rstest;

    /// Build context that includes a 2-char word so next_char() works.
    fn ctx_for_pair(syllable_pair: &str) -> crate::test_helpers::CtxOwned {
        crate::test_helpers::CtxOwned::for_text(syllable_pair, false)
    }

    #[rstest]
    #[case("아예", true)] // ㅇ+ㅏ → ㅇ+ㅖ
    #[case("도예", true)] // ㄷ+ㅗ → ㅇ+ㅖ
    #[case("본예", false)] // current has jong (ㄴ)
    #[case("아이", false)] // next is 이, not 예
    fn rule11_matches_vowel_ye_pattern(#[case] input: &str, #[case] expected: bool) {
        let mut owned = ctx_for_pair(input);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule11.matches(&ctx), expected, "input={input}");
    }

    #[test]
    fn rule11_apply_emits_separator() {
        let mut owned = ctx_for_pair("아예");
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule11.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Continue));
        assert_eq!(owned.result, vec![SEPARATOR]);
    }

    #[test]
    fn rule11_phase_and_priority() {
        assert!(matches!(Rule11.phase(), Phase::InterCharacter));
        assert_eq!(Rule11.priority(), 100);
    }

    /// rule_11 line 53 — `let-else return false` for non-Korean ctx.
    #[test]
    fn rule11_matches_false_for_non_korean_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("Ax", false);
        let ctx = owned.ctx_at(0);
        assert!(!Rule11.matches(&ctx));
    }

    /// rule_11 line 59 — `let-else return false` when no next char.
    #[test]
    fn rule11_matches_false_at_end_of_word() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("아", false);
        let ctx = owned.ctx_at(0);
        // Single Korean char, no next → next_char() returns None.
        assert!(!Rule11.matches(&ctx));
    }
}
