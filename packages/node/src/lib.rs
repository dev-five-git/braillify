mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "encode")]
pub fn encode(text: &str) -> Result<Vec<u8>, String> {
    braillify::encode(text)
}

#[wasm_bindgen(js_name = "translateToUnicode")]
pub fn translate_to_unicode(text: &str) -> Result<String, String> {
    braillify::encode_to_unicode(text)
}

#[wasm_bindgen(js_name = "translateToBrailleFont")]
pub fn translate_to_braille_font(text: &str) -> Result<String, String> {
    braillify::encode_to_braille_font(text)
}

/// One rule that produced part of the braille output.
#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct RuleSpan {
    /// Article number of the 2024 Korean Braille Standard, `"-"` for structural
    /// emitter output and `"?"` for a rule whose article is not yet declared.
    pub section: String,
    pub name: String,
    pub description: String,
    /// Which engine produced it: `korean`, `jamo`, `token`, `math`,
    /// `english-ueb` or `emitter`.
    pub kind: String,
    pub start: u32,
    pub end: u32,
    pub braille: String,
}

/// Braille output plus the rules that produced it.
#[wasm_bindgen(getter_with_clone)]
pub struct TraceResult {
    pub braille: String,
    pub rules: Vec<RuleSpan>,
    /// Output cells at least one rule accounts for.
    pub attributed: u32,
    /// Total output cells. Greater than `attributed` when part of the output
    /// came from an engine that is not instrumented yet — English (UEB) in
    /// particular records nothing.
    pub total: u32,
    /// Which engine owned the input: `korean`, `english-ueb` or `math`.
    pub path: String,
}

#[wasm_bindgen(js_name = "translateToUnicodeWithTrace")]
pub fn translate_to_unicode_with_trace(text: &str) -> Result<TraceResult, String> {
    let (cells, trace) = braillify::encode_with_trace(text)?;
    let braille = to_braille(&cells);
    let rules = trace
        .events()
        .iter()
        .filter_map(|event| rule_span(event.rule, event.output.clone(), &cells))
        .collect();

    Ok(TraceResult {
        braille,
        rules,
        attributed: trace.attributed_cells(),
        total: trace.output_len(),
        path: path_label(trace.path()).to_string(),
    })
}

/// One event as a span, or `None` for an id the registry does not resolve.
fn rule_span(
    rule: braillify::RuleId,
    output: core::ops::Range<u32>,
    cells: &[u8],
) -> Option<RuleSpan> {
    let meta = rule.meta()?;
    let kind = rule.kind()?;
    let range = output.start as usize..output.end as usize;
    Some(RuleSpan {
        section: meta.section.to_string(),
        name: meta.name.to_string(),
        description: meta.description.to_string(),
        kind: kind_label(kind).to_string(),
        start: output.start,
        end: output.end,
        braille: to_braille(cells.get(range).unwrap_or_default()),
    })
}

fn path_label(path: braillify::TracePath) -> &'static str {
    match path {
        braillify::TracePath::KoreanRules => "korean",
        braillify::TracePath::EnglishUeb => "english-ueb",
        braillify::TracePath::MathExpression => "math",
    }
}

fn kind_label(kind: braillify::RuleKind) -> &'static str {
    match kind {
        braillify::RuleKind::Korean => "korean",
        braillify::RuleKind::Token => "token",
        braillify::RuleKind::Math => "math",
        braillify::RuleKind::Jamo => "jamo",
        braillify::RuleKind::EnglishUeb => "english-ueb",
        braillify::RuleKind::Emitter => "emitter",
    }
}

fn to_braille(cells: &[u8]) -> String {
    cells
        .iter()
        .filter_map(|cell| char::from_u32(0x2800 + u32::from(*cell)))
        .collect()
}

#[cfg(test)]
mod tests {
    //! Native-host tests for the wasm-bindgen shim. `wasm_bindgen` macros
    //! collapse to plain Rust functions on non-wasm targets, so the
    //! delegations to `braillify::*` are reachable by `cargo test`.
    use super::*;

    #[test]
    fn encode_delegates_to_core() {
        let result = encode("안녕").expect("encode must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_propagates_error() {
        // Emoji is rejected by core encoder → wasm shim propagates `Err`.
        assert!(encode("😀").is_err());
    }

    #[test]
    fn translate_to_unicode_delegates_to_core() {
        let result = translate_to_unicode("hi").expect("must succeed");
        for ch in result.chars() {
            let cp = ch as u32;
            assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
        }
    }

    #[test]
    fn translate_to_unicode_propagates_error() {
        assert!(translate_to_unicode("😀").is_err());
    }

    #[test]
    fn translate_to_braille_font_delegates_to_core() {
        let result = translate_to_braille_font("hi").expect("must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn translate_to_braille_font_propagates_error() {
        assert!(translate_to_braille_font("😀").is_err());
    }

    #[test]
    fn set_panic_hook_is_callable() {
        // Exercises the no-op path on default (no `console_error_panic_hook` feature).
        utils::set_panic_hook();
    }

    #[test]
    fn trace_reports_the_rules_behind_the_braille() {
        let result = translate_to_unicode_with_trace("안녕").expect("must succeed");

        assert_eq!(result.path, "korean");
        assert_eq!(result.attributed, result.total);
        assert!(!result.rules.is_empty());
        for span in &result.rules {
            assert!(span.end > span.start, "a span must cover a cell");
            assert_eq!(span.braille.chars().count() as u32, span.end - span.start);
        }
    }

    #[test]
    fn trace_propagates_error() {
        assert!(translate_to_unicode_with_trace("😀").is_err());
    }

    /// An id outside the registry names no rule, so it yields no span.
    #[test]
    fn an_unresolvable_id_yields_no_span() {
        assert!(rule_span(braillify::RuleId::UNATTRIBUTED, 0..1, &[0]).is_none());
    }

    /// A span reaching past the output keeps its cells empty rather than
    /// panicking, so a stale range can never take the binding down.
    #[test]
    fn a_span_past_the_output_carries_no_cells() {
        let (_, trace) = braillify::encode_with_trace("안녕").expect("must encode");
        let rule = trace.events().first().expect("안녕 records events").rule;
        let span = rule_span(rule, 0..99, &[0]).expect("the emitter id resolves");

        assert!(span.braille.is_empty());
    }

    #[rstest::rstest]
    #[case::korean(braillify::RuleKind::Korean, "korean")]
    #[case::token(braillify::RuleKind::Token, "token")]
    #[case::math(braillify::RuleKind::Math, "math")]
    #[case::jamo(braillify::RuleKind::Jamo, "jamo")]
    #[case::english_ueb(braillify::RuleKind::EnglishUeb, "english-ueb")]
    #[case::emitter(braillify::RuleKind::Emitter, "emitter")]
    fn every_engine_has_a_label(#[case] kind: braillify::RuleKind, #[case] expected: &str) {
        assert_eq!(kind_label(kind), expected);
    }

    #[rstest::rstest]
    #[case::korean(braillify::TracePath::KoreanRules, "korean")]
    #[case::english_ueb(braillify::TracePath::EnglishUeb, "english-ueb")]
    #[case::math(braillify::TracePath::MathExpression, "math")]
    fn every_path_has_a_label(#[case] path: braillify::TracePath, #[case] expected: &str) {
        assert_eq!(path_label(path), expected);
    }
}
