use std::borrow::Cow;

use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::word_shortcut;

pub struct WordShortcutRule;

impl TokenRule for WordShortcutRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::WordShortcut
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

        // 제18항 [다만] withholds the abbreviation only when another *letter*
        // precedes it (오그리고); an opening quote or bracket is punctuation,
        // so `“그러나` still abbreviates.
        let text = word.text.as_ref();
        let prefix_len = text
            .chars()
            .take_while(|ch| is_opening_punctuation(*ch))
            .map(char::len_utf8)
            .sum::<usize>();
        let (prefix, body) = text.split_at(prefix_len);

        let Some((_, code, rest)) = word_shortcut::split_word_shortcut(body) else {
            return Ok(TokenAction::Noop);
        };

        if prefix.is_empty() && rest.is_empty() {
            return Ok(TokenAction::Replace(Token::PreEncoded(code.to_vec())));
        }

        let mut replacement = Vec::with_capacity(3);
        if !prefix.is_empty() {
            replacement.push(owned_word(prefix.to_string()));
        }
        replacement.push(Token::PreEncoded(code.to_vec()));
        if !rest.is_empty() {
            replacement.push(owned_word(rest));
        }
        Ok(TokenAction::ReplaceMany(replacement))
    }
}

fn is_opening_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '“' | '‘' | '"' | '\'' | '(' | '[' | '{' | '「' | '『' | '〈' | '《' | '〔'
    )
}

fn owned_word(text: String) -> Token<'static> {
    let chars: Vec<char> = text.chars().collect();
    Token::Word(WordToken {
        text: Cow::Owned(text),
        meta: WordMeta::from_chars(&chars),
        chars,
    })
}
