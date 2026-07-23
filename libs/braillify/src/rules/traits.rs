//! The core `BrailleRule` trait — the plugin interface.
//!
//! Every rule implements this trait. The `RuleEngine` calls `matches()` then `apply()`
//! for each registered rule in priority order.

use super::RuleMeta;
use super::context::RuleContext;

/// Result of applying a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleResult {
    /// This rule fully handled the current character. Stop running further rules.
    Consumed,
    /// This rule added supplementary output (e.g., separator). Continue to next rules.
    Continue,
    /// This rule did not apply to the current character.
    Skip,
}

/// Execution phase — rules run in phase order, then by priority within a phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Normalization before encoding (e.g., ellipsis collapse)
    Preprocessing = 0,
    /// Word-level shortcuts (그래서, 그러나, etc.)
    WordShortcut = 1,
    /// Core character encoding (Korean syllables, English letters, digits, symbols)
    CoreEncoding = 3,
    /// Inter-character rules (vowel separators, etc.)
    InterCharacter = 4,
    /// Post-processing (spacing, asterisk handling)
    #[cfg(test)]
    PostProcessing = 5,
}

/// The plugin interface for braille rules.
///
/// Each rule is a self-contained unit:
/// - Has stable metadata (name, standard reference)
/// - Can inspect the current context (character, state, position)
/// - Can produce output and mutate state
/// - Is independently testable
///
/// # Example
/// ```ignore
/// struct Rule11VowelYe;
///
/// impl BrailleRule for Rule11VowelYe {
///     fn meta(&self) -> &'static RuleMeta { &META }
///     fn phase(&self) -> Phase { Phase::InterCharacter }
///     fn matches(&self, ctx: &RuleContext) -> bool { /* check conditions */ }
///     fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
///         ctx.emit(36); // ⠤ separator
///         Ok(RuleResult::Continue)
///     }
/// }
/// ```
pub trait BrailleRule: Send + Sync {
    /// Static metadata: name, standard reference, description.
    fn meta(&self) -> &'static RuleMeta;

    /// Which phase this rule belongs to.
    fn phase(&self) -> Phase;

    /// Priority within phase (lower = runs first). Default: 100.
    fn priority(&self) -> u16 {
        100
    }

    /// Fast check: does this rule apply to the current context?
    /// Return false to skip without calling `apply()`.
    fn matches(&self, ctx: &RuleContext) -> bool;

    /// Apply the rule: mutate context (emit output, change state).
    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String>;
}
