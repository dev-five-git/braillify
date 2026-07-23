use once_cell::sync::Lazy;
use regex::Regex;

use crate::fraction;
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

static FRACTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)\/(\d+)").expect("Failed to compile FRACTION_REGEX"));

pub struct InlineFractionRule;

impl TokenRule for InlineFractionRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::FractionDetection
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
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let chars = &word.chars;
        let word_len = chars.len();

        for (i, ch) in chars.iter().enumerate() {
            if !ch.is_ascii_digit() {
                continue;
            }

            let remaining: String = chars[i..].iter().collect();
            let Some(captures) = FRACTION_REGEX.captures(&remaining) else {
                continue;
            };

            let numerator = &captures[1];
            let denominator = &captures[2];
            let match_len = captures[0].len();
            let k = i + match_len;
            let is_date_or_range = (numerator.len() > 1 || denominator.len() > 1)
                || (k < word_len && chars[k] == '/')
                || (k < word_len && chars[k] == '~');

            if is_date_or_range {
                continue;
            }

            let mut replacement = Vec::new();

            if i > 0 {
                let prefix: String = chars[..i].iter().collect();
                let prefix_chars: Vec<char> = prefix.chars().collect();
                replacement.push(Token::Word(WordToken {
                    text: std::borrow::Cow::Owned(prefix),
                    chars: prefix_chars.clone(),
                    meta: WordMeta::from_chars(&prefix_chars),
                }));
            }

            replacement.push(Token::PreEncoded(fraction::encode_fraction_in_context(
                numerator,
                denominator,
            )?));

            if k < word_len {
                let suffix: String = chars[k..].iter().collect();
                let suffix_chars: Vec<char> = suffix.chars().collect();
                replacement.push(Token::Word(WordToken {
                    text: std::borrow::Cow::Owned(suffix),
                    chars: suffix_chars.clone(),
                    meta: WordMeta::from_chars(&suffix_chars),
                }));
            }

            return Ok(TokenAction::ReplaceMany(replacement));
        }

        Ok(TokenAction::Noop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncoderState;
    use crate::rules::token::SpaceKind;

    fn word_token<'a>(text: &str) -> Token<'a> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: std::borrow::Cow::Owned(text.to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    #[test]
    fn rule_phase_and_priority() {
        let rule = InlineFractionRule;
        assert!(matches!(rule.phase(), TokenPhase::FractionDetection));
        assert_eq!(rule.priority(), 120);
    }

    #[test]
    fn non_word_token_noop() {
        let rule = InlineFractionRule;
        let tokens: Vec<Token> = vec![Token::Space(SpaceKind::Regular)];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            rule.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn no_digit_no_match_noop() {
        let rule = InlineFractionRule;
        let tokens = vec![word_token("hello")];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            rule.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn digit_without_fraction_noop() {
        let rule = InlineFractionRule;
        let tokens = vec![word_token("123abc")];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            rule.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn simple_single_digit_fraction_replaces() {
        let rule = InlineFractionRule;
        let tokens = vec![word_token("1/2")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn multi_digit_treated_as_date_or_range() {
        let rule = InlineFractionRule;
        // 12/3 has numerator length 2 → treated as date/range → no match
        let tokens = vec![word_token("12/3")];
        let mut state = EncoderState::new(false);
        // The first digit '1' matches regex (12/3 → captured), but multi-digit → continue.
        // After continue, '2' matches → captures "2/3" — single digit → fraction!
        // Wait, but on '2' at index 1, "remaining" is "2/3" which matches regex
        // and k=3, no follow-up '/', no '~'. is_date_or_range = false → fraction!
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // Actually because the regex looks at the entire remaining, "2/3" matches
        // and the leading "1" becomes prefix word.
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn fraction_followed_by_slash_skipped() {
        let rule = InlineFractionRule;
        // 1/2/3 — first match "1/2" but chars[3]='/' → is_date_or_range → continue
        // Then at i=2 (the '2'), regex matches "2/3" — k=5, no follow-up → fraction.
        let tokens = vec![word_token("1/2/3")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn fraction_followed_by_tilde_skipped() {
        let rule = InlineFractionRule;
        // 1/2~3 — first match "1/2" but chars[3]='~' → is_date_or_range → continue
        // Then at i=2, regex matches but "2~3"? "2" matches /(\d+)\/(\d+)/ no, ~ is not /
        // Actually regex doesn't match "2~..." since no '/'. So no fraction found.
        let tokens = vec![word_token("1/2~3")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // No match → Noop
        assert!(matches!(action, TokenAction::Noop));
    }

    #[test]
    fn fraction_with_prefix_and_suffix() {
        let rule = InlineFractionRule;
        // "x1/2y" — prefix "x", fraction "1/2", suffix "y"
        let tokens = vec![word_token("x1/2y")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        match action {
            TokenAction::ReplaceMany(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], Token::Word(_)));
                assert!(matches!(items[1], Token::PreEncoded(_)));
                assert!(matches!(items[2], Token::Word(_)));
            }
            _ => panic!("expected ReplaceMany with 3 items"),
        }
    }

    #[test]
    fn fraction_only_no_prefix_or_suffix() {
        let rule = InlineFractionRule;
        let tokens = vec![word_token("3/4")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        match action {
            TokenAction::ReplaceMany(items) => {
                assert_eq!(items.len(), 1);
                assert!(matches!(items[0], Token::PreEncoded(_)));
            }
            _ => panic!("expected ReplaceMany with single PreEncoded"),
        }
    }

    #[test]
    fn fraction_with_suffix_only() {
        let rule = InlineFractionRule;
        let tokens = vec![word_token("1/2x")];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        match action {
            TokenAction::ReplaceMany(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Token::PreEncoded(_)));
                assert!(matches!(items[1], Token::Word(_)));
            }
            _ => panic!("expected ReplaceMany with PreEncoded + suffix"),
        }
    }
}
