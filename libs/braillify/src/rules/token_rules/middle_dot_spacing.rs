use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct MiddleDotSpacingRule;

impl TokenRule for MiddleDotSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        126
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Space(_)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let Some(Token::Word(prev)) = index.checked_sub(1).and_then(|i| tokens.get(i)) else {
            return Ok(TokenAction::Noop);
        };
        let Some(Token::Word(next)) = tokens.get(index + 1) else {
            return Ok(TokenAction::Noop);
        };

        let prev_text = prev.text.as_ref();
        let next_text = next.text.as_ref();

        if (prev_text.ends_with('\'') || prev_text.ends_with('’'))
            && next_text
                .chars()
                .next()
                .is_some_and(crate::utils::is_korean_char)
            && next_text.starts_with("이다")
        {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        Ok(TokenAction::Noop)
    }
}
