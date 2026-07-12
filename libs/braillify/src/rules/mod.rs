//! Rule system for Braille encoding.
//!
//! Each rule is an independent module that implements a specific article
//! of the 2024 Korean Braille Standard (개정 한국 점자 규정).
//!
//! Rules are independently testable and traceable.
//!
//! # Architecture
//!
//! - [`traits::BrailleRule`] — the plugin interface every rule implements
//! - [`engine::RuleEngine`] — the host that registers, sorts, and applies rules
//! - [`context::RuleContext`] — shared state + current position passed to each rule
//!
//! ```ignore
//! let mut engine = RuleEngine::new();
//! engine.register(Box::new(korean::rule_11::Rule11));
//! engine.register(Box::new(korean::rule_12::Rule12));
//! engine.disable("12"); // disable a specific rule
//! engine.apply(&mut ctx)?;  // apply all enabled rules
//! ```

// ── Core infrastructure ─────────────────────────────────
pub mod context;
pub mod emit;
pub mod engine;
pub mod english_shortform;
pub(crate) mod roman_mode;
pub mod token;
pub mod token_engine;
pub mod token_rule;
pub mod token_rules;
pub mod traits;

// ── Rule domains ────────────────────────────────────────
pub mod english_ueb; // 통일영어점자 규정 (Unified English Braille)
pub mod korean; // 한글 점자 규정 (Korean Braille rules)
pub mod math; // 수학 점자 규정 (Math Braille rules)

/// Metadata identifying a braille rule and its source in the standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleMeta {
    /// Article number (e.g., "11" for 제11항)
    pub section: &'static str,
    /// Sub-article (e.g., "b1" for [다만], [붙임])
    pub subsection: Option<&'static str>,
    /// Human-readable name
    pub name: &'static str,
    /// Reference to the 2024 Korean Braille Standard
    pub standard_ref: &'static str,
    /// Short description of what this rule does
    pub description: &'static str,
}
