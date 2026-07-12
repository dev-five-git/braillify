//! §11 Technical Material.
//!
//! Encodes `$...$` technical spans per UEB §11:
//! - §11.2 operation/comparison signs
//! - §11.3 fractions (simple numeric ⠌; general ⠷ ⠨⠌ ⠾ with grade-1 indicators)
//! - §11.4 super/sub scripts ⠔ ⠢ with "item" scope (§11.4.1) and braille grouping ⠣ ⠜
//! - §11.5 radicals ⠩ ⠬
//! - §11.6 arrows ⠳ + tip cells
//! - §11.7 shape symbols ⠫ + description; terminator ⠱
//! - §11.8 matrices — big parens ⠠⠐⠣ ⠠⠐⠜ per row
//! - §11.9 chemistry — grade-1 passage
//!
//! Grade-1 policy (§5, §11.3.4 Note, §11.4.2 Note, §11.5.1 Note):
//! - If the whole span contains a construct whose cells collide with grade-2
//!   groupsigns (⠩⠬ radical, ⠷⠾ fraction with single-letter operands, ⠣⠜ braille
//!   grouping), wrap in grade-1 WORD `⠰⠰` — one indicator covers everything until
//!   the next space.
//! - Otherwise, emit grade-1 SYMBOL `⠰` per grade-1-needing cell (⠷/⠾/⠔/⠢/⠩/⠬).

use crate::unicode::decode_unicode;

const GRADE1: u8 = decode_unicode('⠰');
const NUMERIC: u8 = decode_unicode('⠼');

fn push_cells(out: &mut Vec<u8>, s: &str) {
    out.extend(s.chars().map(decode_unicode));
}

fn digit(c: char) -> Option<u8> {
    super::rule_6::digit_cell(c)
}

fn encode_number_run(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    out.push(NUMERIC);
    for &c in chars {
        match c {
            '0'..='9' => out.push(digit(c)?),
            '.' => out.push(decode_unicode('⠲')),
            ',' => out.push(decode_unicode('⠂')),
            _ => return None,
        }
    }
    Some(())
}

fn encode_denominator_item(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    if chars
        .iter()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | ','))
    {
        for &c in chars {
            if c.is_ascii_digit() {
                out.push(digit(c)?);
            } else if c == '.' {
                out.push(decode_unicode('⠲'));
            } else {
                out.push(decode_unicode('⠂'));
            }
        }
    } else {
        encode_expr_with_options(chars, out, false, false)?;
    }
    Some(())
}

fn take_braced(chars: &[char], start: usize) -> Option<(usize, &[char])> {
    if chars.get(start) != Some(&'{') {
        return None;
    }
    let mut depth = 1usize;
    for i in start + 1..chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((i + 1, &chars[start + 1..i]));
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether the item after `^{...}` or `_{...}` is a "single item" per §11.4.1
/// (no braille grouping needed) or a multi-part expression that needs `⠣...⠜`.
///
/// Single items per §11.4.1: entire number, entire fraction, entire radical, single
/// arrow/shape, expression in parens. Everything else (mixed digit+letter, letter
/// runs with operators) needs braille grouping.
fn item_needs_braille_grouping(item: &[char]) -> bool {
    if item.is_empty() {
        return false;
    }
    // §11.3 fraction as a single item.
    if item.starts_with(&['\\', 'f', 'r', 'a', 'c']) {
        return false;
    }
    // §11.5 radical as a single item.
    if item.starts_with(&['\\', 's', 'q', 'r', 't']) {
        return false;
    }
    // Balanced parens/brackets as a single item.
    if matches!(item[0], '(' | '[' | '{') && matches!(item[item.len() - 1], ')' | ']' | '}') {
        return false;
    }
    // Pure numeric run (digits + decimals + commas).
    if item
        .iter()
        .all(|c| c.is_ascii_digit() || matches!(*c, '.' | ','))
    {
        return false;
    }
    // Single character/symbol.
    if item.len() == 1 {
        return false;
    }
    true
}

/// Whether the whole top-level expression should be wrapped in `⠰⠰` (grade-1 word).
/// Grade-1 word persists until the next space, so it works when the technical
/// content is space-free.
fn needs_grade1_word_wrapper(chars: &[char]) -> bool {
    // Space breaks a grade-1 word — bail if the top-level has a space that is not
    // inside braces (space is significant for §11.2.2 comparison signs).
    let mut depth = 0i32;
    for &c in chars {
        match c {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ' ' if depth == 0 => return has_grade1_construct(chars),
            _ => {}
        }
    }
    has_grade1_construct(chars)
}

/// Whether the expression contains a construct whose cells collide with grade-2
/// groupsigns (⠩⠬ radical, ⠷⠾ fraction with letter operands, ⠣⠜ braille grouping).
fn has_grade1_construct(chars: &[char]) -> bool {
    // §11.5 radical: `⠩` and `⠬` are also groupsigns.
    if chars.windows(5).any(|w| w == ['\\', 's', 'q', 'r', 't']) {
        return true;
    }
    // §11.3.4 general fraction with single-letter operands — each ⠭/⠽ inside
    // ⠷…⠾ would otherwise need its own grade-1 (`⠰⠭` etc. per §5.7.1 wordsign),
    // so covering the whole span with one grade-1 word is more compact. A
    // multi-letter operand (`distance`) keeps grade-2 for its contractions and
    // takes the grade-1 SYMBOL path (⠰⠷…⠰⠾).
    let mut i = 0;
    while i + 5 <= chars.len() {
        if chars[i..].starts_with(&['\\', 'f', 'r', 'a', 'c'])
            && let Some((after_num, num)) = take_braced(chars, i + 5)
            && let Some((_, den)) = take_braced(chars, after_num)
            && num.len() == 1
            && den.len() == 1
            && num[0].is_ascii_alphabetic()
            && den[0].is_ascii_alphabetic()
        {
            return true;
        }
        i += 1;
    }
    // §11.4.3 superscript/subscript needing braille grouping.
    (0..chars.len().saturating_sub(2)).any(|j| {
        matches!(chars[j], '^' | '_')
            && chars[j + 1] == '{'
            && take_braced(chars, j + 1).is_some_and(|(_, item)| item_needs_braille_grouping(item))
    })
}

fn simple_fraction(chars: &[char], out: &mut Vec<u8>) -> Option<usize> {
    if !chars.starts_with(&['\\', 'f', 'r', 'a', 'c']) {
        return None;
    }
    let (after_num, num) = take_braced(chars, 5)?;
    let (after_den, den) = take_braced(chars, after_num)?;
    if !num
        .iter()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | ','))
        || !den
            .iter()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | ','))
    {
        return None;
    }
    encode_number_run(num, out)?;
    out.push(decode_unicode('⠌'));
    encode_denominator_item(den, out)?;
    Some(after_den)
}

fn general_fraction(chars: &[char], out: &mut Vec<u8>, in_grade1_word: bool) -> Option<usize> {
    if !chars.starts_with(&['\\', 'f', 'r', 'a', 'c']) {
        return None;
    }
    let (after_num, num) = take_braced(chars, 5)?;
    let (after_den, den) = take_braced(chars, after_num)?;
    if in_grade1_word {
        // Whole span is already in grade-1 word mode: emit bare open/close.
        out.push(decode_unicode('⠷'));
        encode_expr_with_options(num, out, false, true)?;
        push_cells(out, "⠨⠌");
        encode_expr_with_options(den, out, false, true)?;
        out.push(decode_unicode('⠾'));
    } else {
        // Grade-2 context: grade-1 symbol before each of ⠷ and ⠾. Inner content
        // may contain contractible words (grade-2 continues).
        out.push(GRADE1);
        out.push(decode_unicode('⠷'));
        encode_fraction_operand(num, out)?;
        push_cells(out, "⠨⠌");
        encode_fraction_operand(den, out)?;
        out.push(GRADE1);
        out.push(decode_unicode('⠾'));
    }
    Some(after_den)
}

/// §11.3.4 fraction operand encoding. Same as [`encode_expr_with_options`] in
/// grade-2 mode, except that a multi-letter alphabetic operand starting with
/// `dis`/`con`/`be` spells the prefix letters and grade-2-encodes the rest — the
/// PDF example on page 212 renders `distance` as `⠙⠊⠌⠨⠑` (d-i-st-ance) instead
/// of `⠲⠞⠨⠑` (dis-t-ance) so the restricted lower groupsigns (§10.6.5) do not
/// collide with the technical `⠲`/`⠆`/`⠒` cells used elsewhere in the span.
/// Strong groupsigns (`⠌` st, `⠨⠑` ance) and shortforms (`⠐⠞` time) still apply.
fn encode_fraction_operand(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    if chars.iter().all(char::is_ascii_alphabetic) && chars.len() >= 4 {
        let lower: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
        // §11.3.4 PDF example encodes "distance" as ⠙⠊⠌⠨⠑ (d-i-st-ance) — the
        // §10.6.5 "dis" restricted groupsign is bypassed inside a general
        // fraction span, but the boundary letters must still let the "st" strong
        // groupsign (⠌) form. So spell one letter fewer than the prefix (`d`+`i`
        // for `dis`, `c`+`o` for `con`, none for `be`) and hand the tail to the
        // engine — engine.rs then applies `st`/`ance`/etc. as usual.
        let spell_count = if lower.starts_with("dis") || lower.starts_with("con") {
            2
        } else {
            return encode_expr_with_options(chars, out, false, false);
        };
        for &c in chars.iter().take(spell_count) {
            if c.is_ascii_uppercase() {
                out.push(decode_unicode('⠠'));
            }
            out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
        }
        let rest: String = chars[spell_count..].iter().collect();
        let tokens = super::parser::parse_english(&rest);
        out.extend(super::engine::EnglishUebEngine::new().encode(&tokens, true)?);
        return Some(());
    }
    encode_expr_with_options(chars, out, false, false)
}

fn encode_letter_run(chars: &[char], out: &mut Vec<u8>, in_grade1_word: bool) -> Option<()> {
    if in_grade1_word || chars.len() == 1 {
        // Bare letter encoding — grade-1 word or a single-letter run in prose.
        for &c in chars {
            if c.is_ascii_uppercase() {
                out.push(decode_unicode('⠠'));
            }
            out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
        }
    } else {
        // Multi-letter word — engine applies §10 contractions.
        let text: String = chars.iter().collect();
        let tokens = super::parser::parse_english(&text);
        out.extend(super::engine::EnglishUebEngine::new().encode(&tokens, true)?);
    }
    Some(())
}

fn encode_expr_with_options(
    chars: &[char],
    out: &mut Vec<u8>,
    suppress_level_grade1: bool,
    in_grade1_word: bool,
) -> Option<()> {
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            let prev = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
            let next = chars.get(i + 1).copied();
            if matches!(prev, Some('+' | '×')) || matches!(next, Some('+' | '×')) {
                i += 1;
                continue;
            }
            out.push(0);
            i += 1;
        } else if let Some(next) = simple_fraction(&chars[i..], out) {
            i += next;
        } else if let Some(next) = general_fraction(&chars[i..], out, in_grade1_word) {
            i += next;
        } else if chars[i..].starts_with(&['\\', 's', 'q', 'r', 't']) {
            // §11.5 radical. Open/close cells collide with groupsigns — if not
            // already in grade-1 word, wrap this expression segment.
            if !in_grade1_word {
                out.push(GRADE1);
            }
            push_cells(out, "⠩");
            let mut j = i + 5;
            if chars.get(j) == Some(&'[') {
                let end = chars[j + 1..].iter().position(|&c| c == ']')? + j + 1;
                out.push(decode_unicode('⠔'));
                encode_expr_with_options(&chars[j + 1..end], out, true, in_grade1_word)?;
                j = end + 1;
            }
            let (next, body) = take_braced(chars, j)?;
            encode_expr_with_options(body, out, true, in_grade1_word)?;
            push_cells(out, "⠬");
            i = next;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | ',')) {
                i += 1;
            }
            encode_number_run(&chars[start..i], out)?;
        } else if c.is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            encode_letter_run(&chars[start..i], out, in_grade1_word)?;
        } else if c == '^' || c == '_' {
            let up = c == '^';
            i += 1;
            let (next, item) = if chars.get(i) == Some(&'{') {
                take_braced(chars, i)?
            } else {
                (i + 1, &chars[i..i + 1])
            };
            let need_grouping = item_needs_braille_grouping(item);
            if need_grouping {
                if !in_grade1_word && !suppress_level_grade1 {
                    out.push(GRADE1);
                }
                out.push(decode_unicode(if up { '⠔' } else { '⠢' }));
                out.push(decode_unicode('⠣'));
                encode_expr_with_options(item, out, true, in_grade1_word)?;
                out.push(decode_unicode('⠜'));
            } else {
                if !in_grade1_word && !suppress_level_grade1 {
                    out.push(GRADE1);
                }
                out.push(decode_unicode(if up { '⠔' } else { '⠢' }));
                encode_expr_with_options(item, out, true, in_grade1_word)?;
            }
            i = next;
        } else {
            let cells = match c {
                '+' => "⠐⠖",
                '-' | '−' => "⠐⠤",
                '×' => "⠐⠦",
                '*' => "⠐⠦",
                '/' => "⠐⠌",
                '=' => "⠐⠶",
                '<' => "⠈⠣",
                '>' => "⠈⠜",
                '(' => "⠐⠣",
                ')' => "⠐⠜",
                ';' => "⠆",
                _ => return None,
            };
            push_cells(out, cells);
            i += 1;
        }
    }
    Some(())
}

/// Parse and encode a `\begin{pmatrix} row1 \\ row2 \end{pmatrix}` matrix per §11.8.
/// Each row is wrapped in big parens `⠠⠐⠣ … ⠠⠐⠜` and cells are space-separated.
fn encode_pmatrix(chars: &[char]) -> Option<Vec<u8>> {
    let text: String = chars.iter().collect();
    let begin = text.find("\\begin{pmatrix}")?;
    let end = text.find("\\end{pmatrix}")?;
    if end <= begin {
        return None;
    }
    let prefix = &text[..begin];
    let body = &text[begin + "\\begin{pmatrix}".len()..end];
    // Rows split by `\\`, cells by `&`. Body may have surrounding whitespace.
    let rows: Vec<Vec<String>> = body
        .split("\\\\")
        .map(|row| row.split('&').map(|cell| cell.trim().to_string()).collect())
        .collect();
    let mut out = Vec::new();
    // Encode the prefix (e.g. `I = ` before the matrix) through the engine.
    let prefix_trimmed = prefix.trim_end();
    if !prefix_trimmed.is_empty() {
        let tokens = super::parser::parse_english(prefix_trimmed);
        out.extend(super::engine::EnglishUebEngine::new().encode(&tokens, true)?);
        // Separator space between prefix and first row.
        out.push(0);
    }
    // Emit each row's big-paren enclosure.
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push(0);
        }
        push_cells(&mut out, "⠠⠐⠣");
        for (j, cell) in row.iter().enumerate() {
            if j > 0 {
                out.push(0);
            }
            encode_matrix_cell(cell, &mut out)?;
        }
        push_cells(&mut out, "⠠⠐⠜");
    }
    Some(out)
}

fn encode_matrix_cell(cell: &str, out: &mut Vec<u8>) -> Option<()> {
    let chars: Vec<char> = cell.chars().collect();
    if chars
        .iter()
        .all(|c| c.is_ascii_digit() || matches!(*c, '.' | ','))
    {
        encode_number_run(&chars, out)?;
    } else {
        // Mixed cells — encode via expression walker with grade-2 letters.
        encode_expr_with_options(&chars, out, false, false)?;
    }
    Some(())
}

/// Encode a `$...$` technical span according to §11.
pub fn encode_technical(chars: &[char]) -> Option<Vec<u8>> {
    // UEB 2024 §7.7.1/§11.8: LaTeX array notation with a right multi-line brace
    // represents vertically aligned print material; emit the appropriate multi-line
    // bracket on each braille line and top-justify the following text on row one.
    if let Some(cells) = encode_array_with_right_brace(chars) {
        return Some(cells);
    }
    // §11.8 matrix layout takes precedence over generic expression walking.
    if let Some(cells) = encode_pmatrix(chars) {
        return Some(cells);
    }
    if chars.windows(5).any(|w| w == ['\\', 's', 'q', 'r', 't']) {
        let mut out = vec![GRADE1, GRADE1];
        encode_expr_with_options(chars, &mut out, false, true)?;
        return Some(out);
    }
    if chars.contains(&' ') {
        let mut out = vec![GRADE1, GRADE1, GRADE1];
        encode_expr_with_options(chars, &mut out, false, true)?;
        out.extend([GRADE1, decode_unicode('⠄')]);
        return Some(out);
    }
    let mut out = Vec::new();
    let g1_word = needs_grade1_word_wrapper(chars);
    if g1_word {
        out.extend([GRADE1, GRADE1]);
    }
    encode_expr_with_options(chars, &mut out, false, g1_word)?;
    Some(out)
}

fn encode_array_with_right_brace(chars: &[char]) -> Option<Vec<u8>> {
    let text: String = chars.iter().collect();
    let begin = text.find("\\begin{array}{l}")?;
    let end = text.find("\\end{array}")?;
    if end <= begin || !text[..begin].ends_with("\\left.") {
        return None;
    }
    let after_array = &text[end + "\\end{array}".len()..];
    let suffix = after_array.strip_prefix("\\right\\}")?;
    let trailing = suffix
        .strip_prefix("\\text{")
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or("");
    let body = &text[begin + "\\begin{array}{l}".len()..end];
    let rows: Vec<&str> = body.split("\\\\").map(str::trim).collect();
    let mut out = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        if row_index > 0 {
            out.push(255);
        }
        let row_text = row
            .strip_prefix("\\text{")
            .and_then(|s| s.strip_suffix('}'))?;
        let tokens = super::parser::parse_english(row_text);
        out.extend(super::engine::EnglishUebEngine::new().encode(&tokens, true)?);
        if row_index > 0 {
            out.push(0);
        }
        push_cells(&mut out, "⠠⠸⠜");
        if row_index == 0 && !trailing.is_empty() {
            out.push(0);
            let tokens = super::parser::parse_english(trailing);
            out.extend(super::engine::EnglishUebEngine::new().encode(&tokens, true)?);
        }
    }
    Some(out)
}

/// §11.9 chemical formulae are a grade-1 passage; letters are literal and
/// numeric subscripts use the bare subscript indicator (§11.4 item scope).
pub fn encode_chemical(chars: &[char]) -> Option<Vec<u8>> {
    let mut out = vec![GRADE1, GRADE1, GRADE1];
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => {
                let prev = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
                let next = chars.get(i + 1).copied();
                if !matches!(prev, Some('+')) && !matches!(next, Some('+')) {
                    out.push(0);
                }
            }
            '→' => push_cells(&mut out, "⠳⠕"),
            '+' => push_cells(&mut out, "⠐⠖"),
            '₀'..='₉' => {
                out.push(decode_unicode('⠢'));
                out.push(NUMERIC);
                let digit_char = char::from(b'0' + (chars[i] as u32 - '₀' as u32) as u8);
                out.push(digit(digit_char)?);
            }
            c if c.is_ascii_digit() => {
                out.push(NUMERIC);
                out.push(digit(c)?);
            }
            c if c.is_ascii_alphabetic() => {
                if c.is_ascii_uppercase() {
                    out.push(decode_unicode('⠠'));
                }
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            _ => return None,
        }
        i += 1;
    }
    out.extend([GRADE1, decode_unicode('⠄')]);
    Some(out)
}

/// §11.7 shape symbols and §11.6 simple arrows used in English technical context.
pub fn encode_symbol(c: char, terminator: bool) -> Option<Vec<u8>> {
    let mut out = match c {
        '→' => vec![GRADE1, decode_unicode('⠳'), decode_unicode('⠕')],
        '↓' => vec![GRADE1, decode_unicode('⠳'), decode_unicode('⠩')],
        '△' => vec![GRADE1, decode_unicode('⠫'), NUMERIC, decode_unicode('⠉')],
        '☺' => vec![
            decode_unicode('⠈'),
            decode_unicode('⠫'),
            decode_unicode('⠎'),
            decode_unicode('⠋'),
        ],
        '⊕' => vec![
            GRADE1,
            decode_unicode('⠫'),
            decode_unicode('⠿'),
            decode_unicode('⠪'),
            decode_unicode('⠐'),
            decode_unicode('⠖'),
        ],
        '◯' => vec![decode_unicode('⠫'), decode_unicode('⠿')],
        _ => return None,
    };
    if terminator && matches!(c, '△') {
        out.push(decode_unicode('⠱'));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<u8> {
        s.chars().map(decode_unicode).collect()
    }

    /// §11.3 simple numeric fractions are numerator-first and keep numeric mode.
    #[rstest::rstest]
    #[case::five_eighths("\\frac{5}{8}", "⠼⠑⠌⠓")]
    #[case::decimal_over_comma("\\frac{5.7}{2,000}", "⠼⠑⠲⠛⠌⠃⠂⠚⠚⠚")]
    fn encodes_simple_fractions(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            encode_technical(&input.chars().collect::<Vec<_>>()),
            Some(cells(expected))
        );
    }

    /// §11.3.4 general fractions with single-letter operands use ⠰⠰ grade-1 word.
    #[test]
    fn encodes_letter_fraction_with_grade1_word() {
        assert_eq!(
            encode_technical(&"\\frac{x}{y}".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠷⠭⠨⠌⠽⠾"))
        );
    }

    /// §11.3.4 general fractions with contractible operands use ⠰ per delimiter.
    #[test]
    fn encodes_word_fraction_with_per_delimiter_grade1() {
        assert_eq!(
            encode_technical(&"\\frac{distance}{time}".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠷⠙⠊⠌⠨⠑⠨⠌⠐⠞⠰⠾"))
        );
    }

    #[test]
    fn fraction_operand_con_conserves_prefix_letters_before_contractions() {
        let mut out = Vec::new();

        encode_fraction_operand(&['c', 'o', 'n', 'c', 'e', 'p', 't'], &mut out)
            .expect("fraction operand should encode");

        assert!(out.starts_with(&cells("⠉⠕")));
    }

    #[test]
    fn fraction_operand_uppercase_restricted_prefix_marks_capital() {
        let mut out = Vec::new();

        encode_fraction_operand(&['D', 'i', 's', 't', 'a', 'n', 'c', 'e'], &mut out)
            .expect("fraction operand should encode");

        assert!(out.starts_with(&cells("⠠⠙⠊")));
    }

    #[rstest::rstest]
    #[case::plain_short_word(&['a', 'b', 'c'])]
    #[case::mixed_operand(&['x', '1'])]
    fn fraction_operand_falls_back_to_expression_walker(#[case] input: &[char]) {
        let mut out = Vec::new();

        encode_fraction_operand(input, &mut out).expect("fallback operand should encode");

        assert!(!out.is_empty());
    }

    #[test]
    fn denominator_item_handles_decimal_comma_digits_directly() {
        let mut out = Vec::new();

        encode_denominator_item(&['1', ',', '2', '.', '3'], &mut out)
            .expect("denominator number should encode");

        assert_eq!(out, cells("⠁⠂⠃⠲⠉"));
    }

    /// §11.4.3 superscript with braille-grouped item uses ⠰⠰ grade-1 word.
    #[test]
    fn encodes_superscript_group_with_grade1_word() {
        assert_eq!(
            encode_technical(&"x^{2y}".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠭⠔⠣⠼⠃⠽⠜"))
        );
    }

    /// §11.4.3 superscript whose item is a single fraction — bare base, one ⠰.
    #[test]
    fn encodes_superscript_of_fraction_with_symbol_grade1() {
        assert_eq!(
            encode_technical(&"x^{\\frac{2}{3}}".chars().collect::<Vec<_>>()),
            Some(cells("⠭⠰⠔⠼⠃⠌⠉"))
        );
    }

    /// §11.5.1 algebraic radical uses ⠰⠰ grade-1 word covering ⠩ through ⠬.
    #[test]
    fn encodes_algebraic_radical_with_grade1_word() {
        assert_eq!(
            encode_technical(&"\\sqrt{x^{2} + y^{2}}".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠩⠭⠔⠼⠃⠐⠖⠽⠔⠼⠃⠬"))
        );
    }

    /// §11.7 PDF-defined shape cells are fixed symbol mappings.
    #[rstest::rstest]
    #[case::triangle('△', false, "⠰⠫⠼⠉")]
    #[case::triangle_terminated('△', true, "⠰⠫⠼⠉⠱")]
    #[case::smiley('☺', false, "⠈⠫⠎⠋")]
    #[case::circled_plus('⊕', false, "⠰⠫⠿⠪⠐⠖")]
    fn encodes_shapes(#[case] c: char, #[case] term: bool, #[case] expected: &str) {
        assert_eq!(encode_symbol(c, term), Some(cells(expected)));
    }

    /// §11.2.2: technical multiplication is unspaced, while comparison signs keep
    /// their surrounding print spaces.
    #[test]
    fn operator_spacing_keeps_comparison_spaces() {
        assert_eq!(
            encode_technical(&"3.9 × 4.1 < 16".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠰⠼⠉⠲⠊⠐⠦⠼⠙⠲⠁⠀⠈⠣⠀⠼⠁⠋⠰⠄"))
        );
    }

    #[test]
    fn encodes_pmatrix_rows_and_mixed_cells() {
        let input = "I = \\begin{pmatrix}1&x\\\\2&3\\end{pmatrix}";
        let encoded = encode_technical(&input.chars().collect::<Vec<_>>()).unwrap();
        assert!(encoded.starts_with(&cells("⠠⠊⠀⠐⠶⠀⠠⠐⠣⠼⠁⠀⠭⠠⠐⠜")));
        assert!(encoded.ends_with(&cells("⠠⠐⠣⠼⠃⠀⠼⠉⠠⠐⠜")));
    }

    #[test]
    fn encodes_array_with_right_brace_and_trailing_text() {
        let input =
            "\\left.\\begin{array}{l}\\text{yes}\\\\\\text{no}\\end{array}\\right\\}\\text{choose}";
        let encoded = encode_technical(&input.chars().collect::<Vec<_>>()).unwrap();
        assert!(encoded.starts_with(&cells("⠽⠑⠎⠠⠸⠜⠀⠡⠕⠕⠎⠑")));
        assert!(encoded.contains(&255));
        assert!(encoded.ends_with(&cells("⠝⠕⠀⠠⠸⠜")));
    }

    #[rstest::rstest]
    #[case::subscript_seven('₇', "⠰⠰⠰⠠⠓⠢⠼⠛⠰⠄")]
    #[case::subscript_eight('₈', "⠰⠰⠰⠠⠓⠢⠼⠓⠰⠄")]
    #[case::subscript_nine('₉', "⠰⠰⠰⠠⠓⠢⠼⠊⠰⠄")]
    fn encodes_remaining_chemical_subscript_digits(
        #[case] subscript: char,
        #[case] expected: &str,
    ) {
        let input: Vec<char> = ['H', subscript].into_iter().collect();
        assert_eq!(encode_chemical(&input), Some(cells(expected)));
    }

    #[rstest::rstest]
    #[case::standalone_number(&['4'], "⠼⠙")]
    #[case::uppercase_letter(&['A'], "⠠⠁")]
    #[case::operator(&['+'], "⠐⠖")]
    fn encodes_matrix_cell_variants(#[case] input: &[char], #[case] expected: &str) {
        let mut out = Vec::new();
        encode_matrix_cell(&input.iter().collect::<String>(), &mut out).unwrap();
        assert_eq!(out, cells(expected));
    }

    #[test]
    fn rejects_unknown_technical_symbol() {
        assert_eq!(encode_technical(&"@".chars().collect::<Vec<_>>()), None);
    }

    #[test]
    fn helper_primitives_reject_invalid_inputs() {
        let mut out = Vec::new();
        push_cells(&mut out, "⠁⠃");
        assert_eq!(out, cells("⠁⠃"));

        assert_eq!(digit('x'), None);
        assert_eq!(encode_symbol('@', false), None);
    }

    #[rstest::rstest]
    #[case::bare_symbol(&['x'], false)]
    #[case::empty(&[], false)]
    #[case::numeric(&['1', '2', '.', '3'], false)]
    #[case::fraction(&['\\', 'f', 'r', 'a', 'c', '{', '1', '}', '{', '2', '}'], false)]
    #[case::radical(&['\\', 's', 'q', 'r', 't', '{', 'x', '}'], false)]
    #[case::paren_group(&['(', 'x', '+', '1', ')'], false)]
    #[case::mixed_item(&['2', 'y'], true)]
    fn item_grouping_paths(#[case] item: &[char], #[case] expected: bool) {
        assert_eq!(item_needs_braille_grouping(item), expected);
    }

    #[rstest::rstest]
    #[case::letter_fraction(&['\\', 'f', 'r', 'a', 'c', '{', 'x', '}', '{', 'y', '}'], true)]
    #[case::radical(&['\\', 's', 'q', 'r', 't', '{', 'x', '}'], true)]
    #[case::grouped_superscript(&['x', '^', '{', '2', 'y', '}'], true)]
    #[case::space_with_construct(&['x', ' ', '\\', 's', 'q', 'r', 't', '{', 'y', '}'], true)]
    #[case::plain_word(&['t', 'i', 'm', 'e'], false)]
    fn grade1_word_wrapper_paths(#[case] chars: &[char], #[case] expected: bool) {
        assert_eq!(needs_grade1_word_wrapper(chars), expected);
    }

    #[test]
    fn braced_parser_handles_nested_and_invalid_groups() {
        let chars: Vec<char> = "{a{b}c}".chars().collect();
        let (next, body) = take_braced(&chars, 0).unwrap();
        assert_eq!(next, chars.len());
        assert_eq!(body.iter().collect::<String>(), "a{b}c");
        assert_eq!(take_braced(&chars, 1), None);
        assert_eq!(take_braced(&"{abc".chars().collect::<Vec<_>>(), 0), None);
    }

    #[test]
    fn numeric_helpers_reject_invalid_digits() {
        let mut out = Vec::new();
        assert_eq!(encode_number_run(&['1', 'x'], &mut out), None);
        out.clear();
        assert_eq!(encode_denominator_item(&['🙂'], &mut out), None);
    }

    #[test]
    fn numeric_helpers_accept_comma_decimal_and_reject_non_fraction_prefixes() {
        let mut out = Vec::new();

        encode_number_run(&['1', ',', '2', '.', '3'], &mut out).unwrap();
        assert_eq!(out, cells("⠼⠁⠂⠃⠲⠉"));

        out.clear();
        assert_eq!(simple_fraction(&['x'], &mut out), None);
        assert_eq!(general_fraction(&['x'], &mut out, false), None);
    }

    #[test]
    fn grade1_construct_detects_grouped_script_item_directly() {
        assert!(has_grade1_construct(&"x_{2y}".chars().collect::<Vec<_>>()));
        assert!(!has_grade1_construct(&"x_2".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn grade1_construct_scans_script_groups_from_loop_start() {
        assert!(has_grade1_construct(&"_{ab}".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn grade1_construct_scans_later_grouped_script() {
        assert!(has_grade1_construct(
            &"x+ y^{ab}".chars().collect::<Vec<_>>()
        ));
        assert!(has_grade1_construct(&"x_{ab}".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn grade1_construct_detects_runtime_fraction_and_script() {
        let fraction: Vec<char> = std::hint::black_box("z+\\frac{x}{y}").chars().collect();
        let script: Vec<char> = std::hint::black_box("z+x^{2y}").chars().collect();

        assert!(has_grade1_construct(&fraction));
        assert!(has_grade1_construct(&script));
    }

    #[test]
    fn expression_runtime_radical_path_encodes_body() {
        let input: Vec<char> = std::hint::black_box("\\sqrt{x}").chars().collect();
        let mut out = Vec::new();

        encode_expr_with_options(&input, &mut out, false, false)
            .expect("radical expression should encode");

        assert_eq!(out, cells("⠰⠩⠭⠬"));
    }

    #[test]
    fn expression_options_cover_spaces_scripts_and_unknowns() {
        let mut out = Vec::new();

        encode_expr_with_options(&"x + y".chars().collect::<Vec<_>>(), &mut out, false, false)
            .unwrap();
        assert_eq!(out, cells("⠭⠐⠖⠽"));

        out.clear();
        encode_expr_with_options(&"x y".chars().collect::<Vec<_>>(), &mut out, false, false)
            .unwrap();
        assert_eq!(out, cells("⠭⠀⠽"));

        out.clear();
        encode_expr_with_options(&"x^2".chars().collect::<Vec<_>>(), &mut out, false, false)
            .unwrap();
        assert_eq!(out, cells("⠭⠰⠔⠼⠃"));

        out.clear();
        assert_eq!(
            encode_expr_with_options(&['@'], &mut out, false, false),
            None
        );
    }

    #[test]
    fn expression_options_cover_direct_radical_grouping_and_operator_variants() {
        let mut out = Vec::new();

        encode_expr_with_options(
            &"\\sqrt{x}".chars().collect::<Vec<_>>(),
            &mut out,
            false,
            false,
        )
        .expect("radical should encode");
        assert!(out.starts_with(&cells("⠰⠩")));

        out.clear();
        encode_expr_with_options(
            &"x_{2y}".chars().collect::<Vec<_>>(),
            &mut out,
            false,
            false,
        )
        .expect("grouped subscript should encode");
        assert!(out.contains(&decode_unicode('⠣')));

        for input in ["x>y", "x−y", "x×y"] {
            out.clear();
            encode_expr_with_options(&input.chars().collect::<Vec<_>>(), &mut out, false, false)
                .expect("operator variant should encode");
            assert!(!out.is_empty());
        }
    }

    #[test]
    fn grade1_construct_detects_grouped_subscript_item() {
        let subscript = std::hint::black_box("x_{ab}").chars().collect::<Vec<_>>();
        assert!(needs_grade1_word_wrapper(&subscript));

        let superscript = std::hint::black_box("x^{ab}").chars().collect::<Vec<_>>();
        assert!(needs_grade1_word_wrapper(&superscript));
    }

    #[test]
    fn grade1_construct_runtime_loop_reaches_grouping_check() {
        let mut chars = Vec::new();
        chars.extend(std::hint::black_box("x").chars());
        chars.push(std::hint::black_box('_'));
        chars.extend(std::hint::black_box("{ab}").chars());

        assert!(has_grade1_construct(&chars));
    }

    #[test]
    fn chemistry_paths_cover_spaces_ascii_digits_and_lowercase_letters() {
        assert_eq!(
            encode_chemical(&"H2O".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠰⠠⠓⠼⠃⠠⠕⠰⠄"))
        );
        assert_eq!(
            encode_chemical(&"H + O".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠰⠠⠓⠐⠖⠠⠕⠰⠄"))
        );
        assert_eq!(
            encode_chemical(&"h".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠰⠓⠰⠄"))
        );
        assert_eq!(
            encode_chemical(&"H O".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠰⠠⠓⠀⠠⠕⠰⠄"))
        );
        assert_eq!(encode_chemical(&"H@".chars().collect::<Vec<_>>()), None);
    }

    #[test]
    fn matrix_and_array_reject_reversed_layouts() {
        assert_eq!(
            encode_pmatrix(&"\\end{pmatrix}\\begin{pmatrix}".chars().collect::<Vec<_>>()),
            None
        );
        assert_eq!(
            encode_array_with_right_brace(
                &"\\end{array}\\left.\\begin{array}{l}\\right\\}"
                    .chars()
                    .collect::<Vec<_>>()
            ),
            None
        );
    }

    #[test]
    fn radical_index_and_ascii_expression_paths() {
        assert_eq!(
            encode_technical(&"\\sqrt[3]{x}".chars().collect::<Vec<_>>()),
            Some(cells("⠰⠰⠩⠔⠼⠉⠭⠬"))
        );
        assert_eq!(
            encode_technical(&"A_2".chars().collect::<Vec<_>>()),
            Some(cells("⠠⠁⠰⠢⠼⠃"))
        );
    }

    #[rstest::rstest]
    #[case::minus("1-2", "⠼⠁⠐⠤⠼⠃")]
    #[case::times_star("2*3", "⠼⠃⠐⠦⠼⠉")]
    #[case::slash("4/5", "⠼⠙⠐⠌⠼⠑")]
    #[case::equals("x=1", "⠭⠐⠶⠼⠁")]
    #[case::parens("(x)", "⠐⠣⠭⠐⠜")]
    #[case::semicolon("x;y", "⠭⠆⠽")]
    fn expression_operator_paths(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            encode_technical(&input.chars().collect::<Vec<_>>()),
            Some(cells(expected))
        );
    }

    #[rstest::rstest]
    #[case::left_arrow('→', "⠰⠳⠕")]
    #[case::down_arrow('↓', "⠰⠳⠩")]
    #[case::circle('◯', "⠫⠿")]
    fn symbol_paths(#[case] c: char, #[case] expected: &str) {
        assert_eq!(encode_symbol(c, false), Some(cells(expected)));
    }

    #[test]
    fn technical_layout_rejects_malformed_matrix_and_array() {
        assert_eq!(
            encode_pmatrix(&"\\end{pmatrix}\\begin{pmatrix}".chars().collect::<Vec<_>>()),
            None
        );
        assert_eq!(
            encode_array_with_right_brace(
                &"\\left.\\begin{array}{l}plain\\end{array}\\right\\}"
                    .chars()
                    .collect::<Vec<_>>()
            ),
            None
        );
    }
}
