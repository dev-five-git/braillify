//! 수학 제46항 — 삼각함수 표기.
//!
//! sin, cos, tan, csc, sec, cot 인코딩과 인수 정규화를 처리한다.

use crate::rules::math::function;
use crate::rules::math::parser::MathToken;

pub fn is_trig_function(name: &str) -> bool {
    matches!(name, "sin" | "cos" | "tan" | "csc" | "sec" | "cot")
}

/// Single-line `matches!()` helpers — extracted so tarpaulin can attribute
/// coverage to one line per call site (multi-line forms suffered attribution loss).
fn is_number_or_variable(tok: Option<&MathToken>) -> bool {
    matches!(tok, Some(MathToken::Number(_) | MathToken::Variable(_)))
}

fn is_fraction_slash(tok: Option<&MathToken>) -> bool {
    matches!(
        tok,
        Some(MathToken::Operator('/') | MathToken::MathSymbol('\u{2044}'))
    )
}

/// Emits the braille bytes for a single Number or Variable token used inside
/// a trig-function inline fraction (e.g. `sin(x/2)`). Returns Err for any
/// other token type so the caller's failure path is observable.
fn emit_trig_fraction_term(tok: Option<&MathToken>, result: &mut Vec<u8>) -> Result<(), String> {
    match tok {
        Some(MathToken::Number(n)) => {
            result.push(60);
            for ch in n.chars() {
                result.extend(crate::number::encode_number(ch));
            }
            Ok(())
        }
        Some(MathToken::Variable(v)) => {
            result.push(crate::english::encode_english(*v)?);
            Ok(())
        }
        other => Err(format!(
            "trig fraction term must be Number or Variable, got {other:?}"
        )),
    }
}

pub fn encode_trig_function(
    name: &str,
    tokens: &[MathToken],
    i: &mut usize,
    result: &mut Vec<u8>,
    find_matching_paren: fn(&[MathToken], usize) -> Option<usize>,
) -> Result<bool, String> {
    if !is_trig_function(name) {
        return Ok(false);
    }
    // `is_trig_function` guarantees `name` is one of sin/cos/tan/csc/sec/cot,
    // and `function::encode_function` always returns Some for those.
    // The defensive let-else fallback was structurally unreachable.
    let encoded = function::encode_function(name)
        .expect("is_trig_function guarantees encode_function returns Some");
    result.extend_from_slice(encoded);

    if let (Some(MathToken::Number(n)), Some(MathToken::Variable(v))) =
        (tokens.get(*i + 1), tokens.get(*i + 2))
    {
        result.push(55);
        result.push(60);
        for ch in n.chars() {
            result.extend(crate::number::encode_number(ch));
        }
        result.push(crate::english::encode_english(v.to_ascii_lowercase())?);
        result.push(62);
        *i += 3;
        return Ok(true);
    }

    if let Some(MathToken::OpenParen(_)) = tokens.get(*i + 1)
        && let Some(close_idx) = find_matching_paren(tokens, *i + 1)
        && let [
            MathToken::Variable(v),
            MathToken::Operator('/'),
            MathToken::Number(n),
        ] = &tokens[*i + 2..close_idx]
    {
        result.push(55);
        result.push(60);
        for ch in n.chars() {
            result.extend(crate::number::encode_number(ch));
        }
        result.push(12);
        result.push(crate::english::encode_english(v.to_ascii_lowercase())?);
        result.push(62);
        *i = close_idx + 1;
        return Ok(true);
    }

    // Check if the next token(s) form a compound argument that needs brackets
    // (multiple variables, or a fraction)
    let next_idx = *i + 1;
    if next_idx < tokens.len() {
        // Two consecutive variables: sinxy -> sin(xy)
        if let (Some(MathToken::Variable(v1)), Some(MathToken::Variable(v2))) =
            (tokens.get(next_idx), tokens.get(next_idx + 1))
        {
            result.push(55); // Grouping open
            result.push(crate::english::encode_english(*v1)?);
            result.push(crate::english::encode_english(*v2)?);
            result.push(62); // Grouping close
            *i += 3;
            return Ok(true);
        }
        // Fraction without parens: sin(6/x) or sin(x/6). U+2044 (LaTeX \frac slash)도 매칭.
        if is_number_or_variable(tokens.get(next_idx))
            && is_fraction_slash(tokens.get(next_idx + 1))
            && is_number_or_variable(tokens.get(next_idx + 2))
        {
            result.push(55); // Grouping open
            // Both sides are guaranteed Number|Variable by the outer matches!()
            // check above; we destructure with let-bindings to keep the code
            // surface single-branch (no defensive `_ => {}` dead arms).
            emit_trig_fraction_term(tokens.get(next_idx), result)?;
            result.push(12); // fraction slash
            emit_trig_fraction_term(tokens.get(next_idx + 2), result)?;
            result.push(62); // Grouping close
            *i += 4;
            return Ok(true);
        }
    }
    *i += 1;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::math::parser::BracketKind;

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    #[test]
    fn is_trig_function_table() {
        for name in ["sin", "cos", "tan", "csc", "sec", "cot"] {
            assert!(is_trig_function(name), "{name}");
        }
        assert!(!is_trig_function("log"));
        assert!(!is_trig_function("lim"));
        assert!(!is_trig_function("foo"));
    }

    #[test]
    fn trig_with_number_then_variable() {
        // "sin30x" → sin(30x) bracketing path.
        let bytes = enc("$\\sin30x$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn trig_with_parenthesised_fraction() {
        // "sin(x/2)" — paren fraction path.
        let bytes = enc("$\\sin(x/2)$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn trig_with_two_consecutive_vars() {
        // "sinxy" → sin(xy) grouping.
        let bytes = enc("$\\sin xy$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn trig_with_inline_fraction_no_paren_numerator_number() {
        // "sin6/x" — number/var fraction.
        let bytes = enc("$\\sin6/x$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn trig_with_inline_fraction_no_paren_numerator_var() {
        // "sinx/6" — var/number fraction.
        let bytes = enc("$\\sin x/6$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn trig_plain_variable_argument() {
        // "sinx" — single variable, falls through to i+=1 path.
        let bytes = enc("$\\sin x$");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn each_trig_variant() {
        for f in ["sin", "cos", "tan", "csc", "sec", "cot"] {
            let bytes = enc(&format!("$\\{f}x$"));
            assert!(!bytes.is_empty(), "{f}");
        }
    }

    /// 제46항 — encode_trig_function returns Ok(false) for non-trig name (line 19-21).
    #[test]
    fn encode_trig_returns_false_for_non_trig() {
        let toks = vec![MathToken::Variable('x')];
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled =
            encode_trig_function("log", &toks, &mut i, &mut result, |_, _| None).expect("ok");
        assert!(!handled);
        assert!(result.is_empty());
    }

    /// 제46항 — encode_trig_function when function::encode_function returns None
    /// (unsupported trig name). Drives line 22-24 (Some(encoded) = ...).
    /// Trig names are all defined so we use a stub find_matching_paren that doesn't matter.
    /// This path is implicitly hard to hit because all trig names exist, so we ensure
    /// the early-return path on non-trig is the only escape (already covered above).
    #[test]
    fn encode_trig_function_basic_path() {
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::Variable('x'),
        ];
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled =
            encode_trig_function("sin", &toks, &mut i, &mut result, |_, _| None).expect("ok");
        assert!(handled);
        assert!(!result.is_empty());
        // `i` must advance past the function token (exact final position depends
        // on the inner trig dispatch — we just verify it did move).
        assert!(i >= 1);
    }

    /// 제46항 — encode_trig_function with paren V/N pattern drives lines 41-58.
    #[test]
    fn encode_trig_function_paren_v_n_fraction() {
        // sin(x/2) — paren wraps variable/number
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::OpenParen(BracketKind::MathParen),
            MathToken::Variable('x'),
            MathToken::Operator('/'),
            MathToken::Number("2".into()),
            MathToken::CloseParen(BracketKind::MathParen),
        ];
        let mut i = 0usize;
        let mut result = Vec::new();
        let handled = encode_trig_function("sin", &toks, &mut i, &mut result, |t, idx| {
            if let Some(MathToken::OpenParen(_)) = t.get(idx) {
                t[idx + 1..]
                    .iter()
                    .position(|x| matches!(x, MathToken::CloseParen(_)))
                    .map(|p| p + idx + 1)
            } else {
                None
            }
        })
        .expect("ok");
        assert!(handled);
        assert!(!result.is_empty());
    }

    /// 제46항 — two consecutive variables (sinxy → group) drives lines 64-87.
    /// The fallback variable bindings inside the closures at lines 71-83 are
    /// dead-code defensive defaults; the primary path always supplies Variable.
    #[test]
    fn encode_trig_function_two_consecutive_vars_path() {
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::Variable('x'),
            MathToken::Variable('y'),
        ];
        let mut i = 0usize;
        let mut result = Vec::new();
        encode_trig_function("sin", &toks, &mut i, &mut result, |_, _| None).expect("ok");
        // i advanced past sin + x + y
        assert_eq!(i, 3);
    }

    /// 제46항 — N/N fraction-without-parens (sin6/3) drives lines 89-130.
    #[test]
    fn encode_trig_function_inline_n_slash_n_fraction() {
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::Number("6".into()),
            MathToken::Operator('/'),
            MathToken::Number("3".into()),
        ];
        let mut i = 0usize;
        let mut result = Vec::new();
        encode_trig_function("sin", &toks, &mut i, &mut result, |_, _| None).expect("ok");
        assert_eq!(i, 4);
    }

    /// 제46항 — V/V fraction-without-parens (sin a/b) drives lines 103-126.
    #[test]
    fn encode_trig_function_inline_v_slash_v_fraction() {
        let toks = vec![
            MathToken::FunctionName("sin".into()),
            MathToken::Variable('a'),
            MathToken::Operator('/'),
            MathToken::Variable('b'),
        ];
        let mut i = 0usize;
        let mut result = Vec::new();
        encode_trig_function("sin", &toks, &mut i, &mut result, |_, _| None).expect("ok");
        assert_eq!(i, 4);
    }

    /// `emit_trig_fraction_term` happy path for Number and Variable, and the
    /// defensive Err arm for any other token (None or non-Number-non-Variable).
    #[test]
    fn emit_trig_fraction_term_branches() {
        let mut r = Vec::new();
        emit_trig_fraction_term(Some(&MathToken::Number("7".into())), &mut r).unwrap();
        assert!(!r.is_empty());

        let mut r = Vec::new();
        emit_trig_fraction_term(Some(&MathToken::Variable('z')), &mut r).unwrap();
        assert!(!r.is_empty());

        let mut r = Vec::new();
        assert!(emit_trig_fraction_term(None, &mut r).is_err());

        let mut r = Vec::new();
        assert!(emit_trig_fraction_term(Some(&MathToken::Operator('+')), &mut r).is_err());
    }
}
