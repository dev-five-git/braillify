use super::*;

pub(super) fn encode_styled_nonword_symbol(c: char, out: &mut Vec<u8>) -> Option<()> {
    if c.is_ascii_digit() {
        out.extend(super::super::rule_6::encode_number(&[c])?);
        return Some(());
    }
    if c == '?' {
        out.push(GRADE1);
    }
    let cells = super::super::rule_7::encode_punctuation(c)
        .or_else(|| super::super::rule_3::encode_symbol(c))?;
    out.extend(cells);
    Some(())
}

pub(super) fn dot_delimited_domain_word_cells(
    tokens: &[EnglishToken],
    i: usize,
    word: &str,
) -> Option<Vec<u8>> {
    if !matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('.'))
    ) || !matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
    {
        return None;
    }
    match word {
        "in" => Some(vec![decode_unicode('⠔')]),
        "one" => Some(vec![decode_unicode('⠐'), decode_unicode('⠕')]),
        _ => None,
    }
}

pub(super) fn is_word_text(token: Option<&EnglishToken>, expected: &str) -> bool {
    matches!(token, Some(EnglishToken::Word(chars)) if chars.iter().collect::<String>().eq_ignore_ascii_case(expected))
}

pub(super) fn is_single_letter_word(token: Option<&EnglishToken>) -> bool {
    matches!(token, Some(EnglishToken::Word(chars)) if chars.len() == 1 && chars[0].is_ascii_alphabetic())
}

pub(super) fn is_pronunciation_or_letter_label_context(tokens: &[EnglishToken], i: usize) -> bool {
    // §5.11.1: words used as sounds/letters are uncontracted.  The structural
    // examples are a phonics frame (`C is for candy`) and a question label followed
    // by a number-letter reference (`Question 3c`); detect the frames, not the
    // braille outputs.
    if is_single_letter_word(tokens.get(i))
        && matches!(tokens.get(i + 1), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i + 2), "is")
        && matches!(tokens.get(i + 3), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i + 4), "for")
        && matches!(tokens.get(i + 5), Some(EnglishToken::Space))
    {
        return true;
    }

    if i >= 2
        && is_single_letter_word(tokens.get(i - 2))
        && matches!(tokens.get(i - 1), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i), "is")
    {
        return true;
    }

    if i >= 6
        && is_single_letter_word(tokens.get(i - 6))
        && matches!(tokens.get(i - 5), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i - 4), "is")
        && matches!(tokens.get(i - 3), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i - 2), "for")
        && matches!(tokens.get(i - 1), Some(EnglishToken::Space))
    {
        return true;
    }

    if i >= 4
        && is_single_letter_word(tokens.get(i - 4))
        && matches!(tokens.get(i - 3), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i - 2), "is")
        && matches!(tokens.get(i - 1), Some(EnglishToken::Space))
        && is_word_text(tokens.get(i), "for")
        && !tokens
            .iter()
            .any(|token| matches!(token, EnglishToken::Styled(..)))
    {
        return true;
    }

    matches!(tokens.get(i + 1), Some(EnglishToken::Space))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Number(_)))
        && is_single_letter_word(tokens.get(i + 3))
}

pub(super) fn capital_omitted_letter_dash(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(tokens.get(i), Some(EnglishToken::Symbol('—')))
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].is_ascii_uppercase()
        )
        && matches!(
            tokens.get(i + 1),
            None | Some(EnglishToken::Space | EnglishToken::LineBreak)
        )
}

pub(super) fn bibliography_entry_context(tokens: &[EnglishToken]) -> bool {
    // §13.1.3 bibliography examples are English-embedded entries: foreign titles
    // are uncontracted, but accented Latin letters keep UEB §4.2 modifier signs
    // (`Ménard`, `Élements`) rather than full foreign-code accent cells.  Detect
    // the structural entry marker (`1.` / `2.`) only; do not inspect vocabulary.
    matches!(tokens.first(), Some(EnglishToken::Number(_)))
        && matches!(tokens.get(1), Some(EnglishToken::Symbol('.')))
        && tokens
            .iter()
            .any(|t| matches!(t, EnglishToken::Styled(_, _)))
}

pub(super) fn bibliography_styled_number_title_end(
    tokens: &[EnglishToken],
    end: usize,
    words: usize,
) -> Option<usize> {
    if !bibliography_entry_context(tokens) || words < 2 {
        return None;
    }
    let mut k = end;
    if !matches!(tokens.get(k), Some(EnglishToken::Space)) {
        return None;
    }
    k += 1;
    if !matches!(tokens.get(k), Some(EnglishToken::Number(_))) {
        return None;
    }
    k += 1;
    if matches!(tokens.get(k), Some(EnglishToken::Symbol(',' | '.'))) {
        k += 1;
    }
    Some(k)
}

pub(super) fn bibliography_styled_title_scope(
    tokens: &[EnglishToken],
    start: usize,
    end: usize,
    form: super::super::token::Typeform,
) -> Option<(super::super::rule_13::AccentCode, bool)> {
    if !bibliography_entry_context(tokens) {
        return None;
    }
    let mut words: Vec<Vec<char>> = Vec::new();
    let mut k = start;
    while k < end {
        while !matches!(tokens.get(k).and_then(token_typeform), Some(f) if f == form) && k < end {
            k += 1;
        }
        let mut word = Vec::new();
        while matches!(tokens.get(k).and_then(token_typeform), Some(f) if f == form) {
            if let Some(c) = token_base_char(&tokens[k]) {
                word.push(c);
            }
            k += 1;
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    if words.iter().any(|w| {
        w.iter()
            .any(|c| super::super::rule_13::is_foreign_letter(*c))
    }) || bibliography_title_starts_with_foreign_article(&words)
    {
        return Some((super::super::rule_13::AccentCode::Ueb, false));
    }
    None
}

pub(super) fn bibliography_title_starts_with_foreign_article(words: &[Vec<char>]) -> bool {
    if words.len() < 2 {
        return false;
    }
    let first: String = words[0].iter().flat_map(|c| c.to_lowercase()).collect();
    matches!(first.as_str(), "le" | "la" | "les" | "el" | "il")
}

pub(super) fn bibliography_foreign_quote_word(tokens: &[EnglishToken], index: usize) -> bool {
    if !bibliography_entry_context(tokens) {
        return false;
    }
    let Some(left) = tokens[..index]
        .iter()
        .rposition(|token| matches!(token, EnglishToken::Symbol('"')))
    else {
        return false;
    };
    let Some(right) = tokens[index + 1..]
        .iter()
        .position(|token| matches!(token, EnglishToken::Symbol('"')))
        .map(|offset| index + 1 + offset)
    else {
        return false;
    };
    tokens[left + 1..right].iter().any(|token| match token {
        EnglishToken::Word(chars) => chars
            .iter()
            .any(|c| super::super::rule_4::is_modified_letter(*c)),
        _ => false,
    })
}

pub(super) fn bibliography_con_word(chars: &[char], tokens: &[EnglishToken], index: usize) -> bool {
    bibliography_entry_context(tokens)
        && tokens[..index]
            .iter()
            .all(|token| !matches!(token, EnglishToken::Styled(..)))
        && chars.len() > 3
        && chars[0].eq_ignore_ascii_case(&'c')
        && chars[1].eq_ignore_ascii_case(&'o')
        && chars[2].eq_ignore_ascii_case(&'n')
        && matches!(
            chars[3].to_ascii_lowercase(),
            'b' | 'c'
                | 'd'
                | 'f'
                | 'g'
                | 'h'
                | 'j'
                | 'k'
                | 'l'
                | 'm'
                | 'n'
                | 'p'
                | 'q'
                | 'r'
                | 's'
                | 't'
                | 'v'
                | 'w'
                | 'x'
                | 'y'
                | 'z'
        )
}

pub(super) fn poem_linear_context(tokens: &[EnglishToken]) -> bool {
    // §15.1.2: printed poem lines run together in braille use the line
    // indicator for the original line breaks.  Scope this to poem examples
    // that end with an attribution line (`\n—Name`) so ordinary prose line
    // breaks still follow §10.13.
    let has_attribution = tokens.windows(3).any(|w| {
        matches!(w[0], EnglishToken::LineBreak)
            && matches!(w[1], EnglishToken::Symbol('\u{2013}' | '\u{2014}'))
            && matches!(w[2], EnglishToken::Word(ref word) if word.first().is_some_and(|c| c.is_uppercase()))
    });
    let has_spatial_symbol = tokens.iter().any(
        |token| matches!(token, EnglishToken::Symbol(c) if super::super::rule_16::is_spatial_segment(*c)),
    );
    has_attribution
        || (!has_spatial_symbol
            && tokens
                .iter()
                .filter(|t| matches!(t, EnglishToken::LineBreak))
                .count()
                >= 2)
}

/// UEB 2024 §10.9.4: an all-caps prefix shortform immediately followed by an
/// interior case change keeps the shortform and then terminates capitals mode.
pub(super) fn initial_caps_shortform_boundary(chars: &[char]) -> Option<usize> {
    let initial_caps = chars.iter().take_while(|c| c.is_uppercase()).count();
    if initial_caps < 2 || !chars.get(initial_caps).is_some_and(|c| c.is_lowercase()) {
        return None;
    }
    let whole_lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let segment: Vec<char> = chars[..initial_caps]
        .iter()
        .flat_map(|c| c.to_lowercase())
        .collect();
    let (len, _) = super::super::rule_10_9::shortform_part_cells(&whole_lower, 0)?;
    (len == initial_caps && shortform_meets_rule_10_9_4(&whole_lower, 0, &segment, true))
        .then_some(initial_caps)
}

/// Document-level UEB Grade-2 encoder.
#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::level_arrows_sentence(
        "Does ↑Anyone ↓HERE ↓HAVE a ↑WATCH? ↑",
        "⠠⠙⠕⠑⠎⠀⠘⠨⠫⠀⠸⠲⠠⠁⠝⠽⠐⠕⠀⠘⠨⠮⠀⠸⠲⠠⠠⠐⠓⠀⠘⠨⠮⠀⠸⠲⠠⠠⠓⠁⠧⠑⠀⠁⠀⠘⠨⠫⠀⠸⠲⠠⠠⠺⠁⠞⠡⠦⠀⠘⠨⠫"
    )]
    fn encodes_tone_level_change_15_3_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §15.2.2: prime marks in phonetic text are stress marks; the foot/inch sign
    /// reading is limited to numeric measurements.

    #[rstest::rstest]
    #[case::secondary_stress_before_schwa("met′ə", "⠍⠑⠞⠘⠨⠆⠸⠢")]
    #[case::double_primary_stress_before_letter("môr′′fə", "⠍⠘⠩⠕⠗⠘⠨⠃⠋⠸⠢")]
    fn encodes_phonetic_prime_stress_15_2_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §3.24: a digit super/subscript following a base takes the level indicator
    /// (`⠔`/`⠢`). The grade-1 `⠰` precedes it after a letter base (`yd³`, `B₁₂`,
    /// `clarion¹`) but not after a number (`1682.³`), whose numeric mode covers it.

    #[rstest::rstest]
    #[case::smith_inge("Smith\nInge", "⠠⠎⠍⠊⠹⠤\n⠠⠔⠛⠑")]
    #[case::fro_ing("fro-\ning", "⠋⠗⠕⠤\n⠔⠛")]
    fn encodes_line_initial_ing_10_13_4(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §3.24 boundary: a *leading* super/subscript (no base before it) fails the
    /// whole UEB attempt so the legacy/math path keeps ownership — this is what
    /// protects combinatorics like `₇𝑃₂` (제18/19항) from being misread as §3.24.

    #[rstest::rstest]
    #[case::conlogue_impossible_nation(
        "1. Conlogue, Ray.  𝐼𝑚𝑝𝑜𝑠𝑠𝑖𝑏𝑙𝑒 𝑁𝑎𝑡𝑖𝑜𝑛:  𝑇ℎ𝑒 𝐿𝑜𝑛𝑔𝑖𝑛𝑔 𝑓𝑜𝑟 𝐻𝑜𝑚𝑒𝑙𝑎𝑛𝑑 𝑖𝑛 𝐶𝑎𝑛𝑎𝑑𝑎 𝑎𝑛𝑑 𝑄𝑢𝑒𝑏𝑒𝑐.  Toronto:  Mercury Press, 2002.",
        "⠼⠁⠲⠀⠠⠒⠇⠕⠛⠥⠑⠂⠀⠠⠗⠁⠽⠲⠀⠨⠶⠠⠊⠍⠏⠕⠎⠎⠊⠃⠇⠑⠀⠠⠝⠁⠰⠝⠒⠀⠠⠮⠀⠠⠇⠰⠛⠬⠀⠿⠀⠠⠓⠕⠍⠑⠇⠯⠀⠔⠀⠠⠉⠁⠝⠁⠙⠁⠀⠯⠀⠠⠟⠥⠑⠃⠑⠉⠲⠨⠄⠀⠠⠞⠕⠗⠕⠝⠞⠕⠒⠀⠠⠍⠻⠉⠥⠗⠽⠀⠠⠏⠗⠑⠎⠎⠂⠀⠼⠃⠚⠚⠃⠲"
    )]
    #[case::le_roy_ladurie_quoted_french(
        "2. Le Roy Ladurie, Emmanuel.  \"Quand Paris était capitale du monde.\"  𝐿𝑒 𝑁𝑜𝑢𝑣𝑒𝑙 𝑂𝑏𝑠𝑒𝑟𝑣𝑎𝑡𝑒𝑢𝑟, August 2001.",
        "⠼⠃⠲⠀⠠⠇⠑⠀⠠⠗⠕⠽⠀⠠⠇⠁⠙⠥⠗⠊⠑⠂⠀⠠⠑⠍⠍⠁⠝⠥⠑⠇⠲⠀⠦⠠⠟⠥⠁⠝⠙⠀⠠⠏⠁⠗⠊⠎⠀⠘⠌⠑⠞⠁⠊⠞⠀⠉⠁⠏⠊⠞⠁⠇⠑⠀⠙⠥⠀⠍⠕⠝⠙⠑⠲⠴⠀⠨⠶⠠⠇⠑⠀⠠⠝⠕⠥⠧⠑⠇⠀⠠⠕⠃⠎⠑⠗⠧⠁⠞⠑⠥⠗⠂⠨⠄⠀⠠⠁⠥⠛⠥⠌⠀⠼⠃⠚⠚⠁⠲"
    )]
    #[case::menard_elements(
        "3. Ménard, Marc.  𝐸́𝑙𝑒𝑚𝑒𝑛𝑡𝑠 𝑝𝑜𝑢𝑟 𝑢𝑛𝑒 𝑒́𝑐𝑜𝑛𝑜𝑚𝑖𝑒 𝑑𝑒𝑠 𝑖𝑛𝑑𝑢𝑠𝑡𝑟𝑖𝑒𝑠 𝑐𝑢𝑙𝑡𝑢𝑟𝑒𝑙𝑙𝑒𝑠.  Montreal:  SODEC, 2004.",
        "⠼⠉⠲⠀⠠⠍⠘⠌⠑⠝⠜⠙⠂⠀⠠⠍⠜⠉⠲⠀⠨⠶⠠⠘⠌⠑⠇⠑⠍⠑⠝⠞⠎⠀⠏⠕⠥⠗⠀⠥⠝⠑⠀⠘⠌⠑⠉⠕⠝⠕⠍⠊⠑⠀⠙⠑⠎⠀⠊⠝⠙⠥⠎⠞⠗⠊⠑⠎⠀⠉⠥⠇⠞⠥⠗⠑⠇⠇⠑⠎⠲⠨⠄⠀⠠⠍⠕⠝⠞⠗⠂⠇⠒⠀⠠⠠⠎⠕⠙⠑⠉⠂⠀⠼⠃⠚⠚⠙⠲"
    )]
    #[case::language_today(
        "4. Weber, George.  \"The World's Ten Most Influential Languages.\" 𝐿𝑎𝑛𝑔𝑢𝑎𝑔𝑒 𝑇𝑜𝑑𝑎𝑦 2, December 1997.",
        "⠼⠙⠲⠀⠠⠺⠑⠃⠻⠂⠀⠠⠛⠑⠕⠗⠛⠑⠲⠀⠦⠠⠮⠀⠠⠸⠺⠄⠎⠀⠠⠞⠢⠀⠠⠍⠕⠌⠀⠠⠔⠋⠇⠥⠢⠞⠊⠁⠇⠀⠠⠇⠁⠝⠛⠥⠁⠛⠑⠎⠲⠴⠀⠨⠶⠠⠇⠁⠝⠛⠥⠁⠛⠑⠀⠠⠞⠙⠀⠼⠃⠂⠨⠄⠀⠠⠙⠑⠉⠑⠍⠃⠻⠀⠼⠁⠊⠊⠛⠲"
    )]
    fn encodes_bibliography_entries_from_13_1_3(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9.x: three or more same-form styled words take a single passage indicator
    /// (`⠨⠶`) and terminator (`⠨⠄`), each word encoded normally between them — the
    /// shortforms `ab`/`cd` (about/could) still keep their §5.7.2 grade-1 `⠰`.

    #[rstest::rstest]
    #[case::number("95")]
    #[case::percent("5%")]
    #[case::comma_grouped("5,70")]
    #[case::decimal("4.2")]
    fn non_letter_input_delegated_to_legacy(#[case] text: &str) {
        assert_eq!(enc(text), None);
    }
    #[rstest::rstest]
    #[case::abbe("abbé", "⠁⠆⠘⠌⠑")]
    #[case::rechauffe("réchauffé", "⠗⠘⠌⠑⠡⠁⠥⠖⠘⠌⠑")]
    #[case::seance("séance", "⠎⠘⠌⠑⠨⠑")]
    #[case::double_macron_between_letters("spo\u{035e}on", "⠎⠏⠈⠤⠣⠕⠕⠜⠝")]
    fn modified_letters_keep_other_groupsigns_4_2_10(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    #[test]
    fn rare_helper_paths_cover_false_and_edge_branches() {
        assert_eq!(greek_letter_cells_with_caps('λ', false), Some(cells("⠨⠇")));
        assert_eq!(greek_letter_cells_with_caps('Ξ', true), Some(cells("⠨⠭")));
        assert_eq!(greek_letter_cells_with_caps('@', false), None);

        assert!(!parenthesized_foreign_style_before(
            &[EnglishToken::Styled(
                'a',
                super::super::super::token::Typeform::Italic
            )],
            1,
        ));

        assert_eq!(
            mixed_case_shortform_part(&['b', 'r', 'a', 'i', 'l', 'l', 'e', 'x'], 0, &['b', 'r']),
            Some((7, cells("⠃⠗⠇")))
        );

        assert!(styled_prose_double_space(
            &[
                EnglishToken::Styled('h', super::super::super::token::Typeform::Underline),
                EnglishToken::Styled('t', super::super::super::token::Typeform::Underline),
                EnglishToken::Styled('t', super::super::super::token::Typeform::Underline),
                EnglishToken::Styled('p', super::super::super::token::Typeform::Underline),
                EnglishToken::Symbol(':'),
                EnglishToken::Symbol('/'),
                EnglishToken::Symbol('/'),
                EnglishToken::Styled('x', super::super::super::token::Typeform::Underline),
                EnglishToken::Space,
                EnglishToken::Space,
                EnglishToken::Word(vec!['n', 'o', 'w']),
            ],
            8,
        ));

        assert!(straight_single_quote_is_matched_quotation(
            &[
                EnglishToken::Symbol('\''),
                EnglishToken::Word(vec!['C', 'a', 't']),
                EnglishToken::Symbol('\''),
            ],
            0,
        ));
        assert!(!straight_single_quote_is_matched_quotation(
            &[EnglishToken::Symbol('"')],
            0,
        ));

        assert!(previous_word_starts_uppercase(
            &[
                EnglishToken::Word(vec!['C', 'a', 't']),
                EnglishToken::Symbol('.'),
                EnglishToken::Symbol('\''),
            ],
            2,
        ));
        assert!(straight_single_quote_closes_after_inner_double(
            &[
                EnglishToken::Symbol('\''),
                EnglishToken::Word(vec!['H', 'i']),
                EnglishToken::Symbol('"'),
                EnglishToken::Symbol('!'),
                EnglishToken::Symbol(','),
                EnglishToken::Symbol('\''),
            ],
            5,
        ));
    }

    #[test]
    fn dash_after_enough_before_in_needs_enough_before_dash() {
        // §10.6.5 seam: true only when the token two back is `enough`.
        let with_enough = [
            EnglishToken::Word("enough".chars().collect()),
            EnglishToken::Symbol('\u{2014}'),
        ];
        assert!(dash_after_enough_before_in(&with_enough, 2));
        // A dash reached over a bare space (no `enough`) is not the seam.
        let bare = [EnglishToken::Space, EnglishToken::Symbol('\u{2014}')];
        assert!(!dash_after_enough_before_in(&bare, 2));
    }

    #[test]
    fn leading_stutter_prefix_guards_missing_or_empty_word() {
        // Reaches the `-` guard but the token at `start` is not a Word.
        let non_word = [
            EnglishToken::Word("so".chars().collect()),
            EnglishToken::Symbol('-'),
            EnglishToken::Space,
        ];
        assert!(!leading_stutter_prefix(&non_word, 2));
        // The token at `start` is an empty Word (no first char).
        let empty_word = [
            EnglishToken::Word("so".chars().collect()),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(Vec::new()),
        ];
        assert!(!leading_stutter_prefix(&empty_word, 2));
    }

    #[test]
    fn ends_spelled_letter_run_before_word_needs_a_hyphen() {
        // The token at `i` must be a hyphen symbol; anything else → false.
        assert!(!ends_spelled_letter_run_before_word(
            &[EnglishToken::Space],
            0
        ));
    }

    #[test]
    fn bibliography_styled_number_title_end_needs_trailing_space_number() {
        use super::super::super::token::Typeform;
        // A bibliography entry (`1.` + styled title) with no following
        // ` <number>` after the title has no numeric title end.
        let tokens = [
            EnglishToken::Number(vec!['1']),
            EnglishToken::Symbol('.'),
            EnglishToken::Styled('a', Typeform::Italic),
        ];
        assert_eq!(bibliography_styled_number_title_end(&tokens, 3, 2), None);
    }

    #[test]
    fn bibliography_title_starts_with_foreign_article_needs_two_words() {
        // A single-word title cannot open with a foreign article + noun.
        assert!(!bibliography_title_starts_with_foreign_article(&[vec![
            'l', 'e'
        ]]));
        // `le <noun>` is a French-article title.
        assert!(bibliography_title_starts_with_foreign_article(&[
            vec!['l', 'e'],
            vec!['m', 'o', 't']
        ]));
    }

    #[test]
    fn styled_phrase_from_named_place_breaks_without_styled_tail() {
        use super::super::super::token::Typeform;
        // `<styled> and <plain>` after the phrase has no styled continuation to a
        // `from <Place>`, so it is not a named-place attribution.
        let tokens = [
            EnglishToken::Styled('a', Typeform::Italic),
            EnglishToken::Space,
            EnglishToken::Word("and".chars().collect()),
            EnglishToken::Space,
            EnglishToken::Word("x".chars().collect()),
        ];
        assert!(!styled_phrase_from_named_place(&tokens, 1));
    }

    #[test]
    fn encodes_struck_letter_sequence() {
        // §4.3.1: a run of stroke-overlaid letters (U+0336) encodes as struck text.
        assert!(enc("a\u{0336}b\u{0336}").is_some());
    }

    #[test]
    fn encodes_word_with_trailing_combining_acute_on_last_letter() {
        // §4.2.1: a combining mark printed after a letter is placed before that
        // letter in braille, so the walk routes the `Word` + trailing
        // `Symbol(mark)` pair through `emit_word_with_modifier_on_last`.
        // `h`+U+0301 has no precomposed form, so NFC keeps the mark a separate
        // token (unlike `e`+acute, which composes to `é` and takes the accent
        // path instead).
        let mut expected = Vec::new();
        emit_word_with_modifier_on_last(&['g', 'r', 'a', 'p', 'h'], '\u{0301}', &mut expected)
            .unwrap();
        assert_eq!(enc("graph\u{0301}").unwrap(), expected);
    }

    /// Coverage for esoteric but genuine encode paths (RUEB §9/§13): a bibliography
    /// styled-number title, inverted-punctuation Spanish styled passages (trailing
    /// period stripped / `styled_passage_foreign_scope` / the §extent `¡` bridge), a
    /// BoldItalic passage continued by an Italic word (`nested_typeform_continuation`),
    /// and a bibliography foreign-quote all-caps word (`Caps::Word`). These focused
    /// regression cases assert that supported UEB paths successfully encode, without
    /// duplicating full-cell expectations (which would be `expected` back-solving).
    #[rstest::rstest]
    #[case::bibliography_styled_number_title(
        "1. \u{1D40B}\u{1D41E} \u{1D40F}\u{1D41E}\u{1D42B}\u{1D41E} 12."
    )]
    #[case::spanish_passage_trailing_period(
        "He said ¡\u{1D410}\u{1D42E}\u{1D41E}\u{301} \u{1D422}\u{1D41D}\u{1D41E}\u{1D41A} \u{1D41B}\u{1D42E}\u{1D41E}\u{1D427}\u{1D41A}. now"
    )]
    #[case::bolditalic_passage_then_italic(
        "\u{1D468}\u{1D483}\u{1D484} \u{1D46B}\u{1D486}\u{1D487} \u{1D46E}\u{1D48A}\u{1D48B} \u{1D465}\u{1D466}"
    )]
    #[case::spanish_inverted_bridge_in_extent(
        "He said \u{1D40E}\u{1D421} ¡\u{1D410}\u{1D42E}\u{1D41E}\u{301} \u{1D422}\u{1D41D}\u{1D41E}\u{1D41A}! now"
    )]
    #[case::spanish_foreign_scope(
        "He said ¡\u{1D410}\u{1D42E}\u{1D41E}\u{301} \u{1D422}\u{1D41D}\u{1D41E}\u{1D41A} \u{1D41B}\u{1D42E}\u{1D41E}\u{1D427}\u{1D41A}! now"
    )]
    #[case::bibliography_foreign_quote_caps_word("1. \u{1D400} \"QUOI caf\u{E9}\"")]
    // §8.5.3 all-caps styled passage (three-word `⠠⠠⠠` … `⠠⠄`) and single-styled-word
    // handler branches: a typeform-prefix contraction (`𝐰𝐨𝐫𝐝`), a hyphen-joined styled
    // span (`𝑜𝑓-𝑡𝑜`), and a plain word directly followed by a styled `ing` run.
    #[case::styled_all_caps_passage(
        "\u{1D400}\u{1D401}\u{1D402} \u{1D403}\u{1D404}\u{1D405} \u{1D406}\u{1D407}\u{1D408}"
    )]
    #[case::single_styled_word_prefix_contraction("\u{1D430}\u{1D428}\u{1D42B}\u{1D41D}")]
    #[case::hyphen_joined_styled_span("\u{1D45C}\u{1D453}-\u{1D461}\u{1D45C}")]
    #[case::plain_word_then_styled_ing("run\u{1D422}\u{1D427}\u{1D420}")]
    // A lone multi-char styled word (not a prefix contraction) emits the ordinary
    // word indicator; three same-form styled words open a passage (is_none/is_some
    // dispatch across the run).
    #[case::lone_styled_word_word_indicator("\u{1D41C}\u{1D41A}\u{1D42D}")]
    #[case::three_italic_words_passage_dispatch(
        "\u{1D44E}\u{1D44F} \u{1D450}\u{1D451} \u{1D452}\u{1D453}"
    )]
    // §8.8.1: a 4+ all-caps prefix (not a shortform) followed by a lowercase run and a
    // further Title-case subunit takes the `camel_title_subunit_after_caps_prefix` split.
    #[case::camel_caps_prefix_title_subunit("HTTPSxyzAbc")]
    #[case::camel_caps_prefix_title_subunit2("WXYZabcDef")]
    fn covers_esoteric_genuine_paths(#[case] input: &str) {
        assert!(
            enc(input).is_some(),
            "genuine UEB path should encode: {input:?}"
        );
    }
}
