use super::*;

/// Classify each *curly* single quote — `‘` (U+2018) and `’` (U+2019) — as an
/// opening or closing single quotation mark, or an apostrophe (§7.6).
///
/// A left curly `‘` always opens. A right curly `’` is a *closing* quote when it
/// matches an open on the stack; an apostrophe when it sits between two words
/// (`o’clock`); and otherwise a word-final possessive/elision apostrophe
/// (`Jones’`, `be’`, `rock ’n’ roll`). This matched-pair test is what
/// distinguishes `’` in `mother-‘in-law’` (paired → closing quote) from `’` in
/// `Jones’` (unpaired → apostrophe).
///
/// The straight quote `'` (U+0027) is deliberately *not* classified here: it is
/// genuinely ambiguous in print — a quoted `'Hamlet'` and an apostrophe-delimited
/// `'display will minimise'` are indistinguishable — so it stays an apostrophe
/// (the dominant reading) on the default punctuation path.
pub(super) fn single_quote_roles(tokens: &[EnglishToken]) -> Vec<SingleQuote> {
    let mut roles = vec![SingleQuote::Apostrophe; tokens.len()];
    // Indices of opening curly single quotes still awaiting their close (LIFO).
    let mut open_stack: Vec<usize> = Vec::new();
    let adjacent_text = |t: Option<&EnglishToken>| {
        matches!(
            t,
            Some(
                EnglishToken::Word(_)
                    | EnglishToken::Number(_)
                    | EnglishToken::Styled(..)
                    | EnglishToken::Technical(_),
            )
        )
    };
    for i in 0..tokens.len() {
        match &tokens[i] {
            EnglishToken::Symbol('\u{2018}') => {
                roles[i] = SingleQuote::Open;
                open_stack.push(i);
            }
            EnglishToken::Symbol('\u{2019}') => {
                let prev_text = i > 0 && adjacent_text(tokens.get(i - 1));
                let next_text = adjacent_text(tokens.get(i + 1));
                roles[i] = if prev_text && next_text {
                    // Between two words → apostrophe (`o'clock`).
                    SingleQuote::Apostrophe
                } else if open_stack.pop().is_some() {
                    // Closing side of a matched pair.
                    SingleQuote::Close
                } else if prev_text || next_text {
                    // Unmatched but touching a word → possessive/elision apostrophe
                    // (`Jones'`, `be'`, `'Tis`).
                    SingleQuote::Apostrophe
                } else {
                    // Unmatched and fully detached (space/edge both sides) → a
                    // standalone closing single quote referenced in isolation
                    // (§7.6.10), e.g. "forget the ' at the end".
                    SingleQuote::Close
                };
            }
            _ => {}
        }
    }
    roles
}

pub(super) fn text_token(token: Option<&EnglishToken>) -> bool {
    matches!(
        token,
        Some(
            EnglishToken::Word(_)
                | EnglishToken::Number(_)
                | EnglishToken::Styled(..)
                | EnglishToken::Technical(_),
        )
    )
}

pub(super) fn previous_text_skipping_terminal_punctuation(
    tokens: &[EnglishToken],
    index: usize,
) -> bool {
    let mut k = index;
    while let Some(prev) = k.checked_sub(1) {
        match tokens.get(prev) {
            Some(EnglishToken::Symbol('.' | ',' | ':' | ';' | '!' | '?' | ')' | ']' | '}')) => {
                k = prev;
            }
            token => return text_token(token),
        }
    }
    false
}

pub(super) fn straight_single_quote_role(tokens: &[EnglishToken], index: usize) -> SingleQuote {
    let prev_text = previous_text_skipping_terminal_punctuation(tokens, index);
    let next_text = text_token(tokens.get(index + 1));
    match (prev_text, next_text) {
        (true, true) => SingleQuote::Apostrophe,
        _ if !straight_single_quote_is_matched_quotation(tokens, index) => SingleQuote::Apostrophe,
        (false, true) => SingleQuote::Open,
        (true, false) | (false, false) => SingleQuote::Close,
    }
}

pub(super) fn straight_single_quote_is_matched_quotation(
    tokens: &[EnglishToken],
    index: usize,
) -> bool {
    let Some(EnglishToken::Symbol('\'')) = tokens.get(index) else {
        return false;
    };
    let prev_text = previous_text_skipping_terminal_punctuation(tokens, index);
    let next_text = text_token(tokens.get(index + 1));
    if !prev_text && next_text {
        return next_word_starts_uppercase(tokens.get(index + 1))
            && tokens[index + 1..]
                .iter()
                .any(|t| matches!(t, EnglishToken::Symbol('\'')));
    }
    if prev_text && !next_text {
        return previous_word_starts_uppercase(tokens, index)
            && tokens[..index]
                .iter()
                .any(|t| matches!(t, EnglishToken::Symbol('\'')));
    }
    if straight_single_quote_closes_after_inner_double(tokens, index) {
        return tokens[..index]
            .iter()
            .any(|t| matches!(t, EnglishToken::Symbol('\'')));
    }
    false
}

pub(super) fn straight_single_quote_closes_after_inner_double(
    tokens: &[EnglishToken],
    index: usize,
) -> bool {
    let mut k = index;
    let mut skipped_comma = false;
    while let Some(prev) = k.checked_sub(1) {
        match tokens.get(prev) {
            Some(EnglishToken::Symbol(',')) => {
                skipped_comma = true;
                k = prev;
            }
            Some(EnglishToken::Symbol('.' | ':' | ';' | '!' | '?' | ')' | ']' | '}')) => {
                k = prev;
            }
            Some(EnglishToken::Symbol('"' | '\u{201D}')) => {
                return skipped_comma && previous_text_skipping_terminal_punctuation(tokens, prev);
            }
            _ => return false,
        }
    }
    false
}

pub(super) fn next_word_starts_uppercase(token: Option<&EnglishToken>) -> bool {
    matches!(token, Some(EnglishToken::Word(chars)) if chars.first().is_some_and(|c| c.is_uppercase()))
}

pub(super) fn prev_word_starts_uppercase(token: Option<&EnglishToken>) -> bool {
    next_word_starts_uppercase(token)
}

pub(super) fn previous_word_starts_uppercase(tokens: &[EnglishToken], index: usize) -> bool {
    let mut k = index;
    while let Some(prev) = k.checked_sub(1) {
        match tokens.get(prev) {
            Some(EnglishToken::Symbol('.' | ',' | ':' | ';' | '!' | '?' | ')' | ']' | '}')) => {
                k = prev;
            }
            token => return prev_word_starts_uppercase(token),
        }
    }
    false
}

pub(super) fn straight_single_quote_exchanged(tokens: &[EnglishToken], index: usize) -> bool {
    if !matches!(tokens.get(index), Some(EnglishToken::Symbol('\''))) {
        return false;
    }
    let role = straight_single_quote_role(tokens, index);
    match role {
        SingleQuote::Apostrophe => false,
        SingleQuote::Open | SingleQuote::Close => {
            let has_double = tokens
                .iter()
                .any(|t| matches!(t, EnglishToken::Symbol('"')));
            has_double
                && tokens
                    .iter()
                    .filter(|t| matches!(t, EnglishToken::Symbol('\'')))
                    .count()
                    >= 2
        }
    }
}

pub(super) fn double_quote_needs_two_cell(
    tokens: &[EnglishToken],
    index: usize,
    opening: bool,
) -> bool {
    if opening {
        if matches!(
            index.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('–' | '—'))
        ) {
            return false;
        }
        return index > 0
            && !matches!(
                tokens.get(index - 1),
                Some(EnglishToken::Space | EnglishToken::LineBreak)
            );
    }
    let paired_two_cell_open = tokens[..index]
        .iter()
        .rposition(|t| matches!(t, EnglishToken::Symbol('\u{201C}')))
        .is_some_and(|open| double_quote_needs_two_cell(tokens, open, true));
    let detached = index == 0
        || matches!(
            tokens.get(index - 1),
            Some(EnglishToken::Space | EnglishToken::LineBreak)
        );
    detached || paired_two_cell_open
}

/// RUEB 2024 §7.6.7: an escaped quotation mark in program text uses the
/// two-cell quote, and the quoted code snippet is transcribed letter-for-letter.
pub(super) fn escaped_quote_code_span(tokens: &[EnglishToken]) -> Vec<bool> {
    let mut span = vec![false; tokens.len()];
    let mut active = false;
    let mut i = 0usize;
    while i < tokens.len() {
        if matches!(tokens.get(i), Some(EnglishToken::Symbol('\\'))) {
            let next = tokens.get(i + 1);
            if !active && matches!(next, Some(EnglishToken::Symbol('"' | '\u{201C}'))) {
                active = true;
                i += 2;
                continue;
            }
            if active && matches!(next, Some(EnglishToken::Symbol('"' | '\u{201D}'))) {
                active = false;
                i += 2;
                continue;
            }
        }
        if active {
            span[i] = true;
        }
        i += 1;
    }
    span
}

pub(super) fn apostrophe_wrapped_letter(
    tokens: &[EnglishToken],
    index: usize,
    chars: &[char],
) -> bool {
    // §5.7.1 example `'n' Ma` — an isolated lowercase letter wrapped by
    // apostrophes (`rock 'n' roll`) takes the grade-1 indicator. A capital
    // letter in a caps sequence like `FO'C'S'LE` (§8.4.2) does not — the
    // capital indicator is unambiguous there.
    chars.len() == 1
        && chars[0].is_ascii_lowercase()
        && matches!(
            index.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('\'' | '\u{2019}'))
        )
        && matches!(
            tokens.get(index + 1),
            Some(EnglishToken::Symbol('\'' | '\u{2019}'))
        )
}

/// §3.27: detect a transcriber's-note marker `[open tn]` / `[close tn]` starting
/// at `i`. The print convention spells the boundary as those bracketed words; in
/// braille it is a single note indicator — `⠈⠨⠣` to open, `⠈⠨⠜` to close (the
/// square-bracket signs `⠨⠣`/`⠨⠜` under a dot-4 prefix). Returns `(is_open,
/// next_index)` on a match so the five marker tokens are replaced as a unit.
pub(super) fn transcriber_note_at(tokens: &[EnglishToken], i: usize) -> Option<(bool, usize)> {
    let word_is = |t: Option<&EnglishToken>, s: &str| matches!(t, Some(EnglishToken::Word(w)) if w.iter().collect::<String>() == s);
    if !matches!(tokens.get(i), Some(EnglishToken::Symbol('['))) {
        return None;
    }
    if matches!(tokens.get(i + 2), Some(EnglishToken::Space))
        && word_is(tokens.get(i + 3), "tn")
        && matches!(tokens.get(i + 4), Some(EnglishToken::Symbol(']')))
    {
        if word_is(tokens.get(i + 1), "open") {
            return Some((true, i + 5));
        }
        if word_is(tokens.get(i + 1), "close") {
            return Some((false, i + 5));
        }
    }
    if word_is(tokens.get(i + 1), "tn")
        && matches!(tokens.get(i + 2), Some(EnglishToken::Space))
        && matches!(tokens.get(i + 4), Some(EnglishToken::Symbol(']')))
    {
        if word_is(tokens.get(i + 3), "open") {
            return Some((true, i + 5));
        }
        if word_is(tokens.get(i + 3), "close") {
            return Some((false, i + 5));
        }
    }
    None
}

pub(super) fn transcriber_note_ends_at(tokens: &[EnglishToken], end: usize, is_open: bool) -> bool {
    end.checked_sub(5)
        .and_then(|start| transcriber_note_at(tokens, start))
        .is_some_and(|(open, note_end)| open == is_open && note_end == end)
}

pub(super) fn closing_transcriber_note_starts_at(tokens: &[EnglishToken], i: usize) -> bool {
    transcriber_note_at(tokens, i).is_some_and(|(is_open, _)| !is_open)
}

pub(super) fn closing_transcriber_note_after_transparent_suffix(
    tokens: &[EnglishToken],
    i: usize,
) -> bool {
    let mut j = i + 1;
    while matches!(
        tokens.get(j),
        Some(EnglishToken::Symbol(
            ',' | ';' | ':' | '.' | '!' | '?' | ')' | ']' | '}' | '"' | '”' | '’'
        ))
    ) {
        j += 1;
    }
    j > i + 1 && closing_transcriber_note_starts_at(tokens, j)
}

pub(super) fn word_boundary_prev(tokens: &[EnglishToken], i: usize) -> Option<&EnglishToken> {
    if transcriber_note_ends_at(tokens, i, true) {
        None
    } else {
        i.checked_sub(1).map(|p| &tokens[p])
    }
}

pub(super) fn word_boundary_next(tokens: &[EnglishToken], i: usize) -> Option<&EnglishToken> {
    if closing_transcriber_note_starts_at(tokens, i + 1) {
        None
    } else {
        tokens.get(i + 1)
    }
}

pub(super) fn braille_mention_at(tokens: &[EnglishToken], i: usize) -> Option<(Vec<u8>, usize)> {
    let mut j = i;
    let mut cells = Vec::new();
    while let Some(EnglishToken::Symbol(c)) = tokens.get(j) {
        if !('\u{2800}'..='\u{28ff}').contains(c) {
            break;
        }
        cells.push(decode_unicode(*c));
        j += 1;
    }
    if cells.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(cells.len() + 2);
    out.extend([decode_unicode('⠨'), decode_unicode('⠿')]);
    out.extend(cells);
    Some((out, j))
}

pub(super) fn isolated_shape_circle(tokens: &[EnglishToken], i: usize, chars: &[char]) -> bool {
    chars == ['o']
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            None | Some(EnglishToken::LineBreak)
        )
        && matches!(tokens.get(i + 1), Some(EnglishToken::Space))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Word(w)) if w.first().is_some_and(|c| c.is_uppercase()))
}

pub(super) fn script_letter(c: char) -> Option<(super::super::rule_3_24::ScriptKind, char)> {
    use super::super::rule_3_24::ScriptKind::{Subscript, Superscript};
    Some(match c {
        '\u{1D50}' => (Superscript, 'm'), // ᵐ
        '\u{1D9C}' => (Superscript, 'c'), // ᶜ
        '\u{2090}' => (Subscript, 'a'),   // ₐ
        '\u{2091}' => (Subscript, 'e'),   // ₑ
        '\u{2095}' => (Subscript, 'h'),   // ₕ
        '\u{1D62}' => (Subscript, 'i'),   // ᵢ
        '\u{2C7C}' => (Subscript, 'j'),   // ⱼ
        '\u{2096}' => (Subscript, 'k'),   // ₖ
        '\u{2097}' => (Subscript, 'l'),   // ₗ
        '\u{2098}' => (Subscript, 'm'),   // ₘ
        '\u{2099}' => (Subscript, 'n'),   // ₙ
        '\u{2092}' => (Subscript, 'o'),   // ₒ
        '\u{209A}' => (Subscript, 'p'),   // ₚ
        '\u{1D63}' => (Subscript, 'r'),   // ᵣ
        '\u{209B}' => (Subscript, 's'),   // ₛ
        '\u{209C}' => (Subscript, 't'),   // ₜ
        '\u{1D64}' => (Subscript, 'u'),   // ᵤ
        '\u{1D65}' => (Subscript, 'v'),   // ᵥ
        '\u{2093}' => (Subscript, 'x'),   // ₓ
        _ => return None,
    })
}

pub(super) fn script_kind(c: char) -> Option<super::super::rule_3_24::ScriptKind> {
    super::super::rule_3_24::script_digit(c)
        .map(|(kind, _)| kind)
        .or_else(|| script_letter(c).map(|(kind, _)| kind))
}

pub(super) fn encode_chemical_formula_scripts(tokens: &[EnglishToken]) -> Option<Vec<u8>> {
    let has_script = tokens.iter().any(
        |t| matches!(t, EnglishToken::Symbol(c) if super::super::rule_3_24::script_digit(*c).is_some()),
    );
    if !has_script {
        return None;
    }
    let mut out = Vec::new();
    for token in tokens {
        match token {
            EnglishToken::Word(chars)
                if chars
                    .iter()
                    .all(|c| c.is_ascii_uppercase() && c.is_ascii_alphabetic()) =>
            {
                for &c in chars {
                    out.push(CAPITAL);
                    out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                }
            }
            EnglishToken::Symbol(c) => {
                let (kind, digit) = super::super::rule_3_24::script_digit(*c)?;
                out.extend([
                    GRADE1,
                    kind.indicator(),
                    super::super::rule_6::NUMERIC_INDICATOR,
                ]);
                out.push(super::super::rule_6::digit_cell(digit)?);
            }
            _ => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::wrapped_note("[open tn]cat[close tn]", "⠈⠨⠣⠉⠁⠞⠈⠨⠜")]
    #[case::wrapped_note_reverse_words("[tn open]cat[tn close]", "⠈⠨⠣⠉⠁⠞⠈⠨⠜")]
    #[case::plain_bracket_unchanged("[cat]", "⠨⠣⠉⠁⠞⠨⠜")]
    fn encodes_transcriber_notes_3_27(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §3.8, §3.13, §3.22, §3.26, §3.28: general print/braille symbols are handled
    /// before broader technical or phonetic symbol fallbacks can claim them.

    #[test]
    fn encodes_straight_quote_measurements_3_15_1() {
        assert_eq!(enc("4' 11\""), Some(cells("⠼⠙⠄⠀⠼⠁⠁⠠⠶")));
    }

    /// §7.1.3: a lower-cell punctuation mark whose cell collides with a lower
    /// contraction takes a grade-1 indicator ⠰ where that contraction could be read
    /// — a standing-alone `?` (⠦/his), a word-internal `:` (⠒/con), a word-initial
    /// `.` (⠲/dis). It stays bare in its plain terminal position (`us?`, `cat.`
    /// above) and as an abbreviation dot (`U.S.A.`, covered by §5.7.1 tests).

    #[rstest::rstest]
    #[case::curly_pair_is_quotation("\u{2018}cat\u{2019}", "⠠⠦⠉⠁⠞⠠⠴")]
    #[case::unmatched_curly_close_is_apostrophe("cats\u{2019}", "⠉⠁⠞⠎⠄")]
    #[case::straight_quotes_stay_apostrophe("'cat'", "⠄⠉⠁⠞⠄")]
    #[case::detached_open_takes_grade1("\u{2018} cat\u{2019}", "⠰⠠⠦⠀⠉⠁⠞⠠⠴")]
    #[case::detached_close_takes_grade1("\u{2018}cat \u{2019}", "⠠⠦⠉⠁⠞⠀⠰⠠⠴")]
    #[case::standalone_close_takes_grade1("cat \u{2019} dog", "⠉⠁⠞⠀⠰⠠⠴⠀⠙⠕⠛")]
    fn encodes_single_quotes_7_6(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §7.6.10: a double quotation mark standing alone (space/edge both sides) is
    /// the mark referenced in isolation → ⠰⠠⠶, without flipping the open/close
    /// alternation; a normal dialogue pair still toggles ⠦ … ⠴.

    #[rstest::rstest]
    #[case::standalone_double_quote("cat \" dog", "⠉⠁⠞⠀⠰⠠⠶⠀⠙⠕⠛")]
    #[case::dialogue_double_quote_toggles("\"cat\"", "⠦⠉⠁⠞⠴")]
    fn encodes_standalone_double_quote_7_6_10(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §7.6 matched-pair classification: a left curly `‘` opens; a right curly `’`
    /// closes when it matches an open, otherwise is an apostrophe; a `’` between two
    /// words is an apostrophe.

    #[rstest::rstest]
    #[case::empty(&[], false)]
    #[case::short_word(&[EnglishToken::Word(vec!['A', 'A', 'A', 'A'])], false)]
    #[case::two_words(&[EnglishToken::Word(vec!['H', 'E', 'Y', 'Y', 'Y']), EnglishToken::Word(vec!['A'])], false)]
    #[case::elongated(&[EnglishToken::Word(vec!['H', 'E', 'Y', 'Y', 'Y'])], true)]
    fn detects_single_elongated_caps_word_in_quotes(
        #[case] tokens: &[EnglishToken],
        #[case] expected: bool,
    ) {
        assert_eq!(single_elongated_caps_word_in_quotes(tokens), expected);
    }

    #[test]
    fn straight_single_quote_helpers_cover_quotation_roles() {
        let opened = [
            EnglishToken::Symbol('\''),
            EnglishToken::Word(vec!['H', 'i']),
            EnglishToken::Symbol('\''),
        ];
        assert!(straight_single_quote_is_matched_quotation(&opened, 0));
        assert!(matches!(
            straight_single_quote_role(&opened, 0),
            SingleQuote::Open
        ));
        assert!(matches!(
            straight_single_quote_role(&opened, 2),
            SingleQuote::Close
        ));

        let inner_double_close = [
            EnglishToken::Symbol('\''),
            EnglishToken::Word(vec!['H', 'i']),
            EnglishToken::Symbol('"'),
            EnglishToken::Symbol(','),
            EnglishToken::Symbol('\''),
        ];
        assert!(straight_single_quote_closes_after_inner_double(
            &inner_double_close,
            4
        ));
        assert!(straight_single_quote_exchanged(&inner_double_close, 4));
    }

    #[test]
    fn inner_double_quote_close_requires_an_opening_quote() {
        let terminal_punctuation = [EnglishToken::Symbol('!'), EnglishToken::Symbol(',')];

        assert!(!straight_single_quote_closes_after_inner_double(
            &terminal_punctuation,
            terminal_punctuation.len(),
        ));
    }

    #[test]
    fn rare_helper_branches_cover_styled_sequences_and_quotes() {
        let bold = super::super::super::token::Typeform::Bold;
        let numeric = [
            EnglishToken::Styled('1', bold),
            EnglishToken::Space,
            EnglishToken::Styled('5', bold),
        ];
        assert_eq!(
            styled_numeric_sequence_end(&numeric, 0, bold),
            numeric.len()
        );
        let mut numeric_out = Vec::new();
        encode_styled_numeric_sequence(&numeric, 0, numeric.len(), bold, &mut numeric_out).unwrap();
        assert_eq!(
            numeric_out,
            vec![decode_unicode('⠼'), 1, decode_unicode('⠐'), 17]
        );

        let invalid_numeric = [
            EnglishToken::Styled('1', bold),
            EnglishToken::Word(vec!['x']),
        ];
        let mut invalid_numeric_out = Vec::new();
        assert_eq!(
            encode_styled_numeric_sequence(
                &invalid_numeric,
                0,
                invalid_numeric.len(),
                bold,
                &mut invalid_numeric_out,
            ),
            None
        );

        let symbol_tail = [
            EnglishToken::Styled('R', bold),
            EnglishToken::Symbol('.'),
            EnglishToken::LineBreak,
            EnglishToken::Word(vec!['S']),
        ];
        assert!(styled_capital_starts_symbol_sequence(&symbol_tail, 0, 1));
        assert!(!styled_capital_starts_symbol_sequence(&symbol_tail, 3, 1));

        let adjacent_text = [
            EnglishToken::Word(vec!['A']),
            EnglishToken::Symbol('\''),
            EnglishToken::Word(vec!['B']),
        ];
        assert!(matches!(
            straight_single_quote_role(&adjacent_text, 1),
            SingleQuote::Apostrophe
        ));
        assert!(!straight_single_quote_exchanged(&adjacent_text, 1));
    }

    #[test]
    fn dash_after_quoted_in_before_in_requires_quote_then_in() {
        // A dash not reached through a quotation mark from `in` is not the seam.
        let bare = [EnglishToken::Space, EnglishToken::Symbol('\u{2014}')];
        assert!(!dash_after_quoted_in_before_in(&bare, 2));
    }
}
