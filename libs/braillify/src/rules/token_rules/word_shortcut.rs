use std::borrow::Cow;

use crate::rules::RuleMeta;
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::word_shortcut;

pub struct WordShortcutRule;

static META: RuleMeta = RuleMeta {
    section: "18",
    subsection: None,
    name: "token_word_shortcut",
    standard_ref: "2024 Korean Braille Standard, 제18항",
    description: "Apply Korean word abbreviations while preserving punctuation context",
};

impl TokenRule for WordShortcutRule {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

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
        // precedes it (오그리고); a quote, bracket or period is punctuation, so
        // `“그러나`, `유일한(그리고` and `컸다.그러면서도` still abbreviate.
        let text = word.text.as_ref();
        let Some((prefix, code, rest)) = text
            .char_indices()
            .filter(|(at, _)| {
                text[..*at]
                    .chars()
                    .next_back()
                    .is_none_or(|previous| !previous.is_alphanumeric())
            })
            .find_map(|(at, _)| {
                word_shortcut::split_word_shortcut(&text[at..])
                    .map(|(_, code, rest)| (&text[..at], code, rest))
            })
        else {
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

fn owned_word(text: String) -> Token<'static> {
    let chars: Vec<char> = text.chars().collect();
    Token::Word(WordToken {
        text: Cow::Owned(text),
        meta: WordMeta::from_chars(&chars),
        chars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncoderState;

    fn noop_for(tokens: &[Token<'_>]) -> bool {
        let mut state = EncoderState::new(false);
        matches!(
            WordShortcutRule.apply(tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        )
    }

    /// 제18항: a word carrying no abbreviation, and a token that is not a word
    /// at all, both leave the stream untouched.
    #[test]
    fn a_word_without_an_abbreviation_is_left_alone() {
        assert!(noop_for(&[owned_word("사과".to_string())]));
    }

    #[test]
    fn a_token_that_is_not_a_word_is_left_alone() {
        assert!(noop_for(&[Token::PreEncoded(vec![1])]));
    }

    #[test]
    fn an_index_past_the_end_is_left_alone() {
        assert!(noop_for(&[]));
    }

    /// 제18항 [다만]: 다른 글자가 앞에 붙으면 약어를 쓰지 않는다.
    #[rstest::rstest]
    #[case::syllable_before("쭈그리고")]
    #[case::digit_before("3그리고")]
    fn a_letter_touching_the_abbreviation_withholds_it(#[case] text: &str) {
        assert!(noop_for(&[owned_word(text.to_string())]));
    }

    /// 제18항: the abbreviation is written, and an opening quote before it or a
    /// particle after it is kept as its own token.
    #[rstest::rstest]
    #[case::bare("그리고")]
    #[case::quoted("\u{201C}그리고")]
    #[case::with_tail("그리고도")]
    #[case::after_a_bracketed_word("유일한(그리고")]
    #[case::after_a_period("컸다.그러면서도")]
    fn an_abbreviated_word_is_replaced(#[case] text: &str) {
        let mut state = EncoderState::new(false);
        let tokens = [owned_word(text.to_string())];
        assert!(!matches!(
            WordShortcutRule.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }
}
