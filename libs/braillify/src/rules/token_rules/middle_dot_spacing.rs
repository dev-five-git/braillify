use std::borrow::Cow;

use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct MiddleDotSpacingRule;

fn previous_word<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<&'b WordToken<'a>> {
    tokens[..index]
        .iter()
        .rev()
        .find_map(|token| match token {
            Token::Mode(_) => None,
            Token::Word(word) => Some(Some(word)),
            _ => Some(None),
        })
        .flatten()
}

fn next_word<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<(usize, &'b WordToken<'a>)> {
    tokens
        .iter()
        .enumerate()
        .skip(index + 1)
        .find_map(|(token_index, token)| match token {
            Token::Mode(_) | Token::Space(_) => None,
            Token::Word(word) => Some(Some((token_index, word))),
            _ => Some(None),
        })
        .flatten()
}

/// Rules 51 and 59 attach a Korean colon/semicolon to the item on its left.
/// A spaced colon between two Roman/number items remains UEB print spacing,
/// so require a Korean item on either side of the punctuation boundary.
fn space_precedes_korean_colon_or_semicolon(
    tokens: &[Token<'_>],
    index: usize,
    previous: &WordToken<'_>,
) -> bool {
    let Some((punctuation_index, punctuation)) = next_word(tokens, index) else {
        return false;
    };
    if !punctuation
        .chars
        .first()
        .is_some_and(|symbol| matches!(symbol, ':' | ';'))
        || punctuation.chars.len() != 1
    {
        return false;
    }

    previous
        .chars
        .iter()
        .rev()
        .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
        .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        || next_word(tokens, punctuation_index).is_some_and(|(_, word)| {
            word.chars
                .iter()
                .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        })
}

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
        // Merge a one-sided editorial space at the token boundary so the
        // middle dot is encoded with the same character context as canonical
        // `정치·경제`, not merely emitted as an adjacent second word.
        if let Some(Token::Word(left)) = tokens.get(index)
            && matches!(tokens.get(index + 1), Some(Token::Space(_)))
            && let Some(Token::Word(right)) = tokens.get(index + 2)
            && (left.chars.last() == Some(&'·') || right.chars.first() == Some(&'·'))
        {
            let text = format!("{}{}", left.text, right.text);
            let chars = text.chars().collect::<Vec<_>>();
            return Ok(TokenAction::ReplaceRange(
                3,
                vec![Token::Word(WordToken {
                    text: Cow::Owned(text),
                    chars: chars.clone(),
                    meta: WordMeta::from_chars(&chars),
                })],
            ));
        }

        let Some(Token::Space(_)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let Some(prev) = previous_word(tokens, index) else {
            return Ok(TokenAction::Noop);
        };
        let Some((_, next)) = next_word(tokens, index) else {
            return Ok(TokenAction::Noop);
        };

        // Korean rule 50: the middle dot is attached on both sides. Its print
        // source sometimes contains editorial spaces, but the braille spacing
        // is still canonicalized by the rule.
        if prev.chars.last() == Some(&'·') || next.chars.first() == Some(&'·') {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        if space_precedes_korean_colon_or_semicolon(tokens, index, prev) {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Korean rules 50, 51, and 59 determine braille spacing even when the
    /// print source contains editorial spaces around the punctuation.
    #[rstest::rstest]
    #[case::middle_dot_both_sides("정치 · 경제", "정치·경제")]
    #[case::middle_dot_left("정치 ·경제", "정치·경제")]
    #[case::middle_dot_right("정치· 경제", "정치·경제")]
    #[case::korean_colon("제목 : 내용", "제목: 내용")]
    #[case::roman_to_korean_colon("WHO : 세계", "WHO: 세계")]
    #[case::korean_semicolon("채소 ; 과일", "채소; 과일")]
    fn canonical_korean_punctuation_spacing(#[case] spaced: &str, #[case] canonical: &str) {
        assert_eq!(crate::encode(spaced), crate::encode(canonical));
    }

    /// Rule 32 leaves print spacing inside a Roman section to UEB. A Korean
    /// prefix earlier in the token does not turn `FAPAS : Food` into a Korean
    /// colon boundary because the immediately preceding item is Roman.
    #[test]
    fn attached_roman_item_preserves_space_before_ueb_colon() {
        assert_ne!(
            crate::encode("설명(FAPAS : Food)"),
            crate::encode("설명(FAPAS: Food)")
        );
    }

    #[test]
    fn colon_spacing_probe_returns_false_when_no_punctuation_word_follows() {
        let mut ir = crate::rules::token::DocumentIR::parse("한국", false);
        ir.tokens
            .push(Token::Space(crate::rules::token::SpaceKind::Regular));
        let Token::Word(previous) = &ir.tokens[0] else {
            unreachable!("fixture begins with a word")
        };

        assert!(!space_precedes_korean_colon_or_semicolon(
            &ir.tokens, 1, previous
        ));
    }
}
