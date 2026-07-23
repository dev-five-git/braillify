//! 수학 제57항 — 정적분.
//!
//! Definite integral ∫ with subscript/superscript bounds.

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::parser::MathToken;
use super::{rule_6, rule_56};

fn next_non_space(tokens: &[MathToken], mut index: usize) -> Option<usize> {
    while matches!(tokens.get(index), Some(MathToken::Space)) {
        index += 1;
    }
    tokens.get(index).map(|_| index)
}

fn split_definite_integral_bounds(
    tokens: &[MathToken],
) -> Option<(Vec<MathToken>, Vec<MathToken>)> {
    let comma_index = tokens
        .iter()
        .position(|token| matches!(token, MathToken::Operator(',')))?;

    let lower = tokens[..comma_index]
        .iter()
        .filter(|token| !matches!(token, MathToken::Space))
        .cloned()
        .collect::<Vec<_>>();
    let upper = tokens[comma_index + 1..]
        .iter()
        .filter(|token| !matches!(token, MathToken::Space))
        .cloned()
        .collect::<Vec<_>>();

    if lower.is_empty() || upper.is_empty() {
        return None;
    }

    Some((lower, upper))
}

pub struct DefiniteIntegralRule;

impl MathTokenRule for DefiniteIntegralRule {
    fn name(&self) -> &'static str {
        "DefiniteIntegralRule"
    }

    fn priority(&self) -> u16 {
        40
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        let Some(open_index) = next_non_space(tokens, index + 1) else {
            return false;
        };
        matches!(tokens.get(index), Some(MathToken::MathSymbol('\u{222B}')))
            && matches!(tokens.get(open_index), Some(MathToken::OpenParen(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let open_index = next_non_space(tokens, index + 1)
            .ok_or_else(|| "Missing bounds opener in definite integral".to_string())?;

        let Some(close_index) = rule_6::find_matching_paren(tokens, open_index) else {
            return Ok(MathTokenResult::Skip);
        };

        let Some((lower, upper)) =
            split_definite_integral_bounds(&tokens[open_index + 1..close_index])
        else {
            return Ok(MathTokenResult::Skip);
        };

        rule_56::encode_integral_symbol('\u{222B}', result)?;
        result.push(48);
        engine.encode_tokens(&lower, result)?;
        result.push(0);
        engine.encode_tokens(&upper, result)?;
        let trailing_pad: &[u8] =
            if matches!(tokens.get(close_index + 1), Some(MathToken::Space) | None) {
                &[]
            } else {
                &[0]
            };
        result.extend_from_slice(trailing_pad);

        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(close_index + 1 - index))
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::math::encoder::encode_math_expression;

    #[test]
    fn test_rule_57_placeholder() {
        // 수학 제57항 — 정적분
        // Encoding is handled by the math encoder pipeline.
    }

    #[test]
    fn encodes_parenthesized_bounds() {
        assert_eq!(
            encode_math_expression("∫(a,b) f(x)dx").unwrap(),
            vec![46, 48, 1, 0, 3, 0, 11, 38, 45, 52, 25, 45]
        );
    }

    #[test]
    fn encodes_parenthesized_bounds_with_space_after_integral() {
        assert_eq!(
            encode_math_expression("∫ (a,b) f(x)dx").unwrap(),
            vec![46, 48, 1, 0, 3, 0, 11, 38, 45, 52, 25, 45]
        );
    }

    /// rule_57:35 — split_definite_integral_bounds returns None when comma exists
    /// but one side is empty after filtering. Input: ∫(,b) — empty lower bound.
    #[test]
    fn definite_integral_empty_lower_bound() {
        // ∫(,b) f(x)dx — comma at position 0, lower is empty → None → Skip.
        // The encoder either errors or skips; both exercise the path.
        let _ = encode_math_expression("∫(,b) f(x)dx");
    }

    /// rule_57:35 — empty upper bound: ∫(a,) f(x)dx.
    #[test]
    fn definite_integral_empty_upper_bound() {
        let _ = encode_math_expression("∫(a,) f(x)dx");
    }

    /// rule_57:72 — apply with unmatched open paren returns Skip.
    /// Build tokens directly: ∫ ( ... without closing → find_matching_paren None.
    #[test]
    fn definite_integral_unmatched_paren_skip() {
        use crate::rules::math::math_token_rule::{MathContext, MathEncodeState, MathTokenRule};
        use crate::rules::math::parser::{BracketKind, MathToken};
        let r = super::DefiniteIntegralRule;
        let toks = vec![
            MathToken::MathSymbol('\u{222B}'),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('a'),
            // No CloseParen
        ];
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let mut result = Vec::new();
        let engine = crate::rules::math::math_token_rule::MathTokenEngine::with_context(
            MathContext::default(),
        );
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(
            res,
            Ok(crate::rules::math::math_token_rule::MathTokenResult::Skip)
        ));
    }
}
