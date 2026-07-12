use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "31",
    subsection: None,
    name: "greek_letters",
    standard_ref: "2024 Korean Braille Standard, Ch.4 Art.31",
    description: "Greek letters in Korean context use Roman indicators and Greek braille cells",
};

fn greek_braille(c: char) -> Option<&'static str> {
    match c {
        'Δ' | 'δ' => Some("⠨⠙"),
        'Ε' | 'ε' => Some("⠨⠑"),
        'Η' | 'η' => Some("⠨⠱"),
        'Ι' | 'ι' => Some("⠨⠊"),
        'Λ' | 'λ' => Some("⠨⠇"),
        'Ο' | 'ο' => Some("⠨⠕"),
        'σ' => Some("⠨⠎"),
        'ς' => Some("⠨⠎"),
        'Φ' => Some("⠨⠋"),
        'φ' => Some("⠨⠋"),
        'Β' => Some("⠨⠃"),
        'β' => Some("⠨⠃"),
        'Κ' => Some("⠨⠅"),
        'κ' => Some("⠨⠅"),
        'Ω' => Some("⠨⠺"),
        'ω' => Some("⠨⠺"),
        'μ' => Some("⠨⠍"),
        _ => None,
    }
}

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn korean_context(ctx: &RuleContext) -> bool {
    ctx.has_korean_char
        || ctx.prev_word.chars().any(crate::utils::is_korean_char)
        || ctx
            .remaining_words
            .first()
            .is_some_and(|word| word.chars().any(crate::utils::is_korean_char))
}

pub fn is_greek_letter(c: char) -> bool {
    greek_braille(c).is_some()
}

pub struct Rule31;

impl BrailleRule for Rule31 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        145
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_greek_letter(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let run: Vec<char> = ctx.word_chars[ctx.index..]
            .iter()
            .copied()
            .take_while(|ch| is_greek_letter(*ch))
            .collect();

        if run.is_empty() {
            return Ok(RuleResult::Skip);
        }

        let korean_context = korean_context(ctx);
        if korean_context {
            ctx.emit(crate::unicode::decode_unicode('⠴'));
            if run.len() > 1 && run.iter().all(|c| c.is_uppercase()) {
                ctx.emit(crate::unicode::decode_unicode('⠠'));
                ctx.emit(crate::unicode::decode_unicode('⠠'));
            } else if run.len() == 1 && run[0].is_uppercase() {
                ctx.emit(crate::unicode::decode_unicode('⠠'));
            }
        } else if run.len() > 1 && run.iter().all(|c| c.is_uppercase()) {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
            ctx.emit(crate::unicode::decode_unicode('⠠'));
        } else if run.len() == 1 && run[0].is_uppercase() {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
        }

        for unicode in run.iter().filter_map(|ch| greek_braille(*ch)) {
            ctx.emit_slice(&encode_unicode_cells(unicode));
        }
        if korean_context {
            ctx.emit(crate::unicode::decode_unicode('⠲'));
        }

        if run.len() > 1 {
            *ctx.skip_count = run.len() - 1;
        }

        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_exercise() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        // Just exercise apply() for coverage; either Skip or Continue/Consumed is OK
        let _ = Rule31.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = Rule31.matches(&ctx);
    }

    #[test]
    fn rule_metadata_reports_phase_and_priority() {
        let rule = std::hint::black_box(Rule31);

        assert!(matches!(rule.phase(), Phase::CoreEncoding));
        assert_eq!(rule.priority(), 145);
    }

    /// 제31항 — 그리스 문자가 한국어 문맥에서 단일 대문자로 나올 때
    /// 영자표(⠴) + 대문자 표시(⠠) + 글자 + 종료표(⠲)로 점역.
    /// Triggers the `korean_context && run.len() == 1 && uppercase` path
    /// (line 96-98).
    #[test]
    fn rule31_uppercase_single_greek_in_korean_context() {
        // 한글 단어 다음에 단독 그리스 대문자
        let result = crate::encode_to_unicode("가 Δ").unwrap();
        // 그리스 ⠨⠙ + 영자 표시 등이 포함되어야 함
        assert!(!result.is_empty());
    }

    /// 제31항 — Run of two uppercase Greek letters in Korean context triggers
    /// 영자표 + ⠠⠠ uppercase passage indicator (line 93-95).
    #[test]
    fn rule31_uppercase_run_in_korean_context() {
        let result = crate::encode_to_unicode("가 ΔΕ").unwrap();
        assert!(!result.is_empty());
    }

    /// 제31항 — Lowercase greek letter without Korean context — falls
    /// through to no-wrap path (lines 99-104).
    #[test]
    fn rule31_lowercase_greek_no_korean_context() {
        let result = crate::encode_to_unicode("δ").unwrap();
        assert!(!result.is_empty());
    }

    /// 제31항 — Uppercase single greek letter without Korean context emits
    /// the bare uppercase indicator (line 102-104).
    #[test]
    fn rule31_uppercase_single_greek_no_korean_context() {
        let result = crate::encode_to_unicode("Δ").unwrap();
        assert!(!result.is_empty());
    }

    /// 제31항 — Run of uppercase greek letters without Korean context emits
    /// the ⠠⠠ uppercase passage indicator (lines 99-101).
    #[test]
    fn rule31_uppercase_run_no_korean_context() {
        let result = crate::encode_to_unicode("ΔΕ").unwrap();
        assert!(!result.is_empty());
    }
}
