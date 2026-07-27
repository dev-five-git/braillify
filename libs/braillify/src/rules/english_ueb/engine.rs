//! Document-level UEB Grade-2 engine.
//!
//! Walks the token stream, applies §8 capitalisation indicators, and delegates
//! intra-word contraction to the [`ContractionEngine`]. Returns `None` for any
//! construct not yet supported, so the caller can fall back to the legacy path
//! (this is what keeps the engine safe to grow rule-by-rule).

mod bibliography;
mod caps;
mod documents;
mod foreign;
mod listings;
mod passage;
mod quotes;
mod styled_methods;
mod styled_passage;
mod styled_sequences;
mod tokens;
mod word_methods;
mod words;

use super::contraction::ContractionEngine;
use super::rule_10_3::StrongContractionRule;
use super::standing_alone::{is_standing_alone, lower_wordsign_usable};
use super::token::EnglishToken;
use crate::unicode::decode_unicode;

use bibliography::*;
use caps::*;
use documents::*;
use foreign::*;
use listings::*;
use passage::*;
use quotes::*;
use styled_passage::*;
use styled_sequences::*;
use tokens::*;
use words::*;

include!("engine/encode_space.rs");
include!("engine/encode_word.rs");
include!("engine/encode_double_quote.rs");
include!("engine/encode_curly_single_quote.rs");
include!("engine/encode_straight_single_quote.rs");
include!("engine/encode_line.rs");
include!("engine/encode_script.rs");
include!("engine/encode_symbol.rs");
include!("engine/encode_styled.rs");

/// ⠠ dot-6 — UEB capital indicator (§8).
pub(super) const CAPITAL: u8 = decode_unicode('⠠');
/// ⠰ dots-5-6 — UEB grade-1 indicator (§5/§6.5).
pub(super) const GRADE1: u8 = decode_unicode('⠰');
/// ⠦ — opening double quotation mark (§7.6).
pub(super) const QUOTE_OPEN: u8 = decode_unicode('⠦');
/// ⠴ — closing double quotation mark (§7.6).
pub(super) const QUOTE_CLOSE: u8 = decode_unicode('⠴');
/// Braille space cell.
pub(super) const SPACE: u8 = 0;

type ForeignScope = Option<(super::rule_13::AccentCode, bool)>;
type ActiveTypeformPassage = (usize, super::token::Typeform, bool, ForeignScope);

/// Capitalisation pattern of a word (§8 subset currently supported).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Caps {
    /// All lowercase — no indicator.
    None,
    /// One leading capital, or a single capital letter — `⠠`.
    Single,
    /// Whole word uppercase (len ≥ 2) — `⠠⠠`.
    Word,
}

#[derive(Clone, Copy)]
struct CapsGroup {
    first_cap: usize,
    last_cap: usize,
    caps_sequences: usize,
    has_lower: bool,
    /// Whether the group is exactly one single-letter uppercase word — an
    /// article/initial `A`. §8.5.4 leaves such letters "not necessarily" part
    /// of the surrounding passage when preceded by sentence-terminal
    /// punctuation, so the passage detector can exclude them from the count.
    single_letter_only: bool,
    /// Whether the group ends with sentence-terminal punctuation (`.`, `!`,
    /// `?`) after the last capital — a hint that a following single-letter
    /// caps word (`A`) starts a new sentence.
    ended_with_terminal_sentence_mark: bool,
}

#[derive(Clone, Copy)]
struct Grade1Span {
    end: usize,
    needs_terminator: bool,
    indicator_cells: usize,
}

/// §7.6 role of a single-quote glyph: an opening/closing single *quotation* mark
/// (`⠠⠦`/`⠠⠴`) or an *apostrophe* (`⠄`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SingleQuote {
    Apostrophe,
    Open,
    Close,
}

struct WordContext {
    /// §2.6: the word stands alone, so §10.1/§10.2/§10.5 wordsigns may apply.
    standing_alone: bool,
    /// §10.1/§10.2: upper wordsigns are usable in this standing-alone context.
    upper_usable: bool,
    /// §10.9: a shortform abbreviation may be used for this word.
    shortform_usable: bool,
    /// §10.9.3: longer-word shortforms are available only in ordinary word text,
    /// not inside dot-delimited technical identifiers such as domain names.
    allow_longer_shortforms: bool,
    /// §10.5: the stricter lower-wordsign boundary is also satisfied.
    lower_usable: bool,
    /// §8.4: inside a caps passage — per-word capital indicators are suppressed.
    suppress_caps: bool,
    /// §10.4.3: this token begins a fresh word (after a space/hyphen/dash/edge),
    /// so a word-initial `ing` spells out as `in` (⠔) + `g`.
    word_initial: bool,
    /// §10.6.2: this token begins a word for restricted `be`/`con`/`dis`.
    #[allow(dead_code)]
    restricted_prefix_boundary: bool,
    /// §10.12.1: the word directly abuts a digit (`CH6`, `6CH`), so an all-caps run
    /// is an initialism "used as letters" and takes no contractions.
    digit_adjacent: bool,
}

struct StyledContext<'a> {
    tokens: &'a [EnglishToken],
    suppress_caps: bool,
    foreign_scope: Option<(super::rule_13::AccentCode, bool)>,
}

pub struct EnglishUebEngine {
    contractions: ContractionEngine,
}

impl Default for EnglishUebEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishUebEngine {
    /// Build the engine with the currently-implemented contraction rules.
    pub fn new() -> Self {
        let mut contractions = ContractionEngine::default();
        contractions.register(Box::new(StrongContractionRule));
        // §10.11: the bridge-aware strong groupsign suppresses `th`/`wh`/`sh`
        // that cross a compound boundary (hyphenation-detected).
        contractions.register(Box::new(super::rule_10_11::BridgeAwareStrongGroupsignRule));
        // §10.6.8: `en`/`in` are pronunciation-gated — suppressed where they
        // overlap a word-final `ness` whose `n` onsets the syllable (`busi·ness`,
        // `fi·ness·e`), kept where the `n` closes it (`citi·zen·ess`).
        contractions.register(Box::new(super::rule_10_6_8::EnInBeforeNessRule::new(
            Box::new(super::pronunciation::cmudict::CmuDictProvider::new()),
        )));
        contractions.register(Box::new(super::rule_10_7::InitialContractionRule));
        contractions.register(Box::new(super::rule_10_8::FinalGroupsignRule));
        // §10.6 restricted groupsigns (be/con/dis) judge the first syllable from
        // pronunciation/word-structure (CMUdict).
        contractions.register(Box::new(
            super::rule_10_6_restricted::RestrictedLowerGroupsignRule::new(Box::new(
                super::pronunciation::cmudict::CmuDictProvider::new(),
            )),
        ));
        // §10.6.5 middle lower groupsigns (ea/bb/cc/ff/gg) need the word list to
        // detect morpheme boundaries (pine|apple, dumb|bell).
        contractions.register(Box::new(
            super::rule_10_6_middle::MiddleLowerGroupsignRule::new(Box::new(
                super::pronunciation::cmudict::CmuDictProvider::new(),
            )),
        ));
        // §10.7 deferred initial-letter contractions (part/work/some/where/…) are
        // pronunciation-gated.
        contractions.register(Box::new(
            super::rule_10_7_pron::InitialContractionPronunciationRule::new(Box::new(
                super::pronunciation::cmudict::CmuDictProvider::new(),
            )),
        ));
        // §10.7/§10.11 structure-gated initial-letter contractions (`lord`, `work`):
        // applied only where the letters START a real word component.
        contractions.register(Box::new(
            super::rule_10_7_struct::StructuralInitialContractionRule::new(Box::new(
                super::pronunciation::cmudict::CmuDictProvider::new(),
            )),
        ));
        Self { contractions }
    }

    /// Encode a token stream. Returns `None` if any token is unsupported
    /// (a number, a symbol, or a mixed-case word), so the legacy path — which
    /// handles those — takes over. `explicit_english` is true only under an
    /// explicit `EncodingMode::English` (testcase `context: english`); it threads
    /// to §5.7.1 so an isolated single letter is grade-1-indicated only then.
    pub fn encode(&self, tokens: &[EnglishToken], explicit_english: bool) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut prev_was_number = false;
        // §6.3: numeric mode continues across a `,` or `.` that separates digits
        // (e.g. `5,70`, `4.2`), so the numeric indicator `⠼` is emitted only once.
        let mut numeric_mode = false;
        let mut quote_open = false;
        let mut internal_double_quote_open = false;
        let caret_note = contains_caret(tokens);
        let transcriber_note = contains_transcriber_note(tokens);
        // §9: index past a styled run already emitted as a word indicator, so its
        // member tokens are not re-emitted individually.
        let mut skip_to = 0usize;
        // §16.2: horizontal line mode continues through inline arrow symbols until a
        // space, terminator, or non-line graphic closes it.
        let mut line_mode_active = false;
        // §9.x active typeform passage: (end index exclusive, form, caps, foreign scope) where
        // `caps` marks a passage whose every styled word is all-caps (§8.4), so a
        // capitals passage ⠠⠠⠠ … ⠠⠄ nests inside the typeform ⠨⠶ … ⠨⠄. Its
        // terminator is emitted once the walk passes the styled span.
        let mut passage: Option<ActiveTypeformPassage> = None;
        let mut grade1_passage: Option<Grade1Span> = None;
        if caret_note {
            out.extend([decode_unicode('⠈'), decode_unicode('⠶')]);
        }
        // §8.4 capitals passage: ⠠⠠⠠ … ⠠⠄ around runs of 3+ all-caps words.
        let (cap_start, cap_term, in_passage) = caps_passages(tokens, explicit_english);
        // §7.6 single-quote vs apostrophe role per token (matched-pair analysis).
        let sq_roles = single_quote_roles(tokens);
        let escaped_code = escaped_quote_code_span(tokens);
        let (url_listing, regex_listing) = ascii_listing_spans(tokens);
        let doc_letters = document_letters(tokens);
        let foreign_code = super::rule_13::has_foreign_code_signal(&doc_letters);
        let spanish_foreign = super::rule_13::spanish_context(&doc_letters);
        let foreign_passage = !explicit_english
            && !bibliography_entry_context(tokens)
            && (super::rule_13::likely_foreign_passage(
                &document_prose_words(tokens),
                &doc_letters,
            ) || is_short_typeform_foreign_sentence(tokens));
        let scansion_stress_context = tokens
            .iter()
            .any(|t| matches!(t, EnglishToken::Symbol('/' | '|')))
            && doc_letters.iter().any(|c| {
                matches!(
                    c,
                    'ā' | 'ē' | 'ī' | 'ō' | 'ū' | 'ȳ' | 'ă' | 'ĕ' | 'ĭ' | 'ŏ' | 'ŭ'
                )
            });
        let early_english = !scansion_stress_context
            && (doc_letters
                .iter()
                .any(|c| matches!(c, 'þ' | 'Þ' | 'ð' | 'Ð' | 'ȝ' | 'Ȝ' | 'ƿ' | 'Ƿ'))
                || doc_letters
                    .iter()
                    .any(|c| matches!(c, 'ē' | 'ĕ' | 'ō' | 'ŏ' | 'ȳ')));
        if explicit_english
            && tokens
                .iter()
                .any(|t| matches!(t, EnglishToken::Symbol('×')))
            && tokens.iter().all(|t| {
                matches!(
                    t,
                    EnglishToken::Number(_)
                        | EnglishToken::Space
                        | EnglishToken::Symbol('.' | ',' | '×' | '<' | '>' | '=' | '+' | '-' | '−')
                )
            })
        {
            let chars = token_plain_chars(tokens);
            let mut cells = super::rule_11::encode_technical(&chars)?;
            if cells.starts_with(&[GRADE1, GRADE1, GRADE1])
                && cells.ends_with(&[GRADE1, decode_unicode('⠄')])
            {
                cells.drain(..3);
                let len = cells.len();
                cells.truncate(len - 2);
            }
            return Some(cells);
        }
        if tokens
            .iter()
            .any(|t| matches!(t, EnglishToken::Symbol('₀'..='₉')))
            && tokens
                .iter()
                .any(|t| matches!(t, EnglishToken::Symbol('+' | '→')))
        {
            let mut chars = Vec::new();
            for token in tokens {
                match token {
                    EnglishToken::Word(w)
                    | EnglishToken::Number(w)
                    | EnglishToken::Technical(w) => chars.extend(w),
                    EnglishToken::Symbol(c) => chars.push(*c),
                    EnglishToken::Space => chars.push(' '),
                    EnglishToken::LineBreak => chars.push('\n'),
                    EnglishToken::Styled(c, _) => chars.push(*c),
                    EnglishToken::WordDivision { chars: w, .. } => chars.extend(w),
                }
            }
            return super::rule_11::encode_chemical(&chars);
        }
        if let Some(cells) = encode_chemical_formula_scripts(tokens) {
            return Some(cells);
        }
        let drop_styled_typeform_for_code_switch = (foreign_code || spanish_foreign)
            && all_text_is_styled_or_punctuation(tokens)
            && styled_word_count(tokens) < 3;
        // §16.2/§16.3: spatial-layout tokens (box-drawing runs, guide dots, tabs,
        // multi-line diagrams) need the LineBreak preserved as a literal newline (255)
        // so the visual structure survives. A stray arrow (`↓`/`↑`) in prose (§11.6.1
        // step diagram: `step 1\n↓\nstep 2`) is not spatial layout — it uses ordinary
        // prose line breaks (single cell 0), so the trigger requires either two
        // adjacent box-drawing chars (a genuine line-mode run), a tab, or a leading
        // whitespace column (spatial indentation).
        let chars: Vec<char> = tokens
            .iter()
            .filter_map(|t| {
                if let EnglishToken::Symbol(c) = t {
                    Some(*c)
                } else {
                    None
                }
            })
            .collect();
        if tokens.len() >= 2
            && tokens
                .iter()
                .all(|t| matches!(t, EnglishToken::Symbol('-')))
        {
            let mut cells = Vec::with_capacity(chars.len() + 2);
            cells.push(decode_unicode('⠐'));
            for _ in 0..=chars.len() {
                cells.push(decode_unicode('⠒'));
            }
            return Some(cells);
        }
        if let Some(cells) = encode_rule_3_14_punctuation_box(tokens) {
            return Some(cells);
        }
        if let Some(cells) = encode_rule_3_14_letter_grid(tokens) {
            return Some(cells);
        }
        if let Some(cells) = encode_compact_spatial_example(tokens) {
            return Some(cells);
        }
        let has_box_run = chars
            .windows(2)
            .any(|w| super::rule_16::is_line_char(w[0]) && super::rule_16::is_line_char(w[1]));
        let has_spatial_segment = chars.iter().any(|c| super::rule_16::is_spatial_segment(*c));
        let has_line_only_layout = has_box_run && !has_spatial_segment;
        let has_tab = tokens
            .iter()
            .any(|t| matches!(t, EnglishToken::Symbol('\t')));
        let preserve_spatial_newlines = has_spatial_segment || has_tab;
        let flatten_line_layout = has_line_only_layout && !has_tab;
        let spatial_grade1_passage = preserve_spatial_newlines
            && tokens
                .iter()
                .any(|token| matches!(token, EnglishToken::LineBreak))
            && needs_spatial_grade1_passage(tokens);
        // §11.6.1 arrow-only diagram: a bare arrow between two LineBreaks with no
        // surrounding box frame is a step diagram whose `\n` collapses to one cell.
        let arrow_diagram_context = !preserve_spatial_newlines
            && tokens.windows(3).any(|w| {
                matches!(w[0], EnglishToken::LineBreak)
                    && matches!(
                        w[1],
                        EnglishToken::Symbol('↓' | '↑' | '→' | '←' | '⇒' | '↔')
                    )
                    && matches!(w[2], EnglishToken::LineBreak)
            });
        // §15.2.1 scansion diagram (`Diagram of poetic metre:\n. - . - / . .`) —
        // the notation line consists only of `.`, `-`, `/`, and spaces, so any
        // LineBreak feeding directly into that line collapses to a single cell
        // separator instead of the §10.13 two-cell end-of-line space. Skip in
        // Korean-embedded inputs where numbers separated by punctuation could
        // false-trigger the scan.
        let scansion_diagram_context = !preserve_spatial_newlines
            && tokens.windows(2).any(|w| {
                matches!(w[0], EnglishToken::LineBreak)
                    && matches!(w[1], EnglishToken::Symbol('.' | '-' | '/'))
            });
        let poem_linear_context = poem_linear_context(tokens);

        // §12.4/§7 prose spacing: a stray double space between prose words (a
        // typo like `not:  For`) collapses to one cell. Column-aligned data
        // (§16.5 tables, or any input carrying a 3+ space run anywhere) opts
        // out — 2+ space runs there mark fixed-width columns and must survive.
        let has_wide_space_run = {
            let mut run = 0usize;
            let mut max = 0usize;
            for t in tokens {
                if matches!(t, EnglishToken::Space) {
                    run += 1;
                    if run > max {
                        max = run;
                    }
                } else {
                    run = 0;
                }
            }
            max >= 3
        };
        let collapse_prose_double_space =
            !preserve_spatial_newlines && (!has_wide_space_run || transcriber_note);

        let mut skip_flattened_line_indent = false;
        let mut numeric_separator_count = 0usize;
        let mut nested_inner_passage: Option<(usize, super::token::Typeform)> = None;
        if spatial_grade1_passage {
            out.extend([
                decode_unicode('⠐'),
                decode_unicode('⠐'),
                decode_unicode('⠿'),
                GRADE1,
                GRADE1,
                GRADE1,
                255,
            ]);
        }
        for i in 0..tokens.len() {
            if let Some((end, form)) = nested_inner_passage
                && i >= end
            {
                out.extend(super::rule_9::terminator(form));
                nested_inner_passage = None;
            }
            if let Some((end, form, caps, _)) = passage
                && i >= end
            {
                // §8.4: close the nested capitals passage before the typeform one.
                if caps {
                    out.extend([CAPITAL, decode_unicode('⠄')]);
                }
                out.extend(super::rule_9::terminator(form));
                passage = None;
            }
            if i < skip_to {
                continue;
            }
            if let Some(span) = grade1_passage
                && i >= span.end
            {
                if span.needs_terminator {
                    out.extend([GRADE1, decode_unicode('⠄')]);
                }
                grade1_passage = None;
            }
            if !in_passage[i]
                && let Some(first) = tokens.get(i).and_then(uppercase_greek_symbol)
                && tokens.get(i + 1).is_some_and(|token| {
                    uppercase_greek_symbol(token).is_some()
                        || uppercase_greek_chars(token).is_some()
                })
            {
                out.extend([CAPITAL, CAPITAL]);
                out.extend(greek_letter_cells_with_caps(first, true)?);
                let mut j = i + 1;
                for cells in tokens.iter().skip(j).map_while(uppercase_greek_token_cells) {
                    out.extend(cells);
                    j += 1;
                }
                skip_to = j;
                prev_was_number = false;
                numeric_mode = false;
                continue;
            }
            if !in_passage[i]
                && let Some(chars) = tokens.get(i).and_then(uppercase_greek_chars)
                && (chars.len() >= 2
                    || tokens.get(i + 1).is_some_and(|token| {
                        uppercase_greek_symbol(token).is_some()
                            || uppercase_greek_chars(token).is_some()
                    }))
            {
                out.extend([CAPITAL, CAPITAL]);
                for &c in chars {
                    out.extend(greek_letter_cells_with_caps(c, true)?);
                }
                let mut j = i + 1;
                for cells in tokens.iter().skip(j).map_while(uppercase_greek_token_cells) {
                    out.extend(cells);
                    j += 1;
                }
                skip_to = j;
                prev_was_number = false;
                numeric_mode = false;
                continue;
            }
            let cap_start_grade1 = !spatial_grade1_passage
                && cap_start[i]
                && super::rule_5_7::needs_grade1_indicator(tokens, i, explicit_english);
            if cap_start_grade1 {
                out.push(GRADE1);
            }
            if !spatial_grade1_passage && cap_start[i] {
                out.extend([CAPITAL, CAPITAL, CAPITAL]);
            }
            match &tokens[i] {
                EnglishToken::Space => {
                    encode_space_arm!(tokens, out, prev_was_number, numeric_mode, skip_to, line_mode_active, preserve_spatial_newlines, flatten_line_layout, spatial_grade1_passage, poem_linear_context, collapse_prose_double_space, skip_flattened_line_indent, numeric_separator_count, i)
                }
                EnglishToken::Number(digits) => {
                    skip_flattened_line_indent = false;
                    line_mode_active = false;
                    if numeric_mode {
                        // §6.3: already in numeric mode (digit-separator `,`/`.`
                        // bridged us here) — emit digits only, no second `⠼`.
                        for d in digits {
                            out.push(super::rule_6::digit_cell(*d)?);
                        }
                    } else {
                        out.extend(super::rule_6::encode_number(digits)?);
                        numeric_separator_count = 0;
                    }
                    prev_was_number = true;
                    numeric_mode = true;
                }
                EnglishToken::Technical(chars) => {
                    skip_flattened_line_indent = false;
                    out.extend(super::rule_11::encode_technical(chars)?);
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::Word(chars) => {
                    encode_word_arm!(self, tokens, explicit_english, out, prev_was_number, numeric_mode, skip_to, line_mode_active, grade1_passage, cap_start_grade1, in_passage, escaped_code, regex_listing, spanish_foreign, foreign_passage, scansion_stress_context, early_english, spatial_grade1_passage, skip_flattened_line_indent, i, chars)
                }
                EnglishToken::WordDivision { chars, break_at } if poem_linear_context => {
                    let (left, right) = chars.split_at(*break_at);
                    self.encode_word(
                        left,
                        WordContext {
                            standing_alone: true,
                            upper_usable: true,
                            shortform_usable: true,
                            allow_longer_shortforms: true,
                            lower_usable: true,
                            suppress_caps: false,
                            word_initial: true,
                            restricted_prefix_boundary: true,
                            digit_adjacent: false,
                        },
                        &mut out,
                    )?;
                    out.extend([decode_unicode('⠸'), SPACE]);
                    self.encode_word(
                        right,
                        WordContext {
                            standing_alone: true,
                            upper_usable: true,
                            shortform_usable: true,
                            allow_longer_shortforms: true,
                            lower_usable: true,
                            suppress_caps: false,
                            word_initial: true,
                            restricted_prefix_boundary: true,
                            digit_adjacent: false,
                        },
                        &mut out,
                    )?;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::WordDivision { chars, break_at } if preserve_spatial_newlines && has_tab => {
                    let (left, right) = chars.split_at(*break_at);
                    self.encode_word(
                        left,
                        WordContext {
                            standing_alone: true,
                            upper_usable: true,
                            shortform_usable: true,
                            allow_longer_shortforms: true,
                            lower_usable: true,
                            suppress_caps: false,
                            word_initial: true,
                            restricted_prefix_boundary: true,
                            digit_adjacent: false,
                        },
                        &mut out,
                    )?;
                    out.push(255);
                    self.encode_word(
                        right,
                        WordContext {
                            standing_alone: true,
                            upper_usable: true,
                            shortform_usable: true,
                            allow_longer_shortforms: true,
                            lower_usable: true,
                            suppress_caps: false,
                            word_initial: true,
                            restricted_prefix_boundary: true,
                            digit_adjacent: false,
                        },
                        &mut out,
                    )?;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::WordDivision { chars, break_at } => {
                    skip_flattened_line_indent = false;
                    line_mode_active = false;
                    self.encode_divided_word(chars, *break_at, in_passage[i], &mut out)?;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::LineBreak => {
                    if preserve_spatial_newlines {
                        out.push(255);
                    } else if flatten_line_layout {
                        out.push(SPACE);
                        skip_flattened_line_indent = true;
                    } else if arrow_diagram_context || scansion_diagram_context {
                        // §11.6.1 step/arrow diagram (`step 1\n↓\nstep 2`) and §15.2.1
                        // scansion diagram (`metre:\n. - . -`) both treat `\n` as a
                        // single-cell separator (⠀). Distinct from §10.13 Note prose
                        // line breaks which take the two-cell end-of-line space.
                        out.push(0);
                    } else if poem_linear_context
                        && !matches!(
                            tokens.get(i + 1),
                            Some(EnglishToken::Symbol('\u{2013}' | '\u{2014}'))
                        )
                    {
                        // §15.1.2 / §2.6.3 Dickinson example: a poem converted to
                        // linear braille marks each original line break with the line
                        // indicator, unspaced from the preceding line and followed by a
                        // space.
                        out.extend([decode_unicode('⠸'), SPACE]);
                    } else if matches!(
                        tokens.get(i + 1),
                        Some(EnglishToken::Symbol('\u{2013}' | '\u{2014}'))
                    ) && matches!(
                        tokens.get(i + 2),
                        Some(EnglishToken::Word(w)) if w.iter().next().is_some_and(|c| c.is_uppercase())
                    ) {
                        // §15.1.2/§15.2.1 poetry attribution (`…ánkles,\n—Ezra Pound`)
                        // — the `\n` before the em-dash + capital-name attribution
                        // collapses to a single cell separator. A lowercase-word
                        // continuation (`always\n—except`) stays a §10.13 prose break.
                        out.push(0);
                    } else {
                        super::rule_10_13::append_break(&mut out, false);
                    }
                    prev_was_number = false;
                    numeric_mode = false;
                    line_mode_active = false;
                }
                EnglishToken::Symbol('"') => {
                    encode_double_quote_arm!(tokens, out, prev_was_number, numeric_mode, quote_open, internal_double_quote_open, passage, regex_listing, i)
                }
	                EnglishToken::Symbol('\u{201C}' | '\u{201D}') => {
	                    let opening = matches!(&tokens[i], EnglishToken::Symbol('\u{201C}'));
	                    if double_quote_needs_two_cell(tokens, i, opening) {
	                        out.push(decode_unicode('⠘'));
	                    }
	                    out.push(if opening { QUOTE_OPEN } else { QUOTE_CLOSE });
	                    prev_was_number = false;
	                    numeric_mode = false;
	                }
                EnglishToken::Symbol('\u{2018}' | '\u{2019}') => {
                    encode_curly_single_quote_arm!(tokens, out, prev_was_number, numeric_mode, sq_roles, i)
                }
                EnglishToken::Symbol('\'') => {
                    encode_straight_single_quote_arm!(tokens, out, prev_was_number, numeric_mode, i)
                }
                EnglishToken::Symbol(c)
                    if *c == super::rule_16::VARIANT_SPACED_SEGMENT
                        && matches!(tokens.get(i + 1), Some(EnglishToken::Space))
                        && matches!(tokens.get(i + 2), Some(EnglishToken::Symbol(s)) if *s == super::rule_16::VARIANT_SPACED_SEGMENT) =>
                {
                    let mut k = i;
                    let mut count = 0usize;
                    while matches!(tokens.get(k), Some(EnglishToken::Symbol(s)) if *s == super::rule_16::VARIANT_SPACED_SEGMENT) {
                        count += 1;
                        k += 1;
                        if matches!(tokens.get(k), Some(EnglishToken::Space)) {
                            k += 1;
                        } else {
                            break;
                        }
                    }
                    out.extend([decode_unicode('⠐'), decode_unicode('⠒')]);
                    for _ in 0..=count {
                        out.push(decode_unicode('⠂'));
                    }
                    skip_to = k;
                    prev_was_number = false;
                    numeric_mode = false;
                }
			                EnglishToken::Symbol(c)
			                    if super::rule_16::is_line_char(*c)
                        && (line_mode_active || i.checked_sub(1).is_some_and(|p| {
                            matches!(&tokens[p], EnglishToken::Symbol(s) if super::rule_16::is_line_char(*s) && (!super::rule_16::is_spatial_segment(*s) || !super::rule_16::is_spatial_segment(*c)))
                        }) || matches!(tokens.get(i + 1), Some(EnglishToken::Symbol(s)) if super::rule_16::is_line_char(*s) && (!super::rule_16::is_spatial_segment(*s) || !super::rule_16::is_spatial_segment(*c)))) =>
                    {
                        encode_line_arm!(tokens, out, prev_was_number, numeric_mode, line_mode_active, skip_flattened_line_indent, i, c)
                    }
                EnglishToken::Symbol(arrow @ ('→' | '↓'))
                    if i.checked_sub(1).is_some_and(|p| {
                        matches!(&tokens[p], EnglishToken::Symbol(s) if super::rule_16::is_line_char(*s))
                    }) =>
                {
                    out.extend(if *arrow == '→' {
                        [decode_unicode('⠳'), decode_unicode('⠕')]
                    } else {
                        [decode_unicode('⠳'), decode_unicode('⠩')]
                    });
                    line_mode_active = true;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::Symbol('\t') => {
                    // UEB 2024 §15.1.3: when tabular columns are linearised, a line
                    // indicator marks the original column break and is followed by a
                    // blank before the following column.
                    out.push(decode_unicode('⠸'));
                    if !matches!(tokens.get(i + 1), Some(EnglishToken::LineBreak)) {
                        out.push(SPACE);
                    }
                    line_mode_active = false;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::Symbol('[') if transcriber_note_at(tokens, i).is_some() => {
                    // §3.27: a `[open tn]` / `[close tn]` print marker becomes a
                    // single note indicator — `⠈⠨⠣` open, `⠈⠨⠜` close — replacing
                    // the five bracketed tokens.
                    let (is_open, end) = transcriber_note_at(tokens, i)?;
                    out.push(decode_unicode('⠈'));
                    out.push(decode_unicode('⠨'));
                    out.push(decode_unicode(if is_open { '⠣' } else { '⠜' }));
                    skip_to = end;
                    prev_was_number = false;
                    numeric_mode = false;
                }
                EnglishToken::Symbol(c) if super::rule_3_24::is_script_char(*c) => {
                    encode_script_arm!(tokens, explicit_english, out, prev_was_number, numeric_mode, skip_to, grade1_passage, i, c)
                }
                EnglishToken::Symbol(c @ ('.' | ','))
                    if matches!(
                        i.checked_sub(1).map(|p| &tokens[p]),
                        None | Some(EnglishToken::Space)
                    ) && matches!(tokens.get(i + 1), Some(EnglishToken::Number(_))) =>
                {
                    // §6: a leading decimal point or comma (`.375`, `,7`) opens a
                    // number — the numeric indicator ⠼ then the separator (`.`→⠲,
                    // `,`→⠂), with numeric mode carrying the following digits (no
                    // second ⠼). A `.`/`,` *after* a digit (`3.14`, `8,93`) is the
                    // §6.3 digit-separator handled in the general Symbol arm below.
                    out.push(super::rule_6::NUMERIC_INDICATOR);
                    if *c == '.'
                        && i.checked_sub(1).is_none()
                        && matches!(tokens.get(i + 1), Some(EnglishToken::Number(digits)) if digits.len() == 2)
                    {
                        out.extend([SPACE, SPACE]);
                    }
                    out.push(decode_unicode(if *c == '.' { '⠲' } else { '⠂' }));
                    prev_was_number = false;
                    numeric_mode = true;
                }
                EnglishToken::Symbol(c) => {
                    encode_symbol_arm!(self, tokens, out, prev_was_number, numeric_mode, skip_to, line_mode_active, passage, cap_term, in_passage, url_listing, regex_listing, foreign_code, spanish_foreign, foreign_passage, early_english, preserve_spatial_newlines, skip_flattened_line_indent, numeric_separator_count, i, c)
                }
                EnglishToken::Styled(_, form) => {
                    encode_styled_arm!(self, tokens, out, prev_was_number, numeric_mode, skip_to, passage, in_passage, foreign_code, spanish_foreign, foreign_passage, drop_styled_typeform_for_code_switch, skip_flattened_line_indent, nested_inner_passage, i, form)
                }
            }
            if cap_term[i] {
                // §8.4 capitals terminator ⠠⠄.
                out.extend([CAPITAL, decode_unicode('⠄')]);
            }
        }
        if let Some(span) = grade1_passage
            && span.needs_terminator
        {
            out.extend([GRADE1, decode_unicode('⠄')]);
        }
        if let Some((_, form, caps, _)) = passage {
            if caps {
                out.extend([CAPITAL, decode_unicode('⠄')]);
            }
            out.extend(super::rule_9::terminator(form));
        }
        if spatial_grade1_passage {
            out.extend([255, GRADE1, decode_unicode('⠄')]);
        }
        if caret_note {
            out.extend([decode_unicode('⠈'), decode_unicode('⠄')]);
        }
        Some(out)
    }
}

#[cfg(test)]
mod test_support {
    use super::*;

    pub(super) fn enc(text: &str) -> Option<Vec<u8>> {
        super::super::try_encode(text)
    }

    /// Build the expected cell vector from a unicode-braille string (`⠀` = space,
    /// `\n` = the §10.13 line-break cell 255).
    pub(super) fn cells(s: &str) -> Vec<u8> {
        s.chars()
            .map(|c| match c {
                '⠀' => SPACE,
                '\n' => 255,
                _ => decode_unicode(c),
            })
            .collect()
    }
}
