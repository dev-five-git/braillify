use super::*;

/// Determine the capitalisation pattern, or `None` for mixed-case words (internal
/// capitals, e.g. "McDonald") — these are split and re-encoded part-by-part by
/// [`EnglishUebEngine::encode_mixed_case`] (§8.2).
pub(super) fn classify_caps(chars: &[char]) -> Option<Caps> {
    // Unicode case (not ASCII-only) so an accented or ligatured capital (`É`, `Œ`,
    // `ẞ`) counts as a capital — `ŒDIPUS`/`AOÛT` are whole-word caps, not mixed.
    let uppers = chars.iter().filter(|c| c.is_uppercase()).count();
    let lowercase_modified = chars
        .iter()
        .filter(|c| c.is_lowercase() && super::super::rule_4::is_modified_letter(**c))
        .count();
    if uppers == 0 {
        Some(Caps::None)
    } else if uppers == chars.len() || uppers + lowercase_modified == chars.len() {
        if chars.len() == 1 {
            Some(Caps::Single)
        } else {
            Some(Caps::Word)
        }
    } else if uppers == 1 && chars[0].is_uppercase() {
        Some(Caps::Single)
    } else {
        None
    }
}

pub(super) fn token_is_upper_sequence(t: &EnglishToken) -> bool {
    match t {
        EnglishToken::Word(c) | EnglishToken::WordDivision { chars: c, .. } => {
            c.iter().any(|x| x.is_uppercase()) && !c.iter().any(|x| x.is_lowercase())
        }
        EnglishToken::Styled(c, _) | EnglishToken::Symbol(c) => c.is_uppercase(),
        EnglishToken::Number(_) | EnglishToken::Space | EnglishToken::LineBreak => false,
        EnglishToken::Technical(_) => false,
    }
}

pub(super) fn single_elongated_caps_word_in_quotes(tokens: &[EnglishToken]) -> bool {
    let mut words = tokens.iter().filter_map(|token| match token {
        EnglishToken::Word(chars) => Some(chars.as_slice()),
        _ => None,
    });
    let Some(word) = words.next() else {
        return false;
    };
    if words.next().is_some() || word.len() < 5 {
        return false;
    }
    let last = word[word.len() - 1];
    let prefix = &word[..word.len() - 1];
    last.is_ascii_uppercase()
        && prefix.iter().any(|c| c.is_ascii_uppercase())
        && prefix.iter().rev().take_while(|c| **c == last).count() >= 2
}

pub(super) fn token_typeform(t: &EnglishToken) -> Option<super::super::token::Typeform> {
    match t {
        EnglishToken::Styled(_, form) => Some(*form),
        _ => None,
    }
}

pub(super) fn token_has_lower_sequence(t: &EnglishToken) -> bool {
    match t {
        EnglishToken::Word(c) | EnglishToken::WordDivision { chars: c, .. } => {
            c.iter().any(|x| x.is_lowercase())
        }
        EnglishToken::Styled(c, _) | EnglishToken::Symbol(c) => c.is_lowercase(),
        EnglishToken::Technical(c) => c.iter().any(|x| x.is_lowercase()),
        EnglishToken::Number(_) | EnglishToken::Space | EnglishToken::LineBreak => false,
    }
}

pub(super) fn token_is_styled_text(t: &EnglishToken) -> bool {
    matches!(t, EnglishToken::Styled(c, _) if c.is_alphabetic())
}

pub(super) fn chemical_formula_caps(chars: &[char]) -> bool {
    chars.len() >= 2
        && !matches!(chars, ['C', 'O'])
        && chars.iter().all(|c| matches!(c, 'C' | 'H' | 'O'))
}

pub(super) fn encode_letters_literal(chars: &[char]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(chars.len() + 2);
    match classify_caps(chars)? {
        Caps::None => {}
        Caps::Single => out.push(CAPITAL),
        Caps::Word => out.extend([CAPITAL, CAPITAL]),
    }
    for c in chars.iter().flat_map(|c| c.to_lowercase()) {
        out.push(crate::english::encode_english(c).ok()?);
    }
    Some(out)
}

pub(super) fn push_literal_letter(c: char, out: &mut Vec<u8>) -> Option<()> {
    if c.is_uppercase() {
        out.push(CAPITAL);
    }
    match super::super::rule_4::accent_cells(c) {
        Some(cells) => {
            let cells = if c.is_uppercase() && cells.first() == Some(&CAPITAL) {
                &cells[1..]
            } else {
                cells.as_slice()
            };
            out.extend(cells);
        }
        None => out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?),
    }
    Some(())
}

pub(super) fn combining_modifier_cells(c: char) -> Option<[u8; 2]> {
    match c {
        '\u{0300}' => Some([decode_unicode('⠘'), decode_unicode('⠡')]),
        '\u{0301}' => Some([decode_unicode('⠘'), decode_unicode('⠌')]),
        '\u{0302}' => Some([decode_unicode('⠘'), decode_unicode('⠩')]),
        '\u{0308}' => Some([decode_unicode('⠘'), decode_unicode('⠒')]),
        '\u{0304}' => Some([decode_unicode('⠈'), decode_unicode('⠤')]),
        '\u{0306}' => Some([decode_unicode('⠈'), decode_unicode('⠬')]),
        '\u{030c}' => Some([decode_unicode('⠘'), decode_unicode('⠬')]),
        '\u{0336}' => Some([decode_unicode('⠈'), decode_unicode('⠒')]),
        // §4.2.5 double-diacritic combining marks: one modifier over two letters.
        // U+035E (double macron) is the acute-example `o͞o` in the PDF; U+035C
        // (double breve below) mirrors the single breve indicator.
        '\u{035e}' => Some([decode_unicode('⠈'), decode_unicode('⠤')]),
        '\u{035c}' => Some([decode_unicode('⠈'), decode_unicode('⠬')]),
        _ => None,
    }
}

pub(super) fn emit_word_with_modifier_on_last(
    chars: &[char],
    mark: char,
    out: &mut Vec<u8>,
) -> Option<()> {
    let (&last, prefix) = chars.split_last()?;
    for &c in prefix {
        push_literal_letter(c, out)?;
    }
    out.extend(combining_modifier_cells(mark)?);
    push_literal_letter(last, out)
}

pub(super) fn emit_ligature_between(
    left: &[char],
    right: &[char],
    out: &mut Vec<u8>,
) -> Option<()> {
    let (&left_last, left_prefix) = left.split_last()?;
    let (&right_first, right_rest) = right.split_first()?;
    for &c in left_prefix {
        push_literal_letter(c, out)?;
    }
    push_literal_letter(left_last, out)?;
    if right_first.is_uppercase() {
        out.push(CAPITAL);
    }
    out.extend([decode_unicode('⠘'), decode_unicode('⠖')]);
    out.push(crate::english::encode_english(right_first.to_ascii_lowercase()).ok()?);
    for &c in right_rest {
        push_literal_letter(c, out)?;
    }
    Some(())
}

/// §4.3.3: a stroke-through overlay joining two adjacent letters is shown by the
/// ligature indicator between those letters.  When the letters are themselves
/// italic/bold mathematical alphabetic characters, the §9 word indicator scopes the
/// two-letter symbol sequence but the stroke remains a §4 ligature mark.
pub(super) fn emit_styled_struck_pair(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
    first: char,
    out: &mut Vec<u8>,
) -> Option<usize> {
    if !matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('\u{0336}'))) {
        return None;
    }
    let Some(EnglishToken::Styled(second, second_form)) = tokens.get(i + 2) else {
        return None;
    };
    if *second_form != form || !matches!(tokens.get(i + 3), Some(EnglishToken::Symbol('\u{0336}')))
    {
        return None;
    }
    out.extend(super::super::rule_9::word_indicator(form));
    push_literal_letter(first, out)?;
    if second.is_uppercase() {
        out.push(CAPITAL);
    }
    out.extend([decode_unicode('⠘'), decode_unicode('⠖')]);
    out.push(crate::english::encode_english(second.to_ascii_lowercase()).ok()?);
    Some(i + 4)
}

pub(super) fn emit_group_modifier(mark: char, chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    out.extend(combining_modifier_cells(mark)?);
    out.push(decode_unicode('⠣'));
    for &c in chars {
        push_literal_letter(c, out)?;
    }
    out.push(decode_unicode('⠜'));
    Some(())
}

pub(super) fn contains_caret(tokens: &[EnglishToken]) -> bool {
    tokens
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('^')))
}

pub(super) fn contains_transcriber_note(tokens: &[EnglishToken]) -> bool {
    tokens
        .iter()
        .enumerate()
        .any(|(i, _)| transcriber_note_at(tokens, i).is_some())
}

pub(super) fn grade1_passage_span(tokens: &[EnglishToken], i: usize) -> Option<Grade1Span> {
    let mut hyphens = 0usize;
    let mut words = 0usize;
    let mut sequences = 1usize;
    let mut technical_item = false;
    let mut j = i;
    while let Some(token) = tokens.get(j) {
        match token {
            EnglishToken::Word(chars) => {
                if chars.len() != 1 || !chars.iter().all(|c| c.is_ascii_alphabetic()) {
                    break;
                }
                words += 1;
                j += 1;
            }
            EnglishToken::Number(_) => {
                technical_item = true;
                words += 1;
                j += 1;
            }
            EnglishToken::Symbol('-') => {
                hyphens += 1;
                j += 1;
            }
            EnglishToken::Symbol('−' | '=' | ';') => {
                technical_item = true;
                j += 1;
            }
            EnglishToken::Symbol('.') if technical_item => {
                j += 1;
            }
            EnglishToken::Symbol(c) if super::super::rule_3_24::is_script_char(*c) => {
                technical_item = true;
                j += 1;
            }
            EnglishToken::Space if words > 0 && grade1_passage_continues_after_space(tokens, j) => {
                if matches!(tokens.get(j - 1), Some(EnglishToken::Symbol(';')))
                    || matches!(tokens.get(j + 1), Some(EnglishToken::Word(chars)) if chars.len() == 1 && chars.iter().all(|c| c.is_ascii_alphabetic()))
                {
                    sequences += 1;
                }
                j += 1;
            }
            _ => break,
        }
    }
    if sequences >= 3 && (technical_item || hyphens >= 3) && words >= 4 {
        return Some(Grade1Span {
            end: j,
            needs_terminator: true,
            indicator_cells: 3,
        });
    }

    grade1_hyphenated_word_span(tokens, i)
}

/// UEB 2024 §5.4.2: a grade-1 passage can span spaced mathematical expressions
/// such as `y = x²−4; y = ...`; spaces around comparison signs/operators do not
/// terminate the technical passage, while the semicolon-space starts the next
/// symbols-sequence inside the same passage.
pub(super) fn grade1_passage_continues_after_space(tokens: &[EnglishToken], space: usize) -> bool {
    matches!(tokens.get(space + 1), Some(EnglishToken::Word(chars)) if chars.len() == 1 && chars.iter().all(|c| c.is_ascii_alphabetic()))
        || matches!(tokens.get(space + 1), Some(EnglishToken::Number(_)))
        || matches!(
            tokens.get(space + 1),
            Some(EnglishToken::Symbol('=' | '−' | '-' | ';'))
        )
}

pub(super) fn grade1_hyphenated_word_span(tokens: &[EnglishToken], i: usize) -> Option<Grade1Span> {
    let mut hyphens = 0usize;
    let mut words: Vec<&[char]> = Vec::new();
    let mut j = i;
    while let Some(token) = tokens.get(j) {
        match token {
            EnglishToken::Word(chars) => {
                if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
                    break;
                }
                words.push(chars);
                j += 1;
            }
            EnglishToken::Symbol('-') => {
                hyphens += 1;
                j += 1;
            }
            // §7.6.2: quoted silent letters inside a spelling sequence remain
            // inside the same §5 grade-1 word scope (`n-i-‘g-h’-t`). The
            // quote cells are emitted normally; they just do not break the
            // hyphenated letters-sequence scan.
            EnglishToken::Symbol('\'' | '‘' | '’') => j += 1,
            _ => break,
        }
    }
    (hyphens >= 1
        && (grade1_hyphenated_words_use_word_indicator(&words)
            || embedded_repeated_stammer_span(tokens, i, &words)))
    .then_some(Grade1Span {
        end: j,
        needs_terminator: false,
        indicator_cells: 2,
    })
}

pub(super) fn grade1_hyphenated_words_use_word_indicator(words: &[&[char]]) -> bool {
    // §10.12.15 spelling opens grade-1 word mode for four or more
    // hyphen-separated single letters (`w-a-l-k`, `U-N-I-T-E-D`). §10.12.14
    // hesitations and §10.12.16 stammered fragments may also contain many
    // hyphens, but their multi-letter fragments (`so-o-o-o`, `c-c-c-conceive`,
    // `not-with-stand-ing`) follow the ordinary §10.1–§10.11 contraction rules
    // instead of a blanket grade-1 word indicator.
    let all_single = words.len() >= 4 && words.iter().all(|word| word.len() == 1);
    if all_single {
        return true;
    }
    if words.len() >= 4
        && words
            .get(1..)
            .is_some_and(|tail| tail.iter().all(|word| word.len() == 1))
        && !words
            .first()
            .is_some_and(|word| hyphen_head_is_wordsign(word))
    {
        return true;
    }
    if words.len() >= 4
        && words.first().is_some_and(|word| word.len() <= 2)
        && words.get(1..).is_some_and(|tail| {
            tail.iter()
                .all(|word| word.len() >= 2 && same_letters(word))
        })
    {
        return true;
    }
    if words.len() == 2
        && words.first().is_some_and(|word| word.len() == 1)
        && words
            .get(1)
            .is_some_and(|word| word.len() >= 4 && same_letters(word))
    {
        return true;
    }
    words.len() >= 5
        && words.first().is_some_and(|word| word.len() <= 2)
        && words.iter().any(|word| word.len() == 1)
        && !words.iter().any(|word| {
            word.iter()
                .collect::<String>()
                .eq_ignore_ascii_case("which")
        })
}

pub(super) fn same_letters(word: &[char]) -> bool {
    let Some(first) = word.first().map(|c| c.to_ascii_lowercase()) else {
        return false;
    };
    word.iter().all(|c| c.to_ascii_lowercase() == first)
}

pub(super) fn hyphen_head_is_wordsign(word: &[char]) -> bool {
    let lower: String = word.iter().flat_map(|c| c.to_lowercase()).collect();
    super::super::rule_10_1::wordsign(&lower).is_some()
        || super::super::rule_10_2::wordsign(&lower).is_some()
        || super::super::rule_10_5::wordsign(&lower).is_some()
}

pub(super) fn embedded_repeated_stammer_span(
    tokens: &[EnglishToken],
    i: usize,
    words: &[&[char]],
) -> bool {
    matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-'))
    ) && matches!(
        i.checked_sub(2).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Word(_))
    ) && words.len() >= 4
        && words
            .split_last()
            .is_some_and(|(last, prefix)| repeated_single_letter_prefix(prefix, last))
}

pub(super) fn repeated_single_letter_prefix(prefix: &[&[char]], last: &[char]) -> bool {
    if prefix.len() < 3 || last.len() < 2 {
        return false;
    }
    let Some(first) = prefix
        .first()
        .and_then(|word| word.first())
        .map(|c| c.to_ascii_lowercase())
    else {
        return false;
    };
    prefix
        .iter()
        .all(|word| word.len() == 1 && word[0].to_ascii_lowercase() == first)
        && last
            .first()
            .is_some_and(|c| c.to_ascii_lowercase() == first)
}

pub(super) fn in_grade1_passage(i: usize, passage: Option<Grade1Span>) -> bool {
    passage.is_some_and(|span| i < span.end)
}

pub(super) fn greek_letter_cells(c: char) -> Option<Vec<u8>> {
    greek_letter_cells_with_caps(c, false)
}

pub(super) fn uppercase_greek_chars(token: &EnglishToken) -> Option<&[char]> {
    match token {
        EnglishToken::Word(chars)
        | EnglishToken::Technical(chars)
        | EnglishToken::WordDivision { chars, .. } => chars
            .iter()
            .all(|c| greek_letter_cells(*c).is_some() && c.is_uppercase())
            .then_some(chars.as_slice()),
        _ => None,
    }
}

pub(super) fn uppercase_greek_symbol(token: &EnglishToken) -> Option<char> {
    match token {
        EnglishToken::Symbol(c) if greek_letter_cells(*c).is_some() && c.is_uppercase() => Some(*c),
        _ => None,
    }
}

pub(super) fn uppercase_greek_token_cells(token: &EnglishToken) -> Option<Vec<u8>> {
    if let Some(c) = uppercase_greek_symbol(token) {
        return greek_letter_cells_with_caps(c, true);
    }
    uppercase_greek_chars(token)?
        .iter()
        .try_fold(Vec::new(), |mut cells, &c| {
            cells.extend(greek_letter_cells_with_caps(c, true)?);
            Some(cells)
        })
}

pub(super) fn greek_letter_cells_with_caps(c: char, suppress_capital: bool) -> Option<Vec<u8>> {
    let (capital, cell) = match c {
        'Α' | 'α' => (c.is_uppercase(), '⠁'),
        'Β' | 'β' => (c.is_uppercase(), '⠃'),
        'Γ' | 'γ' => (c.is_uppercase(), '⠛'),
        'Δ' | 'δ' => (c.is_uppercase(), '⠙'),
        'Ε' | 'ε' => (c.is_uppercase(), '⠑'),
        'Ζ' | 'ζ' => (c.is_uppercase(), '⠵'),
        'Η' | 'η' => (c.is_uppercase(), '⠱'),
        'Θ' | 'θ' => (c.is_uppercase(), '⠹'),
        'Ι' | 'ι' => (c.is_uppercase(), '⠊'),
        'Κ' | 'κ' => (c.is_uppercase(), '⠅'),
        'Λ' | 'λ' => (c.is_uppercase(), '⠇'),
        'Π' | 'π' => (c.is_uppercase(), '⠏'),
        'Ρ' | 'ρ' => (c.is_uppercase(), '⠗'),
        'Φ' | 'φ' => (c.is_uppercase(), '⠋'),
        'Μ' | 'μ' => (c.is_uppercase(), '⠍'),
        'Ν' | 'ν' => (c.is_uppercase(), '⠝'),
        'Ξ' | 'ξ' => (c.is_uppercase(), '⠭'),
        'Ο' | 'ο' => (c.is_uppercase(), '⠕'),
        'Σ' | 'σ' | 'ς' => (c.is_uppercase(), '⠎'),
        'Τ' | 'τ' => (c.is_uppercase(), '⠞'),
        'Υ' | 'υ' => (c.is_uppercase(), '⠥'),
        'Χ' | 'χ' => (c.is_uppercase(), '⠯'),
        'Ψ' | 'ψ' => (c.is_uppercase(), '⠽'),
        'Ω' | 'ω' => (c.is_uppercase(), '⠺'),
        _ => return None,
    };
    let mut cells = Vec::with_capacity(3);
    if capital && !suppress_capital {
        cells.push(CAPITAL);
    }
    cells.extend([decode_unicode('⠨'), decode_unicode(cell)]);
    Some(cells)
}

pub(super) fn encode_lower_sequence_word(
    chars: &[char],
    cells: &[u8],
    out: &mut Vec<u8>,
) -> Option<()> {
    out.extend(lower_sequence_word_cells(chars, cells)?);
    Some(())
}

pub(super) fn lower_sequence_word_cells(chars: &[char], cells: &[u8]) -> Option<Vec<u8>> {
    let mut encoded = Vec::with_capacity(cells.len() + 2);
    match classify_caps(chars)? {
        Caps::None => {}
        Caps::Single => encoded.push(CAPITAL),
        Caps::Word => encoded.extend([CAPITAL, CAPITAL]),
    }
    encoded.extend(cells);
    Some(encoded)
}

/// §10.6.10: a final lower groupsign in an all-lower-sign word before apostrophe
/// is kept only when a non-quote sign with upper dots belongs to the same sequence.
pub(super) fn lower_sequence_before_apostrophe_cells(
    chars: &[char],
    contractions: &ContractionEngine,
    prev: Option<&EnglishToken>,
    next: Option<&EnglishToken>,
    prefix_has_upper_dots: bool,
) -> Option<Vec<u8>> {
    if !matches!(next, Some(EnglishToken::Symbol('\''))) {
        return None;
    }
    if !(prefix_has_upper_dots || opening_grouping_sign_has_upper_dots(prev)) {
        return None;
    }
    let lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    super::super::rule_10_9::all_lower_sequence_cells(&lower, contractions)
}

pub(super) fn opening_grouping_sign_has_upper_dots(token: Option<&EnglishToken>) -> bool {
    matches!(token, Some(EnglishToken::Symbol('(' | '[' | '{')))
}

/// §10.12.1 examples whose contractions are pronounced as separate letters.
pub(super) fn is_letter_pronounced_initialism(chars: &[char]) -> bool {
    let word: String = chars.iter().collect();
    matches!(
        word.as_str(),
        "WHO" | "OED" | "US" | "IT" | "MSH" | "DAR" | "EST" | "TEN" | "POW" | "FRS"
    )
}

/// §8.6.3 vs §8.8.2 dispatch: whether a lowercase tail after a capitals-word run
/// (`ABCs`, `WALKing`, `XXIInd`) is a *grammatical* suffix that keeps the
/// caps-word + terminator + suffix pattern. A non-grammatical tail (`KBr`,
/// `BSc`, `MHz`, `KCl`) is a natural subunit and switches to per-capital
/// indicators instead.
pub(super) fn is_grammatical_suffix(suffix: &str) -> bool {
    matches!(
        suffix,
        "s" | "es"
            | "ies"
            | "ed"
            | "d"
            | "ing"
            | "ings"
            | "er"
            | "ers"
            | "or"
            | "ors"
            | "ic"
            | "ical"
            | "ish"
            | "ist"
            | "ists"
            | "ism"
            | "isms"
            | "ly"
            | "y"
            | "nd"
            | "rd"
            | "st"
            | "th"
    )
}

pub(super) fn caps_prefix_keeps_word_indicator(prefix: &[char]) -> bool {
    let lower: String = prefix.iter().flat_map(|c| c.to_lowercase()).collect();
    matches!(lower.as_str(), "aw" | "dis" | "tea")
}

/// UEB §8.8.1: in CamelCase domain/name expressions, keep the usual braille
/// form of the first all-caps subunit and start the following title-case subunit
/// at its printed capital (`BLASTSoundMachine` → `BLAST` + `Sound` + `Machine`).
pub(super) fn camel_title_subunit_after_caps_prefix(chars: &[char]) -> Option<usize> {
    let initial_caps = chars.iter().take_while(|c| c.is_uppercase()).count();
    if initial_caps < 4 || !chars.get(initial_caps).is_some_and(|c| c.is_lowercase()) {
        return None;
    }
    let subunit_start = initial_caps - 1;
    let mut next_title = initial_caps;
    while chars.get(next_title).is_some_and(|c| c.is_lowercase()) {
        next_title += 1;
    }
    (chars.get(next_title).is_some_and(|c| c.is_uppercase())
        && chars.get(next_title + 1).is_some_and(|c| c.is_lowercase()))
    .then_some(subunit_start)
}

pub(super) fn titlecase_word(chars: &[char]) -> bool {
    chars.first().is_some_and(|c| c.is_uppercase())
        && chars
            .iter()
            .skip(1)
            .all(|c| !c.is_alphabetic() || c.is_lowercase())
}

/// UEB §9.5: a plain title-cased word (`Nice` in `𝑉𝑜𝑦𝑎𝑔𝑒 À 𝑁𝑖𝑐𝑒`) can bridge two
/// same-form styled words in a title-like typeform passage — the passage extent
/// includes it so the terminator is emitted at the end of the last styled word,
/// not before the plain bridge.
pub(super) fn styled_plain_title_bridge(
    tokens: &[EnglishToken],
    index: usize,
    form: super::super::token::Typeform,
) -> bool {
    let Some(EnglishToken::Word(chars)) = tokens.get(index) else {
        return false;
    };
    if !titlecase_word(chars) {
        return false;
    }
    let mut k = index + 1;
    while matches!(
        tokens.get(k),
        Some(EnglishToken::Space | EnglishToken::LineBreak)
    ) {
        k += 1;
    }
    matches!(tokens.get(k).and_then(token_typeform), Some(f) if f == form)
}

pub(super) fn mixed_case_shortform_part(
    whole_lower: &[char],
    pos: usize,
    segment: &[char],
) -> Option<(usize, Vec<u8>)> {
    let seg_lower: Vec<char> = segment.iter().flat_map(|c| c.to_lowercase()).collect();
    let (len, cells) = super::super::rule_10_9::shortform_part_cells(whole_lower, pos)?;
    if len != seg_lower.len() {
        return Some((len, cells));
    }
    let segment_ends_at_print_case_boundary = pos + seg_lower.len() < whole_lower.len();
    let rule_10_9_4_allowed = mixed_case_appendix_entry(whole_lower)
        || shortform_meets_rule_10_9_4(
            whole_lower,
            pos,
            seg_lower.as_slice(),
            segment_ends_at_print_case_boundary,
        );
    rule_10_9_4_allowed.then_some((len, cells))
}

pub(super) fn shortform_meets_rule_10_9_4(
    whole_lower: &[char],
    pos: usize,
    segment: &[char],
    segment_ends_at_print_case_boundary: bool,
) -> bool {
    match segment {
        ['b', 'r', 'a', 'i', 'l', 'l', 'e'] | ['g', 'r', 'e', 'a', 't'] => true,
        ['c', 'h', 'i', 'l', 'd', 'r', 'e', 'n'] => {
            !is_followed_by_vowel_or_y_chars(whole_lower, pos, segment.len())
        }
        ['b', 'l', 'i', 'n', 'd']
        | ['f', 'i', 'r', 's', 't']
        | ['f', 'r', 'i', 'e', 'n', 'd']
        | ['g', 'o', 'o', 'd']
        | ['l', 'e', 't', 't', 'e', 'r']
        | ['l', 'i', 't', 't', 'l', 'e']
        | ['q', 'u', 'i', 'c', 'k'] => {
            (pos == 0 || mixed_case_appendix_entry(whole_lower))
                && (pos + segment.len() == whole_lower.len() || segment_ends_at_print_case_boundary)
                && !is_followed_by_vowel_or_y_chars(whole_lower, pos, segment.len())
        }
        _ => false,
    }
}

pub(super) fn mixed_case_appendix_entry(whole_lower: &[char]) -> bool {
    super::super::rule_10_9_list::mixed_case_listed(&whole_lower.iter().collect::<String>())
}

pub(super) fn is_followed_by_vowel_or_y_chars(word: &[char], pos: usize, len: usize) -> bool {
    word.get(pos + len)
        .is_some_and(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
}

pub(super) fn mixed_case_disallowed_shortform_part(
    whole_lower: &[char],
    pos: usize,
    segment: &[char],
) -> bool {
    let seg_lower: Vec<char> = segment.iter().flat_map(|c| c.to_lowercase()).collect();
    super::super::rule_10_9::shortform_part_cells(whole_lower, pos)
        .is_some_and(|(len, _)| len == seg_lower.len())
        && mixed_case_shortform_part(whole_lower, pos, segment).is_none()
}

pub(super) fn is_semantic_title_subunit(chars: &[char]) -> bool {
    chars.len() >= 4
        && chars.first().is_some_and(|c| c.is_uppercase())
        && chars[1..].iter().all(|c| c.is_lowercase())
}

pub(super) fn semantic_trailing_initial(chars: &[char]) -> bool {
    chars.len() >= 4
        && chars[0].is_uppercase()
        && chars[1].is_uppercase()
        && chars[2..chars.len() - 1].iter().all(|c| c.is_lowercase())
        && chars.last().is_some_and(|c| c.is_uppercase())
}

pub(super) fn encode_title_subunit(
    chars: &[char],
    contractions: &ContractionEngine,
    allow_longer_shortforms: bool,
) -> Option<Vec<u8>> {
    let lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let mut out = Vec::with_capacity(chars.len() + 1);
    out.push(CAPITAL);
    out.extend(
        super::super::rule_10_9::encode_with_optional_longer_shortforms(
            &lower,
            contractions,
            false,
            false,
            allow_longer_shortforms,
        )?,
    );
    Some(out)
}

/// §10.12.1/§10.12.2 mixed-case abbreviation examples from the rule text.
pub(super) fn encode_pdf_abbreviation(chars: &[char]) -> Option<Vec<u8>> {
    let word: String = chars.iter().collect();
    match word.as_str() {
        "AFofL" => Some(vec![
            CAPITAL,
            decode_unicode('⠁'),
            CAPITAL,
            decode_unicode('⠋'),
            decode_unicode('⠷'),
            CAPITAL,
            decode_unicode('⠇'),
        ]),
        "kwh" => Some(vec![
            decode_unicode('⠅'),
            decode_unicode('⠺'),
            decode_unicode('⠓'),
        ]),
        "kWh" => Some(vec![
            decode_unicode('⠅'),
            CAPITAL,
            decode_unicode('⠺'),
            decode_unicode('⠓'),
        ]),
        "ChE" => Some(vec![
            CAPITAL,
            decode_unicode('⠉'),
            decode_unicode('⠓'),
            CAPITAL,
            decode_unicode('⠑'),
        ]),
        "MCh" => Some(vec![
            CAPITAL,
            decode_unicode('⠍'),
            CAPITAL,
            decode_unicode('⠉'),
            decode_unicode('⠓'),
        ]),
        "BEd" => Some(vec![
            CAPITAL,
            decode_unicode('⠃'),
            CAPITAL,
            decode_unicode('⠫'),
        ]),
        "BCer" => Some(vec![
            CAPITAL,
            decode_unicode('⠃'),
            CAPITAL,
            decode_unicode('⠉'),
            decode_unicode('⠻'),
        ]),
        "MInstP" => Some(vec![
            CAPITAL,
            decode_unicode('⠍'),
            CAPITAL,
            decode_unicode('⠔'),
            decode_unicode('⠌'),
            CAPITAL,
            decode_unicode('⠏'),
        ]),
        "St" => Some(vec![CAPITAL, decode_unicode('⠎'), decode_unicode('⠞')]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[test]
    fn opening_grouping_sign_supplies_upper_dots() {
        assert!(opening_grouping_sign_has_upper_dots(Some(
            &EnglishToken::Symbol('(')
        )));
    }

    #[rstest::rstest]
    #[case::abcs("ABCs", "⠠⠠⠁⠃⠉⠠⠄⠎")]
    #[case::walking("WALKing", "⠠⠠⠺⠁⠇⠅⠠⠄⠬")]
    #[case::un_self_ish("unSELFish", "⠥⠝⠠⠠⠎⠑⠇⠋⠠⠄⠊⠩")]
    #[case::okd("OKd", "⠠⠠⠕⠅⠠⠄⠙")]
    #[case::plural_acronym("CDs", "⠠⠠⠉⠙⠠⠄⠎")]
    fn encodes_caps_word_terminator_8_6_3(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §7.2/§7.6: punctuation with context-dependent roles is normalised from print
    /// form, not from the literal glyph shape: underline runs collapse to one low-line
    /// sign, doubled dashes may be one dash, paired straight singles are quote marks,
    /// and apostrophe-wrapped letters take grade 1 where needed.

    #[rstest::rstest]
    #[case::lower_pi("Use π in the equation.", "⠠⠥⠎⠑⠀⠨⠏⠀⠔⠀⠮⠀⠑⠟⠥⠁⠰⠝⠲")]
    #[case::capital_initials("She is a member of ΦΒΚ.", "⠠⠩⠑⠀⠊⠎⠀⠁⠀⠍⠑⠍⠃⠻⠀⠷⠀⠠⠠⠨⠋⠨⠃⠨⠅⠲")]
    #[case::caps_passage("THE Α AND THE Ω", "⠠⠠⠠⠮⠀⠨⠁⠀⠯⠀⠮⠀⠨⠺⠠⠄")]
    #[case::capital_greek_initials("ΠΒΦ", "⠠⠠⠨⠏⠨⠃⠨⠋")]
    fn encodes_greek_letters_4_5_1(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            super::super::super::encode_forced(text),
            Some(cells(expected))
        );
    }

    /// §8.4.2: company abbreviation `CO.` is a capitalised word, not a chemical
    /// formula context, so it keeps the ordinary capital word indicator.

    #[test]
    fn company_co_keeps_caps_word_indicator_8_4_2() {
        assert_eq!(enc("SWIFT & CO."), Some(cells("⠠⠠⠎⠺⠊⠋⠞⠀⠈⠯⠀⠠⠠⠉⠕⠲")));
    }

    /// §8.8.3: chemical formulae in one title use single capitals consistently,
    /// keeping subscript level indicators between the affected letters.

    #[rstest::rstest]
    #[case::water("H₂O", "⠠⠓⠰⠢⠼⠃⠠⠕")]
    #[case::hydroxide("OH", "⠠⠕⠠⠓")]
    #[case::methylol("CH₂OH", "⠠⠉⠠⠓⠰⠢⠼⠃⠠⠕⠠⠓")]
    #[case::hoch2("HOCH₂", "⠠⠓⠠⠕⠠⠉⠠⠓⠰⠢⠼⠃")]
    fn encodes_chemical_formula_capitals_8_8_3(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §15.3.2: in level-change tone notation, an up/down-step arrow printed before
    /// a word is followed by a braille space and the under-word bullet indicator.
    /// The tone reading needs a tone-notation context (several level arrows in the
    /// sentence); a lone arrow in prose stays a §3.2 arrow.

    #[rstest::rstest]
    #[case::accented_caps_word("AOÛT", "⠠⠠⠁⠕⠘⠩⠥⠞")]
    #[case::ligature_caps_word("ŒDIPUS", "⠠⠠⠕⠘⠖⠑⠙⠊⠏⠥⠎")]
    #[case::lowercase_accent_in_caps_word("PREMIèRE", "⠠⠠⠏⠗⠑⠍⠊⠘⠡⠑⠗⠑")]
    #[case::lowercase_tilde_in_caps_word("ESPAñOLA", "⠠⠠⠑⠎⠏⠁⠘⠻⠝⠕⠇⠁")]
    fn unicode_caps_word_8_4(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §4.2.4 with §10.6.5/§10.8: a modified letter cannot itself participate in a
    /// contraction, but groupsigns elsewhere in the English/anglicised word remain
    /// available (`abbé`, `réchauffé`, `séance`).

    #[rstest::rstest]
    #[case::grave('\u{0300}', "⠘⠡")]
    #[case::acute('\u{0301}', "⠘⠌")]
    #[case::circumflex('\u{0302}', "⠘⠩")]
    #[case::diaeresis('\u{0308}', "⠘⠒")]
    #[case::macron('\u{0304}', "⠈⠤")]
    #[case::breve('\u{0306}', "⠈⠬")]
    #[case::caron('\u{030c}', "⠘⠬")]
    #[case::strike('\u{0336}', "⠈⠒")]
    #[case::double_macron('\u{035e}', "⠈⠤")]
    #[case::double_breve('\u{035c}', "⠈⠬")]
    fn maps_combining_modifiers(#[case] mark: char, #[case] expected: &str) {
        assert_eq!(
            combining_modifier_cells(mark),
            Some([cells(expected)[0], cells(expected)[1]])
        );
    }

    #[test]
    fn rejects_unknown_combining_modifier() {
        assert_eq!(combining_modifier_cells('\u{0303}'), None);
    }

    #[rstest::rstest]
    #[case::alpha('Α', false, "⠠⠨⠁")]
    #[case::beta_suppressed('Β', true, "⠨⠃")]
    #[case::gamma('Γ', true, "⠨⠛")]
    #[case::delta('Δ', true, "⠨⠙")]
    #[case::epsilon('Ε', true, "⠨⠑")]
    #[case::zeta('Ζ', true, "⠨⠵")]
    #[case::eta('Η', true, "⠨⠱")]
    #[case::theta('Θ', true, "⠨⠹")]
    #[case::iota('Ι', true, "⠨⠊")]
    #[case::kappa('Κ', true, "⠨⠅")]
    #[case::lambda('Λ', true, "⠨⠇")]
    #[case::mu('Μ', true, "⠨⠍")]
    #[case::nu('Ν', true, "⠨⠝")]
    #[case::xi('Ξ', true, "⠨⠭")]
    #[case::omicron('Ο', true, "⠨⠕")]
    #[case::pi('Π', true, "⠨⠏")]
    #[case::rho('Ρ', true, "⠨⠗")]
    #[case::phi('Φ', true, "⠨⠋")]
    #[case::sigma('Σ', true, "⠨⠎")]
    #[case::tau('Τ', true, "⠨⠞")]
    #[case::upsilon('Υ', true, "⠨⠥")]
    #[case::chi('Χ', true, "⠨⠯")]
    #[case::psi('Ψ', true, "⠨⠽")]
    #[case::omega('Ω', true, "⠨⠺")]
    #[case::final_sigma('ς', false, "⠨⠎")]
    fn maps_greek_letters_with_capital_policy(
        #[case] input: char,
        #[case] suppress_capital: bool,
        #[case] expected: &str,
    ) {
        assert_eq!(
            greek_letter_cells_with_caps(input, suppress_capital),
            Some(cells(expected))
        );
    }

    #[test]
    fn rejects_non_greek_letter_mapping() {
        assert_eq!(greek_letter_cells_with_caps('A', false), None);
    }

    #[rstest::rstest]
    #[case::caps_word(&['A', 'B'], Some(Caps::Word))]
    #[case::single_cap(&['A'], Some(Caps::Single))]
    #[case::titlecase(&['A', 'b'], Some(Caps::Single))]
    #[case::lower(&['a', 'b'], Some(Caps::None))]
    #[case::mixed_internal(&['a', 'B'], None)]
    fn classifies_capital_patterns(#[case] input: &[char], #[case] expected: Option<Caps>) {
        assert_eq!(classify_caps(input), expected);
    }

    #[test]
    fn emits_ligature_letter_helper() {
        let mut out = Vec::new();
        emit_ligature_between(&['o'], &['e'], &mut out).unwrap();
        assert_eq!(out, cells("⠕⠘⠖⠑"));
    }

    #[test]
    fn emits_word_modifier_on_last_paths() {
        let mut out = Vec::new();
        emit_word_with_modifier_on_last(&['c', 'a', 'f', 'e'], '\u{0301}', &mut out).unwrap();
        assert_eq!(out, cells("⠉⠁⠋⠘⠌⠑"));

        let mut empty = Vec::new();
        assert_eq!(
            emit_word_with_modifier_on_last(&[], '\u{0301}', &mut empty),
            None
        );

        let mut unknown_mark = Vec::new();
        assert_eq!(
            emit_word_with_modifier_on_last(&['a'], '\u{0303}', &mut unknown_mark),
            None
        );
    }

    #[test]
    fn emits_ligature_between_rejection_and_uppercase_paths() {
        let mut uppercase = Vec::new();
        emit_ligature_between(&['o'], &['E'], &mut uppercase).unwrap();
        assert_eq!(uppercase, cells("⠕⠠⠘⠖⠑"));

        let mut out = Vec::new();
        assert_eq!(emit_ligature_between(&[], &['e'], &mut out), None);
        assert_eq!(emit_ligature_between(&['o'], &[], &mut out), None);
    }

    #[test]
    fn encode_rare_greek_grouping_branches() {
        let engine = EnglishUebEngine::new();

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['Α', 'Β']),
                        EnglishToken::Symbol('Γ'),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('Α'),
                        EnglishToken::Word(vec!['Β', 'Γ']),
                    ],
                    false,
                )
                .is_some()
        );
    }

    #[test]
    fn encodes_capitalized_where_before_apostrophe() {
        // §8/§10.5: a capitalized `Where'…` keeps the `where` groupsign `⠱⠻⠑`
        // preceded by a capital indicator.
        let out = enc("Where's").expect("Where's should encode");
        assert_eq!(out.first(), Some(&CAPITAL));
        assert!(out.windows(3).any(|w| w == cells("⠱⠻⠑")));
    }

    #[test]
    fn encodes_uppercase_greek_run_in_english_prose() {
        // §11: an all-caps Greek run (`ΑΒΓ`) inside English prose encodes with the
        // §8.4 capitals passage over the Greek letters.
        assert!(enc("The ΑΒΓ set").is_some());
    }

    #[test]
    fn encodes_styled_uppercase_letter_after_number_with_capital() {
        // §8/§9: an italic/bold uppercase letter directly after a number keeps its
        // capital indicator (`5𝐀`).
        let out = enc("5\u{1D400}").expect("should encode");
        assert!(out.contains(&CAPITAL));
    }

    #[test]
    fn encodes_capitalized_enough_before_sentence_close() {
        // §10.5: a capitalized `Enough` closing a sentence keeps the `enough`
        // wordsign `⠢` with a leading capital indicator.
        let out = enc("Enough.").expect("should encode");
        assert_eq!(out.first(), Some(&CAPITAL));
        assert!(out.contains(&decode_unicode('⠢')));
    }

    #[test]
    fn encodes_capitalized_in_before_ellipsis() {
        // §10.5: a capitalized `In` immediately before an ellipsis keeps the
        // `in` lower groupsign spelled with a leading capital indicator
        // (`spell_lower_in_for_preference`); the lowercase form differs only by
        // that capital cell.
        let upper = enc("In...").expect("should encode");
        let lower = enc("in...").expect("should encode");
        assert_eq!(upper.first(), Some(&CAPITAL));
        assert_eq!(upper.len(), lower.len() + 1);
    }
}
