//! 수학 제6항 — 괄호 표기.
//!
//! 소괄호, 묶음괄호, 대괄호, 중괄호, 연립식 괄호를 점형으로 변환한다.

use crate::rules::math::parser::{BracketKind, MathToken};

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};

pub fn encode_open_paren(kind: BracketKind, result: &mut Vec<u8>) {
    match kind {
        BracketKind::MathParen => result.push(38),
        BracketKind::Grouping => result.push(55),
        BracketKind::Hangul => {
            result.push(56);
            result.push(55);
        }
        BracketKind::Square => {
            result.push(55);
            result.push(4);
        }
        BracketKind::Curly => result.push(54),
    }
}

pub fn encode_close_paren(kind: BracketKind, result: &mut Vec<u8>) {
    match kind {
        BracketKind::MathParen => result.push(52),
        BracketKind::Grouping => result.push(62),
        BracketKind::Hangul => {
            result.push(56);
            result.push(62);
        }
        BracketKind::Square => {
            result.push(32);
            result.push(62);
        }
        BracketKind::Curly => result.push(54),
    }
}

pub fn find_matching_paren(tokens: &[MathToken], start: usize) -> Option<usize> {
    let open_kind = match tokens.get(start) {
        Some(MathToken::OpenParen(kind)) => *kind,
        _ => return None,
    };

    let mut depth = 0usize;
    for (idx, token) in tokens.iter().enumerate().skip(start) {
        match token {
            MathToken::OpenParen(kind) if *kind == open_kind => depth += 1,
            MathToken::CloseParen(kind) if *kind == open_kind => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

pub struct BracketRule;

impl MathTokenRule for BracketRule {
    fn name(&self) -> &'static str {
        "BracketRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(
            tokens.get(index),
            Some(MathToken::OpenParen(_) | MathToken::CloseParen(_))
        )
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let tok = tokens.get(index);
        if let Some(MathToken::OpenParen(kind)) = tok {
            encode_open_paren(*kind, result);
        } else if let Some(MathToken::CloseParen(kind)) = tok {
            encode_close_paren(*kind, result);
        } else {
            return Ok(MathTokenResult::Skip);
        }

        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(1))
    }
}

#[cfg(test)]
mod tests {
    use super::super::math_token_rule::MathContext;
    use super::*;

    /// rule_6 line 85 - `BracketRule.apply` Skip when token isn't OpenParen/CloseParen.
    #[test]
    fn bracket_rule_apply_skip_for_non_paren() {
        let r = BracketRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Variable('a')];
        let mut result = Vec::new();
        let engine = MathTokenEngine::with_context(MathContext::default());
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}
