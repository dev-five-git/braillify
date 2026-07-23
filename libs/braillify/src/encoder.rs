use std::borrow::Cow;

use crate::rules;
use crate::rules::context::EncodingMode;
use crate::rules::token::{Token, WordMeta, WordToken};

pub struct Encoder {
    pub(crate) is_english: bool,
    triple_big_english: bool,
    english_indicator: bool,
    has_processed_word: bool,
    pub(crate) needs_english_continuation: bool,
    parenthesis_stack: Vec<bool>,
    default_mode: Option<EncodingMode>,
    matrix_context_active: bool,
    math_mode_active: bool,
    rule_engine: rules::engine::RuleEngine,
    token_engine: rules::token_engine::TokenRuleEngine,
}

fn document_has_ascii_and_korean(tokens: &[Token<'_>]) -> bool {
    let mut has_ascii_alphabetic = false;
    let mut has_korean = false;

    for token in tokens {
        if let Token::Word(word) = token {
            has_ascii_alphabetic |= word.meta.has_ascii_alphabetic;
            has_korean |= word.meta.has_korean;
            if has_ascii_alphabetic && has_korean {
                return true;
            }
        }
    }

    false
}

impl Encoder {
    pub fn new(english_indicator: bool) -> Self {
        let mut rule_engine = rules::engine::RuleEngine::new();

        // ── Preprocessing ────────────────────────────────
        rule_engine.register(Box::new(rules::korean::rule_53::Rule53));

        // ── WordShortcut ─────────────────────────────────
        rule_engine.register(Box::new(rules::korean::rule_18::Rule18));

        // ── CoreEncoding ─────────────────────────────────
        rule_engine.register(Box::new(rules::korean::rule_44::Rule44));
        rule_engine.register(Box::new(rules::korean::rule_66::Rule66));
        rule_engine.register(Box::new(rules::korean::rule_67::Rule67));
        rule_engine.register(Box::new(rules::korean::rule_27::Rule27));
        rule_engine.register(Box::new(rules::korean::rule_19::Rule19));
        rule_engine.register(Box::new(rules::korean::rule_20::Rule20));
        rule_engine.register(Box::new(rules::korean::rule_21::Rule21));
        rule_engine.register(Box::new(rules::korean::rule_22::Rule22));
        rule_engine.register(Box::new(rules::korean::rule_23::Rule23));
        rule_engine.register(Box::new(rules::korean::rule_24::Rule24));
        rule_engine.register(Box::new(rules::korean::rule_25::Rule25));
        rule_engine.register(Box::new(rules::korean::rule_26::Rule26));
        rule_engine.register(Box::new(rules::korean::rule_16::Rule16));
        rule_engine.register(Box::new(rules::korean::rule_14::Rule14));
        rule_engine.register(Box::new(rules::korean::rule_13::Rule13));
        rule_engine.register(Box::new(rules::korean::rule_korean::RuleKorean));
        rule_engine.register(Box::new(rules::korean::rule_28::Rule28));
        rule_engine.register(Box::new(rules::korean::rule_40::Rule40));
        rule_engine.register(Box::new(rules::korean::rule_31::Rule31));
        rule_engine.register(Box::new(rules::korean::rule_8::Rule8));
        rule_engine.register(Box::new(rules::korean::rule_2::Rule2));
        rule_engine.register(Box::new(rules::korean::rule_1::Rule1));
        rule_engine.register(Box::new(rules::korean::rule_3::Rule3));
        rule_engine.register(Box::new(
            rules::korean::rule_english_symbol::RuleEnglishSymbol,
        ));
        rule_engine.register(Box::new(rules::korean::rule_68::Rule68));
        rule_engine.register(Box::new(rules::korean::rule_69::Rule69));
        rule_engine.register(Box::new(rules::korean::rule_70::Rule70));
        rule_engine.register(Box::new(rules::korean::rule_71::Rule71));
        rule_engine.register(Box::new(rules::korean::rule_72::Rule72));
        rule_engine.register(Box::new(rules::korean::rule_73::Rule73));
        rule_engine.register(Box::new(rules::korean::rule_74::Rule74));
        rule_engine.register(Box::new(rules::korean::rule_61::Rule61));
        rule_engine.register(Box::new(rules::korean::rule_41::Rule41));
        rule_engine.register(Box::new(rules::korean::rule_56::Rule56));
        rule_engine.register(Box::new(rules::korean::rule_57::Rule57));
        rule_engine.register(Box::new(rules::korean::rule_58::Rule58));
        rule_engine.register(Box::new(rules::korean::rule_60::Rule60));
        rule_engine.register(Box::new(rules::korean::rule_64::Rule64));
        rule_engine.register(Box::new(rules::korean::rule_64::Rule64Square));
        rule_engine.register(Box::new(rules::korean::rule_65::Rule65));
        rule_engine.register(Box::new(rules::korean::rule_49::Rule49));
        rule_engine.register(Box::new(rules::korean::rule_space::RuleSpace));
        rule_engine.register(Box::new(rules::korean::rule_math::RuleMath));
        rule_engine.register(Box::new(rules::korean::rule_fraction::RuleFraction));

        // ── InterCharacter ───────────────────────────────
        rule_engine.register(Box::new(rules::korean::rule_11::Rule11));
        rule_engine.register(Box::new(rules::korean::rule_12::Rule12));

        let mut token_engine = rules::token_engine::TokenRuleEngine::new();
        // PDF 한국어 제73항 [붙임 1] — U+F000 빈칸 + 슬래시-대안 조사 prefix 삽입.
        // 매우 일찍 등록(다른 규칙이 토큰을 분리하기 전).
        token_engine.register(Box::new(
            rules::token_rules::rule_73_appendix_placeholder::Rule73AppendixPlaceholderRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::middle_korean_detector::MiddleKoreanDetectorRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::historical_gloss_spacing::HistoricalGlossSpacingRule,
        ));
        token_engine.register(Box::new(rules::token_rules::normalize::NormalizeEllipsis));
        // PDF 한국어 제33항 — 학술 인용 형식 year-suffix token (1998a,, 1998b;).
        token_engine.register(Box::new(
            rules::token_rules::rule_33_citation::Rule33CitationYearSuffixRule,
        ));
        token_engine.register(Box::new(rules::token_rules::latex_math::LatexMergeRule));
        token_engine.register(Box::new(
            rules::token_rules::emphasis_ring::EmphasisRingRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::math_expression::MathExpressionTokenRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::latex_fraction::LatexFractionRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::inline_fraction::InlineFractionRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::word_shortcut::WordShortcutRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::roman_numeral::RomanNumeralRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::digital_notation::DigitalNotationRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::uppercase_passage::UppercasePassageRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::middle_dot_spacing::MiddleDotSpacingRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::quote_attachment::QuoteAttachmentRule,
        ));
        token_engine.register(Box::new(rules::token_rules::spacing::AsteriskSpacingRule));
        token_engine.register(Box::new(
            rules::token_rules::spacing::KoreanAuxiliaryVerbSpacingRule,
        ));
        token_engine.register(Box::new(
            rules::token_rules::english_dominant_korean_wrap::EnglishDominantKoreanWrapRule,
        ));

        Self {
            english_indicator,
            is_english: false,
            triple_big_english: false,
            has_processed_word: false,
            needs_english_continuation: false,
            parenthesis_stack: Vec::new(),
            default_mode: None,
            matrix_context_active: false,
            math_mode_active: false,
            rule_engine,
            token_engine,
        }
    }

    pub fn english_indicator(&self) -> bool {
        self.english_indicator
    }

    pub fn reset_state(&mut self) {
        self.is_english = false;
        self.triple_big_english = false;
        self.has_processed_word = false;
        self.needs_english_continuation = false;
        self.parenthesis_stack.clear();
        self.default_mode = None;
        self.matrix_context_active = false;
        self.math_mode_active = false;
    }

    pub fn set_default_mode(&mut self, mode: EncodingMode) {
        if mode == EncodingMode::Math {
            self.math_mode_active = true;
        }
        self.default_mode = Some(mode);
    }

    pub fn set_matrix_context_active(&mut self, active: bool) {
        self.matrix_context_active = active;
    }

    pub fn set_math_mode_active(&mut self, active: bool) {
        self.math_mode_active = active;
    }

    fn encode_via_ir(&mut self, text: &str, result: &mut Vec<u8>) -> Result<(), String> {
        self.encode_via_ir_with_transform(text, result, |_, _| Ok(()))
    }

    fn encode_via_ir_with_transform<F>(
        &mut self,
        text: &str,
        result: &mut Vec<u8>,
        transform: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&str, &mut Vec<Token<'_>>) -> Result<(), String>,
    {
        let mut ir = rules::token::DocumentIR::parse(text, self.english_indicator);
        ir.state.matrix_context_active = self.matrix_context_active;
        ir.state.math_mode_active = self.math_mode_active;

        if let Some(mode) = self.default_mode
            && mode != ir.state.current_mode()
        {
            while ir.state.pop_mode().is_some() {}
            ir.state.push_mode(mode);
        }

        // Pre-compute document-level predicates used by EnglishDominantKoreanWrapRule.
        // This keeps PostWord rule dispatch O(1) per token instead of re-scanning
        // the full document for each token.
        if document_has_ascii_and_korean(&ir.tokens) {
            ir.state.doc_summary =
                rules::token_rules::english_dominant_korean_wrap::compute_document_summary(
                    &ir.tokens,
                );
        }

        let state_before_token_rules = ir.state.clone();
        self.token_engine.apply_all(&mut ir.tokens, &mut ir.state)?;
        let mode_stack_after_token_rules = ir.state.mode_stack.clone();
        // 제39항 영-한 wrap 활성화 신호는 token 단계의 결정이며 emit 단계에서도
        // 유효해야 한다. mode_stack과 함께 보존한다.
        let wrap_active_after_token_rules = ir.state.english_dominant_wrap_active;
        let no_indicator_after_token_rules = ir.state.english_dominant_no_indicator;
        ir.state = state_before_token_rules;
        ir.state.mode_stack = mode_stack_after_token_rules;
        ir.state.english_dominant_wrap_active = wrap_active_after_token_rules;
        ir.state.english_dominant_no_indicator = no_indicator_after_token_rules;
        transform(text, &mut ir.tokens)?;

        let output = rules::emit::emit(&mut ir, &mut self.rule_engine)?;
        result.extend(output);

        self.is_english = ir.state.is_english;
        self.triple_big_english = ir.state.triple_big_english;
        self.has_processed_word = ir.state.has_processed_word;
        self.needs_english_continuation = ir.state.needs_english_continuation;
        self.parenthesis_stack = ir.state.parenthesis_stack;
        Ok(())
    }

    pub fn encode(&mut self, text: &str, result: &mut Vec<u8>) -> Result<(), String> {
        // UEB Grade-2 path: pure-English input (no Korean, UEB-eligible, no
        // explicit mode) is encoded by the unified English engine. It returns
        // `Some` only when it fully handles the input; otherwise we fall through
        // to the legacy path so math and mixed Korean contexts keep their routing.
        // Eligibility is an ASCII letter or a §9 typeform signal, so a letterless
        // but emphasised input (`27.̲9`, `83%̲`) is still UEB's (`is_ueb_eligible`).
        if self.default_mode.is_none()
            && !text.is_empty()
            && !text.chars().any(crate::utils::is_korean_char)
            && crate::rules::english_ueb::is_ueb_eligible(text)
            // Preflight (Phase 7): do NOT intercept inputs the legacy math
            // pipeline owns by an *unambiguous* math signal — a function name
            // (`sin`, `log2`) or a letter/digit run with no spaces or English
            // punctuation (`3ab`, `sin3x`, `f(x-1)`). English prose that merely
            // contains `-`, `(`, `,`, `.` is NOT blocked (that over-broad reading
            // of the math detector would swallow `child-ish-ly`, `with(er)`, …).
            && !crate::rules::english_ueb::is_math_owned(text)
            && let Some(bytes) = crate::rules::english_ueb::try_encode(text)
        {
            result.extend(bytes);
            return Ok(());
        }
        self.encode_via_ir(text, result)
    }

    pub fn encode_with_formatting(
        &mut self,
        text: &str,
        spans: &[crate::FormattingSpan],
        result: &mut Vec<u8>,
    ) -> Result<(), String> {
        if spans.is_empty() {
            return self.encode(text, result);
        }

        self.encode_via_ir_with_transform(text, result, |source, tokens| {
            inject_formatting_tokens(source, spans, tokens)
        })
    }
}

fn inject_formatting_tokens(
    text: &str,
    spans: &[crate::FormattingSpan],
    tokens: &mut Vec<Token<'_>>,
) -> Result<(), String> {
    let text_len = text.len();
    let mut starts: std::collections::BTreeMap<usize, Vec<crate::FormattingKind>> =
        std::collections::BTreeMap::new();
    let mut ends: std::collections::BTreeMap<usize, Vec<crate::FormattingKind>> =
        std::collections::BTreeMap::new();

    for span in spans {
        let start = span.range.start;
        let end = span.range.end;
        if start >= end {
            return Err(format!("Invalid formatting span range: {start}..{end}"));
        }
        if end > text_len {
            return Err(format!(
                "Formatting span out of bounds: {start}..{end} (len={text_len})"
            ));
        }
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(format!(
                "Formatting span must align to UTF-8 boundaries: {start}..{end}"
            ));
        }
        starts.entry(start).or_default().push(span.kind);
        ends.entry(end).or_default().push(span.kind);
    }

    let mut new_tokens = Vec::new();
    let mut cursor = 0usize;

    let emit_events_at =
        |pos: usize,
         out: &mut Vec<Token<'_>>,
         start_map: &mut std::collections::BTreeMap<usize, Vec<crate::FormattingKind>>,
         end_map: &mut std::collections::BTreeMap<usize, Vec<crate::FormattingKind>>| {
            if let Some(kinds) = end_map.remove(&pos) {
                for kind in kinds.iter().rev() {
                    let (_, close) = kind.markers();
                    out.push(Token::PreEncoded(close.to_vec()));
                }
            }
            if let Some(kinds) = start_map.remove(&pos) {
                for kind in kinds {
                    let (open, _) = kind.markers();
                    out.push(Token::PreEncoded(open.to_vec()));
                }
            }
        };

    emit_events_at(cursor, &mut new_tokens, &mut starts, &mut ends);

    for token in tokens.iter() {
        match token {
            Token::Word(word) => {
                let text_ref = word.text.as_ref();
                let word_end = cursor.saturating_add(text_ref.len());
                let mut internal_points = starts
                    .keys()
                    .chain(ends.keys())
                    .copied()
                    .filter(|pos| *pos > cursor && *pos < word_end)
                    .map(|pos| pos - cursor)
                    .collect::<Vec<_>>();
                internal_points.sort_unstable();
                internal_points.dedup();

                let mut local_start = 0usize;
                for local_end in internal_points
                    .into_iter()
                    .chain(std::iter::once(text_ref.len()))
                {
                    let seg = &text_ref[local_start..local_end];
                    let seg_chars: Vec<char> = seg.chars().collect();
                    new_tokens.push(Token::Word(WordToken {
                        text: Cow::Owned(seg.to_string()),
                        chars: seg_chars.clone(),
                        meta: WordMeta::from_chars(&seg_chars),
                    }));

                    cursor += seg.len();
                    emit_events_at(cursor, &mut new_tokens, &mut starts, &mut ends);
                    local_start = local_end;
                }
            }
            Token::Space(space) => {
                new_tokens.push(Token::Space(*space));
                cursor += 1;
                emit_events_at(cursor, &mut new_tokens, &mut starts, &mut ends);
            }
            _ => new_tokens.push(token.clone()),
        }
    }

    emit_events_at(cursor, &mut new_tokens, &mut starts, &mut ends);
    if !starts.is_empty() || !ends.is_empty() {
        return Err("Formatting spans could not be mapped to token boundaries".to_string());
    }

    *tokens = new_tokens;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FormattingKind;
    use crate::FormattingSpan;

    /// `inject_formatting_tokens` Err arm for span start >= end (line 297).
    #[test]
    fn inject_formatting_invalid_start_ge_end() {
        let text = "abc";
        let spans = vec![FormattingSpan {
            kind: FormattingKind::Emphasis,
            range: 2..2,
        }];
        let mut tokens: Vec<Token<'_>> = vec![];
        let result = inject_formatting_tokens(text, &spans, &mut tokens);
        assert!(result.is_err());
    }

    /// `inject_formatting_tokens` Err arm for span out of bounds (line 300-302).
    #[test]
    fn inject_formatting_out_of_bounds() {
        let text = "ab";
        let spans = vec![FormattingSpan {
            kind: FormattingKind::Emphasis,
            range: 0..5,
        }];
        let mut tokens: Vec<Token<'_>> = vec![];
        let result = inject_formatting_tokens(text, &spans, &mut tokens);
        assert!(result.is_err());
    }

    /// `inject_formatting_tokens` Err arm for span not aligned to UTF-8
    /// boundary (lines 304-307).
    #[test]
    fn inject_formatting_not_utf8_boundary() {
        let text = "가나"; // 6 bytes (3 each); byte 1 is mid-char
        let spans = vec![FormattingSpan {
            kind: FormattingKind::Emphasis,
            range: 1..6,
        }];
        let mut tokens: Vec<Token<'_>> = vec![];
        let result = inject_formatting_tokens(text, &spans, &mut tokens);
        assert!(result.is_err());
    }

    /// `inject_formatting_tokens` `_ => push(token.clone())` arm (line 375)
    /// fires when tokens contain non-Word non-Space variants.
    #[test]
    fn inject_formatting_with_preencoded_token_passes_through() {
        let text = "ab";
        let spans = vec![FormattingSpan {
            kind: FormattingKind::Emphasis,
            range: 0..2,
        }];
        let mut tokens: Vec<Token<'_>> = vec![
            Token::PreEncoded(vec![0, 1, 2]),
            Token::Word(WordToken {
                text: std::borrow::Cow::Borrowed("ab"),
                chars: vec!['a', 'b'],
                meta: WordMeta::from_chars(&['a', 'b']),
            }),
        ];
        let _ = inject_formatting_tokens(text, &spans, &mut tokens);
    }

    /// `inject_formatting_tokens` Err arm at line 381 when starts/ends cannot
    /// be mapped to token boundaries (span beyond actual tokens).
    #[test]
    fn inject_formatting_spans_unmappable_to_tokens() {
        let text = "abc";
        // Span over "abc" but tokens slice is empty so events can't be emitted.
        let spans = vec![FormattingSpan {
            kind: FormattingKind::Emphasis,
            range: 0..3,
        }];
        let mut tokens: Vec<Token<'_>> = vec![]; // no tokens to absorb events
        let result = inject_formatting_tokens(text, &spans, &mut tokens);
        // Without tokens, starts/ends remain non-empty after the loop → Err.
        assert!(result.is_err());
    }

    /// `encode_with_formatting` with empty spans short-circuits to plain `encode`.
    #[test]
    fn encode_with_formatting_empty_spans_short_circuits() {
        let mut encoder = Encoder::new(false);
        let mut result = Vec::new();
        encoder
            .encode_with_formatting("안녕", &[], &mut result)
            .expect("ok");
        assert!(!result.is_empty());
    }
}
