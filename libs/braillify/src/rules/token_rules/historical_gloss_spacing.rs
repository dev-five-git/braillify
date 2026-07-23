use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct HistoricalGlossSpacingRule;

impl TokenRule for HistoricalGlossSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        120
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

        if prev.text.as_ref() == "〔" || next.text.as_ref() == "〕" {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        Ok(TokenAction::Noop)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::rules::context::EncoderState;
    use crate::rules::token::{SpaceKind, Token, WordMeta, WordToken};

    use super::HistoricalGlossSpacingRule;
    use crate::rules::token_rule::TokenRule;

    fn word(text: &'static str) -> Token<'static> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    #[test]
    fn removes_space_after_left_tortoise_shell() {
        let tokens = vec![word("〔"), Token::Space(SpaceKind::Regular), word("刀")];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 1, &mut state)
            .unwrap();

        assert!(
            matches!(action, crate::rules::token_rule::TokenAction::ReplaceMany(ref ts) if ts.is_empty())
        );
    }

    #[test]
    fn removes_space_before_right_tortoise_shell() {
        let tokens = vec![word("刀"), Token::Space(SpaceKind::Regular), word("〕")];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 1, &mut state)
            .unwrap();

        assert!(
            matches!(action, crate::rules::token_rule::TokenAction::ReplaceMany(ref ts) if ts.is_empty())
        );
    }

    #[test]
    fn noop_when_not_space() {
        let tokens = vec![word("刀")];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 0, &mut state)
            .unwrap();
        assert!(matches!(
            action,
            crate::rules::token_rule::TokenAction::Noop
        ));
    }

    #[test]
    fn noop_when_no_prev_word() {
        let tokens = vec![Token::Space(SpaceKind::Regular), word("刀")];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 0, &mut state)
            .unwrap();
        assert!(matches!(
            action,
            crate::rules::token_rule::TokenAction::Noop
        ));
    }

    /// Covers line 24 (early-return when next is not a Word).
    #[test]
    fn noop_when_next_is_not_word() {
        let tokens = vec![
            word("a"),
            Token::Space(SpaceKind::Regular),
            Token::Space(SpaceKind::Regular),
        ];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 1, &mut state)
            .unwrap();
        assert!(matches!(
            action,
            crate::rules::token_rule::TokenAction::Noop
        ));
    }

    #[test]
    fn noop_when_neither_tortoise_shell() {
        let tokens = vec![word("a"), Token::Space(SpaceKind::Regular), word("b")];
        let mut state = EncoderState::new(false);
        let action = HistoricalGlossSpacingRule
            .apply(&tokens, 1, &mut state)
            .unwrap();
        assert!(matches!(
            action,
            crate::rules::token_rule::TokenAction::Noop
        ));
    }

    #[test]
    fn rule_phase_priority() {
        use crate::rules::token_rule::TokenPhase;
        assert!(matches!(
            HistoricalGlossSpacingRule.phase(),
            TokenPhase::Normalization
        ));
        assert_eq!(HistoricalGlossSpacingRule.priority(), 120);
    }
}
