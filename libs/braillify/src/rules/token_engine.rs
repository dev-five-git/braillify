use super::context::EncoderState;
use super::token::Token;
use super::token_rule::{TokenAction, TokenPhase, TokenRule};

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

    /// Apply all rules in phase order. Handle token insertions/removals correctly.
    pub fn apply_all<'a>(
        &mut self,
        tokens: &mut Vec<Token<'a>>,
        state: &mut EncoderState,
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
                for rule in &self.rules {
                    if rule.phase() != phase {
                        continue;
                    }

                    let action = rule.apply(tokens, i, state)?;
                    let is_noop_fallthrough = matches!(action, TokenAction::Noop)
                        && matches!(phase, TokenPhase::Normalization | TokenPhase::PostWord);
                    if is_noop_fallthrough {
                        continue;
                    }
                    match action {
                        TokenAction::Noop => {}
                        TokenAction::Replace(t) => {
                            tokens[i] = t;
                        }
                        #[cfg(test)]
                        TokenAction::InsertBefore(ts) => {
                            let count = ts.len();
                            tokens.splice(i..i, ts);
                            i += count;
                        }
                        TokenAction::ReplaceMany(ts) => {
                            let count = ts.len();
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
                            tokens.splice(i..end, ts);
                            if new_count == 0 {
                                continue 'outer;
                            }
                            i += new_count - 1;
                        }
                        #[cfg(test)]
                        TokenAction::Remove => {
                            tokens.remove(i);
                            continue;
                        }
                    }
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

    /// token_engine.rs lines 95-96 - `impl Default::default()` body.
    #[test]
    fn token_rule_engine_default_constructs_empty() {
        let _engine = TokenRuleEngine::default();
    }
}
