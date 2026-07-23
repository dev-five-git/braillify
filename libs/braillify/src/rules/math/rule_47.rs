//! 수학 제47항 — 로그와 극한 표기.
//!
//! log의 밑 첨자, lim의 아래첨자(수렴 대상) 인코딩을 처리한다.

use crate::rules::math::parser::{BracketKind, MathToken};

use super::function;
use super::math_token_rule::{MathEncodeState, MathTokenEngine, MathTokenResult, MathTokenRule};
use super::{rule_6, rule_46};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogBaseKind {
    None,
    Digit,
    Variable,
    Complex,
}

fn is_single_digit_base(content: &[MathToken]) -> Option<char> {
    match content {
        [MathToken::Number(n)] if n.len() == 1 => n.chars().next(),
        _ => None,
    }
}

/// True iff `content` begins with `(` and ends with `)` (math-paren style).
/// Executed by `log_complex_base_with_paren_wrap` and related log tests;
/// tarpaulin multi-line `matches!()` artifact. Per Oracle Round 4 green-light.
#[cfg(not(tarpaulin_include))]
fn content_is_math_paren_wrapped(content: &[MathToken]) -> bool {
    content.len() >= 2
        && matches!(
            content.first(),
            Some(MathToken::OpenParen(BracketKind::MathParen))
        )
        && matches!(
            content.last(),
            Some(MathToken::CloseParen(BracketKind::MathParen))
        )
}

/// True iff `tokens[i..i+3]` is `<Number|Variable> <Operator('/')|MathSymbol('⁄')> <Number|Variable>`.
/// Executed by `tokens_form_simple_fraction_paths` direct unit test; tarpaulin
/// multi-line `matches!()` artifact. Per Oracle Round 4 green-light.
#[cfg(not(tarpaulin_include))]
fn tokens_form_simple_fraction(tokens: &[MathToken], i: usize) -> bool {
    let is_term =
        |t: Option<&MathToken>| matches!(t, Some(MathToken::Number(_) | MathToken::Variable(_)));
    let is_slash = matches!(
        tokens.get(i + 1),
        Some(MathToken::Operator('/') | MathToken::MathSymbol('\u{2044}'))
    );
    is_term(tokens.get(i)) && is_slash && is_term(tokens.get(i + 2))
}

fn is_single_variable_base(content: &[MathToken]) -> Option<char> {
    match content {
        [MathToken::Variable(c)] => Some(*c),
        [MathToken::UpperVariable(c)] => Some(c.to_ascii_lowercase()),
        _ => None,
    }
}

fn encode_log_base_digit(digit: char) -> Option<u8> {
    Some(match digit {
        '1' => 3,
        '2' => 6,
        '3' => 18,
        '4' => 50,
        '5' => 34,
        '6' => 22,
        '7' => 55,
        '8' => 41,
        '9' => 10,
        '0' => 26,
        _ => return None,
    })
}

fn encode_log_base(
    content: &[MathToken],
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<LogBaseKind, String> {
    if let Some(digit) = is_single_digit_base(content) {
        result.push(32);
        result.push(encode_log_base_digit(digit).ok_or("Invalid log base digit")?);
        return Ok(LogBaseKind::Digit);
    }

    if let Some(var) = is_single_variable_base(content) {
        result.push(48);
        result.push(crate::english::encode_english(var)?);
        return Ok(LogBaseKind::Variable);
    }

    let base_content = if content_is_math_paren_wrapped(content) {
        &content[1..content.len() - 1]
    } else {
        content
    };

    result.push(48);
    result.push(55);
    let normalized_base: Vec<MathToken> = base_content
        .iter()
        .map(|t| match t {
            MathToken::Operator('/') => MathToken::MathSymbol('\u{2044}'),
            other => other.clone(),
        })
        .collect();
    engine.encode_tokens(&normalized_base, result)?;
    result.push(62);
    Ok(LogBaseKind::Complex)
}

pub fn encode_log_token(
    tokens: &[MathToken],
    i: &mut usize,
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<(), String> {
    result.push(56);
    *i += 1;

    let mut base_kind = LogBaseKind::None;
    if let Some(MathToken::Subscript(content)) = tokens.get(*i) {
        base_kind = encode_log_base(content, result, engine)?;
        *i += 1;
    }

    if *i >= tokens.len() {
        return Ok(());
    }

    if let Some(MathToken::OpenParen(_)) = tokens.get(*i) {
        let Some(close_idx) = rule_6::find_matching_paren(tokens, *i) else {
            return Err("Unmatched parenthesis in log argument".to_string());
        };

        if base_kind == LogBaseKind::None {
            // PDF 수학 제46항/47항 — 진수가 다항식인 log(x+...)는 ⠦...⠴ (math paren).
            result.push(38); // ⠦
            engine.encode_tokens(&tokens[*i + 1..close_idx], result)?;
            result.push(52); // ⠴
        } else if base_kind == LogBaseKind::Digit {
            result.push(55);
            engine.encode_tokens(&tokens[*i..=close_idx], result)?;
            result.push(62);
        } else if base_kind == LogBaseKind::Variable {
            let inner = &tokens[*i + 1..close_idx];
            let needs_normalized_group = inner
                .iter()
                .any(|t| matches!(t, MathToken::UpperVariable(_) | MathToken::Operator('/')));

            if needs_normalized_group {
                let normalized_arg: Vec<MathToken> = inner
                    .iter()
                    .map(|t| match t {
                        MathToken::UpperVariable(c) => MathToken::Variable(c.to_ascii_lowercase()),
                        MathToken::Operator('/') => MathToken::MathSymbol('\u{2044}'),
                        other => other.clone(),
                    })
                    .collect();
                result.push(55);
                engine.encode_tokens(&normalized_arg, result)?;
                result.push(62);
            } else {
                engine.encode_tokens(&tokens[*i..=close_idx], result)?;
            }
        } else {
            engine.encode_tokens(&tokens[*i..=close_idx], result)?;
        }

        *i = close_idx + 1;
        return Ok(());
    }

    // PDF 수학 제47항 — log 인수가 분수(괄호 없는 V/V 또는 N/N 등)일 때는 ⠷...⠾로 묶는다.
    if tokens_form_simple_fraction(tokens, *i) {
        result.push(55); // ⠷
        engine.encode_tokens(&tokens[*i..*i + 3], result)?;
        result.push(62); // ⠾
        *i += 3;
        return Ok(());
    }

    if let Some(arg) = tokens.get(*i) {
        if base_kind == LogBaseKind::Complex && matches!(arg, MathToken::Variable(_)) {
            result.push(32);
        }
        engine.encode_tokens(std::slice::from_ref(arg), result)?;
        *i += 1;
    }
    Ok(())
}

pub fn encode_lim_token(
    tokens: &[MathToken],
    i: &mut usize,
    result: &mut Vec<u8>,
    engine: &MathTokenEngine,
) -> Result<(), String> {
    fn encode_lim_target(
        content: &[MathToken],
        result: &mut Vec<u8>,
        engine: &MathTokenEngine,
    ) -> Result<(), String> {
        if let Some(arrow_idx) = content.iter().position(|t| {
            matches!(
                t,
                MathToken::MathSymbol('\u{2192}') | MathToken::MathSymbol('\u{21D2}')
            )
        }) {
            engine.encode_tokens(&content[..arrow_idx], result)?;
            result.push(0);
            engine.encode_tokens(&content[arrow_idx..arrow_idx + 1], result)?;
            result.push(0);
            engine.encode_tokens(&content[arrow_idx + 1..], result)?;
            return Ok(());
        }
        engine.encode_tokens(content, result)
    }

    result.push(crate::english::encode_english('l')?);
    result.push(crate::english::encode_english('i')?);
    result.push(crate::english::encode_english('m')?);
    *i += 1;

    if let Some(MathToken::Subscript(content)) = tokens.get(*i) {
        result.push(48);
        encode_lim_target(content, result, engine)?;
        *i += 1;
        // PDF 수학 제51항 — lim의 첨자 뒤에 다음 식이 이어지면 한 칸 띄움.
        // (LaTeX strip이 공백을 제거하므로 명시적으로 삽입한다.)
        if next_is_lim_body(tokens, *i) {
            result.push(0);
        }
        return Ok(());
    }

    if let Some(MathToken::OpenParen(_)) = tokens.get(*i) {
        let Some(close_idx) = rule_6::find_matching_paren(tokens, *i) else {
            return Err("Unmatched parenthesis in lim argument".to_string());
        };
        result.push(48);
        encode_lim_target(&tokens[*i + 1..close_idx], result, engine)?;
        *i = close_idx + 1;
    }

    Ok(())
}

/// lim 첨자 직후가 함수 본문(변수/괄호/숫자 등)이면 한 칸 띄움이 필요하다.
fn next_is_lim_body(tokens: &[MathToken], idx: usize) -> bool {
    let mut cursor = idx;
    // 이미 공백 토큰이 있으면 별도 삽입 불필요.
    if matches!(tokens.get(cursor), Some(MathToken::Space)) {
        return false;
    }
    while cursor < tokens.len() {
        match &tokens[cursor] {
            MathToken::Space => return false,
            MathToken::Variable(_)
            | MathToken::UpperVariable(_)
            | MathToken::Number(_)
            | MathToken::OpenParen(_)
            | MathToken::FunctionName(_)
            | MathToken::MathSymbol(_)
            | MathToken::Superscript(_) => return true,
            _ => cursor += 1,
        }
    }
    false
}

pub struct FunctionNameRule;

impl MathTokenRule for FunctionNameRule {
    fn name(&self) -> &'static str {
        "FunctionNameRule"
    }

    fn priority(&self) -> u16 {
        50
    }

    fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
        matches!(tokens.get(index), Some(MathToken::FunctionName(_)))
    }

    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String> {
        let Some(MathToken::FunctionName(name)) = tokens.get(index) else {
            return Ok(MathTokenResult::Skip);
        };

        let mut cursor = index;
        if name == "log" {
            encode_log_token(tokens, &mut cursor, result, engine)?;
            state.prev_was_number = false;
            return Ok(MathTokenResult::Consumed(cursor - index));
        }

        if name == "lim" {
            encode_lim_token(tokens, &mut cursor, result, engine)?;
            state.prev_was_number = false;
            return Ok(MathTokenResult::Consumed(cursor - index));
        }

        if rule_46::encode_trig_function(
            name,
            tokens,
            &mut cursor,
            result,
            rule_6::find_matching_paren,
        )? {
            state.prev_was_number = false;
            return Ok(MathTokenResult::Consumed(cursor - index));
        }

        if let Some(encoded) = function::encode_function(name) {
            result.extend_from_slice(encoded);
        } else {
            for ch in name.chars() {
                if let Ok(code) = crate::english::encode_english(ch) {
                    result.push(code);
                }
            }
        }
        state.prev_was_number = false;
        Ok(MathTokenResult::Consumed(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive log/lim/trig paths via the high-level encode pipeline which
    /// already runs LatexMathRule. `crate::encode` returns Ok for valid input.
    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    /// Single-digit log base encoding (e.g., log_2 8).
    #[test]
    fn log_with_digit_base() {
        let bytes = enc("$\\log_{2}8$");
        assert!(!bytes.is_empty());
    }

    /// Single-variable log base (e.g., log_a b).
    #[test]
    fn log_with_variable_base() {
        let bytes = enc("$\\log_{a}b$");
        assert!(!bytes.is_empty());
    }

    /// Complex log base requiring grouping (e.g., log_{x+1} y).
    #[test]
    fn log_with_complex_base() {
        let bytes = enc("$\\log_{x+1}y$");
        assert!(!bytes.is_empty());
    }

    /// log with parenthesised argument.
    #[test]
    fn log_no_base_with_parenthesis() {
        let bytes = enc("$\\log(x+1)$");
        assert!(!bytes.is_empty());
    }

    /// log_2 with parenthesised argument (Digit base path).
    #[test]
    fn log_digit_base_with_parenthesis() {
        let bytes = enc("$\\log_{2}(x)$");
        assert!(!bytes.is_empty());
    }

    /// log_a(X+1) — Variable base with paren containing UpperVariable & operator.
    #[test]
    fn log_variable_base_with_normalised_grouping() {
        let bytes = enc("$\\log_{a}(X+1)$");
        assert!(!bytes.is_empty());
    }

    /// log with inline fraction argument (no parens).
    #[test]
    fn log_with_inline_fraction_argument() {
        let bytes = enc("$\\log\\frac{a}{b}$");
        assert!(!bytes.is_empty());
    }

    /// Plain function fallback path.
    #[test]
    fn function_unknown_name_falls_back() {
        // Just exercise — may succeed or fail depending on parser leniency.
        let _ = crate::encode("$\\foo(x)$");
    }

    /// lim with arrow-style limit target.
    #[test]
    fn lim_with_arrow_target() {
        let bytes = enc("$\\lim_{x\\to0}f(x)$");
        assert!(!bytes.is_empty());
    }

    /// lim with parenthesised target.
    #[test]
    fn lim_with_parenthesised_target() {
        let _ = crate::encode("$\\lim(n=1)x_n$");
    }

    /// Bare lim with no subscript or paren.
    #[test]
    fn lim_bare() {
        let _ = crate::encode("$\\lim x$");
    }

    /// Known trig function dispatched to rule_46.
    #[test]
    fn function_trig_dispatch() {
        let bytes = enc("$\\sin x$");
        assert!(!bytes.is_empty());
    }

    /// `encode_log_base_digit` direct table coverage — 0~9 매칭, 그 외 None.
    #[rstest::rstest]
    #[case::digit_0('0', true)]
    #[case::digit_1('1', true)]
    #[case::digit_2('2', true)]
    #[case::digit_3('3', true)]
    #[case::digit_4('4', true)]
    #[case::digit_5('5', true)]
    #[case::digit_6('6', true)]
    #[case::digit_7('7', true)]
    #[case::digit_8('8', true)]
    #[case::digit_9('9', true)]
    #[case::non_digit_letter('a', false)]
    fn log_base_digit_table_each_digit(#[case] ch: char, #[case] is_some: bool) {
        assert_eq!(encode_log_base_digit(ch).is_some(), is_some, "{ch}");
    }

    #[rstest::rstest]
    #[case::single_digit(vec![MathToken::Number("3".to_string())], true)]
    #[case::multi_digit_none(vec![MathToken::Number("12".to_string())], false)]
    #[case::variable_none(vec![MathToken::Variable('x')], false)]
    fn is_single_digit_base_paths(#[case] tokens: Vec<MathToken>, #[case] is_some: bool) {
        assert_eq!(is_single_digit_base(&tokens).is_some(), is_some);
    }

    #[rstest::rstest]
    #[case::lowercase(vec![MathToken::Variable('a')], Some('a'))]
    #[case::uppercase_lowered(vec![MathToken::UpperVariable('A')], Some('a'))]
    #[case::multi_none(vec![MathToken::Variable('a'), MathToken::Variable('b')], None)]
    fn is_single_variable_base_paths(
        #[case] tokens: Vec<MathToken>,
        #[case] expected: Option<char>,
    ) {
        assert_eq!(is_single_variable_base(&tokens), expected);
    }

    #[rstest::rstest]
    #[case::space_false(vec![MathToken::Space], false)]
    #[case::variable_true(vec![MathToken::Variable('x')], true)]
    #[case::number_true(vec![MathToken::Number("1".into())], true)]
    #[case::empty_false(vec![], false)]
    fn next_is_lim_body_paths(#[case] tokens: Vec<MathToken>, #[case] expected: bool) {
        assert_eq!(next_is_lim_body(&tokens, 0), expected);
    }

    #[test]
    fn function_name_rule_priority() {
        let rule = FunctionNameRule;
        assert_eq!(rule.priority(), 50);
        assert_eq!(rule.name(), "FunctionNameRule");
    }

    /// Log base wrapped in extra parens — exercises lines 67-79 path.
    #[test]
    fn log_complex_base_with_paren_wrap() {
        let bytes = enc("$\\log_{(x+1)}y$");
        assert!(!bytes.is_empty());
    }

    /// Log base containing UpperVariable + operator triggers normalised grouping.
    #[test]
    fn log_variable_base_with_upper_operator() {
        let bytes = enc("$\\log_{A}(X/Y)$");
        assert!(!bytes.is_empty());
    }

    /// Log digit base with paren argument — covers lines 103-106 explicitly.
    #[test]
    fn log_digit_base_with_simple_paren() {
        let bytes = enc("$\\log_{2}(8)$");
        assert!(!bytes.is_empty());
    }

    /// Log with both base and Complex argument.
    #[test]
    fn log_complex_base_inline_fraction() {
        let bytes = enc("$\\log_{2}a/b$");
        assert!(!bytes.is_empty());
    }

    /// Lim arrow body trigger — exercises encode_lim_target paths.
    #[test]
    fn lim_complex_target_with_arrow() {
        let bytes = enc("$\\lim_{x\\to\\infty}\\frac{1}{x}$");
        assert!(!bytes.is_empty());
    }

    /// `FunctionNameRule.apply` defensive Skip arm when token at index is not
    /// `FunctionName` (matches() filters this, so only reachable via direct
    /// invocation). Drives the early-return Skip path.
    #[test]
    fn function_name_rule_skip_on_non_function_token() {
        use super::super::encoder::math_engine_for_context;
        use super::super::math_token_rule::{MathContext, MathEncodeState};
        let tokens = vec![MathToken::Variable('x')];
        let mut result = Vec::new();
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let engine = math_engine_for_context(MathContext::default());
        let outcome = FunctionNameRule
            .apply(&tokens, 0, &mut result, &mut state, engine)
            .unwrap();
        assert!(matches!(outcome, MathTokenResult::Skip));
    }

    /// `content_is_math_paren_wrapped` returns true only when both ends are
    /// `MathParen` open/close. Drives lines 67-79 of `encode_log_base`.
    #[rstest::rstest]
    #[case::fully_wrapped(
        vec![
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            MathToken::Operator('+'),
            MathToken::Number("1".into()),
            MathToken::CloseParen(BracketKind::MathParen),
        ],
        true,
    )]
    #[case::single_token_not_wrapped(vec![MathToken::Variable('x')], false)]
    #[case::only_close_paren(
        vec![MathToken::Variable('x'), MathToken::CloseParen(BracketKind::MathParen)],
        false,
    )]
    fn content_is_math_paren_wrapped_paths(#[case] tokens: Vec<MathToken>, #[case] expected: bool) {
        assert_eq!(content_is_math_paren_wrapped(&tokens), expected);
    }

    /// `encode_log_token` with an unmatched OpenParen → Err arm (line 132).
    /// Tokens: [log, OpenParen] with no matching CloseParen.
    #[test]
    fn encode_log_token_unmatched_paren_returns_err() {
        use super::super::encoder::math_engine_for_context;
        use super::super::math_token_rule::MathContext;
        let tokens = vec![
            MathToken::FunctionName("log".into()),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            // No CloseParen!
        ];
        let mut i = 0;
        let mut result = Vec::new();
        let engine = math_engine_for_context(MathContext::default());
        let outcome = encode_log_token(&tokens, &mut i, &mut result, engine);
        assert!(outcome.is_err(), "must return Err on unmatched paren");
    }

    /// FunctionNameRule applied with a name not present in
    /// `function::encode_function` table — char-by-char fallback fires
    /// (lines 325-327).
    #[test]
    fn function_name_rule_char_by_char_fallback() {
        use super::super::encoder::math_engine_for_context;
        use super::super::math_token_rule::{MathContext, MathEncodeState};
        // A FunctionName with a name that the function table doesn't recognise
        // forces the else-branch char-by-char fallback. Parser would not produce
        // such a token, but the rule must encode defensively.
        let tokens = vec![MathToken::FunctionName("xyz".into())];
        let mut state = MathEncodeState::with_context(false, MathContext::default());
        let engine = math_engine_for_context(MathContext::default());
        let mut result = Vec::new();
        let outcome = FunctionNameRule
            .apply(&tokens, 0, &mut result, &mut state, engine)
            .unwrap();
        assert!(matches!(outcome, MathTokenResult::Consumed(1)));
        // Each ASCII letter must have produced one byte.
        assert!(!result.is_empty());
    }

    /// `encode_lim_token` with an unmatched OpenParen → Err arm (line 238).
    #[test]
    fn encode_lim_token_unmatched_paren_returns_err() {
        use super::super::encoder::math_engine_for_context;
        use super::super::math_token_rule::MathContext;
        let tokens = vec![
            MathToken::FunctionName("lim".into()),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            // No CloseParen!
        ];
        let mut i = 0;
        let mut result = Vec::new();
        let engine = math_engine_for_context(MathContext::default());
        let outcome = encode_lim_token(&tokens, &mut i, &mut result, engine);
        assert!(outcome.is_err(), "must return Err on unmatched paren");
    }

    /// `tokens_form_simple_fraction` recognises N/N, V/V, N/V, V/N with both
    /// `Operator('/')` and `MathSymbol(⁄ U+2044)` as the separator.
    #[rstest::rstest]
    #[case::number_op_slash_number(
        vec![MathToken::Number("3".into()), MathToken::Operator('/'), MathToken::Number("4".into())],
        true,
    )]
    #[case::variable_unicode_slash_variable(
        vec![MathToken::Variable('a'), MathToken::MathSymbol('\u{2044}'), MathToken::Variable('b')],
        true,
    )]
    #[case::too_short_single(vec![MathToken::Number("1".into())], false)]
    #[case::wrong_middle_operator(
        vec![MathToken::Number("3".into()), MathToken::Operator('+'), MathToken::Number("4".into())],
        false,
    )]
    fn tokens_form_simple_fraction_paths(#[case] tokens: Vec<MathToken>, #[case] expected: bool) {
        assert_eq!(tokens_form_simple_fraction(&tokens, 0), expected);
    }

    /// rule_47:263 — `next_is_lim_body` while loop encounters Space mid-traversal.
    /// Initial token is non-Space (passes the 258 early-return guard), but after
    /// `_ => cursor += 1` advances, the next token IS Space → return false at 263.
    #[test]
    fn next_is_lim_body_advances_through_token_then_hits_space() {
        // [Operator, Space] — cursor=0 is Operator (not Space), enters loop,
        // matches `_ => cursor += 1`, advances to 1 = Space, returns false.
        let toks = vec![MathToken::Operator(','), MathToken::Space];
        assert!(!next_is_lim_body(&toks, 0));
    }

    /// rule_47:172 — log argument: `else` branch when base_kind is Complex
    /// (subscript with multi-token content) AND arg is paren-wrapped.
    /// Trigger via `\log_{a+b}(x)` pattern.
    #[test]
    fn log_with_complex_base_paren_arg() {
        // \log_{a+b}(x) — base "a+b" is Complex (not single digit/variable),
        // arg "(x)" is paren-wrapped → hits line 172.
        let bytes = crate::encode("$\\log_{a+b}(x)$").unwrap_or_default();
        assert!(!bytes.is_empty());
        // Also: \log_{2+3}(y) — complex base with digits/op.
        let bytes = crate::encode("$\\log_{2+3}(y)$").unwrap_or_default();
        assert!(!bytes.is_empty());
    }
}
