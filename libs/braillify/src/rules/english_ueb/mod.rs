//! Unified English Braille (UEB) Grade-2 encoder.
//!
//! Mirrors the Math token-engine architecture: a parser produces a token
//! stream ([`parser`]), a document engine ([`engine`]) manages modes and
//! capitalisation indicators, and per-rule [`contraction::ContractionRule`]
//! impls (one file per UEB §10.x clause) apply contractions via longest-match +
//! priority — the §10.10 preference rule, encoded structurally.
//!
//! [`try_encode`] returns `Some` only when the engine fully handles the input;
//! otherwise the caller falls back to the legacy path. This lets the engine
//! grow rule-by-rule without regressing already-passing cases.
//!
//! Source of truth: `docs/Rules-of-Unified-English-Braille-2024.pdf`.

pub mod compound;
pub mod contraction;
pub mod engine;
pub(crate) mod korean_context;
pub mod parser;
pub mod pronunciation;
pub mod rule_10_1;
pub mod rule_10_11;
pub mod rule_10_12;
pub mod rule_10_13;
pub mod rule_10_2;
pub mod rule_10_3;
pub mod rule_10_4;
pub mod rule_10_5;
pub mod rule_10_6;
pub mod rule_10_6_8;
pub mod rule_10_6_middle;
pub mod rule_10_6_restricted;
pub mod rule_10_7;
pub mod rule_10_7_pron;
pub mod rule_10_7_struct;
pub mod rule_10_8;
pub mod rule_10_9;
pub mod rule_10_9_list;
pub mod rule_11;
pub mod rule_12;
pub mod rule_13;
pub mod rule_14;
pub mod rule_15;
pub mod rule_16;
pub mod rule_3;
pub mod rule_3_24;
pub mod rule_4;
pub mod rule_5_7;
pub mod rule_6;
pub mod rule_7;
pub mod rule_9;
pub mod span;
pub mod standing_alone;
pub mod token;

use engine::EnglishUebEngine;

thread_local! {
    /// Word attempts and structural indicators, in emission order. The engine
    /// encodes a word under several constraint combinations and keeps one, so
    /// [`align_selected`] separates kept attempts from discarded ones while
    /// retaining indicators emitted directly into the selected output.
    static ATTRIBUTIONS: std::cell::RefCell<Option<Vec<AttributionRecord>>> =
        const { std::cell::RefCell::new(None) };
}

enum AttributionRecord {
    Word(WordAttempt),
    Indicator(NonWordAttempt),
    Direct(NonWordAttempt),
}

/// The cells one attempt produced, plus where each rule's cells sat inside them.
struct WordAttempt {
    cells: Vec<u8>,
    moves: Vec<(crate::rules::trace::RuleId, u32, u32)>,
    /// Where the cells were written, for an attempt that went straight into a
    /// buffer rather than being one of several the engine chose between. The
    /// literal `<sub>` markup of a chemical line repeats the same two-cell run
    /// a dozen times, and a search cannot tell those occurrences apart.
    offset: Option<usize>,
}

struct NonWordAttempt {
    cells: Vec<u8>,
    rule: crate::rules::trace::RuleId,
    /// Where the cells were written, when they went straight into the selected
    /// output. A one-cell indicator such as `⠠` recurs all over a capitalised
    /// line, so looking for it afterwards finds an earlier occurrence than the
    /// one this record wrote. `None` marks a record taken against a buffer that
    /// is appended elsewhere, whose final position is not known here.
    offset: Option<usize>,
}

/// Accumulates the moves of one word-encoding attempt.
///
/// Offsets are taken against the attempt's own output as it is built, because
/// the encoder can insert cells between moves (a §10.13 line break), so a move's
/// position is not the running sum of the moves before it.
pub(super) struct AttemptRecorder {
    /// `None` when no trace is being collected, so an untraced encode allocates
    /// nothing per word. The check costs one thread-local read per attempt
    /// rather than one per move.
    moves: Option<Vec<(crate::rules::trace::RuleId, u32, u32)>>,
}

impl AttemptRecorder {
    pub(super) fn new() -> Self {
        let collecting = ATTRIBUTIONS.with(|slot| slot.borrow().is_some());
        Self {
            moves: collecting.then(Vec::new),
        }
    }

    pub(super) fn push(&mut self, rule: crate::rules::trace::RuleId, offset: usize, len: usize) {
        if let Some(moves) = self.moves.as_mut() {
            moves.push((rule, offset as u32, len as u32));
        }
    }

    pub(super) fn finish(self, cells: &[u8]) {
        self.finish_at(cells, None);
    }

    pub(super) fn finish_at(self, cells: &[u8], offset: Option<usize>) {
        let Some(moves) = self.moves else {
            return;
        };
        ATTRIBUTIONS.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(records) = slot.as_mut()
            {
                records.push(AttributionRecord::Word(WordAttempt {
                    cells: cells.to_vec(),
                    moves,
                    offset,
                }));
            }
        });
    }
}

/// [`try_encode`] plus the rule behind each stretch of the output.
///
/// A word encoder does not know where its cells land in the finished document,
/// so each attempt's ranges are recovered by locating that attempt's cells in
/// the output. Attempts whose cells are absent were discarded by the engine and
/// contribute nothing. A reported range therefore always points at cells its
/// rule actually produced.
pub(crate) fn try_encode_traced(text: &str) -> Option<(Vec<u8>, Vec<UebSpan>)> {
    let encoded = collect_selected(|| try_encode(text));
    encoded.map(|(cells, moves)| {
        let spans = align_selected(&cells, &moves);
        (cells, spans)
    })
}

/// [`encode_forced`] plus the rule behind each stretch of the output.
pub(crate) fn encode_forced_traced(text: &str) -> Option<(Vec<u8>, Vec<UebSpan>)> {
    let encoded = collect_selected(|| encode_forced(text));
    encoded.map(|(cells, moves)| {
        let spans = align_selected(&cells, &moves);
        (cells, spans)
    })
}

/// One stretch of output and the UEB rule that produced it.
pub(crate) type UebSpan = (crate::rules::trace::RuleId, core::ops::Range<u32>);

fn collect_selected(
    encode: impl FnOnce() -> Option<Vec<u8>>,
) -> Option<(Vec<u8>, Vec<AttributionRecord>)> {
    ATTRIBUTIONS.with(|slot| *slot.borrow_mut() = Some(Vec::new()));
    let encoded = encode();
    let records = ATTRIBUTIONS
        .with(|slot| slot.borrow_mut().take())
        .unwrap_or_default();
    encoded.map(|cells| (cells, records))
}

fn attribution_checkpoint() -> usize {
    ATTRIBUTIONS.with(|slot| slot.borrow().as_ref().map_or(0, Vec::len))
}

/// Move records taken since `checkpoint` from a local buffer's coordinates to
/// the output's, once that buffer has been appended at `base`.
///
/// A word is assembled in its own buffer, so a record made while filling it
/// knows only its place inside that buffer. Rebasing at the append is what
/// turns those into positions the finished output can be indexed by.
fn rebase_attributions(checkpoint: usize, base: usize) {
    ATTRIBUTIONS.with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut()
            && let Some(records) = slot.as_mut()
        {
            for record in records.iter_mut().skip(checkpoint) {
                let offset = match record {
                    AttributionRecord::Word(w) => &mut w.offset,
                    AttributionRecord::Indicator(a) | AttributionRecord::Direct(a) => &mut a.offset,
                };
                if let Some(offset) = offset.as_mut() {
                    *offset += base;
                }
            }
        }
    });
}

fn rollback_attributions(checkpoint: usize) {
    ATTRIBUTIONS.with(|slot| {
        if let Some(records) = slot.borrow_mut().as_mut() {
            records.truncate(checkpoint);
        }
    });
}

/// Place each attempt's moves in the finished output, skipping attempts the
/// engine discarded.
///
/// The scan only moves forward, so an attempt is matched at or after everything
/// already placed. A discarded attempt is recognised by its cells not appearing
/// there — the engine never emitted them.
fn align_selected(cells: &[u8], records: &[AttributionRecord]) -> Vec<UebSpan> {
    let mut direct_spans = Vec::new();
    let mut direct_cursor = 0usize;
    for record in records {
        if let AttributionRecord::Direct(direct) = record
            && let Some(range) = locate(cells, direct, &mut direct_cursor, &[])
        {
            direct_spans.push((direct.rule, range));
        }
    }

    let mut indicator_spans = Vec::new();
    let mut indicator_cursor = 0usize;
    for record in records {
        if let AttributionRecord::Indicator(indicator) = record
            && let Some(range) = locate(cells, indicator, &mut indicator_cursor, &direct_spans)
        {
            push_without_indicators(&mut indicator_spans, (indicator.rule, range), &direct_spans);
        }
    }

    let mut fixed_spans = direct_spans.clone();
    fixed_spans.extend(indicator_spans.iter().cloned());
    fixed_spans.sort_by_key(|(_, range)| range.start);

    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for record in records {
        let attempt = match record {
            AttributionRecord::Word(attempt) => attempt,
            AttributionRecord::Indicator(_) | AttributionRecord::Direct(_) => continue,
        };
        let placed = attempt.offset.filter(|offset| {
            cells.get(*offset..offset + attempt.cells.len()) == Some(attempt.cells.as_slice())
        });
        let Some(base) =
            placed.or_else(|| find_from_outside(cells, &attempt.cells, cursor, &direct_spans))
        else {
            continue;
        };
        for (rule, offset, len) in &attempt.moves {
            let start = base + *offset as usize;
            let end = start + *len as usize;
            push_without_indicators(&mut spans, (*rule, start as u32..end as u32), &fixed_spans);
        }
        cursor = base + attempt.cells.len();
    }
    spans.extend(indicator_spans);
    spans.extend(direct_spans);
    // An empty cell between words is the inter-word blank, the same structural
    // output the Korean emitter accounts for. It carries no dots, so there is no
    // other thing it could be.
    let blank = crate::rules::trace::RuleId::emitter(crate::rules::trace::EmitterRule::WordSpace);
    spans.extend(
        cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| **cell == 0)
            .map(|(index, _)| (blank, index as u32..index as u32 + 1)),
    );
    spans
}

/// Where a record's cells sit in the finished output.
///
/// The position it was written at wins when the output still carries those
/// cells there and nothing already claims them. Searching is the fallback, and
/// the only option for a record taken against a buffer appended elsewhere.
fn locate(
    cells: &[u8],
    record: &NonWordAttempt,
    cursor: &mut usize,
    excluded: &[UebSpan],
) -> Option<core::ops::Range<u32>> {
    let len = record.cells.len();
    if let Some(offset) = record.offset
        && cells.get(offset..offset + len) == Some(record.cells.as_slice())
        && !excluded
            .iter()
            .any(|(_, taken)| taken.start < (offset + len) as u32 && (offset as u32) < taken.end)
    {
        *cursor = offset + len;
        return Some(offset as u32..(offset + len) as u32);
    }
    let base = if excluded.is_empty() {
        find_from(cells, &record.cells, *cursor)?
    } else {
        find_from_outside(cells, &record.cells, *cursor, excluded)?
    };
    *cursor = base + len;
    Some(base as u32..(base + len) as u32)
}

fn push_without_indicators(spans: &mut Vec<UebSpan>, candidate: UebSpan, indicators: &[UebSpan]) {
    let (rule, range) = candidate;
    let mut start = range.start;
    for (_, indicator) in indicators {
        if indicator.end <= start {
            continue;
        }
        if indicator.start >= range.end {
            break;
        }
        if start < indicator.start {
            spans.push((rule, start..indicator.start));
        }
        start = start.max(indicator.end);
    }
    if start < range.end {
        spans.push((rule, start..range.end));
    }
}

fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from + needle.len() > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| offset + from)
}

fn find_from_outside(
    haystack: &[u8],
    needle: &[u8],
    from: usize,
    excluded: &[UebSpan],
) -> Option<usize> {
    let mut cursor = from;
    loop {
        let base = find_from(haystack, needle, cursor)?;
        let end = base + needle.len();
        let overlap = excluded
            .iter()
            .find(|(_, range)| range.start < end as u32 && (base as u32) < range.end);
        let Some((_, range)) = overlap else {
            return Some(base);
        };
        cursor = range.end as usize;
    }
}

/// Sources of a selected contraction move that are not [`ContractionRule`]
/// objects. They occupy the first slots of the UEB id space so a contraction
/// rule's id stays a fixed offset from its registration index.
///
/// [`ContractionRule`]: contraction::ContractionRule
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UebMoveSource {
    Shortform = 0,
    Anglicised = 1,
    Letter = 2,
    AlphabeticWordsign = 3,
    StrongWordsign = 4,
    LowerWordsign = 5,
    Numeric = 6,
    Symbol = 7,
    Grade1Indicator = 8,
    CapitalLetterIndicator = 9,
    CapitalisedWordIndicator = 10,
    InlineNemethCode = 11,
}

/// Number of non-rule slots reserved before the contraction rules.
pub(crate) const UEB_RESERVED_SLOTS: usize = 12;

/// Record a whole word that a lookup table resolved in one step, bypassing the
/// contraction search. Without this a wordsign or shortform would leave its
/// cells unexplained even though its section is known exactly.
/// How many attempts have been recorded so far, so a caller can tell whether the
/// encoder it just ran attributed its own output.
pub(super) fn attempt_count() -> usize {
    ATTRIBUTIONS.with(|slot| {
        slot.borrow().as_ref().map_or(0, |records| {
            records
                .iter()
                .filter(|record| matches!(record, AttributionRecord::Word(_)))
                .count()
        })
    })
}

/// A word whose attribution has not been settled yet: where its cells start in
/// the output, and how many attempts existed before it ran.
pub(super) type PendingWord = (usize, usize);

/// Attribute a finished word that nothing else claimed.
///
/// A word normally names itself through the contraction search or a wordsign
/// lookup. The branches that simply spell it out — letters after a digit, an
/// acronym abutting one — reach neither, and this leaves their cells explained
/// as §4.1 letters without double-counting the words that did claim themselves.
pub(super) fn settle_word_attribution(pending: Option<PendingWord>, out: &[u8]) {
    let Some((start, attempts_before)) = pending else {
        return;
    };
    if attempt_count() == attempts_before && out.len() > start {
        record_whole_word_at(UebMoveSource::Letter, &out[start..], Some(start));
    }
}

pub(super) fn settle_symbol_attribution(start: Option<usize>, out: &[u8]) {
    if let Some(start) = start
        && out.len() > start
    {
        record_whole_word_at(UebMoveSource::Symbol, &out[start..], Some(start));
    }
}

pub(super) fn record_whole_word(source: UebMoveSource, cells: &[u8]) {
    record_whole_word_at(source, cells, None);
}

pub(super) fn record_whole_word_at(source: UebMoveSource, cells: &[u8], offset: Option<usize>) {
    let mut attempt = AttemptRecorder::new();
    attempt.push(
        crate::rules::trace::RuleId::ueb(source as usize),
        0,
        cells.len(),
    );
    attempt.finish_at(cells, offset);
}

pub(super) fn push_indicator(out: &mut Vec<u8>, source: UebMoveSource, cells: &[u8]) {
    let offset = Some(out.len());
    out.extend_from_slice(cells);
    push_record(source, cells, offset, AttributionRecord::Indicator);
}

pub(super) fn push_direct(out: &mut Vec<u8>, source: UebMoveSource, cells: &[u8]) {
    let offset = Some(out.len());
    out.extend_from_slice(cells);
    push_record(source, cells, offset, AttributionRecord::Direct);
}

/// [`push_direct`] for a buffer that is appended into the output later, where
/// the position here would not be the position the cells end up at.
pub(super) fn push_direct_unplaced(out: &mut Vec<u8>, source: UebMoveSource, cells: &[u8]) {
    out.extend_from_slice(cells);
    push_record(source, cells, None, AttributionRecord::Direct);
}

pub(super) fn record_direct(source: UebMoveSource, cells: &[u8], offset: usize) {
    push_record(source, cells, Some(offset), AttributionRecord::Direct);
}

fn push_record(
    source: UebMoveSource,
    cells: &[u8],
    offset: Option<usize>,
    wrap: fn(NonWordAttempt) -> AttributionRecord,
) {
    ATTRIBUTIONS.with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut()
            && let Some(records) = slot.as_mut()
        {
            records.push(wrap(NonWordAttempt {
                cells: cells.to_vec(),
                rule: crate::rules::trace::RuleId::ueb(source as usize),
                offset,
            }));
        }
    });
}

static UEB_NON_RULE_METAS: [crate::rules::RuleMeta; UEB_RESERVED_SLOTS] = [
    crate::rules::RuleMeta {
        section: "10.9",
        subsection: None,
        name: "ueb_shortform",
        standard_ref: "UEB 2024 §10.9",
        description: "Shortform standing for a longer word",
    },
    crate::rules::RuleMeta {
        section: "13.2",
        subsection: Some("3"),
        name: "ueb_anglicised_contraction",
        standard_ref: "UEB 2024 §13.2.3",
        description: "Contraction in an anglicised or borrowed word",
    },
    crate::rules::RuleMeta {
        section: "4.1",
        subsection: None,
        name: "ueb_letter",
        standard_ref: "UEB 2024 §4.1 / §4.2",
        description: "Uncontracted letter, with an accent indicator where needed",
    },
    crate::rules::RuleMeta {
        section: "10.1",
        subsection: None,
        name: "ueb_alphabetic_wordsign",
        standard_ref: "UEB 2024 §10.1",
        description: "Single letter standing for a whole word",
    },
    crate::rules::RuleMeta {
        section: "10.2",
        subsection: None,
        name: "ueb_strong_wordsign",
        standard_ref: "UEB 2024 §10.2",
        description: "Strong groupsign cell standing for a whole word",
    },
    crate::rules::RuleMeta {
        section: "10.5",
        subsection: None,
        name: "ueb_lower_wordsign",
        standard_ref: "UEB 2024 §10.5",
        description: "Lower-cell sign standing for a whole word",
    },
    crate::rules::RuleMeta {
        section: "6",
        subsection: None,
        name: "ueb_numeric",
        standard_ref: "UEB 2024 §6",
        description: "Numeric indicator and the digits that follow it",
    },
    crate::rules::RuleMeta {
        section: "3",
        subsection: None,
        name: "ueb_symbol",
        standard_ref: "UEB 2024 §3",
        description: "General symbol such as percent, ampersand or asterisk",
    },
    crate::rules::RuleMeta {
        section: "5",
        subsection: None,
        name: "ueb_grade1_indicator",
        standard_ref: "RUEB 2024 §5",
        description: "Grade-1 indicator establishing grade-1 mode",
    },
    crate::rules::RuleMeta {
        section: "8.3",
        subsection: None,
        name: "ueb_capital_letter_indicator",
        standard_ref: "RUEB 2024 §8.3",
        description: "Capital indicator applying to the following letter",
    },
    crate::rules::RuleMeta {
        section: "8.4",
        subsection: None,
        name: "ueb_capitalised_word_indicator",
        standard_ref: "RUEB 2024 §8.4",
        description: "Capital indicators applying to the following word",
    },
    crate::rules::RuleMeta {
        section: "14.6.2",
        subsection: None,
        name: "ueb_inline_nemeth_code",
        standard_ref: "RUEB 2024 §14.6.2",
        description: "Nemeth Code within UEB text",
    },
];

/// Metadata of every UEB move source, in [`crate::rules::trace::RuleId`] order:
/// the reserved non-rule slots first, then the contraction rules.
pub(crate) fn ueb_rule_registry() -> Vec<&'static crate::rules::RuleMeta> {
    let mut metas: Vec<&'static crate::rules::RuleMeta> = UEB_NON_RULE_METAS.iter().collect();
    metas.extend(EnglishUebEngine::new().contraction_rule_metas());
    metas
}

/// Attempt to encode `text` as standalone UEB Grade-2. Returns `None` if the
/// input is empty or contains a construct the engine does not yet support, so
/// the caller can fall back to the legacy encoding path.
pub fn try_encode(text: &str) -> Option<Vec<u8>> {
    // The math pipeline NFD-decomposes accented Latin (제65항 combining marks),
    // turning `é` into `e`+◌́. UEB owns accents as precomposed letters (§4.2,
    // [`rule_4`]), so recompose (NFC) here to undo that split for the English
    // path only. Pure ASCII is NFC-stable, so non-accented inputs are unchanged.
    use unicode_normalization::UnicodeNormalization;
    let composed: String = text.nfc().collect();
    if !is_ueb_eligible(&composed) {
        return None;
    }
    // Content-routing is normally not an explicit English declaration (`x` remains
    // bare), but the source syntaxes below are themselves English-document markup:
    // §14.6 inline Nemeth spans, §9 styled print words, quoted all-caps fragments,
    // and §6 telephone groups need the same document-level handling as forced English.
    encode_english(&composed, content_route_uses_document_english(&composed))
}

/// Encode `text` through the UEB engine WITHOUT the [`is_ueb_eligible`] content
/// heuristic. For an EXPLICIT `EncodingMode::English` the caller has already
/// committed to English, so a letterless fragment like `4:30` (`⠼⠙⠒⠼⠉⠚`) — which
/// the heuristic would defer to the Korean path — is encoded too; and an isolated
/// single wordsign letter takes the §2.6/§10.12.2 grade-1 indicator (`x`→⠰⠭).
pub fn encode_forced(text: &str) -> Option<Vec<u8>> {
    encode_english(text, true)
}

/// Run the UEB engine over `text` (no eligibility heuristic). `explicit_english`
/// is true only when the caller declared English mode via [`encode_forced`]; it
/// threads down to §5.7.1 so an isolated single letter is grade-1-indicated under
/// an explicit `context: english` but bare under default content-routing.
fn encode_english(text: &str, explicit_english: bool) -> Option<Vec<u8>> {
    use unicode_normalization::UnicodeNormalization;
    let composed: String = text.nfc().collect();
    if let Some(cells) = encode_struck_ligature_text(&composed) {
        return Some(cells);
    }
    if let Some(cells) = encode_single_caron_word(&composed, explicit_english) {
        return Some(cells);
    }
    // §14.3.3: a whole input that is exactly a printed language-name row emits its
    // non-UEB passage identifier (`Afrikaans` → ⠐⠷⠁⠋⠄). Whole-input match only, so
    // a sentence merely *containing* "French" still encodes as ordinary English.
    if let Some(cells) = rule_14::table_language_identifier(&composed) {
        return Some(cells);
    }
    // §14.3.1/14.3.2: non-UEB (Arabic/Greek/IPA/music) runs inside English prose
    // take the non-UEB word/passage indicators, with the surrounding English
    // encoded by the closure. Returns None when no code-switch span is present.
    let attribution_checkpoint = attribution_checkpoint();
    if let Some(cells) = rule_14::encode_with_code_switches(&composed, |segment| {
        let tokens = parser::parse_english(segment);
        if tokens.is_empty() {
            Some(Vec::new())
        } else {
            EnglishUebEngine::new().encode(&tokens, explicit_english)
        }
    }) {
        return Some(cells);
    }
    rollback_attributions(attribution_checkpoint);
    let tokens = parser::parse_english(&composed);
    if tokens.is_empty() {
        return None;
    }
    EnglishUebEngine::new().encode(&tokens, explicit_english)
}

fn content_route_uses_document_english(text: &str) -> bool {
    has_inline_dollar_math_in_prose(text)
        || parenthesized_digit_group_before_number(&text.chars().collect::<Vec<_>>())
}

/// UEB 2024 §4.3.1/§4.3.4: adjacent letters marked by a stroke overlay are joined
/// by the ligature indicator; any modifier for either joined letter remains
/// immediately before the letter to which it applies.
fn encode_struck_ligature_text(text: &str) -> Option<Vec<u8>> {
    if !text.contains('\u{0336}') {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_ligature_letter(chars[i]) && chars.get(i + 1) == Some(&'\u{0336}') {
            let second = *chars.get(i + 2)?;
            if !is_ligature_letter(second) || chars.get(i + 3) != Some(&'\u{0336}') {
                return None;
            }
            push_rule4_letter(chars[i], &mut out)?;
            if second.is_uppercase() {
                out.push(crate::unicode::decode_unicode('⠠'));
            }
            out.extend([
                crate::unicode::decode_unicode('⠘'),
                crate::unicode::decode_unicode('⠖'),
            ]);
            push_rule4_letter_without_leading_cap(second, &mut out)?;
            i += 4;
            continue;
        }
        match chars[i] {
            ' ' => out.push(0),
            '?' => out.push(crate::unicode::decode_unicode('⠦')),
            c if is_ligature_letter(c) => push_rule4_letter(c, &mut out)?,
            _ => return None,
        }
        i += 1;
    }
    Some(out)
}

/// UEB 2024 §4.2.1/§4.2.4: a single modified word whose distinctive print
/// evidence is a caron uses the UEB modifier before the affected base letter, and
/// the modified letter is not part of a contraction.
fn encode_single_caron_word(text: &str, _explicit_english: bool) -> Option<Vec<u8>> {
    let has_caron = text
        .chars()
        .any(|c| matches!(c, 'č' | 'Č' | 'ě' | 'Ě' | 'ř' | 'Ř' | 'š' | 'Š' | 'ž' | 'Ž'));
    if !has_caron || !text.chars().all(is_ligature_letter) {
        return None;
    }
    let mut out = Vec::new();
    for c in text.chars() {
        push_rule4_letter(c, &mut out)?;
    }
    Some(out)
}

fn is_ligature_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || rule_4::is_modified_letter(c)
}

fn push_rule4_letter(c: char, out: &mut Vec<u8>) -> Option<()> {
    if let Some(cells) = rule_4::accent_cells(c) {
        out.extend(cells);
    } else {
        if c.is_uppercase() {
            out.push(crate::unicode::decode_unicode('⠠'));
        }
        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
    }
    Some(())
}

fn push_rule4_letter_without_leading_cap(c: char, out: &mut Vec<u8>) -> Option<()> {
    if let Some(cells) = rule_4::accent_cells(c) {
        let cells =
            if c.is_uppercase() && cells.first() == Some(&crate::unicode::decode_unicode('⠠')) {
                &cells[1..]
            } else {
                cells.as_slice()
            };
        out.extend(cells);
    } else {
        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
    }
    Some(())
}

/// Whether `text` carries content the UEB path may own. The legacy/math path
/// keeps every input with no such signal — in particular a lone accented letter
/// (`ã`, `ä`) is an ambiguous standalone diacritic owned by 수학 제65항, and a
/// pure number/symbol run (`7:30`, `9-10`) is legacy numeric.
///
/// Two signals qualify: (1) an ASCII letter — UEB §4.2 accents only arise inside
/// an alphabetic word (`café`), which always has one; and (2) a §9 typeform
/// signal — a Mathematical-Alphanumeric styled char or a combining underline
/// (U+0332) — so a *letterless* but emphasised input (`3̲4̲`, `27.̲9`, `83%̲`) is
/// still UEB's. Korean is excluded by the callers' own `is_korean_char` guard.
pub fn is_ueb_eligible(text: &str) -> bool {
    text.chars().any(|c| {
        c.is_ascii_alphabetic()
            || c == '\u{0332}'
            || rule_9::decode_styled(c).is_some()
            // §9: a small-cap styled letter is a §9 typeform signal even in a
            // letterless-looking run, so route it to the UEB typeform path.
            || rule_9::decode_small_cap(c).is_some()
            // §3.10 currency: the regular-width cent/pound/yen signs are UEB's. The
            // Korean 제65항 currency rule owns the *fullwidth* forms (`￠`/`￡`/`￥`)
            // and the shared `$`/`€`/`₣`, so only these three regular code points
            // are unambiguously English and safe to route here.
            || matches!(c, '\u{00A2}' | '\u{00A3}' | '\u{00A5}')
            // §3.8 copyright/registered/trademark and §3.26 transcriber-defined
            // signs (`©`/`®`/`™`/`✓`/`฿`/`❀`) are English-exclusive, so a
            // letterless example such as `©2009` routes through UEB. Korean 제65항
            // owns the shared per-mille `‰`, which is excluded.
            || matches!(
                c,
                '\u{00A9}' | '\u{00AE}' | '\u{2122}' | '\u{0E3F}' | '\u{2713}' | '\u{2740}'
            )
    })
    // §3.10 example uses shared euro/franc currency symbols in an exchange-rate
    // expression (`1 € = 6.55957₣`). A bare amount like `€75` remains legacy/Korean
    // owned, but an equation with digits and a spaced comparison sign is UEB prose.
    || (text.chars().any(|c| matches!(c, '\u{20AC}' | '\u{20A3}'))
        && text.chars().any(|c| c.is_ascii_digit())
        && text.contains(" = "))
    || {
        let chars: Vec<char> = text.chars().collect();
        // §16.2: a run of two or more adjacent box-drawing characters is a
        // horizontal line. A lone box char (a single mathematical `≡` or `─`)
        // is left to the legacy/math path so its non-line meaning is preserved.
        chars
            .windows(2)
            .any(|w| rule_16::is_line_char(w[0]) && rule_16::is_line_char(w[1]))
            // §3.15.1: a straight apostrophe/double-quote immediately after a
            // digit is a foot/inch sign in an English measurement (`4' 11"`);
            // this letterless shape routes to UEB, not the legacy quote path.
            || chars
                .windows(2)
                .any(|w| w[0].is_ascii_digit() && matches!(w[1], '\'' | '"'))
            // §6.6: a digit-space-digit run is a UEB numeric-space grouping
            // (`3 245 000` → ⠼⠉⠐⠃⠙⠑⠐⠚⠚⠚), not repeated Korean number signs.
            || chars
                .windows(3)
                .any(|w| w[0].is_ascii_digit() && w[1] == ' ' && w[2].is_ascii_digit())
            // §3.10: a `$`-space-digit currency amount inside a phrase
            // (`$2bn (2 billion dollars)`) is UEB currency prose.
            || chars
                .windows(3)
                .any(|w| w[0] == '$' && w[1] == ' ' && w[2].is_ascii_digit())
            // §6 numeric prose: a parenthesized area code followed by another digit
            // group is a telephone number, not a mathematical parenthetical.
            || parenthesized_digit_group_before_number(&chars)
    }
}

fn parenthesized_digit_group_before_number(chars: &[char]) -> bool {
    matches!(chars.first(), Some('('))
        && chars.iter().position(|c| *c == ')').is_some_and(|close| {
            close > 1
                && chars[1..close].iter().all(|c| c.is_ascii_digit())
                && matches!(chars.get(close + 1), Some(' '))
                && chars.get(close + 2).is_some_and(|c| c.is_ascii_digit())
        })
}

/// Whether `text` is *unambiguously* a math expression the legacy math pipeline
/// owns, so the UEB dispatch must NOT intercept it (Phase 7 preflight).
///
/// This is deliberately narrow: it keys only on hard math signals and never on
/// punctuation that also appears in English prose (`-`, `(`, `,`, `.`). The two
/// signals are (1) a known trig/log **function name** prefix (`sin`, `log2`,
/// `2cosx`), and (2) a single space-free token that **adjacently mixes ASCII
/// letters and digits** (`3ab`, `sin3x`, `f(x1)`) — a shape English words never
/// take. Hyphenated or multi-word English (`child-ish-ly`, `9-in dia.`) is left
/// to UEB.
pub fn is_math_owned(text: &str) -> bool {
    // (0) a balanced `$…$` span is LaTeX math, owned by the math engine. A lone
    // or trailing `$` (currency: `$6`, `US$`) is NOT, and is handled by §3.10.
    if text.len() >= 2 && text.starts_with('$') && text.ends_with('$') {
        return true;
    }
    // UEB §14.6 prose may contain embedded `$...$` Nemeth fragments. Once there is
    // ordinary prose outside the dollar span, tight `=`/`(`/`)` inside the fragment
    // is owned by the UEB code-switch encoder, not by default Korean/math routing.
    if has_inline_dollar_math_in_prose(text) {
        return false;
    }

    // (0a) §11 math/logic symbols — circled plus ⊕, the double-arrow implications
    // ⇒/↔ — are math by default wherever they appear (`a ⊕ b`, `p ⇒ q`). The UEB
    // §11 transcription of the bare glyph declares `context: english`, which routes
    // through `encode_forced` and bypasses this guard, so the english test cases
    // still reach §3; only content-routed (mode-less) expressions are kept on math.
    if text.chars().any(|c| {
        matches!(
            c,
            '\u{2295}'
                | '\u{21D2}'
                | '\u{2194}'
                | '→'
                | '←'
                | '↗'
                | '↘'
                | '↑'
                | '↓'
                | '△'
                | '□'
                | '′'
                | '″'
                | '|'
                | '‖'
                | '\u{0304}'
                | '\u{0302}'
        )
    }) {
        let chars: Vec<char> = text.chars().collect();
        let has_line_run = chars
            .windows(2)
            .any(|w| rule_16::is_line_char(w[0]) && rule_16::is_line_char(w[1]));
        let mut run = 0usize;
        let mut longest_lower_run = 0usize;
        for c in text.chars() {
            if c.is_ascii_lowercase() {
                run += 1;
                longest_lower_run = longest_lower_run.max(run);
            } else {
                run = 0;
            }
        }
        if !has_line_run && longest_lower_run < 3 {
            return true;
        }
    }

    // (0b) a comparison/equation operator (`=`, `<`, `>`) bound *tightly* to an
    // operand — no space on at least one side — is a math relation the legacy
    // math pipeline owns (`ax=b`, `a>b`, `x<0`, `y=f(x)`, `A={2, 4, …}`). English
    // prose always spaces these operators (`2 + 2 = 4`, `positron < posi`), so a
    // space on *both* sides leaves the input to the UEB §3.17 signs of comparison.
    let cells: Vec<char> = text.chars().collect();
    if cells.iter().enumerate().any(|(i, &c)| {
        matches!(c, '=' | '<' | '>')
            && !is_angle_bracket_prose(&cells, i)
            && (i.checked_sub(1).is_some_and(|j| cells[j] != ' ')
                || cells.get(i + 1).is_some_and(|&n| n != ' '))
    }) {
        return true;
    }

    // (0c) §3.24: a single space-free token carrying a Unicode super/subscript is a
    // math expression (`c²`, `x₂`, `³√x`, `log₂(x+1)`) — the same code points take
    // the 제18/19항 point shape there, not the UEB §3.24 indicator. Only a
    // multi-word English prose usage (`vitamin B₁₂`, `3 yd³`) reaches the UEB path.
    if !text.contains(' ')
        && text.chars().any(rule_3_24::is_script_char)
        && longest_ascii_letter_run(text) < 3
    {
        return true;
    }

    // (1) function-name expression: a trig/log name, optionally with a leading
    // coefficient (`2cosx`), where the name is NOT merely the prefix of a longer
    // English word (`singe` starts with `sin`, `arccosine` with `arccos`). The
    // character after the function name must be a non-letter (digit, `(`, end) —
    // a math argument, never a continuation of an English word.
    let after_coeff = text.trim_start_matches(|c: char| c.is_ascii_digit());
    if let Some((name, _)) = crate::rules::math::function::match_function_prefix(after_coeff) {
        let rest = &after_coeff[name.len()..];
        // The remainder is a math argument — never an English word continuation —
        // when it is empty, begins with a non-letter (`sin3x`, `f(`), begins with
        // an *uppercase* letter (a math variable, `arcsinA`, `cosX` — an English
        // word never has a mid-word capital after a function prefix), or is a run
        // of *vowel-free* letters (math variables `x`, `xy`). An English word
        // sharing the prefix (`singe`=sin+ge, `arccosine`=arccos+ine) always has a
        // lowercase, vowel-bearing continuation, so it is left to UEB.
        let rest_is_math_arg = rest.is_empty()
            || !rest.starts_with(|c: char| c.is_ascii_alphabetic())
            || rest.starts_with(|c: char| c.is_ascii_uppercase())
            || (!rest.contains(' ') && rest.chars().any(rule_3_24::is_script_char))
            || rest.chars().all(|c| {
                c.is_ascii_alphabetic()
                    && !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
            });
        if rest_is_math_arg {
            return true;
        }
    }

    // (1b) a digit enclosed in parentheses is bracketed math only when the bracket
    // attaches to adjacent math syntax: an alnum before `(` (`f(x-1)`, `7(2)`) or
    // a non-space after `)` (`(3n)!`). UEB §6 phone groups and §3.10 currency prose
    // use parenthesized digit groups without that math adjacency, and `$` inside the
    // brackets is currency rather than a math operand.
    if let Some(open) = text.find('(') {
        let inner = &text[open + 1..];
        let close = inner.find(')').unwrap_or(inner.len());
        let bracketed = &inner[..close];
        let before_math = text[..open]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        let after_math = inner
            .get(close + ')'.len_utf8()..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|c| !c.is_whitespace());
        if bracketed.chars().any(|c| c.is_ascii_digit())
            && !bracketed.contains(' ')
            && !bracketed.contains('$')
            && (before_math || after_math)
        {
            return true;
        }
    }

    // (2) a single space-free token where a digit is immediately followed by
    // *two or more* letters that are NOT an ordinal suffix — an implied product
    // of variables (`3ab`, `sin3x`). A single trailing letter (`3b`, `2d`,
    // `99c`) is a unit and an ordinal (`2nd`, `3rd`, `1st`, `4th`) is English;
    // both are encoded correctly by UEB, so they are NOT blocked.
    if text.contains("://") || text.contains('\\') {
        return true;
    }
    if text.contains(' ')
        || text.contains('-')
        || text.contains('@')
        || longest_ascii_letter_run(text) >= 4
    {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    let is_ordinal = ["st", "nd", "rd", "th"].iter().any(|suf| {
        lower.ends_with(suf) && lower[..lower.len() - 2].chars().all(|c| c.is_ascii_digit())
    });
    if is_ordinal {
        return false;
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1].is_ascii_alphabetic() && w[2].is_ascii_alphabetic())
}

fn longest_ascii_letter_run(text: &str) -> usize {
    let mut run = 0usize;
    let mut longest = 0usize;
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    longest
}

fn has_inline_dollar_math_in_prose(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.matches('$').count() == 2 {
        return false;
    }
    let mut in_span = false;
    let mut has_span = false;
    let mut outside_letters = 0usize;
    for c in text.chars() {
        if c == '$' {
            if in_span {
                has_span = true;
            }
            in_span = !in_span;
        } else if !in_span && c.is_ascii_alphabetic() {
            outside_letters += 1;
        }
    }
    has_span && outside_letters >= 3
}

/// UEB §3.17 signs of comparison are math-owned only when used as relations.
/// Email/listing delimiters and prose angle-bracket insertions (`<in file>`,
/// `<x, y>`, `Name <user@example.net>`) are paired punctuation, not comparison
/// operators, so tight `<`/`>` in those spans must not block UEB routing.
fn is_angle_bracket_prose(chars: &[char], index: usize) -> bool {
    match chars[index] {
        '<' => {
            let starts_after_boundary = index == 0 || chars[index - 1].is_whitespace();
            starts_after_boundary && matching_prose_angle_close(chars, index).is_some()
        }
        '>' => chars[..index]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| **c == '<')
            .is_some_and(|(open, _)| is_angle_bracket_prose(chars, open)),
        _ => false,
    }
}

fn matching_prose_angle_close(chars: &[char], open: usize) -> Option<usize> {
    let close = chars
        .iter()
        .enumerate()
        .skip(open + 1)
        .find_map(|(i, c)| if *c == '>' { Some(i) } else { None })?;
    let before_close = chars.get(close.checked_sub(1)?).copied()?;
    let after_close = chars.get(close + 1).copied();
    let closes_before_boundary =
        after_close.is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation());
    if before_close != '<' && closes_before_boundary {
        Some(close)
    } else {
        None
    }
}

#[cfg(test)]
mod is_math_owned_tests {
    use super::{
        encode_english, encode_struck_ligature_text, has_inline_dollar_math_in_prose, is_math_owned,
    };
    use crate::unicode::decode_unicode;

    /// Inputs the legacy math engine owns — UEB must NOT intercept these.
    #[rstest::rstest]
    #[case::sin("sin")]
    #[case::cos("cos")]
    #[case::sinh("sinh")]
    #[case::log2("log2")]
    #[case::two_log7("2log7")]
    #[case::sin3x("sin3x")]
    #[case::sinxy("sinxy")] // function + vowel-free variable run
    #[case::two_cosx("2cosx")] // leading coefficient
    #[case::three_ab("3ab")] // digit + 2 variables (implied product)
    #[case::f_paren("f(x-1)")] // digit inside parentheses
    #[case::factorial("(3n)!")]
    // §3.17: a comparison/equation operator bound tightly (no surrounding space)
    // is a math relation, not English prose.
    #[case::eq_relation("ax=b")]
    #[case::gt_relation("a>b")]
    #[case::lt_relation("x<0")]
    #[case::eq_func("y=f(x)")]
    #[case::set_eq("A={2, 4, 6, ...}")] // tight `=` even though spaces follow
    #[case::interval("-1<x<3")]
    #[case::vars_equal("VarsEqual=(x==y);")]
    // §3.24: a single space-free token with a Unicode super/subscript is math.
    #[case::script_c_squared("c\u{00B2}")]
    #[case::script_x_sub2("x\u{2082}")]
    #[case::script_chemical("H\u{2082}O")]
    #[case::script_unit("4m\u{00B2}")]
    #[case::script_cube_root("\u{00B3}\u{221A}x\u{00B3}")]
    #[case::script_log_sub2("log\u{2082}(x+1)")]
    // Shared math glyphs with no prose-length lowercase word are math-owned.
    #[case::spaced_right_arrow("p → q")]
    #[case::prime("x′")]
    #[case::absolute_value("|x|")]
    #[case::triangle_name("△ABC")]
    #[case::combining_hat("p\u{0302}")]
    fn math_owned_inputs_are_blocked(#[case] text: &str) {
        assert!(is_math_owned(text), "{text:?} should be math-owned");
    }

    /// English (or unit/ordinal) inputs UEB owns — must NOT be blocked.
    #[rstest::rstest]
    #[case::singe("singe")] // English word sharing the `sin` prefix
    #[case::singeing("singeing")]
    #[case::arccosine("arccosine")] // shares `arccos`, has vowel continuation
    #[case::ordinal_2nd("2nd")]
    #[case::ordinal_3rd("3rd")]
    #[case::unit_3b("3b")] // single trailing letter — unit, not product
    #[case::cents_99c("99c")]
    #[case::hyphenated("child-ish-ly")]
    #[case::sentence("That is quite fair.")]
    #[case::plain_word("cat")]
    // §3.17 prose: operators with a space on BOTH sides are English signs of
    // comparison, not a math relation (`2 + 2 = 4`, `positron < posi`).
    #[case::spaced_eq("a = b")]
    #[case::spaced_lt("positron < posi")]
    #[case::spaced_sum("as easy as 2 + 2 = 4")]
    // §3.10: a parenthetical *phrase* (spaces inside `(...)`) is prose, even with a
    // digit — not a bracketed math argument. Currency amounts gloss this way.
    #[case::paren_phrase_billion("$2bn (2 billion dollars)")]
    #[case::paren_phrase_escudos("20$00 (20 escudos)")]
    #[case::paren_phrase_pounds("\u{00A3}3m (3 million pounds)")]
    #[case::paren_phrase_enough("Buy meat (enough for 2).")]
    #[case::script_footnote("knowledge.\u{00B3}")]
    #[case::script_word_subscripts("mass\u{209B}\u{1D64}\u{2099}")]
    #[case::angle_phrase("<in file>")]
    #[case::angle_variables("<x, y>")]
    #[case::angle_email("<J.Child@children.net>")]
    #[case::angle_email_after_name("Jan Swan <swanj@iafrica.com>")]
    #[case::phone_area("phone: (61) 3 1234 5678")]
    #[case::currency_paren("Balance:  ($52.68)")]
    #[case::phone_number("(416) 486-2500")]
    #[case::shopping4you("shopping4you")]
    #[case::address_digit_word("4starhotel@webnet.com")]
    #[case::inline_nemeth_prose("The result will be in the form $(ax+by)(cx+dy)$, where $ac=12$.")]
    fn english_inputs_are_not_blocked(#[case] text: &str) {
        assert!(!is_math_owned(text), "{text:?} should NOT be math-owned");
    }

    #[test]
    fn struck_ligature_handles_uppercase_second_letter() {
        assert_eq!(
            encode_struck_ligature_text("a\u{0336}B\u{0336}"),
            Some(vec![
                decode_unicode('⠁'),
                decode_unicode('⠠'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠃'),
            ])
        );
    }

    #[test]
    fn struck_ligature_rejects_unmarked_second_letter() {
        assert_eq!(encode_struck_ligature_text("a\u{0336}B"), None);
    }

    #[test]
    fn struck_ligature_keeps_capital_for_accented_second_letter() {
        assert_eq!(
            encode_struck_ligature_text("a\u{0336}É\u{0336}"),
            Some(vec![
                decode_unicode('⠁'),
                decode_unicode('⠠'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠘'),
                decode_unicode('⠌'),
                decode_unicode('⠑'),
            ])
        );
    }

    #[test]
    fn code_switch_closure_encodes_non_empty_english_segment() {
        let encoded = encode_english("word قُ", true).expect("Arabic code switch should encode");

        assert!(encoded.contains(&decode_unicode('⠺')));
        assert!(encoded.iter().any(|cell| *cell != 0));
    }

    #[test]
    fn plain_english_route_uses_engine_after_parsing_tokens() {
        let input = std::hint::black_box("cat");

        assert_eq!(
            encode_english(input, true),
            Some(vec![
                decode_unicode('⠉'),
                decode_unicode('⠁'),
                decode_unicode('⠞')
            ])
        );
    }

    #[test]
    fn full_dollar_span_is_not_inline_prose_math() {
        assert!(!has_inline_dollar_math_in_prose("$x+1$"));
    }
}

#[cfg(test)]
mod is_ueb_eligible_tests {
    use super::is_ueb_eligible;

    /// §3.10: regular-width cent/pound/yen are English-exclusive (Korean 제65항
    /// owns the fullwidth forms), so a letterless currency run routes to UEB.
    #[rstest::rstest]
    #[case::cent_amount("10\u{00A2}")]
    #[case::pound_amount("\u{00A3}24")]
    #[case::yen_amount("\u{00A5}360")]
    #[case::euro_franc_equation("1 \u{20AC} = 6.55957\u{20A3}")]
    #[case::ascii_letter("cat")]
    #[case::styled_digit("3\u{0332}4")] // §9 combining-underline typeform
    #[case::phone_number("(416) 486-2500")]
    fn ueb_owned_inputs_are_eligible(#[case] text: &str) {
        assert!(is_ueb_eligible(text), "{text:?} should be UEB-eligible");
    }

    /// The shared `$`/`€`/`₣` and the *fullwidth* `￠`/`￡`/`￥`/`￦` are owned by
    /// Korean 제65항 (`⠴⠈ + letter`); a letterless run of them must stay in the
    /// legacy path, so it must NOT become UEB-eligible.
    #[rstest::rstest]
    #[case::dollar("$50")]
    #[case::euro("\u{20AC}75")]
    #[case::franc("\u{20A3}1")]
    #[case::fullwidth_cent("25\u{FFE0}")]
    #[case::fullwidth_pound("\u{FFE1}88")]
    #[case::fullwidth_yen("\u{FFE5}1")]
    #[case::fullwidth_won("\u{FFE6}100")]
    fn korean_owned_currency_is_not_eligible(#[case] text: &str) {
        assert!(
            !is_ueb_eligible(text),
            "{text:?} must stay in the legacy path"
        );
    }
}

#[cfg(test)]
mod encode_pipeline_tests {
    use super::encode_forced;

    #[rstest::rstest]
    #[case::two_assignments("$P_{D}$＝1,000kN, $P_{L}$＝600kN", 12)]
    #[case::preceded_by_prose("abc $P_{D}$", 6)]
    #[case::followed_by_prose("$I_{7}H^{T}$(mod 2)", 12)]
    #[case::ethylene_equation("$C_{2}H_{4}$(g)＋$H_{2}O$(g)→$C_{2}H_{5}OH$(g)", 36)]
    #[case::carbon_monoxide_equation("$CO$(g)＋$H_{2}O$(g)→$CO_{2}$(g)＋$H_{2}$(g)", 27)]
    #[case::capitalised_spelled_word_after_span("abc $P_{D}$ Cat", 6)]
    fn inline_technical_cells_are_claimed_by_14_6_2(
        #[case] input: &str,
        #[case] expected_technical_cells: usize,
    ) {
        let (cells, trace) =
            crate::encode_with_trace(input).expect("inline technical input must encode");
        let untraced = crate::encode(input).expect("inline technical input must encode untraced");
        let technical_cells: usize = trace
            .events()
            .iter()
            .filter(|event| {
                event
                    .rule
                    .meta()
                    .is_some_and(|meta| meta.section == "14.6.2")
            })
            .map(|event| event.output.len())
            .sum();
        let mut claims = vec![0u8; cells.len()];
        for event in trace.events() {
            for index in event.output.clone() {
                claims[index as usize] += 1;
            }
        }

        assert_eq!(cells, untraced, "trace collection must not change output");
        assert!(
            claims.iter().all(|count| *count == 1),
            "claims={claims:?}, events={:?}",
            trace.events()
        );
        assert_eq!(
            technical_cells,
            expected_technical_cells,
            "events={:?}",
            trace.events()
        );
    }

    /// §8.8.2 gives a two-letter chemical symbol its capitals one at a time
    /// (`CCl`, `HCl`). Those indicators and letters are written straight into
    /// the output, so each must claim the cell it wrote.
    #[rstest::rstest]
    #[case::two_letter_symbols("SO<sub>2</sub>, CCl<sub>4</sub>, HCl, $SF_{6}$")]
    #[case::camel_subunit_word("aMgO")]
    #[case::camel_caps_word("dCO")]
    #[case::balanced_equation("aMgO(s)$+$bC(s)→cMg(s)$+$dCO(g)$+$eCO<sub>2</sub>(g)")]
    #[case::repeated_subscript_markup("CO<sub>2</sub>, SO<sub>2</sub>, CO<sub>2</sub>")]
    #[case::word_repeated_later_in_the_line("CO$+$H<sub>2</sub>O↔CO<sub>2</sub>$+$H<sub>2</sub>")]
    #[case::lone_capitals_between_inline_spans("A $1s^{2}2s^{2}2p^{5}$, B $1s^{2}2s^{2}2p^{2}$")]
    fn every_cell_of_a_chemical_line_names_a_rule(#[case] input: &str) {
        let (cells, trace) = crate::encode_with_trace(input).expect("input must encode");
        let untraced = crate::encode(input).expect("input must encode untraced");

        assert_eq!(cells, untraced, "trace collection must not change output");
        assert_eq!(
            trace.unattributed_cells(),
            0,
            "{} of {} cells name no rule",
            trace.unattributed_cells(),
            cells.len()
        );
    }

    /// An input that parses to zero tokens — the empty string, reached through
    /// the eligibility-free `encode_forced` entry — yields None rather than an
    /// empty cell vector.
    #[test]
    fn forced_empty_input_yields_none() {
        assert_eq!(encode_forced(""), None);
    }
}

#[cfg(test)]
mod indicator_clipping_tests {
    use super::push_without_indicators;
    use crate::rules::trace::{EmitterRule, RuleId};

    /// An indicator can land inside the cells a rule produced. The cells before
    /// it still belong to that rule, so they are recorded as their own span
    /// instead of being surrendered along with the indicator.
    #[test]
    fn a_span_interrupted_by_an_indicator_keeps_the_part_before_it() {
        let rule = RuleId::emitter(EmitterRule::WordSpace);
        let mut spans = Vec::new();

        push_without_indicators(&mut spans, (rule, 0..6), &[(rule, 2..4)]);

        assert_eq!(
            spans
                .iter()
                .map(|(_, range)| range.clone())
                .collect::<Vec<_>>(),
            vec![0..2, 4..6]
        );
    }
}
