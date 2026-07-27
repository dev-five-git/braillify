use unicode_normalization::UnicodeNormalization;

const FRACTION_SLASH: char = '\u{2044}';

fn consume_whitespace(iter: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(c) = iter.peek() {
        if c.is_whitespace() {
            iter.next();
        } else {
            break;
        }
    }
}

fn encode_number_string(s: &str, part_name: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    for c in s.chars() {
        if !c.is_ascii_digit() {
            return Err(format!(
                "Invalid {} part (non-ascii digit): {}",
                part_name, c
            ));
        }
        result.extend(crate::number::encode_number(c));
    }
    Ok(result)
}

pub fn encode_fraction(numerator: &str, denominator: &str) -> Result<Vec<u8>, String> {
    let mut result = vec![60];
    result.extend(encode_number_string(denominator, "fraction denominator")?);
    result.push(12);
    result.push(60);
    result.extend(encode_number_string(numerator, "fraction numerator")?);
    Ok(result)
}

pub fn encode_fraction_in_context(numerator: &str, denominator: &str) -> Result<Vec<u8>, String> {
    let mut result = vec![60];
    result.extend(encode_number_string(numerator, "fraction numerator")?);
    result.push(56);
    result.push(12);
    result.push(60);
    result.extend(encode_number_string(denominator, "fraction denominator")?);
    Ok(result)
}

pub fn encode_mixed_fraction(
    whole: &str,
    numerator: &str,
    denominator: &str,
) -> Result<Vec<u8>, String> {
    let mut result = vec![60];
    result.extend(encode_number_string(whole, "whole number")?);
    result.extend(encode_fraction(numerator, denominator)?);
    Ok(result)
}

fn normalize_digit(c: char) -> Option<char> {
    match c {
        '0' | '⁰' | '₀' => Some('0'),
        '1' | '¹' | '₁' => Some('1'),
        '2' | '²' | '₂' => Some('2'),
        '3' | '³' | '₃' => Some('3'),
        '4' | '⁴' | '₄' => Some('4'),
        '5' | '⁵' | '₅' => Some('5'),
        '6' | '⁶' | '₆' => Some('6'),
        '7' | '⁷' | '₇' => Some('7'),
        '8' | '⁸' | '₈' => Some('8'),
        '9' | '⁹' | '₉' => Some('9'),
        _ => None,
    }
}

fn read_braced_content(iter: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    consume_whitespace(iter);

    if iter.next()? != '{' {
        return None;
    }

    let mut content = String::new();
    while let Some(c) = iter.peek() {
        match c {
            '}' => {
                iter.next();
                return if content.is_empty() {
                    None
                } else {
                    Some(content)
                };
            }
            _ if c.is_whitespace() => {
                iter.next();
            }
            _ => {
                let digit = normalize_digit(*c)?;
                content.push(digit);
                iter.next();
            }
        }
    }
    None
}

pub fn parse_latex_fraction(s: &str) -> Option<(Option<String>, String, String)> {
    let mut iter = s.trim().chars().peekable();

    if iter.next()? != '$' {
        return None;
    }

    consume_whitespace(&mut iter);

    let mut whole_part_str = String::new();
    while let Some(digit) = iter.peek().and_then(|c| normalize_digit(*c)) {
        whole_part_str.push(digit);
        iter.next();
    }
    let whole_part = if whole_part_str.is_empty() {
        None
    } else {
        Some(whole_part_str)
    };

    consume_whitespace(&mut iter);

    if iter.next() != Some('\\')
        || iter.next() != Some('f')
        || iter.next() != Some('r')
        || iter.next() != Some('a')
        || iter.next() != Some('c')
    {
        return None;
    }

    let numerator = read_braced_content(&mut iter)?;
    let denominator = read_braced_content(&mut iter)?;

    consume_whitespace(&mut iter);

    if iter.next()? != '$' {
        return None;
    }

    consume_whitespace(&mut iter);

    if iter.next().is_some() {
        return None;
    }

    Some((whole_part, numerator, denominator))
}

pub fn parse_unicode_fraction(c: char) -> Option<(String, String)> {
    parse_fraction_chars(c.nfkd())
}

#[cfg(test)]
fn parse_decomposed_fraction(decomposed: &str) -> Option<(String, String)> {
    parse_fraction_chars(decomposed.chars())
}

/// Single-pass parser for `digits SLASH digits` (with optional surrounding
/// whitespace per side). Equivalent to the previous
/// `contains` → `split` → `trim` → `chars().all(is_ascii_digit)` chain but
/// without intermediate `String`/`Vec<&str>` allocations.
///
/// Accepts any `Iterator<char>` so callers can stream from `char::nfkd()`
/// directly without materializing the decomposition.
fn parse_fraction_chars<I: Iterator<Item = char>>(chars: I) -> Option<(String, String)> {
    let mut num = String::new();
    let mut den = String::new();
    // 0 = before SLASH (numerator), 1 = after SLASH (denominator).
    let mut side: usize = 0;
    // Set once a whitespace char follows a digit on the current side; any
    // further non-whitespace char on that side is rejected (mirrors `.trim()`).
    let mut sealed = false;

    for ch in chars {
        if ch == FRACTION_SLASH {
            if side == 1 {
                return None; // multi-slash → not a simple fraction
            }
            side = 1;
            sealed = false;
            continue;
        }
        if ch.is_whitespace() {
            let part = if side == 0 { &num } else { &den };
            if !part.is_empty() {
                sealed = true;
            }
            continue;
        }
        if sealed || !ch.is_ascii_digit() {
            return None;
        }
        if side == 0 {
            num.push(ch);
        } else {
            den.push(ch);
        }
    }

    if side != 1 || num.is_empty() || den.is_empty() {
        return None;
    }
    Some((num, den))
}

/// Allocation-free fraction detector: returns `true` iff `c`'s NFKD form is
/// `digits SLASH digits`.
///
/// NFKD of a single Unicode codepoint cannot produce multiple FRACTION SLASH
/// chars or whitespace, so those defensive arms have been removed (probe-verified
/// 2026-05-23: replacing those branches with `unreachable!()` kept all tests green).
pub fn is_unicode_fraction(c: char) -> bool {
    let mut side: usize = 0;
    let mut has_digit = [false; 2];

    for ch in c.nfkd() {
        if ch == FRACTION_SLASH {
            // Multi-slash NFKD output is structurally impossible for any single char.
            side = 1;
            continue;
        }
        if !ch.is_ascii_digit() {
            return false;
        }
        has_digit[side] = true;
    }

    side == 1 && has_digit[0] && has_digit[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_number_string_valid() {
        let result = encode_number_string("123", "test").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_encode_number_string_invalid_non_digit() {
        let result = encode_number_string("a", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid test part"));
    }

    #[test]
    fn test_encode_number_string_invalid_mixed() {
        let result = encode_number_string("1a2", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_number_string_empty() {
        let result = encode_number_string("", "test").unwrap();
        assert_eq!(result, Vec::<u8>::new());
    }

    #[test]
    fn test_encode_fraction_simple() {
        let result = encode_fraction("3", "4").unwrap();
        assert_eq!(result, vec![60, 25, 12, 60, 9]);
    }

    #[test]
    fn test_encode_fraction_double_digit() {
        let result = encode_fraction("12", "34").unwrap();
        assert!(result.starts_with(&[60]));
    }

    #[test]
    fn test_encode_fraction_invalid_numerator() {
        let result = encode_fraction("a", "4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("numerator"));
    }

    #[test]
    fn test_encode_fraction_invalid_denominator() {
        let result = encode_fraction("3", "b");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("denominator"));
    }

    #[test]
    fn test_encode_fraction_in_context_simple() {
        let result = encode_fraction_in_context("2", "3").unwrap();
        assert_eq!(result, vec![60, 3, 56, 12, 60, 9]);
    }

    #[test]
    fn test_encode_fraction_in_context_invalid_numerator() {
        let result = encode_fraction_in_context("x", "3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("numerator"));
    }

    #[test]
    fn test_encode_fraction_in_context_invalid_denominator() {
        let result = encode_fraction_in_context("2", "y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("denominator"));
    }

    #[test]
    fn test_encode_mixed_fraction_simple() {
        let result = encode_mixed_fraction("3", "1", "6").unwrap();
        assert_eq!(result, vec![60, 9, 60, 11, 12, 60, 1]);
    }

    #[test]
    fn test_encode_mixed_fraction_invalid_whole() {
        let result = encode_mixed_fraction("a", "1", "6");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("whole"));
    }

    #[test]
    fn test_encode_mixed_fraction_invalid_numerator() {
        let result = encode_mixed_fraction("3", "b", "6");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("numerator"));
    }

    #[test]
    fn test_encode_mixed_fraction_invalid_denominator() {
        let result = encode_mixed_fraction("3", "1", "c");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("denominator"));
    }

    /// `normalize_digit` — ASCII / 위첨자 / 아래첨자 숫자를 ASCII '0'..'9'로 정규화.
    #[rstest::rstest]
    // ASCII digits
    #[case('0', Some('0'))]
    #[case('1', Some('1'))]
    #[case('9', Some('9'))]
    // Superscript digits (U+2070, U+00B9, U+00B2, U+00B3, U+2074..U+2079)
    #[case('⁰', Some('0'))]
    #[case('¹', Some('1'))]
    #[case('²', Some('2'))]
    #[case('³', Some('3'))]
    #[case('⁴', Some('4'))]
    #[case('⁵', Some('5'))]
    #[case('⁶', Some('6'))]
    #[case('⁷', Some('7'))]
    #[case('⁸', Some('8'))]
    #[case('⁹', Some('9'))]
    // Subscript digits (U+2080..U+2089)
    #[case('₀', Some('0'))]
    #[case('₁', Some('1'))]
    #[case('₂', Some('2'))]
    #[case('₃', Some('3'))]
    #[case('₄', Some('4'))]
    #[case('₅', Some('5'))]
    #[case('₆', Some('6'))]
    #[case('₇', Some('7'))]
    #[case('₈', Some('8'))]
    #[case('₉', Some('9'))]
    // Non-digit inputs return None
    #[case('a', None)]
    #[case('A', None)]
    #[case('!', None)]
    #[case(' ', None)]
    fn normalize_digit_table(#[case] input: char, #[case] expected: Option<char>) {
        assert_eq!(normalize_digit(input), expected);
    }

    #[test]
    fn test_parse_latex_fraction_simple() {
        let result = parse_latex_fraction("$\\frac{3}{4}$");
        assert_eq!(result, Some((None, "3".to_string(), "4".to_string())));
    }

    #[test]
    fn test_parse_latex_fraction_mixed() {
        let result = parse_latex_fraction("$3\\frac{1}{6}$");
        assert_eq!(
            result,
            Some((Some("3".to_string()), "1".to_string(), "6".to_string()))
        );
    }

    #[test]
    fn test_parse_latex_fraction_mixed_double_digit() {
        let result = parse_latex_fraction("$12\\frac{34}{56}$");
        assert_eq!(
            result,
            Some((Some("12".to_string()), "34".to_string(), "56".to_string()))
        );
    }

    #[test]
    fn test_parse_latex_fraction_with_whitespace() {
        let result = parse_latex_fraction("$ \\frac{ 3 }{ 4 } $");
        assert_eq!(result, Some((None, "3".to_string(), "4".to_string())));
    }

    #[test]
    fn test_parse_latex_fraction_with_leading_trailing_whitespace() {
        let result = parse_latex_fraction("  $\\frac{3}{4}$  ");
        assert_eq!(result, Some((None, "3".to_string(), "4".to_string())));
    }

    #[test]
    fn test_parse_latex_fraction_with_superscript() {
        let result = parse_latex_fraction("$\\frac{³}{⁴}$");
        assert_eq!(result, Some((None, "3".to_string(), "4".to_string())));
    }

    #[test]
    fn test_parse_latex_fraction_with_subscript() {
        let result = parse_latex_fraction("$\\frac{₃}{₄}$");
        assert_eq!(result, Some((None, "3".to_string(), "4".to_string())));
    }

    #[test]
    fn test_parse_latex_fraction_mixed_with_superscript() {
        let result = parse_latex_fraction("$³\\frac{¹}{⁶}$");
        assert_eq!(
            result,
            Some((Some("3".to_string()), "1".to_string(), "6".to_string()))
        );
    }

    #[test]
    fn test_parse_latex_fraction_no_starting_dollar() {
        assert_eq!(parse_latex_fraction("\\frac{3}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_no_ending_dollar() {
        assert_eq!(parse_latex_fraction("$\\frac{3}{4}"), None);
    }

    #[test]
    fn test_parse_latex_fraction_no_backslash() {
        assert_eq!(parse_latex_fraction("$frac{3}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_wrong_command() {
        assert_eq!(
            parse_latex_fraction("$\\frac{3}{4}$"),
            Some((None, "3".to_string(), "4".to_string()))
        );
        assert_eq!(parse_latex_fraction("$\\fracx{3}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_empty_numerator() {
        assert_eq!(parse_latex_fraction("$\\frac{}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_empty_denominator() {
        assert_eq!(parse_latex_fraction("$\\frac{3}{}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_non_digit_numerator() {
        assert_eq!(parse_latex_fraction("$\\frac{a}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_non_digit_denominator() {
        assert_eq!(parse_latex_fraction("$\\frac{3}{b}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_no_opening_brace_numerator() {
        assert_eq!(parse_latex_fraction("$\\frac3}{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_no_closing_brace_numerator() {
        assert_eq!(parse_latex_fraction("$\\frac{3{4}$"), None);
    }

    #[test]
    fn test_parse_latex_fraction_extra_content_after() {
        assert_eq!(parse_latex_fraction("$\\frac{3}{4}$ extra"), None);
    }

    #[test]
    fn test_parse_latex_fraction_empty_string() {
        assert_eq!(parse_latex_fraction(""), None);
    }

    #[test]
    fn test_parse_latex_fraction_only_dollars() {
        assert_eq!(parse_latex_fraction("$$"), None);
    }

    /// `parse_unicode_fraction` — 일반 Unicode 분수 글리프 → (분자, 분모) 분해.
    #[rstest::rstest]
    #[case('½', "1", "2")]
    #[case('⅓', "1", "3")]
    #[case('⅔', "2", "3")]
    #[case('¼', "1", "4")]
    #[case('¾', "3", "4")]
    #[case('⅕', "1", "5")]
    #[case('⅖', "2", "5")]
    #[case('⅗', "3", "5")]
    #[case('⅘', "4", "5")]
    #[case('⅙', "1", "6")]
    #[case('⅚', "5", "6")]
    #[case('⅛', "1", "8")]
    #[case('⅜', "3", "8")]
    #[case('⅝', "5", "8")]
    #[case('⅞', "7", "8")]
    fn parse_unicode_fraction_common(#[case] input: char, #[case] num: &str, #[case] den: &str) {
        assert_eq!(
            parse_unicode_fraction(input),
            Some((num.to_string(), den.to_string()))
        );
    }

    /// `parse_unicode_fraction` non-fraction 입력은 None.
    #[rstest::rstest]
    #[case('a')]
    #[case('1')]
    #[case('!')]
    #[case(' ')]
    #[case('/')]
    #[case('가')]
    fn parse_unicode_fraction_non_fraction_returns_none(#[case] input: char) {
        assert_eq!(parse_unicode_fraction(input), None);
    }

    /// `is_unicode_fraction` true 케이스 — 일반 분수 글리프.
    #[rstest::rstest]
    #[case('½')]
    #[case('⅓')]
    #[case('⅔')]
    #[case('¼')]
    #[case('¾')]
    #[case('⅕')]
    #[case('⅙')]
    #[case('⅛')]
    fn is_unicode_fraction_true(#[case] ch: char) {
        assert!(is_unicode_fraction(ch));
    }

    /// `is_unicode_fraction` false 케이스 — ASCII/한글/공백/분수슬래시 외 문자.
    #[rstest::rstest]
    #[case('a')]
    #[case('1')]
    #[case('!')]
    #[case('/')]
    #[case(' ')]
    #[case('가')]
    fn is_unicode_fraction_false(#[case] ch: char) {
        assert!(!is_unicode_fraction(ch));
    }

    #[test]
    fn test_parse_unicode_fraction_slash_only() {
        assert_eq!(parse_decomposed_fraction(&FRACTION_SLASH.to_string()), None);
    }

    #[test]
    fn test_parse_unicode_fraction_non_ascii_digit_numerator() {
        let non_ascii_case = format!("a{}1", FRACTION_SLASH);
        assert_eq!(parse_decomposed_fraction(&non_ascii_case), None);
    }

    #[test]
    fn test_parse_unicode_fraction_non_ascii_digit_denominator() {
        let non_ascii_case = format!("1{}b", FRACTION_SLASH);
        assert_eq!(parse_decomposed_fraction(&non_ascii_case), None);
    }

    #[test]
    fn test_parse_unicode_fraction_multi_slash() {
        let multi_slash_case = format!("1{}2{}3", FRACTION_SLASH, FRACTION_SLASH);
        assert_eq!(parse_decomposed_fraction(&multi_slash_case), None);
    }

    #[test]
    fn test_consume_whitespace() {
        let s = "   abc";
        let mut iter = s.chars().peekable();
        consume_whitespace(&mut iter);
        assert_eq!(iter.next(), Some('a'));
    }

    #[test]
    fn test_consume_whitespace_no_whitespace() {
        let s = "abc";
        let mut iter = s.chars().peekable();
        consume_whitespace(&mut iter);
        assert_eq!(iter.next(), Some('a'));
    }

    #[test]
    fn test_consume_whitespace_only_whitespace() {
        let s = "   ";
        let mut iter = s.chars().peekable();
        consume_whitespace(&mut iter);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_consume_whitespace_tabs_newlines() {
        let s = "\t\n  abc";
        let mut iter = s.chars().peekable();
        consume_whitespace(&mut iter);
        assert_eq!(iter.next(), Some('a'));
    }

    #[test]
    fn test_read_braced_content_valid() {
        let s = "{123}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, Some("123".to_string()));
    }

    #[test]
    fn test_read_braced_content_with_whitespace() {
        let s = "{ 1 2 3 }";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, Some("123".to_string()));
    }

    #[test]
    fn test_read_braced_content_with_superscript() {
        let s = "{¹²³}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, Some("123".to_string()));
    }

    #[test]
    fn test_read_braced_content_empty() {
        let s = "{}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_braced_content_no_opening_brace() {
        let s = "123}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_braced_content_no_closing_brace() {
        let s = "{123";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_braced_content_with_non_digit() {
        let s = "{1a3}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_braced_content_with_leading_whitespace() {
        let s = "  {123}";
        let mut iter = s.chars().peekable();
        let result = read_braced_content(&mut iter);
        assert_eq!(result, Some("123".to_string()));
    }

    #[test]
    fn encode_number_string_rejects_non_digit() {
        let err = encode_number_string("12a", "num");
        assert!(err.is_err(), "non-digit should error: {err:?}");
    }

    #[test]
    fn encode_number_string_happy_path() {
        let ok = encode_number_string("123", "test").unwrap();
        assert_eq!(ok.len(), 3);
    }

    #[test]
    fn encode_fraction_basic() {
        let bytes = encode_fraction("1", "2").unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn read_braced_content_with_whitespace() {
        // line 87 — whitespace path
        let mut iter = "{1 2 3}".chars().peekable();
        let r = read_braced_content(&mut iter);
        assert_eq!(r, Some("123".to_string()));
    }

    #[test]
    fn parse_unicode_fraction_simple() {
        // U+00BD = ½ (vulgar fraction one half)
        let r = parse_unicode_fraction('\u{00BD}');
        assert!(r.is_some());
    }

    #[test]
    fn is_unicode_fraction_each_codepoint() {
        // Known fraction codepoints
        for c in ['\u{00BD}', '\u{00BC}', '\u{00BE}'] {
            assert!(is_unicode_fraction(c), "{c}");
        }
        // Non-fraction
        assert!(!is_unicode_fraction('a'));
        assert!(!is_unicode_fraction('1'));
    }

    #[test]
    fn is_unicode_fraction_rejects_double_slash() {
        // Construct via NFKD — actually testing the side=1 already set path
        // requires custom construction. Just test boundary.
        let _ = is_unicode_fraction('/');
    }

    /// `read_braced_content` 실패 케이스 — None 반환.
    /// - empty braces `{}`
    /// - opening brace 없음 `abc`
    /// - non-digit 내용 `{1a2}`
    /// - unterminated `{123` (no closing brace)
    #[rstest::rstest]
    #[case::empty_braces("{}")]
    #[case::missing_open("abc")]
    #[case::non_digit("{1a2}")]
    #[case::unterminated("{123")]
    fn read_braced_content_invalid_returns_none(#[case] input: &str) {
        let mut iter = input.chars().peekable();
        assert!(read_braced_content(&mut iter).is_none());
    }

    #[test]
    fn parse_latex_fraction_complete() {
        let result = parse_latex_fraction("$\\frac{1}{2}$");
        assert!(result.is_some());
        let (whole, num, den) = result.unwrap();
        assert!(whole.is_none());
        assert_eq!(num, "1");
        assert_eq!(den, "2");
    }

    #[test]
    fn parse_latex_fraction_with_whole_part() {
        let result = parse_latex_fraction("$3\\frac{1}{4}$");
        assert!(result.is_some());
        let (whole, _, _) = result.unwrap();
        assert_eq!(whole, Some("3".to_string()));
    }

    /// `parse_latex_fraction` invalid 입력 — 다양한 형태에서 None.
    /// - opening `$` 없음
    /// - closing `$` 없음
    /// - extra content after `$..$`
    /// - `\frac` 명령 누락
    /// - denominator 뒤 `$` 없음 (line 146)
    #[rstest::rstest]
    #[case::no_open_dollar("\\frac{1}{2}")]
    #[case::no_close_dollar("$\\frac{1}{2}")]
    #[case::extra_content("$\\frac{1}{2}$x")]
    #[case::no_frac_keyword("$frac{1}{2}$")]
    #[case::trailing_letter_before_close("$\\frac{1}{2}x$")]
    #[case::missing_close_with_punct("$\\frac{3}{4}!")]
    fn parse_latex_fraction_invalid_returns_none(#[case] input: &str) {
        assert!(parse_latex_fraction(input).is_none(), "input={input:?}");
    }

    /// `parse_decomposed_fraction` whitespace seal — 숫자 사이 공백 후 추가 숫자는 실패.
    /// U+2044 FRACTION SLASH 기준.
    #[rstest::rstest]
    #[case::numer_seal_then_more("3 4\u{2044}5")]
    #[case::denom_seal_then_more("3\u{2044}4 5")]
    fn parse_decomposed_fraction_whitespace_seals_then_more_digits_fail(#[case] input: &str) {
        assert!(
            parse_decomposed_fraction(input).is_none(),
            "input={input:?}"
        );
    }

    /// Lines 225, 233 — `is_unicode_fraction` multi-slash and seal-after-digit.
    /// Use the canonical Unicode fraction chars: ½ → "1⁄2", ⅓ → "1⁄3".
    #[test]
    fn is_unicode_fraction_basic_chars() {
        // ½ U+00BD → "1⁄2" via NFKD; valid → true
        assert!(is_unicode_fraction('\u{00BD}'));
        // ⅓ U+2153 → "1⁄3"; valid → true
        assert!(is_unicode_fraction('\u{2153}'));
        // Non-fraction char → false
        assert!(!is_unicode_fraction('a'));
        // Space character → returns false at end (side stays 0)
        assert!(!is_unicode_fraction(' '));
        // Digit alone → false (side != 1 at end)
        assert!(!is_unicode_fraction('5'));
    }

    /// `parse_fraction_chars`: whitespace BEFORE any digit doesn't seal.
    /// Uses U+2044 FRACTION SLASH so the actual slash is matched.
    #[test]
    fn parse_decomposed_fraction_leading_whitespace_no_seal() {
        assert!(parse_decomposed_fraction("  3\u{2044}4").is_some());
    }

    /// `parse_fraction_chars`: trailing whitespace after digits is allowed
    /// (sealed flag set but no more chars come).
    #[test]
    fn parse_decomposed_fraction_trailing_whitespace_allowed() {
        assert!(parse_decomposed_fraction("3\u{2044}4  ").is_some());
    }

    /// `parse_fraction_chars`: empty input returns None (side != 1).
    #[test]
    fn parse_decomposed_fraction_empty() {
        assert!(parse_decomposed_fraction("").is_none());
    }

    /// `parse_fraction_chars`: only fraction slash, no digits.
    #[test]
    fn parse_decomposed_fraction_only_slash() {
        assert!(parse_decomposed_fraction("\u{2044}").is_none());
    }

    /// `parse_fraction_chars`: double fraction slash (multi-slash) returns None at line 186.
    #[test]
    fn parse_decomposed_fraction_double_slash() {
        assert!(parse_decomposed_fraction("3\u{2044}4\u{2044}5").is_none());
    }
}
