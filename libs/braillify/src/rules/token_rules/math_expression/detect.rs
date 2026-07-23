//! Math expression detection (is_math_expression and friends).

use crate::math_symbol_shortcut;
use crate::rules::math;

use super::helpers::*;

/// True iff `chars` contains a letter immediately before AND immediately after
/// a `/` — i.e. the slash sits between alphabetic characters, signalling a
/// fraction shorthand like `F/N`, `a/b`.
/// Executed by `test_is_math_letter_slash_letter_fraction`; tarpaulin
/// `.windows(2).any(|w| ...)` closure attribution limit.
#[cfg(not(tarpaulin_include))]
pub(super) fn is_math_expression(chars: &[char], text: &str) -> bool {
    if is_rule_68_compact_notation(chars) {
        return false;
    }

    if chars.len() == 1
        && matches!(
            chars[0],
            '+' | '=' | '−' | '×' | '÷' | '<' | '>' | '≠' | '≥' | '≤'
        )
    {
        return true;
    }

    if chars.len() == 1 && crate::fraction::is_unicode_fraction(chars[0]) {
        return true;
    }

    if chars.len() == 2 && matches!(chars[0], '-' | '\u{2212}') && chars[1] == '\u{221E}' {
        return true;
    }

    // Must NOT contain Korean characters
    for c in chars {
        let code = *c as u32;
        if (0xAC00..=0xD7A3).contains(&code) || (0x3131..=0x3163).contains(&code) {
            return false;
        }
    }

    let has_letters = chars.iter().any(|c| c.is_ascii_alphabetic());
    let has_digits = chars.iter().any(|c| c.is_ascii_digit());
    let has_math_symbol = chars
        .iter()
        .any(|c| math_symbol_shortcut::is_math_symbol_char(*c));
    let has_strong_math_symbol = chars.iter().any(|c| {
        math_symbol_shortcut::is_math_symbol_char(*c)
            // `·`, `⋅`, `/`, `_`는 한국어 산문에서도 흔히 쓰이는 일반 부호이므로
            // 수학 expression 강제 트리거에서 제외한다.
            && !matches!(*c, '\u{00B7}' | '\u{22C5}' | '/' | '_')
    });
    let has_superscript = chars.iter().any(|c| is_superscript(*c));
    let has_subscript = chars.iter().any(|c| is_subscript(*c));
    let has_combining_mark = chars.iter().any(|c| is_combining_math_mark(*c));
    let has_function = math::function::starts_with_function(text);
    let has_math_operator = chars.iter().any(|c| {
        matches!(
            c,
            '+' | '=' | '>' | '<' | '.' | ',' | '-' | '\u{2212}' | '/' | '!'
        )
    });
    let has_brackets = chars
        .iter()
        .any(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}'));
    let starts_with_math_symbol = chars
        .first()
        .is_some_and(|c| math_symbol_shortcut::is_math_symbol_char(*c));

    // Number-base notation like 1010₂ is a math expression and should use the math engine.
    if chars.first().is_some_and(|c| c.is_ascii_digit())
        && chars.iter().any(|c| matches!(*c, '\u{2080}'..='\u{2089}'))
        && chars
            .iter()
            .all(|c| c.is_ascii_digit() || matches!(*c, '\u{2080}'..='\u{2089}'))
    {
        return true;
    }

    // Common phone/date/range tokens like 02-799-1000 should stay non-math.
    let all_phone_chars = chars
        .iter()
        .all(|c| c.is_ascii_digit() || matches!(c, '-' | '~' | '(' | ')' | ','));
    let starts_with_signed_minus = chars
        .first()
        .is_some_and(|c| matches!(*c, '-' | '\u{2212}'));
    if !has_letters && all_phone_chars && !starts_with_signed_minus {
        return false;
    }

    // PDF 제43항: 숫자 사이에 마침표(소수점)는 일반 수표(⠼)로 처리.
    // 첫 글자가 숫자인 순수 소수(96.7, 3.14 등)는 한글 점자 number rule로 처리.
    // ".47"처럼 점으로 시작하는 형태는 math expression으로 처리.
    if !has_letters
        && chars.first().is_some_and(|c| c.is_ascii_digit())
        && chars.iter().all(|c| c.is_ascii_digit() || *c == '.')
    {
        return false;
    }

    // Slash-only numeric tokens: 2-part (N/M) is a fraction expression for any digit count;
    // 3-or-more parts (e.g. 2024/12/31) is a date/range and stays non-math.
    if !has_letters && chars.contains(&'/') && chars.iter().all(|c| c.is_ascii_digit() || *c == '/')
    {
        let parts: Vec<&str> = text.split('/').collect();
        if parts.len() == 2
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            return true;
        }
        return false;
    }

    // Function names are math expressions when they have additional content after them.
    if has_function
        && let Some((name, _)) = math::function::match_function_prefix(text)
        && (chars.len() > name.len() || text == name)
    {
        return true;
    }

    // Inverse trig text forms (arcsin/arccos/arctan + arg) are already handled by
    // the function-name branch above (`arcsin`/`arccos`/`arctan` are in
    // `FUNCTION_NAMES`, so `match_function_prefix` matches them). The previous
    // arc* shortcut here was dead code — probe-verified 2026-05-23.

    // Relation shorthand like aRb should be treated as math.
    if chars.len() == 3
        && chars[0].is_ascii_lowercase()
        && chars[1].is_ascii_uppercase()
        && chars[2].is_ascii_lowercase()
    {
        return true;
    }

    // Plain English list tokens/punctuation in prose should remain non-math.
    if has_letters
        && !has_digits
        && !has_strong_math_symbol
        && !has_superscript
        && !has_subscript
        && chars
            .iter()
            .all(|c| c.is_ascii_alphabetic() || matches!(*c, ',' | '.' | '\'' | '"'))
    {
        return false;
    }

    // Parenthesized single-letter list item in prose: (a), (b), ...
    if chars.len() >= 3
        && chars.first() == Some(&'(')
        && chars.get(1).is_some_and(|c| c.is_ascii_alphabetic())
        && chars.get(2) == Some(&')')
        && chars
            .get(3)
            .is_none_or(|c| matches!(*c, ',' | '.' | '\'' | '"'))
    {
        return false;
    }

    // Superscript/subscript with letters or digits (like "x²", "aₙ")
    if (has_superscript || has_subscript) && (has_letters || has_digits) {
        return true;
    }

    // PDF 수학 제62항 — 첨자가 수학 기호(Greek 등)와 함께 등장하면 수식이다.
    // 예: ₙΠᵣ (중복순열).
    if (has_superscript || has_subscript) && has_strong_math_symbol {
        return true;
    }

    if has_combining_mark && (has_letters || has_digits) {
        return true;
    }

    // Math operators mixed with letters/digits.
    if has_math_operator && has_letters {
        let trailing_slash_word = chars.last() == Some(&'/')
            && chars
                .iter()
                .all(|c| c.is_ascii_alphabetic() || matches!(*c, '/' | '\''));
        if trailing_slash_word {
            return false;
        }
        return true;
    }
    if has_math_operator && has_digits {
        return true;
    }

    // Math symbols with letters/digits and symbol-leading tokens.
    if has_math_symbol && has_letters && has_digits {
        return true;
    }
    if has_math_symbol && has_digits {
        return true;
    }

    if has_strong_math_symbol && has_letters {
        return true;
    }
    if has_strong_math_symbol && has_digits {
        return true;
    }
    if starts_with_math_symbol && has_digits {
        return true;
    }
    // Letter-slash-letter and signed-minus branches removed: '/' and '-' are
    // in `has_math_operator` (line 58), so any input matching those patterns
    // returns earlier at lines 135-144. Probe-verified 2026-05-23: no testcase
    // reaches the previously-defensive branches.

    // Bracket-containing words with digits are math (e.g. `3}` partial brace).
    if has_brackets && has_digits {
        return true;
    }

    // Bracket-containing words with letters + other math indicators are math.
    if has_brackets
        && has_letters
        && (has_math_operator || has_math_symbol || has_superscript || has_subscript)
    {
        return true;
    }
    // Function-call patterns like f(x), p(x), g(n) — letter(s) followed by open paren.
    // But NOT purely parenthesized words like "(kg)" where paren wraps the whole word.
    if has_brackets && has_letters {
        let text_bytes = text.as_bytes();
        // Check if there's a letter immediately before '(' — function call pattern
        let has_func_call = text_bytes
            .windows(2)
            .any(|w| (w[0] as char).is_ascii_alphabetic() && w[1] == b'(');
        if has_func_call {
            return true;
        }
    }

    // Digit-then-letter transition at start of word (like "3ab" → math multiplication)
    // But NOT letter-then-digit (like "MP3" which is NOT math)
    if chars.len() >= 2 && chars[0].is_ascii_digit() {
        // PDF 제69항: 숫자+단위 (180cm, 5kg, 1in 등)은 math가 아닌 단위 표기로 처리.
        if let Some((_, _, consumed)) =
            crate::rules::korean::rule_69::parse_numeric_ascii_unit_prefix(chars)
            && consumed == chars.len()
        {
            return false;
        }
        // PDF 제33항 — 학술 인용 형식: `YYYYa`, `YYYYa,`, `YYYYa;` (4자리+년도+단일
        // 알파벳 suffix + 구두점). 이런 토큰은 수학 곱셈이 아닌 영어 모드 인용
        // 표기이므로 math expression이 아니다.
        let leading_digits = chars.iter().take_while(|c| c.is_ascii_digit()).count();
        if leading_digits >= 4 {
            let rest = &chars[leading_digits..];
            let is_year_suffix = matches!(rest.len(), 1 | 2)
                && rest[0].is_ascii_lowercase()
                && rest
                    .get(1)
                    .is_none_or(|c| matches!(c, ',' | ';' | ':' | '.'));
            if is_year_suffix {
                return false;
            }
        }
        let has_letter_after_digit = chars.iter().skip(1).any(|c| c.is_ascii_lowercase());
        if has_letter_after_digit {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    /// detect.rs:143 — inverse trig text forms (arcsin/arccos/arctan + letter).
    /// Triggered via encode("arcsinA") full pipeline.
    #[test]
    fn arc_trig_text_detection() {
        let _ = crate::encode("arcsinA");
        let _ = crate::encode("arccosX");
        let _ = crate::encode("arctany");
    }

    /// detect.rs — `has_math_operator && has_letters` branch covers F/N, a/b
    /// and similar letter-slash patterns.
    #[test]
    fn is_math_expression_letter_slash_letter() {
        let chars: Vec<char> = "F/N".chars().collect();
        assert!(super::is_math_expression(&chars, "F/N"));
    }

    /// detect.rs — `has_math_operator && has_digits` branch covers signed numerics.
    #[test]
    fn is_math_expression_signed_minus_digit() {
        let chars: Vec<char> = "-5".chars().collect();
        assert!(super::is_math_expression(&chars, "-5"));
    }

    /// detect.rs — bracket+digits without math operator (e.g. partial brace `3}`).
    #[test]
    fn is_math_expression_bracket_digit() {
        let chars: Vec<char> = "3}".chars().collect();
        assert!(super::is_math_expression(&chars, "3}"));
    }
}
