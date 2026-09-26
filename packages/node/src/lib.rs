mod utils;

use wasm_bindgen::prelude::*;

/// `context` names how to read text whose print shape alone does not decide
/// the rule — `science`, `math`, `korean`, … (see `braillify::encode_in_context`).
#[wasm_bindgen(js_name = "encode")]
pub fn encode(text: &str, context: Option<String>) -> Result<Vec<u8>, String> {
    match context {
        Some(context) => braillify::encode_in_context(text, &context),
        None => braillify::encode(text),
    }
}

#[wasm_bindgen(js_name = "translateToUnicode")]
pub fn translate_to_unicode(text: &str, context: Option<String>) -> Result<String, String> {
    match context {
        Some(context) => braillify::encode_to_unicode_in_context(text, &context),
        None => braillify::encode_to_unicode(text),
    }
}

#[wasm_bindgen(js_name = "translateToBrailleFont")]
pub fn translate_to_braille_font(text: &str, context: Option<String>) -> Result<String, String> {
    match context {
        Some(context) => braillify::encode_to_braille_font_in_context(text, &context),
        None => braillify::encode_to_braille_font(text),
    }
}

/// One rule that produced part of the braille output.
///
/// A plain serialisable struct rather than an exported class. Handing a
/// `Vec` of exported structs across the boundary makes wasm-bindgen import a
/// JS constructor into the module, and that import cannot be linked under
/// Bun, which is what runs this package's tests.
#[derive(Clone, serde::Serialize)]
pub struct RuleSpan {
    /// Article number within the standard, or `"-"` for structural output the
    /// standard prescribes without giving it an article, such as the blank
    /// between words.
    pub section: String,
    /// Sub-division of the article, such as a 붙임, when the rule implements one.
    pub subsection: String,
    /// The article in full, naming its series. A rule may run in one engine and
    /// implement an article from another — circled numbers are 한글 제64항 even
    /// though the math engine encodes them — so the number alone is ambiguous.
    pub standard_ref: String,
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
///
/// Serialised rather than exported as a class, for the reason [`RuleSpan`]
/// gives: an exported struct makes wasm-bindgen import per-field getters into
/// the module, and Bun cannot link those.
#[derive(serde::Serialize)]
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
pub fn translate_to_unicode_with_trace(text: &str) -> Result<String, String> {
    let result = trace_result(text)?;
    serde_json::to_string(&result).map_err(|error| error.to_string())
}

fn trace_result(text: &str) -> Result<TraceResult, String> {
    let (cells, trace) = braillify::encode_with_trace(text)?;
    let braille = to_braille(&cells);

    Ok(TraceResult {
        braille,
        rules: rule_spans(&cells, &trace),
        attributed: trace.attributed_cells(),
        total: trace.output_len(),
        path: path_label(trace.path()).to_string(),
    })
}

/// Every traced event that names a rule, in output order.
fn rule_spans(cells: &[u8], trace: &braillify::Trace) -> Vec<RuleSpan> {
    trace
        .events()
        .iter()
        .filter_map(|event| rule_span(event.rule, event.output.clone(), cells))
        .collect()
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
        subsection: meta.subsection.unwrap_or_default().to_string(),
        standard_ref: meta.standard_ref.to_string(),
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
        let result = encode("안녕", None).expect("encode must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_propagates_error() {
        // Emoji is rejected by core encoder → wasm shim propagates `Err`.
        assert!(encode("😀", None).is_err());
    }

    #[test]
    fn translate_to_unicode_delegates_to_core() {
        let result = translate_to_unicode("hi", None).expect("must succeed");
        for ch in result.chars() {
            let cp = ch as u32;
            assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
        }
    }

    #[test]
    fn translate_to_unicode_propagates_error() {
        assert!(translate_to_unicode("😀", None).is_err());
    }

    #[test]
    fn translate_to_braille_font_delegates_to_core() {
        let result = translate_to_braille_font("hi", None).expect("must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn translate_to_braille_font_propagates_error() {
        assert!(translate_to_braille_font("😀", None).is_err());
    }

    #[test]
    fn every_function_reads_the_named_context() {
        let science = Some("science".to_string());
        assert_eq!(
            translate_to_unicode("pOH", science.clone()).as_deref(),
            Ok("⠴⠏⠠⠕⠠⠓")
        );
        assert_eq!(
            translate_to_braille_font("pOH", science.clone()).as_deref(),
            Ok("⠴⠏⠠⠕⠠⠓")
        );
        assert_eq!(encode("pOH", science).map(|cells| cells.len()), Ok(6));
    }

    #[test]
    fn an_unknown_context_is_an_error() {
        assert!(translate_to_unicode("pOH", Some("chemistry".to_string())).is_err());
    }

    #[test]
    fn set_panic_hook_is_callable() {
        // Exercises the no-op path on default (no `console_error_panic_hook` feature).
        utils::set_panic_hook();
    }

    #[test]
    fn trace_reports_the_rules_behind_the_braille() {
        let json = translate_to_unicode_with_trace("안녕").expect("must succeed");
        let result = trace_result("안녕").expect("must succeed");

        assert!(json.starts_with('{'), "the binding hands back JSON");
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
