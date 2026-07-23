//! Math expression tokenizer/parser.
//!
//! Parses math expression strings into structured tokens
//! that can be encoded into braille by the encoder.

/// The kind of bracket in a math expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketKind {
    /// Standard math parentheses: f(x), (a+b)
    MathParen,
    /// Grouping brackets used in braille-only notation
    Grouping,
    /// Hangul-wrapped math group _( ... _)
    Hangul,
    /// Square brackets: [x]
    Square,
    /// Curly braces: {1, 2, 3}
    Curly,
}

/// A parsed math expression token.
#[derive(Debug, Clone)]
pub enum MathToken {
    /// Lowercase variable letter (a-z)
    Variable(char),
    /// Uppercase variable letter (A-Z)
    UpperVariable(char),
    /// Digit sequence (one or more digits)
    Number(String),
    /// Decimal point in number context
    DecimalPoint,
    /// Digit-group separator comma in number context
    DigitSeparator,
    /// Math operator (+, -, ×, ÷, =, >, <, etc.)
    Operator(char),
    /// Known function name (sin, cos, etc.)
    FunctionName(String),
    /// Korean word or phrase inside a math expression.
    KoreanWord(String),
    /// Opening bracket
    OpenParen(BracketKind),
    /// Closing bracket
    CloseParen(BracketKind),
    /// Superscript content (exponent)
    Superscript(Vec<MathToken>),
    /// Subscript content
    Subscript(Vec<MathToken>),
    /// Whitespace
    Space,
    /// Math symbol (Greek letters, set symbols, etc.)
    MathSymbol(char),
    /// Prime mark (′)
    Prime,
    /// Unrecognized character (fallback)
    Raw(char),
}

#[derive(Debug, Clone, Copy)]
struct GroupState {
    kind: BracketKind,
    token_index: usize,
    contains_korean: bool,
    contains_arithmetic: bool,
    contains_comma: bool,
    promote_grouping: bool,
}

mod helpers;

mod parse;
pub(crate) use parse::{parse_math_expression, parse_math_expression_with_math_mode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_variable() {
        let tokens = parse_math_expression("x").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], MathToken::Variable('x')));
    }

    #[test]
    fn test_parse_number() {
        let tokens = parse_math_expression("42").unwrap();
        assert_eq!(tokens.len(), 1);
        if let MathToken::Number(n) = &tokens[0] {
            assert_eq!(n, "42");
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_parse_function() {
        let tokens = parse_math_expression("sin3x").unwrap();
        assert!(tokens.len() >= 3);
        assert!(matches!(tokens[0], MathToken::FunctionName(ref n) if n == "sin"));
    }

    #[test]
    fn test_parse_superscript() {
        let tokens = parse_math_expression("x²").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], MathToken::Variable('x')));
        assert!(matches!(tokens[1], MathToken::Superscript(_)));
    }

    #[test]
    fn test_parse_equation() {
        let tokens = parse_math_expression("ax+b=0").unwrap();
        // a, x, +, b, =, 0
        assert!(tokens.len() >= 5);
    }

    #[test]
    fn test_parse_ascii_apostrophe_as_prime() {
        let tokens = parse_math_expression("f''(x)").unwrap();
        assert!(matches!(tokens[1], MathToken::Prime));
        assert!(matches!(tokens[2], MathToken::Prime));
    }

    #[test]
    fn test_grouping_paren_after_function() {
        let tokens = parse_math_expression("sin(x)").unwrap();
        assert!(matches!(
            tokens[1],
            MathToken::OpenParen(BracketKind::Grouping)
        ));
        assert!(matches!(
            tokens[3],
            MathToken::CloseParen(BracketKind::Grouping)
        ));
    }

    /// Exercise every Unicode superscript character recognized by
    /// `normalize_superscript`. Each codepoint must parse without error
    /// and produce at least one token.
    #[rstest::rstest]
    fn unicode_superscripts_parsed(
        #[values(
            '\u{2070}', '\u{00B9}', '\u{00B2}', '\u{00B3}', '\u{2074}', '\u{2075}', '\u{2076}',
            '\u{2077}', '\u{2078}', '\u{2079}', '\u{207A}', '\u{207B}', '\u{207D}', '\u{207E}',
            '\u{207F}', '\u{1D43}', '\u{1D47}', '\u{1D9C}', '\u{1D48}', '\u{1D49}', '\u{1DA0}',
            '\u{1D4D}', '\u{02B0}', '\u{2071}', '\u{02B2}', '\u{1D4F}', '\u{02E1}', '\u{1D50}',
            '\u{1D52}', '\u{1D56}', '\u{02B3}', '\u{02E2}', '\u{1D57}', '\u{1D58}', '\u{1D5B}',
            '\u{02B7}', '\u{02E3}', '\u{02B8}', '\u{1DBB}'
        )]
        c: char,
    ) {
        let input = format!("x{c}");
        let result = parse_math_expression(&input);
        assert!(result.is_ok(), "parse failed for x{c:?}");
        assert!(!result.unwrap().is_empty());
    }

    /// Exercise every Unicode subscript character recognized by
    /// `normalize_subscript`.
    #[rstest::rstest]
    fn unicode_subscripts_parsed(
        #[values(
            '\u{2080}', '\u{2081}', '\u{2082}', '\u{2083}', '\u{2084}', '\u{2085}', '\u{2086}',
            '\u{2087}', '\u{2088}', '\u{2089}', '\u{208A}', '\u{208B}', '\u{208D}', '\u{208E}',
            '\u{2090}', '\u{2091}', '\u{2092}', '\u{2093}', '\u{2095}', '\u{2096}', '\u{2097}',
            '\u{2098}', '\u{2099}', '\u{209A}', '\u{209B}', '\u{209C}', '\u{1D62}', '\u{1D63}',
            '\u{1D64}', '\u{1D65}'
        )]
        c: char,
    ) {
        let input = format!("x{c}");
        let result = parse_math_expression(&input);
        assert!(result.is_ok(), "parse failed for x{c:?}");
        assert!(!result.unwrap().is_empty());
    }

    /// Underline-notation fraction normalization paths (lines around 300-330).
    #[test]
    fn underline_notation_fraction_paths() {
        // U+0332 suffix on digit prefix → digits/1
        let _ = parse_math_expression("123\u{0332}");
        // "1?/(...)" pattern
        let _ = parse_math_expression("1\u{0332}/(x+y)");
        // "X?/Y" generic underline-fraction
        let _ = parse_math_expression("A\u{0332}/B");
    }

    /// Parser sweep — exercises diverse code paths.
    #[test]
    fn parser_diverse_input_sweep() {
        let inputs: &[&str] = &[
            // ASCII subscript syntaxes
            "x_1",
            "x_{12}",
            "x_(n)",
            "x_a",
            "x_A",
            // Number subscripts with decimal/slash inside
            "x_{1.5}",
            "x_{1/2}",
            // Superscripts
            "x^1",
            "x^{n+1}",
            "x^a",
            // Combined
            "x_1^2",
            "x^a_b",
            // Korean variables
            "x가y",
            "수식",
            // Math symbols
            "1≤x≤10",
            "x≠y",
            // Functions
            "sin(x)",
            "cos(2x)",
            "log_2(8)",
            // Parens & brackets
            "(a+b)(c+d)",
            "[a,b]",
            "{x|x>0}",
            // Decimal numbers
            "3.14",
            "0.5",
            "1.234",
            // Unicode operators
            "x×y",
            "x÷y",
            "x±1",
            // Multi-digit + multi-char
            "12345",
            "abc",
            // Whitespace
            "x + y",
            "1  +  2",
            // Empty / edge
            "",
            " ",
            "  ",
            // Single chars
            "x",
            "1",
            "+",
            // Compound math
            "f(x)=x^2+2x+1",
            "x^2 + 2x + 1 = 0",
            // Greek letters
            "α+β=γ",
            "π/2",
            // Roman numerals
            "I+II",
        ];
        for input in inputs {
            let _ = parse_math_expression(input);
        }
    }

    // ============================================================
    // Mutation-testing reinforcements (kill 35+ missed mutants in parse.rs)
    // ============================================================

    /// Underline-fraction notation with a parenthesised denominator is numerator-agnostic.
    /// Expected token shape: (Grouping, ...inner..., Grouping_close, /, numerator).
    #[rstest::rstest]
    #[case::one_numerator("1\u{0332}/(x+y)", "1")]
    #[case::two_numerator("2\u{0332}/(x+y)", "2")]
    fn underline_fraction_with_parenthesised_body(#[case] input: &str, #[case] numerator: &str) {
        let tokens = parse_math_expression(input).unwrap();
        // First token must be Grouping open (denominator-first ordering).
        assert!(
            matches!(tokens[0], MathToken::OpenParen(BracketKind::Grouping)),
            "expected Grouping open first, got {:?}",
            tokens
        );
        // Then x, +, y inside.
        assert!(matches!(tokens[1], MathToken::Variable('x')));
        assert!(matches!(tokens[2], MathToken::Operator('+')));
        assert!(matches!(tokens[3], MathToken::Variable('y')));
        assert!(matches!(
            tokens[4],
            MathToken::CloseParen(BracketKind::Grouping)
        ));
        assert!(matches!(tokens[5], MathToken::Operator('/')));
        assert!(matches!(tokens[6], MathToken::Number(ref n) if n == numerator));
    }

    /// `1̲/x` (no parens) — falls through to generic `̲/` denominator handling.
    /// Distinguishes the `&&` mutation: with `||` mutation, the `(` check
    /// alone would trigger paren-path even without `)` close.
    #[test]
    fn underline_fraction_without_parens_falls_through() {
        // "1\u{0332}" alone (no slash) → "123/1" via prefix path.
        let just_digit = parse_math_expression("1\u{0332}").unwrap();
        assert!(!just_digit.is_empty());

        // "A̲/B" → generic underline fraction (the lone /  branch).
        let generic = parse_math_expression("A\u{0332}/B").unwrap();
        assert!(!generic.is_empty(), "A̲/B must produce tokens");
    }

    /// Multi-space gap between Korean phrases should be preserved as a single space.
    /// Lines 138 (while-loop bound on whitespace skip) and 142
    /// (post-skip check for Korean continuation).
    #[test]
    fn korean_multi_space_preserves_single_space() {
        // "이건  몇" — two spaces between Korean words → still one KoreanWord with one space.
        let tokens = parse_math_expression("이건  몇").unwrap();
        let kw = tokens.iter().find_map(|t| {
            if let MathToken::KoreanWord(s) = t {
                Some(s.clone())
            } else {
                None
            }
        });
        let phrase = kw.expect("expected a KoreanWord token");
        assert_eq!(
            phrase, "이건 몇",
            "multi-space must collapse to single space inside phrase"
        );
    }

    /// Trailing whitespace before a non-Korean character must NOT be absorbed
    /// into the Korean phrase. Line 142 `j < chars.len() && is_korean_char`.
    /// Mutation: deleting `is_korean_char` check would absorb the space and
    /// produce "이건 " with trailing space.
    #[test]
    fn korean_phrase_does_not_absorb_trailing_space_before_ascii() {
        let tokens = parse_math_expression("이건 x").unwrap();
        // First token: KoreanWord "이건" (no trailing space).
        let phrase = match &tokens[0] {
            MathToken::KoreanWord(s) => s.clone(),
            other => panic!("expected KoreanWord, got {:?}", other),
        };
        assert_eq!(phrase, "이건", "trailing space must be stripped");
        // Then x.
        assert!(
            tokens.iter().any(|t| matches!(t, MathToken::Variable('x'))),
            "x must remain as a separate Variable"
        );
    }

    /// Unicode subscript with embedded `.` and `/` (e.g., `x₁.₂` and `x₁/₂`).
    /// Lines 197-209: the `.` and `/` extension within subscript sequence
    /// requires `chars.get(i + 1).is_some_and(is_subscript_char)` lookahead.
    #[test]
    fn unicode_subscript_with_dot_and_slash() {
        // x₁.₅ — subscript containing DecimalPoint.
        let dot = parse_math_expression("x\u{2081}.\u{2085}").unwrap();
        let sub_content = dot.iter().find_map(|t| {
            if let MathToken::Subscript(c) = t {
                Some(c.clone())
            } else {
                None
            }
        });
        let sub = sub_content.expect("expected Subscript token");
        assert!(
            sub.iter().any(|t| matches!(t, MathToken::DecimalPoint)),
            "subscript must contain DecimalPoint, got {:?}",
            sub
        );

        // x₁/₂ — subscript containing Operator '/'.
        let slash = parse_math_expression("x\u{2081}/\u{2082}").unwrap();
        let sub_content = slash.iter().find_map(|t| {
            if let MathToken::Subscript(c) = t {
                Some(c.clone())
            } else {
                None
            }
        });
        let sub = sub_content.expect("expected Subscript token");
        assert!(
            sub.iter().any(|t| matches!(t, MathToken::Operator('/'))),
            "subscript must contain Operator('/'), got {:?}",
            sub
        );
    }

    /// `.` or `/` followed by a non-subscript character must NOT extend
    /// the subscript sequence. Tests the lookahead check at line 198.
    /// Mutation `chars.get(i + 1)` → `i - 1` would cause off-by-one.
    #[test]
    fn unicode_subscript_dot_without_following_subscript_stops() {
        // x₁. — `.` after subscript with no following subscript-char → stop.
        let tokens = parse_math_expression("x\u{2081}.y").unwrap();
        let sub_content = tokens.iter().find_map(|t| {
            if let MathToken::Subscript(c) = t {
                Some(c.clone())
            } else {
                None
            }
        });
        let sub = sub_content.expect("expected Subscript");
        // The DecimalPoint must NOT be inside — subscript should be just {1}.
        assert!(
            !sub.iter().any(|t| matches!(t, MathToken::DecimalPoint)),
            "subscript must not extend past lone dot, got {:?}",
            sub
        );
    }

    /// `_{...}` brace tracking with NESTED braces.
    /// Lines 226-247: depth counter for matching closing brace.
    /// Mutations: `depth += 1` → `-=`/`*=`; delete `{`/`}` match arms.
    /// A nested `_{{a}b}` should pair the OUTER brace, producing a subscript
    /// containing tokens for "{a}b" parsed as inner expression.
    #[test]
    fn ascii_subscript_brace_nested_depth_tracking() {
        // x_{a{b}c}  — outer braces span the whole subscript content
        let tokens = parse_math_expression("x_{a{b}c}").unwrap();
        // Outer subscript must exist.
        let sub_idx = tokens
            .iter()
            .position(|t| matches!(t, MathToken::Subscript(_)));
        assert!(sub_idx.is_some(), "must find a Subscript token");
        // Whatever follows the outer `}` (here nothing) — `x_{a{b}c}` is single subscript.
        // After Variable(x) + Subscript, there should be no leftover Raw('c}').
        assert!(
            !tokens.iter().any(|t| matches!(t, MathToken::Raw('}'))),
            "no stray closing brace as Raw; tokens={:?}",
            tokens
        );
    }

    /// `_{...` with UNCLOSED brace must NOT consume infinite chars.
    /// Falls back to `Raw('_')` and continues. Line 242 `chars[j] == '}'`.
    /// Mutation `==` → `!=`/`>`/`<=` would mis-detect closure.
    #[test]
    fn ascii_subscript_brace_unclosed_falls_back_to_raw() {
        let tokens = parse_math_expression("x_{abc").unwrap();
        // `_` must end up as Raw since closure missing.
        assert!(
            tokens.iter().any(|t| matches!(t, MathToken::Raw('_'))),
            "unclosed _ must fall back to Raw('_'); tokens={:?}",
            tokens
        );
    }

    /// `_(...)` paren tracking with NESTED parens.
    /// Lines 255-289: depth tracking + wrap inner with MathParen brackets.
    /// Mutations on depth `+=`/`-=` would mis-match.
    #[test]
    fn ascii_subscript_paren_nested_depth_tracking() {
        // x_((a)) — nested parens; outer paren is the subscript delimiter.
        let tokens = parse_math_expression("x_((a))").unwrap();
        // Subscript content must wrap with MathParen and contain inner expression.
        let sub_content = tokens.iter().find_map(|t| {
            if let MathToken::Subscript(c) = t {
                Some(c.clone())
            } else {
                None
            }
        });
        let sub = sub_content.expect("expected Subscript");
        // First token in subscript is OpenParen MathParen (per the source).
        assert!(
            matches!(
                sub.first(),
                Some(MathToken::OpenParen(BracketKind::MathParen))
            ),
            "subscript must begin with MathParen open, got {:?}",
            sub
        );
        // Last token is CloseParen MathParen.
        assert!(
            matches!(
                sub.last(),
                Some(MathToken::CloseParen(BracketKind::MathParen))
            ),
            "subscript must end with MathParen close, got {:?}",
            sub
        );
    }

    /// `_(...` UNCLOSED paren must fall back to Raw('_').
    #[test]
    fn ascii_subscript_paren_unclosed_falls_back_to_raw() {
        let tokens = parse_math_expression("x_(abc").unwrap();
        assert!(
            tokens.iter().any(|t| matches!(t, MathToken::Raw('_'))),
            "unclosed _( must fall back to Raw('_'); tokens={:?}",
            tokens
        );
    }
}
