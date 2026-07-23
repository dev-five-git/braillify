//! 수학 제53항 — 프라임 표기.
//!
//! 프라임 기호(')를 점형으로 인코딩한다.

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::parser::MathToken;

pub fn encode_prime(result: &mut Vec<u8>) {
    result.push(36);
}

pub struct PrimeRule;

impl MathTokenRule for PrimeRule {
    fn name(&self) -> &'static str {
        "PrimeRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Prime))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        if !matches!(tokens.get(index), Some(MathToken::Prime)) {
            return Ok(MathTokenResult::Skip);
        }
        encode_prime(result);
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(1))
    }
}

#[cfg(test)]
mod tests {
    use super::super::math_token_rule::MathContext;
    use super::*;

    /// rule_53 line 29 - `PrimeRule.apply` Skip when token isn't Prime.
    #[test]
    fn prime_rule_apply_skip_for_non_prime() {
        let r = PrimeRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Variable('a')];
        let mut result = Vec::new();
        let engine = MathTokenEngine::with_context(MathContext::default());
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}
