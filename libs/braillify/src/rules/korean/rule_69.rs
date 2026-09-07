use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::{EncoderState, RuleContext};
use crate::rules::english_ueb::span::encode_korean_word;
use crate::rules::korean::rule_29::{ENGLISH_CONTINUATION, ROMAN_INDICATOR};
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use unicode_normalization::UnicodeNormalization;

pub static META: RuleMeta = RuleMeta {
    section: "69",
    subsection: None,
    name: "measurement_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.69",
    description: "Measurement and scientific unit symbols",
};

const SINGLE_MAPPINGS: &[(char, &str)] = &[
    ('Ω', "⠴⠠⠨⠺⠲"),
    ('%', "⠴⠏"),
    ('‰', "⠴⠏⠍"),
    ('°', "⠴⠙"),
    ('℃', "⠴⠙⠠⠉"),
    ('℉', "⠴⠙⠠⠋"),
    ('′', "⠴⠤"),
    ('″', "⠴⠤⠤"),
    ('Å', "⠴⠡"),
];

const ASCII_UNIT_MAPPINGS: &[(&str, &str)] = &[
    ("cm", "⠴⠉⠍⠲"),
    ("kg", "⠴⠅⠛⠲"),
    ("in", "⠴⠊⠝⠲"),
    ("mm", "⠴⠍⠍⠲"),
    ("min", "⠴⠍⠔⠲"),
    ("cal", "⠴⠉⠁⠇⠲"),
    ("GB", "⠴⠠⠠⠛⠃⠲"),
    ("m", "⠴⠍⠲"),
    ("h", "⠴⠓⠲"),
];

/// Roman unit symbols printed in the Rule 69 / science-braille unit tables
/// which do not all have a Unicode square-unit presentation form. Their cells
/// are derived through the ordinary Rule 37 letter encoder below, rather than
/// duplicated here as an input-to-output lookup table.
const PDF_ASCII_UNIT_SYMBOLS: &[&str] = &[
    "yard", "sec", "dyn", "kgf", "mmHg", "erg", "HP", "dB", "Hz", "pH", "hPa",
];

/// SI prefixes are case-sensitive. This set is used only as the grammar for a
/// complete measured-unit suffix; it never reclassifies a separated Roman word.
const SI_PREFIXES: &[&str] = &[
    "q", "r", "y", "z", "a", "f", "p", "n", "u", "m", "c", "d", "da", "h", "k", "M", "G", "T", "P",
    "E", "Z", "Y", "R", "Q",
];

const PERCENT_ABBREVIATION_MAPPINGS: &[(&str, &str)] = &[("%ile", "⠴⠏⠞"), ("%p", "⠴⠏⠏")];

const SEPARATED_SYMBOLS: &[char] = &['%', '‰', '°', '℃', '℉'];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

/// Unicode contains compatibility presentation forms for Roman unit symbols:
/// CJK square units (`㎏` → `kg`, `㎓` → `GHz`, `㎥` → `m3`) and the letterlike
/// litre sign (`ℓ` → `l`). Rules 68/69 define the transcription from the
/// semantic Roman unit, so recognize these families from their compatibility
/// decomposition instead of assigning input-specific braille cells. Japanese
/// square words and other CJK compatibility characters are rejected by the
/// code-point ranges and component grammar.
pub(crate) fn compatibility_unit_decomposition(c: char) -> Option<Vec<char>> {
    // Unicode CJK Compatibility contains non-unit square abbreviations too
    // (`㏑` ln, `㏒` log, `㏚` PR). Keep the accepted ranges to scientific and
    // measurement symbols; the component grammar is an additional guard, not
    // the sole evidence that a square abbreviation is a unit.
    let is_unit_codepoint = c == 'ℓ'
        || matches!(
            c as u32,
            0x3371..=0x337a
            | 0x3380..=0x33c6
            | 0x33c8..=0x33cc
            | 0x33ce..=0x33d0
            | 0x33d3..=0x33d9
            | 0x33db..=0x33df
            | 0x33ff
        );
    if !is_unit_codepoint || super::rule_68::is_rule_68_symbol(c) {
        return None;
    }
    let parts = c.to_string().nfkc().collect::<Vec<_>>();
    (parts.iter().any(|part| part.is_ascii_alphabetic())
        && parts.iter().all(|part| {
            part.is_ascii_alphabetic()
                || matches!(part, '2' | '3' | '/' | '\u{2044}' | '\u{2215}' | 'μ')
        }))
    .then_some(parts)
}

/// Unicode presentation forms whose printed meaning is a Roman measurement
/// unit. Rule 68 owns `㎡` and `㏊`; the remaining forms are decoded by this
/// module from their compatibility decomposition. Keeping this predicate at
/// the shared Roman/number state boundary prevents an intervening glyph from
/// breaking a section that began with Roman text and continued through digits.
pub(crate) fn is_compatibility_unit_presentation(c: char) -> bool {
    matches!(c, '㎡' | '㏊') || compatibility_unit_decomposition(c).is_some()
}

/// Rule 69 delegates only to rule 37's multi-letter groupsigns. This is not
/// ordinary UEB word encoding: whole-word signs and shortforms are disabled,
/// and a lower groupsign cannot consume the whole entry run (`in` is spelled
/// `i`-`n`, while the same `in` may contract inside `min`).
fn encode_rule_69_unit_letters(letters: &[char]) -> Result<Vec<u8>, String> {
    match encode_korean_word(letters, false, false, false, true, false, false, false) {
        Some(encoded) => Ok(encoded),
        None => Err(format!(
            "cannot encode rule 69 Roman unit letters: {}",
            letters.iter().collect::<String>()
        )),
    }
}

fn encode_compatibility_unit(
    parts: &[char],
    needs_roman_indicator: bool,
    needs_roman_terminator: bool,
) -> Result<Vec<u8>, String> {
    let mut encoded = Vec::new();
    if needs_roman_indicator {
        encoded.push(ROMAN_INDICATOR);
    }

    let mut index = 0usize;
    while index < parts.len() {
        match parts[index] {
            'μ' => {
                encoded.extend(encode_unicode_cells("⠨⠍"));
                index += 1;
            }
            '2' | '3' => {
                encoded.extend(encode_unicode_cells("⠘⠼"));
                encoded.push(crate::number::encode_number(parts[index])?);
                index += 1;
            }
            '/' | '\u{2044}' | '\u{2215}' => {
                encoded.extend(encode_unicode_cells("⠸⠌"));
                index += 1;
            }
            ch if ch.is_ascii_alphabetic() => {
                let end = index
                    + parts[index..]
                        .iter()
                        .take_while(|part| part.is_ascii_alphabetic())
                        .count();
                let letters = &parts[index..end];
                let unit = encode_rule_69_unit_letters(letters)?;
                encoded.extend(unit);
                index = end;
            }
            unsupported => {
                return Err(format!(
                    "unsupported compatibility unit component: U+{:04X}",
                    unsupported as u32
                ));
            }
        }
    }

    // Rule 68's superscript closes the compact unit without a Roman terminator
    // (`㎡` → `0m^#b`). Otherwise rule 69 terminates the Roman unit unless the
    // same Roman unit chain continues through a slash.
    if needs_roman_terminator && !matches!(parts.last(), Some('2' | '3')) {
        encoded.push(crate::unicode::decode_unicode('⠲'));
    }
    Ok(encoded)
}

fn is_roman_unit_component(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == 'μ' || compatibility_unit_decomposition(ch).is_some()
}

fn roman_unit_chain_continues_before(ctx: &RuleContext) -> bool {
    ctx.index >= 2
        && ctx.word_chars.get(ctx.index - 1) == Some(&'/')
        && ctx
            .word_chars
            .get(ctx.index - 2)
            .is_some_and(|previous| is_roman_unit_component(*previous))
}

fn roman_unit_chain_continues_after(ctx: &RuleContext) -> bool {
    ctx.word_chars.get(ctx.index + 1) == Some(&'/')
        && ctx
            .word_chars
            .get(ctx.index + 2)
            .is_some_and(|next| is_roman_unit_component(*next))
}

pub fn is_rule_69_symbol(c: char) -> bool {
    SINGLE_MAPPINGS.iter().any(|(candidate, _)| *candidate == c)
        || c == 'μ'
        || compatibility_unit_decomposition(c).is_some()
}

fn is_numeric_or_unit_context(ctx: &RuleContext) -> bool {
    let mut numeric_start = ctx.index;
    while numeric_start > 0
        && (ctx.word_chars[numeric_start - 1].is_ascii_digit()
            || matches!(ctx.word_chars[numeric_start - 1], ',' | '.'))
    {
        numeric_start -= 1;
    }
    let compact_numeric_prefix = numeric_start < ctx.index
        && ctx.word_chars[numeric_start..ctx.index]
            .iter()
            .any(char::is_ascii_digit)
        && numeric_start
            .checked_sub(1)
            .and_then(|index| ctx.word_chars.get(index))
            .is_none_or(|previous| !previous.is_ascii_alphabetic());

    compact_numeric_prefix
        || ctx
            .prev_char()
            .is_some_and(|prev| matches!(prev, '/' | 'μ'))
        || ctx.prev_word.chars().next().is_some()
            && ctx
                .prev_word
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'))
        || ctx.prev_char() == Some('/')
}

/// 단어 자체가 단위 연쇄(cal/㎠/min 등)로 구성된 경우 첫 음절이 한국어 뒤에 와도
/// 단위로 해석한다. 단위 연쇄의 특징: 단어 내에 `/`가 있거나 제69항 단위 기호(㎠, ㎏ 등)가
/// 섞여 있다.
fn word_looks_like_unit_chain(word: &[char]) -> bool {
    let mut has_separator = false;
    let mut has_unit_symbol = false;
    for c in word {
        if *c == '/' {
            has_separator = true;
        } else if is_rule_69_symbol(*c) || *c == 'μ' {
            has_unit_symbol = true;
        }
    }
    let has_ascii_letter = word.iter().any(char::is_ascii_alphabetic);
    has_separator && (has_unit_symbol || has_ascii_letter)
}

fn is_symbol_measurement_context(ctx: &RuleContext, symbol: char) -> bool {
    match symbol {
        'μ' => {
            ctx.next_char().is_some_and(|ch| ch.is_ascii_alphabetic())
                || is_numeric_or_unit_context(ctx)
        }
        'Ω' => {
            ctx.next_char().is_some_and(crate::utils::is_korean_char)
                || is_numeric_or_unit_context(ctx)
        }
        _ => true,
    }
}

/// Check whether `tail` starts with the ASCII-only string `s` (char-by-char).
/// All entries in `ASCII_UNIT_MAPPINGS` are ASCII, so byte length and char count
/// coincide; we avoid materializing `tail` into a `String` on the hot path.
fn chars_start_with_ascii(tail: &[char], s: &str) -> bool {
    if tail.len() < s.len() {
        return false;
    }
    s.bytes().zip(tail.iter()).all(|(b, c)| (b as char) == *c)
}

fn encode_ascii_unit_letters(spelling: &[char]) -> Option<Vec<u8>> {
    encode_compatibility_unit(spelling, true, true).ok()
}

/// `Wh` and `Ah` are products of the Rule-69 Roman unit symbols watt/ampere
/// and hour. Accept every case-sensitive SI-prefixed form (`mAh`, `kWh`,
/// `GWh`, ...), rather than enumerating values observed in a corpus.
fn is_si_prefixed_electrical_hour_unit(spelling: &str) -> bool {
    let Some(head) = spelling.strip_suffix('h') else {
        return false;
    };
    let Some(base) = head.chars().last() else {
        return false;
    };
    if !matches!(base, 'A' | 'W') {
        return false;
    }
    let prefix = &head[..head.len() - base.len_utf8()];
    prefix.is_empty() || SI_PREFIXES.contains(&prefix)
}

/// The litre symbol may be printed as either `l` or `L`; an SI prefix retains
/// its case (`dL`, `mL`, `kL`, ...).  Rule 69's printed `㎗` example owns the
/// decilitre semantics, while this grammar preserves the case of an ASCII
/// spelling instead of copying the compatibility character's lowercase NFKC.
fn is_si_prefixed_litre_unit(spelling: &str) -> bool {
    let Some(base) = spelling.chars().last() else {
        return false;
    };
    if !matches!(base, 'l' | 'L') {
        return false;
    }
    let prefix = &spelling[..spelling.len() - base.len_utf8()];
    prefix.is_empty() || SI_PREFIXES.contains(&prefix)
}

/// Rule 69 prints `GB` as its storage-unit example.  Treat the same `B` unit
/// with another case-sensitive SI prefix as one unit symbol, rather than
/// enumerating each storage capacity.  A bare `B` remains ambiguous with a
/// Roman letter and therefore is not selected by this automatic prose route.
fn is_si_prefixed_byte_unit(spelling: &str) -> bool {
    let Some(prefix) = spelling.strip_suffix('B') else {
        return false;
    };
    !prefix.is_empty() && SI_PREFIXES.contains(&prefix)
}

fn standard_ascii_unit_candidate(tail: &[char]) -> Option<(Vec<u8>, usize)> {
    let consumed = tail
        .iter()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count();
    if consumed == 0 {
        return None;
    }
    let spelling = tail[..consumed].iter().collect::<String>();
    if !PDF_ASCII_UNIT_SYMBOLS.contains(&spelling.as_str())
        && !is_si_prefixed_electrical_hour_unit(&spelling)
        && !is_si_prefixed_litre_unit(&spelling)
        && !is_si_prefixed_byte_unit(&spelling)
    {
        return None;
    }
    Some((encode_ascii_unit_letters(&tail[..consumed])?, consumed))
}

/// ASCII spellings that are canonically exposed by the same Unicode
/// compatibility-unit family already accepted above. This derives the unit
/// lexicon from semantic unit code points instead of maintaining a second
/// corpus-shaped list (`㎞` -> `km`, `㎎` -> `mg`, `㎾` -> `kW`, ...).
fn compatibility_ascii_unit_candidate(glyph: char) -> Option<(String, Vec<u8>)> {
    let parts = glyph.to_string().nfkc().collect::<Vec<_>>();
    if !parts.iter().all(char::is_ascii_alphabetic) {
        return None;
    }
    let encoded = if compatibility_unit_decomposition(glyph).is_some() {
        encode_compatibility_unit(&parts, true, true).ok()?
    } else {
        super::rule_68::encode_rule_68_symbol(glyph)?
    };
    Some((parts.into_iter().collect(), encoded))
}

fn compatibility_ascii_unit_owners() -> BTreeMap<String, Vec<(char, Vec<u8>)>> {
    let mut by_spelling = BTreeMap::<String, Vec<(char, Vec<u8>)>>::new();
    for glyph in (0x3300..=0x33ff).filter_map(char::from_u32) {
        if let Some((spelling, encoded)) = compatibility_ascii_unit_candidate(glyph) {
            by_spelling
                .entry(spelling)
                .or_default()
                .push((glyph, encoded));
        }
    }
    by_spelling
}

fn retain_unambiguous_ascii_unit_spellings(
    owners_by_spelling: BTreeMap<String, Vec<(char, Vec<u8>)>>,
) -> Vec<(String, Vec<u8>)> {
    let mut spellings = owners_by_spelling
        .into_iter()
        .filter_map(|(spelling, owners)| {
            let first = &owners.first()?.1;
            owners
                .iter()
                .all(|(_, encoded)| encoded == first)
                .then(|| (spelling, first.clone()))
        })
        .collect::<Vec<_>>();
    spellings.sort_by(|left, right| {
        right
            .0
            .len()
            .cmp(&left.0.len())
            .then_with(|| left.0.cmp(&right.0))
    });
    spellings
}

fn compatibility_ascii_unit_spellings() -> &'static [(String, Vec<u8>)] {
    static SPELLINGS: OnceLock<Vec<(String, Vec<u8>)>> = OnceLock::new();
    SPELLINGS
        .get_or_init(|| retain_unambiguous_ascii_unit_spellings(compatibility_ascii_unit_owners()))
}

pub(crate) fn encode_ascii_unit(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let tail = &word[index..];
    let explicit = ASCII_UNIT_MAPPINGS
        .iter()
        .filter(|(unit, _)| chars_start_with_ascii(tail, unit))
        .max_by_key(|(unit, _)| unit.len())
        .map(|(unit, unicode)| (encode_unicode_cells(unicode), unit.len()));
    let standard = standard_ascii_unit_candidate(tail);

    match (explicit, standard) {
        (Some(explicit), Some(standard)) if explicit.1 == standard.1 => {
            (explicit.0 == standard.0).then_some(explicit)
        }
        (Some(explicit), Some(standard)) if explicit.1 < standard.1 => Some(standard),
        (Some(explicit), _) => Some(explicit),
        (None, standard) => standard,
    }
}

/// Numeric-compact Rule 69 path. Compatibility-derived spellings are limited
/// to this measured boundary so an unrelated English word after a separated
/// number cannot become a unit merely because it starts with a unit spelling.
fn encode_numeric_ascii_unit(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let tail = &word[index..];
    let explicit = encode_ascii_unit(word, index);
    let derived = compatibility_ascii_unit_spellings()
        .iter()
        .filter(|(unit, _)| chars_start_with_ascii(tail, unit))
        .max_by_key(|(unit, _)| unit.len());

    if let Some((encoded, consumed)) = explicit {
        match derived {
            Some((candidate, derived_encoded)) if consumed == candidate.len() => {
                return (encoded.as_slice() == derived_encoded.as_slice())
                    .then_some((encoded, consumed));
            }
            Some((candidate, _)) if consumed < candidate.len() => {}
            _ => return Some((encoded, consumed)),
        }
    }

    let (unit, encoded) = derived?;
    Some((encoded.clone(), unit.len()))
}

fn encode_complete_numeric_ascii_unit(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let (encoded, consumed) = encode_numeric_ascii_unit(word, index)?;
    if word
        .get(index + consumed)
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    Some((encoded, consumed))
}

/// Length of a complete Rule-69 ASCII unit beginning at `index`.
///
/// Token-level capitalization uses this predicate to leave a separated
/// uppercase unit (`5 GB`, `350 PB`) to Rule 69. Otherwise it would emit a
/// Roman/capital prefix before the character rule emits the unit's own prefix.
pub(crate) fn complete_ascii_unit_len(word: &[char], index: usize) -> Option<usize> {
    encode_complete_numeric_ascii_unit(word, index).map(|(_, consumed)| consumed)
}

pub(crate) fn is_ascii_unit_chain_slash(word: &[char], index: usize) -> bool {
    if word.get(index) != Some(&'/') || index == 0 {
        return false;
    }

    let left_start = (0..index)
        .rev()
        .take_while(|position| word[*position].is_ascii_alphabetic())
        .last()
        .unwrap_or(index);
    let left_len = index.saturating_sub(left_start);
    let left_is_complete = left_len > 0
        && encode_complete_numeric_ascii_unit(word, left_start)
            .is_some_and(|(_, consumed)| consumed == left_len);
    let right_is_complete = encode_complete_numeric_ascii_unit(word, index + 1).is_some();

    left_is_complete && right_is_complete
}

fn encode_percent_abbreviation(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let tail = &word[index..];
    for (abbr, unicode) in PERCENT_ABBREVIATION_MAPPINGS {
        if !chars_start_with_ascii(tail, abbr) {
            continue;
        }
        if *abbr == "%p"
            && tail
                .get(abbr.len())
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        {
            continue;
        }
        return Some((encode_unicode_cells(unicode), abbr.len()));
    }
    None
}

pub(crate) fn parse_numeric_ascii_unit_prefix(word: &[char]) -> Option<(String, Vec<u8>, usize)> {
    let numeric_len = word
        .iter()
        .take_while(|c| c.is_ascii_digit() || matches!(**c, ',' | '.'))
        .count();
    if numeric_len == 0 || numeric_len >= word.len() {
        return None;
    }

    let numeric = word[..numeric_len].iter().collect::<String>();
    let (unit, consumed) = encode_complete_numeric_ascii_unit(word, numeric_len)?;
    Some((numeric, unit, numeric_len + consumed))
}

fn numeric_component_len(word: &[char], start: usize) -> usize {
    let len = word[start..]
        .iter()
        .take_while(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'))
        .count();
    if word[start..start + len].iter().any(char::is_ascii_digit) {
        len
    } else {
        0
    }
}

/// Parse a complete Rule-69 measurement expression which starts with a
/// number, including a range (`3.5~8.5m`), a Roman-unit quotient
/// (`240mg/dL`), or a Rule-50 middle-dot list whose every member owns a
/// complete unit (`256GB·512GB·1TB`).  This is a routing predicate only: the
/// ordinary Korean character rules still emit every number, range sign,
/// middle dot, slash and unit.
///
/// Requiring every slash component and the final expression to contain a
/// recognized Rule-69 unit keeps general fractions, dates, model numbers and
/// arbitrary ASCII suffixes outside this route.
pub(crate) fn parse_numeric_ascii_unit_expression(word: &[char]) -> Option<usize> {
    let mut cursor = 0usize;
    let mut saw_unit = false;
    let mut current_component_requires_unit = false;

    loop {
        let numeric_len = numeric_component_len(word, cursor);
        if numeric_len == 0 {
            return None;
        }
        cursor += numeric_len;

        let mut component_has_unit = false;
        if let Some((_, unit_len)) = encode_complete_numeric_ascii_unit(word, cursor) {
            cursor += unit_len;
            saw_unit = true;
            component_has_unit = true;
        }

        while component_has_unit && word.get(cursor) == Some(&'/') {
            let unit_start = cursor + 1;
            let Some((_, unit_len)) = encode_complete_numeric_ascii_unit(word, unit_start) else {
                break;
            };
            cursor = unit_start + unit_len;
            saw_unit = true;
            component_has_unit = true;
        }

        if current_component_requires_unit && !component_has_unit {
            return None;
        }

        if word.get(cursor).is_some_and(|ch| matches!(ch, '~' | '∼')) {
            cursor += 1;
            current_component_requires_unit = false;
            continue;
        }
        if word.get(cursor) == Some(&'·') && component_has_unit {
            cursor += 1;
            current_component_requires_unit = true;
            continue;
        }
        break;
    }

    saw_unit.then_some(cursor)
}

fn trim_recent_english_indicator(result: &mut Vec<u8>) {
    if result
        .last()
        .is_some_and(|cell| matches!(*cell, ENGLISH_CONTINUATION | ROMAN_INDICATOR))
    {
        result.pop();
    }
}

/// Rules 33/34 override rule 69's ordinary trailing Roman terminator when a
/// listed Korean punctuation mark or an enclosing mark closes the Roman run;
/// rule 35 likewise omits it when an attached digit continues the Roman/number
/// chain.
/// Unit encoders include their ordinary terminator so standalone/end/Korean
/// boundaries stay unchanged; this helper applies only at the actual following
/// input boundary.
fn omit_roman_terminator_before_boundary(
    encoded: &mut Vec<u8>,
    word: &[char],
    boundary_index: usize,
) {
    let skips_for_punctuation = word
        .get(boundary_index)
        .is_some_and(|symbol| crate::english_logic::should_skip_terminator_for_symbol(*symbol));
    let continues_through_slash = word.get(boundary_index) == Some(&'/')
        && word
            .get(boundary_index + 1)
            .is_some_and(|next| is_roman_unit_component(*next));
    let continues_into_number = word
        .get(boundary_index)
        .is_some_and(|next| next.is_ascii_digit());
    if (skips_for_punctuation || continues_through_slash || continues_into_number)
        && encoded.last() == Some(&crate::unicode::decode_unicode('⠲'))
    {
        encoded.pop();
    }
}

/// A comma after a Roman unit remains inside the same Roman section when the
/// next print item begins with another Roman/numeric item.  Rule 33 switches
/// to the Korean comma only at an actual Roman-to-Korean boundary; a later
/// Korean particle does not retroactively change the comma in a measurement
/// list such as `173cm, 68kg의`.
fn roman_unit_comma_continues_section(ctx: &RuleContext, boundary_index: usize) -> bool {
    // Rule 68's superscript cell closes a compact square/cubic unit without a
    // Roman terminator. The following comma is therefore Korean punctuation,
    // and a later unit starts a fresh Roman section.
    let closes_with_superscript = ctx.current_char() == '㎡'
        || compatibility_unit_decomposition(ctx.current_char())
            .is_some_and(|parts| matches!(parts.last(), Some('2' | '3')));
    if closes_with_superscript {
        return false;
    }

    ctx.word_chars.get(boundary_index) == Some(&',')
        && crate::english_logic::should_render_symbol_as_english(
            ctx.state.english_indicator,
            true,
            ctx.state.doc_summary.is_english_majority,
            &ctx.state.parenthesis_stack,
            ',',
            ctx.word_chars,
            boundary_index,
            ctx.remaining_words,
        )
}

/// Rules 29 and 35 keep a separated following Roman/number word in the same
/// section. This is the cross-word counterpart of
/// [`omit_roman_terminator_before_boundary`], whose look-ahead is intentionally
/// limited to the current print word.
fn roman_unit_continues_into_next_word(ctx: &RuleContext, boundary_index: usize) -> bool {
    boundary_index == ctx.word_chars.len()
        && ctx
            .remaining_words
            .first()
            .and_then(|word| word.chars().next())
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
}

fn omit_trailing_roman_terminator(encoded: &mut Vec<u8>) {
    if encoded.last() == Some(&crate::unicode::decode_unicode('⠲')) {
        encoded.pop();
    }
}

/// Rule 35 keeps a Roman unit directly following a number in the already-open
/// Roman section.  The number temporarily places the emitter in
/// `roman_number_chain`; in that state the unit's self-contained Rule-69 entry
/// marker would be a duplicate.
fn omit_unit_entry_in_open_roman_number_chain(encoded: &mut Vec<u8>, state: &EncoderState) {
    if state.roman_number_chain && encoded.first() == Some(&ROMAN_INDICATOR) {
        encoded.remove(0);
    }
}

/// Apply the common Rule 29/33/34/35 boundary behavior to a self-contained
/// Roman unit encoding and report whether the section continues into a later
/// print word. Rule 68 reuses this for its two Roman unit presentations.
pub(crate) fn adjust_roman_unit_boundary(
    ctx: &RuleContext,
    boundary_index: usize,
    encoded: &mut Vec<u8>,
) -> bool {
    omit_unit_entry_in_open_roman_number_chain(encoded, ctx.state);
    omit_roman_terminator_before_boundary(encoded, ctx.word_chars, boundary_index);
    let comma_continues = roman_unit_comma_continues_section(ctx, boundary_index);
    let separated_continues = roman_unit_continues_into_next_word(ctx, boundary_index);
    if separated_continues {
        omit_trailing_roman_terminator(encoded);
    }
    comma_continues || separated_continues
}

fn should_insert_separator_after_symbol(symbol: char, next: Option<char>) -> bool {
    SEPARATED_SYMBOLS.contains(&symbol) && next.is_some_and(crate::utils::is_korean_char)
}

pub struct Rule69;

impl BrailleRule for Rule69 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        90
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_rule_69_symbol(*c) && is_symbol_measurement_context(ctx, *c))
            || matches!(ctx.char_type, CharType::Number(_)
                if ctx.index == 0 && parse_numeric_ascii_unit_prefix(ctx.word_chars).is_some())
            || matches!(ctx.char_type, CharType::English(_)
                if (is_numeric_or_unit_context(ctx)
                    || (ctx.index == 0 && word_looks_like_unit_chain(ctx.word_chars)))
                    && encode_complete_numeric_ascii_unit(ctx.word_chars, ctx.index).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if matches!(ctx.char_type, CharType::Number(_))
            && ctx.index == 0
            && let Some((numeric, mut unit, consumed)) =
                parse_numeric_ascii_unit_prefix(ctx.word_chars)
        {
            let continues = adjust_roman_unit_boundary(ctx, consumed, &mut unit);
            let mut encoded = crate::encode(&numeric)?;
            encoded.extend(unit);
            ctx.emit_slice(&encoded);
            ctx.state.is_english = continues;
            ctx.state.needs_english_continuation = false;
            if continues {
                ctx.state.roman_number_chain = false;
            }
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        if matches!(ctx.char_type, CharType::English(_))
            && (is_numeric_or_unit_context(ctx)
                || (ctx.index == 0 && word_looks_like_unit_chain(ctx.word_chars)))
            && let Some((mut encoded, consumed)) =
                encode_complete_numeric_ascii_unit(ctx.word_chars, ctx.index)
        {
            let continues = adjust_roman_unit_boundary(ctx, ctx.index + consumed, &mut encoded);
            if roman_unit_chain_continues_before(ctx) && encoded.first() == Some(&ROMAN_INDICATOR) {
                encoded.remove(0);
            }
            trim_recent_english_indicator(ctx.result);
            ctx.emit_slice(&encoded);
            ctx.state.is_english = continues;
            ctx.state.needs_english_continuation = false;
            if continues {
                ctx.state.roman_number_chain = false;
            }
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '%'
            && let Some((encoded, consumed)) =
                encode_percent_abbreviation(ctx.word_chars, ctx.index)
        {
            ctx.emit_slice(&encoded);
            *ctx.skip_count = consumed.saturating_sub(1);
            if ctx
                .word_chars
                .get(ctx.index + consumed)
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
            {
                ctx.emit(0);
            }
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == 'μ' {
            trim_recent_english_indicator(ctx.result);
            let mut encoded = encode_unicode_cells("⠴⠨⠍");
            let mut consumed = 1usize;

            if let Some((unit_encoded, unit_len)) = encode_ascii_unit(ctx.word_chars, ctx.index + 1)
            {
                let mut unit_without_prefix = unit_encoded;
                if unit_without_prefix.first() == Some(&crate::unicode::decode_unicode('⠴')) {
                    unit_without_prefix.remove(0);
                }
                encoded.extend(unit_without_prefix);
                consumed += unit_len;
            } else {
                encoded.extend(encode_unicode_cells("⠍"));
            }

            omit_roman_terminator_before_boundary(
                &mut encoded,
                ctx.word_chars,
                ctx.index + consumed,
            );

            ctx.emit_slice(&encoded);
            ctx.state.is_english = false;
            ctx.state.needs_english_continuation = false;
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        if let Some(parts) = compatibility_unit_decomposition(ctx.current_char()) {
            let continues_from_previous =
                roman_unit_chain_continues_before(ctx) || ctx.state.roman_number_chain;
            let continues_within_word = roman_unit_chain_continues_after(ctx);
            let separated_continues = roman_unit_continues_into_next_word(ctx, ctx.index + 1);
            let continues_after = continues_within_word || separated_continues;
            let mut encoded =
                encode_compatibility_unit(&parts, !continues_from_previous, !continues_after)?;
            let continues = adjust_roman_unit_boundary(ctx, ctx.index + 1, &mut encoded);
            ctx.emit_slice(&encoded);
            if matches!(parts.last(), Some('2' | '3'))
                && ctx
                    .next_char()
                    .is_some_and(super::rule_44::is_number_confusable_korean_char)
            {
                ctx.emit(0);
            }
            // Same-word compatibility-unit chains are emitted component by
            // component and must retain their existing closed state. Only a
            // continuation across a print space needs to survive into the next
            // Word token.
            ctx.state.is_english = continues;
            ctx.state.needs_english_continuation = false;
            ctx.state.roman_number_chain = false;
            return Ok(RuleResult::Consumed);
        }

        // `matches()` guard `is_rule_69_symbol(c)` is a `SINGLE_MAPPINGS` lookup,
        // so reaching here without the prior μ/ASCII-unit/`%`-shortcut paths
        // means the char is guaranteed to be in `SINGLE_MAPPINGS`.
        let (_, unicode) = SINGLE_MAPPINGS
            .iter()
            .find(|(candidate, _)| *candidate == ctx.current_char())
            .expect("matches() guarantees the char is in SINGLE_MAPPINGS");
        let mut encoded = encode_unicode_cells(unicode);
        omit_roman_terminator_before_boundary(&mut encoded, ctx.word_chars, ctx.index + 1);
        ctx.emit_slice(&encoded);
        if should_insert_separator_after_symbol(ctx.current_char(), ctx.next_char()) {
            ctx.emit(0);
        }
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Rule69, compatibility_ascii_unit_owners, compatibility_unit_decomposition,
        encode_ascii_unit, encode_compatibility_unit, encode_complete_numeric_ascii_unit,
        encode_numeric_ascii_unit, encode_percent_abbreviation, encode_rule_69_unit_letters,
        encode_unicode_cells, is_ascii_unit_chain_slash, is_si_prefixed_byte_unit,
        is_si_prefixed_electrical_hour_unit, is_si_prefixed_litre_unit,
        omit_roman_terminator_before_boundary, omit_trailing_roman_terminator,
        parse_numeric_ascii_unit_expression, parse_numeric_ascii_unit_prefix,
        retain_unambiguous_ascii_unit_spellings, word_looks_like_unit_chain,
    };

    #[rstest::rstest]
    #[case::slash_with_ascii_unit("cal/min", true)]
    #[case::slash_with_unit_symbol("kg/㎠", true)]
    #[case::slash_without_unit_component("//", false)]
    #[case::unit_symbol_without_slash("㎠", false)]
    fn detects_unit_chain_words(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();

        assert_eq!(word_looks_like_unit_chain(&chars), expected);
    }

    #[rstest::rstest]
    #[case::kilogram('㎏', "kg")]
    #[case::gigahertz('㎓', "GHz")]
    #[case::cubic_metre('㎥', "m3")]
    #[case::metres_per_second('㎧', "m∕s")]
    #[case::milliwatt('㎽', "mW")]
    #[case::kilowatt('㎾', "kW")]
    #[case::sievert('㏜', "Sv")]
    #[case::litre('ℓ', "l")]
    fn decomposes_compatibility_unit_symbols(#[case] input: char, #[case] expected: &str) {
        assert_eq!(
            compatibility_unit_decomposition(input),
            Some(expected.chars().collect())
        );
    }

    /// Unicode CJK Compatibility names distinguish the accepted SQUARE IU
    /// (U+337A) from non-unit square abbreviations LN, LOG, and PR. In
    /// particular, U+33DA is SQUARE PR, not SQUARE IU.
    #[rstest::rstest]
    #[case::international_unit('㍺', Some("IU"))]
    #[case::natural_logarithm('㏑', None)]
    #[case::logarithm('㏒', None)]
    #[case::public_relations('㏚', None)]
    fn accepts_only_unit_semantics(#[case] input: char, #[case] expected: Option<&str>) {
        assert_eq!(
            compatibility_unit_decomposition(input),
            expected.map(|text| text.chars().collect())
        );
    }

    const ACCEPTED_GLYPHS: &str = "㍱㍲㍳㍴㍵㍶㍷㍸㍹㍺㎀㎁㎂㎃㎄㎅㎆㎇㎈㎉㎊㎋㎌㎍㎎㎏㎐㎑㎒㎓㎔㎕㎖㎗㎘㎙㎚㎛㎜㎝㎞㎟㎠㎢㎣㎤㎥㎦㎧㎨㎩㎪㎫㎬㎭㎮㎯㎰㎱㎲㎳㎴㎵㎶㎷㎸㎹㎺㎻㎼㎽㎾㎿㏃㏄㏅㏆㏈㏉㏋㏌㏎㏏㏐㏓㏔㏕㏖㏗㏙㏛㏜㏝㏞㏟㏿";

    #[test]
    fn accepted_compatibility_unit_set_is_stable() {
        let actual = (0x3300..=0x33ff)
            .filter_map(char::from_u32)
            .filter(|ch| compatibility_unit_decomposition(*ch).is_some())
            .collect::<String>();

        assert_eq!(actual, ACCEPTED_GLYPHS);
    }

    #[test]
    fn every_accepted_compatibility_unit_encodes_without_panicking() {
        // Generated property check: the set identity is asserted separately,
        // while this loop only proves that every accepted decomposition and
        // each of its ASCII letter runs reaches the fallible Rule 69 encoder.
        for glyph in ACCEPTED_GLYPHS.chars() {
            let parts = compatibility_unit_decomposition(glyph).unwrap();
            let mut index = 0usize;
            while index < parts.len() {
                if !parts[index].is_ascii_alphabetic() {
                    index += 1;
                    continue;
                }
                let end = index
                    + parts[index..]
                        .iter()
                        .take_while(|part| part.is_ascii_alphabetic())
                        .count();
                encode_rule_69_unit_letters(&parts[index..end]).unwrap();
                index = end;
            }
            encode_compatibility_unit(&parts, true, true).unwrap();
        }
    }

    #[test]
    fn compatibility_unit_encoder_rejects_components_outside_its_grammar() {
        let error = encode_compatibility_unit(&['?'], true, true).unwrap_err();

        assert_eq!(error, "unsupported compatibility unit component: U+003F");
    }

    /// The compatibility-unit grammar sends only ASCII-letter runs here.
    /// Rejecting a numeric component directly keeps that defensive contract
    /// observable without weakening the accepted Rule 68/69 glyph set.
    #[rstest::rstest]
    #[case::single_letter("m", true)]
    #[case::multi_letter_unit("min", true)]
    #[case::non_letter_component("1", false)]
    fn unit_letter_encoder_accepts_only_roman_letter_runs(
        #[case] input: &str,
        #[case] expected_ok: bool,
    ) {
        let letters = input.chars().collect::<Vec<_>>();
        let result = encode_rule_69_unit_letters(&letters);

        assert_eq!(result.is_ok(), expected_ok);
        if !expected_ok {
            assert_eq!(
                result.unwrap_err(),
                "cannot encode rule 69 Roman unit letters: 1"
            );
        }
    }

    #[test]
    fn every_rule_68_or_69_ascii_derivation_matches_every_owner_glyph() {
        for (spelling, owners) in compatibility_ascii_unit_owners() {
            let first = &owners[0].1;
            for (glyph, owner_encoding) in &owners {
                assert_eq!(
                    owner_encoding, first,
                    "conflicting owner cells for NFKC spelling {spelling:?}: U+{:04X}",
                    *glyph as u32
                );

                let chars = spelling.chars().collect::<Vec<_>>();
                let (derived, consumed) = encode_numeric_ascii_unit(&chars, 0)
                    .unwrap_or_else(|| {
                        panic!(
                            "unambiguous ASCII compatibility-unit spelling {spelling:?} from U+{:04X} must be recognized",
                            *glyph as u32
                        )
                    });
                assert_eq!(consumed, spelling.len(), "partial match for {spelling}");
                assert_eq!(
                    &derived, owner_encoding,
                    "derived cells differ from owner U+{:04X} for {spelling}",
                    *glyph as u32
                );

                let ascii_input = format!("값은 1{spelling}이다");
                let glyph_input = format!("값은 1{glyph}이다");
                assert_eq!(
                    crate::encode_to_unicode(&ascii_input).unwrap(),
                    crate::encode_to_unicode(&glyph_input).unwrap(),
                    "full encoder differs for {spelling} and owner U+{:04X}",
                    *glyph as u32
                );
            }
        }
    }

    #[test]
    fn conflicting_nfkc_owner_cells_are_excluded_instead_of_first_wins() {
        let owners = std::collections::BTreeMap::from([
            (
                "safe".to_string(),
                vec![('A', vec![1, 2]), ('B', vec![1, 2])],
            ),
            ("conflict".to_string(), vec![('C', vec![3]), ('D', vec![4])]),
        ]);

        let resolved = retain_unambiguous_ascii_unit_spellings(owners);

        assert!(resolved.iter().any(|(spelling, _)| spelling == "safe"));
        assert!(resolved.iter().all(|(spelling, _)| spelling != "conflict"));
    }

    #[rstest::rstest]
    #[case::kilometre("80km", "80㎞")]
    #[case::pdf_milligram("160mg", "160㎎")]
    #[case::numeric_invariance_milligram("240mg", "240㎎")]
    #[case::kilowatt("30kW", "30㎾")]
    #[case::megahertz("96.7MHz", "96.7㎒")]
    #[case::hectare("15.2ha", "15.2㏊")]
    fn compact_ascii_units_match_supported_compatibility_forms(
        #[case] ascii: &str,
        #[case] compatibility: &str,
    ) {
        let ascii = format!("값은 {ascii}이다");
        let compatibility = format!("값은 {compatibility}이다");
        assert_eq!(
            crate::encode_to_unicode(&ascii).unwrap(),
            crate::encode_to_unicode(&compatibility).unwrap()
        );
    }

    #[rstest::rstest]
    #[case::letter_after_digit("3m", "⠼⠉⠍")]
    #[case::letter_after_decimal_punctuation("4.m", "⠼⠙⠲⠍")]
    fn pure_english_ambiguous_suffixes_remain_on_ueb_path(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    #[rstest::rstest]
    #[case::longest_derived("30mW", 4)]
    #[case::hectare_derived("15.2ha", 6)]
    #[case::compound_kilowatt_hour("30kWh", 5)]
    #[case::compound_milliampere_hour("900mAh", 6)]
    #[case::pdf_millimetres_of_mercury("140mmHg", 7)]
    #[case::reject_partial_suffix("30kWhours", 0)]
    fn parses_only_complete_compatibility_derived_units(
        #[case] input: &str,
        #[case] expected_consumed: usize,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        assert_eq!(
            parse_numeric_ascii_unit_prefix(&chars).map_or(0, |(_, _, consumed)| consumed),
            expected_consumed
        );
    }

    #[rstest::rstest]
    #[case::simple_unit("180cm", 5)]
    #[case::range_with_final_unit("3.5~8.5m", 8)]
    #[case::unicode_range_with_final_unit("3∼5kg", 5)]
    #[case::compound_unit_quotient("240mg/dL", 8)]
    #[case::middle_dot_unit_list("256GB·512GB·1TB", 15)]
    #[case::middle_dot_mixed_units("3kg·4cm", 7)]
    #[case::range_without_unit("3.5~8.5", 0)]
    #[case::fraction_with_units_as_operands("3m/4m", 2)]
    #[case::middle_dot_missing_left_unit("3·4kg", 0)]
    #[case::middle_dot_missing_right_unit("3kg·4", 0)]
    #[case::middle_dot_numeric_list("54·55·56", 0)]
    #[case::unknown_ascii_suffix("3.5~8.5models", 0)]
    fn recognizes_only_complete_numeric_unit_expressions(
        #[case] input: &str,
        #[case] expected_consumed: usize,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        assert_eq!(
            parse_numeric_ascii_unit_expression(&chars).unwrap_or(0),
            expected_consumed
        );
    }

    #[rstest::rstest]
    #[case::ascii_range("범위는 3.5~8.5m이다", "⠼⠉⠲⠑⠈⠔⠼⠓⠲⠑⠴⠍⠲")]
    #[case::unicode_range("범위는 3∼5kg이다", "⠼⠉⠈⠔⠼⠑⠴⠅⠛⠲")]
    fn numeric_unit_ranges_stay_on_korean_number_and_unit_rules(
        #[case] input: &str,
        #[case] expected_segment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        assert!(
            actual.contains(expected_segment),
            "missing standard range {expected_segment:?} in {actual:?}"
        );
    }

    #[rstest::rstest]
    #[case::storage_capacities("256GB·512GB·1TB")]
    #[case::mixed_measurements("3kg·4cm")]
    fn separated_middle_dot_unit_lists_stay_on_korean_rules(#[case] expression: &str) {
        let separated = crate::encode_to_unicode(&format!("가는 {expression} 나다")).unwrap();
        let attached = crate::encode_to_unicode(&format!("가는 {expression}이다")).unwrap();
        let segment = attached
            .strip_prefix(&crate::encode_to_unicode("가는 ").unwrap())
            .and_then(|tail| tail.strip_suffix(&crate::encode_to_unicode("이다").unwrap()))
            .expect("attached control must contain the measurement segment");

        assert!(
            separated.contains(segment),
            "measurement segment {segment:?} was rerouted in {separated:?}"
        );
    }

    #[rstest::rstest]
    #[case::pdf_gigabyte("가는 5 GB 나다", "⠫⠉⠵⠀⠼⠑⠀⠴⠠⠠⠛⠃⠲⠀⠉⠊")]
    #[case::petabyte("가는 5 PB 나다", "⠫⠉⠵⠀⠼⠑⠀⠴⠠⠠⠏⠃⠲⠀⠉⠊")]
    #[case::terabyte("가는 5 TB 나다", "⠫⠉⠵⠀⠼⠑⠀⠴⠠⠠⠞⠃⠲⠀⠉⠊")]
    fn separated_uppercase_units_emit_one_roman_capital_prefix(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    /// Rules 29 and 35 omit a unit's Roman terminator when another separated
    /// Roman/number item follows.  Exercise both ASCII and compatibility-unit
    /// Rule-69 paths and the Roman-word continuation path.
    #[rstest::rstest]
    #[case::ascii_unit_before_number("가는 12km 3구간", "⠫⠉⠵⠀⠼⠁⠃⠴⠅⠍⠀⠼⠉⠈⠍⠫⠒")]
    #[case::compatibility_unit_before_number("가는 8.4㎞ 2구간", "⠫⠉⠵⠀⠼⠓⠲⠙⠴⠅⠍⠀⠼⠃⠈⠍⠫⠒")]
    #[case::ascii_unit_before_roman("가는 1TB SSD 나다", "⠫⠉⠵⠀⠼⠁⠴⠠⠠⠞⠃⠀⠠⠠⠎⠎⠙⠲⠀⠉⠊")]
    fn separated_rule_69_unit_continues_roman_number_section(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    #[test]
    fn ascii_unit_quotient_preserves_printed_case_in_one_roman_section() {
        let actual = crate::encode_to_unicode("수치는 240mg/dL이다").unwrap();
        assert!(
            actual.contains("⠼⠃⠙⠚⠴⠍⠛⠸⠌⠙⠠⠇⠲"),
            "unexpected Rule-69 quotient: {actual}"
        );
    }

    #[rstest::rstest]
    #[case::decilitre("dL", true)]
    #[case::millilitre_lower_l("ml", true)]
    #[case::millilitre_upper_l("mL", true)]
    #[case::litre("L", true)]
    #[case::word_ending_l("model", false)]
    #[case::invalid_prefix("xL", false)]
    #[case::empty_spelling("", false)]
    fn recognizes_case_preserving_si_litre_symbols(#[case] spelling: &str, #[case] expected: bool) {
        assert_eq!(is_si_prefixed_litre_unit(spelling), expected);
    }

    #[rstest::rstest]
    #[case::pdf_gigabyte("GB", true)]
    #[case::terabyte("TB", true)]
    #[case::megabyte("MB", true)]
    #[case::bare_letter("B", false)]
    #[case::wrong_base_case("Gb", false)]
    #[case::unknown_prefix("xB", false)]
    fn recognizes_si_prefixed_byte_units(#[case] spelling: &str, #[case] expected: bool) {
        assert_eq!(is_si_prefixed_byte_unit(spelling), expected);
    }

    #[rstest::rstest]
    #[case::milligram_per_decilitre("240mg/dL", 5, true)]
    #[case::calorie_per_minute("cal/min", 3, true)]
    #[case::fraction_operands("3m/4m", 2, false)]
    #[case::arbitrary_letters("F/N", 1, false)]
    fn recognizes_only_slashes_between_complete_unit_components(
        #[case] input: &str,
        #[case] slash_index: usize,
        #[case] expected: bool,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        assert_eq!(
            is_ascii_unit_chain_slash(&chars, slash_index),
            expected,
            "input={input}"
        );
    }

    #[rstest::rstest]
    #[case::watt_hour("Wh", true)]
    #[case::gigawatt_hour("GWh", true)]
    #[case::milliampere_hour("mAh", true)]
    #[case::decaampere_hour("daAh", true)]
    #[case::missing_hour("GW", false)]
    #[case::invalid_base_before_hour("mh", false)]
    #[case::unknown_prefix("xWh", false)]
    #[case::wrong_case("gWh", false)]
    fn recognizes_si_prefixed_electrical_hour_units(
        #[case] spelling: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(is_si_prefixed_electrical_hour_unit(spelling), expected);
    }

    #[test]
    fn trailing_roman_terminator_is_removed_when_section_continues() {
        let terminator = crate::unicode::decode_unicode('⠲');
        let mut encoded = vec![1, terminator];

        omit_trailing_roman_terminator(&mut encoded);

        assert_eq!(encoded, vec![1]);
    }

    #[test]
    fn continuing_ascii_unit_clears_a_prior_roman_number_chain() {
        use crate::rules::traits::BrailleRule;

        let mut owned = crate::test_helpers::CtxOwned::for_text("GB", true)
            .with_prev_word("5")
            .with_remaining_words(["SSD"]);
        owned.state.roman_number_chain = true;
        let mut ctx = owned.ctx_at(0);

        let outcome = Rule69.apply(&mut ctx).expect("Rule 69 unit must encode");

        assert!(matches!(
            outcome,
            crate::rules::traits::RuleResult::Consumed
        ));
        assert!(!ctx.state.roman_number_chain);
        assert!(ctx.state.is_english);
    }

    /// Rule 69 and its science-braille unit table: a complete Roman-written
    /// unit is one section, including its ordinary entry/exit indicators.
    #[rstest::rstest]
    #[case::minute("90min이다", "⠼⠊⠚⠴⠍⠔⠲⠕⠊")]
    #[case::millimetres_of_mercury("140mmHg이다", "⠼⠁⠙⠚⠴⠍⠍⠠⠓⠛⠲⠕⠊")]
    #[case::kilogram_force("75.5kgf이다", "⠼⠛⠑⠲⠑⠴⠅⠛⠋⠲⠕⠊")]
    #[case::gigawatt_hour("13GWh이다", "⠼⠁⠉⠴⠠⠛⠠⠺⠓⠲⠕⠊")]
    #[case::milliampere_hour("900mAh이다", "⠼⠊⠚⠚⠴⠍⠠⠁⠓⠲⠕⠊")]
    fn compact_standard_units_form_one_roman_section(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    #[rstest::rstest]
    #[case::korean_thousand("1천59ha", "1천59㏊")]
    #[case::korean_ten_thousand("3만433ha", "3만433㏊")]
    fn mixed_korean_numbers_keep_the_complete_ascii_unit(
        #[case] ascii: &str,
        #[case] compatibility: &str,
    ) {
        assert_eq!(
            crate::encode_to_unicode(ascii).unwrap(),
            crate::encode_to_unicode(compatibility).unwrap()
        );
    }

    #[test]
    fn complete_unit_matching_rejects_an_ascii_word_with_a_unit_prefix() {
        let chars = "harmony".chars().collect::<Vec<_>>();
        assert!(encode_complete_numeric_ascii_unit(&chars, 0).is_none());
    }

    #[rstest::rstest]
    #[case::inch('㏌', "in")]
    #[case::centimetre('㎝', "cm")]
    #[case::millimetre('㎜', "mm")]
    #[case::gigabyte('㎇', "GB")]
    fn compatibility_units_match_existing_ascii_unit_spelling(
        #[case] glyph: char,
        #[case] ascii: &str,
    ) {
        let ascii_chars = ascii.chars().collect::<Vec<_>>();
        let expected = encode_ascii_unit(&ascii_chars, 0)
            .expect("existing rule 69 ASCII unit")
            .0;
        let decomposition = compatibility_unit_decomposition(glyph).unwrap();
        let actual = encode_compatibility_unit(&decomposition, true, true).unwrap();
        assert_eq!(actual, expected);
    }

    /// Rules 68/69: a compatibility presentation form follows the same general
    /// Roman-unit and superscript algorithm as its Unicode decomposition.
    #[rstest::rstest]
    #[case::kilogram("㎏", "⠴⠅⠛⠲")]
    #[case::gigahertz("㎓", "⠴⠠⠛⠠⠓⠵⠲")]
    #[case::cubic_metre("㎥", "⠴⠍⠘⠼⠉")]
    #[case::milliwatt("㎽", "⠴⠍⠠⠺⠲")]
    #[case::kilowatt("㎾", "⠴⠅⠠⠺⠲")]
    #[case::sievert("㏜", "⠴⠠⠎⠧⠲")]
    #[case::litre("ℓ", "⠴⠇⠲")]
    fn encodes_compatibility_unit_symbols(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    /// Rules 29, 33, 34 and 35 depend on the semantic Roman unit, not on
    /// whether print used ordinary letters or a Unicode compatibility glyph.
    #[rstest::rstest]
    #[case::comma_separated_megawatts("수치는 12.5㎿, 30㎿이다", "수치는 12.5MW, 30MW이다")]
    #[case::unit_after_roman_number_chain("용량은 Lidocaine 5㎖이다", "용량은 Lidocaine 5ml이다")]
    #[case::unit_after_separated_roman_number_chain("대역은 5G 28㎓이다", "대역은 5G 28GHz이다")]
    #[case::rule_68_unit_before_closing_parenthesis("면적은(141㏊)이다", "면적은(141ha)이다")]
    fn compatibility_unit_presentations_match_semantic_roman_spelling(
        #[case] presentation: &str,
        #[case] expanded: &str,
    ) {
        assert_eq!(
            crate::encode_to_unicode(presentation).unwrap(),
            crate::encode_to_unicode(expanded).unwrap(),
            "presentation={presentation:?}"
        );
    }

    #[test]
    fn superscript_closed_compatibility_units_restart_after_a_comma() {
        let actual = crate::encode_to_unicode("농도는 29㎍/㎥, 16㎍/㎥이다").unwrap();
        assert!(
            actual.contains("⠍⠘⠼⠉⠐⠀⠼⠁⠋⠴⠨⠍"),
            "square/cubic unit must close before Korean comma: {actual:?}"
        );
    }

    #[rstest::rstest]
    #[case::confusable_counter("3㎠당", true)]
    #[case::vowel_initial_predicate("3㎠이다", false)]
    fn compatibility_unit_superscript_separates_only_number_confusable_korean(
        #[case] input: &str,
        #[case] expects_separator: bool,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        let unit = "⠴⠉⠍⠘⠼⠃";
        let unit_end = actual.find(unit).expect("square-centimetre cells") + unit.len();
        let follows_with_space = actual[unit_end..].starts_with('⠀');
        assert_eq!(follows_with_space, expects_separator, "input={input}");
    }

    #[test]
    fn slash_after_korean_starts_a_new_roman_unit_chain() {
        let encoded = crate::encode_to_unicode("시/㎏").unwrap();
        assert!(
            encoded.ends_with("⠸⠌⠴⠅⠛⠲"),
            "the Roman indicator must not be suppressed after a Korean component: {encoded}"
        );
    }

    /// Exact PDF examples exercise both Roman-unit continuation through `/`
    /// and termination before a slash followed by a Korean unit.
    #[rstest::rstest]
    #[case::milligram_per_decilitre("160㎎/㎗", "⠼⠁⠋⠚⠴⠍⠛⠸⠌⠙⠇⠲")]
    #[case::calorie_per_square_centimetre_per_minute("cal/㎠/min", "⠴⠉⠁⠇⠸⠌⠉⠍⠘⠼⠃⠸⠌⠍⠔⠲")]
    #[case::megahertz("96.7 ㎒", "⠼⠊⠋⠲⠛⠀⠴⠠⠍⠠⠓⠵⠲")]
    #[case::kilometres_per_hour("80 ㎞/시", "⠼⠓⠚⠀⠴⠅⠍⠲⠸⠌⠠⠕")]
    fn preserves_pdf_unit_examples(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    /// Rules 33/34/35/69: the ordinary unit terminator is omitted when the
    /// actual following boundary is a standard punctuation/enclosing mark or
    /// an attached digit continuing the Roman/number chain. These are
    /// full-encoder checks, including numeric-prefix routing.
    #[rstest::rstest]
    #[case::kilogram_in_parentheses("상자(20kg)당", "⠼⠃⠚⠴⠅⠛⠠⠴", "⠴⠅⠛⠲⠠⠴")]
    #[case::centimetre_before_korean_comma("키는 173cm, 몸무게는", "⠼⠁⠛⠉⠴⠉⠍⠐", "⠴⠉⠍⠲⠐")]
    #[case::centimetre_before_next_measurement("키 173cm, 68kg", "⠼⠁⠛⠉⠴⠉⠍⠂", "⠴⠉⠍⠲⠂")]
    #[case::metre_before_sentence_period("비거리 130m.", "⠼⠁⠉⠚⠴⠍⠲", "⠴⠍⠲⠲")]
    #[case::compatibility_kilogram_in_parentheses("상자(20㎏)당", "⠼⠃⠚⠴⠅⠛⠠⠴", "⠴⠅⠛⠲⠠⠴")]
    #[case::metre_between_numbers("기록은 2m36이다", "⠼⠃⠴⠍⠼⠉⠋", "⠴⠍⠲⠼")]
    #[case::metre_between_larger_numbers("기록은 57m57이다", "⠼⠑⠛⠴⠍⠼⠑⠛", "⠴⠍⠲⠼")]
    #[case::compatibility_kilometre_before_number("거리는 2㎞30이다", "⠼⠃⠴⠅⠍⠼⠉⠚", "⠴⠅⠍⠲⠼")]
    fn omits_unit_terminator_at_standard_override_boundary(
        #[case] input: &str,
        #[case] expected_segment: &str,
        #[case] forbidden_segment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        assert!(
            actual.contains(expected_segment),
            "missing rule-33/34 unit boundary {expected_segment:?} in {actual:?}"
        );
        assert!(
            !actual.contains(forbidden_segment),
            "unexpected rule-69 terminator at rule-33/34 boundary {forbidden_segment:?} in {actual:?}"
        );
    }

    /// Rules 29, 33, 35 and 69: a comma between consecutive measurements is
    /// UEB punctuation inside one Roman section.  The second unit therefore
    /// does not repeat the Roman indicator, even when a Korean particle is
    /// attached after that unit.
    #[rstest::rstest]
    #[case::particle_after_second_unit("키는 173cm, 68kg의 차이다", "⠼⠁⠛⠉⠴⠉⠍⠂⠀⠼⠋⠓⠅⠛⠲")]
    #[case::second_unit_at_end("키는 173cm, 68kg", "⠼⠁⠛⠉⠴⠉⠍⠂⠀⠼⠋⠓⠅⠛⠲")]
    #[case::nanometre_list("공정은 5nm, 1nm는 다르다", "⠼⠑⠴⠝⠍⠂⠀⠼⠁⠝⠍⠲")]
    fn keeps_comma_separated_measurements_in_one_roman_section(
        #[case] input: &str,
        #[case] expected_segment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        assert!(
            actual.contains(expected_segment),
            "missing continuous Roman measurement list {expected_segment:?} in {actual:?}"
        );
    }

    /// Rule 69 remains the default outside the rule-33/34 override. End of
    /// input, a following Korean syllable, and forced slash boundaries retain
    /// the ordinary Roman terminator.
    #[rstest::rstest]
    #[case::end_of_input("180cm", "⠴⠉⠍⠲")]
    #[case::calorie_at_end("열량은 3cal", "⠴⠉⠁⠇⠲")]
    #[case::before_korean("1m는", "⠴⠍⠲")]
    #[case::before_forced_slash("3m/시", "⠴⠍⠲⠸⠌")]
    fn retains_unit_terminator_at_ordinary_rule_69_boundary(
        #[case] input: &str,
        #[case] expected_segment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        assert!(
            actual.contains(expected_segment),
            "missing ordinary rule-69 unit boundary {expected_segment:?} in {actual:?}"
        );
    }

    #[test]
    fn boundary_helper_does_not_remove_non_terminator_cells() {
        let word = "kg)".chars().collect::<Vec<_>>();
        let mut encoded = encode_unicode_cells("⠴⠅⠛");
        omit_roman_terminator_before_boundary(&mut encoded, &word, 2);
        assert_eq!(encoded, encode_unicode_cells("⠴⠅⠛"));
    }

    #[test]
    fn parses_compact_number_unit_word() {
        let chars: Vec<char> = "180cm".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse 180cm");
        assert_eq!(parsed.0, "180");
        assert_eq!(parsed.2, chars.len());
    }

    #[test]
    fn parses_decimal_number_unit_word() {
        let chars: Vec<char> = "1,234.5kg".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse decimal kg");

        assert_eq!(parsed.0, "1,234.5");
        assert_eq!(parsed.2, chars.len());
    }

    #[test]
    fn parses_leading_decimal_numeric_unit_word() {
        let chars: Vec<char> = ".5kg".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse .5kg");

        assert_eq!(parsed.0, ".5");
        assert_eq!(parsed.2, chars.len());
    }

    /// 제69항 — percent-derived measurement abbreviations are data-driven, and
    /// `%p` only contracts at an abbreviation boundary.
    #[rstest::rstest]
    #[case::percentile("%ile", 4)]
    #[case::percentage_point("%p는", 2)]
    fn encodes_percent_abbreviation(#[case] input: &str, #[case] consumed: usize) {
        let chars: Vec<char> = input.chars().collect();
        let (encoded, actual_consumed) = encode_percent_abbreviation(&chars, 0).expect("abbr");
        assert!(!encoded.is_empty());
        assert_eq!(actual_consumed, consumed);
    }

    #[test]
    fn percent_p_does_not_match_inside_ascii_word() {
        let chars: Vec<char> = "%point".chars().collect();
        assert!(encode_percent_abbreviation(&chars, 0).is_none());
    }

    #[test]
    fn ascii_unit_scan_continues_past_non_matching_candidates() {
        let chars: Vec<char> = "zzz".chars().collect();

        assert!(encode_ascii_unit(&chars, 0).is_none());
    }

    #[test]
    fn rule69_metadata_is_stable() {
        use crate::rules::traits::BrailleRule;

        assert_eq!(Rule69.meta().name, "measurement_symbols");
        assert_eq!(Rule69.phase(), crate::rules::traits::Phase::CoreEncoding);
        assert_eq!(Rule69.priority(), 90);
    }

    /// rule_69:255 — `μ` (mu) alone or followed by non-unit chars triggers the
    /// else branch where `encode_unicode_cells("⠍")` is appended.
    #[test]
    fn rule69_mu_alone_without_unit() {
        // μ followed by Korean (no ASCII unit) → encode_ascii_unit returns None →
        // else branch at line 255 fires.
        let result = crate::encode_to_unicode("3μ가");
        assert!(result.is_ok());
        // μ at end with no following text.
        let result = crate::encode_to_unicode("3μ");
        assert!(result.is_ok());
    }
}
