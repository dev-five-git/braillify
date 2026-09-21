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
    /// Rule consumed tokens under one of its declared metadata variants.
    ConsumedWithMeta {
        tokens: usize,
        meta: &'static crate::rules::RuleMeta,
    },
    /// Rule did not apply. Try next rule.
    Skip,
}

use crate::rules::trace::{RuleId, TraceSink};

/// Placeholder for a math rule that has not declared its source article yet.
/// Rules keeping this default are reported as unattributed rather than being
/// credited to an article nobody checked against the standard.
pub static UNDECLARED_MATH_RULE: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "?",
    subsection: None,
    name: "undeclared_math_rule",
    standard_ref: "",
    description: "",
};

/// Plugin interface for math token encoding rules.
pub trait MathTokenRule: Send + Sync {
    /// Rule name for debugging.
    fn name(&self) -> &'static str;

    /// The standard article this rule implements. Defaults to
    /// [`UNDECLARED_MATH_RULE`] until someone checks the article against the PDF.
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &UNDECLARED_MATH_RULE
    }

    /// Additional articles this rule can select while dispatching variants.
    fn variant_metas(&self) -> &'static [&'static crate::rules::RuleMeta] {
        &[]
    }

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

    /// Metadata of every registered math rule, in [`RuleId`] order.
    pub(crate) fn registry(&self) -> Vec<&'static crate::rules::RuleMeta> {
        self.rules
            .iter()
            .flat_map(|rule| {
                std::iter::once(rule.meta()).chain(rule.variant_metas().iter().copied())
            })
            .collect()
    }

    /// Encode a sequence of math tokens into braille bytes.
    pub fn encode_tokens(&self, tokens: &[MathToken], result: &mut Vec<u8>) -> Result<(), String> {
        self.encode_tokens_traced(tokens, result, None)
    }

    /// [`Self::encode_tokens`], recording which rule produced each stretch.
    ///
    /// The spans go to a collector rather than to a sink parameter because a math
    /// expression usually reaches here from a token rule, which emits the cells
    /// much later without knowing where they land. [`crate::rules::emit`] pairs
    /// the collected spans back to those cells.
    pub(crate) fn encode_tokens_traced(
        &self,
        tokens: &[MathToken],
        result: &mut Vec<u8>,
        mut trace: Option<&mut TraceSink<'_>>,
    ) -> Result<(), String> {
        let mut attempt = super::MathAttempt::new();
        let attempt_base = result.len();
        let logic_context = Self::has_logic_symbol(tokens);
        let mut state = MathEncodeState::with_context(logic_context, self.context);
        let mut i = 0usize;

        while i < tokens.len() {
            let mut handled = false;
            let mut registry_base = 0usize;
            for rule in &self.rules {
                let _ = rule.name();
                let variant_metas = rule.variant_metas();
                let registry_len = 1 + variant_metas.len();
                if !rule.matches(tokens, i, &state) {
                    registry_base += registry_len;
                    continue;
                }

                let start = result.len();
                let (consumed, local_offset) =
                    match rule.apply(tokens, i, result, &mut state, self)? {
                        MathTokenResult::Consumed(tokens) => (tokens, 0),
                        MathTokenResult::ConsumedWithMeta { tokens, meta } => {
                            let local_offset = std::iter::once(rule.meta())
                                .chain(variant_metas.iter().copied())
                                .position(|declared| std::ptr::eq(declared, meta))
                                .ok_or_else(|| {
                                    format!(
                                        "{} reported undeclared metadata: {} {}",
                                        rule.name(),
                                        meta.section,
                                        meta.name
                                    )
                                })?;
                            (tokens, local_offset)
                        }
                        MathTokenResult::Skip => {
                            registry_base += registry_len;
                            continue;
                        }
                    };
                let rule_id = RuleId::math(registry_base + local_offset);
                attempt.push(rule_id, start - attempt_base, result.len() - start);
                if let Some(sink) = trace.as_deref_mut() {
                    let token_index = sink.token_index() as usize;
                    sink.record_span(rule_id, token_index, start..result.len());
                }
                i += consumed;
                handled = true;
                break;
            }
            if !handled {
                return Err(format!(
                    "No rule matched token at index {}: {:?}",
                    i, tokens[i]
                ));
            }
        }
        attempt.finish(&result[attempt_base..]);
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
    use crate::rules::RuleMeta;
    use crate::rules::trace::Trace;

    static PRIMARY_META: RuleMeta = RuleMeta {
        section: "101",
        subsection: None,
        name: "test_primary",
        standard_ref: "test primary",
        description: "test primary",
    };
    static SECONDARY_META: RuleMeta = RuleMeta {
        section: "102",
        subsection: None,
        name: "test_secondary",
        standard_ref: "test secondary",
        description: "test secondary",
    };
    static FOLLOWING_META: RuleMeta = RuleMeta {
        section: "103",
        subsection: None,
        name: "test_following",
        standard_ref: "test following",
        description: "test following",
    };
    static FOREIGN_META: RuleMeta = RuleMeta {
        section: "104",
        subsection: None,
        name: "test_foreign",
        standard_ref: "test foreign",
        description: "test foreign",
    };
    static VARIANT_METAS: [&RuleMeta; 1] = [&SECONDARY_META];

    struct VariantRule;

    impl MathTokenRule for VariantRule {
        fn name(&self) -> &'static str {
            "VariantRule"
        }

        fn meta(&self) -> &'static RuleMeta {
            &PRIMARY_META
        }

        fn variant_metas(&self) -> &'static [&'static RuleMeta] {
            &VARIANT_METAS
        }

        fn priority(&self) -> u16 {
            10
        }

        fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
            matches!(tokens.get(index), Some(MathToken::Variable(_)))
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
            Ok(MathTokenResult::ConsumedWithMeta {
                tokens: 1,
                meta: &SECONDARY_META,
            })
        }
    }

    struct FollowingRule;

    impl MathTokenRule for FollowingRule {
        fn name(&self) -> &'static str {
            "FollowingRule"
        }

        fn meta(&self) -> &'static RuleMeta {
            &FOLLOWING_META
        }

        fn priority(&self) -> u16 {
            20
        }

        fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
            matches!(tokens.get(index), Some(MathToken::Number(_)))
        }

        fn apply(
            &self,
            _tokens: &[MathToken],
            _index: usize,
            result: &mut Vec<u8>,
            _state: &mut MathEncodeState,
            _engine: &MathTokenEngine,
        ) -> Result<MathTokenResult, String> {
            result.push(2);
            Ok(MathTokenResult::Consumed(1))
        }
    }

    struct UndeclaredMetaRule;

    impl MathTokenRule for UndeclaredMetaRule {
        fn name(&self) -> &'static str {
            "UndeclaredMetaRule"
        }

        fn meta(&self) -> &'static RuleMeta {
            &PRIMARY_META
        }

        fn matches(&self, tokens: &[MathToken], index: usize, _state: &MathEncodeState) -> bool {
            matches!(tokens.get(index), Some(MathToken::Variable(_)))
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
            Ok(MathTokenResult::ConsumedWithMeta {
                tokens: 1,
                meta: &FOREIGN_META,
            })
        }
    }

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
        assert!(r.variant_metas().is_empty());
    }

    #[test]
    fn registry_flattens_primary_then_variant_metadata_per_rule() {
        let mut engine = MathTokenEngine::with_context(MathContext::default());
        engine.register(Box::new(FollowingRule));
        engine.register(Box::new(VariantRule));
        engine.finalize();

        let registry = engine.registry();

        assert_eq!(registry.len(), 3);
        assert!(std::ptr::eq(registry[0], &PRIMARY_META));
        assert!(std::ptr::eq(registry[1], &SECONDARY_META));
        assert!(std::ptr::eq(registry[2], &FOLLOWING_META));
    }

    #[test]
    fn traced_variant_uses_its_secondary_registry_slot() {
        let mut engine = MathTokenEngine::with_context(MathContext::default());
        engine.register(Box::new(VariantRule));
        engine.finalize();
        let mut output = Vec::new();
        let mut trace = Trace::default();
        let mut sink = TraceSink::new(&mut trace);

        engine
            .encode_tokens_traced(&[MathToken::Variable('x')], &mut output, Some(&mut sink))
            .unwrap();

        assert_eq!(trace.events()[0].rule, RuleId::math(1));
    }

    #[test]
    fn traced_rule_after_variant_rule_uses_flattened_registry_base() {
        let mut engine = MathTokenEngine::with_context(MathContext::default());
        engine.register(Box::new(FollowingRule));
        engine.register(Box::new(VariantRule));
        engine.finalize();
        let mut output = Vec::new();
        let mut trace = Trace::default();
        let mut sink = TraceSink::new(&mut trace);

        engine
            .encode_tokens_traced(
                &[MathToken::Number("1".to_string())],
                &mut output,
                Some(&mut sink),
            )
            .unwrap();

        assert_eq!(trace.events()[0].rule, RuleId::math(2));
    }

    #[test]
    fn encoded_variant_rejects_metadata_the_rule_never_declared() {
        let mut engine = MathTokenEngine::with_context(MathContext::default());
        engine.register(Box::new(UndeclaredMetaRule));
        engine.finalize();
        let mut output = Vec::new();

        let error = engine
            .encode_tokens(&[MathToken::Variable('x')], &mut output)
            .unwrap_err();

        assert!(error.contains("UndeclaredMetaRule reported undeclared metadata"));
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
