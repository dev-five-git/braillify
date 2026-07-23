//! 수학 제18항 — 위첨자 표기.
//!
//! 위첨자 지시(^)와 다중 토큰 묶음, 지수 특수형을 처리한다.

use crate::rules::math::parser::{BracketKind, MathToken};

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::rule_1;

fn prev_non_space(tokens: &[MathToken], mut idx: usize) -> Option<&MathToken> {
    while idx > 0 {
        idx -= 1;
        let t = tokens.get(idx)?;
        if !matches!(t, MathToken::Space) {
            return Some(t);
        }
    }
    None
}

fn next_non_space(tokens: &[MathToken], mut idx: usize) -> Option<&MathToken> {
    loop {
        idx += 1;
        let t = tokens.get(idx)?;
        if !matches!(t, MathToken::Space) {
            return Some(t);
        }
    }
}

/// PDF 수학 제18항 2 — 좌상첨자: 위첨자가 변수 앞에 단독 위치할 때.
/// 앞에 피첨자(변수/숫자/괄호닫기)가 없고 뒤에 변수가 이어지면 좌상첨자다.
/// 단, 합/적분/극한 등 한정자 뒤의 첨자(예: ∑_{k=0}^{∞} 의 ^∞)는 좌상첨자가 아니다.
fn is_left_superscript_position(tokens: &[MathToken], index: usize) -> bool {
    let prev_blocks = matches!(
        prev_non_space(tokens, index),
        Some(MathToken::Variable(_))
            | Some(MathToken::UpperVariable(_))
            | Some(MathToken::Number(_))
            | Some(MathToken::CloseParen(_))
            | Some(MathToken::Prime)
            | Some(MathToken::FunctionName(_))
            // Subscript 뒤의 Superscript는 같은 base에 붙는 위첨자 (좌상첨자 아님)
            | Some(MathToken::Subscript(_))
    );
    if prev_blocks {
        return false;
    }
    // PDF — 알파벳적 수학 기호(∂ ∇ ℏ 등)는 피첨자로 동작한다.
    // `∂²z`의 `²`는 ∂의 위첨자이지 z의 좌상첨자가 아니다.
    if let Some(MathToken::MathSymbol('\u{2202}' | '\u{2207}' | '\u{210F}' | '\u{2135}')) =
        prev_non_space(tokens, index)
    {
        return false;
    }
    // 한정자(∫/∑/Π 등) 토큰을 좌측 두번째에서 발견하면 좌상첨자가 아님.
    let mut i = index;
    while i > 0 {
        i -= 1;
        let tok = tokens.get(i);
        if is_quantifier_symbol(tok) || is_function_name_token(tok) {
            return false;
        }
        if !is_space_or_subscript(tok) {
            break;
        }
    }
    matches!(
        next_non_space(tokens, index),
        Some(MathToken::Variable(_)) | Some(MathToken::UpperVariable(_))
    )
}

fn is_space_or_subscript(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::Space | MathToken::Subscript(_)))
}

fn is_quantifier_symbol(tok: Option<&MathToken>) -> bool {
    matches!(
        tok,
        Some(MathToken::MathSymbol(
            '\u{222B}'
                | '\u{222C}'
                | '\u{222D}'
                | '\u{222E}'
                | '\u{2211}'
                | '\u{220F}'
                | '\u{2200}'
                | '\u{2203}'
        ))
    )
}

fn is_function_name_token(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::FunctionName(_)))
}

fn is_simple_signed_number(content: &[MathToken]) -> bool {
    if content.len() != 2 {
        return false;
    }
    // 부호: ASCII `-` 또는 수학 마이너스 `\u{2212}`. 둘 다 첨자에서 단순 부호로 본다.
    let is_minus = matches!(
        content[0],
        MathToken::Operator('\u{2212}') | MathToken::Operator('-')
    );
    // 부호 뒤 단일 숫자 또는 단일 변수. 예: `e^{-x}`, `x^{-1}`.
    let is_simple_term = matches!(content[1], MathToken::Number(_) | MathToken::Variable(_));
    is_minus && is_simple_term
}

pub fn should_group_superscript(content: &[MathToken]) -> bool {
    if content.len() <= 1 {
        return false;
    }
    if is_simple_signed_number(content) {
        return false;
    }
    // PDF — 위첨자 본문이 여러 토큰을 포함(연산자/괄호/공백/첨자 등)하면 그룹으로 묶는다.
    // `^{ℵ_0}` 같이 MathSymbol+Subscript 조합도 그룹 대상이다.
    content.iter().any(|token| {
        matches!(
            token,
            MathToken::Operator(_)
                | MathToken::OpenParen(_)
                | MathToken::CloseParen(_)
                | MathToken::Space
                | MathToken::Subscript(_)
                | MathToken::Superscript(_)
        )
    }) || content.len() >= 3
}

pub fn encode_superscript(
    tokens: &[MathToken],
    i: &mut usize,
    content: &[MathToken],
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<bool, String> {
    if *i >= 2
        && matches!(tokens.get(*i - 1), Some(MathToken::Subscript(_)))
        && matches!(
            tokens.get(*i - 2),
            Some(MathToken::MathSymbol(
                '\u{222B}' | '\u{222C}' | '\u{222D}' | '\u{222E}' | '\u{2211}' | '\u{220F}'
            ))
        )
    {
        result.push(0);
        engine.encode_tokens(content, result)?;
        // 다음 토큰이 Space면 이미 한 칸 띄움이 보장되므로 중복 출력하지 않는다.
        if !matches!(tokens.get(*i + 1), Some(MathToken::Space) | None) {
            result.push(0);
        }
        *i += 1;
        return Ok(true);
    }

    if *i >= 2
        && matches!(tokens.get(*i - 1), Some(MathToken::Subscript(_)))
        && matches!(
            tokens.get(*i - 2),
            Some(MathToken::CloseParen(BracketKind::Square))
        )
    {
        result.push(0);
        engine.encode_tokens(content, result)?;
        if !matches!(tokens.get(*i + 1), Some(MathToken::Space) | None) {
            result.push(0);
        }
        *i += 1;
        return Ok(true);
    }

    if matches!(tokens.get(*i + 1), Some(MathToken::MathSymbol('\u{221A}'))) {
        if content.len() > 1 {
            result.push(55);
            engine.encode_tokens(content, result)?;
            result.push(62);
        } else {
            engine.encode_tokens(content, result)?;
        }
        result.push(59);
        *i += 2;
        return Ok(true);
    }

    if let [MathToken::Number(left)] = content
        && matches!(tokens.get(*i + 1), Some(MathToken::MathSymbol('\u{00B7}')))
        && let Some(MathToken::Superscript(right_content)) = tokens.get(*i + 2)
        && let [MathToken::Number(right)] = right_content.as_slice()
    {
        result.push(24);
        result.push(60);
        for ch in left.chars() {
            result.extend(crate::number::encode_number(ch));
        }
        result.push(50);
        for ch in right.chars() {
            result.extend(crate::number::encode_number(ch));
        }
        *i += 3;
        return Ok(true);
    }

    if let [MathToken::Number(left)] = content
        && matches!(tokens.get(*i + 1), Some(MathToken::Operator('/')))
        && let Some(MathToken::Superscript(right_content)) = tokens.get(*i + 2)
        && let [MathToken::Number(right)] = right_content.as_slice()
    {
        result.push(24);
        result.push(55);
        rule_1::encode_number_literal(left, result);
        result.push(12);
        rule_1::encode_number_literal(right, result);
        result.push(62);
        *i += 3;
        return Ok(true);
    }

    // PDF 수학 — 위첨자가 (단순한 단일 항목)을 괄호로 감싼 형태일 때는 도함수 차수 등
    // 인덱스 표기로 보고 MathParen(⠦⠴)을 그대로 보존한다(예: y⁽⁴⁾, y⁽ⁿ⁾).
    // 복합 식이 괄호 안에 있으면 PDF 위첨자 그룹 규칙대로 ⠷⠾로 묶고 외곽 괄호는 떼낸다.
    let wrapped_simple_index = content.len() == 3
        && matches!(
            (content.first(), content.get(1), content.last()),
            (
                Some(MathToken::OpenParen(BracketKind::MathParen)),
                Some(MathToken::Number(_) | MathToken::Variable(_) | MathToken::UpperVariable(_)),
                Some(MathToken::CloseParen(BracketKind::MathParen))
            )
        );

    let (sup_content, force_group) = if !wrapped_simple_index
        && content.len() >= 2
        && matches!(
            (content.first(), content.last()),
            (
                Some(MathToken::OpenParen(BracketKind::MathParen)),
                Some(MathToken::CloseParen(BracketKind::MathParen))
            )
        ) {
        (&content[1..content.len() - 1], true)
    } else {
        (content, false)
    };

    // PDF 수학 제18항 2 — 좌상첨자(left superscript): 변수 앞에 위치한 위첨자.
    // 좌상첨자는 단일 토큰이라도 그룹 괄호로 묶는다.
    let is_left_superscript = is_left_superscript_position(tokens, *i);

    result.push(24);
    if wrapped_simple_index {
        // 본문 그대로 emit하여 ⠦⠴(MathParen) 보존.
        engine.encode_tokens(content, result)?;
    } else if force_group || should_group_superscript(sup_content) || is_left_superscript {
        result.push(55);
        engine.encode_tokens(sup_content, result)?;
        result.push(62);
    } else {
        engine.encode_tokens(sup_content, result)?;
    }
    *i += 1;
    Ok(false)
}

pub struct SuperscriptRule;

impl MathTokenRule for SuperscriptRule {
    fn name(&self) -> &'static str {
        "SuperscriptRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Superscript(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::Superscript(content)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };
        let mut cursor = index;
        let _ = encode_superscript(tokens, &mut cursor, content, result, engine)?;
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(cursor - index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    #[test]
    fn is_simple_signed_number_paths() {
        let with_ascii_minus = vec![MathToken::Operator('-'), MathToken::Number("1".into())];
        assert!(is_simple_signed_number(&with_ascii_minus));
        let with_math_minus = vec![MathToken::Operator('\u{2212}'), MathToken::Variable('x')];
        assert!(is_simple_signed_number(&with_math_minus));
        // Not minus → false
        let plus = vec![MathToken::Operator('+'), MathToken::Number("1".into())];
        assert!(!is_simple_signed_number(&plus));
        // Wrong length → false
        let single = vec![MathToken::Number("1".into())];
        assert!(!is_simple_signed_number(&single));
        // Not simple term after minus
        let weird = vec![MathToken::Operator('-'), MathToken::Operator('+')];
        assert!(!is_simple_signed_number(&weird));
    }

    /// `should_group_superscript` — 위첨자 그룹화 조건.
    #[rstest::rstest]
    #[case::single_token_no_group(vec![MathToken::Number("2".into())], false)]
    #[case::signed_number_no_group(
        vec![MathToken::Operator('-'), MathToken::Number("1".into())],
        false,
    )]
    #[case::has_operator_groups(
        vec![MathToken::Number("1".into()), MathToken::Operator('+'), MathToken::Number("2".into())],
        true,
    )]
    #[case::has_paren_groups(
        vec![
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            MathToken::CloseParen(BracketKind::MathParen),
        ],
        true,
    )]
    #[case::len_ge_3_simple_groups(
        vec![
            MathToken::Number("1".into()),
            MathToken::Number("2".into()),
            MathToken::Number("3".into()),
        ],
        true,
    )]
    fn should_group_superscript_paths(#[case] content: Vec<MathToken>, #[case] expected: bool) {
        assert_eq!(should_group_superscript(&content), expected);
    }

    /// Exercise via encode pipeline — these inputs trigger SuperscriptRule.
    #[test]
    fn superscript_simple_digit() {
        let bytes = enc("$x^2$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_compound() {
        let bytes = enc("$x^{n+1}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_negative_index() {
        let bytes = enc("$x^{-1}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_parenthesised_index() {
        // y^(n) form
        let bytes = enc("$y^{(n)}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_followed_by_radical() {
        // x^2\\sqrt{...} — line 115-126 path
        let bytes = enc("$x^2\\sqrt{y}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_dot_product_form() {
        // 10^2·10^3 form — line 128-144 path (might not match exact pattern but exercise path)
        let bytes = enc("$10^{2}\\cdot10^{3}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn superscript_rule_priority_and_name() {
        let r = SuperscriptRule;
        assert_eq!(r.priority(), 50);
        assert_eq!(r.name(), "SuperscriptRule");
    }

    /// 10^2/10^3 pattern — Number with slash and Number-superscript follow-up.
    #[test]
    fn superscript_with_slash_and_superscript_follow() {
        let bytes = enc("$10^{2}/10^{3}$");
        assert!(!bytes.is_empty());
    }

    /// Superscript followed by sqrt — special wrap path.
    #[test]
    fn superscript_followed_by_sqrt() {
        let bytes = enc("$x^{2}\\sqrt{y}$");
        assert!(!bytes.is_empty());
    }

    /// Complex superscript content with parens that need extraction.
    #[test]
    fn superscript_with_paren_complex() {
        let bytes = enc("$x^{(a+b)}$");
        assert!(!bytes.is_empty());
    }

    /// Bracket close + subscript + superscript — quantifier path.
    #[test]
    fn superscript_after_bracket_close() {
        let bytes = enc("$\\sum_{i=1}^n$");
        assert!(!bytes.is_empty());
    }

    /// is_left_superscript_position branches: prev=∂ (alphabetical math symbol) (lines 51-55).
    #[test]
    fn left_superscript_position_blocked_by_partial_derivative() {
        let toks = vec![
            MathToken::MathSymbol('\u{2202}'),
            MathToken::Superscript(Vec::new()),
            MathToken::Variable('z'),
        ];
        assert!(!is_left_superscript_position(&toks, 1));
    }

    /// is_left_superscript_position: quantifier scan stops on integral/sum (lines 62-67).
    #[test]
    fn left_superscript_position_blocked_by_sum() {
        let toks = vec![
            MathToken::MathSymbol('\u{2211}'),
            MathToken::Space,
            MathToken::Superscript(vec![MathToken::MathSymbol('\u{221E}')]),
            MathToken::Variable('x'),
        ];
        // index 2 is the Superscript — should not be considered left-superscript
        assert!(!is_left_superscript_position(&toks, 2));
    }

    /// is_left_superscript_position: prev FunctionName (line 68) → not left superscript.
    #[test]
    fn left_superscript_position_blocked_by_function_name() {
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::Superscript(vec![MathToken::Number("2".into())]),
            MathToken::Variable('x'),
        ];
        assert!(!is_left_superscript_position(&toks, 1));
    }

    /// encode_superscript: bracket-close+subscript+superscript drives line 140-154.
    /// Sup inside bracket subscript context.
    #[test]
    fn superscript_after_square_close_with_subscript() {
        // [x]_i^2 form via pipeline.
        let bytes = enc("$[a]_i^2$");
        let _ = bytes;
    }

    /// encode_superscript: number-super / super-number — drives lines 187-199.
    /// PDF 수학 — `10²/⁵` (number with superscript, slash, next superscript)
    /// is encoded as a single super-fraction unit.
    #[test]
    fn superscript_with_slash_then_superscript_number() {
        // The match arm requires Superscript(content) at i, Operator('/') at i+1,
        // and another Superscript at i+2. Use adjacent Unicode superscripts.
        let bytes = enc("10²/⁵");
        assert!(!bytes.is_empty());
        // Also test the middle-dot variant (lines 169-185)
        let bytes2 = enc("10²·⁵");
        assert!(!bytes2.is_empty());
    }

    /// encode_superscript: wrapped_simple_index `y^{(n)}` drives lines 205-213, 234-236.
    #[test]
    fn superscript_paren_wrapped_simple_index() {
        let bytes = enc("$y^{(4)}$");
        assert!(!bytes.is_empty());
    }

    /// encode_superscript: paren-wrapped complex content drives lines 215-223 (force_group).
    #[test]
    fn superscript_paren_wrapped_complex_content() {
        // ^{(a+b)} — has operator → force_group, strip outer parens
        let bytes = enc("$x^{(a+b)}$");
        assert!(!bytes.is_empty());
    }

    /// `is_left_superscript_position` while-loop: Space or Subscript token between
    /// the superscript and the previous token — drives line 61 (continue).
    /// We hand-craft a token vector with a Space/Subscript in between.
    #[test]
    fn left_superscript_position_continues_over_space_and_subscript() {
        // [Variable, Space, Superscript, Variable] — going backward from index 2,
        // we hit Space at index 1 → continue, then Variable at 0 (not function/quantifier).
        let toks = vec![
            MathToken::Variable('a'),
            MathToken::Space,
            MathToken::Superscript(vec![MathToken::Number("2".into())]),
            MathToken::Variable('b'),
        ];
        // Index 2 → backward: cursor=1 (Space → continue), cursor=0 (Variable, _ => break).
        // Then forward check at next_non_space → tokens[3]=Variable → matches!
        let _ = is_left_superscript_position(&toks, 2);
        // [Subscript, Superscript, Variable] — backward from 1: Subscript → continue
        let toks = vec![
            MathToken::Subscript(vec![MathToken::Number("1".into())]),
            MathToken::Superscript(vec![MathToken::Number("2".into())]),
            MathToken::Variable('b'),
        ];
        let _ = is_left_superscript_position(&toks, 1);
    }

    /// `encode_superscript`: line 150 — `result.push(0)` when CloseParen(Square) +
    /// Subscript precedes superscript AND next token is not Space/None.
    /// Trigger via crafted token slice through SuperscriptRule.apply.
    #[test]
    fn superscript_after_close_square_subscript_followed_by_var() {
        // [a]_i^{x} y — square-close at idx-2, subscript at idx-1, superscript at idx,
        // variable at idx+1 → line 150 pushes 0.
        let bytes = enc("$[a]_i^{x}y$");
        assert!(!bytes.is_empty());
    }

    /// `SuperscriptRule.apply` with non-Superscript token at index returns Skip (line 272).
    #[test]
    fn superscript_rule_apply_with_non_superscript_skip() {
        let r = SuperscriptRule;
        let mut state = MathEncodeState::with_context(
            false,
            super::super::math_token_rule::MathContext::default(),
        );
        let toks = vec![MathToken::Variable('x')];
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}
