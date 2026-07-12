//! 제13항 — 다음 글자들은 약자를 사용하여 적는다.
//! (가, 나, 다, 마, 바, 사, 자, 카, 타, 파, 하, 것, 억, 언, 얼, 연, 열, 영, 옥, 온, 옹, 운, 울, 은, 을, 인, 성, 정, 청)
//!
//! 제15항 — 추가 약자 목록 (억, 언, 얼, 연, 열, 영, 옥, 온, 옹, 운, 울, 은, 을, 인, 것)
//!
//! Abbreviations are looked up from `char_shortcut::SHORTCUT_MAP` (PHF).
//! Encoding is delegated to `char_shortcut::encode_char_shortcut()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 2, Section 6, Articles 13, 15

use crate::char_shortcut;
use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META_13: RuleMeta = RuleMeta {
    section: "13",
    subsection: None,
    name: "syllable_abbreviation",
    standard_ref: "2024 Korean Braille Standard, Ch.2 Sec.6 Art.13",
    description: "Common syllable abbreviations (가,나,다,...,하)",
};

/// Try to encode a character using the abbreviation shortcut table.
/// Returns the abbreviated braille encoding, or Err if no abbreviation exists.
#[cfg(test)]
fn apply(ch: char) -> Result<&'static [u8], String> {
    char_shortcut::encode_char_shortcut(ch)
}

/// Check if a character has an abbreviation in the shortcut table.
pub fn has_abbreviation(ch: char) -> bool {
    char_shortcut::SHORTCUT_MAP.contains_key(&ch)
}

/// Plugin struct for the rule engine.
///
/// Handles syllable abbreviation lookup (제13항, 제15항).
/// Runs after rule_14 (which may suppress abbreviation). If a Korean syllable
/// has a shortcut entry, this rule emits the abbreviated form and Consumes.
pub struct Rule13;

impl BrailleRule for Rule13 {
    fn meta(&self) -> &'static RuleMeta {
        &META_13
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        90 // Before generic Korean encoding, after rule_14 (priority 80)
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        if let CharType::Korean(_) = ctx.char_type {
            has_abbreviation(ctx.current_char())
        } else {
            false
        }
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let encoded = char_shortcut::encode_char_shortcut(ctx.current_char())?;
        ctx.emit_slice(encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    /// 제13항·제15항 — 약자 음절을 점형 단일 또는 이중 셀로 매핑.
    #[rstest::rstest]
    #[case::ga('가', vec!['⠫'])]
    #[case::na('나', vec!['⠉'])]
    #[case::da('다', vec!['⠊'])]
    #[case::sa('사', vec!['⠇'])]
    #[case::ha('하', vec!['⠚'])]
    #[case::geot('것', vec!['⠸', '⠎'])]
    #[case::yeong('영', vec!['⠻'])]
    #[case::eun('은', vec!['⠵'])]
    #[case::in_char('인', vec!['⠟'])]
    fn encodes_syllable_abbreviations(#[case] syllable: char, #[case] expected_unicode: Vec<char>) {
        let expected: Vec<u8> = expected_unicode.into_iter().map(decode_unicode).collect();
        assert_eq!(apply(syllable).unwrap(), expected.as_slice());
    }

    /// `has_abbreviation` — 약자 사전 등록 여부.
    #[rstest::rstest]
    #[case::known_ga('가', true)]
    #[case::known_geot('것', true)]
    #[case::known_yeong('영', true)]
    #[case::unknown_gon('곤', false)]
    #[case::unknown_ascii('A', false)]
    #[case::unknown_digit('1', false)]
    fn has_abbreviation_paths(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(has_abbreviation(ch), expected);
    }

    #[test]
    fn non_abbreviated_char_returns_error() {
        assert!(apply('곤').is_err());
    }
}
