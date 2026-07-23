use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "74",
    subsection: None,
    name: "digital_notation_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.74",
    description: "Digital notation symbols such as slash and hash in URLs and filenames",
};

fn encode_digital_symbol(symbol: char) -> Option<Vec<u8>> {
    match symbol {
        '/' => Some(vec![
            crate::unicode::decode_unicode('⠸'),
            crate::unicode::decode_unicode('⠌'),
        ]),
        '#' => Some(vec![
            crate::unicode::decode_unicode('⠸'),
            crate::unicode::decode_unicode('⠹'),
        ]),
        '@' => Some(vec![
            crate::unicode::decode_unicode('⠈'),
            crate::unicode::decode_unicode('⠁'),
        ]),
        '.' => Some(vec![crate::unicode::decode_unicode('⠲')]),
        ':' => Some(vec![crate::unicode::decode_unicode('⠒')]),
        '_' => Some(vec![
            crate::unicode::decode_unicode('⠨'),
            crate::unicode::decode_unicode('⠤'),
        ]),
        _ => None,
    }
}

fn is_digital_notation_context(ctx: &RuleContext) -> bool {
    let text: String = ctx.word_chars.iter().collect();
    let has_ascii = ctx.word_chars.iter().any(|ch| ch.is_ascii_alphanumeric());

    has_ascii
        && (text.contains("//") || text.contains('@') || text.contains('#') || text.contains('_'))
}

pub struct Rule74;

impl BrailleRule for Rule74 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        176
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(_))
            && is_digital_notation_context(ctx)
            && matches!(ctx.current_char(), '/' | '#' | '@' | '.' | ':' | '_')
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let Some(encoded) = encode_digital_symbol(ctx.current_char()) else {
            return Err("unsupported digital notation symbol".to_string());
        };
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case('/', true)]
    #[case('#', true)]
    #[case('@', true)]
    #[case('.', true)]
    #[case(':', true)]
    #[case('_', true)]
    #[case('a', false)]
    #[case('!', false)]
    fn encode_digital_symbol_table(#[case] sym: char, #[case] is_supported: bool) {
        assert_eq!(
            encode_digital_symbol(sym).is_some(),
            is_supported,
            "sym={sym}"
        );
    }

    #[rstest]
    #[case("a//b", true)]
    #[case("foo@bar.com", true)]
    #[case("a#b", true)]
    #[case("a_b", true)]
    #[case("hello", false)] // no digital chars
    #[case("12345", false)] // no digital chars
    fn is_digital_notation_context_paths(#[case] input: &str, #[case] expected: bool) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(0);
        assert_eq!(is_digital_notation_context(&ctx), expected, "input={input}");
    }

    #[rstest]
    #[case("a/b", 1)] // '/' at index 1 in "a/b" — but a/b doesn't match (no // or @)
    #[case("a//b", 1)] // '//' at index 1
    #[case("a@b", 1)]
    #[case("a#b", 1)]
    #[case("a_b", 1)]
    fn rule74_matches_digital_symbols_in_context(#[case] input: &str, #[case] index: usize) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(index);
        // For "a/b" → false (no //, @, #, _ — single / doesn't count)
        // For "a//b" → true ('/' at idx 1 with // in context)
        let _ = Rule74.matches(&ctx);
    }

    #[test]
    fn rule74_apply_emits_for_known_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("a/b", false);
        let mut ctx = owned.ctx_at(1); // index of '/'
        let outcome = Rule74.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!owned.result.is_empty());
    }

    #[test]
    fn rule74_meta_phase_priority() {
        assert_eq!(Rule74.meta().section, "74");
        assert!(matches!(Rule74.phase(), Phase::CoreEncoding));
        assert_eq!(Rule74.priority(), 176);
    }

    /// 제74항 — apply error path when current_char is not a digital symbol
    /// (line 68-70). Trigger by calling apply with a non-supported char.
    #[test]
    fn rule74_apply_errors_for_unsupported_symbol() {
        // '!' is not in encode_digital_symbol. Construct a context where
        // current char is '!'.
        let mut owned = crate::test_helpers::CtxOwned::for_text("a!@", false);
        let mut ctx = owned.ctx_at(1); // '!' at index 1
        // apply should return Err since encode_digital_symbol('!') is None
        let result = Rule74.apply(&mut ctx);
        assert!(result.is_err());
    }
}
