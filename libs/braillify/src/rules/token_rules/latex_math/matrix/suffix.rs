use crate::rules::math;
use crate::rules::math::math_token_rule::MathContext;
use crate::unicode::decode_unicode;

use super::encode_trimmed_math;

pub(in crate::rules::token_rules::latex_math) fn subscript_digit_to_ascii(
    ch: char,
) -> Option<char> {
    match ch {
        '₀' => Some('0'),
        '₁' => Some('1'),
        '₂' => Some('2'),
        '₃' => Some('3'),
        '₄' => Some('4'),
        '₅' => Some('5'),
        '₆' => Some('6'),
        '₇' => Some('7'),
        '₈' => Some('8'),
        '₉' => Some('9'),
        _ => None,
    }
}

pub(super) fn encode_matrix_letter_with_numeric_subscripts(
    text: &str,
    math_context: MathContext,
) -> Result<Option<Vec<u8>>, String> {
    let mut chars = text.chars();
    let Some(variable) = chars.next() else {
        return Ok(None);
    };
    if !variable.is_ascii_alphabetic() {
        return Ok(None);
    }

    let subscripts: Vec<char> = chars.collect();
    if subscripts.is_empty()
        || !subscripts
            .iter()
            .all(|ch| subscript_digit_to_ascii(*ch).is_some())
    {
        return Ok(None);
    }

    let mut out =
        math::encoder::encode_math_expression_with_context(&variable.to_string(), math_context)?;
    out.push(decode_unicode('⠰'));
    for subscript in subscripts {
        if let Some(digit) = subscript_digit_to_ascii(subscript) {
            out.extend(math::encoder::encode_math_expression_with_context(
                &digit.to_string(),
                math_context,
            )?);
        }
    }
    Ok(Some(out))
}

fn parse_latex_letter_numeric_subscript(term: &str) -> Option<(char, Vec<char>)> {
    let mut chars = term.chars();
    let variable = chars.next()?;
    if !variable.is_ascii_alphabetic() || chars.next()? != '_' || chars.next()? != '{' {
        return None;
    }

    let mut digits = Vec::new();
    for ch in chars {
        if ch == '}' {
            return Some((variable, digits));
        }
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            return None;
        }
    }
    None
}

fn encode_latex_letter_numeric_subscript(
    variable: char,
    digits: &[char],
    math_context: MathContext,
) -> Result<Vec<u8>, String> {
    let mut out =
        math::encoder::encode_math_expression_with_context(&variable.to_string(), math_context)?;
    out.push(decode_unicode('⠰'));
    for digit in digits {
        out.extend(math::encoder::encode_math_expression_with_context(
            &digit.to_string(),
            math_context,
        )?);
    }
    Ok(out)
}

pub(super) fn encode_matrix_suffix(
    suffix: &str,
    math_context: MathContext,
) -> Result<Vec<u8>, String> {
    let parts: Vec<&str> = suffix.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(Vec::new());
    }
    if !parts
        .iter()
        .any(|part| parse_latex_letter_numeric_subscript(part).is_some())
    {
        return encode_trimmed_math(suffix, math_context);
    }

    let mut out = Vec::new();
    let mut previous_was_operand = false;
    for part in parts {
        if let Some((variable, digits)) = parse_latex_letter_numeric_subscript(part) {
            if previous_was_operand {
                out.push(decode_unicode('⠐'));
            }
            out.extend(encode_latex_letter_numeric_subscript(
                variable,
                &digits,
                math_context,
            )?);
            previous_was_operand = true;
            continue;
        }

        out.extend(encode_trimmed_math(part, math_context)?);
        // PDF — 행렬 suffix 식에서 `-`는 인접한 단위(예: `a_{11}a_{22} - a_{12}a_{21}`)에
        // 공백 없이 결합된다. 점역기는 `⠔` 단독으로 emit하고 다음 피연산자가 곧 이어진다.
        previous_was_operand = false;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> MathContext {
        MathContext::default()
    }

    #[test]
    fn matrix_letter_subscripts_empty_text() {
        let result = encode_matrix_letter_with_numeric_subscripts("", ctx()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn matrix_letter_subscripts_non_alpha_first() {
        let result = encode_matrix_letter_with_numeric_subscripts("1₂", ctx()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn matrix_letter_subscripts_no_subscripts() {
        let result = encode_matrix_letter_with_numeric_subscripts("a", ctx()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn matrix_letter_subscripts_non_digit_subscript() {
        let result = encode_matrix_letter_with_numeric_subscripts("aₐ", ctx()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn matrix_letter_subscripts_valid() {
        let result = encode_matrix_letter_with_numeric_subscripts("x₁₂", ctx()).unwrap();
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn parse_letter_subscript_invalid_char() {
        assert!(parse_latex_letter_numeric_subscript("a_{1x}").is_none());
    }

    #[test]
    fn parse_letter_subscript_missing_closing_brace() {
        assert!(parse_latex_letter_numeric_subscript("a_{12").is_none());
    }

    #[test]
    fn parse_letter_subscript_non_alpha_first() {
        assert!(parse_latex_letter_numeric_subscript("1_{2}").is_none());
    }

    #[test]
    fn parse_letter_subscript_no_underscore() {
        assert!(parse_latex_letter_numeric_subscript("ab{2}").is_none());
    }

    #[test]
    fn parse_letter_subscript_valid() {
        let result = parse_latex_letter_numeric_subscript("a_{12}").unwrap();
        assert_eq!(result.0, 'a');
        assert_eq!(result.1, vec!['1', '2']);
    }

    #[test]
    fn matrix_suffix_empty() {
        let result = encode_matrix_suffix("", ctx()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn matrix_suffix_no_subscript_pattern() {
        let result = encode_matrix_suffix("+ x", ctx()).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn matrix_suffix_with_subscript_parts() {
        let result = encode_matrix_suffix("a_{11} a_{22} - a_{12} a_{21}", ctx()).unwrap();
        assert!(!result.is_empty());
    }
}
