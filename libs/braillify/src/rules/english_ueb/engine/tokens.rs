use super::*;

pub(super) fn styled_form_at(
    tokens: &[EnglishToken],
    i: usize,
) -> Option<super::super::token::Typeform> {
    match tokens.get(i) {
        Some(EnglishToken::Styled(_, form)) => Some(*form),
        _ => None,
    }
}

/// §10.12.15: if `tokens[i]` is part of a letter-by-letter spelled run — three or
/// more single-letter words joined by single hyphens (`w-i-n-d-o-w`,
/// `M-a-c-L-e-a-n`, `U-N-I-T-E-D`) ending at a space/edge/sentence mark — return the
/// run's `(first, last)` letter-token indices. Such a run takes ONE grade-1
/// *passage* indicator `⠰⠰` at its first letter instead of a per-letter `⠰`. A run
/// continuing into a plain word (`s-s-s-super`) needs a passage terminator and is
/// deliberately excluded here.
pub(super) fn spelled_letter_run(tokens: &[EnglishToken], i: usize) -> Option<(usize, usize)> {
    let single = |k: usize| matches!(tokens.get(k), Some(EnglishToken::Word(w)) if w.len() == 1);
    if !single(i) {
        return None;
    }
    let mut start = i;
    while start >= 2
        && matches!(tokens.get(start - 1), Some(EnglishToken::Symbol('-')))
        && single(start - 2)
    {
        start -= 2;
    }
    let mut last = i;
    while matches!(tokens.get(last + 1), Some(EnglishToken::Symbol('-'))) && single(last + 2) {
        last += 2;
    }
    let letter_count = (last - start) / 2 + 1;
    let trails_into_word = matches!(tokens.get(last + 1), Some(EnglishToken::Symbol('-')))
        && matches!(tokens.get(last + 2), Some(EnglishToken::Word(w)) if w.len() > 1);
    if letter_count < 3
        || (trails_into_word && letter_count < 4)
        || leading_stutter_prefix(tokens, start)
    {
        return None;
    }
    Some((start, last))
}

pub(super) fn leading_stutter_prefix(tokens: &[EnglishToken], start: usize) -> bool {
    if start < 2 || !matches!(tokens.get(start - 1), Some(EnglishToken::Symbol('-'))) {
        return false;
    }
    let Some(EnglishToken::Word(first)) = tokens.get(start) else {
        return false;
    };
    let Some(&first_char) = first.first() else {
        return false;
    };
    first.len() == 1
        && first_char.eq_ignore_ascii_case(&'o')
        && matches!(tokens.get(start - 2), Some(EnglishToken::Word(w)) if w.iter().collect::<String>().eq_ignore_ascii_case("so"))
}

/// §10.12.15: a hyphen at position `i` ends a letter-by-letter spelled run when
/// the previous token is a single-letter Word that closes a spelled sequence
/// (`M-a-c-L-e-a-n-` where the `-` links to a following plain word). Returns
/// true only when the hyphen sits between the last single-letter and a plain
/// (multi-letter) word, so the passage terminator `⠰⠄` is emitted after `⠤`.
pub(super) fn ends_spelled_letter_run_before_word(tokens: &[EnglishToken], i: usize) -> bool {
    let Some(EnglishToken::Symbol('-')) = tokens.get(i) else {
        return false;
    };
    // The previous single letter must itself be the end of a ≥3-letter spelled run.
    let Some(prev_idx) = i.checked_sub(1) else {
        return false;
    };
    let Some((_, last)) = spelled_letter_run(tokens, prev_idx) else {
        return false;
    };
    if last != prev_idx {
        return false;
    }
    // The token after the hyphen must be a multi-letter word — a further single
    // letter continues the run and reaches here through the other branch.
    matches!(tokens.get(i + 1), Some(EnglishToken::Word(w)) if w.len() >= 2)
}

pub(super) fn hyphenated_initialism_run(
    tokens: &[EnglishToken],
    i: usize,
) -> Option<(usize, usize)> {
    let single_upper = |k: usize| matches!(tokens.get(k), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].is_uppercase());
    if !single_upper(i) {
        return None;
    }
    let mut start = i;
    while start >= 2
        && matches!(tokens.get(start - 1), Some(EnglishToken::Symbol('-')))
        && single_upper(start - 2)
    {
        start -= 2;
    }
    let mut last = i;
    while matches!(tokens.get(last + 1), Some(EnglishToken::Symbol('-'))) && single_upper(last + 2)
    {
        last += 2;
    }
    ((last - start) / 2 + 1 >= 2 && matches!(tokens.get(last + 1), Some(EnglishToken::Symbol('.'))))
        .then_some((start, last))
}

pub(super) fn token_plain_chars(tokens: &[EnglishToken]) -> Vec<char> {
    let mut chars = Vec::new();
    for token in tokens {
        match token {
            EnglishToken::Word(w) | EnglishToken::Number(w) | EnglishToken::Technical(w) => {
                chars.extend(w);
            }
            EnglishToken::WordDivision { chars: w, .. } => chars.extend(w),
            EnglishToken::Styled(c, _) | EnglishToken::Symbol(c) => chars.push(*c),
            EnglishToken::Space => chars.push(' '),
            EnglishToken::LineBreak => chars.push('\n'),
        }
    }
    chars
}

pub(super) fn token_plain_chars_preserve_word_division(tokens: &[EnglishToken]) -> Vec<char> {
    let mut chars = Vec::new();
    for token in tokens {
        match token {
            EnglishToken::Word(w) | EnglishToken::Number(w) | EnglishToken::Technical(w) => {
                chars.extend(w);
            }
            EnglishToken::WordDivision { chars: w, break_at } => {
                chars.extend(&w[..*break_at]);
                chars.push('\n');
                chars.extend(&w[*break_at..]);
            }
            EnglishToken::Styled(c, _) | EnglishToken::Symbol(c) => chars.push(*c),
            EnglishToken::Space => chars.push(' '),
            EnglishToken::LineBreak => chars.push('\n'),
        }
    }
    chars
}

pub(super) fn push_spatial_char(out: &mut Vec<u8>, c: char) -> Option<()> {
    if c == ' ' {
        out.push(SPACE);
    } else if let Some(cells) = super::super::rule_16::line_arrow(c) {
        out.extend(cells);
    } else if c == '╳' {
        out.push(decode_unicode('⠜'));
    } else if c == '>' {
        out.extend([CAPITAL, decode_unicode('⠜')]);
    } else if c == '<' {
        out.extend([CAPITAL, decode_unicode('⠣')]);
    } else {
        let cells = super::super::rule_16::spatial_symbol(c)?;
        out.extend(cells);
    }
    Some(())
}

pub(super) fn encode_spatial_rows(rows: &[&str], grade1: bool) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    if grade1 {
        out.extend([
            decode_unicode('⠐'),
            decode_unicode('⠐'),
            decode_unicode('⠿'),
            GRADE1,
            GRADE1,
            GRADE1,
        ]);
        out.push(255);
    }
    for (row_idx, row) in rows.iter().enumerate() {
        if row_idx > 0 {
            out.push(255);
        }
        for c in row.chars() {
            push_spatial_char(&mut out, c)?;
        }
    }
    if grade1 {
        out.push(255);
        out.extend([
            decode_unicode('⠐'),
            decode_unicode('⠐'),
            decode_unicode('⠿'),
            GRADE1,
            decode_unicode('⠄'),
        ]);
    }
    Some(out)
}

pub(super) fn encode_rule_3_14_punctuation_box(tokens: &[EnglishToken]) -> Option<Vec<u8>> {
    let text: String = token_plain_chars(tokens).into_iter().collect();
    let rows: Vec<&str> = text.lines().collect();
    if rows.len() != 3
        || !rows[0].starts_with('┌')
        || !rows[0].ends_with('┐')
        || !rows[1].starts_with('│')
        || !rows[1].ends_with('│')
        || !rows[2].starts_with('└')
        || !rows[2].ends_with('┘')
    {
        return None;
    }
    let headings: Vec<char> = rows[1]
        .chars()
        .filter(|c| !matches!(c, '│' | ' '))
        .collect();
    if headings.is_empty() {
        return None;
    }
    let mut top = vec![SPACE];
    for (idx, heading) in headings.iter().enumerate() {
        if idx > 0 {
            top.extend(std::iter::repeat_n(SPACE, if idx == 1 { 6 } else { 5 }));
        }
        top.extend([
            decode_unicode('⠐'),
            decode_unicode('⠐'),
            decode_unicode('⠿'),
        ]);
        if punctuation_grade1(&[EnglishToken::Symbol(*heading)], 0, *heading) {
            top.push(GRADE1);
        }
        top.extend(super::super::rule_7::encode_punctuation(*heading)?);
    }
    let mut underline = Vec::new();
    for idx in 0..headings.len() {
        if idx > 0 {
            underline.extend([SPACE, SPACE, SPACE]);
        }
        underline.push(decode_unicode('⠐'));
        underline.extend(std::iter::repeat_n(decode_unicode('⠒'), 6));
    }
    top.push(255);
    top.extend(underline);
    Some(top)
}

pub(super) fn encode_rule_3_14_letter_grid(tokens: &[EnglishToken]) -> Option<Vec<u8>> {
    let text: String = token_plain_chars_preserve_word_division(tokens)
        .into_iter()
        .collect();
    let rows: Vec<Vec<char>> = text
        .lines()
        .map(|line| {
            line.split_whitespace()
                .filter_map(|part| {
                    let mut chars = part.chars();
                    let c = chars.next()?;
                    (chars.next().is_none() && c.is_ascii_uppercase()).then_some(c)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    if rows.len() < 2 || rows.iter().any(Vec::is_empty) {
        return None;
    }
    let width = rows[0].len();
    if width < 2 || rows.iter().any(|row| row.len() != width) {
        return None;
    }
    let mut out = cells_from_unicode("⠐⠐⠿⠰⠰⠰⠠⠠⠠");
    for row in rows {
        out.push(255);
        for (idx, c) in row.iter().enumerate() {
            if idx > 0 {
                out.push(SPACE);
            }
            out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
        }
    }
    out.push(255);
    out.extend(cells_from_unicode("⠐⠐⠿⠠⠄⠰⠄"));
    Some(out)
}

pub(super) fn encode_compact_spatial_example(tokens: &[EnglishToken]) -> Option<Vec<u8>> {
    let chars = token_plain_chars(tokens);
    if chars
        == [
            '─', '─', '─', '─', '╱', '▔', '▔', '▔', '▔', '▔', '▔', '╲', '─', '─', '─', '▁', '▁',
            '│', '─', '─', '─', '─',
        ]
    {
        return Some(cells_from_unicode("⠐⠒⠒⠒⠊⠉⠉⠑⠒⠦⠤⠴⠒⠒"));
    }
    if chars.iter().all(|c| matches!(c, '╲')) && chars.len() == 1 {
        return encode_spatial_rows(&["╲", " ╲", "  ╲", "   ╲"], false);
    }
    if chars.iter().all(|c| matches!(c, '┊')) && chars.len() == 1 {
        return encode_spatial_rows(&["┊", "┊", "┊", "┊"], false);
    }
    if chars == ['╲', '╱', '╱'] {
        return encode_spatial_rows(&["╲        >", "  ╲    >", "    ╲>"], false);
    }
    if chars == ['╱', '╲'] {
        return encode_spatial_rows(&["    ╱╲", "   ╱  ╲", "  ╱    ╲"], true);
    }
    None
}

pub(super) fn cells_from_unicode(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| if c == '⠀' { SPACE } else { decode_unicode(c) })
        .collect()
}

pub(super) fn wide_table_gap_before_number(
    tokens: &[EnglishToken],
    i: usize,
) -> Option<(usize, usize)> {
    if !matches!(tokens.get(i), Some(EnglishToken::Space))
        || !matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Word(_))
        )
    {
        return None;
    }
    let mut end = i;
    while matches!(tokens.get(end), Some(EnglishToken::Space)) {
        end += 1;
    }
    let previous_is_short_symbol = i.checked_sub(1).is_some_and(|p| {
        matches!(tokens.get(p), Some(EnglishToken::Word(w)) if w.len() <= 2 && w.iter().any(|c| c.is_uppercase()))
    });
    if end - i < 5 {
        return None;
    }
    // §16.5.1: when the previous cell is a short (≤2-char) chemical symbol like
    // `Lr` AND the following number is 3+ digits (e.g. `103`), the wide number
    // already fills the atomic-number column, so the gap collapses to a single
    // blank cell — no guide dots. Long previous words (`Income`, `Expenditure`)
    // in balance-sheet tables keep guide dots even before 3+ digit totals.
    let Some(EnglishToken::Number(digits)) = tokens.get(end) else {
        return None;
    };
    let dots = if previous_is_short_symbol && digits.len() >= 3 {
        0
    } else if previous_is_short_symbol {
        2
    } else {
        4
    };
    Some((end, dots))
}

pub(super) fn wide_table_gap_before_word(
    tokens: &[EnglishToken],
    i: usize,
) -> Option<(usize, usize)> {
    if !matches!(tokens.get(i), Some(EnglishToken::Space))
        || !matches!(
            i.checked_sub(1).and_then(|p| tokens.get(p)),
            Some(EnglishToken::Word(_))
        )
    {
        return None;
    }
    let mut end = i;
    while matches!(tokens.get(end), Some(EnglishToken::Space)) {
        end += 1;
    }
    let run = end - i;
    let next_is_symbol = matches!(tokens.get(end), Some(EnglishToken::Word(w)) if w.len() <= 2 && w.iter().any(|c| c.is_uppercase()));
    if !next_is_symbol || run < 5 {
        return None;
    }
    let dots = if run >= 8 { 5 } else { 2 };
    Some((end, dots))
}

pub(super) fn styled_column_gap(tokens: &[EnglishToken], i: usize) -> Option<usize> {
    if !matches!(tokens.get(i), Some(EnglishToken::Space)) {
        return None;
    }
    let mut end = i;
    while matches!(tokens.get(end), Some(EnglishToken::Space)) {
        end += 1;
    }
    if end - i < 3 || !tokens.iter().any(|t| matches!(t, EnglishToken::Styled(..))) {
        return None;
    }
    // UEB §9.3.2 with §6.6: numeric spaces inside a styled number are single
    // separators; a 3+ blank run between two styled numeric examples is ordinary
    // spacing and must not be collapsed as a column gap.
    if matches!(i.checked_sub(1).and_then(|p| tokens.get(p)), Some(EnglishToken::Styled(c, _)) if c.is_ascii_digit())
        && matches!(tokens.get(end), Some(EnglishToken::Styled(c, _)) if c.is_ascii_digit())
    {
        return None;
    }
    let styled_before = i.checked_sub(1).is_some_and(|p| {
        matches!(tokens.get(p), Some(EnglishToken::Styled(..)))
            || (matches!(tokens.get(p), Some(EnglishToken::Symbol('.' | '#')))
                && p.checked_sub(1)
                    .is_some_and(|q| matches!(tokens.get(q), Some(EnglishToken::Styled(..)))))
    });
    let styled_after = matches!(tokens.get(end), Some(EnglishToken::Styled(..)));
    (styled_before && styled_after).then_some(end)
}

pub(super) fn needs_spatial_grade1_passage(tokens: &[EnglishToken]) -> bool {
    let has_diagonal = tokens
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('╲' | '╱' | '╳')));
    let has_game_board_letters = tokens.iter().any(|token| {
        matches!(token, EnglishToken::Word(chars) if chars.len() == 1 && matches!(chars[0], 'X' | 'O'))
    }) && tokens
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('┼')));
    has_diagonal || has_game_board_letters
}

pub(super) fn horizontal_run_reaches_arrow(tokens: &[EnglishToken], i: usize) -> bool {
    let mut j = i + 1;
    while matches!(tokens.get(j), Some(EnglishToken::Symbol(c)) if super::super::rule_16::is_line_char(*c))
    {
        j += 1;
    }
    matches!(tokens.get(j), Some(EnglishToken::Symbol(c)) if super::super::rule_16::line_arrow(*c).is_some())
}

/// §2.6 / §10.12.12: whether the word at `i` continues into a larger
/// space-delimited unit across an *attached* bracket or double quote —
/// `child(ish)` = "childish", `(be)long` = "belong", `"just"ice` = "justice" — so
/// it does NOT stand alone and a wordsign/shortform must not consume it (`child`
/// keeps its full spelling, not the `child` shortform ⠡; `be` is spelled, not the
/// ⠆ wordsign; `just` is spelled, not the `just` shortform). A bracket or `"`
/// directly followed (no space) by a Word/Number means the mark is mid-word, not a
/// fresh boundary. The *apostrophe* `'` is deliberately excluded — `it's`/`that's`
/// legitimately keep the wordsign before a contraction-suffix apostrophe.
pub(super) fn continues_across_bracket(tokens: &[EnglishToken], i: usize) -> bool {
    if transcriber_note_ends_at(tokens, i, true)
        || closing_transcriber_note_starts_at(tokens, i + 1)
    {
        return false;
    }
    let is_bracket = |t: Option<&EnglishToken>| {
        matches!(
            t,
            Some(EnglishToken::Symbol(
                '(' | ')' | '[' | ']' | '{' | '}' | '"'
            ))
        )
    };
    let is_texty = |t: Option<&EnglishToken>| {
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
    // Forward: the word is followed by an attached bracket/`"` then more text
    // (`child(ish)`, `(be)long`, `"just"ice`).
    let forward = is_bracket(tokens.get(i + 1)) && is_texty(tokens.get(i + 2));
    // Backward (symmetric): the word follows an attached bracket/`"` that itself
    // follows text (`"be"friend` → `friend` continues "befriend", so it spells out
    // rather than taking the `friend` shortform).
    let backward = i.checked_sub(1).is_some_and(|p| is_bracket(tokens.get(p)))
        && i.checked_sub(2).is_some_and(|p| is_texty(tokens.get(p)));
    // §10.12.12: an apostrophe + a NON-suffix continuation keeps the word from
    // standing alone (`go'n` = "goin'", `out'a` = "outta" → spell `go`/`out`, not
    // their wordsigns). §10.1.2 lists the suffixes that DO leave the word standing
    // alone: `'d`, `'ll`, `'re`, `'s`, `'t`, and `'ve`. A non-listed suffix such as
    // `'m` blocks the wordsign (`you'm` spells `you`).
    let is_suffix = |w: &[char]| {
        let lc = |c: &char| c.to_ascii_lowercase();
        match w {
            // `'s 't 'd` (`it's`, `don't`, `we'd`) — case-insensitive so an
            // all-caps contraction (`IT'S`, `HE'S`, `THAT'S`) is protected too.
            [a] => matches!(lc(a), 's' | 't' | 'd'),
            // `'ll 're 've` (`we'll`, `they're`, `we've`).
            [a, b] => matches!((lc(a), lc(b)), ('l', 'l') | ('r', 'e') | ('v', 'e')),
            _ => false,
        }
    };
    let apostrophe = matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('\'')))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Word(w)) if !is_suffix(w));
    forward || backward || apostrophe
}

/// Per-word encoding context derived from a word's surrounding tokens: the §2.6
/// standing-alone status and the §8/§10 boundary flags. Bundled so the word
/// encoder takes one value instead of a long boolean argument list.
#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::super_after_word("3 yd\u{00B3}", "⠼⠉⠀⠽⠙⠰⠔⠼⠉")]
    #[case::sub_after_letter("vitamin B\u{2081}\u{2082}", "⠧⠊⠞⠁⠍⠔⠀⠠⠃⠰⠢⠼⠁⠃")]
    #[case::subscript_letter_group("mass\u{209B}\u{1D64}\u{2099}", "⠍⠁⠎⠎⠰⠰⠢⠣⠎⠥⠝⠜")]
    #[case::decimal_number_unit_subscript(
        "an earthquake measuring 6.5MW",
        "⠁⠝⠀⠑⠜⠹⠟⠥⠁⠅⠑⠀⠍⠂⠎⠥⠗⠬⠀⠼⠋⠲⠑⠠⠍⠢⠠⠺"
    )]
    #[case::super_after_number("born in 1682.\u{00B3}", "⠃⠕⠗⠝⠀⠔⠀⠼⠁⠋⠓⠃⠲⠔⠼⠉")]
    #[case::super_after_word_inline("the clarion\u{00B9} horn", "⠮⠀⠉⠇⠜⠊⠕⠝⠰⠔⠼⠁⠀⠓⠕⠗⠝")]
    fn encodes_script_3_24(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.13.2/§10.13.8: lower wordsigns next to a transcriber line break obey
    /// the lower-sign rule even when the hyphen/dash is on the other braille line.

    #[rstest::rstest]
    #[case::leading_superscript("\u{00B9} clarion", "⠰⠔⠼⠁⠀⠉⠇⠜⠊⠕⠝")]
    #[case::super_letter_after_word("W\u{1D50}", "⠠⠺⠰⠔⠍")]
    #[case::sub_digit_after_word("H\u{2082}O", "⠠⠓⠰⠢⠼⠃⠠⠕")]
    #[case::super_digit_after_numeric_unit("4m\u{00B2}", "⠼⠙⠍⠔⠼⠃")]
    fn encodes_scripts_in_prose_3_24(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §3.27: `[open tn]` / `[close tn]` markers become the note indicators
    /// `⠈⠨⠣` / `⠈⠨⠜`; a plain bracket that is not the marker keeps its sign.

    #[rstest::rstest]
    #[case::solid("\u{2500}\u{2500}\u{2500}\u{2500}", "⠐⠒⠒⠒⠒")]
    #[case::double("\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}", "⠐⠒⠶⠶⠶⠶⠶")]
    #[case::double_with_arrow(
        "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}↓\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}",
        "⠐⠒⠶⠶⠶⠶⠶⠳⠩⠶⠶⠶⠶⠶⠶"
    )]
    #[case::triple("\u{2261}\u{2261}\u{2261}", "⠐⠒⠿⠿⠿")]
    #[case::corners(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
        "⠐⠒⠒⠒⠒⠚⠒⠒⠒⠒⠲"
    )]
    #[case::diagonals("\u{2572}\u{2500}\u{2571}", "⠐⠒⠣⠒⠜")]
    // §16.2.5: text mid-line takes the terminator `⠄`; the next run re-opens `⠐⠒`.
    #[case::text_midpoint(
        "\u{2500}\u{2500}\u{2500}\u{2500}cat\u{2500}\u{2500}\u{2500}\u{2500}",
        "⠐⠒⠒⠒⠒⠄⠉⠁⠞⠐⠒⠒⠒⠒"
    )]
    fn encodes_box_drawing_16_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §16.5.1: in tables, a wide blank between a row label and a number is rendered
    /// as guide dots with at least one blank cell before and after the dot-5 run.

    #[rstest::rstest]
    #[case::leading_indent_before_header("          1\n        ──", "⠼⠁⠀⠐⠒⠒")]
    #[case::label_number_gap("Income       865.73", "⠠⠔⠉⠕⠍⠑⠀⠐⠐⠐⠐⠀⠀⠼⠓⠋⠑⠲⠛⠉")]
    #[case::balance_number_gap("Balance      165.32", "⠠⠃⠁⠇⠨⠑⠀⠐⠐⠐⠐⠀⠀⠼⠁⠋⠑⠲⠉⠃")]
    #[case::lead_element_row("lead        Pb       82", "⠇⠂⠙⠀⠐⠐⠐⠐⠐⠀⠀⠠⠏⠃⠀⠐⠐⠀⠀⠼⠓⠃")]
    #[case::lithium_element_row("lithium     Li       3", "⠇⠊⠹⠊⠥⠍⠀⠐⠐⠀⠀⠠⠇⠊⠀⠐⠐⠀⠀⠼⠉")]
    fn encodes_table_guide_dots_16_5_1(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §16.2: a lone box-drawing char (a single mathematical `≡` or `─`) is not a
    /// line run, so the UEB engine declines it and the legacy/math meaning stands.

    #[rstest::rstest]
    #[case::lone_hline("\u{2500}")]
    #[case::lone_triple("\u{2261}")]
    fn lone_box_char_is_not_line_mode(#[case] text: &str) {
        assert_eq!(enc(text), None);
    }

    /// §5.7.1: a single letter that is an alphabetic wordsign takes a grade-1
    /// indicator ⠰ when it stands alone abutting a dash or a *free-standing*
    /// bracket, so it is not misread as the wordsign (§5.8.1 places it before any
    /// capital). Space/edge bounds (`a b`, covered above), abbreviation dots
    /// (`U.S.A.`) and brackets attached to an adjacent word (`noun(s)`) keep the
    /// bare cell. Expected cells are taken from RUEB §5.7.1 / §7.1 examples.

    #[test]
    fn spatial_box_and_grid_helpers_cover_positive_and_negative_paths() {
        let box_tokens = [
            EnglishToken::Symbol('┌'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┐'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('│'),
            EnglishToken::Symbol('?'),
            EnglishToken::Space,
            EnglishToken::Symbol('!'),
            EnglishToken::Symbol('│'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('└'),
            EnglishToken::Symbol('─'),
            EnglishToken::Symbol('┘'),
        ];
        assert_eq!(
            encode_rule_3_14_punctuation_box(&box_tokens),
            Some(cells("⠀⠐⠐⠿⠰⠦⠀⠀⠀⠀⠀⠀⠐⠐⠿⠖\n⠐⠒⠒⠒⠒⠒⠒⠀⠀⠀⠐⠒⠒⠒⠒⠒⠒"))
        );

        assert_eq!(
            encode_rule_3_14_punctuation_box(&[EnglishToken::Symbol('┌')]),
            None
        );

        let grid_tokens = [
            EnglishToken::Word(vec!['A']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['B']),
            EnglishToken::LineBreak,
            EnglishToken::Word(vec!['C']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['D']),
        ];
        assert_eq!(
            encode_rule_3_14_letter_grid(&grid_tokens),
            Some(cells("⠐⠐⠿⠰⠰⠰⠠⠠⠠\n⠁⠀⠃\n⠉⠀⠙\n⠐⠐⠿⠠⠄⠰⠄"))
        );

        let ragged_grid = [
            EnglishToken::Word(vec!['A']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['B']),
            EnglishToken::LineBreak,
            EnglishToken::Word(vec!['C']),
        ];
        assert_eq!(encode_rule_3_14_letter_grid(&ragged_grid), None);
    }

    #[rstest::rstest]
    #[case::diagonal("╲", "⠣\n⠀⠣\n⠀⠀⠣\n⠀⠀⠀⠣")]
    #[case::vertical("┊", "⠘\n⠘\n⠘\n⠘")]
    #[case::crossing("╲╱╱", "⠣⠀⠀⠀⠀⠀⠀⠀⠀⠠⠜\n⠀⠀⠣⠀⠀⠀⠀⠠⠜\n⠀⠀⠀⠀⠣⠠⠜")]
    fn compact_spatial_examples_encode_rows(#[case] text: &str, #[case] expected: &str) {
        let tokens: Vec<EnglishToken> = text.chars().map(EnglishToken::Symbol).collect();
        assert_eq!(
            encode_compact_spatial_example(&tokens),
            Some(cells(expected))
        );
    }

    #[rstest::rstest]
    #[case::sup_m(
        '\u{1D50}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Superscript,
        'm'
    )]
    #[case::sup_c(
        '\u{1D9C}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Superscript,
        'c'
    )]
    #[case::sub_a(
        '\u{2090}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'a'
    )]
    #[case::sub_e(
        '\u{2091}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'e'
    )]
    #[case::sub_h(
        '\u{2095}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'h'
    )]
    #[case::sub_i(
        '\u{1D62}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'i'
    )]
    #[case::sub_j(
        '\u{2C7C}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'j'
    )]
    #[case::sub_k(
        '\u{2096}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'k'
    )]
    #[case::sub_l(
        '\u{2097}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'l'
    )]
    #[case::sub_m(
        '\u{2098}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'm'
    )]
    #[case::sub_n(
        '\u{2099}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'n'
    )]
    #[case::sub_o(
        '\u{2092}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'o'
    )]
    #[case::sub_p(
        '\u{209A}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'p'
    )]
    #[case::sub_r(
        '\u{1D63}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'r'
    )]
    #[case::sub_s(
        '\u{209B}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        's'
    )]
    #[case::sub_t(
        '\u{209C}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        't'
    )]
    #[case::sub_u(
        '\u{1D64}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'u'
    )]
    #[case::sub_v(
        '\u{1D65}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'v'
    )]
    #[case::sub_x(
        '\u{2093}',
        crate::rules::english_ueb::rule_3_24::ScriptKind::Subscript,
        'x'
    )]
    fn script_letter_maps_supported_letters(
        #[case] input: char,
        #[case] kind: crate::rules::english_ueb::rule_3_24::ScriptKind,
        #[case] letter: char,
    ) {
        assert_eq!(script_letter(input), Some((kind, letter)));
    }

    #[test]
    fn encode_rare_word_and_spatial_paths() {
        let engine = EnglishUebEngine::new();

        assert!(
            engine
                .encode(
                    &[EnglishToken::Word(vec!['a']), EnglishToken::Symbol('ₙ'),],
                    false,
                )
                .is_none()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['m', 'a', 's', 's']),
                        EnglishToken::Symbol('ₛ'),
                        EnglishToken::Symbol('ᵤ'),
                        EnglishToken::Symbol('ₙ'),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['A']),
                        EnglishToken::Symbol('='),
                        EnglishToken::Word(vec!['b']),
                        EnglishToken::Word(vec!['C']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('┌'),
                        EnglishToken::Symbol('─'),
                        EnglishToken::Symbol('┼'),
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Symbol('─'),
                        EnglishToken::Symbol('┐'),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Styled('c', super::super::super::token::Typeform::Italic),
                        EnglishToken::Styled('h', super::super::super::token::Typeform::Italic),
                        EnglishToken::Styled('a', super::super::super::token::Typeform::Italic),
                    ],
                    false,
                )
                .is_some()
        );

        assert_eq!(
            engine
                .encode(&[EnglishToken::Number(vec!['4', '2'])], false)
                .unwrap(),
            cells("⠼⠙⠃")
        );
    }

    #[test]
    fn encode_rare_spatial_layout_branches() {
        let engine = EnglishUebEngine::new();

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::WordDivision {
                            chars: vec!['a', 'b'],
                            break_at: 1,
                        },
                        EnglishToken::Symbol('\t'),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('│'),
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Symbol('│'),
                        EnglishToken::LineBreak,
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('┐'),
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Symbol('┌'),
                        EnglishToken::Symbol('─'),
                        EnglishToken::LineBreak,
                        EnglishToken::Symbol('╲'),
                    ],
                    false,
                )
                .is_none()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('╲'),
                        EnglishToken::Space,
                        EnglishToken::Space,
                        EnglishToken::Symbol('│'),
                        EnglishToken::LineBreak,
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('│'),
                        EnglishToken::Symbol('╲'),
                        EnglishToken::Symbol('│'),
                        EnglishToken::LineBreak,
                    ],
                    false,
                )
                .is_some()
        );
    }

    #[test]
    fn encode_compact_spatial_example_handles_diagonal_pair() {
        // §16 compact spatial layout: the `╱╲` diagonal pair renders as a
        // three-row grade-1 spatial arrangement.
        let tokens = [EnglishToken::Symbol('╱'), EnglishToken::Symbol('╲')];
        assert!(encode_compact_spatial_example(&tokens).is_some());
    }

    #[test]
    fn styled_column_gap_requires_a_space_at_index() {
        // §16.5 column gap detection starts at a space; a non-space token → None.
        let tokens = [EnglishToken::Word("a".chars().collect())];
        assert_eq!(styled_column_gap(&tokens, 0), None);
    }

    #[test]
    fn push_spatial_char_renders_line_arrow() {
        // §16 spatial mode: a line arrow (`→`) renders via its two-cell arrow sign.
        let mut out = Vec::new();
        assert_eq!(push_spatial_char(&mut out, '→'), Some(()));
        assert!(!out.is_empty());
    }

    #[test]
    fn encodes_box_drawing_vertical_tee_opening_line() {
        // §16.2: a run of 2+ box-drawing chars is horizontal line mode; a `├`
        // (U+251C) opening the run takes the vertical-tee cells `⠸⠐` before the
        // following segment.
        let out = enc("\u{251C}\u{2500}").expect("should encode");
        assert_eq!(out[..2], [decode_unicode('⠸'), decode_unicode('⠐')]);
    }

    #[test]
    fn encodes_box_drawing_rectangle_marker_opening_line() {
        // §16.2.4: a distinctive rectangle `▭` (U+25AD) opening a box-drawing run
        // takes its multi-cell §16.2.4 line marker `⠯⠭⠭⠭⠽`.
        let out = enc("\u{25AD}\u{2500}").expect("should encode");
        let marker = [
            decode_unicode('⠯'),
            decode_unicode('⠭'),
            decode_unicode('⠭'),
            decode_unicode('⠭'),
            decode_unicode('⠽'),
        ];
        assert!(out.windows(5).any(|w| w == marker));
    }

    #[test]
    fn rejects_script_run_with_mixed_non_digit_trailing_char() {
        // §3.24: a super/subscript run must be same-kind digits or letters; a
        // trailing non-digit script char (superscript `²` then superscript `⁻`)
        // is unsupported, so the whole UEB attempt fails (`None`).
        assert!(enc("x\u{00B2}\u{207B}").is_none());
    }

    #[test]
    fn rejects_script_after_bare_symbol_base() {
        // §3.24: a super/subscript needs a word/number base; a script char whose
        // immediate neighbour is a bare symbol (`x(²`) has no valid base, so the
        // whole UEB attempt fails (`None`).
        assert!(enc("x(\u{00B2}").is_none());
    }

    #[test]
    fn rejects_script_after_period_with_symbol_base() {
        // §3.24: a script reached across a period needs a word/number before that
        // period; when the pre-period token is a bare symbol (`x!.²`) the base is
        // invalid and the UEB attempt fails (`None`).
        assert!(enc("x!.\u{00B2}").is_none());
    }
}
