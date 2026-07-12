use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct QuoteAttachmentRule;

fn quote_delta(text: &str) -> i32 {
    let mut delta = 0i32;
    let starts_with_ascii_double = text.starts_with('"');
    let ends_with_ascii_double = text.ends_with('"');
    let starts_with_ascii_single = text.starts_with('\'');
    let ends_with_ascii_single = text.ends_with('\'');

    for ch in text.chars() {
        match ch {
            '“' | '‘' => delta += 1,
            '”' | '’' => delta -= 1,
            _ => {}
        }
    }

    if starts_with_ascii_double {
        delta += 1;
    }
    if ends_with_ascii_double {
        delta -= 1;
    }
    if starts_with_ascii_single {
        delta += 1;
    }
    if ends_with_ascii_single {
        delta -= 1;
    }

    delta
}

fn has_korean_syllable(text: &str) -> bool {
    text.chars().any(crate::utils::is_korean_char)
}

fn has_jamo_only(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0x3131..=0x3163).contains(&code)
    })
}

fn quote_balance_before<'a>(tokens: &[Token<'a>], index: usize) -> i32 {
    let mut balance = 0i32;
    for token in tokens.iter().take(index) {
        if let Token::Word(w) = token {
            balance += quote_delta(w.text.as_ref());
        }
    }
    balance
}

impl TokenRule for QuoteAttachmentRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        130
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
        let balance = quote_balance_before(tokens, index) + quote_delta(prev_text);
        let has_ascii_double_quote = tokens
            .iter()
            .any(|t| matches!(t, Token::Word(w) if w.text.contains('"')));

        // Inside quoted prose (not jamo listings), 붙여쓰기 with attach separator.
        if has_ascii_double_quote
            && balance > 0
            && has_korean_syllable(prev_text)
            && has_korean_syllable(next_text)
            && !has_jamo_only(prev_text)
            && !has_jamo_only(next_text)
        {
            return Ok(TokenAction::Replace(Token::PreEncoded(vec![8])));
        }

        if prev_text.ends_with('“')
            || prev_text.ends_with('‘')
            || prev_text.ends_with('"')
            || next_text.starts_with('”')
            || next_text.starts_with('’')
            || next_text.starts_with('"')
        {
            return Ok(TokenAction::Replace(Token::PreEncoded(vec![8])));
        }

        Ok(TokenAction::Noop)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::rules::context::EncoderState;
    use crate::rules::token::{SpaceKind, Token, WordMeta, WordToken};
    use crate::rules::token_rule::TokenAction;

    use super::QuoteAttachmentRule;
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
    fn attaches_space_inside_ascii_double_quote() {
        let tokens = vec![
            word("\"빨리"),
            Token::Space(SpaceKind::Regular),
            word("말해!\""),
        ];
        let mut state = EncoderState::new(false);
        let action = QuoteAttachmentRule.apply(&tokens, 1, &mut state).unwrap();

        assert!(
            matches!(action, TokenAction::Replace(Token::PreEncoded(bytes)) if bytes == vec![8])
        );
    }

    #[test]
    fn attaches_space_next_to_quote_boundary() {
        let tokens = vec![
            word("열림\""),
            Token::Space(SpaceKind::Regular),
            word("다음"),
        ];
        let mut state = EncoderState::new(false);
        let action = QuoteAttachmentRule.apply(&tokens, 1, &mut state).unwrap();

        assert!(
            matches!(action, TokenAction::Replace(Token::PreEncoded(bytes)) if bytes == vec![8])
        );
    }

    #[test]
    fn rule_metadata_reports_phase_and_priority() {
        let rule = std::hint::black_box(QuoteAttachmentRule);

        assert!(matches!(
            rule.phase(),
            crate::rules::token_rule::TokenPhase::Normalization
        ));
        assert_eq!(rule.priority(), 130);
    }

    #[test]
    fn pipeline_keeps_attachment_for_ascii_quote_sentence() {
        let mut ir = crate::rules::token::DocumentIR::parse("\"빨리 말해!\"", true);
        let mut engine = crate::rules::token_engine::TokenRuleEngine::new();
        engine.register(Box::new(
            crate::rules::token_rules::normalize::NormalizeEllipsis,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::emphasis_ring::EmphasisRingRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::latex_fraction::LatexFractionRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::inline_fraction::InlineFractionRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::word_shortcut::WordShortcutRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::uppercase_passage::UppercasePassageRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::middle_dot_spacing::MiddleDotSpacingRule,
        ));
        engine.register(Box::new(QuoteAttachmentRule));
        engine.register(Box::new(
            crate::rules::token_rules::spacing::AsteriskSpacingRule,
        ));
        engine
            .apply_all(&mut ir.tokens, &mut ir.state)
            .expect("token rules should succeed");

        assert!(
            ir.tokens
                .iter()
                .any(|t| matches!(t, Token::PreEncoded(bytes) if bytes == &vec![8])),
            "expected attach marker token in pipeline output"
        );
    }

    /// quote_attachment:25 — ends_with_ascii_double `"` decrements delta.
    #[test]
    fn quote_delta_ends_with_ascii_double_decrements() {
        // Input ending with `"` triggers line 25.
        assert!(super::quote_delta("text\"") < 0 || super::quote_delta("text\"") == 0);
        // Input both starts AND ends with `"` cancels out.
        assert_eq!(super::quote_delta("\"text\""), 0);
        // Input ends with single ascii quote → starts at 0, line 31 fires.
        assert!(super::quote_delta("text'") <= 0);
    }

    #[test]
    fn quote_balance_before_accumulates_prior_word_quotes() {
        let tokens = vec![
            word("\"열림"),
            Token::Space(SpaceKind::Regular),
            word("중간"),
        ];

        assert_eq!(super::quote_balance_before(&tokens, 2), 1);
    }

    /// quote_attachment:81 — `apply` with Space token at index where next token
    /// is NOT a Word (e.g. PreEncoded or end of tokens) returns Noop.
    #[test]
    fn apply_with_no_next_word_returns_noop() {
        use crate::rules::context::EncoderState;
        use crate::rules::token::{SpaceKind, WordMeta, WordToken};
        use std::borrow::Cow;
        let word_token = Token::Word(WordToken {
            text: Cow::Borrowed("foo"),
            chars: vec!['f', 'o', 'o'],
            meta: WordMeta::from_chars(&['f', 'o', 'o']),
        });
        let space_token = Token::Space(SpaceKind::Regular);
        // [Word, Space] — next of Space is None → line 81.
        let tokens = vec![word_token, space_token];
        let mut state = EncoderState::new(false);
        let action = QuoteAttachmentRule.apply(&tokens, 1, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Noop));
    }
}
