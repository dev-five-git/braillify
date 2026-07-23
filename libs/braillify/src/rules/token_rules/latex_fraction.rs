use crate::fraction;
use crate::rules::token::{FractionToken, Token};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct LatexFractionRule;

impl TokenRule for LatexFractionRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::FractionDetection
    }

    fn priority(&self) -> u16 {
        100
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let word_text = word.text.as_ref();
        if !(word_text.starts_with('$') && word_text.ends_with('$')) {
            return Ok(TokenAction::Noop);
        }

        let parsed = fraction::parse_latex_fraction(word_text);
        if parsed.is_none() {
            return Ok(TokenAction::Noop);
        }
        let (whole, numerator, denominator) = parsed.unwrap();

        Ok(TokenAction::Replace(Token::Fraction(FractionToken {
            whole,
            numerator,
            denominator,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncoderState;
    use crate::rules::token::{WordMeta, WordToken};
    use std::borrow::Cow;

    /// latex_fraction:32-33 — `$...$` wrapped input that is NOT a valid \frac
    /// returns Noop via the parse_latex_fraction let-else.
    /// Direct apply call to bypass any earlier token rules that might handle it.
    #[rstest::rstest]
    #[case::single_variable("$x$")]
    #[case::empty_frac("$\\frac{}$")]
    #[case::digits("$123$")]
    #[case::alpha("$abc$")]
    fn dollar_wrapped_non_fraction_direct_apply_noop(#[case] text: &'static str) {
        let r = LatexFractionRule;
        let mut state = EncoderState::new(false);
        let chars: Vec<char> = text.chars().collect();
        let word = Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        });
        let tokens = vec![word];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(
            matches!(action, TokenAction::Noop),
            "expected Noop for {text}"
        );
    }

    /// latex_fraction:22-23 — apply called on Space token returns Noop.
    #[test]
    fn non_word_token_returns_noop() {
        use crate::rules::token::SpaceKind;
        let r = LatexFractionRule;
        let mut state = EncoderState::new(false);
        let tokens = vec![Token::Space(SpaceKind::Regular)];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Noop));
    }
}
