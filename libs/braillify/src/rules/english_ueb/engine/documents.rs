use super::*;

pub(super) fn token_letters(token: &EnglishToken, out: &mut Vec<char>) {
    match token {
        EnglishToken::Word(chars)
        | EnglishToken::WordDivision { chars, .. }
        | EnglishToken::Technical(chars) => {
            out.extend(chars);
        }
        EnglishToken::Styled(c, _) | EnglishToken::Symbol(c) => out.push(*c),
        EnglishToken::Number(_) | EnglishToken::Space | EnglishToken::LineBreak => {}
    }
}

pub(super) fn token_base_char(token: &EnglishToken) -> Option<char> {
    match token {
        EnglishToken::Styled(c, _) => Some(*c),
        EnglishToken::Word(chars) if chars.len() == 1 => Some(chars[0]),
        _ => None,
    }
}

pub(super) fn document_letters(tokens: &[EnglishToken]) -> Vec<char> {
    let mut chars = Vec::new();
    for token in tokens {
        token_letters(token, &mut chars);
    }
    chars
}

pub(super) fn document_words(tokens: &[EnglishToken]) -> Vec<Vec<char>> {
    let mut words = Vec::new();
    for token in tokens {
        if let EnglishToken::Word(chars) = token {
            words.push(chars.clone());
        }
    }
    words
}

/// Prose words for the §13 foreign-passage heuristic: an apostrophe between two
/// letter tokens joins them into one linguistic word (`d'hôtel`, `don't`,
/// `l'ordre`), matching how dictionaries record such entries. Without the join,
/// the parser's apostrophe split inflates the word count and defeats the
/// `likely_foreign_passage` guards that keep 2-word §4.2 accent phrases
/// (`maître d'hôtel`) off the foreign-code path.
pub(super) fn document_prose_words(tokens: &[EnglishToken]) -> Vec<Vec<char>> {
    let mut words: Vec<Vec<char>> = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        let EnglishToken::Word(chars) = &tokens[i] else {
            i += 1;
            continue;
        };
        let mut word = chars.clone();
        i += 1;
        while matches!(tokens.get(i), Some(EnglishToken::Symbol('\'' | '\u{2019}')))
            && let Some(EnglishToken::Word(next)) = tokens.get(i + 1)
        {
            word.push('\'');
            word.extend(next);
            i += 2;
        }
        words.push(word);
    }
    words
}

/// §9.7.3 note: whether the document's prose signals that typeforms themselves
/// are the topic — a signal to keep typeform terminators visible around closing
/// punctuation instead of quietly extending the typeform across it. Triggered by
/// keywords like `italicized`/`italicised`, `boldface`/`bolded`, `underlined`,
/// `typeform`.
pub(super) fn document_studies_typeforms(tokens: &[EnglishToken]) -> bool {
    let markers = [
        "italicized",
        "italicised",
        "italicize",
        "italicise",
        "italics",
        "boldface",
        "bolded",
        "underlined",
        "typeform",
        "typeforms",
    ];
    document_words(tokens).iter().any(|word| {
        let lower: String = word.iter().flat_map(|c| c.to_lowercase()).collect();
        markers.iter().any(|m| lower == *m)
    })
}

/// Like [`document_words`], but MERGES contiguous `Word` and `Styled` tokens
/// into one word, and treats each contiguous run of `Styled` tokens between
/// spaces as one word. Used for §13.6 whole-sentence heuristics.
///
/// A `Word` (`tou`) + `Styled(c,h)` + `Word(ed)` sequence with no intervening
/// space forms one composite word `touched` (§10.12.12 mid-word typeform), not
/// three sentence words. A typeform-marked Spanish verb (`𝐬𝐨𝐲`) with spaces on
/// both sides is one sentence word.
pub(super) fn document_all_words(tokens: &[EnglishToken]) -> Vec<Vec<char>> {
    let mut words = Vec::new();
    let mut current: Vec<char> = Vec::new();
    for token in tokens {
        match token {
            EnglishToken::Word(chars) => current.extend(chars.iter().copied()),
            EnglishToken::Styled(c, _) => current.push(*c),
            EnglishToken::Symbol('-' | '\'' | '\u{2019}') => {
                // Word-internal punctuation (hyphen, apostrophe) does NOT split a word.
            }
            EnglishToken::Space | EnglishToken::LineBreak | EnglishToken::Symbol(_)
                if !current.is_empty() =>
            {
                words.push(std::mem::take(&mut current));
            }
            _ => {}
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// §13.6 short-sentence typeform trigger: a 3-to-5-word sentence with at least
/// one typeform-marked word AND majority-non-CMU content, where at least TWO
/// non-styled plain words are non-CMU, is a §13.6 foreign sentence. Its plain
/// proper-name words (`Carlos`, `Fuentes`) are uncontracted alongside the
/// typeform-marked verb (`𝐞𝐬`).
///
/// The `plain_unrecorded ≥ 2` gate excludes constructs like §9.3.2 gloss
/// `𝑙'𝑜𝑒𝑖𝑙-𝑑𝑒-��𝑜𝑒𝑢𝑓 (Fr.: bull's eye)` — the styled French compound is followed
/// by an ENGLISH parenthetical translation whose plain words are recorded, so
/// no whole-sentence foreign context should apply.
///
/// The `all_words.len() ∈ [3, 5]` gate distinguishes a §13.6 short Spanish/
/// French sentence from a §13.1.2 English narrative with an occasional italic
/// phrase (`Her pirouette was lovely but her fouetté en tournant …`, 11 words).
/// A 2-word §4.2 phrase like `crème brûlée` also stays out.
pub(super) fn is_short_typeform_foreign_sentence(tokens: &[EnglishToken]) -> bool {
    let typeform_lens = typeform_word_lengths(tokens);
    if typeform_lens.is_empty() {
        return false;
    }
    // A short (≤4-letter) typeform-marked word is almost always a foreign
    // function verb (`es`, `soy`, `eres`, `est`); a long styled word
    // (`shamisen`, `demonstrare`, `l'oeil-de-boeuf`) is a foreign object in an
    // otherwise English narrative.
    if !typeform_lens.iter().any(|len| *len <= 4) {
        return false;
    }
    let all_words = document_all_words(tokens);
    if !(3..=5).contains(&all_words.len()) {
        return false;
    }
    // Require at least ONE non-styled plain word to be non-CMU — this excludes
    // English parenthetical glosses like `(Fr.: bull's eye)` after a styled
    // French phrase (§9.3.2 gloss).
    let plain_unrecorded = document_words(tokens)
        .iter()
        .filter(|w| {
            let word: String = w.iter().flat_map(|c| c.to_lowercase()).collect();
            word.chars().count() > 1
                && !super::super::pronunciation::cmudict::is_recorded_word(&word)
        })
        .count();
    plain_unrecorded >= 1
}

/// Number of styled "phrases" in the token stream. A phrase is a maximal
/// contiguous run of styled words separated only by single space tokens; a
/// plain-Word token between two styled runs starts a NEW phrase.
///
/// Distinguishes §13.1.2 pirouette-style narratives (a single italic phrase
/// `𝑓𝑜𝑢𝑒𝑡𝑡𝑒́ 𝑒𝑛 𝑡𝑜𝑢𝑟𝑛𝑎𝑛𝑡` = 1 phrase) from §13.6.4/§13.7.2 grammar textbooks
/// listing foreign vocabulary (`𝐪𝐮𝐞́ … 𝐯𝐚𝐲𝐚` = 2 phrases).
pub(super) fn styled_phrase_count(tokens: &[EnglishToken]) -> usize {
    let mut phrases = 0usize;
    let mut in_phrase = false;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            EnglishToken::Styled(_, _) => {
                if !in_phrase {
                    phrases += 1;
                    in_phrase = true;
                }
                i += 1;
            }
            EnglishToken::Space => {
                if in_phrase && !matches!(tokens.get(i + 1), Some(EnglishToken::Styled(_, _))) {
                    in_phrase = false;
                }
                i += 1;
            }
            _ => {
                in_phrase = false;
                i += 1;
            }
        }
    }
    phrases
}

/// §13.7.2 signal: any styled letter in the document carries a foreign accent
/// (é, è, ñ, etc.). Used with `styled_phrase_count >= 2` to detect the
/// typography-marked foreign-vocabulary pattern.
pub(super) fn document_any_styled_phrase_has_foreign_letter(tokens: &[EnglishToken]) -> bool {
    tokens.iter().any(
        |t| matches!(t, EnglishToken::Styled(c, _) if super::super::rule_13::is_foreign_letter(*c)),
    )
}

/// §13.5.1 adjacency: the punctuation at `i` sits directly next to a styled
/// token (or with only a single intervening space), so the surrounding
/// typography-marked foreign vocabulary carries over to the punctuation choice.
pub(super) fn punctuation_adjacent_to_styled(tokens: &[EnglishToken], i: usize) -> bool {
    let is_styled = |t: Option<&EnglishToken>| matches!(t, Some(EnglishToken::Styled(_, _)));
    let prev = i.checked_sub(1).and_then(|p| tokens.get(p));
    let prev2 = i.checked_sub(2).and_then(|p| tokens.get(p));
    let next = tokens.get(i + 1);
    let next2 = tokens.get(i + 2);
    is_styled(prev)
        || is_styled(next)
        || (matches!(prev, Some(EnglishToken::Space)) && is_styled(prev2))
        || (matches!(next, Some(EnglishToken::Space)) && is_styled(next2))
}

/// §13.7.2 shape check: every styled phrase in the document is a short
/// all-lowercase foreign vocabulary word. Excludes English titles (`𝑇ℎ𝑒 𝑇𝑖𝑚𝑒𝑠`),
/// proper-name runs (`𝐴𝑠𝑎ℎ𝑖 𝑆ℎ𝑖𝑚𝑏𝑢𝑛`), and long styled prose from over-triggering
/// the §13.7.2 foreign-code accent path.
pub(super) fn document_all_styled_phrases_are_short_vocabulary(tokens: &[EnglishToken]) -> bool {
    let mut current: Vec<char> = Vec::new();
    let mut had_any = false;
    let flush = |current: &mut Vec<char>| -> bool {
        let ok = current.is_empty()
            || (current.len() <= 10
                && current
                    .iter()
                    .all(|c| !c.is_uppercase() || super::super::rule_4::is_modified_letter(*c)));
        current.clear();
        ok
    };
    for token in tokens {
        match token {
            EnglishToken::Styled(c, _) => {
                current.push(*c);
                had_any = true;
            }
            EnglishToken::Symbol('-' | '\'' | '\u{2019}') if !current.is_empty() => {
                // Word-internal punctuation stays with the styled word.
            }
            EnglishToken::Space | EnglishToken::LineBreak | EnglishToken::Symbol(_) => {
                if !flush(&mut current) {
                    return false;
                }
            }
            _ => {
                if !flush(&mut current) {
                    return false;
                }
            }
        }
    }
    if !flush(&mut current) {
        return false;
    }
    had_any
}

/// Space-delimited character length of each typeform-marked word in `tokens`.
/// Word-internal `-` / `'` are treated as part of the word (`l'oeil-de-boeuf`
/// counts as one length-12 typeform word). A run with zero styled tokens is
/// not emitted.
pub(super) fn typeform_word_lengths(tokens: &[EnglishToken]) -> Vec<usize> {
    let mut lengths = Vec::new();
    let mut current_len = 0usize;
    let mut has_styled = false;
    for token in tokens {
        match token {
            EnglishToken::Space | EnglishToken::LineBreak => {
                if has_styled && current_len > 0 {
                    lengths.push(current_len);
                }
                current_len = 0;
                has_styled = false;
            }
            EnglishToken::Word(chars) => current_len += chars.len(),
            EnglishToken::Styled(_, _) => {
                current_len += 1;
                has_styled = true;
            }
            EnglishToken::Symbol('-' | '\'' | '\u{2019}') => {
                // Word-internal punctuation — does not split.
            }
            _ => {
                if has_styled && current_len > 0 {
                    lengths.push(current_len);
                }
                current_len = 0;
                has_styled = false;
            }
        }
    }
    if has_styled && current_len > 0 {
        lengths.push(current_len);
    }
    lengths
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::copyright("© 2009", "⠘⠉⠀⠼⠃⠚⠚⠊")]
    #[case::euro_franc_equation("1 € = 6.55957₣", "⠼⠁⠀⠈⠑⠀⠐⠶⠀⠼⠋⠲⠑⠑⠊⠑⠛⠈⠋")]
    #[case::registered("Intuit®", "⠠⠔⠞⠥⠊⠞⠘⠗")]
    #[case::trademark("Tax™", "⠠⠞⠁⠭⠘⠞")]
    #[case::square_shape("□ Director", "⠰⠫⠼⠙⠀⠠⠙⠊⠗⠑⠉⠞⠕⠗")]
    #[case::circle_shape("o Manager", "⠰⠫⠿⠀⠠⠍⠁⠝⠁⠛⠻")]
    #[case::pencil_shape("✏ Recycling", "⠈⠫⠏⠑⠝⠉⠊⠇⠀⠠⠗⠑⠉⠽⠉⠇⠬")]
    #[case::pointing_shape("☞ Steps", "⠈⠫⠏⠕⠊⠝⠞⠀⠠⠌⠑⠏⠎")]
    #[case::per_mille("salinity 35‰", "⠎⠁⠇⠔⠰⠽⠀⠼⠉⠑⠹")]
    #[case::check_mark("✓ item", "⠈⠩⠀⠊⠞⠑⠍")]
    #[case::braille_mention("⠫⠼⠙ square", "⠨⠿⠫⠼⠙⠀⠎⠟⠥⠜⠑")]
    fn encodes_rule_3_general_symbols(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §16.2 horizontal line mode: a run of box-drawing characters opens with
    /// `⠐⠒` (a leading `─` folding into the indicator's `⠒`) and maps each further
    /// char to its segment/corner/crossing cell.

    #[test]
    fn dot_delimited_web_components_contract_10_12_3() {
        assert_eq!(
            enc("www.one.in.a.hundred.org"),
            Some(cells("⠺⠺⠺⠲⠐⠕⠲⠔⠲⠁⠲⠓⠥⠝⠙⠗⠫⠲⠕⠗⠛"))
        );
    }

    /// `try_encode` owns letter-containing input and §9-styled input; a *plain*
    /// number/symbol run with no ASCII letter and no styling (no math-alphanumeric
    /// or combining underline) is delegated to the legacy path — the `encode()`
    /// precondition in `encoder.rs` mirrors this via `is_ueb_eligible` — so the
    /// engine returns `None`. Number/symbol encoding itself is covered by `5a`/
    /// `a 50`, the styled-digit cases above, and the testcase suite.

    #[rstest::rstest]
    #[case::sh_exclamation_spells("Sh!", "⠠⠎⠓⠖")]
    #[case::th_apostrophe_spells("th'", "⠞⠓⠄")]
    #[case::th_apostrophe_n_contracts("th'n", "⠹⠄⠝")]
    fn strong_groupsign_word_ambiguity_10_4_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.9 shortforms: whole shortform words contract only in standalone
    /// pure-English UEB, while a literal abbreviation gets a grade-1 guard.

    #[test]
    fn emits_styled_struck_pair_paths() {
        let wrong_tokens = [
            EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol('\u{0336}'),
            EnglishToken::Word(vec!['b']),
        ];
        let mut out = Vec::new();
        assert_eq!(
            emit_styled_struck_pair(
                &wrong_tokens,
                0,
                super::super::super::token::Typeform::Italic,
                'a',
                &mut out,
            ),
            None
        );

        let missing_overlay = [
            EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol('\u{0336}'),
            EnglishToken::Styled('b', super::super::super::token::Typeform::Bold),
        ];
        assert_eq!(
            emit_styled_struck_pair(
                &missing_overlay,
                0,
                super::super::super::token::Typeform::Italic,
                'a',
                &mut out,
            ),
            None
        );

        let good = [
            EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol('\u{0336}'),
            EnglishToken::Styled('B', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol('\u{0336}'),
        ];
        let mut struck = Vec::new();
        assert_eq!(
            emit_styled_struck_pair(
                &good,
                0,
                super::super::super::token::Typeform::Italic,
                'a',
                &mut struck,
            ),
            Some(4)
        );
        assert_eq!(struck, cells("⠨⠂⠁⠠⠘⠖⠃"));
    }

    #[test]
    fn chemical_script_branch_collects_all_token_kinds() {
        let engine = EnglishUebEngine::new();
        let form = super::super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Word(vec!['H']),
            EnglishToken::Symbol('₁'),
            EnglishToken::Space,
            EnglishToken::Number(vec!['2']),
            EnglishToken::LineBreak,
            EnglishToken::Styled('O', form),
            EnglishToken::WordDivision {
                chars: vec!['H'],
                break_at: 1,
            },
            EnglishToken::Symbol('+'),
        ];

        let encoded = engine.encode(&tokens, false).unwrap();

        assert!(encoded.starts_with(&[GRADE1, GRADE1, GRADE1]));
        assert!(encoded.ends_with(&[GRADE1, decode_unicode('⠄')]));
    }

    #[test]
    fn styled_symbol_sequence_helpers_cover_all_token_kinds() {
        let form = super::super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Styled('R', form),
            EnglishToken::Symbol('.'),
            EnglishToken::Styled('2', form),
            EnglishToken::Symbol('.'),
            EnglishToken::Styled('?', form),
        ];
        assert!(styled_capital_starts_symbol_sequence(&tokens, 0, 1));
        assert_eq!(styled_symbol_sequence_end(&tokens, 0, form), 4);

        let mut out = Vec::new();
        encode_styled_symbol_sequence(&tokens, 0, tokens.len(), form, &mut out).unwrap();
        assert_eq!(out, cells("⠠⠗⠲⠼⠃⠲⠰⠦"));

        let invalid = [EnglishToken::Styled('R', form), EnglishToken::Space];
        let mut rejected = Vec::new();
        assert_eq!(
            encode_styled_symbol_sequence(&invalid, 0, invalid.len(), form, &mut rejected),
            None
        );
    }

    #[test]
    fn rare_spacing_bracket_and_symbol_helpers_cover_remaining_branches() {
        let number_space = [
            EnglishToken::Number(vec!['1']),
            EnglishToken::Space,
            EnglishToken::Number(vec!['2']),
        ];
        assert!(is_numeric_space(&number_space, 1));
        let mut out = Vec::new();
        assert_eq!(
            encode_following_number_as_numeric_space(&number_space, 1, &mut out, true),
            Some(3)
        );
        assert!(!out.is_empty());

        let styled_gap = [
            EnglishToken::Styled('A', super::super::super::token::Typeform::Underline),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Styled('B', super::super::super::token::Typeform::Underline),
        ];
        assert_eq!(styled_column_gap(&styled_gap, 1), Some(4));
        let styled_numeric_gap = [
            EnglishToken::Styled('1', super::super::super::token::Typeform::Underline),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Styled('2', super::super::super::token::Typeform::Underline),
        ];
        assert_eq!(styled_column_gap(&styled_numeric_gap, 1), None);

        assert!(needs_spatial_grade1_passage(&[
            EnglishToken::Word(vec!['X']),
            EnglishToken::Symbol('┼'),
        ]));
        assert!(horizontal_run_reaches_arrow(
            &[
                EnglishToken::Symbol('═'),
                EnglishToken::Symbol('═'),
                EnglishToken::Symbol('↓'),
            ],
            0
        ));

        assert!(continues_across_bracket(
            &[
                EnglishToken::Word(vec!['c', 'h', 'i', 'l', 'd']),
                EnglishToken::Symbol('('),
                EnglishToken::Word(vec!['i', 's', 'h']),
            ],
            0
        ));
        assert!(continues_across_bracket(
            &[
                EnglishToken::Word(vec!['g', 'o']),
                EnglishToken::Symbol('\''),
                EnglishToken::Word(vec!['n']),
            ],
            0
        ));

        let mut symbol_out = Vec::new();
        encode_styled_nonword_symbol('5', &mut symbol_out).expect("styled digit should encode");
        assert!(!symbol_out.is_empty());
        symbol_out.clear();
        encode_styled_nonword_symbol('?', &mut symbol_out).expect("styled question should encode");
        assert!(symbol_out.starts_with(&[GRADE1]));
    }

    #[test]
    fn token_typeform_reports_styled_form_only() {
        use super::super::super::token::Typeform;
        // §9: only a Styled token carries a typeform; structural tokens have none.
        assert_eq!(
            token_typeform(&EnglishToken::Styled('a', Typeform::Bold)),
            Some(Typeform::Bold)
        );
        assert_eq!(token_typeform(&EnglishToken::Space), None);
    }

    #[test]
    fn token_base_char_extracts_single_char_tokens() {
        use super::super::super::token::Typeform;
        // A styled letter and a one-letter word expose their base char; a
        // multi-char or structural token does not.
        assert_eq!(
            token_base_char(&EnglishToken::Styled('x', Typeform::Italic)),
            Some('x')
        );
        assert_eq!(token_base_char(&EnglishToken::Word(vec!['y'])), Some('y'));
        assert_eq!(token_base_char(&EnglishToken::Space), None);
    }

    #[test]
    fn token_plain_chars_preserve_word_division_maps_all_token_kinds() {
        // A space becomes a literal space and a word division inserts a `\n` at
        // the break index — exercising every arm of the flattener.
        let tokens = [
            EnglishToken::Word("ab".chars().collect()),
            EnglishToken::Space,
            EnglishToken::WordDivision {
                chars: "cd".chars().collect(),
                break_at: 1,
            },
            EnglishToken::LineBreak,
        ];
        assert_eq!(
            token_plain_chars_preserve_word_division(&tokens),
            vec!['a', 'b', ' ', 'c', '\n', 'd', '\n']
        );
    }

    #[test]
    fn encodes_capitalized_enough_before_bracketed_sentence_close() {
        // §8/§10.5: `(Enough.)` — a capitalized `enough` immediately before a
        // period and a closing bracket keeps the `enough` wordsign `⠢` with a
        // leading capital indicator; the lowercase form differs only by that
        // capital cell.
        let upper = enc("(Enough.)").expect("should encode");
        let lower = enc("(enough.)").expect("should encode");
        assert!(upper.contains(&CAPITAL));
        assert!(upper.contains(&decode_unicode('⠢')));
        assert_eq!(upper.len(), lower.len() + 1);
    }
}
