//! 제12항 — 'ㅑ, ㅘ, ㅜ, ㅝ'에 '애'가 붙어 나올 때에는 두 모음자 사이에 구분표 ⠤을 적어 나타낸다.
//!
//! When specific vowels (ㅑ, ㅘ, ㅜ, ㅝ) are followed by '애' (ㅇ+ㅐ),
//! insert separator ⠤ (code 36) between them.
//! Condition: current syllable has no final consonant (jongseong).
//!
//! Reference: 2024 Korean Braille Standard, Chapter 1, Section 5, Article 12

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "12",
    subsection: None,
    name: "vowel_ae_separator",
    standard_ref: "2024 Korean Braille Standard, Ch.1 Sec.5 Art.12",
    description: "Insert separator ⠤ between ㅑ/ㅘ/ㅜ/ㅝ and 애 (ㅇ+ㅐ)",
};

const SEPARATOR: u8 = 36; // ⠤
const TRIGGERING_VOWELS: [char; 4] = ['ㅑ', 'ㅘ', 'ㅜ', 'ㅝ'];

/// Apply rule 12: insert ⠤ separator before 애 when preceded by ㅑ/ㅘ/ㅜ/ㅝ.
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
        && TRIGGERING_VOWELS.contains(&current.jung)
        && korean.cho == 'ㅇ'
        && korean.jung == 'ㅐ'
    {
        result.push(SEPARATOR);
    }
    Ok(())
}

/// Plugin struct for the rule engine.
pub struct Rule12;

impl BrailleRule for Rule12 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::InterCharacter
    }

    fn priority(&self) -> u16 {
        110 // Runs after Rule11 (priority 100)
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        ctx.as_korean().is_some_and(|korean| {
            korean.jong.is_none()
                && TRIGGERING_VOWELS.contains(&korean.jung)
                && ctx.next_char().is_some_and(|next| {
                    matches!(
                        CharType::new(next),
                        Ok(CharType::Korean(next_k)) if next_k.cho == 'ㅇ' && next_k.jung == 'ㅐ'
                    )
                })
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

    /// 제12항 — current 자모(중성) + next 음절(애로 시작) 시 분리표 삽입.
    /// 트리거 모음: ㅑ/ㅘ/ㅜ/ㅝ + (jong 없음) + next 가 ㅇ+ㅐ 시작 →
    /// `vec![SEPARATOR]`, 그 외는 빈 결과.
    #[rstest::rstest]
    #[case::ya_followed_by_ae('야', '애', vec![SEPARATOR])]
    #[case::hwa_followed_by_aek('화', '액', vec![SEPARATOR])]
    #[case::su_followed_by_aek('수', '액', vec![SEPARATOR])]
    #[case::weo_followed_by_aem('워', '앰', vec![SEPARATOR])]
    #[case::a_non_triggering('가', '애', vec![])]
    #[case::eo_non_triggering('서', '애', vec![])]
    #[case::current_has_jong_skipped('숙', '애', vec![])]
    #[case::next_not_ae_skipped('야', '이', vec![])]
    fn rule12_apply_separator_paths(
        #[case] current_syllable: char,
        #[case] next_char: char,
        #[case] expected: Vec<u8>,
    ) {
        let current = make_korean(current_syllable);
        let mut result = Vec::new();
        apply(&current, next_char, &mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn meta_is_correct() {
        assert_eq!(META.section, "12");
        assert_eq!(META.name, "vowel_ae_separator");
    }

    use rstest::rstest;

    #[rstest]
    #[case("야애", true)] // ㅇ+ㅑ → ㅇ+ㅐ
    #[case("화애", true)] // ㅎ+ㅘ → ㅇ+ㅐ
    #[case("아애", false)] // ㅏ is non-triggering
    #[case("어애", false)] // ㅓ is non-triggering
    #[case("관애", false)] // current has jong (ㄴ)
    #[case("야이", false)] // next is 이, not 애
    #[case("A", false)] // non-Korean
    #[case("야", false)] // single char, no next → line 65 hit
    fn rule12_matches_triggering_vowel_ae(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(Rule12.matches(&ctx), expected, "input={input}");
    }
}
