//! Superscript/subscript sequence helpers and fraction grouping logic
//! (extracted from latex_math.rs).

pub(super) fn to_superscript_sequence(input: &str) -> String {
    let mut out = String::new();
    for ec in input.chars() {
        match ec {
            '0' => out.push('\u{2070}'),
            '1' => out.push('\u{00B9}'),
            '2' => out.push('\u{00B2}'),
            '3' => out.push('\u{00B3}'),
            '4' => out.push('\u{2074}'),
            '5' => out.push('\u{2075}'),
            '6' => out.push('\u{2076}'),
            '7' => out.push('\u{2077}'),
            '8' => out.push('\u{2078}'),
            '9' => out.push('\u{2079}'),
            '+' => out.push('\u{207A}'),
            '-' => out.push('\u{207B}'),
            'n' => out.push('\u{207F}'),
            'k' => out.push('\u{1D4F}'),
            'm' => out.push('\u{1D50}'),
            'x' => out.push('\u{02E3}'),
            '(' => out.push('\u{207D}'),
            ')' => out.push('\u{207E}'),
            '/' => out.push('\u{2044}'),
            '.' => out.push('\u{00B7}'),
            _ => out.push(ec),
        }
    }
    out
}

pub(super) fn to_subscript_sequence(input: &str) -> Option<String> {
    let mut out = String::new();
    for ch in input.chars() {
        let mapped = match ch {
            '0' => '\u{2080}',
            '1' => '\u{2081}',
            '2' => '\u{2082}',
            '3' => '\u{2083}',
            '4' => '\u{2084}',
            '5' => '\u{2085}',
            '6' => '\u{2086}',
            '7' => '\u{2087}',
            '8' => '\u{2088}',
            '9' => '\u{2089}',
            'a' => '\u{2090}',
            'e' => '\u{2091}',
            'o' => '\u{2092}',
            'x' => '\u{2093}',
            'h' => '\u{2095}',
            'k' => '\u{2096}',
            'l' => '\u{2097}',
            'm' => '\u{2098}',
            'n' => '\u{2099}',
            'p' => '\u{209A}',
            's' => '\u{209B}',
            't' => '\u{209C}',
            'i' => '\u{1D62}',
            'r' => '\u{1D63}',
            'u' => '\u{1D64}',
            'v' => '\u{1D65}',
            '+' => '\u{208A}',
            '-' => '\u{208B}',
            '(' => '\u{208D}',
            ')' => '\u{208E}',
            _ => return None,
        };
        out.push(mapped);
    }
    Some(out)
}

/// PDF 수학 제7항 3: 분수의 분자/분모가 묶음 괄호를 필요로 하는지 판별한다.
pub(super) fn needs_grouping_in_fraction(expr: &str) -> bool {
    let chars: Vec<char> = expr.chars().collect();
    if chars.is_empty() {
        return false;
    }
    if chars.first() == Some(&'(') && chars.last() == Some(&')') {
        // 외곽이 단일 괄호 쌍이면 wrap 불필요. 단, `(...)(...)` 같이 인접한 다중 괄호
        // 그룹이면 외곽이 단일 쌍이 아니므로 wrap 필요.
        // 단일 쌍 판정: 처음 `(`에서 시작한 depth가 마지막 `)`에서만 0으로 돌아옴.
        let mut depth = 0i32;
        let mut returned_to_zero_before_end = false;
        for (idx, &c) in chars.iter().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && idx < chars.len() - 1 {
                        returned_to_zero_before_end = true;
                    }
                }
                _ => {}
            }
        }
        if !returned_to_zero_before_end {
            return false;
        }
    }
    let mut depth = 0usize;
    let mut paren_groups = 0usize;
    for &c in &chars {
        match c {
            '(' | '[' | '{' => {
                if depth == 0 {
                    paren_groups += 1;
                }
                depth += 1;
            }
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            // PDF 제7항 3 — 분자/분모가 산술 연산자(+, -, ×, ÷)를 포함하면 그룹 묶음 필요.
            '+' | '-' | '\u{00D7}' | '\u{00F7}' | '\u{2212}' if depth == 0 => return true,
            // PDF — 편미분 `∂^2 z` 같이 복수 토큰의 분수 본문은 그룹 처리한다.
            ' ' | '\u{2202}' if depth == 0 => return true,
            _ => {}
        }
    }
    // PDF — `(x+1)(x+2)(x+3)` 같이 인접한 다중 paren 그룹은 wrap 필요.
    if paren_groups >= 2 {
        return true;
    }
    if chars.first() == Some(&'d') && chars.len() >= 2 {
        let rest = &chars[1..];
        let is_differential = rest.iter().all(|&c| {
            c.is_ascii_alphabetic()
                || c == '^'
                || c == '_'
                || ('\u{00B2}'..='\u{00B3}').contains(&c)
                || c == '\u{00B9}'
                || ('\u{2070}'..='\u{2079}').contains(&c)
                || ('\u{2080}'..='\u{2089}').contains(&c)
        });
        if is_differential {
            return false;
        }
    }
    let base_chars: Vec<char> = chars
        .iter()
        .copied()
        .filter(|&c| {
            !c.is_ascii_digit()
                && !c.is_ascii_alphabetic()
                && c != '^'
                && c != '_'
                && !('\u{00B9}'..='\u{00B3}').contains(&c)
                && !('\u{2070}'..='\u{2079}').contains(&c)
                && !('\u{2080}'..='\u{2089}').contains(&c)
        })
        .collect();
    if base_chars.is_empty() {
        let alpha_count = chars.iter().filter(|&&c| c.is_ascii_alphabetic()).count();
        let digit_count = chars.iter().filter(|&&c| c.is_ascii_digit()).count();
        if alpha_count == 1 && digit_count == 0 {
            return false;
        }
        if alpha_count == 0 {
            return false;
        }
        if alpha_count >= 2 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `to_superscript_sequence` must map every documented char and pass others
    /// through unchanged. Exercises every `match` arm including the `/` case.
    #[test]
    fn superscript_table_full_coverage() {
        let cases = [
            ("0", "\u{2070}"),
            ("1", "\u{00B9}"),
            ("2", "\u{00B2}"),
            ("3", "\u{00B3}"),
            ("4", "\u{2074}"),
            ("5", "\u{2075}"),
            ("6", "\u{2076}"),
            ("7", "\u{2077}"),
            ("8", "\u{2078}"),
            ("9", "\u{2079}"),
            ("+", "\u{207A}"),
            ("-", "\u{207B}"),
            ("n", "\u{207F}"),
            ("k", "\u{1D4F}"),
            ("m", "\u{1D50}"),
            ("x", "\u{02E3}"),
            ("(", "\u{207D}"),
            (")", "\u{207E}"),
            ("/", "\u{2044}"),
            (".", "\u{00B7}"),
            ("z", "z"), // fall-through
        ];
        for (input, expected) in cases {
            assert_eq!(to_superscript_sequence(input), expected, "input={input}");
        }
    }

    /// `to_subscript_sequence` returns `Some` for every documented char and
    /// `None` for anything else.
    #[test]
    fn subscript_table_full_coverage() {
        let mapped = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'e', 'o', 'x', 'h', 'k', 'l',
            'm', 'n', 'p', 's', 't', 'i', 'r', 'u', 'v', '+', '-', '(', ')',
        ];
        for c in mapped {
            assert!(
                to_subscript_sequence(&c.to_string()).is_some(),
                "char {c:?}"
            );
        }
        // Multi-char input with all mapped chars
        assert!(to_subscript_sequence("aeo").is_some());
        // Unmapped chars short-circuit to None
        assert!(to_subscript_sequence("z").is_none());
        assert!(to_subscript_sequence("a1z").is_none());
    }

    /// `needs_grouping_in_fraction` decision matrix.
    #[test]
    fn fraction_grouping_decision_matrix() {
        // Empty body → false
        assert!(!needs_grouping_in_fraction(""));
        // Single outer paren pair → false (single-pair check at line 81-101)
        assert!(!needs_grouping_in_fraction("(x+1)"));
        // Adjacent paren pairs → true (depth returns to 0 before end)
        assert!(needs_grouping_in_fraction("(a)(b)"));
        // Arithmetic operator → true
        assert!(needs_grouping_in_fraction("a+b"));
        assert!(needs_grouping_in_fraction("a-b"));
        assert!(needs_grouping_in_fraction("a\u{00D7}b"));
        assert!(needs_grouping_in_fraction("a\u{00F7}b"));
        assert!(needs_grouping_in_fraction("a\u{2212}b"));
        // Space at top level → true
        assert!(needs_grouping_in_fraction("a b"));
        // Partial-derivative `∂` → true (multi-token form)
        assert!(needs_grouping_in_fraction("\u{2202}f"));
        // Differential `dx` etc. → false
        assert!(!needs_grouping_in_fraction("dx"));
        assert!(!needs_grouping_in_fraction("dxy"));
        // 2+ adjacent paren groups → true
        assert!(needs_grouping_in_fraction("(x)(y)(z)"));
        // Single alpha char only → false (single letter denominator)
        assert!(!needs_grouping_in_fraction("a"));
        // Pure digits → false (no alpha, no operator)
        assert!(!needs_grouping_in_fraction("123"));
        // Multiple alpha chars (non-differential prefix) → true
        // (e.g., variable product like "ab" treated as multi-token)
        assert!(needs_grouping_in_fraction("ab"));
    }
}
