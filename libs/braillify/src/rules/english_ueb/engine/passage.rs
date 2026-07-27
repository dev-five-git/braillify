use super::*;

/// §10.4.3: whether a word token preceded by `prev` begins a fresh word.
pub(super) fn word_initial_boundary(prev: Option<&EnglishToken>) -> bool {
    matches!(
        prev,
        None | Some(EnglishToken::Space | EnglishToken::LineBreak)
            | Some(EnglishToken::Symbol('-' | '\u{2013}' | '\u{2014}'))
    )
}

/// §10.6.2: restricted `be`/`con`/`dis` may start after opening punctuation and
/// indicators listed by §2.6.2, but not after slash or internal case splits.
pub(super) fn restricted_prefix_boundary(prev: Option<&EnglishToken>) -> bool {
    matches!(
        prev,
        None | Some(EnglishToken::Space | EnglishToken::LineBreak)
            | Some(EnglishToken::Symbol(
                '-' | '\u{2013}'
                    | '\u{2014}'
                    | '('
                    | '['
                    | '{'
                    | '"'
                    | '\''
                    | '\u{2018}'
                    | '\u{201c}'
                    | '«'
            ))
    )
}

pub(super) fn spell_line_division_in(tokens: &[EnglishToken], i: usize, lower_word: &str) -> bool {
    if lower_word != "in" {
        return false;
    }
    let prev = i.checked_sub(1).and_then(|p| tokens.get(p));
    let prev2 = i.checked_sub(2).and_then(|p| tokens.get(p));
    let next = tokens.get(i + 1);
    let next2 = tokens.get(i + 2);
    let parenthesized_enough_dash = matches!(
        prev2,
        Some(EnglishToken::Symbol('-' | '\u{2013}' | '\u{2014}'))
    ) && matches!(prev, Some(EnglishToken::LineBreak))
        && matches!(i.checked_sub(3).and_then(|p| tokens.get(p)), Some(EnglishToken::Word(w)) if w.iter().collect::<String>().eq_ignore_ascii_case("enough"))
        && matches!(
            i.checked_sub(4).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('('))
        );
    let quoted_break = matches!(prev, Some(EnglishToken::Symbol('"' | '“')))
        && matches!(next, Some(EnglishToken::Symbol('-')))
        && matches!(tokens.get(i + 2), Some(EnglishToken::LineBreak));
    let dash_linebreak = matches!(
        (prev2, prev),
        (
            Some(EnglishToken::Symbol('-' | '\u{2013}' | '\u{2014}')),
            Some(EnglishToken::LineBreak)
        ) | (
            Some(EnglishToken::LineBreak),
            Some(EnglishToken::Symbol('\u{2013}' | '\u{2014}'))
        )
    ) && !matches!(next, Some(EnglishToken::Symbol('-')))
        && !matches!(
            (next, next2),
            (
                Some(EnglishToken::Symbol('.')),
                Some(EnglishToken::Symbol(')' | ']' | '}'))
            )
        )
        && !parenthesized_enough_dash;
    quoted_break || dash_linebreak
}

pub(super) fn spell_lower_in_for_preference(tokens: &[EnglishToken], i: usize) -> bool {
    let next = tokens.get(i + 1);
    let ellipsis_follows = matches!(next, Some(EnglishToken::Symbol('.')))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Symbol('.')));
    ellipsis_follows
        || dash_after_enough_before_in(tokens, i)
        || dash_after_quoted_in_before_in(tokens, i)
}

pub(super) fn dash_after_enough_before_in(tokens: &[EnglishToken], i: usize) -> bool {
    if !matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('–' | '—'))
    ) {
        return false;
    }
    let mut k = i.saturating_sub(2);
    loop {
        match tokens.get(k) {
            Some(EnglishToken::Word(w)) => {
                return w.iter().collect::<String>().eq_ignore_ascii_case("enough");
            }
            Some(EnglishToken::Symbol('!' | '?' | '"' | '”' | '\u{2019}')) if k > 0 => k -= 1,
            _ => return false,
        }
    }
}

pub(super) fn spell_in_for_lower_wordsign_limit(tokens: &[EnglishToken], i: usize) -> bool {
    let prev = i.checked_sub(1).and_then(|p| tokens.get(p));
    let prev2 = i.checked_sub(2).and_then(|p| tokens.get(p));
    let next = tokens.get(i + 1);
    let after_line_division_hyphen = matches!(
        (prev2, prev),
        (
            Some(EnglishToken::Symbol('-' | '\u{2013}' | '\u{2014}')),
            Some(EnglishToken::LineBreak)
        )
    );
    let terminal_lower_punctuation =
        matches!(next, Some(EnglishToken::Symbol(',' | '.'))) && !after_line_division_hyphen;
    let quoted_by_lower_signs = matches!(prev, Some(EnglishToken::Symbol('"' | '“')))
        && !matches!(prev2, Some(EnglishToken::Symbol('(' | '[' | '{')))
        && matches!(
            next,
            Some(EnglishToken::Space | EnglishToken::Symbol('"' | '”'))
        )
        && !lower_quote_sequence_reaches_dash(tokens, i + 1);
    terminal_lower_punctuation || quoted_by_lower_signs
}

pub(super) fn standalone_hyphen_in(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-'))
    ) && matches!(
        i.checked_sub(2).and_then(|p| tokens.get(p)),
        None | Some(EnglishToken::Space)
    ) && matches!(tokens.get(i + 1), None | Some(EnglishToken::Space))
}

pub(super) fn lower_quote_sequence_reaches_dash(tokens: &[EnglishToken], mut k: usize) -> bool {
    loop {
        match tokens.get(k) {
            Some(EnglishToken::Symbol('!' | '?' | '"' | '”' | '\u{2019}')) => k += 1,
            Some(EnglishToken::Symbol('–' | '—')) => return true,
            _ => return false,
        }
    }
}

pub(super) fn dash_after_quoted_in_before_in(tokens: &[EnglishToken], i: usize) -> bool {
    if !matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('–' | '—'))
    ) {
        return false;
    }
    let mut k = i.saturating_sub(2);
    let mut saw_quote_or_lower_punctuation = false;
    loop {
        match tokens.get(k) {
            Some(EnglishToken::Word(w)) => {
                return saw_quote_or_lower_punctuation
                    && w.iter().collect::<String>().eq_ignore_ascii_case("in");
            }
            Some(EnglishToken::Symbol('!' | '?' | '"' | '”' | '\u{2019}')) if k > 0 => {
                saw_quote_or_lower_punctuation = true;
                k -= 1;
            }
            _ => return false,
        }
    }
}

pub(super) fn enough_followed_by_upper_dot_sequence(tokens: &[EnglishToken], i: usize) -> bool {
    let mut k = i + 1;
    let mut saw_lower_punctuation = false;
    loop {
        match tokens.get(k) {
            Some(EnglishToken::Symbol('!' | '?' | '"' | '”' | '\u{2019}')) => {
                saw_lower_punctuation = true;
                k += 1;
            }
            Some(EnglishToken::Symbol('–' | '—')) => return saw_lower_punctuation,
            _ => return false,
        }
    }
}

pub(super) fn enough_followed_by_sentence_close(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
        && matches!(
            tokens.get(i + 2),
            Some(EnglishToken::Symbol(')' | ']' | '}'))
        )
}

pub(super) fn styled_lower_wordsign_usable(
    lower_word: &str,
    prev: Option<&EnglishToken>,
    next: Option<&EnglishToken>,
) -> bool {
    lower_wordsign_usable(prev, next)
        || (matches!(lower_word, "be" | "were" | "was")
            && matches!(
                next,
                None | Some(
                    EnglishToken::Space
                        | EnglishToken::Symbol(
                            ')' | ']'
                                | '}'
                                | '?'
                                | '!'
                                | '.'
                                | ','
                                | ';'
                                | ':'
                                | '"'
                                | '\u{201D}'
                                | '\''
                                | '\u{2019}'
                        )
                )
            ))
}

pub(super) fn styled_scansion_word(tokens: &[EnglishToken], lower_word: &str) -> bool {
    lower_word == "be"
        && tokens
            .iter()
            .any(|t| matches!(t, EnglishToken::Symbol('/')))
}

pub(super) fn lower_contact_after_division_word(token: Option<&EnglishToken>) -> bool {
    matches!(
        token,
        Some(EnglishToken::Symbol(
            '"' | '\'' | '”' | '’' | '?' | '!' | '.'
        ))
    )
}

pub(super) fn touches_hyphen_or_line_break(
    prev: Option<&EnglishToken>,
    next: Option<&EnglishToken>,
) -> bool {
    matches!(
        prev,
        Some(EnglishToken::Symbol('-' | '–' | '—') | EnglishToken::LineBreak)
    ) || matches!(
        next,
        Some(EnglishToken::Symbol('-' | '–' | '—') | EnglishToken::LineBreak)
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::hyphen_bounded_x("I like x–it works.", "⠠⠊⠀⠇⠀⠰⠭⠠⠤⠭⠀⠐⠺⠎⠲")]
    #[case::ellipsis_keeps_ch_groupsign("ch...f", "⠡⠲⠲⠲⠋")]
    #[case::word_script_digit("knowledge.³", "⠐⠅⠇⠫⠛⠑⠲⠰⠔⠼⠉")]
    #[case::single_curly_quote_standalone(
        "Use single quotes ‘ and ’.",
        "⠠⠥⠎⠑⠀⠎⠬⠇⠑⠀⠟⠥⠕⠞⠑⠎⠀⠰⠠⠦⠀⠯⠀⠠⠴⠲"
    )]
    fn encodes_rule2_6_boundaries(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §5.4.1/§5.9.1: a technical expression spanning three or more spaced
    /// symbol-sequences uses grade-1 passage mode, even when its terms are not
    /// hyphenated spelling sequences.

    #[rstest::rstest]
    #[case::equation_terms("a=b c=d e=f", "⠰⠰⠰⠁⠐⠶⠃⠀⠉⠐⠶⠙⠀⠑⠐⠶⠋⠰⠄")]
    fn technical_sequences_open_grade1_passage_5(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.1/§10.4/§10.5/§10.9 with §9: typeform indicators may cover a
    /// single symbol, word, or passage while the underlying letters still take
    /// the ordinary wordsign/shortform decisions.

    #[rstest::rstest]
    #[case::curly_quote_spelling_run(
        "note silent letters in n-i-‘g-h’-t",
        "⠝⠕⠞⠑⠀⠎⠊⠇⠢⠞⠀⠇⠗⠎⠀⠔⠀⠰⠰⠝⠤⠊⠤⠠⠦⠛⠤⠓⠠⠴⠤⠞"
    )]
    #[case::solidus_linebreak_keeps_space(
        "There were several schoolchildren/teachers/parents present.",
        "⠠⠐⠮⠀⠶⠀⠎⠐⠑⠁⠇⠀⠎⠡⠕⠕⠇⠡⠊⠇⠙⠗⠢⠸⠌⠀⠞⠂⠡⠻⠎⠸⠌⠏⠜⠢⠞⠎⠀⠏⠗⠑⠎⠢⠞⠲"
    )]
    #[case::url_ascii_quote_listing(
        "‘https://www.example.com/query?item='bobs-internal-folder'.’",
        "⠠⠦⠓⠞⠞⠏⠎⠒⠸⠌⠸⠌⠺⠺⠺⠲⠑⠭⠁⠍⠏⠇⠑⠲⠉⠕⠍⠸⠌⠐⠀⠟⠥⠻⠽⠦⠊⠞⠑⠍⠐⠶⠄⠃⠕⠃⠎⠤⠔⠞⠻⠝⠁⠇⠤⠐⠀⠋⠕⠇⠙⠻⠄⠲⠠⠴"
    )]
    #[case::regex_ascii_quote_listing(
        "“Is she correct in saying our regex pattern would be ‘\"?+[a-zA-Z]\"?’?”",
        "⠦⠠⠊⠎⠀⠩⠑⠀⠉⠕⠗⠗⠑⠉⠞⠀⠔⠀⠎⠁⠽⠬⠀⠳⠗⠀⠗⠑⠛⠑⠭⠀⠏⠁⠞⠞⠻⠝⠀⠺⠙⠀⠆⠀⠠⠦⠠⠶⠰⠦⠐⠖⠨⠣⠁⠤⠵⠠⠐⠀⠁⠤⠰⠠⠵⠨⠜⠠⠶⠦⠠⠴⠦⠴"
    )]
    #[case::escaped_quote_code_snippet(
        "\\“Remember those backslashes\\”",
        "⠸⠡⠘⠦⠠⠗⠑⠍⠑⠍⠃⠑⠗⠀⠞⠓⠕⠎⠑⠀⠃⠁⠉⠅⠎⠇⠁⠎⠓⠑⠎⠸⠡⠘⠴"
    )]
    #[case::caps_word_continues_across_bold_tail("FREE𝐅𝐎𝐑𝐌", "⠠⠠⠋⠗⠑⠑⠘⠂⠿⠍")]
    #[case::italic_caps_heading_is_one_caps_passage(
        "𝐿𝐼𝑆𝑇 𝑂𝐹 𝑆𝑈𝑅𝑉𝐸𝑌 𝑅𝐸𝐶𝐼𝑃𝐼𝐸𝑁𝑇𝑆 𝑂𝑅𝐺𝐴𝑁𝐼𝑆𝐸𝐷 𝐵𝑌 𝐶𝑂𝑈𝑁𝑇𝑅𝑌",
        "⠨⠶⠠⠠⠠⠇⠊⠌⠀⠷⠀⠎⠥⠗⠧⠑⠽⠀⠗⠑⠉⠊⠏⠊⠢⠞⠎⠀⠕⠗⠛⠁⠝⠊⠎⠫⠀⠃⠽⠀⠉⠨⠞⠗⠽⠠⠄⠨⠄"
    )]
    #[case::italic_title_with_plain_modified_middle_word("𝑉𝑜𝑦𝑎𝑔𝑒 À 𝑁𝑖𝑐𝑒", "⠨⠶⠠⠧⠕⠽⠁⠛⠑⠀⠠⠘⠡⠁⠀⠠⠝⠊⠉⠑⠨⠄")]
    #[case::domain_camel_title_subunit_keeps_usual_braille_form(
        "www.BLASTSoundMachine.com",
        "⠺⠺⠺⠲⠠⠠⠃⠇⠁⠌⠠⠎⠨⠙⠠⠍⠁⠡⠔⠑⠲⠉⠕⠍"
    )]
    fn encodes_ueb_7_8_indicator_scope_regressions(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §8.5.3: capitalised passages may include single-letter words and Greek
    /// capitals; a three-plus symbol-sequence passage uses ⠠⠠⠠ … ⠠⠄.

    #[rstest::rstest]
    #[case::teach_in_period("teach-\nin.", "⠞⠂⠡⠤\n⠊⠝⠲")]
    #[case::quoted_in_depth("\"In-\ndepth", "⠦⠠⠊⠝⠤\n⠙⠑⠏⠹")]
    #[case::enough_dash_in("Enough—\nin my case", "⠠⠢⠳⠣⠠⠤\n⠊⠝⠀⠍⠽⠀⠉⠁⠎⠑")]
    #[case::enough_break_dash_in("Enough\n—in my case", "⠠⠢\n⠠⠤⠊⠝⠀⠍⠽⠀⠉⠁⠎⠑")]
    fn encodes_line_division_lower_sign_rule_10_13(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.13.4: `ing` at the start of the second braille line is spelled as
    /// `in`+`g`, including a capitalised second segment.

    #[rstest::rstest]
    #[case::after_hyphen("b-1", "⠰⠃⠤⠼⠁")]
    #[case::free_standing_paren("(h)", "⠐⠣⠰⠓⠐⠜")]
    #[case::attached_paren("noun(s)", "⠝⠳⠝⠐⠣⠎⠐⠜")]
    #[case::abbreviation_dots("U.S.A.", "⠠⠥⠲⠠⠎⠲⠠⠁⠲")]
    #[case::period_ends_run("p. 7", "⠰⠏⠲⠀⠼⠛")]
    #[case::abbreviation_dot_digit("p.7", "⠏⠲⠼⠛")]
    fn grade1_single_letter_5_7_1(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §5.3/§5.9/§5.10: extended grade-1 mode begins at the start of a
    /// hyphenated symbols-sequence, avoiding repeated single-letter indicators in
    /// spelling and stammering examples.

    #[rstest::rstest]
    #[case::word_indicator_spelling("u-n-t-i-d-y", "⠰⠰⠥⠤⠝⠤⠞⠤⠊⠤⠙⠤⠽")]
    #[case::choice_unemotional("un-e-mo-tion-al", "⠰⠰⠥⠝⠤⠑⠤⠍⠕⠤⠞⠊⠕⠝⠤⠁⠇")]
    #[case::choice_stammer("br-r-r-r", "⠰⠰⠃⠗⠤⠗⠤⠗⠤⠗")]
    #[case::choice_embedded_stammer("about-f-f-f-face", "⠁⠃⠤⠰⠰⠋⠤⠋⠤⠋⠤⠋⠁⠉⠑")]
    #[case::optional_equivalent_grade1("rm-mm-mm-mm", "⠰⠰⠗⠍⠤⠍⠍⠤⠍⠍⠤⠍⠍")]
    #[case::optional_repeated_tail("r-mmmmmmm", "⠰⠰⠗⠤⠍⠍⠍⠍⠍⠍⠍")]
    #[case::passage_spelled_name("H-o C-h-i M-i-n-h City", "⠰⠰⠰⠠⠓⠤⠕⠀⠠⠉⠤⠓⠤⠊⠀⠠⠍⠤⠊⠤⠝⠤⠓⠰⠄⠀⠠⠉⠰⠽")]
    fn grade1_word_indicator_for_hyphenated_sequences_5_3_5_9_5_10(
        #[case] text: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9: a styled letter takes a symbol-level typeform indicator before its base
    /// cell (italic ⠨⠆, bold ⠘⠆, underline ⠸⠆) and is a contraction boundary, so
    /// the plain neighbours still contract (`story̲` keeps its `st` groupsign).

    #[rstest::rstest]
    #[case::italic_y_wordsign("\u{1D466}", "⠨⠆⠰⠽")]
    #[case::italic_i_exempt("\u{1D456}", "⠨⠆⠊")]
    fn typeform_single_letter_grade1_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9.5: a *word* typeform indicator is terminated when the emphasis ends
    /// before the space-delimited word does — including across attached
    /// punctuation, so the underlined `and` in `a̲n̲d̲/or` closes with `⠸⠄` before
    /// the plain `/or` completes the word.

    #[rstest::rstest]
    #[case::colon_between_words("a:o", "⠁⠰⠒⠕")]
    #[case::colon_in_word("lang:uk", "⠇⠁⠝⠛⠰⠒⠥⠅")]
    #[case::word_initial_period(".doc", "⠰⠲⠙⠕⠉")]
    #[case::standalone_question("cat ? dog", "⠉⠁⠞⠀⠰⠦⠀⠙⠕⠛")]
    #[case::embedded_exclamation("Ai!!ams", "⠠⠁⠊⠰⠖⠖⠁⠍⠎")]
    fn punctuation_grade1_7_1_3(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §7.6: a *curly* single quote is an opening (`⠠⠦`) or closing (`⠠⠴`) single
    /// quotation mark only as part of a matched pair; an unmatched right curly is a
    /// word-final apostrophe (`⠄`). The straight `'` is ambiguous in print
    /// (`'Hamlet'` vs `'display will minimise'`) so it always stays an apostrophe.
    /// §7.6.10: a single quote *detached* from its text by a space (or referenced
    /// in isolation) takes a leading grade-1 indicator `⠰`.

    #[rstest::rstest]
    // `CD` = "could" shortform → ⠰⠠⠠CD.
    #[case::cd_collides("CD", vec![GRADE1, CAPITAL, CAPITAL, decode_unicode('⠉'), decode_unicode('⠙')])]
    // `XY` is not a shortform → plain ⠠⠠XY.
    #[case::xy_no_collision("XY", vec![CAPITAL, CAPITAL, decode_unicode('⠭'), decode_unicode('⠽')])]
    fn caps_shortform_grade1(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §6.3: within letter-containing input the numeric indicator `⠼` restarts
    /// after a letter splits a digit run. (Pure-number inputs with `,`/`.`
    /// separators have no ASCII letter and are delegated to the legacy path — see
    /// `non_letter_input_delegated_to_legacy`.)

    #[rstest::rstest]
    #[case::listen_in("listen-in", "⠇⠊⠌⠢⠤⠔")]
    #[case::come_in_comma("Come in, stay in.", "⠠⠉⠕⠍⠑⠀⠊⠝⠂⠀⠌⠁⠽⠀⠊⠝⠲")]
    #[case::quoted_in_no_dash("“in”", "⠦⠊⠝⠴")]
    #[case::quoted_in_dash_in("‘Is that “in”?–in style.’", "⠠⠦⠠⠊⠎⠀⠞⠀⠦⠔⠴⠦⠠⠤⠊⠝⠀⠌⠽⠇⠑⠲⠠⠴")]
    #[case::enough_dash_in("\"That's enough!\"–in a firm voice", "⠦⠠⠞⠄⠎⠀⠢⠖⠴⠠⠤⠊⠝⠀⠁⠀⠋⠊⠗⠍⠀⠧⠕⠊⠉⠑")]
    #[case::paren_quote_in("(\"In no way.\")", "⠐⠣⠦⠠⠔⠀⠝⠕⠀⠺⠁⠽⠲⠴⠐⠜")]
    fn lower_sign_sequences_10_5(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §8.4 capitals passage (3+ all-caps words) vs §8.3 capital word (1–2).

    #[rstest::rstest]
    #[case::four_single(&[&['w'][..], &['a'][..], &['l'][..], &['k'][..]], true)]
    #[case::tail_single(&[&['b', 'r'][..], &['r'][..], &['r'][..], &['r'][..]], true)]
    #[case::same_tail(&[&['s', 'o'][..], &['o', 'o'][..], &['o', 'o'][..], &['o', 'o'][..]], true)]
    #[case::one_then_long_same(&[&['r'][..], &['m', 'm', 'm', 'm'][..]], true)]
    #[case::with_which(&[&['n', 'o', 't'][..], &['w', 'i', 't', 'h'][..], &['s', 't', 'a', 'n', 'd'][..], &['i', 'n', 'g'][..], &['x'][..]], false)]
    fn grade1_hyphenated_word_indicator_paths(#[case] words: &[&[char]], #[case] expected: bool) {
        assert_eq!(grade1_hyphenated_words_use_word_indicator(words), expected);
    }

    #[test]
    fn grade1_hyphenated_span_and_stammer_helpers_cover_edges() {
        let tokens = [
            EnglishToken::Word(vec!['w']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['a']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['l']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['k']),
        ];
        let span = grade1_hyphenated_word_span(&tokens, 0).expect("spelling run should span");
        assert_eq!(span.end, tokens.len());
        assert_eq!(span.indicator_cells, 2);

        assert!(!same_letters(&[]));
        assert!(!repeated_single_letter_prefix(
            &[&['f'][..], &['f'][..]],
            &['f', 'a']
        ));
        assert!(!repeated_single_letter_prefix(
            &[&[][..], &['f'][..], &['f'][..]],
            &['f', 'a'],
        ));
    }

    #[test]
    fn spatial_helpers_encode_grade1_rows_and_symbols() {
        let engine = EnglishUebEngine::new();
        let mut chars = Vec::new();
        push_spatial_char(&mut chars, ' ').unwrap();
        push_spatial_char(&mut chars, '╳').unwrap();
        push_spatial_char(&mut chars, '>').unwrap();
        push_spatial_char(&mut chars, '<').unwrap();
        assert_eq!(chars, cells("⠀⠜⠠⠜⠠⠣"));

        let grade1_rows = encode_spatial_rows(&["╱╲", " ╳"], true).unwrap();
        assert_eq!(grade1_rows, cells("⠐⠐⠿⠰⠰⠰\n⠜⠣\n⠀⠜\n⠐⠐⠿⠰⠄"));

        let mut unsupported = Vec::new();
        assert_eq!(push_spatial_char(&mut unsupported, 'x'), None);

        let cross_gap = [
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┼'),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('╲'),
        ];
        let encoded = engine.encode(&cross_gap, false).unwrap();
        assert_eq!(encoded, cells("⠐⠒⠺⠀⠐⠒⠣"));

        let game_board = [
            EnglishToken::Symbol('╲'),
            EnglishToken::LineBreak,
            EnglishToken::Word(vec!['X']),
            EnglishToken::Space,
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┼'),
            EnglishToken::Symbol('─'),
            EnglishToken::Space,
            EnglishToken::Word(vec!['O']),
        ];
        let encoded = engine.encode(&game_board, false).unwrap();
        assert!(encoded.starts_with(&cells("⠐⠐⠿⠰⠰⠰\n")));
        assert!(encoded.ends_with(&cells("\n⠰⠄")));
        assert!(encoded.contains(&decode_unicode('⠭')));
        assert!(encoded.contains(&decode_unicode('⠕')));
    }

    #[test]
    fn encode_rare_document_level_symbol_paths() {
        let engine = EnglishUebEngine::new();

        assert_eq!(
            engine.encode(
                &[EnglishToken::Symbol('-'), EnglishToken::Symbol('-')],
                false
            ),
            Some(cells("⠐⠒⠒⠒"))
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Number(vec!['2']),
                        EnglishToken::Space,
                        EnglishToken::Symbol('×'),
                        EnglishToken::Space,
                        EnglishToken::Number(vec!['3']),
                    ],
                    true,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['H']),
                        EnglishToken::Symbol('₂'),
                        EnglishToken::Symbol('+'),
                        EnglishToken::Word(vec!['O']),
                        EnglishToken::Symbol('→'),
                        EnglishToken::Word(vec!['H']),
                        EnglishToken::Symbol('₂'),
                        EnglishToken::Word(vec!['O']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('.'),
                        EnglishToken::Number(vec!['3', '7']),
                    ],
                    false,
                )
                .is_some()
        );
    }

    #[test]
    fn rare_helper_branches_cover_lower_sign_and_foreign_word_paths() {
        assert!(!spell_line_division_in(
            &[EnglishToken::Word(vec!['o', 'u', 't'])],
            0,
            "out"
        ));

        let enough = [
            EnglishToken::Word(vec!['e', 'n', 'o', 'u', 'g', 'h']),
            EnglishToken::Symbol('!'),
            EnglishToken::Symbol('”'),
            EnglishToken::Symbol('—'),
            EnglishToken::Word(vec!['i', 'n']),
        ];
        assert!(dash_after_enough_before_in(&enough, 4));

        let quoted_in = [
            EnglishToken::Word(vec!['i', 'n']),
            EnglishToken::Symbol('?'),
            EnglishToken::Symbol('”'),
            EnglishToken::Symbol('—'),
            EnglishToken::Word(vec!['i', 'n']),
        ];
        assert!(dash_after_quoted_in_before_in(&quoted_in, 4));

        assert!(!space_delimited_syllables_form_word(
            &[EnglishToken::Word(vec!['a'])],
            0
        ));
        assert!(foreign_en_spells_letters(None, Some(&EnglishToken::Space)));
        assert!(!styled_word_is_foreign(&['c', 'h']));
        assert!(!styled_single_word_is_foreign(&['t', 'h']));
        assert!(styled_word_has_foreign_signal(&['c', 'h', 'a', 'o', 's']));
    }

    #[test]
    fn rare_document_and_modified_word_helpers_cover_remaining_branches() {
        let italic = super::super::super::token::Typeform::Italic;
        let bold = super::super::super::token::Typeform::Bold;
        let underline = super::super::super::token::Typeform::Underline;
        let bold_italic = super::super::super::token::Typeform::BoldItalic;

        let adjacent = [
            EnglishToken::Styled('c', italic),
            EnglishToken::Space,
            EnglishToken::Symbol('?'),
        ];
        assert!(punctuation_adjacent_to_styled(&adjacent, 2));
        assert!(document_any_styled_phrase_has_foreign_letter(&[
            EnglishToken::Styled('é', italic),
        ]));
        assert!(document_all_styled_phrases_are_short_vocabulary(&[
            EnglishToken::Styled('l', italic),
            EnglishToken::Styled('o', italic),
            EnglishToken::Symbol('-'),
            EnglishToken::Styled('e', italic),
            EnglishToken::Styled('i', italic),
            EnglishToken::Space,
            EnglishToken::Styled('d', italic),
            EnglishToken::Styled('e', italic),
        ]));
        assert!(!document_all_styled_phrases_are_short_vocabulary(&[
            EnglishToken::Styled('T', italic),
            EnglishToken::Styled('H', italic),
            EnglishToken::Styled('E', italic),
        ]));

        assert_eq!(typeform_word_lengths(&[]), Vec::<usize>::new());
        assert_eq!(
            typeform_word_lengths(&[
                EnglishToken::Styled('l', bold),
                EnglishToken::Symbol('\''),
                EnglishToken::Styled('o', bold),
                EnglishToken::Symbol('-'),
                EnglishToken::Styled('e', bold),
                EnglishToken::Space,
                EnglishToken::Word(vec!['x']),
            ]),
            vec![3]
        );

        let mut out = Vec::new();
        let engine = ContractionEngine::default();
        encode_modified_word(&engine, &['a', 'é', 'a'], true, true, &mut out)
            .expect("modified word should encode");
        assert!(!out.is_empty());

        for (left, right, expected) in [
            ('e', 'a', '⠂'),
            ('b', 'b', '⠆'),
            ('c', 'c', '⠒'),
            ('f', 'f', '⠖'),
            ('g', 'g', '⠶'),
        ] {
            assert_eq!(
                middle_lower_pair_cell(left, right),
                Some(decode_unicode(expected))
            );
        }
        assert_eq!(middle_lower_pair_cell('x', 'x'), None);

        assert!(!styled_url_before(
            &[
                EnglishToken::Styled('h', underline),
                EnglishToken::Word(vec!['x']),
            ],
            1,
        ));
        assert!(!styled_url_before(
            &[
                EnglishToken::Styled('h', underline),
                EnglishToken::Symbol(':'),
                EnglishToken::Word(vec!['x']),
            ],
            2,
        ));
        assert_eq!(
            nested_typeform_continuation(
                &[
                    EnglishToken::Styled('a', bold_italic),
                    EnglishToken::Space,
                    EnglishToken::Word(vec!['x']),
                ],
                1,
                bold_italic,
            ),
            None
        );
        assert!(!styled_underline_url_span(
            &[
                EnglishToken::Styled('h', underline),
                EnglishToken::Word(vec!['x']),
            ],
            0,
            2,
            underline,
        ));
        assert!(styled_letter_needs_grade1(
            &[
                EnglishToken::Symbol('('),
                EnglishToken::Styled('x', italic),
                EnglishToken::Symbol(')'),
            ],
            1,
            2,
        ));
    }

    #[test]
    fn document_all_styled_phrases_short_vocabulary_flushes_at_boundaries() {
        use super::super::super::token::Typeform;
        let it = Typeform::Italic;
        // Short lowercase styled words separated by a space → all short vocabulary.
        let ok = [
            EnglishToken::Styled('a', it),
            EnglishToken::Styled('b', it),
            EnglishToken::Space,
            EnglishToken::Styled('c', it),
        ];
        assert!(document_all_styled_phrases_are_short_vocabulary(&ok));
        // A styled word longer than 10 chars fails the flush at the trailing space.
        let long_then_space: Vec<EnglishToken> = "abcdefghijkl"
            .chars()
            .map(|c| EnglishToken::Styled(c, it))
            .chain([EnglishToken::Space])
            .collect();
        assert!(!document_all_styled_phrases_are_short_vocabulary(
            &long_then_space
        ));
        // A non-space/non-symbol token (a Number) after a too-long styled run hits
        // the catch-all flush arm.
        let long_then_number: Vec<EnglishToken> = "abcdefghijkl"
            .chars()
            .map(|c| EnglishToken::Styled(c, it))
            .chain([EnglishToken::Number(vec!['1'])])
            .collect();
        assert!(!document_all_styled_phrases_are_short_vocabulary(
            &long_then_number
        ));
    }

    #[test]
    fn encodes_styled_letter_a_to_j_after_number_with_grade1() {
        // §6.5/§9: an italic letter a–j directly after a number takes a grade-1
        // indicator so `5𝑎` is not misread as a continuation of the number.
        let out = enc("5\u{1D44E}").expect("should encode");
        assert!(out.contains(&GRADE1));
    }
}
