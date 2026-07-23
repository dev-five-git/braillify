//! 수학 제8항 — 소수점 표기.
//!
//! 소수점(⠲)과 선행 소수(.47)의 수표 부여를 처리한다.

use crate::rules::math::parser::MathToken;

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};

/// Unicode combining-mark blocks (PDF — used to skip overline-like marks when
/// walking back to find the baseline Number for decimal-point handling).
const COMBINING_MARK_RANGES: &[(u32, u32)] = &[
    (0x0300, 0x036F),
    (0x1AB0, 0x1AFF),
    (0x1DC0, 0x1DFF),
    (0x20D0, 0x20FF),
    (0xFE20, 0xFE2F),
];

// Executed by every decimal-point test exercising the overline-decimal
// pattern (e.g. `2̄.3010`); tarpaulin can't attribute the iter-any closure.
#[cfg(not(tarpaulin_include))]
fn is_combining_mark_codepoint(c: char) -> bool {
    let cp = c as u32;
    COMBINING_MARK_RANGES
        .iter()
        .any(|(lo, hi)| (*lo..=*hi).contains(&cp))
}

// Executed by `encode_decimal_point` callers; tarpaulin `matches!()` with
// guard attribution limitation.
#[cfg(not(tarpaulin_include))]
fn is_combining_mark_token(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::MathSymbol(c)) if is_combining_mark_codepoint(*c))
}

pub fn encode_decimal_point(
    tokens: &[MathToken],
    i: usize,
    prev_was_number: &mut bool,
    result: &mut Vec<u8>,
) {
    // PDF — 직전이 결합 부호(예: `̄` overline)면 그 앞 baseline이 Number인지 본다.
    // 예: `2̄.3010` 에서 overline U+0305 사이를 건너뛰고 `2` (Number)를 인식한다.
    let prev_baseline_is_number = {
        let mut j = i;
        while j > 0 && is_combining_mark_token(tokens.get(j - 1)) {
            j -= 1;
        }
        j > 0 && matches!(tokens.get(j - 1), Some(MathToken::Number(_)))
    };
    if !*prev_was_number && !prev_baseline_is_number {
        let next = tokens.get(i + 1);
        let has_next_number = matches!(next, Some(MathToken::Number(_)))
            || (matches!(next, Some(MathToken::MathSymbol('\u{0307}')))
                && matches!(tokens.get(i + 2), Some(MathToken::Number(_))));
        if has_next_number {
            result.push(60);
            *prev_was_number = true;
        }
    } else if prev_baseline_is_number {
        // 직전 baseline이 Number이면 그 number context를 유지한다.
        *prev_was_number = true;
    }
    result.push(50);
}

pub struct DecimalPointRule;

impl MathTokenRule for DecimalPointRule {
    fn name(&self) -> &'static str {
        "DecimalPointRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::DecimalPoint))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        encode_decimal_point(tokens, index, &mut state.prev_was_number, result);
        Ok(MathTokenResult::Consumed(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    /// 제8항 — DecimalPointRule metadata.
    #[test]
    fn decimal_point_rule_metadata() {
        let r = DecimalPointRule;
        assert_eq!(r.priority(), 50);
        assert_eq!(r.name(), "DecimalPointRule");
    }

    /// 제8항 — leading decimal `.47` emits number sign before dot (lines 36-46).
    #[test]
    fn leading_decimal_emits_number_sign() {
        let tokens = vec![MathToken::DecimalPoint, MathToken::Number("47".into())];
        let mut prev = false;
        let mut result = Vec::new();
        encode_decimal_point(&tokens, 0, &mut prev, &mut result);
        // [60, 50] — number sign then decimal point
        assert_eq!(result.first(), Some(&60));
        assert!(prev);
    }

    /// 제8항 — prev baseline is a Number through combining marks (lines 17-34).
    /// Drives lines 24-27 (combining mark ranges) and 41 (early return path).
    #[test]
    fn decimal_with_combining_mark_baseline() {
        // 2̄.3 — Number, U+0305 (combining overline), DecimalPoint, Number
        let tokens = vec![
            MathToken::Number("2".into()),
            MathToken::MathSymbol('\u{0305}'),
            MathToken::DecimalPoint,
            MathToken::Number("3".into()),
        ];
        let mut prev = false;
        let mut result = Vec::new();
        encode_decimal_point(&tokens, 2, &mut prev, &mut result);
        // prev_baseline_is_number=true → prev_was_number is set true; only `.` byte emitted.
        assert!(prev);
        assert_eq!(result, vec![50]);
    }

    /// 제8항 — decimal after dot-above (U+0307) sequence in next position (line 38-40).
    #[test]
    fn leading_decimal_with_dot_above_next() {
        let tokens = vec![
            MathToken::DecimalPoint,
            MathToken::MathSymbol('\u{0307}'),
            MathToken::Number("9".into()),
        ];
        let mut prev = false;
        let mut result = Vec::new();
        encode_decimal_point(&tokens, 0, &mut prev, &mut result);
        // has_next_number=true via dot-above lookahead → 60, then 50.
        assert!(result.starts_with(&[60]));
    }

    /// Smoke test for full pipeline.
    #[test]
    fn decimal_in_full_expression() {
        let bytes = enc("$3.14$");
        assert!(!bytes.is_empty());
    }
}
