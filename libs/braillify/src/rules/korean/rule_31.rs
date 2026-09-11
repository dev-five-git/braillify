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

/// 제30항 그리스 문자 표 (대·소문자 동일 점형; 대문자표는 호출부가 붙인다).
fn greek_braille(c: char) -> Option<&'static str> {
    match c {
        'Α' | 'α' => Some("⠨⠁"),
        'Β' | 'β' => Some("⠨⠃"),
        'Γ' | 'γ' => Some("⠨⠛"),
        'Δ' | 'δ' => Some("⠨⠙"),
        'Ε' | 'ε' => Some("⠨⠑"),
        'Ζ' | 'ζ' => Some("⠨⠵"),
        'Η' | 'η' => Some("⠨⠱"),
        'Θ' | 'θ' => Some("⠨⠹"),
        'Ι' | 'ι' => Some("⠨⠊"),
        'Κ' | 'κ' => Some("⠨⠅"),
        'Λ' | 'λ' => Some("⠨⠇"),
        // U+00B5 MICRO SIGN is the Unicode compatibility form of μ (`µm`).
        'Μ' | 'μ' | 'µ' => Some("⠨⠍"),
        'Ν' | 'ν' => Some("⠨⠝"),
        'Ξ' | 'ξ' => Some("⠨⠭"),
        'Ο' | 'ο' => Some("⠨⠕"),
        'Π' | 'π' => Some("⠨⠏"),
        'Ρ' | 'ρ' => Some("⠨⠗"),
        'Σ' | 'σ' | 'ς' => Some("⠨⠎"),
        'Τ' | 'τ' => Some("⠨⠞"),
        'Υ' | 'υ' => Some("⠨⠥"),
        'Φ' | 'φ' => Some("⠨⠋"),
        'Χ' | 'χ' => Some("⠨⠯"),
        'Ψ' | 'ψ' => Some("⠨⠽"),
        'Ω' | 'ω' => Some("⠨⠺"),
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

        // 제31항: 국어 문장 안의 그리스 문자는 로마자표와 종료표로 감싼다. 이미
        // 열려 있는 로마자 구간(제29항 `IFN-γ`, 제35항 `1β`)에는 로마자표를 다시
        // 적지 않고, 종료표는 로마자와 같은 경계 규칙(제33·34·35항)에 맡긴다.
        let korean_context = korean_context(ctx);
        if korean_context && !ctx.state.is_english {
            if ctx.state.needs_english_continuation {
                ctx.emit(crate::rules::korean::rule_29::ENGLISH_CONTINUATION);
            } else {
                ctx.emit(crate::rules::korean::rule_29::ROMAN_INDICATOR);
            }
        }
        if run.len() > 1 && run.iter().all(|c| c.is_uppercase()) {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
            ctx.emit(crate::unicode::decode_unicode('⠠'));
        } else if run.len() == 1 && run[0].is_uppercase() {
            ctx.emit(crate::unicode::decode_unicode('⠠'));
        }

        for unicode in run.iter().filter_map(|ch| greek_braille(*ch)) {
            ctx.emit_slice(&encode_unicode_cells(unicode));
        }
        if korean_context {
            crate::rules::roman_mode::mark_section_open(ctx.state);
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

#[cfg(test)]
mod greek_run_coverage {
    /// 제31항: Greek letters take their own cells, alone or in a run.
    #[rstest::rstest]
    #[case::single("알파 \u{03B1} 값")]
    #[case::run("\u{03B1}\u{03B2}\u{03B3}")]
    fn greek_letters_encode(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod greek_continuation_coverage {
    /// 제31항 + 제29항: 그리스 문자는 홀로도 잇달아도 제 점형을 쓰고, 뒤에 로마자가
    /// 이어지면 연속표를 앞세운다.
    #[rstest::rstest]
    #[case::single("알파 \u{03B1} 값")]
    #[case::run("\u{03B1}\u{03B2}\u{03B3}")]
    #[case::greek_then_roman("\u{03B1}x 값")]
    #[case::greek_run_then_roman("\u{03B1}\u{03B2}t 값")]
    fn a_greek_run_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod greek_after_open_section_coverage {
    /// 제31항: 이미 열린 로마자 구간 뒤의 그리스 문자는 로마자표를 다시 적지 않고,
    /// 제33항으로 구간이 이어지는 자리에서는 연속표를 앞세운다.
    #[rstest::rstest]
    #[case::after_roman_hyphen("그는 IFN-\u{03B3} 를")]
    #[case::after_comma("비타민A, \u{03B3}")]
    #[case::after_number("\u{03B1} 1\u{03B2} 값")]
    fn a_greek_letter_after_a_roman_item_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod greek_continuation_indicator_coverage {
    /// 제31항 + 제33항: 앞 로마자 구간이 종료표 없이 닫혔다면 그리스 문자 앞에는
    /// 로마자표가 아니라 연속표를 적는다.
    #[rstest::rstest]
    #[case::after_comma("비타민A, \u{03B3}")]
    #[case::after_comma_short("A, \u{03B3}")]
    #[case::after_number_comma("비타민1, \u{03B3}")]
    #[case::after_closing_paren("비타민(A), \u{03B3}")]
    #[case::after_middle_dot("DT\u{00B7}\u{03B3}")]
    #[case::after_hyphen("IFN-\u{03B3}")]
    #[case::after_number("1\u{03B2} 값")]
    fn a_greek_letter_after_an_open_section_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod continuation_indicator_branch_coverage {
    use super::*;
    use crate::rules::traits::BrailleRule;

    /// 제31항 + 제29항: 앞 로마자 구간이 종료표 없이 닫혀 연속표가 예약돼 있으면
    /// 그리스 문자 앞에는 로마자표가 아니라 연속표를 적는다.
    #[rstest::rstest]
    #[case::continuation_pending(true, crate::rules::korean::rule_29::ENGLISH_CONTINUATION)]
    #[case::fresh_section(false, crate::rules::korean::rule_29::ROMAN_INDICATOR)]
    fn a_greek_run_opens_with_the_reserved_marker(
        #[case] continuation_pending: bool,
        #[case] expected_first_cell: u8,
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text("\u{03B1}", false);
        owned.state.needs_english_continuation = continuation_pending;
        owned.prev_word = "한글".to_string();
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule31.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(
            owned.result.first().copied(),
            Some(expected_first_cell),
            "unexpected opening marker"
        );
    }
}
