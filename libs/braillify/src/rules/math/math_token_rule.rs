//! MathTokenRule trait — plugin interface for math token encoding.
//!
//! Each rule handles specific math token patterns. The MathTokenEngine
//! runs rules in priority order, dispatching to the first matching rule.

use super::parser::MathToken;

/// Encoder-owned context flags that affect math parsing/encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MathContext {
    /// PDF 제12항 붙임 1 — matrix-name mode for uppercase identifiers.
    pub matrix_context_active: bool,
    /// Explicit math mode keeps Hangul-containing parentheses as math parentheses.
    pub math_mode_active: bool,
}

/// Shared mutable state across math token encoding.
pub struct MathEncodeState {
    pub prev_was_number: bool,
    pub logic_context: bool,
    pub matrix_context_active: bool,
}

impl MathEncodeState {
    pub fn with_context(logic_context: bool, context: MathContext) -> Self {
        Self {
            prev_was_number: false,
            logic_context,
            matrix_context_active: context.matrix_context_active,
        }
    }
}

/// Result of applying a math token rule.
pub enum MathTokenResult {
    /// Rule consumed N tokens (advance index by N).
    Consumed(usize),
    /// Rule did not apply. Try next rule.
    Skip,
}

/// Plugin interface for math token encoding rules.
pub trait MathTokenRule: Send + Sync {
    /// Rule name for debugging.
    fn name(&self) -> &'static str;

    /// Priority (lower runs first). Default: 100.
    fn priority(&self) -> u16 {
        100
    }

    /// Fast check: does this rule handle the token at the given index?
    fn matches(&self, tokens: &[MathToken], index: usize, state: &MathEncodeState) -> bool;

    /// Encode the matched tokens. Returns how many tokens were consumed.
    fn apply(
        &self,
        tokens: &[MathToken],
        index: usize,
        result: &mut Vec<u8>,
        state: &mut MathEncodeState,
        engine: &MathTokenEngine,
    ) -> Result<MathTokenResult, String>;
}

/// Engine that dispatches math tokens to registered rules.
pub struct MathTokenEngine {
    rules: Vec<Box<dyn MathTokenRule>>,
    context: MathContext,
}

impl MathTokenEngine {
    pub fn with_context(context: MathContext) -> Self {
        Self {
            rules: Vec::new(),
            context,
        }
    }

    pub fn register(&mut self, rule: Box<dyn MathTokenRule>) {
        self.rules.push(rule);
    }

    /// Sort rules by priority (call once after all rules registered).
    pub fn finalize(&mut self) {
        self.rules.sort_by_key(|r| r.priority());
    }

    /// Encode a sequence of math tokens into braille bytes.
    pub fn encode_tokens(&self, tokens: &[MathToken], result: &mut Vec<u8>) -> Result<(), String> {
        let logic_context = Self::has_logic_symbol(tokens);
        let mut state = MathEncodeState::with_context(logic_context, self.context);
        let mut i = 0usize;

        while i < tokens.len() {
            let mut handled = false;
            for rule in &self.rules {
                let _ = rule.name();
                if rule.matches(tokens, i, &state)
                    && let MathTokenResult::Consumed(n) =
                        rule.apply(tokens, i, result, &mut state, self)?
                {
                    i += n;
                    handled = true;
                    break;
                }
            }
            if !handled {
                return Err(format!(
                    "No rule matched token at index {}: {:?}",
                    i, tokens[i]
                ));
            }
        }
        Ok(())
    }

    fn has_logic_symbol(tokens: &[MathToken]) -> bool {
        tokens.iter().any(|token| {
            matches!(
                token,
                MathToken::MathSymbol(
                    '\u{00AC}'
                        | '\u{21D2}'
                        | '\u{2194}'
                        | '\u{21D4}'
                        | '\u{21C4}'
                        | '\u{2227}'
                        | '\u{2228}'
                        | '\u{22BB}'
                        | '\u{2193}'
                        | '\u{2191}'
                        | '\u{2200}'
                        | '\u{2203}'
                        | '\u{2204}'
                )
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MathTokenRule::priority()` default implementation returns 100.
    /// Exercised by a dummy rule that doesn't override `priority()`.
    /// Drives the default-impl lines 48-50.
    #[test]
    fn priority_default_impl_returns_100() {
        struct DummyRule;
        impl MathTokenRule for DummyRule {
            fn name(&self) -> &'static str {
                "DummyRule"
            }
            fn matches(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                _state: &MathEncodeState,
            ) -> bool {
                false
            }
            fn apply(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                _result: &mut Vec<u8>,
                _state: &mut MathEncodeState,
                _engine: &MathTokenEngine,
            ) -> Result<MathTokenResult, String> {
                Ok(MathTokenResult::Skip)
            }
        }
        let r = DummyRule;
        assert_eq!(r.priority(), 100);
    }

    /// math_token_rule.rs line 97 - `MathTokenEngine.encode_tokens` returns Err
    /// when no registered rule matches the input token.
    #[test]
    fn encode_tokens_errors_when_no_rule_matches() {
        // Empty engine: no rules registered, any token will be unhandled.
        let engine = MathTokenEngine::with_context(MathContext::default());
        let mut result = Vec::new();
        let toks = vec![MathToken::Variable('x')];
        let err = engine.encode_tokens(&toks, &mut result).unwrap_err();
        assert!(
            err.contains("No rule matched token at index 0"),
            "got: {err}"
        );
    }

    #[test]
    fn encode_tokens_continues_after_matching_rule_skips() {
        struct SkippingRule;
        impl MathTokenRule for SkippingRule {
            fn name(&self) -> &'static str {
                "SkippingRule"
            }
            fn priority(&self) -> u16 {
                10
            }
            fn matches(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                _state: &MathEncodeState,
            ) -> bool {
                true
            }
            fn apply(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                _result: &mut Vec<u8>,
                _state: &mut MathEncodeState,
                _engine: &MathTokenEngine,
            ) -> Result<MathTokenResult, String> {
                Ok(MathTokenResult::Skip)
            }
        }

        struct ConsumingRule;
        impl MathTokenRule for ConsumingRule {
            fn name(&self) -> &'static str {
                "ConsumingRule"
            }
            fn priority(&self) -> u16 {
                20
            }
            fn matches(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                _state: &MathEncodeState,
            ) -> bool {
                true
            }
            fn apply(
                &self,
                _tokens: &[MathToken],
                _index: usize,
                result: &mut Vec<u8>,
                _state: &mut MathEncodeState,
                _engine: &MathTokenEngine,
            ) -> Result<MathTokenResult, String> {
                result.push(1);
                Ok(MathTokenResult::Consumed(1))
            }
        }

        let mut engine = MathTokenEngine::with_context(MathContext::default());
        engine.register(Box::new(ConsumingRule));
        engine.register(Box::new(SkippingRule));
        engine.finalize();

        let mut result = Vec::new();
        engine
            .encode_tokens(&[MathToken::Variable('x')], &mut result)
            .unwrap();

        assert_eq!(result, vec![1]);
    }
}
