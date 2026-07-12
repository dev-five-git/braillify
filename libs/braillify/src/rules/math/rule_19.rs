//! 수학 제19항 — 아래첨자 표기.
//!
//! 아래첨자 지시(;)와 조합수 표기(₃P₁, ₃C₂)를 처리한다.

use crate::rules::math::parser::{BracketKind, MathToken};

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::rule_1;

fn single_numeric(content: &[MathToken]) -> Option<String> {
    match content {
        [MathToken::Number(n)] => Some(n.clone()),
        _ => None,
    }
}

fn prev_non_space_index(tokens: &[MathToken], mut idx: usize) -> Option<(usize, &MathToken)> {
    while idx > 0 {
        idx -= 1;
        let token = tokens.get(idx)?;
        if !matches!(token, MathToken::Space) {
            return Some((idx, token));
        }
    }
    None
}

fn prev_non_space(tokens: &[MathToken], idx: usize) -> Option<&MathToken> {
    prev_non_space_index(tokens, idx).map(|(_, token)| token)
}

fn is_plain_numeric_subscript(content: &[MathToken]) -> bool {
    content
        .iter()
        .all(|token| matches!(token, MathToken::Number(_) | MathToken::DecimalPoint))
}

pub fn should_group_subscript(content: &[MathToken]) -> bool {
    content.len() > 1
        && !matches!(
            (content.first(), content.last()),
            (
                Some(MathToken::OpenParen(BracketKind::MathParen)),
                Some(MathToken::CloseParen(BracketKind::MathParen))
            )
        )
        && !is_plain_numeric_subscript(content)
}

/// PDF 수학 제62항 — 순열/조합 묶음 안의 첨자 내용을 인코딩한다.
/// 단일 숫자/변수/연산자 조합을 평탄하게 출력한다.
fn encode_combo_subscript_content(
    content: &[MathToken],
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<(), String> {
    if let [MathToken::Number(n)] = content {
        rule_1::encode_number_literal(n, result);
        return Ok(());
    }
    engine.encode_tokens(content, result)
}

fn next_non_space(tokens: &[MathToken], mut idx: usize) -> Option<&MathToken> {
    loop {
        idx += 1;
        let token = tokens.get(idx)?;
        if !matches!(token, MathToken::Space) {
            return Some(token);
        }
    }
}

/// PDF 수학 제19항 2 — 좌하첨자(left subscript): 변수 앞에 위치한 아래첨자.
/// 좌하첨자는 다음 조건을 모두 만족할 때만 인정한다:
/// 1. 앞 토큰이 피첨자(변수/숫자/닫기괄호 등)가 아니다. (그렇지 않으면 우하첨자)
/// 2. 앞 토큰이 함수명/적분/합산 등 첨자를 매개변수로 받는 토큰이 아니다.
///    (예: `lim_{x→b}`의 첨자는 lim의 범위이며 다음 변수의 좌하첨자가 아니다.)
/// 3. 뒤 토큰이 좌하첨자의 대상이 될 변수/기호다.
fn is_left_subscript_position(tokens: &[MathToken], index: usize) -> bool {
    let prev_blocks = match prev_non_space(tokens, index) {
        // 피첨자: 이 경우는 우하첨자.
        Some(MathToken::Variable(_))
        | Some(MathToken::UpperVariable(_))
        | Some(MathToken::Number(_))
        | Some(MathToken::CloseParen(_))
        | Some(MathToken::Prime) => true,
        // 함수명(lim, sin, cos 등)은 첨자를 매개변수로 받는다.
        Some(MathToken::FunctionName(_)) => true,
        // 적분/합산/곱 등은 첨자를 한정자로 받는다.
        Some(MathToken::MathSymbol(
            '\u{222B}' // ∫
            | '\u{222C}' // ∬
            | '\u{222D}' // ∭
            | '\u{222E}' // ∮
            | '\u{2211}' // ∑
            | '\u{220F}' // ∏
            | '\u{22C3}' // ⋃
            | '\u{22C2}' // ⋂
            | '\u{2200}' // ∀
            | '\u{2203}', // ∃
        )) => true,
        _ => false,
    };
    if prev_blocks {
        return false;
    }
    // 뒤에 좌하첨자의 대상이 될 토큰이 있어야 한다.
    matches!(
        next_non_space(tokens, index),
        Some(MathToken::Variable(_))
            | Some(MathToken::UpperVariable(_))
            | Some(MathToken::MathSymbol(_))
    )
}

pub fn encode_subscript(
    tokens: &[MathToken],
    i: &mut usize,
    content: &[MathToken],
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<bool, String> {
    // PDF 수학 제62항 — 순열(_nP_r) / 조합(_nC_r) / 중복조합(_nH_r) 표기.
    // 좌하첨자 + 대문자 변수(P/C/H) + 우하첨자가 연속되면 특수 표기를 적용한다.
    //   ⠠ <letter> ⠷ left ⠀ right ⠾
    if matches!(
        tokens.get(*i + 1),
        Some(MathToken::UpperVariable('P' | 'C' | 'H'))
    ) && let Some(MathToken::Subscript(right_content)) = tokens.get(*i + 2)
        && let Some(MathToken::UpperVariable(mark)) = tokens.get(*i + 1)
    {
        result.push(32); // ⠠ (대문자 표지)
        result.push(crate::english::encode_english(mark.to_ascii_lowercase())?);
        result.push(55); // ⠷ (열린 묶음)
        encode_combo_subscript_content(content, result, engine)?;
        result.push(0);
        encode_combo_subscript_content(right_content, result, engine)?;
        result.push(62); // ⠾ (닫힌 묶음)
        *i += 3;
        return Ok(true);
    }

    // PDF 수학 제62항 4 — 중복순열(_nΠ_r) 표기.
    //   ⠠⠨⠏ ⠷ left ⠀ right ⠾
    if matches!(tokens.get(*i + 1), Some(MathToken::MathSymbol('\u{03A0}')))
        && let Some(MathToken::Subscript(right_content)) = tokens.get(*i + 2)
    {
        result.push(32); // ⠠ (대문자 표지)
        result.push(40); // ⠨ (그리스 표지)
        result.push(crate::english::encode_english('p')?); // ⠏
        result.push(55); // ⠷
        encode_combo_subscript_content(content, result, engine)?;
        result.push(0);
        encode_combo_subscript_content(right_content, result, engine)?;
        result.push(62); // ⠾
        *i += 3;
        return Ok(true);
    }

    if let Some(base) = single_numeric(content)
        && matches!(prev_non_space(tokens, *i), Some(MathToken::Number(_)))
    {
        result.push(48);
        result.push(38);
        rule_1::encode_number_literal(&base, result);
        result.push(52);
        *i += 1;
        return Ok(false);
    }

    result.push(48);
    // 적분/합/곱(∫ ∑ ∏ 등) 한정자 뒤 첨자는 묶음 없이 본문 그대로 출력한다.
    // PDF 제51항 [붙임] — `\substack`로 펼쳐진 두 번째 이상 첨자도 동일한 한정자
    // 컨텍스트에 속하므로 묶음 없이 출력한다. (이전 첨자를 거슬러 올라가 한정자를 찾는다.)
    let mut cursor = *i;
    let prev_is_quantifier_op = loop {
        match prev_non_space_index(tokens, cursor) {
            Some((
                _,
                MathToken::MathSymbol(
                    '\u{222B}' | '\u{222C}' | '\u{222D}' | '\u{222E}' | '\u{2211}' | '\u{220F}'
                    | '\u{2200}' | '\u{2203}',
                ),
            ))
            | Some((_, MathToken::FunctionName(_))) => break true,
            Some((progress, MathToken::Subscript(_))) => {
                // 이전 토큰이 첨자이면 한 단계 더 거슬러 본다 (substack 펼침 케이스).
                // prev_non_space로 Space를 건너뛰며 가장 가까운 non-Space 위치를 찾는다.
                cursor = progress;
            }
            _ => break false,
        }
    };
    if prev_is_quantifier_op {
        engine.encode_tokens(content, result)?;
        *i += 1;
        if needs_quantifier_trailing_space(tokens, *i) {
            result.push(0);
        }
        return Ok(false);
    }
    // 좌하첨자는 단일 토큰이라도 그룹 괄호로 묶는다 (PDF 제19항 2).
    let force_group = is_left_subscript_position(tokens, *i);
    if should_group_subscript(content) || force_group {
        result.push(55);
        if let [MathToken::Number(n), MathToken::Variable(v)] = content {
            rule_1::encode_number_literal(n, result);
            result.push(16);
            result.push(crate::english::encode_english(v.to_ascii_lowercase())?);
        } else if let [MathToken::Number(n), MathToken::UpperVariable(v)] = content {
            rule_1::encode_number_literal(n, result);
            result.push(16);
            result.push(crate::english::encode_english(v.to_ascii_lowercase())?);
        } else {
            engine.encode_tokens(content, result)?;
        }
        result.push(62);
    } else {
        engine.encode_tokens(content, result)?;
    }
    *i += 1;
    // PDF 수학 제56~59항 — 적분/합산/곱 등 한정자형 토큰에 붙은 첨자 뒤에 본문이
    // 이어지면 한 칸 띄움이 필요하다. (LaTeX strip이 공백을 제거하므로 명시적으로 삽입.)
    let prev_is_quantifier = matches!(
        prev_non_space(tokens, *i - 1),
        Some(MathToken::FunctionName(_))
            | Some(MathToken::MathSymbol(
                '\u{222B}' // ∫
                | '\u{222C}' // ∬
                | '\u{222D}' // ∭
                | '\u{222E}' // ∮
                | '\u{2211}' // ∑
                | '\u{220F}' // ∏
                | '\u{2200}' // ∀
                | '\u{2203}' // ∃
            ))
    );
    let needs_pad = prev_is_quantifier && needs_quantifier_trailing_space(tokens, *i);
    let pad_bytes: &[u8] = if needs_pad { &[0] } else { &[] };
    result.extend_from_slice(pad_bytes);
    Ok(false)
}

fn needs_quantifier_trailing_space(tokens: &[MathToken], idx: usize) -> bool {
    let mut cursor = idx;
    if matches!(tokens.get(cursor), Some(MathToken::Space)) {
        return false;
    }
    // Superscript이 바로 따라오면 한정자의 위첨자(예: ∫_a^b)이므로 한 칸 띄움을 보류.
    // (이 경우 위첨자 인코더가 자체적으로 한 칸 띄움을 처리한다.)
    if matches!(tokens.get(idx), Some(MathToken::Superscript(_))) {
        return false;
    }
    while cursor < tokens.len() {
        match &tokens[cursor] {
            MathToken::Space => return false,
            MathToken::Superscript(_) => return false,
            MathToken::Variable(_)
            | MathToken::UpperVariable(_)
            | MathToken::Number(_)
            | MathToken::OpenParen(_)
            | MathToken::FunctionName(_)
            | MathToken::MathSymbol(_) => return true,
            _ => cursor += 1,
        }
    }
    false
}

pub struct SubscriptRule;

impl MathTokenRule for SubscriptRule {
    fn name(&self) -> &'static str {
        "SubscriptRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Subscript(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::Subscript(content)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };
        let mut cursor = index;
        let _ = encode_subscript(tokens, &mut cursor, content, result, engine)?;
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(cursor - index))
    }
}

#[cfg(test)]
mod tests {
    use super::super::encoder::encode_math_expression;
    use super::*;

    #[test]
    fn encodes_number_base_notation_without_explicit_subscript_parentheses() {
        assert_eq!(
            encode_math_expression("1010₂").expect("math encoding should succeed"),
            vec![60, 1, 26, 1, 26, 48, 38, 60, 3, 52]
        );
    }

    #[test]
    fn encodes_number_base_notation_with_explicit_subscript_parentheses() {
        assert_eq!(
            encode_math_expression("1101₍₂₎").expect("math encoding should succeed"),
            vec![60, 1, 1, 26, 1, 48, 38, 60, 3, 52]
        );
    }

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    #[test]
    fn subscript_simple_digit() {
        let bytes = enc("$x_2$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subscript_compound_index() {
        let bytes = enc("$x_{i+1}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subscript_quantifier_with_following_var() {
        // ∑_{i=1}^n i — subscript follows quantifier, then superscript path
        let bytes = enc("$\\sum_{i=1}^{n} i$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subscript_after_function_then_paren() {
        // log_2(x) — exercise subscript after function name, then paren arg
        let bytes = enc("$\\log_{2}(x)$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subscript_multi_digit_index() {
        let bytes = enc("$a_{12}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subscript_with_negative_index() {
        let bytes = enc("$x_{-1}$");
        assert!(!bytes.is_empty());
    }

    /// 제19항 — `needs_quantifier_trailing_space`의 토큰별 분기.
    /// Variable/Number/FunctionName/OpenParen/MathSymbol/UpperVariable → true,
    /// Space/Superscript/empty/Operator-only-tail → false.
    #[rstest::rstest]
    #[case::space_false(vec![MathToken::Space], false)]
    #[case::variable_true(vec![MathToken::Variable('x')], true)]
    #[case::superscript_at_idx_false(
        vec![MathToken::Superscript(vec![MathToken::Number("2".into())])],
        false,
    )]
    #[case::number_true(vec![MathToken::Number("1".into())], true)]
    #[case::empty_false(vec![], false)]
    #[case::function_name_true(vec![MathToken::FunctionName("sin".into())], true)]
    #[case::open_paren_true(vec![MathToken::OpenParen(BracketKind::MathParen)], true)]
    #[case::math_symbol_true(vec![MathToken::MathSymbol('+')], true)]
    #[case::upper_variable_true(vec![MathToken::UpperVariable('X')], true)]
    #[case::operator_tail_empty_false(
        vec![MathToken::Operator('+'), MathToken::Operator('+')],
        false,
    )]
    fn needs_quantifier_trailing_space_branches(
        #[case] tokens: Vec<MathToken>,
        #[case] expected: bool,
    ) {
        assert_eq!(needs_quantifier_trailing_space(&tokens, 0), expected);
    }

    #[test]
    fn subscript_rule_priority_and_name() {
        let r = SubscriptRule;
        assert_eq!(r.priority(), 50);
        assert_eq!(r.name(), "SubscriptRule");
    }

    /// 제19항 — is_left_subscript_position: blocked by function name (line 89).
    #[test]
    fn left_subscript_position_blocked_by_function_name() {
        let toks = vec![
            MathToken::FunctionName("lim".into()),
            MathToken::Subscript(vec![MathToken::Variable('n')]),
            MathToken::Variable('x'),
        ];
        assert!(!is_left_subscript_position(&toks, 1));
    }

    /// 제19항 — is_left_subscript_position: blocked by quantifier (line 91-102).
    #[test]
    fn left_subscript_position_blocked_by_universal_quantifier() {
        let toks = vec![
            MathToken::MathSymbol('\u{2200}'),
            MathToken::Subscript(vec![MathToken::Variable('x')]),
            MathToken::Variable('y'),
        ];
        assert!(!is_left_subscript_position(&toks, 1));
    }

    /// 제19항 — substack scan: prev is Subscript (line 185-198).
    #[test]
    fn subscript_after_substack_chain() {
        // ∫_{...}_{...} substack scan path via full pipeline.
        let bytes = enc("$\\sum_{i=1}\\substack{j=1}$");
        let _ = bytes;
    }

    #[test]
    fn subscript_after_quantifier_encodes_content_without_grouping() {
        let tokens = vec![
            MathToken::MathSymbol('\u{2211}'),
            MathToken::Subscript(vec![MathToken::Variable('i')]),
        ];
        let mut i = 1;
        let mut result = Vec::new();
        let engine = super::super::encoder::math_engine_for_context(
            super::super::math_token_rule::MathContext::default(),
        );

        let grouped = encode_subscript(
            &tokens,
            &mut i,
            &[MathToken::Variable('i')],
            &mut result,
            engine,
        )
        .expect("quantifier subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(
            result,
            vec![48, crate::english::encode_english('i').unwrap()]
        );
    }

    #[test]
    fn subscript_after_subscript_without_quantifier_scans_back_to_false() {
        let tokens = vec![
            MathToken::Variable('x'),
            MathToken::Subscript(vec![MathToken::Variable('i')]),
            MathToken::Subscript(Vec::new()),
        ];
        let mut i = 2;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 3);
    }

    #[test]
    fn plain_variable_subscript_uses_group_path_without_quantifier_context() {
        let bytes = enc("$a_{bc}$");
        assert!(!bytes.is_empty());
    }

    /// 제19항 — Number + UpperVariable subscript content drives lines 219-222.
    #[test]
    fn subscript_with_number_upper_var_content() {
        // a_{1X} via pipeline.
        let bytes = enc("$a_{1X}$");
        assert!(!bytes.is_empty());
    }

    /// 제19항 — quantifier trailing space insertion (lines 247-249).
    #[test]
    fn quantifier_trailing_space_after_subscript() {
        // \\sum_{i=1} f(x) drives the trailing space insertion path.
        let bytes = enc("$\\sum_{i=1}f(x)$");
        assert!(!bytes.is_empty());
    }

    /// 제19항 — should_group_subscript: paren-wrapped content returns false (lines 38-46).
    #[rstest::rstest]
    #[case::paren_wrapped_no_group(
        vec![
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('a'),
            MathToken::CloseParen(BracketKind::MathParen),
        ],
        false,
    )]
    #[case::multi_token_non_numeric_groups(
        vec![MathToken::Variable('a'), MathToken::Operator('+'), MathToken::Variable('b')],
        true,
    )]
    fn should_group_subscript_paren_wrapped_content_skipped(
        #[case] content: Vec<MathToken>,
        #[case] expected: bool,
    ) {
        assert_eq!(should_group_subscript(&content), expected);
    }

    #[test]
    fn single_numeric_extracts_number_content() {
        assert_eq!(
            single_numeric(&[MathToken::Number("7".into())]),
            Some("7".into())
        );
    }

    #[test]
    fn numeric_subscript_after_number_uses_plain_numeric_subscript_path() {
        let tokens = vec![
            MathToken::Number("2".into()),
            MathToken::Subscript(vec![MathToken::Number("3".into())]),
        ];
        let mut i = 1;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(
            &tokens,
            &mut i,
            &[MathToken::Number("3".into())],
            &mut result,
            &engine,
        )
        .expect("numeric subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(result, vec![48, 38, 60, 9, 52]);
    }

    /// 제19항 — encode_combo_subscript_content via _nP_r pattern drives line 303.
    #[test]
    fn left_subscript_combinatorics_pattern() {
        // ₂P₃ style via pipeline
        let bytes = enc("$\\sum_{n}P_{r}$");
        let _ = bytes;
    }

    /// 제19항 — needs_quantifier_trailing_space: while-loop encounters Space
    /// after advancing past an Operator/other token (line 265).
    #[test]
    fn needs_quantifier_trailing_space_loop_encounters_space() {
        // Operator at idx=0, Space at idx=1. cursor=0 starts; line 256 not triggered
        // (cursor=idx=0, tokens[0]=Operator → not Space). Line 260 also not (not Superscript).
        // While-loop hits `_` arm at 273 → cursor=1, then matches Space at 265 → false.
        let toks = vec![MathToken::Operator(','), MathToken::Space];
        assert!(!needs_quantifier_trailing_space(&toks, 0));
    }

    /// 제19항 — needs_quantifier_trailing_space: while-loop encounters Superscript
    /// after advancing (line 266).
    #[test]
    fn needs_quantifier_trailing_space_loop_encounters_superscript() {
        // Operator advances cursor, then Superscript at cursor=1 → return false.
        let toks = vec![
            MathToken::Operator(','),
            MathToken::Superscript(vec![MathToken::Number("2".into())]),
        ];
        assert!(!needs_quantifier_trailing_space(&toks, 0));
    }

    /// 제19항 — SubscriptRule.apply with non-Subscript token at index returns Skip (line 303).
    #[test]
    fn subscript_rule_apply_with_non_subscript_returns_skip() {
        let r = SubscriptRule;
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

    #[test]
    fn subscript_after_subscript_without_quantifier_uses_regular_group_path() {
        let tokens = vec![
            MathToken::Subscript(vec![MathToken::Variable('i')]),
            MathToken::Subscript(Vec::new()),
        ];
        let mut i = 1;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn leading_subscript_without_previous_token_is_regular_group() {
        let tokens = vec![MathToken::Subscript(Vec::new())];
        let mut i = 0;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 1);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn subscript_after_only_space_scans_to_false() {
        let tokens = vec![MathToken::Space, MathToken::Subscript(Vec::new())];
        let mut i = 1;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn subscript_after_empty_prefix_scans_to_false() {
        let tokens = vec![MathToken::Subscript(Vec::new())];
        let mut i = 0;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 1);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn variable_subscript_without_quantifier_emits_regular_group() {
        let tokens = vec![MathToken::Variable('x'), MathToken::Subscript(Vec::new())];
        let mut i = 1;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn subscript_after_previous_subscript_at_start_scans_to_false() {
        let tokens = vec![
            MathToken::Subscript(Vec::new()),
            MathToken::Subscript(Vec::new()),
        ];
        let mut i = 1;
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 2);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn subscript_after_runtime_previous_subscript_without_progress_scans_false() {
        let tokens = vec![
            MathToken::Subscript(Vec::new()),
            MathToken::Subscript(Vec::new()),
        ];
        let mut i = std::hint::black_box(1);
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
    }

    #[test]
    fn subscript_after_space_then_prior_subscript_at_start_scans_to_false() {
        let tokens = vec![
            MathToken::Subscript(Vec::new()),
            MathToken::Space,
            MathToken::Subscript(Vec::new()),
        ];
        let mut i = std::hint::black_box(2);
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());

        let grouped = encode_subscript(&tokens, &mut i, &[], &mut result, &engine)
            .expect("subscript should encode");

        assert!(!grouped);
        assert_eq!(i, 3);
        assert_eq!(result, vec![48]);
    }

    #[test]
    fn prev_non_space_index_skips_space_to_previous_token() {
        let tokens = vec![
            MathToken::Variable('x'),
            MathToken::Space,
            MathToken::Subscript(Vec::new()),
        ];

        let found = prev_non_space_index(&tokens, 2);

        assert!(matches!(found, Some((0, MathToken::Variable('x')))));
    }

    #[test]
    fn should_group_subscript_rejects_runtime_single_token_content() {
        let content = vec![MathToken::Number(std::hint::black_box("1".to_string()))];

        assert!(!should_group_subscript(&content));
        assert!(!should_group_subscript(&[]));
    }
}
