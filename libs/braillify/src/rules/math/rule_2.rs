//! 수학 제2항 — 사칙연산 기호.
//!
//! +, -, ×, ÷, · 및 팩토리얼, 쉼표, 슬래시 연산자 처리를 담당한다.

use crate::math_symbol_shortcut;
use crate::rules::math::parser::MathToken;

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::rule_7;

pub fn is_algebraic_neighbor(token: Option<&MathToken>) -> bool {
    matches!(
        token,
        Some(
            MathToken::Variable(_)
                | MathToken::UpperVariable(_)
                | MathToken::Number(_)
                | MathToken::Subscript(_)
                | MathToken::Superscript(_)
                | MathToken::OpenParen(_)
                | MathToken::CloseParen(_)
                | MathToken::MathSymbol('\u{221E}')
        )
    )
}

pub fn needs_binary_spacing(c: char) -> bool {
    matches!(
        c,
        '\u{2192}'
            | '\u{2190}'
            | '\u{21D2}'
            | '\u{21CF}'
            | '\u{2194}'
            | '\u{21D4}'
            | '\u{21C4}'
            | '\u{21CC}' // ⇌ (PDF 제61항 7)
            | '\u{2227}'
            | '\u{2228}'
            | '\u{22BB}'
            | '\u{2193}'
            | '\u{2191}'
            | '\u{2229}'
            | '\u{222A}'
            | '\u{25A1}'
            | '\u{2295}'
            | '\u{2296}'
            | '\u{2297}'
            | '\u{29BE}'
            | '\u{2217}'
            | '\u{2219}'
            | '\u{2206}'
            | '\u{2234}'
            | '\u{2235}'
            | '\u{2248}'
            | '\u{224A}'
            | '\u{2243}'
            | '\u{2245}'
            | '\u{25B7}'
            | '\u{25C1}'
            // PDF 수학 제60항 6 — 추론 기호 ⊢ ⊣ ⊨ ⫤
            | '\u{22A2}'
            | '\u{22A3}'
            | '\u{22A8}'
            | '\u{2AE4}'
            // PDF 수학 제60항 7~8 — 순서 관계 ≲ ≺
            | '\u{2272}'
            | '\u{227A}'
    )
}

pub fn encode_operator(
    token: char,
    tokens: &[MathToken],
    i: usize,
    result: &mut Vec<u8>,
) -> Result<(), String> {
    if token == '+' {
        let prev = tokens[..i]
            .iter()
            .rev()
            .find(|t| !matches!(t, MathToken::Space));
        let next = tokens[i + 1..]
            .iter()
            .find(|t| !matches!(t, MathToken::Space));
        let has_set_triangle = tokens
            .iter()
            .any(|t| matches!(t, MathToken::MathSymbol('\u{2206}')));

        if has_set_triangle
            && matches!(prev, Some(MathToken::CloseParen(_)))
            && matches!(next, Some(MathToken::OpenParen(_)))
        {
            result.push(0);
            result.push(44);
            result.push(0);
            return Ok(());
        }
    }

    if token == '!' {
        result.push(22);
        return Ok(());
    }

    if token == ',' {
        let divisibility_context = matches!(
            (
                tokens.get(i.saturating_sub(1)),
                tokens.get(i.saturating_sub(2))
            ),
            (Some(MathToken::Number(_)), Some(MathToken::MathSymbol('|')))
        );

        if tokens.get(i + 1).is_none() && divisibility_context {
            return Ok(());
        }
        if matches!(tokens.get(i + 1), Some(MathToken::Space)) && divisibility_context {
            result.push(0);
            return Ok(());
        }

        result.push(16);
        if matches!(
            tokens.get(i + 1),
            Some(
                MathToken::UpperVariable(_)
                    | MathToken::Variable(_)
                    | MathToken::Number(_)
                    | MathToken::Subscript(_)
                    | MathToken::Superscript(_)
                    | MathToken::MathSymbol(_)
            )
        ) {
            result.push(0);
        }
        return Ok(());
    }

    if token == '/' {
        if rule_7::slash_as_fraction_symbol(tokens, i) {
            let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(token)?;
            result.extend_from_slice(encoded);
        } else {
            result.push(12);
        }
        return Ok(());
    }

    let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(token)?;
    result.extend_from_slice(encoded);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::rules::math::math_token_rule::{MathContext, MathEncodeState, MathTokenEngine};

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    fn dummy_engine() -> MathTokenEngine {
        MathTokenEngine::with_context(MathContext::default())
    }

    /// 제2항 — divisibility context: `n|a, ` (trailing) returns early without emit.
    /// Drives line 116: `tokens.get(i + 1).is_none() && divisibility_context`.
    #[test]
    fn comma_in_divisibility_context_at_end() {
        let tokens = vec![
            MathToken::MathSymbol('|'),
            MathToken::Number("3".into()),
            MathToken::Operator(','),
        ];
        let mut result = Vec::new();
        encode_operator(',', &tokens, 2, &mut result).expect("encode_operator should succeed");
        assert!(
            result.is_empty(),
            "trailing , in divisibility context emits nothing"
        );
    }

    /// 제2항 — divisibility context with trailing Space pushes a single space.
    /// Drives lines 119-120.
    #[test]
    fn comma_in_divisibility_context_before_space() {
        let tokens = vec![
            MathToken::MathSymbol('|'),
            MathToken::Number("3".into()),
            MathToken::Operator(','),
            MathToken::Space,
        ];
        let mut result = Vec::new();
        encode_operator(',', &tokens, 2, &mut result).expect("encode_operator should succeed");
        assert_eq!(result, vec![0]);
    }

    /// 제2항 — operator rule basic dispatch metadata.
    #[test]
    fn operator_rule_metadata() {
        let rule = OperatorRule;
        assert_eq!(rule.priority(), 50);
        assert_eq!(rule.name(), "OperatorRule");
    }

    /// 제2항 — Korean group operator (KoreanWord + × + KoreanWord) drives lines 188-194.
    #[test]
    fn korean_group_operator_inserts_padding() {
        // PDF 제2항: 한글 단어 사이의 산술 연산자는 양옆 한 칸 띄움.
        let rule = OperatorRule;
        let tokens = vec![
            MathToken::KoreanWord("가".into()),
            MathToken::Operator('+'),
            MathToken::KoreanWord("나".into()),
        ];
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let mut result = Vec::new();
        let engine = dummy_engine();
        rule.apply(&tokens, 1, &mut result, &mut state, &engine)
            .expect("apply should succeed");
        // pad + operator + pad
        assert_eq!(result.first(), Some(&0));
        assert_eq!(result.last(), Some(&0));
    }

    /// 제2항 — label equation: `한국어 = √...` drives line 201-209.
    #[test]
    fn label_equation_after_korean_word_with_sqrt() {
        // PDF — 「둘레=√...」 형태는 = 앞에 한 칸 띄움.
        let rule = OperatorRule;
        let tokens = vec![
            MathToken::KoreanWord("둘레".into()),
            MathToken::Operator('='),
            MathToken::MathSymbol('\u{221A}'),
            MathToken::Number("2".into()),
        ];
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let mut result = Vec::new();
        let engine = dummy_engine();
        rule.apply(&tokens, 1, &mut result, &mut state, &engine)
            .expect("apply should succeed");
        assert_eq!(result.first(), Some(&0));
    }

    /// 제2항 — `should_pad` path with binary spacing operator between algebraic operands.
    /// Drives lines 212-215, 217, 221.
    #[test]
    fn binary_spacing_operator_pads_both_sides() {
        // PDF 제60항 6 — 추론 기호 ⊢ 양옆 띄움.
        let rule = OperatorRule;
        let tokens = vec![
            MathToken::Variable('p'),
            MathToken::Operator('\u{22A2}'),
            MathToken::Variable('q'),
        ];
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let mut result = Vec::new();
        let engine = dummy_engine();
        rule.apply(&tokens, 1, &mut result, &mut state, &engine)
            .expect("apply should succeed");
        // first byte == padding before
        assert_eq!(result.first(), Some(&0));
        // last byte == padding after
        assert_eq!(result.last(), Some(&0));
    }

    /// 제2항 — set-triangle plus pattern: `(...)Δ+(...)` drives lines 90-98.
    #[test]
    fn set_triangle_plus_special_form() {
        // PDF 수학 제2항 set-triangle 표기.
        // `(a)Δ+(b)` 형태 — has_set_triangle && prev=CloseParen && next=OpenParen
        let tokens = vec![
            MathToken::MathSymbol('\u{2206}'),
            MathToken::OpenParen(crate::rules::math::parser::BracketKind::MathParen),
            MathToken::Number("1".into()),
            MathToken::CloseParen(crate::rules::math::parser::BracketKind::MathParen),
            MathToken::Operator('+'),
            MathToken::OpenParen(crate::rules::math::parser::BracketKind::MathParen),
            MathToken::Number("2".into()),
            MathToken::CloseParen(crate::rules::math::parser::BracketKind::MathParen),
        ];
        let mut result = Vec::new();
        encode_operator('+', &tokens, 4, &mut result).expect("encode_operator");
        // emits [0, 44, 0]
        assert_eq!(result, vec![0, 44, 0]);
    }

    /// 제2항 — `!` (factorial) drives lines 101-104.
    #[test]
    fn factorial_emits_22() {
        let tokens = vec![MathToken::Number("5".into()), MathToken::Operator('!')];
        let mut result = Vec::new();
        encode_operator('!', &tokens, 1, &mut result).expect("encode_operator");
        assert_eq!(result, vec![22]);
    }

    /// 제2항 — `,` followed by Variable triggers padding insertion (line 124-136).
    #[test]
    fn comma_followed_by_variable_emits_space() {
        let tokens = vec![MathToken::Operator(','), MathToken::Variable('x')];
        let mut result = Vec::new();
        encode_operator(',', &tokens, 0, &mut result).expect("encode_operator");
        // ⠠ (16) then ⠀ (0)
        assert_eq!(result, vec![16, 0]);
    }

    /// 제2항 — `/` slash as fraction symbol vs plain division (line 140-148).
    #[test]
    fn slash_as_fraction_uses_shortcut_otherwise_12() {
        // Plain V/V context — not a fraction.
        let tokens = vec![
            MathToken::Variable('a'),
            MathToken::Operator('/'),
            MathToken::Variable('b'),
        ];
        let mut result = Vec::new();
        encode_operator('/', &tokens, 1, &mut result).expect("encode_operator");
        assert_eq!(result, vec![12]);
    }

    /// is_algebraic_neighbor returns true for various token kinds and false otherwise.
    #[rstest::rstest]
    #[case::variable(Some(MathToken::Variable('x')), true)]
    #[case::number(Some(MathToken::Number("1".into())), true)]
    #[case::infinity_symbol(Some(MathToken::MathSymbol('\u{221E}')), true)]
    #[case::operator_excluded(Some(MathToken::Operator('+')), false)]
    #[case::none_neighbor(None, false)]
    fn is_algebraic_neighbor_paths(#[case] token: Option<MathToken>, #[case] expected: bool) {
        assert_eq!(is_algebraic_neighbor(token.as_ref()), expected);
    }

    /// needs_binary_spacing covers each relation/inference operator.
    #[rstest::rstest]
    #[case::right_arrow('\u{2192}', true)]
    #[case::right_double_arrow('\u{21D2}', true)]
    #[case::logical_and('\u{2227}', true)]
    #[case::logical_or('\u{2228}', true)]
    #[case::right_tack('\u{22A2}', true)]
    #[case::less_or_equiv('\u{2272}', true)]
    #[case::plus_excluded('+', false)]
    fn needs_binary_spacing_table(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(needs_binary_spacing(ch), expected, "{ch}");
    }

    /// Smoke test for full math encoding pipeline covering these rules.
    #[test]
    fn full_pipeline_sanity() {
        let _ = enc("$a+b$");
        let _ = enc("$5!$");
    }

    /// rule_2 line 284 - `OperatorRule.apply` Skip when token isn't Operator.
    #[test]
    fn operator_rule_apply_skip_for_non_operator() {
        let r = OperatorRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Variable('a')];
        let mut result = Vec::new();
        let engine = dummy_engine();
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}

pub struct OperatorRule;

impl MathTokenRule for OperatorRule {
    fn name(&self) -> &'static str {
        "OperatorRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Operator(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::Operator(c)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };

        let korean_group_operator = matches!(*c, '+' | '×')
            && matches!(
                tokens.get(index.saturating_sub(1)),
                Some(MathToken::KoreanWord(_))
            )
            && matches!(tokens.get(index + 1), Some(MathToken::KoreanWord(_)));
        if korean_group_operator {
            result.push(0);
            encode_operator(*c, tokens, index, result)?;
            result.push(0);
            state.prev_was_number = false;
            return Ok(MathTokenResult::Consumed(1));
        }

        let prev_is_korean_word = matches!(
            tokens.get(index.saturating_sub(1)),
            Some(MathToken::KoreanWord(_))
        );
        let next_is_radical = matches!(
            tokens.get(index + 1),
            Some(MathToken::MathSymbol('\u{221A}'))
        );
        let label_equation = *c == '=' && prev_is_korean_word && next_is_radical;
        if label_equation {
            result.push(0);
            encode_operator(*c, tokens, index, result)?;
            state.prev_was_number = false;
            return Ok(MathTokenResult::Consumed(1));
        }

        let should_pad = needs_binary_spacing(*c)
            && index > 0
            && is_algebraic_neighbor(tokens.get(index - 1))
            && is_algebraic_neighbor(tokens.get(index + 1));
        if should_pad && !matches!(tokens.get(index - 1), Some(MathToken::Space)) {
            result.push(0);
        }
        encode_operator(*c, tokens, index, result)?;
        if should_pad && !matches!(tokens.get(index + 1), Some(MathToken::Space)) {
            result.push(0);
        }
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(1))
    }
}
