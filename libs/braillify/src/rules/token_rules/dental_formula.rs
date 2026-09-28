//! 과학 제26항 — 치식은 묶음 괄호 ( )을 사용하여 분수 표기법으로 적는다.
//!
//! 치식의 가운뎃점은 수학의 곱셈 점과 묵자 모양이 같아, 과학 문맥에서만 치식으로
//! 읽는다.

use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;

pub struct DentalFormulaRule;

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "26",
    subsection: None,
    name: "science_dental_formula",
    standard_ref: "2024 Korean Braille Standard, 과학 제26항",
    description: "치식",
};

/// 한쪽 턱의 이 수(`3∙1∙4∙2`)를 묶음 괄호로 싸고, 가운뎃점은 ⠐⠆ 으로 적는다.
fn jaw(counts: &str) -> Option<Vec<u8>> {
    let counts: Vec<&str> = counts.split(['∙', '·', '⋅']).collect();
    if counts.len() < 2
        || counts
            .iter()
            .any(|count| count.is_empty() || !count.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    let mut out = vec![decode_unicode('⠷')];
    for (at, count) in counts.iter().enumerate() {
        if at > 0 {
            out.extend([decode_unicode('⠐'), decode_unicode('⠆')]);
        }
        out.push(decode_unicode('⠼'));
        for digit in count.chars() {
            out.push(crate::number::encode_number(digit).ok()?);
        }
    }
    out.push(decode_unicode('⠾'));
    Some(out)
}

/// `\frac{위턱}{아래턱}` — 분수이므로 분모를 먼저 적는다.
fn dental_formula(latex: &str) -> Option<Vec<u8>> {
    let body = latex.trim().strip_prefix("\\frac{")?.strip_suffix('}')?;
    let (upper, lower) = body.split_once("}{")?;
    let mut out = jaw(lower)?;
    out.push(decode_unicode('⠌'));
    out.extend(jaw(upper)?);
    Some(out)
}

impl TokenRule for DentalFormulaRule {
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &META
    }

    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        1
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let cells = match tokens.get(index) {
            Some(Token::Word(word)) if state.science_context_active => word
                .text
                .strip_prefix('$')
                .and_then(|latex| latex.strip_suffix('$'))
                .and_then(dental_formula),
            _ => None,
        };
        Ok(cells.map_or(TokenAction::Noop, |cells| {
            TokenAction::Replace(Token::PreEncoded(cells))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(cells: &[u8]) -> String {
        cells
            .iter()
            .map(|cell| crate::unicode::encode_unicode(*cell))
            .collect()
    }

    #[rstest::rstest]
    #[case::permanent_teeth("\\frac{3∙1∙4∙2}{3∙1∙4∙3}", "⠷⠼⠉⠐⠆⠼⠁⠐⠆⠼⠙⠐⠆⠼⠉⠾⠌⠷⠼⠉⠐⠆⠼⠁⠐⠆⠼⠙⠐⠆⠼⠃⠾")]
    #[case::middle_dot_and_two_digits("\\frac{2·10}{2·1}", "⠷⠼⠃⠐⠆⠼⠁⠾⠌⠷⠼⠃⠐⠆⠼⠁⠚⠾")]
    fn writes_dental_formulas(#[case] latex: &str, #[case] expected: &str) {
        assert_eq!(
            braille(&dental_formula(latex).expect("is a dental formula")),
            expected
        );
    }

    #[rstest::rstest]
    #[case::single_count("\\frac{3}{4}")]
    #[case::letters("\\frac{a∙b}{c∙d}")]
    #[case::empty_count("\\frac{3∙}{3∙1}")]
    #[case::not_a_fraction("3∙1∙4∙2")]
    #[case::text_after_fraction("\\frac{3∙1}{3∙1}x")]
    fn leaves_other_latex_alone(#[case] latex: &str) {
        assert!(dental_formula(latex).is_none());
    }

    #[rstest::rstest]
    #[case::science_context(true, true)]
    #[case::other_context(false, false)]
    fn reads_dental_formulas_only_in_the_science_context(
        #[case] science: bool,
        #[case] replaced: bool,
    ) {
        let text = "$\\frac{3∙1}{3∙1}$";
        let chars: Vec<char> = text.chars().collect();
        let tokens = vec![Token::Word(crate::rules::token::WordToken {
            text: std::borrow::Cow::Borrowed(text),
            meta: crate::rules::token::WordMeta::from_chars(&chars),
            chars,
        })];
        let mut state = EncoderState::new(false);
        state.science_context_active = science;
        let action = DentalFormulaRule
            .apply(&tokens, 0, &mut state)
            .expect("applies");
        assert_eq!(matches!(action, TokenAction::Replace(_)), replaced);
    }

    #[test]
    fn skips_tokens_that_are_not_words() {
        let tokens = vec![Token::PreEncoded(vec![1])];
        let mut state = EncoderState::new(false);
        state.science_context_active = true;
        let action = DentalFormulaRule
            .apply(&tokens, 0, &mut state)
            .expect("applies");
        assert!(matches!(action, TokenAction::Noop));
    }
}
