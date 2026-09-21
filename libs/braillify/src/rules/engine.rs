//! `RuleEngine` — the plugin host.
//!
//! Collects rules, sorts by phase+priority, applies them in order.
//! Supports enabling/disabling rules by section ID.

use std::collections::HashSet;

use super::RuleMeta;
use super::context::RuleContext;
use super::trace::{RuleId, RuleOutcome, TraceEvent, TraceSink};
use super::traits::{BrailleRule, Phase, RuleResult};

/// The rule engine — holds all registered rules and applies them.
///
/// # Usage
/// ```ignore
/// let mut engine = RuleEngine::new();
/// engine.register(Box::new(Rule11VowelYe));
/// engine.register(Box::new(Rule12VowelAe));
///
/// // Disable a specific rule:
/// engine.disable("12");
///
/// // Apply to a character context:
/// engine.apply(&mut ctx)?;
/// ```
pub struct RuleEngine {
    rules: Vec<Box<dyn BrailleRule>>,
    /// Rules disabled by section ID (e.g., "11", "14")
    disabled: HashSet<String>,
    sorted: bool,
}

impl RuleEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            disabled: HashSet::new(),
            sorted: false,
        }
    }

    /// Register a rule plugin.
    pub fn register(&mut self, rule: Box<dyn BrailleRule>) {
        self.rules.push(rule);
        self.sorted = false;
    }

    /// Metadata of every registered rule, in [`RuleId`] order.
    pub(crate) fn registry(&mut self) -> Vec<&'static RuleMeta> {
        self.ensure_sorted();
        self.rules.iter().map(|rule| rule.meta()).collect()
    }

    /// Disable a rule by its section ID (e.g., "11" to disable 제11항).
    #[cfg(test)]
    pub fn disable(&mut self, section: &str) {
        self.disabled.insert(section.to_string());
    }

    /// Enable a previously disabled rule.
    #[cfg(test)]
    pub fn enable(&mut self, section: &str) {
        self.disabled.remove(section);
    }

    /// Check if a rule is currently enabled.
    pub fn is_enabled(&self, section: &str) -> bool {
        !self.disabled.contains(section)
    }

    /// Get count of registered rules.
    #[cfg(test)]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get count of currently enabled rules.
    #[cfg(test)]
    pub fn enabled_count(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| self.is_enabled(r.meta().section))
            .count()
    }

    /// List all registered rule metadata (for introspection/debugging).
    #[cfg(test)]
    pub fn list_rules(&self) -> Vec<&'static RuleMeta> {
        self.rules.iter().map(|r| r.meta()).collect()
    }

    /// Sort rules by (phase, priority). Called automatically before first apply.
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.rules.sort_by_key(|r| (r.phase() as u8, r.priority()));
            self.sorted = true;
        }
    }

    /// Apply all enabled rules to the current character context.
    ///
    /// Rules run in phase order, then by priority within a phase.
    /// If a rule returns `Consumed`, subsequent rules are skipped.
    /// If a rule returns `Continue`, the next rule runs.
    /// If a rule returns `Skip`, it didn't apply — next rule runs.
    #[cfg(test)]
    pub fn apply(&mut self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        self.ensure_sorted();

        for rule in &self.rules {
            let meta = rule.meta();
            if !self.is_enabled(meta.section) {
                continue;
            }
            if !rule.matches(ctx) {
                continue;
            }
            match rule.apply(ctx)? {
                RuleResult::Consumed => return Ok(RuleResult::Consumed),
                RuleResult::Continue => {}
                RuleResult::Skip => {}
            }
        }
        Ok(RuleResult::Skip)
    }

    /// Apply only rules in a specific phase.
    pub fn apply_phase(
        &mut self,
        phase: Phase,
        ctx: &mut RuleContext,
        mut trace: Option<TraceSink<'_>>,
    ) -> Result<RuleResult, String> {
        self.ensure_sorted();

        // `rule.apply` is an opaque dyn call that mutates `ctx`, so the reads
        // `TraceSpan::open` performs cannot be sunk past it. Gating them on a
        // loop-invariant flag keeps the untraced path free of that work.
        let tracing = trace.is_some();

        for (index, rule) in self.rules.iter().enumerate() {
            if rule.phase() != phase {
                continue;
            }
            let meta = rule.meta();
            if self.is_enabled(meta.section) {
                if !rule.matches(ctx) {
                    continue;
                }
                let span = tracing.then(|| TraceSpan::open(ctx));
                let outcome = rule.apply(ctx)?;
                if let (Some(span), Some(sink)) = (span, trace.as_mut()) {
                    span.close(RuleId::korean(index), outcome, ctx, sink);
                }
                match outcome {
                    RuleResult::Consumed => return Ok(RuleResult::Consumed),
                    RuleResult::Continue => {}
                    RuleResult::Skip => {}
                }
            }
        }
        Ok(RuleResult::Skip)
    }
}

struct TraceSpan {
    output_start: u32,
    char_start: u32,
    skip_before: u32,
}

impl TraceSpan {
    fn open(ctx: &RuleContext) -> Self {
        Self {
            output_start: ctx.result.len() as u32,
            char_start: ctx.index as u32,
            skip_before: *ctx.skip_count as u32,
        }
    }

    /// Records by what a rule PRODUCED, not by what it returned.
    ///
    /// `Skip` normally means the rule declined and explains nothing, so it is
    /// dropped — but a few rules emit a mode indicator and still return `Skip`
    /// to let the next rule encode the character. Those cells are in the output
    /// and something has to account for them.
    ///
    /// A rule that reported per-article spans (syllable composition) is recorded
    /// as those articles instead of as itself, so a syllable names 제3항 for its
    /// 받침 rather than one composite entry for the whole character.
    fn close(
        self,
        rule: RuleId,
        result: RuleResult,
        ctx: &mut RuleContext,
        sink: &mut TraceSink<'_>,
    ) {
        let produced_cells = ctx.result.len() as u32 > self.output_start;
        let outcome = match result {
            RuleResult::Consumed => RuleOutcome::Consumed,
            RuleResult::Continue => RuleOutcome::Continued,
            RuleResult::Skip if produced_cells => RuleOutcome::Continued,
            RuleResult::Skip => return,
        };
        let consumed_extra = (*ctx.skip_count as u32).saturating_sub(self.skip_before);
        let word_chars = self.char_start..self.char_start + 1 + consumed_extra;
        let end = ctx.result.len() as u32;

        let mut recorded_any = false;
        if let Some(spans) = ctx.state.jamo_spans.as_deref_mut() {
            for (jamo, span) in spans.drain() {
                recorded_any = true;
                sink.trace.push(TraceEvent {
                    rule: RuleId::jamo(jamo),
                    outcome,
                    token_index: sink.token_index,
                    word_chars: word_chars.clone(),
                    output: self.output_start + span.start..self.output_start + span.end,
                });
            }
        }
        if !recorded_any {
            sink.trace.push(TraceEvent {
                rule,
                outcome,
                token_index: sink.token_index,
                word_chars,
                output: self.output_start..end,
            });
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleMeta;
    use crate::rules::context::EncoderState;

    static TEST_META: RuleMeta = RuleMeta {
        section: "test",
        subsection: None,
        name: "test_rule",
        standard_ref: "test",
        description: "test rule that emits byte 99",
    };

    struct TestRule;
    impl BrailleRule for TestRule {
        fn meta(&self) -> &'static RuleMeta {
            &TEST_META
        }
        fn phase(&self) -> Phase {
            Phase::CoreEncoding
        }
        fn matches(&self, _ctx: &RuleContext) -> bool {
            true
        }
        fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
            ctx.emit(99);
            Ok(RuleResult::Consumed)
        }
    }

    #[test]
    fn engine_registers_and_applies() {
        let mut engine = RuleEngine::new();
        engine.register(Box::new(TestRule));
        assert_eq!(engine.rule_count(), 1);

        let word_chars = vec!['가'];
        let char_type = crate::char_struct::CharType::new('가').unwrap();
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let mut skip = 0usize;
        let empty: Vec<&str> = vec![];
        let mut ctx = RuleContext {
            word_chars: &word_chars,
            index: 0,
            char_type: &char_type,
            prev_word: "",
            remaining_words: &empty,
            has_korean_char: true,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            roman_section_continues_from_previous_word: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut result,
        };

        let outcome = engine.apply(&mut ctx).unwrap();
        assert_eq!(outcome, RuleResult::Consumed);
        assert_eq!(result, vec![99]);
    }

    #[test]
    fn engine_disables_rules() {
        let mut engine = RuleEngine::new();
        engine.register(Box::new(TestRule));
        engine.disable("test");

        assert_eq!(engine.enabled_count(), 0);
        assert!(!engine.is_enabled("test"));

        engine.enable("test");
        assert_eq!(engine.enabled_count(), 1);
    }

    #[test]
    fn engine_sorts_by_phase_and_priority() {
        static META_A: RuleMeta = RuleMeta {
            section: "a",
            subsection: None,
            name: "post",
            standard_ref: "",
            description: "",
        };
        static META_B: RuleMeta = RuleMeta {
            section: "b",
            subsection: None,
            name: "core",
            standard_ref: "",
            description: "",
        };

        struct PostRule;
        impl BrailleRule for PostRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_A
            }
            fn phase(&self) -> Phase {
                Phase::PostProcessing
            }
            fn matches(&self, _: &RuleContext) -> bool {
                false
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Skip)
            }
        }
        struct CoreRule;
        impl BrailleRule for CoreRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_B
            }
            fn phase(&self) -> Phase {
                Phase::CoreEncoding
            }
            fn matches(&self, _: &RuleContext) -> bool {
                false
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Skip)
            }
        }

        let mut engine = RuleEngine::new();
        engine.register(Box::new(PostRule));
        engine.register(Box::new(CoreRule));
        engine.ensure_sorted();

        let metas = engine.list_rules();
        assert_eq!(metas[0].name, "core"); // CoreEncoding before PostProcessing
        assert_eq!(metas[1].name, "post");
    }

    /// `RuleEngine::default()` returns an engine with no rules.
    /// Drives lines 151-152.
    #[test]
    fn engine_default_constructs_empty() {
        let engine = RuleEngine::default();
        assert_eq!(engine.list_rules().len(), 0);
    }

    /// `apply` skips disabled rules (drives line 107 `continue`).
    /// `apply` skips non-matching rules (drives line 110 `continue`).
    /// `apply` skips when no rule consumes → final `Ok(Skip)` (drives line 118).
    /// `apply` runs through a Continue → next rule → Skip path (drives line 115).
    #[test]
    fn engine_apply_skip_disabled_nonmatching_and_final_skip() {
        use crate::char_struct::CharType;
        use crate::rules::context::EncoderState;

        static META_DIS: RuleMeta = RuleMeta {
            section: "dis",
            subsection: None,
            name: "disabled",
            standard_ref: "",
            description: "",
        };
        static META_NOMATCH: RuleMeta = RuleMeta {
            section: "nomatch",
            subsection: None,
            name: "no-match",
            standard_ref: "",
            description: "",
        };
        static META_CONT: RuleMeta = RuleMeta {
            section: "cont",
            subsection: None,
            name: "continuer",
            standard_ref: "",
            description: "",
        };
        static META_SKIP: RuleMeta = RuleMeta {
            section: "skip",
            subsection: None,
            name: "skipper",
            standard_ref: "",
            description: "",
        };

        // Rule that matches everything but always returns Continue.
        struct ContinueRule;
        impl BrailleRule for ContinueRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_CONT
            }
            fn phase(&self) -> Phase {
                Phase::CoreEncoding
            }
            fn matches(&self, _: &RuleContext) -> bool {
                true
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Continue)
            }
        }

        // Rule that matches but returns Skip.
        struct SkipRule;
        impl BrailleRule for SkipRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_SKIP
            }
            fn phase(&self) -> Phase {
                Phase::CoreEncoding
            }
            fn matches(&self, _: &RuleContext) -> bool {
                true
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Skip)
            }
        }

        // Rule that never matches (drives the `!rule.matches(ctx) => continue` arm).
        struct NoMatchRule;
        impl BrailleRule for NoMatchRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_NOMATCH
            }
            fn phase(&self) -> Phase {
                Phase::CoreEncoding
            }
            fn matches(&self, _: &RuleContext) -> bool {
                false
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Consumed)
            }
        }

        // Disabled rule (drives the `!self.is_enabled => continue` arm).
        struct DisabledRule;
        impl BrailleRule for DisabledRule {
            fn meta(&self) -> &'static RuleMeta {
                &META_DIS
            }
            fn phase(&self) -> Phase {
                Phase::CoreEncoding
            }
            fn matches(&self, _: &RuleContext) -> bool {
                true
            }
            fn apply(&self, _: &mut RuleContext) -> Result<RuleResult, String> {
                Ok(RuleResult::Consumed)
            }
        }

        let mut engine = RuleEngine::new();
        engine.register(Box::new(DisabledRule));
        engine.register(Box::new(NoMatchRule));
        engine.register(Box::new(ContinueRule));
        engine.register(Box::new(SkipRule));
        engine.disable("dis");

        let word_chars = vec!['x'];
        let char_type = CharType::English('x');
        let empty: [&str; 0] = [];
        let mut skip = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word_chars,
            index: 0,
            char_type: &char_type,
            prev_word: "",
            remaining_words: &empty,
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            roman_section_continues_from_previous_word: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut result,
        };
        let outcome = engine.apply(&mut ctx).expect("ok");
        // Disabled and non-matching skipped �� Continue �� Skip �� no Consumed.
        // Final return value is Skip.
        assert_eq!(outcome, RuleResult::Skip);
    }

    /// engine.rs line 124 - `apply_phase` skip arm for disabled rules.
    #[test]
    fn engine_apply_phase_skips_disabled_rules() {
        use crate::char_struct::CharType;

        let mut engine = RuleEngine::new();
        engine.register(Box::new(TestRule));
        engine.disable("test");

        let word_chars = vec!['x'];
        let char_type = CharType::English('x');
        let empty: [&str; 0] = [];
        let mut skip = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word_chars,
            index: 0,
            char_type: &char_type,
            prev_word: "",
            remaining_words: &empty,
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            roman_section_continues_from_previous_word: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut result,
        };
        // TestRule.phase() = CoreEncoding; with disabled section "test", apply_phase
        // hits the `if !self.is_enabled(meta.section) { continue; }` arm.
        let outcome = engine
            .apply_phase(Phase::CoreEncoding, &mut ctx, None)
            .unwrap();
        assert_eq!(outcome, RuleResult::Skip);
    }
}
