//! General LaTeX math expression handler.
//!
//! Strips `$...$` wrapping from LaTeX math expressions and encodes
//! the inner content using the math braille engine.
//!
//! Runs after LatexFractionRule to catch any `$...$` patterns
//! that aren't simple fractions.

use crate::rules::context::EncoderState;
use crate::rules::math;
use crate::rules::math::math_token_rule::MathContext;

fn math_context_from_state(state: &EncoderState) -> MathContext {
    MathContext {
        matrix_context_active: state.matrix_context_active,
        math_mode_active: state.math_mode_active,
    }
}

fn read_braced_content(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    if chars.peek() != Some(&'{') {
        return None;
    }
    chars.next();
    let mut content = String::new();
    let mut depth = 1usize;
    for ch in chars.by_ref() {
        match ch {
            '{' => {
                depth += 1;
                content.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
                content.push(ch);
            }
            _ => content.push(ch),
        }
    }
    Some(content)
}

mod matrix;
#[cfg(test)]
use matrix::subscript_digit_to_ascii;
use matrix::{encode_latex_matrix, find_latex_matrix};

pub(crate) fn encode_latex_math_bytes_with_context(
    latex_inner: &str,
    math_context: MathContext,
) -> Result<Vec<u8>, String> {
    if let Some(matrix) = find_latex_matrix(latex_inner) {
        return encode_latex_matrix(&matrix, math_context);
    }

    let math_text = strip_latex_to_math(latex_inner);

    // 제68항 compact notation: 단일 대문자 + 첨자(+/-/숫자) 패턴은
    // 한국어 모드 rule_68가 ⠴(영자) + ⠠(대문자) + letter + 첨자로 처리한다.
    // math engine은 첨자에 ⠴ 영자표시를 추가하지 않으므로 LaTeX 변환 결과가
    // 동일한 Unicode form일 때 일반 한국어 인코더로 위임한다.
    let chars: Vec<char> = math_text.chars().collect();
    if chars.len() >= 2
        && chars[0].is_ascii_uppercase()
        && chars[1..]
            .iter()
            .all(|c| matches!(*c, '⁺' | '⁻' | '₀'..='₉'))
    {
        return crate::encode(&math_text);
    }

    math::encoder::encode_math_expression_with_context(&math_text, math_context)
}

mod spacing;
pub(crate) use spacing::wrap_latex_math_tokens_with_inner;

mod grouping;

mod strip;
pub(crate) use strip::strip_latex_to_math;

/// Merges `$...$` token sequences into single Word tokens.
/// This runs at Normalization phase so that downstream fraction/math rules
/// see the complete LaTeX expression as one token.
mod merge_rule;
pub use merge_rule::LatexMergeRule;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_neq() {
        let result = strip_latex_to_math("y \\neq 0");
        assert!(result.contains('\u{2260}'));
        assert!(result.contains('y'));
        assert!(result.contains('0'));
    }

    #[test]
    fn test_strip_geq() {
        let result = strip_latex_to_math("x \\geq 5");
        assert!(result.contains('\u{2265}'));
    }

    #[test]
    fn test_strip_sin() {
        let result = strip_latex_to_math("\\sin x");
        assert!(result.contains("sin"));
        assert!(result.contains('x'));
    }

    #[test]
    fn test_strip_exponent() {
        let result = strip_latex_to_math("x^{2}");
        assert!(result.contains('\u{00B2}'));
    }

    #[test]
    fn test_strip_subscript() {
        let result = strip_latex_to_math("x_{2}");
        assert!(result.contains('\u{2082}'));
    }

    /// Exercise `subscript_digit_to_ascii` for each subscript codepoint
    /// (U+2080-U+2089 ↔ '0'-'9'), plus non-subscript inputs → None.
    #[rstest::rstest]
    #[case::sub_0('\u{2080}', Some('0'))]
    #[case::sub_1('\u{2081}', Some('1'))]
    #[case::sub_2('\u{2082}', Some('2'))]
    #[case::sub_3('\u{2083}', Some('3'))]
    #[case::sub_4('\u{2084}', Some('4'))]
    #[case::sub_5('\u{2085}', Some('5'))]
    #[case::sub_6('\u{2086}', Some('6'))]
    #[case::sub_7('\u{2087}', Some('7'))]
    #[case::sub_8('\u{2088}', Some('8'))]
    #[case::sub_9('\u{2089}', Some('9'))]
    #[case::ascii_letter_none('a', None)]
    #[case::ascii_digit_none('0', None)]
    fn subscript_digit_to_ascii_table(#[case] sub: char, #[case] expected: Option<char>) {
        assert_eq!(subscript_digit_to_ascii(sub), expected, "sub={sub:?}");
    }

    fn enc(input: &str) -> Vec<u8> {
        crate::encode(input).unwrap_or_default()
    }

    #[rstest::rstest]
    #[case::bare_matrix("$\\begin{matrix} 1 & 2 \\\\ 3 & 4 \\end{matrix}$", '⠦', '⠴')]
    #[case::parenthesized("$\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}$", '⠦', '⠴')]
    #[case::bracketed("$\\begin{bmatrix} 1 \\\\ 2 \\end{bmatrix}$", '⠦', '⠴')]
    #[case::braced("$\\begin{Bmatrix} x & y \\\\ z & w \\end{Bmatrix}$", '⠦', '⠴')]
    #[case::determinant("$\\begin{vmatrix} a & b \\\\ c & d \\end{vmatrix}$", '⠳', '⠳')]
    #[case::norm("$\\begin{Vmatrix} 1 & 0 \\\\ 0 & 1 \\end{Vmatrix}$", '⠳', '⠳')]
    #[case::generic_array("$\\begin{array}{cc} x & y \\\\ z & w \\end{array}$", '⠦', '⠴')]
    fn latex_matrix_environments(#[case] input: &str, #[case] open: char, #[case] close: char) {
        let output = crate::encode_to_unicode(input).unwrap();

        assert!(output.contains(open));
        assert!(output.contains(close));
        assert!(output.contains('⠜'));
    }

    #[test]
    fn latex_variation_table_array_uses_box_borders() {
        let output = crate::encode_to_unicode(
            "$\\begin{array}{|c|c|c|c|}\\hline x & y & z & w \\\\ \\hline\\end{array}$",
        )
        .unwrap();

        assert!(output.starts_with('⠖'));
        assert!(output.contains('⠓'));
    }

    /// Various LaTeX command stripping cases.
    #[test]
    fn latex_command_stripping_diverse() {
        let inputs: &[&str] = &[
            "$\\alpha$",
            "$\\beta$",
            "$\\gamma$",
            "$\\delta$",
            "$\\theta$",
            "$\\lambda$",
            "$\\mu$",
            "$\\nu$",
            "$\\pi$",
            "$\\sigma$",
            "$\\tau$",
            "$\\phi$",
            "$\\chi$",
            "$\\psi$",
            "$\\omega$",
            "$\\Alpha$",
            "$\\Gamma$",
            "$\\Delta$",
            "$\\Theta$",
            "$\\infty$",
            "$\\partial$",
            "$\\nabla$",
            "$\\forall$",
            "$\\exists$",
            "$\\emptyset$",
            "$\\in$",
            "$\\notin$",
            "$\\subset$",
            "$\\supset$",
            "$\\cup$",
            "$\\cap$",
            "$\\land$",
            "$\\lor$",
            "$\\neg$",
            "$\\Rightarrow$",
            "$\\Leftrightarrow$",
            "$\\rightarrow$",
            "$\\cdot$",
            "$\\times$",
            "$\\div$",
            "$\\le$",
            "$\\ge$",
            "$\\equiv$",
            "$\\approx$",
            "$\\sum$",
            "$\\prod$",
            "$\\int$",
            "$\\oint$",
            // Compound
            "$x \\to \\infty$",
            "$a \\equiv b \\pmod{n}$",
            "$\\sqrt{a^2 + b^2}$",
            "$\\sqrt[n]{x}$",
        ];
        for input in inputs {
            let _ = enc(input);
        }
    }

    /// LaTeX with combining marks and accents.
    #[test]
    fn latex_accents_and_marks() {
        let inputs: &[&str] = &[
            "$\\bar{x}$",
            "$\\overline{AB}$",
            "$\\underline{x}$",
            "$\\vec{v}$",
            "$\\overrightarrow{AB}$",
            "$\\hat{x}$",
            "$\\widehat{ABC}$",
            "$\\tilde{x}$",
            "$\\widetilde{xy}$",
            "$\\dot{x}$",
            "$\\ddot{x}$",
            "$\\acute{a}$",
            "$\\grave{a}$",
            "$\\check{x}$",
            "$\\breve{x}$",
        ];
        for input in inputs {
            let _ = enc(input);
        }
    }

    /// LaTeX fraction variants.
    #[test]
    fn latex_fractions_diverse() {
        let inputs: &[&str] = &[
            "$\\frac{1}{2}$",
            "$\\frac{a}{b}$",
            "$\\frac{a+b}{c-d}$",
            "$\\frac{x^2}{y^2}$",
            "$\\frac{\\sqrt{2}}{2}$",
            "$\\frac{\\sin x}{\\cos x}$",
            "$\\dfrac{1}{2}$",
            "$\\tfrac{1}{2}$",
            "$\\cfrac{1}{2}$",
            "$\\binom{n}{k}$",
            "$\\dbinom{n}{k}$",
        ];
        for input in inputs {
            let _ = enc(input);
        }
    }

    /// LaTeX paren / bracket variations.
    #[test]
    fn latex_brackets_diverse() {
        let inputs: &[&str] = &[
            "$(x)$",
            "$[x]$",
            "$\\{x\\}$",
            "$\\langle x \\rangle$",
            "$\\left(x\\right)$",
            "$\\left[x\\right]$",
            "$\\left\\{x\\right\\}$",
            "$\\left| x \\right|$",
            "$\\lfloor x \\rfloor$",
            "$\\lceil x \\rceil$",
        ];
        for input in inputs {
            let _ = enc(input);
        }
    }
}
