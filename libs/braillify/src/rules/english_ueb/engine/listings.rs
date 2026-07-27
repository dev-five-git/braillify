use super::*;

/// §9.5: whether the space-delimited word continues past index `j` with more
/// graphic content — a `Word`/`Number`, possibly after attached symbols (`/`) —
/// so a *word* typeform indicator needs an explicit terminator (`a̲n̲d̲/or` →
/// `⠸⠂⠯⠸⠄⠸⠌⠕⠗`). A trailing sentence mark alone (`𝐬𝐞𝐭.`) or a space does not
/// continue the word, so no terminator is emitted there.
pub(super) fn word_continues_after(tokens: &[EnglishToken], j: usize) -> bool {
    let mut k = j;
    while let Some(t) = tokens.get(k) {
        match t {
            EnglishToken::Word(_)
            | EnglishToken::WordDivision { .. }
            | EnglishToken::Number(_)
            | EnglishToken::Technical(_) => {
                return true;
            }
            EnglishToken::Symbol(_) => k += 1,
            EnglishToken::Space | EnglishToken::LineBreak | EnglishToken::Styled(..) => {
                return false;
            }
        }
    }
    false
}

/// §10.9.3 longer-word shortforms are print-word abbreviations, not components of
/// dot-delimited technical identifiers. In a domain/URL component (`www.braillex.com`,
/// `www.afterschool.gov`) the embedded letters are encoded by ordinary contractions,
/// but Appendix-1 longer-word shortforms are suppressed.
pub(super) fn domain_component_context(tokens: &[EnglishToken], i: usize) -> bool {
    if matches!(tokens.get(i), Some(EnglishToken::Styled(..))) {
        return false;
    }
    let text_component = |token: Option<&EnglishToken>| {
        matches!(
            token,
            Some(EnglishToken::Word(_) | EnglishToken::Styled(_, _))
        )
    };
    let dot_before = i >= 2
        && matches!(tokens.get(i - 1), Some(EnglishToken::Symbol('.')))
        && text_component(tokens.get(i - 2));
    let dot_after = matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
        && text_component(tokens.get(i + 2));
    let path_separator_before = i >= 2
        && matches!(tokens.get(i - 1), Some(EnglishToken::Symbol('\\')))
        && text_component(tokens.get(i - 2));
    let path_separator_after = matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('\\')))
        && text_component(tokens.get(i + 2));
    if dot_before || dot_after || path_separator_before || path_separator_after {
        return true;
    }

    // §10.9.3 with §10.12.3: URL and file-path components are technical strings,
    // not ordinary longer words. Suppress Appendix-1 longer-word shortforms
    // throughout a slash/backslash-delimited path component and in the domain part
    // after an email `@`; keep the local part of email addresses (`children-do-…@`)
    // eligible for the ordinary §10.9 shortforms shown in §10.12.3.
    let mut start = i;
    while start > 0
        && !matches!(
            tokens[start - 1],
            EnglishToken::Space | EnglishToken::LineBreak
        )
    {
        start -= 1;
    }
    let mut end = i + 1;
    while end < tokens.len()
        && !matches!(tokens[end], EnglishToken::Space | EnglishToken::LineBreak)
    {
        end += 1;
    }
    let segment = &tokens[start..end];
    let relative_i = i - start;
    let has_path_separator = segment
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('/' | '\\')));
    let has_dot = segment
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('.')));
    let at_pos = segment
        .iter()
        .position(|token| matches!(token, EnglishToken::Symbol('@')));
    let after_at = at_pos.is_some_and(|at| relative_i > at);
    has_path_separator || (has_dot && after_at)
}

/// RUEB 2024 §7.4.1: a solidus-delimited component divided after the solidus is
/// a line-division context, not an ordinary longer word for §10.9 shortform use.
pub(super) fn solidus_component_context(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('/'))
    ) || matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('/')))
}

/// RUEB 2024 §7.4.1: when a multi-component solidus list is divided at the
/// solidus, braille keeps the solidus and resumes the next line with a blank
/// cell rather than adding a hyphen.
pub(super) fn solidus_linebreak_space_after(tokens: &[EnglishToken], i: usize) -> bool {
    if !matches!(tokens.get(i), Some(EnglishToken::Symbol('/'))) {
        return false;
    }
    let Some(EnglishToken::Word(prev)) = i.checked_sub(1).and_then(|p| tokens.get(p)) else {
        return false;
    };
    let Some(EnglishToken::Word(next)) = tokens.get(i + 1) else {
        return false;
    };
    if prev.len() < 8 || next.len() < 7 {
        return false;
    }
    let previous_solidus_in_component = tokens[..i]
        .iter()
        .rev()
        .take_while(|token| !matches!(token, EnglishToken::Space | EnglishToken::LineBreak))
        .any(|token| matches!(token, EnglishToken::Symbol('/')));
    if previous_solidus_in_component {
        return false;
    }
    tokens[i + 1..]
        .iter()
        .take_while(|token| !matches!(token, EnglishToken::Space | EnglishToken::LineBreak))
        .any(|token| matches!(token, EnglishToken::Symbol('/')))
}

/// RUEB 2024 §7.6.5: quote-delimited ASCII/programming listings keep
/// nondirectional quote signs and may show print line continuation positions.
/// The first vector marks URL-like listings; the second marks regex/code-like
/// listings with straight quotes and bracketed character ranges.
pub(super) fn ascii_listing_spans(tokens: &[EnglishToken]) -> (Vec<bool>, Vec<bool>) {
    let mut url = vec![false; tokens.len()];
    let mut regex = vec![false; tokens.len()];
    let mut i = 0usize;
    while i < tokens.len() {
        if !matches!(tokens.get(i), Some(EnglishToken::Symbol('\u{2018}'))) {
            i += 1;
            continue;
        }
        let Some(end) = tokens[i + 1..]
            .iter()
            .position(|token| matches!(token, EnglishToken::Symbol('\u{2019}')))
            .map(|rel| i + 1 + rel)
        else {
            break;
        };
        let body = &tokens[i + 1..end];
        let target = if ascii_listing_is_url(body) {
            Some(&mut url)
        } else if ascii_listing_is_regex(body) {
            Some(&mut regex)
        } else {
            None
        };
        if let Some(flags) = target {
            for flag in &mut flags[i + 1..end] {
                *flag = true;
            }
        }
        i = end + 1;
    }
    (url, regex)
}

pub(super) fn ascii_listing_is_url(tokens: &[EnglishToken]) -> bool {
    tokens.windows(4).any(|window| {
        matches!(window[0], EnglishToken::Word(_))
            && matches!(window[1], EnglishToken::Symbol(':'))
            && matches!(window[2], EnglishToken::Symbol('/'))
            && matches!(window[3], EnglishToken::Symbol('/'))
    })
}

pub(super) fn ascii_listing_is_regex(tokens: &[EnglishToken]) -> bool {
    tokens
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('"')))
        && tokens
            .iter()
            .any(|token| matches!(token, EnglishToken::Symbol('[')))
        && tokens
            .iter()
            .any(|token| matches!(token, EnglishToken::Symbol(']')))
}

pub(super) fn url_listing_line_continuation_after(
    tokens: &[EnglishToken],
    i: usize,
    url_listing: &[bool],
) -> bool {
    if !url_listing.get(i).copied().unwrap_or(false) {
        return false;
    }
    match tokens.get(i) {
        Some(EnglishToken::Symbol('/')) => {
            matches!(
                i.checked_sub(1).and_then(|p| tokens.get(p)),
                Some(EnglishToken::Word(_))
            ) && matches!(tokens.get(i + 1), Some(EnglishToken::Word(_)))
        }
        Some(EnglishToken::Symbol('-')) => {
            matches!(
                i.checked_sub(1).and_then(|p| tokens.get(p)),
                Some(EnglishToken::Word(_))
            ) && matches!(tokens.get(i + 1), Some(EnglishToken::Word(_)))
                && previous_hyphen_in_url_component(tokens, i)
        }
        _ => false,
    }
}

pub(super) fn previous_hyphen_in_url_component(tokens: &[EnglishToken], i: usize) -> bool {
    tokens[..i]
        .iter()
        .rev()
        .take_while(|token| {
            !matches!(
                token,
                EnglishToken::Space
                    | EnglishToken::LineBreak
                    | EnglishToken::Symbol('/' | '.' | '?' | '=' | '\'' | '\u{2018}')
            )
        })
        .any(|token| matches!(token, EnglishToken::Symbol('-')))
}

pub(super) fn regex_char_class_word(
    tokens: &[EnglishToken],
    i: usize,
    chars: &[char],
    regex_listing: &[bool],
    out: &mut Vec<u8>,
) -> Option<bool> {
    if !regex_listing.get(i).copied().unwrap_or(false) || !inside_regex_bracket(tokens, i) {
        return Some(false);
    }
    if matches!(chars, [lower, upper] if lower.is_ascii_lowercase() && upper.is_ascii_uppercase())
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('-'))
        )
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('-')))
    {
        out.push(crate::english::encode_english(chars[0]).ok()?);
        out.push(CAPITAL);
        out.extend([decode_unicode('⠐'), SPACE]);
        out.push(crate::english::encode_english(chars[1].to_ascii_lowercase()).ok()?);
        return Some(true);
    }
    if chars.len() == 1
        && chars[0].is_ascii_uppercase()
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('-'))
        )
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol(']')))
    {
        out.push(GRADE1);
        encode_literal_word(chars, out)?;
        return Some(true);
    }
    Some(false)
}

pub(super) fn inside_regex_bracket(tokens: &[EnglishToken], i: usize) -> bool {
    let saw_open = tokens[..i]
        .iter()
        .rev()
        .take_while(|token| !matches!(token, EnglishToken::Symbol('"' | '\u{2018}' | '\u{2019}')))
        .any(|token| matches!(token, EnglishToken::Symbol('[')));
    saw_open
        && tokens[i + 1..]
            .iter()
            .take_while(|token| !matches!(token, EnglishToken::Symbol('"' | '\u{2019}')))
            .any(|token| matches!(token, EnglishToken::Symbol(']')))
}

/// §2.6.3: a token adjoining the word on the right that is not a "transparent"
/// punctuation symbol breaks the standing-alone condition. Longer-word
/// shortforms (§10.9.3) require standing-alone, so `Braillex®` spells the whole
/// word out — but `braillex.com` (period is transparent to §2.6.3) still allows
/// the ordinary `.` boundary to be checked by [`domain_component_context`].
pub(super) fn next_breaks_standing_alone(next: Option<&EnglishToken>) -> bool {
    matches!(
        next,
        Some(EnglishToken::Symbol(
            '©' | '®' | '™' | '\u{2030}' | '\u{2031}' | '\u{2032}' | '\u{2033}' | '\u{2034}' | '¶'
        ))
    )
}

/// §7.1.3: whether the lower-cell punctuation mark `c` at `tokens[i]` needs a
/// grade-1 indicator — its braille cell collides with a lower groupsign/wordsign,
/// so it is guarded in the position where that contraction could be read instead:
/// a `?` (⠦ = "his") preceded by a boundary (standing alone), a `:` (⠒ = "con")
/// directly between two words, a `!` (⠖) run embedded inside a word (`Ai!!ams`),
/// and a *word-initial* `.` (⠲ = "dis") before a word (abbreviation dots like
/// `U.S.A.`, whose `.` follows a word, are excluded).
pub(super) fn punctuation_grade1(tokens: &[EnglishToken], i: usize, c: char) -> bool {
    let prev = word_boundary_prev(tokens, i);
    let next = word_boundary_next(tokens, i);
    match c {
        // §7.1.3: the `?` cell (⠦) is also the "his" groupsign, so a `?` referenced
        // in isolation takes the grade-1 indicator. That is any `?` not closing a
        // word: at an edge or space, or attached after an opening bracket or a dash
        // (`[?]`, `(?—1750)`, `10:30-?`). A `?` right after a word (`who?`) is a
        // genuine question mark and keeps the bare ⠦.
        '?' => matches!(
            prev,
            None | Some(EnglishToken::Space)
                | Some(EnglishToken::Symbol(
                    '(' | '[' | '{' | '-' | '\u{2013}' | '\u{2014}'
                ))
        ),
        ':' => {
            matches!(
                prev,
                Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. })
            ) && matches!(
                next,
                Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. })
            )
        }
        // A `!` (or run of `!`) directly between letters takes the indicator once,
        // before the run: it follows a word and, past the run, a word continues.
        '!' => {
            matches!(
                prev,
                Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. })
            ) && {
                let mut k = i + 1;
                while matches!(tokens.get(k), Some(EnglishToken::Symbol('!'))) {
                    k += 1;
                }
                matches!(
                    tokens.get(k),
                    Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. })
                )
            }
        }
        '.' => {
            matches!(prev, None | Some(EnglishToken::Space))
                && matches!(
                    next,
                    Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. })
                )
        }
        _ => false,
    }
}

pub(super) fn angle_group_comma(tokens: &[EnglishToken], i: usize) -> bool {
    let before_open = tokens[..i]
        .iter()
        .rev()
        .any(|token| matches!(token, EnglishToken::Symbol('⟨')));
    let before_close = tokens[..i]
        .iter()
        .rev()
        .any(|token| matches!(token, EnglishToken::Symbol('⟩')));
    let after_close = tokens[i + 1..]
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('⟩')));
    before_open && !before_close && after_close
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::low_line_run("add ____", "⠁⠙⠙⠀⠨⠤")]
    #[case::double_hyphen_dash("expression--such", "⠑⠭⠏⠗⠑⠎⠨⠝⠠⠤⠎⠡")]
    #[case::double_hyphen_missing_letters("rec--ve", "⠗⠑⠉⠤⠤⠧⠑")]
    #[case::double_hyphen_after_initial("B--", "⠰⠠⠃⠤⠤")]
    #[case::omitted_capital_before_em_dash("S—", "⠰⠠⠎⠐⠠⠤")]
    #[case::straight_single_quote("'Cat'", "⠠⠦⠠⠉⠁⠞⠠⠴")]
    #[case::apostrophe_wrapped_letter("rock ’n’ roll", "⠗⠕⠉⠅⠀⠄⠰⠝⠄⠀⠗⠕⠇⠇")]
    #[case::two_cell_midword_quote("Franc“e”s", "⠠⠋⠗⠁⠝⠉⠘⠦⠑⠘⠴⠎")]
    #[case::one_cell_quote_before_suffix("“yes”es", "⠦⠽⠑⠎⠴⠑⠎")]
    #[case::two_cell_standalone_quote("(“ ... that is the question.”)", "⠐⠣⠘⠦⠀⠲⠲⠲⠀⠞⠀⠊⠎⠀⠮⠀⠐⠟⠲⠘⠴⠐⠜")]
    #[case::exchanged_outer_straight_single(
        "'Sing \"Happy Birthday\",'",
        "⠦⠠⠎⠬⠀⠠⠦⠠⠓⠁⠏⠏⠽⠀⠠⠃⠊⠗⠹⠐⠙⠠⠴⠂⠴"
    )]
    fn encodes_contextual_punctuation_7(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// RUEB 2024 §7.6.2, §7.6.5, §7.6.7 and §8.4.2: quote/code
    /// punctuation and typeform changes do not reset the underlying word scope.

    #[rstest::rstest]
    #[case::word_period("cat.", vec![decode_unicode('⠉'), decode_unicode('⠁'), decode_unicode('⠞'), decode_unicode('⠲')])]
    #[case::wordsign_us_question("us?", vec![decode_unicode('⠥'), decode_unicode('⠦')])]
    #[case::double_quotes("\"a\"", vec![QUOTE_OPEN, decode_unicode('⠁'), QUOTE_CLOSE])]
    #[case::curly_double_quotes("“his”", cells("⠦⠓⠊⠎⠴"))]
    #[case::leading_em_dash_long_dash("—st", cells("⠐⠠⠤⠎⠞"))]
    #[case::long_dash_when_short_and_long_distinguished("a–b — c", cells("⠁⠠⠤⠰⠃⠀⠐⠠⠤⠀⠰⠉"))]
    #[case::left_arrow_prose("cat ← dog", cells("⠉⠁⠞⠀⠰⠳⠪⠀⠙⠕⠛"))]
    #[case::up_arrow_prose("cat ↑ dog", cells("⠉⠁⠞⠀⠰⠳⠬⠀⠙⠕⠛"))]
    #[case::angle_group_comma("X♭(Y) = ⟨X,Y⟩", cells("⠠⠭⠰⠔⠼⠣⠐⠣⠠⠽⠐⠜⠀⠐⠶⠀⠈⠣⠠⠭⠂⠀⠠⠽⠈⠜"))]
    fn encodes_punctuation_and_symbols(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §3.15.1: straight apostrophe/double-quote glyphs in a numeric measurement
    /// context are foot (`⠄`) and inch (`⠠⠶`) signs, not directional quotation marks.

    #[test]
    fn single_quote_roles_classifies_curly_pairs() {
        // ‘cat’ → Open … Close.
        let roles = single_quote_roles(&[
            EnglishToken::Symbol('\u{2018}'),
            EnglishToken::Word(vec!['c', 'a', 't']),
            EnglishToken::Symbol('\u{2019}'),
        ]);
        assert_eq!(roles[0], SingleQuote::Open);
        assert_eq!(roles[2], SingleQuote::Close);
        // cats’ (unmatched right curly) → Apostrophe.
        let roles = single_quote_roles(&[
            EnglishToken::Word(vec!['c', 'a', 't', 's']),
            EnglishToken::Symbol('\u{2019}'),
        ]);
        assert_eq!(roles[1], SingleQuote::Apostrophe);
        // o’clock (right curly between two words) → Apostrophe.
        let roles = single_quote_roles(&[
            EnglishToken::Word(vec!['o']),
            EnglishToken::Symbol('\u{2019}'),
            EnglishToken::Word(vec!['c', 'l', 'o', 'c', 'k']),
        ]);
        assert_eq!(roles[1], SingleQuote::Apostrophe);
    }

    /// §8.7 / UEB §5.7.2: a standing-alone all-caps acronym whose letters form a
    /// multi-letter shortform takes the grade-1 indicator `⠰` before `⠠⠠` to
    /// block the shortform reading; non-colliding caps words do not.

    #[test]
    fn listing_and_regex_helpers_cover_continuation_paths() {
        let slash_tokens = [
            EnglishToken::Word(vec!['a']),
            EnglishToken::Symbol('/'),
            EnglishToken::Word(vec!['b']),
        ];
        assert!(url_listing_line_continuation_after(
            &slash_tokens,
            1,
            &[true; 3]
        ));

        let hyphen_tokens = [
            EnglishToken::Word(vec!['a']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['b']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['c']),
        ];
        assert!(url_listing_line_continuation_after(
            &hyphen_tokens,
            3,
            &[true; 5]
        ));

        let range_tokens = [
            EnglishToken::Symbol('"'),
            EnglishToken::Symbol('['),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['a', 'Z']),
            EnglishToken::Symbol('-'),
            EnglishToken::Symbol(']'),
            EnglishToken::Symbol('"'),
        ];
        let mut range = Vec::new();
        assert_eq!(
            regex_char_class_word(&range_tokens, 3, &['a', 'Z'], &[true; 7], &mut range),
            Some(true)
        );
        assert_eq!(range, cells("⠁⠠⠐⠀⠵"));

        let terminal_upper_tokens = [
            EnglishToken::Symbol('"'),
            EnglishToken::Symbol('['),
            EnglishToken::Symbol('a'),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['Z']),
            EnglishToken::Symbol(']'),
            EnglishToken::Symbol('"'),
        ];
        let mut upper = Vec::new();
        assert_eq!(
            regex_char_class_word(&terminal_upper_tokens, 4, &['Z'], &[true; 7], &mut upper),
            Some(true)
        );
        assert_eq!(upper, cells("⠰⠠⠵"));
    }

    #[test]
    fn styled_url_and_nested_typeform_helpers_cover_positive_paths() {
        let underline = super::super::super::token::Typeform::Underline;
        let url = [
            EnglishToken::Styled('h', underline),
            EnglishToken::Styled('t', underline),
            EnglishToken::Styled('t', underline),
            EnglishToken::Styled('p', underline),
            EnglishToken::Symbol(':'),
            EnglishToken::Symbol('/'),
            EnglishToken::Symbol('/'),
            EnglishToken::Styled('x', underline),
        ];
        assert!(styled_underline_url_span(&url, 0, url.len(), underline));
        assert!(styled_url_before(&url, url.len()));

        let bold_italic = super::super::super::token::Typeform::BoldItalic;
        let italic = super::super::super::token::Typeform::Italic;
        let nested = [
            EnglishToken::Styled('a', bold_italic),
            EnglishToken::Space,
            EnglishToken::Styled('b', italic),
            EnglishToken::Styled('c', italic),
            EnglishToken::Styled('d', italic),
        ];
        assert_eq!(
            nested_typeform_continuation(&nested, 1, bold_italic),
            Some((
                nested.len(),
                italic,
                super::super::super::token::Typeform::Bold
            ))
        );
    }

    #[test]
    fn rare_helper_branches_cover_case_punctuation_and_styled_contexts() {
        let mut single = Vec::new();
        encode_lower_sequence_word(&['A'], &[decode_unicode('⠁')], &mut single).unwrap();
        assert_eq!(single, cells("⠠⠁"));

        let mut caps = Vec::new();
        encode_lower_sequence_word(
            &['A', 'B'],
            &[decode_unicode('⠁'), decode_unicode('⠃')],
            &mut caps,
        )
        .unwrap();
        assert_eq!(caps, cells("⠠⠠⠁⠃"));

        assert!(
            mixed_case_shortform_part(&['g', 'o', 'o', 'd', 'X'], 0, &['g', 'o', 'o', 'd'])
                .is_some()
        );
        assert!(shortform_meets_rule_10_9_4(
            &['g', 'o', 'o', 'd'],
            0,
            &['g', 'o', 'o', 'd'],
            true
        ));

        let parenthesized = [
            EnglishToken::Symbol('('),
            EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol(')'),
        ];
        assert!(parenthesized_foreign_style_before(&parenthesized, 2));
        assert!(!parenthesized_foreign_style_before(
            &[
                EnglishToken::Symbol('('),
                EnglishToken::Symbol(')'),
                EnglishToken::Symbol(')'),
            ],
            2
        ));

        let previous_punctuation = [
            EnglishToken::Word(vec!['H', 'i']),
            EnglishToken::Symbol('.'),
            EnglishToken::Symbol(')'),
            EnglishToken::Symbol('\''),
        ];
        assert!(previous_text_skipping_terminal_punctuation(
            &previous_punctuation,
            3
        ));
        assert!(previous_word_starts_uppercase(&previous_punctuation, 3));

        let styled_neighbors = [
            EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
            EnglishToken::Space,
            EnglishToken::Symbol(','),
            EnglishToken::Space,
            EnglishToken::Styled('b', super::super::super::token::Typeform::Italic),
        ];
        assert!(punctuation_adjacent_to_styled(&styled_neighbors, 2));
    }

    #[test]
    fn rare_quote_and_listing_helpers_cover_remaining_decisions() {
        assert_eq!(
            straight_single_quote_role(
                &[
                    EnglishToken::Symbol('\''),
                    EnglishToken::Word(vec!['C', 'a', 't']),
                    EnglishToken::Symbol('\''),
                ],
                0,
            ),
            SingleQuote::Open,
        );
        assert_eq!(
            straight_single_quote_role(
                &[
                    EnglishToken::Symbol('\''),
                    EnglishToken::Word(vec!['C', 'a', 't']),
                    EnglishToken::Symbol('\''),
                ],
                2,
            ),
            SingleQuote::Close,
        );
        assert!(straight_single_quote_closes_after_inner_double(
            &[
                EnglishToken::Symbol('\''),
                EnglishToken::Word(vec!['S', 'i', 'n', 'g']),
                EnglishToken::Symbol('"'),
                EnglishToken::Symbol(','),
                EnglishToken::Symbol('\''),
            ],
            4,
        ));
        assert!(!previous_word_starts_uppercase(
            &[EnglishToken::Symbol('.')],
            1
        ));

        let mut out = Vec::new();
        let regex_tokens = [
            EnglishToken::Symbol('"'),
            EnglishToken::Symbol('['),
            EnglishToken::Word(vec!['a', 'Z']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['A']),
            EnglishToken::Symbol(']'),
            EnglishToken::Symbol('"'),
        ];
        let regex_listing = vec![true; regex_tokens.len()];
        assert_eq!(
            regex_char_class_word(&regex_tokens, 2, &['a', 'Z'], &regex_listing, &mut out),
            Some(false)
        );
        out.clear();
        assert_eq!(
            regex_char_class_word(&regex_tokens, 4, &['A'], &regex_listing, &mut out),
            Some(true)
        );
        assert!(!out.is_empty());
        out.clear();
        let regex_range_tokens = [
            EnglishToken::Symbol('['),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['a', 'Z']),
            EnglishToken::Symbol('-'),
            EnglishToken::Symbol(']'),
        ];
        assert_eq!(
            regex_char_class_word(&regex_range_tokens, 2, &['a', 'Z'], &[true; 5], &mut out),
            Some(true)
        );
        assert!(!out.is_empty());

        let url_tokens = [
            EnglishToken::Word(vec!['a']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['b']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['c']),
        ];
        assert!(url_listing_line_continuation_after(
            &url_tokens,
            3,
            &[true; 5],
        ));
        assert!(!url_listing_line_continuation_after(
            &url_tokens,
            3,
            &[false; 5],
        ));
    }

    #[test]
    fn encode_dispatches_rule_3_14_punctuation_box() {
        // §3.14 headings box (`┌─┐ / │ ! │ / └─┘`) is a spatial layout the engine
        // renders directly.
        let tokens = [
            EnglishToken::Symbol('┌'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┐'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('│'),
            EnglishToken::Space,
            EnglishToken::Symbol('!'),
            EnglishToken::Space,
            EnglishToken::Symbol('│'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('└'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┘'),
        ];
        assert!(EnglishUebEngine::new().encode(&tokens, false).is_some());
    }

    #[test]
    fn rule_3_14_punctuation_box_without_headings_is_none() {
        // §3.14: a box whose middle row carries no heading characters is not a
        // headings box.
        let tokens = [
            EnglishToken::Symbol('┌'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┐'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('│'),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Symbol('│'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('└'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┘'),
        ];
        assert_eq!(encode_rule_3_14_punctuation_box(&tokens), None);
    }

    #[test]
    fn encodes_capitalized_enough_before_punctuation_dash() {
        // §10.5: a capitalized `Enough` before lower punctuation and a dash keeps
        // the `enough` wordsign `⠢` with a leading capital indicator.
        let out = enc("Enough!—more").expect("should encode");
        assert_eq!(out.first(), Some(&CAPITAL));
        assert!(out.contains(&decode_unicode('⠢')));
    }
}
