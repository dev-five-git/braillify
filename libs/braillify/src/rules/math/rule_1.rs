//! 수학 제1항 — 수표와 숫자 표기.
//!
//! 숫자 시작 시 수표(#)를 붙이고 숫자 점형으로 변환한다.

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::parser::MathToken;

/// # (수표)
pub const NUMBER_PREFIX: u8 = 60;

/// 현재 토큰이 숫자 시작인지 판단한다.
pub fn needs_number_prefix(prev_was_number: bool) -> bool {
    !prev_was_number
}

/// 숫자 토큰을 수표 문맥에 맞춰 인코딩한다.
pub fn encode_number_with_prefix(digits: &str, prev_was_number: bool, result: &mut Vec<u8>) {
    if needs_number_prefix(prev_was_number) {
        result.push(NUMBER_PREFIX);
    }
    for ch in digits.chars() {
        result.extend(crate::number::encode_number(ch));
    }
}

/// 독립 숫자 리터럴(항상 수표 포함) 인코딩.
pub fn encode_number_literal(digits: &str, result: &mut Vec<u8>) {
    result.push(NUMBER_PREFIX);
    for ch in digits.chars() {
        result.extend(crate::number::encode_number(ch));
    }
}

pub struct NumberRule;

impl MathTokenRule for NumberRule {
    fn name(&self) -> &'static str {
        "NumberRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Number(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::Number(digits)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };

        encode_number_with_prefix(digits, state.prev_was_number, result);
        state.prev_was_number = true;
        Ok(MathTokenResult::Consumed(1))
    }
}

#[cfg(test)]
mod tests {
    use super::super::math_token_rule::MathContext;
    use super::*;

    /// rule_1 line 51 - `NumberRule.apply` let-else Skip when token isn't Number.
    #[test]
    fn number_rule_apply_skip_for_non_number() {
        let r = NumberRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Variable('a')];
        let mut result = Vec::new();
        let engine = MathTokenEngine::with_context(MathContext::default());
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}
