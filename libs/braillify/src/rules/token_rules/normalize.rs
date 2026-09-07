use std::borrow::Cow;

use crate::rules::token::{Token, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

/// Normalize the ASCII glyph substitutes `<` and `>` to the single angle
/// brackets defined by Korean Braille Standard Article 49 when they form a
/// balanced prose enclosure in a Korean document.
///
/// U+003C/U+003E still retain their mathematical comparison meaning in an
/// actual relation (`x<y>z`).  News and publishing text commonly substitutes
/// the ASCII glyphs for U+3008/U+3009 around titles, including enclosures that
/// span print spaces, so the decision has to be document-wide rather than
/// character-local.
pub struct NormalizeAsciiAngleBrackets;

#[derive(Clone, Copy)]
struct FlatChar {
    token_index: usize,
    char_index: usize,
    ch: char,
}

fn flattened_chars(tokens: &[Token<'_>]) -> Vec<FlatChar> {
    let mut flattened = Vec::new();
    for (token_index, token) in tokens.iter().enumerate() {
        match token {
            Token::Word(word) => flattened.extend(word.chars.iter().copied().enumerate().map(
                |(char_index, ch)| FlatChar {
                    token_index,
                    char_index,
                    ch,
                },
            )),
            Token::Space(_) => flattened.push(FlatChar {
                token_index,
                char_index: usize::MAX,
                ch: ' ',
            }),
            Token::Fraction(_) | Token::Mode(_) | Token::PreEncoded(_) => {}
        }
    }
    flattened
}

fn is_simple_relation_operand(chars: &[FlatChar]) -> bool {
    let visible = chars
        .iter()
        .filter(|item| !item.ch.is_whitespace())
        .map(|item| item.ch)
        .collect::<Vec<_>>();
    if visible.is_empty() || !visible.iter().all(char::is_ascii_alphanumeric) {
        return false;
    }

    visible.iter().all(char::is_ascii_digit)
        || (visible.iter().all(char::is_ascii_alphabetic) && visible.len() <= 2)
}

fn is_chained_comparison(flattened: &[FlatChar], open: usize, close: usize) -> bool {
    let left = flattened[..open]
        .iter()
        .rev()
        .find(|item| !item.ch.is_whitespace())
        .map(|item| item.ch);
    let right = flattened[close + 1..]
        .iter()
        .find(|item| !item.ch.is_whitespace())
        .map(|item| item.ch);

    left.is_some_and(|ch| ch.is_ascii_alphanumeric())
        && right.is_some_and(|ch| ch.is_ascii_alphanumeric())
        && is_simple_relation_operand(&flattened[open + 1..close])
}

fn ascii_angle_replacements(tokens: &[Token<'_>]) -> Vec<(usize, usize, char)> {
    if !tokens
        .iter()
        .any(|token| matches!(token, Token::Word(word) if word.meta.has_korean))
    {
        return Vec::new();
    }

    let flattened = flattened_chars(tokens);
    let mut openings = Vec::new();
    let mut replacements = Vec::new();

    for (index, item) in flattened.iter().enumerate() {
        match item.ch {
            '<' | '〈' => openings.push(index),
            '>' | '〉' => {
                let Some(open) = openings.pop() else {
                    continue;
                };
                if open + 1 == index || is_chained_comparison(&flattened, open, index) {
                    continue;
                }

                let opening = flattened[open];
                if opening.ch == '<' {
                    replacements.push((opening.token_index, opening.char_index, '〈'));
                }
                if item.ch == '>' {
                    replacements.push((item.token_index, item.char_index, '〉'));
                }
            }
            _ => {}
        }
    }

    replacements
}

impl TokenRule for NormalizeAsciiAngleBrackets {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        90
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
        if !word.chars.iter().any(|ch| matches!(ch, '<' | '>')) {
            return Ok(TokenAction::Noop);
        }

        let replacements = ascii_angle_replacements(tokens);
        let mut chars = word.chars.clone();
        let mut changed = false;
        for (_, char_index, replacement) in replacements
            .into_iter()
            .filter(|(token_index, _, _)| *token_index == index)
        {
            if let Some(ch) = chars.get_mut(char_index) {
                *ch = replacement;
                changed = true;
            }
        }
        if !changed {
            return Ok(TokenAction::Noop);
        }

        let normalized = chars.iter().collect::<String>();
        Ok(TokenAction::Replace(Token::Word(WordToken {
            text: Cow::Owned(normalized),
            meta: crate::rules::token::WordMeta::from_chars(&chars),
            chars,
        })))
    }
}

pub struct NormalizeEllipsis;

impl TokenRule for NormalizeEllipsis {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
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

        let has_literal_quote_context = word.text.contains('‘') || word.text.contains('’');
        let normalized = if has_literal_quote_context {
            word.text.to_string()
        } else {
            word.text.replace("......", "...").replace("……", "…")
        };
        if normalized == word.text {
            return Ok(TokenAction::Noop);
        }

        let chars: Vec<char> = normalized.chars().collect();
        Ok(TokenAction::Replace(Token::Word(WordToken {
            text: Cow::Owned(normalized),
            chars: chars.clone(),
            meta: crate::rules::token::WordMeta::from_chars(&chars),
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::{DocumentIR, SpaceKind};
    use crate::rules::token_engine::TokenRuleEngine;

    fn normalize(input: &str) -> String {
        let mut ir = DocumentIR::parse(input, false);
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(NormalizeAsciiAngleBrackets));
        engine
            .apply_all(&mut ir.tokens, &mut ir.state)
            .expect("normalization must succeed");

        ir.tokens
            .iter()
            .map(|token| match token {
                Token::Word(word) => word.chars.iter().collect::<String>(),
                Token::Space(SpaceKind::Regular) => " ".to_string(),
                Token::Fraction(_) | Token::Mode(_) | Token::PreEncoded(_) => String::new(),
            })
            .collect()
    }

    #[rstest::rstest]
    #[case::title_at_start("<제목>을 읽다", "〈제목〉을 읽다")]
    #[case::attached_title("책<긴 제목>이다", "책〈긴 제목〉이다")]
    #[case::score_tiebreak("경기 7-6<7-3> 2-6", "경기 7-6〈7-3〉 2-6")]
    #[case::roman_title("영화 <Das Boot>이다", "영화 〈Das Boot〉이다")]
    #[case::comparison_chain("식 x<y>z이다", "식 x<y>z이다")]
    #[case::single_comparison("식 x<y이다", "식 x<y이다")]
    fn ascii_angles_are_disambiguated_by_balanced_prose_syntax(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(normalize(input), expected);
    }

    #[test]
    fn flattening_ignores_non_textual_tokens() {
        let tokens = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            Token::Mode(crate::rules::token::ModeEvent::EnterEnglish),
            Token::PreEncoded(vec![1]),
        ];

        assert!(flattened_chars(&tokens).is_empty());
    }
}
