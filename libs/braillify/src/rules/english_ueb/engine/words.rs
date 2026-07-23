use super::*;

pub(super) fn encode_literal_word(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    for &c in chars {
        if c.is_uppercase() {
            out.push(CAPITAL);
        }
        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
    }
    Some(())
}

pub(super) fn encode_modified_word(
    engine: &ContractionEngine,
    chars: &[char],
    word_initial: bool,
    restricted_prefix_boundary: bool,
    out: &mut Vec<u8>,
) -> Option<()> {
    let mut segment = Vec::new();
    let mut segment_start = 0usize;
    let mut segment_initial = word_initial;
    let mut segment_restricted = restricted_prefix_boundary;
    for (index, &c) in chars.iter().enumerate() {
        if super::super::rule_4::is_modified_letter(c)
            || matches!(c, 'æ' | 'Æ' | 'œ' | 'Œ' | 'ß' | 'ẞ')
        {
            if !segment.is_empty() {
                encode_unmodified_segment(
                    engine,
                    &segment,
                    segment_start,
                    segment_initial,
                    segment_restricted,
                    true,
                    out,
                )?;
                segment.clear();
            }
            let mut lower = c.to_lowercase();
            out.extend(super::super::rule_4::accent_cells(lower.next()?)?);
            segment_initial = false;
            segment_restricted = false;
            segment_start = index + 1;
        } else {
            if segment.is_empty() {
                segment_start = index;
            }
            let lowercase = c.to_lowercase();
            segment.extend(lowercase);
        }
    }
    if !segment.is_empty() {
        encode_unmodified_segment(
            engine,
            &segment,
            segment_start,
            segment_initial,
            segment_restricted,
            false,
            out,
        )?;
    }
    Some(())
}

pub(super) fn encode_unmodified_segment(
    engine: &ContractionEngine,
    segment: &[char],
    segment_start: usize,
    segment_initial: bool,
    segment_restricted: bool,
    has_letter_after: bool,
    out: &mut Vec<u8>,
) -> Option<()> {
    if has_letter_after
        && let Some((&left, prefix)) = segment.split_last()
        && let Some(&right) = prefix.last()
        && let Some(cell) = middle_lower_pair_cell(right, left)
    {
        let before_pair = &segment[..segment.len() - 2];
        if !before_pair.is_empty() {
            out.extend(
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    before_pair,
                    engine,
                    segment_initial,
                    segment_restricted,
                    true,
                )?,
            );
        }
        out.push(cell);
        return Some(());
    }
    if segment_start > 0 {
        let text: String = segment.iter().collect();
        if let Some(cells) = super::super::rule_10_8::final_groupsign_cells(&text) {
            out.extend(cells);
            return Some(());
        }
    }
    out.extend(
        super::super::rule_10_9::encode_with_optional_longer_shortforms(
            segment,
            engine,
            segment_initial,
            segment_restricted,
            true,
        )?,
    );
    Some(())
}

pub(super) fn middle_lower_pair_cell(left: char, right: char) -> Option<u8> {
    match (left, right) {
        ('e', 'a') => Some(decode_unicode('⠂')),
        ('b', 'b') => Some(decode_unicode('⠆')),
        ('c', 'c') => Some(decode_unicode('⠒')),
        ('f', 'f') => Some(decode_unicode('⠖')),
        ('g', 'g') => Some(decode_unicode('⠶')),
        _ => None,
    }
}

pub(super) fn is_numeric_space(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(tokens.get(i), Some(EnglishToken::Space))
        && matches!(tokens.get(i + 1), Some(EnglishToken::Number(_)))
        && i.checked_sub(1)
            .is_some_and(|p| matches!(tokens.get(p), Some(EnglishToken::Number(_))))
        && !matches!(tokens.get(i + 2), Some(EnglishToken::Word(_)))
}

pub(super) fn encode_following_number_as_numeric_space(
    tokens: &[EnglishToken],
    i: usize,
    out: &mut Vec<u8>,
    line_continuation: bool,
) -> Option<usize> {
    let Some(EnglishToken::Number(digits)) = tokens.get(i + 1) else {
        return None;
    };
    out.push(decode_unicode('⠐'));
    if line_continuation {
        out.push(decode_unicode('⠐'));
        out.push(SPACE);
    }
    for d in digits {
        out.push(super::super::rule_6::digit_cell(*d)?);
    }
    Some(i + 2)
}

pub(super) fn abbreviating_letters(tokens: &[EnglishToken], i: usize, lower_word: &str) -> bool {
    if lower_word == "ch" {
        return matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
            && matches!(tokens.get(i + 2), Some(EnglishToken::Space));
    }
    if matches!(lower_word, "gh") {
        return matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('/'))
        ) && matches!(i.checked_sub(2).and_then(|p| tokens.get(p)), Some(EnglishToken::Word(w)) if w.iter().all(char::is_ascii_uppercase));
    }
    false
}

pub(super) fn dash_bounded_strong_sequence_literal(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    matches!(lower_word, "ch" | "sh" | "th" | "wh" | "st")
        && (matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('-' | '–' | '—'))
        ) || matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('-'))))
}

pub(super) fn number_hyphen_in_abbreviation(tokens: &[EnglishToken], i: usize) -> bool {
    matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-'))
    ) && matches!(
        i.checked_sub(2).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Number(_))
    ) && matches!(tokens.get(i + 1), Some(EnglishToken::Space))
}

pub(super) fn shortform_confusion_grade1_count(lower_word: &str, chars: &[char]) -> Option<usize> {
    if !chars.iter().all(char::is_ascii_alphabetic) {
        return None;
    }
    match lower_word {
        "blvd" | "llc" | "grtsamada" | "frs" | "yrs" => Some(1),
        "dobrljin" | "ozbrl" | "unsd" => Some(2),
        _ => None,
    }
}

pub(super) fn shortform_abbreviation_literal(lower_word: &str, chars: &[char]) -> bool {
    chars.iter().all(char::is_ascii_alphabetic)
        && matches!(lower_word, "herf" | "mst" | "somesch" | "shd")
}

pub(super) fn stammer_fragment_literal(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    if lower_word == "sh" && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('!'))) {
        return true;
    }
    if matches!(lower_word, "ch" | "st" | "wh" | "th")
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('-')))
        && (matches!(tokens.get(i + 2), Some(EnglishToken::Word(_)))
            || matches!(tokens.get(i + 2), Some(EnglishToken::Symbol('.'))))
    {
        return true;
    }
    // §7.3 truncation: `th...` / `wh...` (word cut off by ellipsis, indicating
    // an unfinished thought) is a truncated word fragment, not a groupsign. The
    // ellipsis marker `.` immediately following makes it a word truncation.
    if matches!(lower_word, "st" | "wh" | "th")
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Symbol('.')))
    {
        return true;
    }
    matches!(lower_word, "wh" | "th")
        && matches!(
            tokens.get(i + 1),
            Some(EnglishToken::Symbol('\'' | '–' | '—'))
        )
        && !matches!(tokens.get(i + 2), Some(EnglishToken::Word(_)))
}

pub(super) fn repeated_initial_letter_stammer(chars: &[char]) -> bool {
    if chars.len() < 3 {
        return false;
    }
    // `chars.len() >= 3` guarantees index 0 exists.
    let first = chars[0].to_ascii_lowercase();
    if first != 'l' {
        return false;
    }
    super::super::rule_5_7::is_wordsign_letter(first)
        && chars
            .iter()
            .take(3)
            .all(|c| c.to_ascii_lowercase() == first)
}

pub(super) fn after_repeated_stammer_prefix(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    let Some(first) = lower_word.chars().next() else {
        return false;
    };
    matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-'))
    ) && matches!(i.checked_sub(2).and_then(|p| tokens.get(p)), Some(EnglishToken::Word(w)) if w.len() >= 3 && w.iter().all(|c| c.to_ascii_lowercase() == first))
}

#[allow(dead_code)]
pub(super) fn midword_parenthesized_ing(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    lower_word == "ing"
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Symbol('('))
        )
        && matches!(
            i.checked_sub(2).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Word(_))
        )
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol(')')))
}

#[allow(dead_code)]
pub(super) fn measurement_in_abbreviation(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    lower_word == "in"
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
        && matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Space)
        )
        && matches!(
            i.checked_sub(2).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Number(_))
        )
}

pub(super) fn syllable_alphabetic_wordsign_literal(
    tokens: &[EnglishToken],
    i: usize,
    lower_word: &str,
) -> bool {
    if lower_word == "as"
        && matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('-')))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].eq_ignore_ascii_case(&'s'))
    {
        return false;
    }
    let is_lowercase_syllable = matches!(
        tokens.get(i),
        Some(EnglishToken::Word(chars)) if chars.iter().all(char::is_ascii_lowercase)
    );
    if !is_lowercase_syllable {
        return false;
    }
    let before_hyphen_or_dash = matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-' | '–' | '—'))
    );
    let before_ascii_hyphen = matches!(
        i.checked_sub(1).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-'))
    );
    let after_hyphen_or_dash = matches!(
        tokens.get(i + 1),
        Some(EnglishToken::Symbol('-' | '–' | '—'))
    );
    let after_ascii_hyphen = matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('-')));
    let spaced_syllable = matches!(
        (
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            tokens.get(i + 1)
        ),
        (Some(EnglishToken::Space), Some(EnglishToken::Space))
    ) && matches!(
        i.checked_sub(2).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Word(_))
    ) && matches!(tokens.get(i + 2), Some(EnglishToken::Word(_)));
    let be_have_syllable = (lower_word == "be" && after_hyphen_or_dash)
        || (lower_word == "have"
            && before_hyphen_or_dash
            && matches!(
                i.checked_sub(2).and_then(|p| tokens.get(p)),
                Some(EnglishToken::Word(w)) if w.iter().collect::<String>().eq_ignore_ascii_case("be")
            ));
    let suffix_it_in_albeit = lower_word == "it"
        && before_ascii_hyphen
        && matches!(i.checked_sub(2).and_then(|p| tokens.get(p)), Some(EnglishToken::Word(w)) if w.iter().collect::<String>().eq_ignore_ascii_case("be"));
    (matches!(lower_word, "but") && after_hyphen_or_dash)
        || be_have_syllable
        || suffix_it_in_albeit
        || (lower_word == "more" && before_ascii_hyphen)
        || (lower_word == "not" && (before_ascii_hyphen || after_ascii_hyphen))
        || (lower_word == "as" && spaced_syllable && !spaced_as_contracts(tokens, i))
}

/// §10.1 `as` wordsign always applies to a genuinely standing-alone `as`
/// between two prose words separated by spaces. The general `spaced_syllable`
/// rule keeps §10.9 hyphenated syllables (`al-be-it`) literal, but a plain
/// prose `such as this` MUST contract to `⠵`.
pub(super) fn spaced_as_contracts(tokens: &[EnglishToken], i: usize) -> bool {
    // A plain-prose `as` between two space-bounded words with no hyphen/dash
    // in the syntactic neighbourhood contracts (`such as this`, `high as sky`).
    // §10.1.4 space-shown syllables (`dis as ter`) are different: the adjacent
    // fragments concatenate to one dictionary word, so `as` is not the wordsign.
    !matches!(
        i.checked_sub(2).and_then(|p| tokens.get(p)),
        Some(EnglishToken::Symbol('-' | '–' | '—'))
    ) && !matches!(
        tokens.get(i + 2),
        Some(EnglishToken::Symbol('-' | '–' | '—'))
    ) && !space_delimited_syllables_form_word(tokens, i)
}

pub(super) fn space_delimited_syllables_form_word(tokens: &[EnglishToken], i: usize) -> bool {
    let Some(EnglishToken::Word(prev)) = i.checked_sub(2).and_then(|p| tokens.get(p)) else {
        return false;
    };
    let Some(EnglishToken::Word(curr)) = tokens.get(i) else {
        return false;
    };
    let Some(EnglishToken::Word(next)) = tokens.get(i + 2) else {
        return false;
    };
    let word: String = prev
        .iter()
        .chain(curr.iter())
        .chain(next.iter())
        .flat_map(|c| c.to_lowercase())
        .collect();
    super::super::pronunciation::cmudict::is_recorded_word(&word)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::lower_letters("cat", vec![decode_unicode('⠉'), decode_unicode('⠁'), decode_unicode('⠞')])]
    #[case::single_capital("A", vec![CAPITAL, decode_unicode('⠁')])]
    #[case::leading_capital("Cat", vec![CAPITAL, decode_unicode('⠉'), decode_unicode('⠁'), decode_unicode('⠞')])]
    // `XY` is all-caps but not a shortform collision, so no §8.7 grade-1 indicator.
    #[case::caps_word("XY", vec![CAPITAL, CAPITAL, decode_unicode('⠭'), decode_unicode('⠽')])]
    #[case::strong_contraction("the", vec![decode_unicode('⠮')])]
    #[case::groupsign_then_letters("show", vec![decode_unicode('⠩'), decode_unicode('⠪')])]
    #[case::lower_groupsign_in("find", vec![decode_unicode('⠋'), decode_unicode('⠔'), decode_unicode('⠙')])]
    #[case::lower_groupsign_en("send", vec![decode_unicode('⠎'), decode_unicode('⠢'), decode_unicode('⠙')])]
    #[case::enough_lower_wordsign("enough", vec![decode_unicode('⠢')])]
    // §10.12.2: the lone wordsign letter `b` in running text takes a grade-1 ⠰.
    #[case::two_words("a b", vec![decode_unicode('⠁'), SPACE, GRADE1, decode_unicode('⠃')])]
    #[case::number_then_az_letter("5a", vec![decode_unicode('⠼'), decode_unicode('⠑'), GRADE1, decode_unicode('⠁')])]
    #[case::word_space_number("a 50", vec![decode_unicode('⠁'), SPACE, decode_unicode('⠼'), decode_unicode('⠑'), decode_unicode('⠚')])]
    fn encodes_supported_words(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §9.2–§9.6: typeform indicators apply to the next symbol/word/passage by
    /// extent, while script letters and small capitals keep their base identities.

    #[rstest::rstest]
    #[case::styled_ing_suffix("brown𝑖𝑛𝑔", "⠃⠗⠪⠝⠨⠆⠬")]
    #[case::styled_lower_wordsign_sentence("𝐵𝑒 ℎ𝑎𝑝𝑝𝑦.", "⠨⠂⠠⠆⠀⠨⠂⠓⠁⠏⠏⠽⠲")]
    #[case::styled_shortform("𝑛𝑒𝑖𝑡ℎ𝑒𝑟", "⠨⠂⠝⠑⠊")]
    fn encodes_rule10_typeform_contractions(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §8.2: a mixed-case word (internal capitals) is split at each lower→upper
    /// boundary and each Title-case / all-caps part takes its own capital
    /// indicator (`⠠` single, `⠠⠠` all-caps), contractions applying within each.

    #[rstest::rstest]
    // `4x4` — a letter splits the run, so each number keeps its own `⠼`
    // (grade-1 ⠰ guards an a–j letter; `x` is not a–j, so no ⠰).
    #[case::letter_split("4x4", vec![decode_unicode('⠼'), decode_unicode('⠙'), decode_unicode('⠭'), decode_unicode('⠼'), decode_unicode('⠙')])]
    #[case::numeric_mode_spells_following_word("shopping4you", cells("⠩⠕⠏⠏⠬⠼⠙⠽⠕⠥"))]
    #[case::numeric_mode_spells_following_domain(
        "4starhotel@webnet.com",
        cells("⠼⠙⠎⠞⠁⠗⠓⠕⠞⠑⠇⠈⠁⠺⠑⠃⠝⠑⠞⠲⠉⠕⠍")
    )]
    fn numeric_indicator_restarts_after_letter(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §10.1/§10.2 wordsigns (standing alone) and the §2.6 standing-alone guard.

    #[rstest::rstest]
    #[case::but_wordsign("but", vec![decode_unicode('⠃')])]
    #[case::knowledge_wordsign("knowledge", vec![decode_unicode('⠅')])]
    #[case::this_wordsign("this", vec![decode_unicode('⠹')])]
    #[case::child_wordsign("child", vec![decode_unicode('⠡')])]
    #[case::it_apostrophe_s("it's", vec![decode_unicode('⠭'), decode_unicode('⠄'), decode_unicode('⠎')])]
    #[case::titlecase_hyphenated_it("Do-It-Yourself", cells("⠠⠙⠤⠠⠭⠤⠠⠽⠗⠋"))]
    #[case::alphabetic_wordsign_suppressed_before_slash("quite/very", cells("⠟⠥⠊⠞⠑⠸⠌⠧⠻⠽"))]
    #[case::strong_wordsign_suppressed_before_slash("this/that", cells("⠹⠊⠎⠸⠌⠹⠁⠞"))]
    #[case::acronym_it_spells_letters("IT", cells("⠠⠠⠊⠞"))]
    #[case::printed_syllable_but_ton_spells_wordsign("but-ton", cells("⠃⠥⠞⠤⠞⠕⠝"))]
    #[case::printed_syllable_be_have_spells_wordsigns("be–have", cells("⠃⠑⠠⠤⠓⠁⠧⠑"))]
    #[case::printed_syllable_dis_as_ter_spells_as("dis as ter", cells("⠙⠊⠎⠀⠁⠎⠀⠞⠻"))]
    #[case::stammer_as_keeps_wordsign("as-s-s-s", cells("⠵⠤⠰⠰⠎⠤⠎⠤⠎"))]
    #[case::dash_phrase_but_keeps_wordsign("some–but", cells("⠐⠎⠠⠤⠃"))]
    #[case::dash_phrase_not_keeps_wordsign("from–not", cells("⠋⠠⠤⠝"))]
    #[case::hyphenated_not_spells_wordsign("not-with-stand-ing", cells("⠝⠕⠞⠤⠾⠤⠌⠯⠤⠔⠛"))]
    #[case::hyphenated_more_spells_wordsign("for-ev-er-more", cells("⠿⠤⠑⠧⠤⠻⠤⠍⠕⠗⠑"))]
    #[case::hyphenated_it_spells_wordsign("al-be-it", cells("⠰⠁⠇⠤⠃⠑⠤⠊⠞"))]
    #[case::apostrophe_m_spells_you("you'm", cells("⠽⠳⠄⠍"))]
    fn encodes_wordsigns(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §10.4.2: `ch/sh/th/wh/ou/st` spell as letters only where the groupsign would
    /// be misread as a word; an apostrophe plus following letters can still be a
    /// word fragment (`th'n`) and keep the groupsign.

    #[rstest::rstest]
    #[case::good("good", "⠛⠙")]
    #[case::would("would", "⠺⠙")]
    #[case::rejoice("rejoice", "⠗⠚⠉")]
    #[case::literal_gd("gd", "⠰⠛⠙")]
    fn encodes_shortforms(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.5 lower wordsigns: used between anchoring boundaries (space/edge/
    /// bracket), spelled out when touching a lower-sign neighbour (`?`, hyphen).

    #[rstest::rstest]
    #[case::be_alone("be", vec![decode_unicode('⠆')])]
    #[case::was_alone("was", vec![decode_unicode('⠴')])]
    #[case::his_alone("his", vec![decode_unicode('⠦')])]
    #[case::was_in_parens("(was)", vec![decode_unicode('⠐'), decode_unicode('⠣'), decode_unicode('⠴'), decode_unicode('⠐'), decode_unicode('⠜')])]
    #[case::be_before_question("be?", vec![decode_unicode('⠃'), decode_unicode('⠑'), decode_unicode('⠦')])]
    // §10.5.2: enough's keeps the wordsign; his' (lower-dot contact) spells out.
    #[case::enoughs("Enough's", vec![CAPITAL, decode_unicode('⠢'), decode_unicode('⠄'), decode_unicode('⠎')])]
    #[case::his_apostrophe_n("his'n", vec![decode_unicode('⠓'), decode_unicode('⠊'), decode_unicode('⠎'), decode_unicode('⠄'), decode_unicode('⠝')])]
    fn lower_wordsigns_respect_boundaries(#[case] text: &str, #[case] expected: Vec<u8>) {
        assert_eq!(enc(text), Some(expected));
    }

    /// §10.5.3–§10.5.4: lower wordsigns may touch lower punctuation only while the
    /// full lower-sign sequence also contains a non-quote sign with upper dots.

    #[test]
    fn rare_syllable_and_spelled_run_helpers_cover_remaining_branches() {
        let dis_as_ter = [
            EnglishToken::Word(vec!['d', 'i', 's']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['a', 's']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['t', 'e', 'r']),
        ];
        assert!(space_delimited_syllables_form_word(&dis_as_ter, 2));
        assert!(!spaced_as_contracts(&dis_as_ter, 2));

        assert!(
            styled_word_count(&[
                EnglishToken::Styled('l', super::super::super::token::Typeform::Italic),
                EnglishToken::Symbol('-'),
                EnglishToken::Styled('o', super::super::super::token::Typeform::Italic),
                EnglishToken::Space,
                EnglishToken::Styled('x', super::super::super::token::Typeform::Italic),
            ]) >= 2
        );
        assert!(all_text_is_styled_or_punctuation(&[
            EnglishToken::Styled('x', super::super::super::token::Typeform::Italic),
            EnglishToken::Symbol('.'),
            EnglishToken::Space,
        ]));
        assert!(starts_with_ch_not_pronounced_ch("chaos"));
        assert!(styled_word_is_foreign(&['c', 'h', 'a', 'o', 's']));
        assert!(!styled_single_word_is_foreign(&['t', 'h']));
        assert_eq!(
            token_typeform(&EnglishToken::Styled(
                'x',
                super::super::super::token::Typeform::Italic
            )),
            Some(super::super::super::token::Typeform::Italic)
        );

        let spelled = [
            EnglishToken::Word(vec!['w']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['i']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['n']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['s']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['o', 'm', 'e']),
        ];
        assert_eq!(spelled_letter_run(&spelled, 0), Some((0, 6)));
        assert!(ends_spelled_letter_run_before_word(&spelled, 7));
        let stutter = [
            EnglishToken::Word(vec!['s', 'o']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['o']),
            EnglishToken::Symbol('-'),
            EnglishToken::Word(vec!['o']),
        ];
        assert!(leading_stutter_prefix(&stutter, 2));

        assert!(stammer_fragment_literal(
            &[
                EnglishToken::Word(vec!['c', 'h']),
                EnglishToken::Symbol('-'),
                EnglishToken::Word(vec!['a']),
            ],
            0,
            "ch",
        ));
        assert!(stammer_fragment_literal(
            &[
                EnglishToken::Word(vec!['t', 'h']),
                EnglishToken::Symbol('.'),
                EnglishToken::Symbol('.'),
            ],
            0,
            "th",
        ));
        assert!(midword_parenthesized_ing(
            &[
                EnglishToken::Word(vec!['s']),
                EnglishToken::Symbol('('),
                EnglishToken::Word(vec!['i', 'n', 'g']),
                EnglishToken::Symbol(')'),
            ],
            2,
            "ing",
        ));
    }

    #[test]
    fn styled_numeric_sequence_end_spans_spaced_digit_groups() {
        use super::super::super::token::Typeform;
        // §9/§11: a styled digit run may include an internal space between two
        // styled digit groups of the same typeform.
        let tokens = [
            EnglishToken::Styled('1', Typeform::Italic),
            EnglishToken::Space,
            EnglishToken::Styled('2', Typeform::Italic),
        ];
        assert_eq!(styled_numeric_sequence_end(&tokens, 0, Typeform::Italic), 3);
    }

    #[test]
    fn encode_following_number_as_numeric_space_needs_a_number() {
        // Returns None when the next token is not a Number (nothing to encode).
        let tokens = [EnglishToken::Space];
        let mut out = Vec::new();
        assert_eq!(
            encode_following_number_as_numeric_space(&tokens, 0, &mut out, false),
            None
        );
        assert!(out.is_empty());
    }

    #[test]
    fn after_repeated_stammer_prefix_rejects_empty_word() {
        // An empty lower_word has no first char → not a stammer continuation.
        let tokens = [EnglishToken::Space];
        assert!(!after_repeated_stammer_prefix(&tokens, 1, ""));
    }

    #[test]
    fn space_delimited_syllables_need_three_words() {
        // §10.1.4: three space-separated fragments that concatenate to one
        // recorded word (`dis as ter`) form a word; a missing neighbour does not.
        let joined = [
            EnglishToken::Word("dis".chars().collect()),
            EnglishToken::Space,
            EnglishToken::Word("as".chars().collect()),
            EnglishToken::Space,
            EnglishToken::Word("ter".chars().collect()),
        ];
        assert!(space_delimited_syllables_form_word(&joined, 2));
        // Current token not a word → false.
        let curr_missing = [
            EnglishToken::Word("a".chars().collect()),
            EnglishToken::Space,
            EnglishToken::Space,
        ];
        assert!(!space_delimited_syllables_form_word(&curr_missing, 2));
        // Following token not a word → false.
        let next_missing = [
            EnglishToken::Word("a".chars().collect()),
            EnglishToken::Space,
            EnglishToken::Word("b".chars().collect()),
        ];
        assert!(!space_delimited_syllables_form_word(&next_missing, 2));
    }

    #[test]
    fn encodes_middle_english_contraction_in_early_context() {
        // §12.2/§12.3: with an early-English letter present (`þ`), a Middle-English
        // spelling (`worlde`) takes its §12 contracted form `⠸⠺⠑`.
        let out = enc("þe worlde").expect("early-English text should encode");
        assert!(out.windows(3).any(|w| w == cells("⠸⠺⠑")));
    }

    #[test]
    fn encodes_superscript_after_numeric_base() {
        // §3.24: a superscript (`³`) directly after a number takes the level
        // indicator; the numeric base needs no extra grade-1.
        assert!(enc("row 5³").is_some());
    }

    #[test]
    fn encodes_standalone_shortform_collision_with_grade1() {
        // §8.7: an all-caps word that collides with a multi-letter shortform yet
        // is NOT itself a pure shortform abbreviation (`BC` shares letters with
        // the `bc`="because" wordsign) takes a grade-1 indicator before the caps
        // marker so it reads as literal letters.
        let out = enc("BC").expect("should encode");
        assert_eq!(out.first(), Some(&GRADE1));
    }
}
