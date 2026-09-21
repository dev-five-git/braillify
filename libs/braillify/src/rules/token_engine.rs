use super::RuleMeta;
use super::context::EncoderState;
use super::token::Token;
use super::token_rule::{TokenAction, TokenPhase, TokenRule};
use super::trace::{RuleId, TokenOrigins};

pub struct TokenRuleEngine {
    rules: Vec<Box<dyn TokenRule>>,
    sorted: bool,
}

impl TokenRuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            sorted: false,
        }
    }

    pub fn register(&mut self, rule: Box<dyn TokenRule>) {
        self.rules.push(rule);
        self.sorted = false;
    }

    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.rules.sort_by_key(|r| (r.phase() as u8, r.priority()));
            self.sorted = true;
        }
    }

    /// Metadata of every registered token rule, in [`RuleId`] order.
    pub(crate) fn registry(&mut self) -> Vec<&'static RuleMeta> {
        self.ensure_sorted();
        self.rules.iter().map(|rule| rule.meta()).collect()
    }

    /// Apply all rules in phase order. Handle token insertions/removals correctly.
    #[cfg(test)]
    pub fn apply_all<'a>(
        &mut self,
        tokens: &mut Vec<Token<'a>>,
        state: &mut EncoderState,
    ) -> Result<(), String> {
        self.apply_all_tracked(tokens, state, None)
    }

    /// [`Self::apply_all`], recording which rule produced each resulting token.
    pub fn apply_all_tracked<'a>(
        &mut self,
        tokens: &mut Vec<Token<'a>>,
        state: &mut EncoderState,
        mut origins: Option<&mut TokenOrigins>,
    ) -> Result<(), String> {
        self.ensure_sorted();

        for phase in [
            TokenPhase::Normalization,
            TokenPhase::FractionDetection,
            TokenPhase::WordShortcut,
            TokenPhase::ModeEntry,
            TokenPhase::UppercasePassage,
            TokenPhase::PostWord,
        ] {
            let mut i = 0usize;

            'outer: while i < tokens.len() {
                for (rule_index, rule) in self.rules.iter().enumerate() {
                    if rule.phase() != phase {
                        continue;
                    }

                    let action = rule.apply(tokens, i, state)?;
                    let is_noop_fallthrough = matches!(action, TokenAction::Noop)
                        && matches!(phase, TokenPhase::Normalization | TokenPhase::PostWord);
                    if is_noop_fallthrough {
                        continue;
                    }
                    let id = RuleId::token(rule_index);
                    match action {
                        TokenAction::Noop => {}
                        TokenAction::Replace(t) => {
                            tokens[i] = t;
                            if let Some(origins) = origins.as_deref_mut() {
                                origins.set(i, id);
                            }
                        }
                        #[cfg(test)]
                        TokenAction::InsertBefore(ts) => {
                            let count = ts.len();
                            if let Some(origins) = origins.as_deref_mut() {
                                origins.splice(i..i, id, count);
                            }
                            tokens.splice(i..i, ts);
                            i += count;
                        }
                        TokenAction::ReplaceMany(ts) => {
                            let count = ts.len();
                            if let Some(origins) = origins.as_deref_mut() {
                                origins.splice(i..i + 1, id, count);
                            }
                            tokens.splice(i..=i, ts);
                            if count == 0 {
                                // Array shrank by 1: the next original token now sits at `i`.
                                // Skip the outer `i += 1` so we re-process this slot
                                // (otherwise the shifted token would be silently skipped,
                                // letting e.g. ring-only word tokens leak into char encoding).
                                continue 'outer;
                            }
                            i += count - 1;
                        }
                        TokenAction::ReplaceRange(consume_count, ts) => {
                            // 현재 위치 i부터 consume_count개의 토큰을 통째로 ts로 교체한다.
                            let end = (i + consume_count).min(tokens.len());
                            let new_count = ts.len();
                            if let Some(origins) = origins.as_deref_mut() {
                                origins.splice(i..end, id, new_count);
                            }
                            tokens.splice(i..end, ts);
                            if new_count == 0 {
                                continue 'outer;
                            }
                            i += new_count - 1;
                        }
                        #[cfg(test)]
                        TokenAction::Remove => {
                            tokens.remove(i);
                            if let Some(origins) = origins.as_deref_mut() {
                                origins.remove(i);
                            }
                            continue;
                        }
                    }
                    let tracked = origins.as_deref().map_or(tokens.len(), TokenOrigins::len);
                    debug_assert_eq!(tracked, tokens.len(), "origin table lost lockstep");
                    break;
                }
                i += 1;
            }
        }

        Ok(())
    }
}

impl Default for TokenRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::rules::token::{SpaceKind, WordMeta, WordToken};

    struct ReplaceWordAt0;
    impl TokenRule for ReplaceWordAt0 {
        fn phase(&self) -> TokenPhase {
            TokenPhase::Normalization
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            if index == 0 {
                return Ok(TokenAction::Replace(Token::PreEncoded(vec![9])));
            }
            if matches!(tokens.get(index), Some(Token::Word(_))) {
                return Ok(TokenAction::Noop);
            }
            Ok(TokenAction::Noop)
        }
    }

    struct InsertSpaceBeforeSecond;
    impl TokenRule for InsertSpaceBeforeSecond {
        fn phase(&self) -> TokenPhase {
            TokenPhase::PostWord
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            if index == 1 && matches!(tokens.get(index), Some(Token::Word(_))) {
                return Ok(TokenAction::InsertBefore(vec![Token::Space(
                    SpaceKind::Regular,
                )]));
            }
            Ok(TokenAction::Noop)
        }
    }

    struct RemoveWordB;
    impl TokenRule for RemoveWordB {
        fn phase(&self) -> TokenPhase {
            TokenPhase::PostWord
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            if let Some(Token::Word(w)) = tokens.get(index)
                && w.text == "b"
            {
                return Ok(TokenAction::Remove);
            }
            Ok(TokenAction::Noop)
        }
    }

    struct ReplaceManyForB;
    impl TokenRule for ReplaceManyForB {
        fn phase(&self) -> TokenPhase {
            TokenPhase::PostWord
        }
        fn priority(&self) -> u16 {
            50
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            if let Some(Token::Word(w)) = tokens.get(index)
                && w.text == "b"
            {
                return Ok(TokenAction::ReplaceMany(vec![
                    Token::PreEncoded(vec![1]),
                    Token::PreEncoded(vec![2]),
                ]));
            }
            Ok(TokenAction::Noop)
        }
    }

    fn word_token(text: &'static str) -> Token<'static> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    #[test]
    fn token_engine_sorts_and_applies_by_phase_priority() {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(InsertSpaceBeforeSecond));
        engine.register(Box::new(ReplaceWordAt0));

        let mut tokens = vec![word_token("a"), word_token("b")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();

        assert!(matches!(tokens[0], Token::PreEncoded(ref b) if b == &vec![9]));
        assert!(matches!(tokens[1], Token::Space(SpaceKind::Regular)));
        assert!(matches!(tokens[2], Token::Word(_)));
    }

    #[test]
    fn ensure_sorted_orders_registered_rules_by_phase_then_priority() {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(InsertSpaceBeforeSecond));
        engine.register(Box::new(ReplaceWordAt0));

        engine.ensure_sorted();

        assert_eq!(engine.rules[0].phase(), TokenPhase::Normalization);
        assert_eq!(engine.rules[1].phase(), TokenPhase::PostWord);
    }

    #[test]
    fn token_engine_insert_replace_remove_index_handling() {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(ReplaceWordAt0));
        engine.register(Box::new(RemoveWordB));

        let mut tokens = vec![word_token("a"), word_token("b"), word_token("c")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::PreEncoded(_)));
        assert!(matches!(&tokens[1], Token::Word(w) if w.text == "c"));
    }

    #[test]
    fn token_engine_replace_many_updates_index_safely() {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(ReplaceManyForB));

        let mut tokens = vec![word_token("a"), word_token("b"), word_token("c")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0], Token::Word(w) if w.text == "a"));
        assert!(matches!(tokens[1], Token::PreEncoded(ref b) if b == &vec![1]));
        assert!(matches!(tokens[2], Token::PreEncoded(ref b) if b == &vec![2]));
        assert!(matches!(&tokens[3], Token::Word(w) if w.text == "c"));
    }

    /// token_engine:87 — `ReplaceRange(_, vec![])` triggers `continue 'outer`
    /// because new_count == 0. Use a dummy rule that returns ReplaceRange with
    /// empty replacement.
    struct ReplaceRangeEmpty;
    impl TokenRule for ReplaceRangeEmpty {
        fn phase(&self) -> TokenPhase {
            TokenPhase::Normalization
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            if let Some(Token::Word(w)) = tokens.get(index)
                && w.text == "b"
            {
                // Consume 1 token, replace with empty vec → new_count == 0.
                return Ok(TokenAction::ReplaceRange(1, Vec::new()));
            }
            Ok(TokenAction::Noop)
        }
    }

    #[test]
    fn token_engine_replace_range_empty_triggers_continue_outer() {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(ReplaceRangeEmpty));

        let mut tokens = vec![word_token("a"), word_token("b"), word_token("c")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();

        // "b" removed; "c" now at index 1.
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0], Token::Word(w) if w.text == "a"));
        assert!(matches!(&tokens[1], Token::Word(w) if w.text == "c"));
    }

    /// token_engine:55 — `TokenAction::Noop` arm coverage (re-attribution via
    /// direct dispatch test). A rule returning Noop in Normalization phase
    /// allows fall-through to next rule.
    #[test]
    fn token_engine_noop_normalization_continues_to_next_rule() {
        struct AlwaysNoop;
        impl TokenRule for AlwaysNoop {
            fn phase(&self) -> TokenPhase {
                TokenPhase::Normalization
            }
            fn priority(&self) -> u16 {
                10 // run before ReplaceWordAt0
            }
            fn apply<'a>(
                &self,
                _tokens: &[Token<'a>],
                _index: usize,
                _state: &mut EncoderState,
            ) -> Result<TokenAction<'a>, String> {
                Ok(TokenAction::Noop)
            }
        }
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(AlwaysNoop));
        engine.register(Box::new(ReplaceWordAt0));

        let mut tokens = vec![word_token("a")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();
        // AlwaysNoop returns Noop → fall through to ReplaceWordAt0 which fires at index 0.
        assert!(matches!(tokens[0], Token::PreEncoded(ref b) if b == &vec![9]));
    }

    #[test]
    fn token_engine_runtime_noop_normalization_continues_to_next_rule() {
        struct RuntimeNoop;
        impl TokenRule for RuntimeNoop {
            fn phase(&self) -> TokenPhase {
                std::hint::black_box(TokenPhase::Normalization)
            }
            fn priority(&self) -> u16 {
                std::hint::black_box(10)
            }
            fn apply<'a>(
                &self,
                _tokens: &[Token<'a>],
                _index: usize,
                _state: &mut EncoderState,
            ) -> Result<TokenAction<'a>, String> {
                Ok(std::hint::black_box(TokenAction::Noop))
            }
        }

        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(RuntimeNoop));
        engine.register(Box::new(ReplaceWordAt0));
        let mut tokens = vec![word_token("a")];
        let mut state = EncoderState::new(false);

        engine.apply_all(&mut tokens, &mut state).unwrap();

        assert!(matches!(tokens[0], Token::PreEncoded(ref b) if b == &vec![9]));
    }

    #[test]
    fn token_engine_noop_wordshortcut_stops_current_index_rules() {
        struct WordShortcutNoop;
        impl TokenRule for WordShortcutNoop {
            fn phase(&self) -> TokenPhase {
                TokenPhase::WordShortcut
            }
            fn priority(&self) -> u16 {
                10
            }
            fn apply<'a>(
                &self,
                _tokens: &[Token<'a>],
                _index: usize,
                _state: &mut EncoderState,
            ) -> Result<TokenAction<'a>, String> {
                Ok(TokenAction::Noop)
            }
        }

        struct WordShortcutReplace;
        impl TokenRule for WordShortcutReplace {
            fn phase(&self) -> TokenPhase {
                TokenPhase::WordShortcut
            }
            fn priority(&self) -> u16 {
                20
            }
            fn apply<'a>(
                &self,
                _tokens: &[Token<'a>],
                _index: usize,
                _state: &mut EncoderState,
            ) -> Result<TokenAction<'a>, String> {
                Ok(TokenAction::Replace(Token::PreEncoded(vec![7])))
            }
        }

        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(WordShortcutNoop));
        engine.register(Box::new(WordShortcutReplace));

        let mut tokens = vec![word_token("a")];
        let mut state = EncoderState::new(false);
        engine.apply_all(&mut tokens, &mut state).unwrap();

        assert!(matches!(&tokens[0], Token::Word(w) if w.text == "a"));
    }

    #[derive(Clone, Copy, Debug)]
    enum Rewrite {
        InsertBefore,
        ReplaceMany,
        ReplaceRange,
        Remove,
    }

    struct RewriteB(Rewrite);
    impl TokenRule for RewriteB {
        fn phase(&self) -> TokenPhase {
            TokenPhase::WordShortcut
        }
        fn apply<'a>(
            &self,
            tokens: &[Token<'a>],
            index: usize,
            _state: &mut EncoderState,
        ) -> Result<TokenAction<'a>, String> {
            let Some(Token::Word(word)) = tokens.get(index) else {
                return Ok(TokenAction::Noop);
            };
            if word.text != "b" {
                return Ok(TokenAction::Noop);
            }
            Ok(match self.0 {
                Rewrite::InsertBefore => {
                    TokenAction::InsertBefore(vec![Token::PreEncoded(vec![1])])
                }
                Rewrite::ReplaceMany => TokenAction::ReplaceMany(vec![
                    Token::PreEncoded(vec![1]),
                    Token::PreEncoded(vec![2]),
                ]),
                Rewrite::ReplaceRange => {
                    TokenAction::ReplaceRange(1, vec![Token::PreEncoded(vec![3])])
                }
                Rewrite::Remove => TokenAction::Remove,
            })
        }
    }

    /// The emitter names a token's producer by looking its position up in the
    /// origin table, so every rewrite shape must leave that table the same
    /// length as the stream and must claim exactly the slots it created. A
    /// shape that resized one but not the other would silently shift every
    /// later token's attribution onto the wrong rule.
    #[rstest::rstest]
    #[case::insert_before(Rewrite::InsertBefore)]
    #[case::replace_many(Rewrite::ReplaceMany)]
    #[case::replace_range(Rewrite::ReplaceRange)]
    #[case::remove(Rewrite::Remove)]
    fn origin_tracking_stays_in_lockstep_with_every_rewrite_shape(#[case] rewrite: Rewrite) {
        let mut engine = TokenRuleEngine::new();
        engine.register(Box::new(RewriteB(rewrite)));

        let mut tokens = vec![word_token("a"), word_token("b"), word_token("c")];
        let mut state = EncoderState::new(false);
        let mut origins = TokenOrigins::seeded(tokens.len());

        engine
            .apply_all_tracked(&mut tokens, &mut state, Some(&mut origins))
            .expect("the rewrite rule never fails");

        assert_eq!(origins.len(), tokens.len(), "{rewrite:?} resized one side");
        for (index, token) in tokens.iter().enumerate() {
            let expected = matches!(token, Token::PreEncoded(_)).then(|| RuleId::token(0));
            assert_eq!(origins.get(index), expected, "{rewrite:?} slot {index}");
        }
    }

    /// token_engine.rs lines 95-96 - `impl Default::default()` body.
    #[test]
    fn token_rule_engine_default_constructs_empty() {
        let _engine = TokenRuleEngine::default();
    }
}
