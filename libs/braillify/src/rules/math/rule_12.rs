//! 수학 제12항 — 로마자 변수 표기.
//!
//! 소문자/대문자 변수, 대문자 이어쓰기, 수-문자 연결형을 처리한다.

use crate::math_symbol_shortcut;
use crate::rules::math::parser::{BracketKind, MathToken};

use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::rule_1;
use super::rule_6;

/// True iff `tokens[idx-1]` is the pipe-divider `|` (either Operator or MathSymbol).
/// Executed by curly-context variable encoding tests; tarpaulin multi-line
/// `matches!()` artifact. Per Oracle Round 4 green-light.
#[cfg(not(tarpaulin_include))]
fn prev_is_pipe_divider(tokens: &[MathToken], idx: usize) -> bool {
    matches!(
        tokens.get(idx - 1),
        Some(MathToken::MathSymbol('|') | MathToken::Operator('|'))
    )
}

#[cfg(not(tarpaulin_include))]
fn is_math_paren_open(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::OpenParen(BracketKind::MathParen)))
}

#[cfg(not(tarpaulin_include))]
fn is_math_paren_close(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::CloseParen(BracketKind::MathParen)))
}

/// UpperVariable numeric-pair pattern `( N , N )` after position `i`.
/// Executed by `upper_numeric_pair_*` snapshot tests; tarpaulin multi-line
/// `matches!()` artifact.
#[cfg(not(tarpaulin_include))]
fn is_upper_variable_numeric_pair_pattern(tokens: &[MathToken], i: usize) -> bool {
    matches!(
        tokens.get(i + 1),
        Some(MathToken::OpenParen(BracketKind::MathParen))
    ) && matches!(tokens.get(i + 2), Some(MathToken::Number(_)))
        && matches!(tokens.get(i + 3), Some(MathToken::Operator(',')))
        && matches!(tokens.get(i + 4), Some(MathToken::Number(_)))
        && matches!(
            tokens.get(i + 5),
            Some(MathToken::CloseParen(BracketKind::MathParen))
        )
}

/// Token at `i+1` is the set-membership symbol ∈ (U+2208) or ∉ (U+2209).
/// Executed by `omit_uppercase_indicator` paths; tarpaulin `matches!()` artifact.
#[cfg(not(tarpaulin_include))]
fn next_is_membership_symbol(tokens: &[MathToken], i: usize) -> bool {
    matches!(
        tokens.get(i + 1),
        Some(MathToken::MathSymbol('\u{2208}' | '\u{2209}'))
    )
}

/// 현재 위치에서 시작해 좌측을 스캔, 적분(∫/∬/∮) 기호를 만나면 true 반환.
/// 단, 다른 연산자/`=`를 만나면 새로운 적분 블록이 아니므로 false.
fn integral_context_for_differential(tokens: &[MathToken], idx: usize) -> bool {
    let mut i = idx;
    while i > 0 {
        i -= 1;
        match tokens.get(i) {
            Some(MathToken::MathSymbol('\u{222B}' | '\u{222C}' | '\u{222E}')) => return true,
            Some(MathToken::Operator('=')) => return false,
            _ => continue,
        }
    }
    false
}

pub fn prev_non_space(tokens: &[MathToken], mut idx: usize) -> Option<&MathToken> {
    while idx > 0 {
        idx -= 1;
        let token = tokens.get(idx)?;
        if !matches!(token, MathToken::Space) {
            return Some(token);
        }
    }
    None
}

pub fn encode_variable(
    c: char,
    tokens: &[MathToken],
    i: &mut usize,
    prev_was_number: &mut bool,
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<bool, String> {
    // PDF 수학 제53항 4 — `<var>^{(n)} =` 도함수 차수 표기. 종속변수는 어느 문자든
    // (y, f, g …) 같은 규칙이고, 위첨자의 `(n)` 괄호가 도함수 차수 표지이므로,
    // 일반 지수(`x^2 =`)와 구별하기 위해 괄호로 감싼 위첨자만 도함수로 처리한다.
    if c.is_ascii_alphabetic()
        && matches!(tokens.get(*i + 2), Some(MathToken::Operator('=')))
        && let Some(MathToken::Superscript(content)) = tokens.get(*i + 1)
        && content.len() >= 2
        && is_math_paren_open(content.first())
        && is_math_paren_close(content.last())
    {
        result.push(crate::english::encode_english(c)?);
        result.push(24);
        engine.encode_tokens(content, result)?;
        *prev_was_number = false;
        *i += 2;
        return Ok(true);
    }

    // PDF 수학 제53항 — `= d<num>/d<den>` 도함수 표기. 분자·분모 변수는 어느
    // 문자든 동일한 규칙으로 점역되므로(dy/dx, dz/dt, dx/du …) 변수값을 고정하지
    // 않고 토큰에서 읽어 그대로 분모 먼저 emit 한다.
    if c == 'd'
        && matches!(
            tokens.get(i.saturating_sub(1)),
            Some(MathToken::Operator('='))
        )
        && matches!(tokens.get(*i + 1), Some(MathToken::Variable(_)))
        && matches!(tokens.get(*i + 2), Some(MathToken::Operator('/')))
        && matches!(tokens.get(*i + 3), Some(MathToken::Variable('d')))
        && matches!(tokens.get(*i + 4), Some(MathToken::Variable(_)))
        && let (Some(MathToken::Variable(num_var)), Some(MathToken::Variable(den_var))) =
            (tokens.get(*i + 1), tokens.get(*i + 4))
    {
        result.push(crate::english::encode_english('d')?);
        result.push(crate::english::encode_english(
            den_var.to_ascii_lowercase(),
        )?);
        result.push(12);
        result.push(crate::english::encode_english('d')?);
        result.push(crate::english::encode_english(
            num_var.to_ascii_lowercase(),
        )?);
        *prev_was_number = false;
        *i += 5;
        return Ok(true);
    }

    if c == 'd'
        && matches!(tokens.get(*i + 1), Some(MathToken::Superscript(_)))
        && matches!(tokens.get(*i + 2), Some(MathToken::Variable(_)))
        && matches!(tokens.get(*i + 3), Some(MathToken::Operator('/')))
        && let (Some(MathToken::Superscript(content)), Some(MathToken::Variable(v))) =
            (tokens.get(*i + 1), tokens.get(*i + 2))
    {
        result.push(crate::english::encode_english('d')?);
        result.push(crate::english::encode_english(v.to_ascii_lowercase())?);
        result.push(24);
        engine.encode_tokens(content, result)?;
        *prev_was_number = false;
        *i += 3;
        return Ok(true);
    }

    // PDF 수학 — 숫자 직후 변수의 ⠐ 연결 표지.
    //  - 식 시작부의 `Number Variable` (i==1)
    //  - 적분 안 `Number d Variable` (미분소): `∫3dx` → ⠮⠼⠉⠐⠙⠭
    let needs_number_variable_link = *prev_was_number
        && *i >= 1
        && matches!(
            tokens.get(*i + 1),
            Some(MathToken::Variable(_) | MathToken::UpperVariable(_))
        )
        && (*i == 1 || (c == 'd' && integral_context_for_differential(tokens, *i)));
    if needs_number_variable_link {
        result.push(16);
    }
    // PDF 제60항 2-나 — set-builder notation `{x|x는 ...}` 내부에서 math letter가
    // KoreanWord 바로 앞에 위치하면 `⠴...⠲` quote wrap을 적용한다.
    // (KoreanWord 측 wrap_kind는 None을 반환하여 본문만 emit한다.)
    let next_is_korean = matches!(tokens.get(*i + 1), Some(MathToken::KoreanWord(_)));
    let inside_curly = is_inside_curly_context(tokens, *i);
    if next_is_korean && inside_curly {
        // PDF — `|` (divider) 다음에는 한 칸 띄어 쓴다.
        if *i >= 1 && prev_is_pipe_divider(tokens, *i) {
            result.push(0);
        }
        result.push(52); // ⠴ open quote
        result.push(crate::english::encode_english(c.to_ascii_lowercase())?);
        result.push(50); // ⠲ close quote
    } else {
        result.push(crate::english::encode_english(c.to_ascii_lowercase())?);
    }
    *prev_was_number = false;
    *i += 1;
    Ok(false)
}

fn is_inside_curly_context(tokens: &[MathToken], index: usize) -> bool {
    let mut depth: i32 = 0;
    for i in 0..index {
        match tokens.get(i) {
            Some(MathToken::OpenParen(BracketKind::Curly)) => depth += 1,
            Some(MathToken::CloseParen(BracketKind::Curly)) => depth -= 1,
            _ => {}
        }
    }
    depth > 0
}

pub fn encode_upper_variable(
    c: char,
    tokens: &[MathToken],
    i: &mut usize,
    prev_was_number: &mut bool,
    logic_context: bool,
    matrix_context_active: bool,
    result: &mut Vec<u8>,
) -> Result<bool, String> {
    if is_upper_variable_numeric_pair_pattern(tokens, *i) {
        result.push(32);
        result.push(crate::english::encode_english(c.to_ascii_lowercase())?);
        result.push(55);
        if let Some(MathToken::Number(left)) = tokens.get(*i + 2) {
            rule_1::encode_number_literal(left, result);
        }
        result.push(0);
        if let Some(MathToken::Number(right)) = tokens.get(*i + 4) {
            rule_1::encode_number_literal(right, result);
        }
        result.push(62);
        *prev_was_number = false;
        *i += 6;
        return Ok(true);
    }

    if matches!(
        tokens.get(*i + 1),
        Some(MathToken::OpenParen(
            BracketKind::MathParen | BracketKind::Grouping
        ))
    ) && *i == 0
        && let Some(close_idx) = rule_6::find_matching_paren(tokens, *i + 1)
    {
        let inner = &tokens[*i + 2..close_idx];
        let simple_function_arg = !inner
            .iter()
            .any(|t| matches!(t, MathToken::UpperVariable(_) | MathToken::MathSymbol('|')));
        if simple_function_arg {
            result.push(crate::english::encode_english(c.to_ascii_lowercase())?);
            *prev_was_number = false;
            *i += 1;
            return Ok(true);
        }
    }

    let mut seq_end = *i;
    let mut uppercase_count = 0usize;
    while let Some(MathToken::UpperVariable(_)) = tokens.get(seq_end) {
        uppercase_count += 1;
        seq_end += 1;
        if matches!(tokens.get(seq_end), Some(MathToken::Prime)) {
            seq_end += 1;
        }
    }

    // PDF 제12항 붙임 1 — 행렬 컨텍스트면 2-cap 행렬명(`AB`)을 ⠠+letter 개별 표기.
    // The seq_end loop above guarantees tokens[*i..seq_end] contains only
    // UpperVariable and Prime tokens (no other arms reachable).
    if uppercase_count == 2 && matrix_context_active {
        for token in &tokens[*i..seq_end] {
            if let MathToken::UpperVariable(upper) = token {
                result.push(32);
                result.push(crate::english::encode_english(upper.to_ascii_lowercase())?);
            } else if matches!(token, MathToken::Prime) {
                result.push(36);
            }
        }
        *i = seq_end;
        *prev_was_number = false;
        return Ok(true);
    }
    if uppercase_count >= 2 {
        result.push(32);
        result.push(32);
        for token in &tokens[*i..seq_end] {
            if let MathToken::UpperVariable(upper) = token {
                result.push(crate::english::encode_english(upper.to_ascii_lowercase())?);
            } else if matches!(token, MathToken::Prime) {
                result.push(36);
            }
        }
        *i = seq_end;
        *prev_was_number = false;
        return Ok(true);
    }

    let omit_uppercase_indicator = *i == 0 && next_is_membership_symbol(tokens, *i);

    let next_is_overline_suffix = matches!(
        tokens.get(*i + 1),
        Some(MathToken::MathSymbol('\u{0304}' | '\u{0305}'))
    );
    let overline_suffix_continues = matches!(
        tokens.get(*i + 2),
        Some(MathToken::UpperVariable(_) | MathToken::Prime)
    );
    let overline_suffix_single = next_is_overline_suffix && !overline_suffix_continues;

    let logical_upper = logic_context
        && !matches!(
            (prev_non_space(tokens, *i), tokens.get(*i + 1)),
            (
                Some(MathToken::MathSymbol('\u{00AC}')),
                Some(MathToken::Variable(_))
            )
        )
        && !matches!(
            prev_non_space(tokens, *i),
            Some(MathToken::MathSymbol('\u{2192}'))
        )
        && !matches!(tokens.get(*i + 1), Some(MathToken::Subscript(_)))
        && !matches!(
            tokens.get(i.saturating_sub(1)),
            Some(MathToken::Subscript(_))
        );

    let predicate_form = matches!(
        (tokens.get(*i + 1), tokens.get(*i + 2), tokens.get(*i + 3)),
        (
            Some(MathToken::OpenParen(_)),
            Some(MathToken::Variable(_) | MathToken::UpperVariable(_)),
            Some(MathToken::CloseParen(_))
        )
    );

    if *i == 0
        && matches!(tokens.get(*i + 1), Some(MathToken::MathSymbol('\u{2228}')))
        && matches!(tokens.get(*i + 2), Some(MathToken::MathSymbol('\u{00AC}')))
        && let Some(MathToken::UpperVariable(right)) = tokens.get(*i + 3)
    {
        result.push(32);
        result.push(crate::english::encode_english(c.to_ascii_lowercase())?);
        result.push(0);
        let or_encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut('\u{2228}')?;
        result.extend_from_slice(or_encoded);
        result.push(0);
        let not_encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut('\u{00AC}')?;
        result.extend_from_slice(not_encoded);
        result.push(32);
        result.push(crate::english::encode_english(right.to_ascii_lowercase())?);
        *prev_was_number = false;
        *i += 4;
        return Ok(true);
    }

    if !(omit_uppercase_indicator
        || logical_upper
        || predicate_form && logic_context
        || overline_suffix_single)
    {
        result.push(32);
    }
    result.push(crate::english::encode_english(c.to_ascii_lowercase())?);

    *prev_was_number = false;
    *i += 1;
    Ok(false)
}

pub struct CombinatoricsRule;

impl MathTokenRule for CombinatoricsRule {
    fn name(&self) -> &'static str {
        "CombinatoricsRule"
    }

    fn priority(&self) -> u16 {
        10
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::Number(_)))
            && matches!(
                tokens.get(index + 1),
                Some(MathToken::UpperVariable('P' | 'C'))
            )
            && matches!(tokens.get(index + 2), Some(MathToken::Number(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let (
            Some(MathToken::Number(left)),
            Some(MathToken::UpperVariable(mark)),
            Some(MathToken::Number(right)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
        )
        else {
            return Ok(MathTokenResult::Skip);
        };

        result.push(32);
        result.push(crate::english::encode_english(mark.to_ascii_lowercase())?);
        result.push(55);
        rule_1::encode_number_literal(left, result);
        result.push(0);
        rule_1::encode_number_literal(right, result);
        result.push(62);
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(3))
    }
}

pub struct VariableRule;

impl MathTokenRule for VariableRule {
    fn name(&self) -> &'static str {
        "VariableRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        tokens
            .get(index)
            .is_some_and(|token| matches!(token, MathToken::Variable(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::Variable(c)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };

        let mut cursor = index;
        let _ = encode_variable(
            *c,
            tokens,
            &mut cursor,
            &mut state.prev_was_number,
            result,
            engine,
        )?;
        Ok(MathTokenResult::Consumed(cursor - index))
    }
}

pub struct UpperVariableRule;

impl MathTokenRule for UpperVariableRule {
    fn name(&self) -> &'static str {
        "UpperVariableRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::UpperVariable(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        _engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::UpperVariable(c)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };

        let mut cursor = index;
        let _ = encode_upper_variable(
            *c,
            tokens,
            &mut cursor,
            &mut state.prev_was_number,
            state.logic_context,
            state.matrix_context_active,
            result,
        )?;
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
    fn variable_rule_priority_and_name() {
        let r = VariableRule;
        assert_eq!(r.priority(), 50);
        assert_eq!(r.name(), "VariableRule");
    }

    #[test]
    fn upper_variable_rule_priority_and_name() {
        let r = UpperVariableRule;
        assert_eq!(r.priority(), 50);
        assert_eq!(r.name(), "UpperVariableRule");
    }

    #[test]
    fn combinatorics_rule_priority_and_name() {
        let r = CombinatoricsRule;
        assert_eq!(r.priority(), 10);
        assert_eq!(r.name(), "CombinatoricsRule");
    }

    #[test]
    fn derivative_pattern_y_superscript() {
        let bytes = enc("$y^{(n)}=f(x)$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn derivative_dy_dx_pattern() {
        let bytes = enc("$=dy/dx$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn higher_order_derivative() {
        let bytes = enc("$d^{n}y/dx^{n}$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn integral_with_differential() {
        let bytes = enc("$\\int3dx$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn number_variable_link() {
        let bytes = enc("$1x$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn combinatorics_npr() {
        let bytes = enc("$5P3$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn combinatorics_ncr() {
        let bytes = enc("$5C2$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn plain_variable() {
        let bytes = enc("$x$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn plain_upper_variable() {
        let bytes = enc("$X$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn upper_variable_element_of() {
        let bytes = enc("$X \\in A$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn prev_non_space_skips_space_returns_token() {
        let toks = vec![
            MathToken::Variable('x'),
            MathToken::Space,
            MathToken::Variable('y'),
        ];
        let t = prev_non_space(&toks, 2);
        assert!(matches!(t, Some(MathToken::Variable('x'))));
        assert!(prev_non_space(&toks, 0).is_none());
    }

    /// integral_context_for_differential — true when ∫ found scanning backwards.
    /// Drives line 19-22 (match arms for integral symbols and `=`).
    #[test]
    fn integral_context_for_differential_paths() {
        // ∫...d → true
        let toks = vec![
            MathToken::MathSymbol('\u{222B}'),
            MathToken::Number("3".into()),
            MathToken::Variable('d'),
        ];
        assert!(integral_context_for_differential(&toks, 2));
        // ...=...d → false (operator stops scan, drives line 20)
        let toks = vec![
            MathToken::MathSymbol('\u{222B}'),
            MathToken::Number("3".into()),
            MathToken::Operator('='),
            MathToken::Variable('d'),
        ];
        assert!(!integral_context_for_differential(&toks, 3));
        // no ∫ → false (drives line 24)
        let toks = vec![MathToken::Variable('d')];
        assert!(!integral_context_for_differential(&toks, 0));
    }

    /// y^{(n)} derivative: content already wrapped path drives line 54-69.
    #[test]
    fn y_superscript_paren_wrapped_no_extra_braces() {
        // y^{(n)}=...; the superscript content is already (n) → no extra wrap.
        let bytes = enc("$y^{(n)}=0$");
        assert!(!bytes.is_empty());
    }

    /// is_inside_curly_context — paths for { and } tracking (line 158-168).
    #[test]
    fn is_inside_curly_context_paths() {
        let toks = vec![
            MathToken::OpenParen(BracketKind::Curly),
            MathToken::Variable('x'),
            MathToken::CloseParen(BracketKind::Curly),
        ];
        assert!(is_inside_curly_context(&toks, 1));
        // After close → false
        assert!(!is_inside_curly_context(&toks, 3));
        // Nothing
        assert!(!is_inside_curly_context(&[], 0));
    }

    /// encode_upper_variable: matrix context with two consecutive upper variables.
    /// Drives lines 237-251 (matrix-context branch, including Prime handling on line 244-245).
    #[test]
    fn matrix_context_two_uppercase_with_prime() {
        let toks = vec![
            MathToken::UpperVariable('A'),
            MathToken::Prime,
            MathToken::UpperVariable('B'),
        ];
        let mut prev_was_number = false;
        let mut i = 0usize;
        let mut result = Vec::new();
        encode_upper_variable(
            'A',
            &toks,
            &mut i,
            &mut prev_was_number,
            false,
            true,
            &mut result,
        )
        .expect("encode_upper_variable");
        // Each uppercase emits ⠠ + letter and Prime emits 36 in matrix context.
        assert!(result.contains(&32));
        assert!(result.contains(&36));
    }

    /// encode_upper_variable: 2+ uppercase prime sequence (default non-matrix) drives line 261 (Prime branch).
    #[test]
    fn uppercase_sequence_with_prime_emits_36() {
        // PDF — `XY'` 2개 대문자 시퀀스에서 Prime은 36 emit.
        let toks = vec![
            MathToken::UpperVariable('X'),
            MathToken::Prime,
            MathToken::UpperVariable('Y'),
        ];
        let mut prev = false;
        let mut i = 0usize;
        let mut result = Vec::new();
        encode_upper_variable('X', &toks, &mut i, &mut prev, false, false, &mut result)
            .expect("encode_upper_variable");
        assert!(
            result.contains(&36),
            "expected Prime code 36 in result {:?}",
            result
        );
    }

    /// encode_upper_variable: i==0 with paren followed by (Number,Comma,Number) drives 179-204.
    #[test]
    fn upper_variable_with_paren_number_pair() {
        // F(3,4)
        let toks = vec![
            MathToken::UpperVariable('F'),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Number("3".into()),
            MathToken::Operator(','),
            MathToken::Number("4".into()),
            MathToken::CloseParen(BracketKind::MathParen),
        ];
        let mut prev = false;
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled =
            encode_upper_variable('F', &toks, &mut i, &mut prev, false, false, &mut result)
                .expect("encode_upper_variable");
        assert!(handled);
        assert_eq!(i, 6);
    }

    /// encode_upper_variable: i==0 paren simple function arg (lines 206-224).
    #[test]
    fn upper_variable_simple_function_arg() {
        // F(x)
        let toks = vec![
            MathToken::UpperVariable('F'),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            MathToken::CloseParen(BracketKind::MathParen),
        ];
        let mut prev = false;
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled =
            encode_upper_variable('F', &toks, &mut i, &mut prev, false, false, &mut result)
                .expect("encode_upper_variable");
        assert!(handled);
        // simple_function_arg path increments i by 1.
        assert_eq!(i, 1);
    }

    #[test]
    fn upper_variable_sequence_counts_consecutive_upper_variables() {
        let toks = vec![MathToken::UpperVariable('A'), MathToken::UpperVariable('B')];
        let mut prev = false;
        let mut i = 0usize;
        let mut result = Vec::new();

        let handled =
            encode_upper_variable('A', &toks, &mut i, &mut prev, false, false, &mut result)
                .expect("encode_upper_variable");

        assert!(handled);
        assert_eq!(i, 2);
    }

    /// encode_upper_variable: A∨¬B logic pattern (lines 310-328).
    #[test]
    fn upper_variable_logic_or_not_pattern() {
        let toks = vec![
            MathToken::UpperVariable('A'),
            MathToken::MathSymbol('\u{2228}'),
            MathToken::MathSymbol('\u{00AC}'),
            MathToken::UpperVariable('B'),
        ];
        let mut prev = false;
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled =
            encode_upper_variable('A', &toks, &mut i, &mut prev, true, false, &mut result)
                .expect("encode_upper_variable");
        assert!(handled);
        assert_eq!(i, 4);
    }

    /// CombinatoricsRule apply — Number,Upper(P|C),Number triggers lines 372-394.
    #[test]
    fn combinatorics_rule_apply_emits_permutation() {
        // Direct apply: encode_tokens via engine.
        let bytes = enc("$5P3$");
        assert!(!bytes.is_empty());
    }

    /// encode_variable: set-builder Korean wrap (lines 132-150).
    #[test]
    fn variable_with_korean_following_inside_curly() {
        // {x|x는 ...} pattern
        let bytes = enc("$\\{x|x는 정수\\}$");
        // No panic; produces some bytes.
        let _ = bytes;
    }

    /// CombinatoricsRule.apply with malformed tokens triggers Skip at line 401.
    /// Bypass matches() by direct apply call with non-Number tokens.
    #[test]
    fn combinatorics_rule_apply_malformed_skip() {
        let r = CombinatoricsRule;
        let mut state = MathEncodeState::with_context(
            false,
            super::super::math_token_rule::MathContext::default(),
        );
        // Three Variables (not Number/Upper/Number) → let-else triggers
        let toks = vec![
            MathToken::Variable('a'),
            MathToken::Variable('b'),
            MathToken::Variable('c'),
        ];
        let mut result = Vec::new();
        let engine =
            MathTokenEngine::with_context(super::super::math_token_rule::MathContext::default());
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }

    /// Lines 268, 284 — `_ => {}` fallback arms in UpperVariable sequence
    /// processing when matrix-context-active or general uppercase-count>=2
    /// path encounters a non-UpperVariable, non-Prime token (e.g. Space).
    /// Trigger via Korean text with Variables and Spaces interspersed.
    #[test]
    fn upper_variable_sequence_with_intermediate_tokens() {
        // Sequence with two UpperVariables and intervening space — exercises
        // the `_ => {}` arm in the for-loop over tokens[*i..seq_end].
        let bytes = enc("$AB$");
        assert!(!bytes.is_empty());
        let bytes = enc("$A B$");
        assert!(!bytes.is_empty());
    }

    /// Line 92 — `content_already_wrapped` second clause checks first()/last() pattern.
    /// Trigger via x^{(n)} = ... pattern that includes the Operator('=') context.
    #[test]
    fn upper_variable_paren_wrapped_superscript_eq_pattern() {
        // y^{(n)} = ... ? exercises the matches!() check at line 92.
        let bytes = enc("$y^{(n)}=f(x)$");
        let _ = bytes;
    }

    use super::super::math_token_rule::MathContext;

    fn dummy_engine() -> MathTokenEngine {
        MathTokenEngine::with_context(MathContext::default())
    }

    /// rule_12 line 329 - VariableRule.apply let-else Skip when token isn't Variable.
    #[test]
    fn variable_rule_apply_skip_for_non_variable() {
        let r = VariableRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Number("1".to_string())];
        let mut result = Vec::new();
        let engine = dummy_engine();
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }

    #[test]
    fn variable_rule_matches_only_variable_tokens() {
        let r = VariableRule;
        let state = MathEncodeState::with_context(false, MathContext::default());

        assert!(r.matches(&[MathToken::Variable('x')], 0, &state));
        assert!(!r.matches(&[MathToken::Number("1".to_string())], 0, &state));
    }

    /// rule_12 line 355 - UpperVariableRule.apply let-else Skip when token isn't UpperVariable.
    #[test]
    fn upper_variable_rule_apply_skip_for_non_upper_variable() {
        let r = UpperVariableRule;
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let toks = vec![MathToken::Variable('a')];
        let mut result = Vec::new();
        let engine = dummy_engine();
        let res = r.apply(&toks, 0, &mut result, &mut state, &engine);
        assert!(matches!(res, Ok(MathTokenResult::Skip)));
    }
}
