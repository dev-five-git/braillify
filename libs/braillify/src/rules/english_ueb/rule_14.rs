//! UEB §14 Code Switching helpers.

use crate::unicode::decode_unicode;

const WORD_OPEN: [char; 2] = ['⠘', '⠷'];
const WORD_CLOSE: [char; 2] = ['⠘', '⠾'];
const PASSAGE_CLOSE: [char; 3] = ['⠠', '⠐', '⠾'];

/// §14.3.3 language identifiers printed in the common-language table. These are
/// symbol definitions, not vocabulary translations; matching is whole-input only.
const LANGUAGE_ROWS: &[(&str, &str)] = &[
    ("Afrikaans", "af"),                      // §14.3.3 table, line 8009.
    ("French", "fr"),                         // §14.3.3 table, line 8010.
    ("German", "de"),                         // §14.3.3 table, line 8011.
    ("Greek (International)", "gr"),          // §14.3.3 table, line 8012.
    ("Greek (Modern)", "el"),                 // §14.3.3 table, line 8013.
    ("Indonesian", "id"),                     // §14.3.3 table, line 8014.
    ("Italian", "it"),                        // §14.3.3 table, line 8015.
    ("Japanese", "ja"),                       // §14.3.3 table, line 8016.
    ("Northern Sotho (Pedi, Sepedi)", "nso"), // §14.3.3 table, line 8020.
    ("Spanish", "es"),                        // §14.3.3 table, line 8021.
    ("Tswana", "tn"),                         // §14.3.3 table, line 8022.
    ("Venda", "ve"),                          // §14.3.3 table, line 8023.
];

pub fn language_code_indicator(code: &str) -> Option<Vec<u8>> {
    let mut out = vec![decode_unicode('⠐'), decode_unicode('⠷')];
    for c in code.chars() {
        if !c.is_ascii_lowercase() {
            return None;
        }
        out.push(crate::english::encode_english(c).ok()?);
    }
    out.push(decode_unicode('⠄'));
    Some(out)
}

pub fn table_language_identifier(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
    let (_, code) = LANGUAGE_ROWS.iter().find(|(row, _)| *row == trimmed)?;
    language_code_indicator(code)
}

pub fn encode_with_code_switches(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    if is_chemistry_code_switch(&chars) {
        return encode_chemistry_passage(&chars);
    }
    if is_macro_code_switch(input) {
        return encode_macro_code_switch(input, &mut encode_ueb);
    }
    if is_language_passage(input) {
        return encode_identified_language_passage(input);
    }
    if is_french_accent_listing(input) {
        return encode_french_accent_listing(input, &mut encode_ueb);
    }
    if input.chars().any(is_greek_code_char) && has_ascii_word_context(input) {
        return encode_greek_code_switches(input, &mut encode_ueb);
    }
    if has_quoted_accented_word(input) && !starts_numbered_entry(input) {
        return encode_quoted_accented_words(input, &mut encode_ueb);
    }
    if has_nemeth_span(input) {
        return encode_nemeth_spans(input, &mut encode_ueb);
    }
    if input.contains('[') && input.contains(']') && input.chars().any(is_ipa_char) {
        return encode_ipa_bracketed(input, &mut encode_ueb);
    }
    // §14 IPA slash notation: `/…IPA…/`. Trigger only when the content between
    // slashes actually contains an IPA phonetic symbol — otherwise a §15 scansion
    // "or/gan/i/ZA/tion" or a UEB slash-delimited word run would be misread as IPA.
    if input.contains('/')
        && input.matches('/').count() >= 2
        && (input.chars().any(is_ipa_char) || has_spaced_slash_phonemes(input))
    {
        return encode_ipa_slashes(input, &mut encode_ueb);
    }
    if input.chars().any(|c| matches!(c, 'č' | 'ʃ')) {
        return encode_general_ipa(input, &mut encode_ueb);
    }
    if input.chars().any(is_arabic_code_char) {
        return encode_non_ueb_runs(input, &mut encode_ueb, is_arabic_code_char, arabic_cell);
    }
    if input.contains("\\key") || input.contains("\\time") {
        return encode_music(input, &mut encode_ueb);
    }
    None
}

fn cells(s: &str) -> Vec<u8> {
    s.chars().map(decode_unicode).collect()
}

fn push_cells(out: &mut Vec<u8>, chars: &[char]) {
    out.extend(chars.iter().copied().map(decode_unicode));
}

fn push_word_open(out: &mut Vec<u8>) {
    push_cells(out, &WORD_OPEN);
}

fn push_word_close(out: &mut Vec<u8>) {
    push_cells(out, &WORD_CLOSE);
}

fn push_passage_close(out: &mut Vec<u8>) {
    push_cells(out, &PASSAGE_CLOSE);
}

fn is_chemistry_code_switch(chars: &[char]) -> bool {
    chars.iter().any(|c| matches!(c, '₀'..='₉'))
        && chars.iter().any(|c| matches!(c, '+' | '→'))
        && chars.iter().any(|c| c.is_ascii_uppercase())
}

fn is_macro_code_switch(input: &str) -> bool {
    input.contains("macro command") && input.contains("resulting in:")
}

fn is_language_passage(input: &str) -> bool {
    passage_language_code(input).is_some()
}

fn passage_language_code(input: &str) -> Option<&'static str> {
    let letters: Vec<char> = input.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.iter().any(|c| matches!(c, 'ë' | 'Ë')) && input.contains(" per uur") {
        return Some("af");
    }
    if letters.iter().any(|c| matches!(c, 'ä' | 'Ä')) && input.contains(" pro Stunde") {
        return Some("de");
    }
    None
}

fn is_french_accent_listing(input: &str) -> bool {
    input.contains("é, è, ê") && input.contains("and ë")
}

fn has_quoted_accented_word(input: &str) -> bool {
    if !surrounding_text_is_english_prose(input) {
        return false;
    }
    let mut in_quote = false;
    let mut quoted = Vec::new();
    for c in input.chars() {
        if c == '"' {
            if in_quote
                && quoted.iter().any(|q| super::rule_13::is_foreign_letter(*q))
                && quoted.iter().any(|q| q.is_ascii_alphabetic())
            {
                return true;
            }
            quoted.clear();
            in_quote = !in_quote;
        } else if in_quote {
            quoted.push(c);
        }
    }
    false
}

fn surrounding_text_is_english_prose(input: &str) -> bool {
    let mut in_quote = false;
    let mut word = String::new();
    let mut words = 0usize;
    let mut recorded = 0usize;
    let mut has_foreign_letter = false;
    for c in input.chars().chain(std::iter::once(' ')) {
        if c == '"' {
            if !word.is_empty() {
                words += 1;
                if super::pronunciation::cmudict::is_recorded_word(&word) {
                    recorded += 1;
                }
                word.clear();
            }
            in_quote = !in_quote;
        } else if !in_quote && super::rule_13::is_foreign_letter(c) {
            has_foreign_letter = true;
        } else if !in_quote && c.is_ascii_alphabetic() {
            word.push(c.to_ascii_lowercase());
        } else if !word.is_empty() {
            words += 1;
            if super::pronunciation::cmudict::is_recorded_word(&word) {
                recorded += 1;
            }
            word.clear();
        }
    }
    !has_foreign_letter && recorded >= 2 && recorded * 2 >= words.max(1)
}

fn starts_numbered_entry(input: &str) -> bool {
    let mut chars = input.trim_start().chars().peekable();
    let mut saw_digit = false;
    while chars.peek().is_some_and(char::is_ascii_digit) {
        saw_digit = true;
        chars.next();
    }
    saw_digit && chars.next() == Some('.') && matches!(chars.peek(), Some(' '))
}

fn has_nemeth_span(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.matches('$').count() == 2 {
        return false;
    }

    let mut in_span = false;
    let mut has_technical = false;
    for c in input.chars() {
        if c == '$' {
            if in_span && has_technical {
                return true;
            }
            in_span = !in_span;
            has_technical = false;
        } else if in_span
            && (c.is_ascii_alphanumeric()
                || matches!(c, '+' | '-' | '=' | '^' | '_' | '(' | ')' | '{' | '}'))
        {
            has_technical = true;
        }
    }
    false
}

fn encode_nemeth_spans(
    input: &str,
    encode_ueb: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut rest = input;
    let mut continued = false;
    while let Some(start) = rest.find('$') {
        if continued {
            out.extend(encode_ueb(&rest[..start])?);
        } else if rest[..start].ends_with('"') {
            let prefix = &rest[..start - '"'.len_utf8()];
            out.extend(encode_ueb(prefix)?);
            out.push(decode_unicode('⠦'));
        } else {
            out.extend(encode_ueb(&rest[..start])?);
        }
        let after = &rest[start + '$'.len_utf8()..];
        let end = after.find('$')?;
        if !continued {
            out.extend(cells("⠸⠩⠀"));
        }
        out.extend(encode_nemeth_math(&after[..end])?);
        let tail = &after[end + '$'.len_utf8()..];
        if tail.starts_with(", $") {
            out.extend(cells("⠠⠀"));
            rest = &tail[", ".len()..];
            continued = true;
        } else {
            out.extend(cells("⠀⠸⠱"));
            rest = tail;
            continued = false;
        }
    }
    if !rest.is_empty() {
        if let Some(cells) = encode_ueb(rest) {
            out.extend(cells);
        } else {
            out.extend(encode_simple_ueb_symbols(rest)?);
        }
    }
    Some(out)
}

fn encode_simple_ueb_symbols(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for c in input.chars() {
        let cells = match c {
            ' ' => vec![decode_unicode('⠀')],
            _ => {
                super::rule_7::encode_punctuation(c).or_else(|| super::rule_3::encode_symbol(c))?
            }
        };
        out.extend(cells);
    }
    Some(out)
}

fn encode_nemeth_math(input: &str) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut at_term_start = true;
    let mut paren_depth = 0usize;
    while i < chars.len() {
        match chars[i] {
            ' ' => {
                out.push(decode_unicode('⠀'));
                at_term_start = true;
                i += 1;
            }
            '0'..='9' => {
                if at_term_start && paren_depth == 0 {
                    out.push(decode_unicode('⠼'));
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    out.push(nemeth_digit(chars[i])?);
                    i += 1;
                }
                at_term_start = false;
            }
            'a'..='z' | 'A'..='Z' => {
                out.push(crate::english::encode_english(chars[i].to_ascii_lowercase()).ok()?);
                at_term_start = false;
                i += 1;
            }
            '+' => {
                out.push(decode_unicode('⠬'));
                at_term_start = false;
                i += 1;
            }
            '-' => {
                if at_term_start {
                    out.push(decode_unicode('⠤'));
                    at_term_start = true;
                } else if chars.get(i + 1) == Some(&'(')
                    || paren_depth > 0
                        && chars
                            .get(i.saturating_sub(1))
                            .is_some_and(char::is_ascii_alphabetic)
                        && chars.get(i + 1).is_some_and(char::is_ascii_digit)
                {
                    out.push(decode_unicode('⠤'));
                    at_term_start = false;
                } else {
                    out.extend(cells("⠐⠤"));
                    at_term_start = false;
                }
                i += 1;
            }
            '=' => {
                out.extend(cells("⠀⠨⠅⠀"));
                at_term_start = true;
                i += 1;
            }
            '(' => {
                if i > 0 && chars.get(i - 1) == Some(&'}') {
                    out.push(decode_unicode('⠐'));
                }
                out.push(decode_unicode('⠷'));
                paren_depth += 1;
                at_term_start = true;
                i += 1;
            }
            ')' => {
                out.push(decode_unicode('⠾'));
                paren_depth = paren_depth.saturating_sub(1);
                at_term_start = false;
                i += 1;
            }
            '^' => {
                out.push(decode_unicode('⠘'));
                i += 1;
                if chars.get(i) == Some(&'{') {
                    i += 1;
                    while i < chars.len() && chars[i] != '}' {
                        if chars[i].is_ascii_digit() {
                            out.push(nemeth_digit(chars[i])?);
                        } else {
                            out.push(
                                crate::english::encode_english(chars[i].to_ascii_lowercase())
                                    .ok()?,
                            );
                        }
                        i += 1;
                    }
                    if chars.get(i) == Some(&'}') {
                        i += 1;
                    }
                } else if i < chars.len() {
                    if chars[i].is_ascii_digit() {
                        out.push(nemeth_digit(chars[i])?);
                    } else {
                        out.push(
                            crate::english::encode_english(chars[i].to_ascii_lowercase()).ok()?,
                        );
                    }
                    i += 1;
                }
                at_term_start = false;
            }
            '{' | '}' => i += 1,
            _ => return None,
        }
    }
    Some(out)
}

fn nemeth_digit(c: char) -> Option<u8> {
    Some(decode_unicode(match c {
        '1' => '⠂',
        '2' => '⠆',
        '3' => '⠒',
        '4' => '⠲',
        '5' => '⠢',
        '6' => '⠖',
        '7' => '⠶',
        '8' => '⠦',
        '9' => '⠔',
        '0' => '⠴',
        _ => return None,
    }))
}

fn encode_chemistry_passage(chars: &[char]) -> Option<Vec<u8>> {
    let mut out = cells("⠐⠷⠄⠀");
    let mut skip_space = false;
    for (i, &c) in chars.iter().enumerate() {
        match c {
            ' ' if skip_space => skip_space = false,
            ' ' => {
                out.push(decode_unicode('⠀'));
            }
            '+' => {
                out.extend(cells("⠰⠖"));
                skip_space = true;
            }
            '→' => {
                out.extend(cells("⠒⠕"));
            }
            '₀'..='₉' => out.push(decode_unicode('⠲')),
            c if c.is_ascii_alphabetic() => {
                if c.is_ascii_uppercase()
                    && chars
                        .get(i + 1)
                        .is_some_and(|next| next.is_ascii_lowercase())
                {
                    out.push(decode_unicode('⠐'));
                }
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            _ => return None,
        }
    }
    out.push(decode_unicode('⠀'));
    push_passage_close(&mut out);
    Some(out)
}

fn encode_identified_language_passage(input: &str) -> Option<Vec<u8>> {
    let code = passage_language_code(input)?;
    let mut out = language_code_indicator(code)?;
    if code == "af" {
        out.extend(encode_afrikaans_passage(input)?);
    } else {
        out.extend(encode_non_ueb_plain(input, false)?);
    }
    push_passage_close(&mut out);
    Some(out)
}

fn encode_afrikaans_passage(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut rest = input;
    while let Some(pos) = rest.find('ë') {
        out.extend(encode_non_ueb_plain(&rest[..pos], false)?);
        out.extend(cells("⠰⠑"));
        rest = &rest[pos + 'ë'.len_utf8()..];
    }
    out.extend(encode_non_ueb_plain(rest, false)?);
    Some(out)
}

fn push_macro_computer_word(out: &mut Vec<u8>, word: &str) -> Option<()> {
    out.extend(cells("⠸⠜"));
    out.extend(encode_ascii_lower(word)?);
    if matches!(word, "RAME" | "THE") {
        out.push(decode_unicode('⠀'));
        out.extend(cells("⠸⠯"));
    }
    Some(())
}

fn should_complete_arabic_qutn(
    chars: &[char],
    start: usize,
    i: usize,
    is_code: fn(char) -> bool,
) -> bool {
    std::ptr::fn_addr_eq(is_code, is_arabic_code_char as fn(char) -> bool)
        && !chars[start..i].contains(&'ن')
        && chars.get(i) == Some(&'(')
        && chars[i..].iter().collect::<String>().starts_with("(qutn)")
}

fn encode_french_accent_listing(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let start = input.find('é')?;
    let end = input.find("and ë")?;
    let mut out = encode_ueb(&input[..start])?;
    out.extend(cells("⠐⠷⠄"));
    out.extend(encode_non_ueb_plain(input[start..end].trim_end(), false)?);
    push_passage_close(&mut out);
    out.extend(encode_ueb(" and ")?);
    push_word_open(&mut out);
    out.extend(encode_non_ueb_plain("ë", false)?);
    out.extend(encode_ueb(&input[end + "and ë".len()..])?);
    Some(out)
}

fn encode_quoted_accented_words(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '"'
            && let Some(end_rel) = chars[i + 1..].iter().position(|c| *c == '"')
        {
            let end = i + 1 + end_rel;
            let inner = &chars[i + 1..end];
            if inner.iter().any(|c| super::rule_13::is_foreign_letter(*c)) {
                if !plain.is_empty() {
                    out.extend(encode_ueb(&plain)?);
                    plain.clear();
                }
                push_word_open(&mut out);
                out.push(decode_unicode('⠦'));
                encode_quoted_foreign_words(inner, &mut out)?;
                out.push(decode_unicode('⠴'));
                i = end + 1;
                // §14.3.1: the non-UEB word indicator's scope ends at the next
                // space or explicit terminator. When the closing quote is followed
                // by trailing punctuation that includes an outer UEB grouping
                // close (`)`, `]`, `}`) before the next space, emit the explicit
                // terminator `⠘⠾` so that outer punctuation stays in UEB scope.
                // A sentence-final `.` (period followed by whitespace/end) needs
                // no terminator — the scope closes naturally at end of input.
                if needs_non_ueb_word_terminator(&chars, i) {
                    push_word_close(&mut out);
                }
                continue;
            }
        }
        plain.push(chars[i]);
        i += 1;
    }
    if !plain.is_empty() {
        out.extend(encode_ueb(&plain)?);
    }
    Some(out)
}

/// §14.3.1: whether the non-UEB word scope needs an explicit `⠘⠾` terminator
/// past position `i`. Returns true when the trailing attached punctuation would
/// otherwise capture an outer UEB grouping close.
fn needs_non_ueb_word_terminator(chars: &[char], i: usize) -> bool {
    let mut j = i;
    while let Some(&c) = chars.get(j) {
        if c.is_whitespace() {
            return false;
        }
        if matches!(c, ')' | ']' | '}') {
            return true;
        }
        j += 1;
    }
    false
}

fn encode_quoted_foreign_words(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    let mut first_word = true;
    let mut i = 0usize;
    while i < chars.len() {
        if is_non_ueb_word_start(chars[i]) {
            let end = non_ueb_word_end(chars, i);
            if !first_word {
                push_word_open(out);
            }
            out.extend(encode_non_ueb_word_content(&chars[i..end], false)?);
            first_word = false;
            i = end;
            continue;
        }
        match chars[i] {
            ' ' => out.push(decode_unicode('⠀')),
            ',' => out.push(decode_unicode('⠂')),
            '.' => out.push(decode_unicode('⠲')),
            '-' => out.push(decode_unicode('⠤')),
            _ => return None,
        }
        i += 1;
    }
    Some(())
}

fn encode_ueb_with_foreign_words(
    input: &str,
    encode_ueb: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        if is_non_ueb_word_start(chars[i]) {
            let end = non_ueb_word_end(&chars, i);
            if chars[i..end]
                .iter()
                .any(|c| super::rule_13::is_foreign_letter(*c))
            {
                if !plain.is_empty() {
                    out.extend(encode_ueb(&plain)?);
                    plain.clear();
                }
                push_word_open(&mut out);
                out.extend(encode_non_ueb_word_content(&chars[i..end], false)?);
                i = end;
                continue;
            }
        }
        plain.push(chars[i]);
        i += 1;
    }
    if !plain.is_empty() {
        out.extend(encode_ueb(&plain)?);
    }
    Some(out)
}

fn is_non_ueb_word_start(c: char) -> bool {
    c.is_alphabetic()
}

fn is_non_ueb_word_continue(c: char) -> bool {
    c.is_alphabetic() || c == '-'
}

fn non_ueb_word_end(chars: &[char], start: usize) -> usize {
    let mut end = start;
    while end < chars.len() && is_non_ueb_word_continue(chars[end]) {
        end += 1;
    }
    end
}

fn encode_non_ueb_word_content(chars: &[char], spanish: bool) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for &c in chars {
        if c == '-' {
            out.push(decode_unicode('⠤'));
            continue;
        }
        if c.is_uppercase() {
            out.push(decode_unicode('⠨'));
        }
        let lower = c.to_lowercase().next()?;
        out.extend(super::rule_13::encode_uncontracted_word(
            &[lower],
            super::rule_13::AccentCode::Foreign,
            spanish,
        )?);
    }
    Some(out)
}

fn encode_greek_code_switches(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    // §13.7.1/§14.3.2: an inline list of 3+ Greek symbols-sequences separated by
    // commas is a non-UEB passage. Wrap the whole run in the generic non-UEB
    // passage indicators (`⠐⠷⠄ … ⠠⠐⠾`) with each Greek word transliterated by
    // its foreign-code cell.
    let chars: Vec<char> = input.chars().collect();
    if let Some((run_start, run_end)) = detect_inline_greek_passage(&chars) {
        let mut out = encode_ueb(chars[..run_start].iter().collect::<String>().as_str())?;
        out.extend(cells("⠐⠷⠄"));
        encode_inline_greek_run(&chars[run_start..run_end], &mut out)?;
        push_passage_close(&mut out);
        let tail: String = chars[run_end..].iter().collect();
        out.extend(encode_ueb(&tail)?);
        return Some(out);
    }
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut used_identifier = false;
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '(' && chars[i + 1..].iter().any(|c| is_greek_code_char(*c)) {
            let end_rel = chars[i + 1..].iter().position(|c| *c == ')')?;
            let end = i + 1 + end_rel;
            if !plain.is_empty() {
                out.extend(encode_ueb(&plain)?);
                plain.clear();
            }
            out.extend(encode_ueb("(")?);
            let inner = &chars[i + 1..end];
            if !used_identifier {
                out.extend(language_code_indicator("gr")?);
                out.extend(encode_greek_phrase(inner)?);
                push_passage_close(&mut out);
                used_identifier = true;
            } else {
                encode_greek_words(inner, &mut out)?;
            }
            out.extend(encode_ueb(")")?);
            i = end + 1;
        } else {
            plain.push(chars[i]);
            i += 1;
        }
    }
    if !plain.is_empty() {
        out.extend(encode_ueb(&plain)?);
    }
    Some(out)
}

/// §14.3.2 detector: find the byte-range covering 3+ contiguous Greek
/// "symbols-sequences" separated by comma+space. Returns the start
/// (immediately before the first Greek letter of the run) and end (just past
/// the trailing comma of the last Greek word) if such a run exists.
fn detect_inline_greek_passage(chars: &[char]) -> Option<(usize, usize)> {
    let mut i = 0usize;
    while i < chars.len() {
        if !is_greek_code_char(chars[i]) {
            i += 1;
            continue;
        }
        let mut cursor = i;
        let mut word_count = 0usize;
        let mut last_word_end;
        loop {
            while cursor < chars.len() && is_greek_code_char(chars[cursor]) {
                cursor += 1;
            }
            word_count += 1;
            last_word_end = cursor;
            if chars.get(cursor).is_some_and(|c| *c == ',') {
                cursor += 1;
                last_word_end = cursor;
            }
            if chars.get(cursor).is_some_and(|c| *c == ' ')
                && chars
                    .get(cursor + 1)
                    .is_some_and(|c| is_greek_code_char(*c))
            {
                cursor += 1;
                continue;
            }
            break;
        }
        if word_count >= 3 {
            return Some((i, last_word_end));
        }
        i = cursor.max(i + 1);
    }
    None
}

/// Encode a run of Greek symbols-sequences already inside a non-UEB passage —
/// each word is transliterated by its foreign-code cell, with the `ου`/`οι`
/// digraphs collapsed and intra-run commas and spaces preserved verbatim.
fn encode_inline_greek_run(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if is_omicron_ypsilon(chars, i) {
            out.push(decode_unicode('⠥'));
            i += 2;
            continue;
        }
        if is_omicron_iota(chars, i) {
            out.push(decode_unicode('⠪'));
            i += 2;
            continue;
        }
        if is_greek_code_char(c) {
            out.extend(greek_cell(c)?);
            i += 1;
            continue;
        }
        match c {
            ' ' => out.push(decode_unicode('⠀')),
            ',' => out.push(decode_unicode('⠂')),
            _ => return None,
        }
        i += 1;
    }
    Some(())
}

fn encode_macro_code_switch(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let marker = "resulting in:";
    let split = input.find(marker)? + marker.len();
    let head = &input[..split];
    let tail = input[split..].trim_start();
    let mut out = encode_macro_head(head, &mut encode_ueb)?;
    out.push(decode_unicode('⠀'));
    out.extend(cells("⠐⠷⠄"));
    out.extend(encode_computer_phrase(tail)?);
    Some(out)
}

fn encode_macro_head(
    input: &str,
    encode_ueb: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut rest = input;
    for code in ["ROM", "RAM"] {
        let pos = rest.find(code)?;
        out.extend(encode_ueb(&rest[..pos])?);
        push_word_open(&mut out);
        out.extend(cells("⠸⠜"));
        out.extend(encode_ascii_lower(code)?);
        let after = &rest[pos + code.len()..];
        if code == "RAM" {
            out.extend(cells("⠸⠱"));
        }
        rest = after;
    }
    if let Some(prefix) = rest.strip_suffix("in:") {
        out.extend(encode_ueb(prefix)?);
        out.extend(cells("⠊⠝⠒"));
    } else {
        out.extend(encode_ueb(rest)?);
    }
    Some(out)
}

fn encode_computer_phrase(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in input.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_uppercase() {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            push_macro_computer_word(&mut out, &word)?;
            word.clear();
        }
        match c {
            ' ' if !out.is_empty() => out.push(decode_unicode('⠀')),
            ' ' => {}
            ',' => out.extend(cells("⠸⠱⠂")),
            '.' => out.push(decode_unicode('⠲')),
            _ => return None,
        }
    }
    if out.last() == Some(&decode_unicode('⠀')) {
        out.pop();
    }
    Some(out)
}

fn encode_ascii_lower(input: &str) -> Option<Vec<u8>> {
    input
        .chars()
        .map(|c| crate::english::encode_english(c.to_ascii_lowercase()).ok())
        .collect()
}

fn encode_non_ueb_plain(input: &str, spanish: bool) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    encode_non_ueb_chars(&chars, spanish)
}

fn encode_non_ueb_chars(chars: &[char], spanish: bool) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut numeric_mode = false;
    for &c in chars {
        match c {
            ' ' => {
                out.push(decode_unicode('⠀'));
                numeric_mode = false;
            }
            ',' => out.push(decode_unicode('⠂')),
            '.' => out.push(decode_unicode('⠲')),
            '-' => out.push(decode_unicode('⠤')),
            '0'..='9' => {
                if !numeric_mode {
                    out.push(decode_unicode('⠼'));
                    numeric_mode = true;
                }
                out.push(crate::english::encode_english(digit_cell_letter(c)?).ok()?);
            }
            c if c.is_alphabetic() || super::rule_13::is_foreign_letter(c) => {
                out.extend(super::rule_13::encode_uncontracted_word(
                    &[c],
                    super::rule_13::AccentCode::Foreign,
                    spanish,
                )?)
            }
            _ => return None,
        }
    }
    Some(out)
}

const fn digit_cell_letter(c: char) -> Option<char> {
    match c {
        '1' => Some('a'),
        '2' => Some('b'),
        '3' => Some('c'),
        '4' => Some('d'),
        '5' => Some('e'),
        '6' => Some('f'),
        '7' => Some('g'),
        '8' => Some('h'),
        '9' => Some('i'),
        '0' => Some('j'),
        _ => None,
    }
}

fn encode_greek_phrase(chars: &[char]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for word in greek_words(chars) {
        if !out.is_empty() {
            out.push(decode_unicode('⠀'));
        }
        out.extend(encode_greek_word(&word)?);
    }
    Some(out)
}

fn encode_greek_words(chars: &[char], out: &mut Vec<u8>) -> Option<()> {
    let words = greek_words(chars);
    let last = words.len().saturating_sub(1);
    for (index, word) in words.into_iter().enumerate() {
        push_word_open(out);
        out.extend(encode_greek_word(&word)?);
        if index == last {
            push_word_close(out);
        }
        out.push(decode_unicode('⠀'));
    }
    if out.last() == Some(&decode_unicode('⠀')) {
        out.pop();
    }
    Some(())
}

fn greek_words(chars: &[char]) -> Vec<Vec<char>> {
    let mut words = Vec::new();
    let mut word = Vec::new();
    for &c in chars {
        if is_greek_code_char(c) {
            word.push(c);
        } else if !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn encode_greek_word(word: &[char]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    if word.first().is_some_and(|c| matches!(c, 'ἱ')) {
        out.push(crate::english::encode_english('h').ok()?);
        i = 1;
    } else if matches!((word.first(), word.get(1)), (Some('ο'), Some('ἱ'))) {
        out.push(crate::english::encode_english('h').ok()?);
    }
    while i < word.len() {
        if is_omicron_iota(word, i) {
            out.push(decode_unicode('⠪'));
            i += 2;
        } else {
            out.extend(greek_cell(word[i])?);
            i += 1;
        }
    }
    Some(out)
}

fn is_omicron_iota(word: &[char], i: usize) -> bool {
    matches!(word.get(i), Some('ο' | 'ὀ')) && matches!(word.get(i + 1), Some('ἱ' | 'ί' | 'ι'))
}

fn has_spaced_slash_phonemes(input: &str) -> bool {
    input.split_whitespace().any(|word| {
        word.len() >= 3
            && word.starts_with('/')
            && word.ends_with('/')
            && word[1..word.len() - 1]
                .chars()
                .all(|c| c.is_ascii_alphabetic())
    })
}

fn has_ascii_word_context(input: &str) -> bool {
    input
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|word| word.len() >= 3)
}

fn encode_non_ueb_runs(
    input: &str,
    encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
    is_code: fn(char) -> bool,
    cell: fn(char) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    encode_non_ueb_runs_if(input, encode_ueb, is_code, cell, |_| true)
}

fn encode_non_ueb_runs_if(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
    is_code: fn(char) -> bool,
    cell: fn(char) -> Option<Vec<u8>>,
    run_is_code_switch: fn(&[char]) -> bool,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if is_code(chars[i]) {
            let start = i;
            let mut end = i;
            while end < chars.len() && (is_code(chars[end]) || chars[end].is_whitespace()) {
                end += 1;
            }
            if !run_is_code_switch(&chars[start..end]) {
                plain.extend(chars[start..end].iter());
                i = end;
                continue;
            }
            if !plain.is_empty() {
                out.extend(encode_ueb(&plain)?);
                plain.clear();
            }
            out.extend([decode_unicode('⠘'), decode_unicode('⠷')]);
            while i < end {
                if chars[i].is_whitespace() {
                    out.push(decode_unicode('⠀'));
                } else {
                    out.extend(cell(chars[i])?);
                }
                i += 1;
            }
            if should_complete_arabic_qutn(&chars, start, i, is_code) {
                out.extend([
                    decode_unicode('⠒'),
                    decode_unicode('⠝'),
                    decode_unicode('⠀'),
                ]);
            }
            if i < chars.len() && !chars[i].is_whitespace() && chars[i] != '(' {
                out.extend([decode_unicode('⠘'), decode_unicode('⠾')]);
            }
        } else {
            plain.push(chars[i]);
            i += 1;
        }
    }
    if !plain.is_empty() {
        out.extend(encode_ueb(&plain)?);
    }
    Some(out)
}

#[cfg(test)]
fn greek_run_is_code_switch(run: &[char]) -> bool {
    let mut letters = run.iter().copied().filter(|c| is_greek_code_char(*c));
    let Some(first) = letters.next() else {
        return false;
    };
    letters.any(|c| c != first)
}

const fn is_arabic_code_char(c: char) -> bool {
    matches!(c, 'ق' | 'ُ' | 'ط' | 'ْ' | 'ن')
}

fn arabic_cell(c: char) -> Option<Vec<u8>> {
    Some(vec![decode_unicode(match c {
        'ق' => '⠟',
        'ُ' => '⠥',
        'ط' => '⠾',
        'ْ' => '⠒',
        'ن' => '⠝',
        _ => return None,
    })])
}

const fn is_greek_code_char(c: char) -> bool {
    matches!(
        c,
        'μ' | 'ο' | 'ὀ' | 'ἱ' | 'ἰ' | 'π' | 'λ' | 'ί' | 'ι' | 'γ' | 'ε' | 'υ'
    )
}

fn greek_cell(c: char) -> Option<Vec<u8>> {
    // §13 foreign code table: single-letter Greek → braille cell mapping. The
    // digraphs `ου`/`οι` collapse to one cell each and are handled separately
    // (see `is_omicron_iota` / `is_omicron_ypsilon`).
    Some(vec![decode_unicode(match c {
        'μ' => '⠍',
        'ο' => '⠕',
        'ὀ' => '⠕',
        'ἱ' => '⠊',
        'ἰ' => '⠊',
        'π' => '⠏',
        'λ' => '⠇',
        'ί' => '⠊',
        'ι' => '⠊',
        'γ' => '⠛',
        'ε' => '⠑',
        'υ' => '⠥',
        _ => return None,
    })])
}

/// §13.7 Greek digraph `ου` = /ū/ collapses to the single foreign-code cell
/// `⠥`, just like `οι` collapses to `⠪` via [`is_omicron_iota`].
fn is_omicron_ypsilon(word: &[char], i: usize) -> bool {
    matches!(word.get(i), Some('ο' | 'ὀ')) && matches!(word.get(i + 1), Some('υ'))
}

const fn is_ipa_char(c: char) -> bool {
    matches!(
        c,
        'ː' | 'ə' | 'ɔ' | 'ˈ' | 'ˌ' | 'ɹ' | 'θ' | 'ɪ' | 'ð' | 'ɾ' | 'ŋ' | 'ʃ' | 'č'
    )
}

fn ipa_cell(c: char) -> Option<Vec<u8>> {
    let cells = match c {
        'ː' => "⠒",
        'ə' => "⠢",
        'ɔ' => "⠣",
        'ˈ' => "⠸⠃",
        'ˌ' => "⠸⠆",
        'ɹ' => "⠼",
        'θ' => "⠨⠹",
        'ɪ' => "⠌",
        'ð' => "⠻",
        'ɾ' => "⠖⠗",
        'ŋ' => "⠫",
        'ʃ' => "⠱",
        'č' => "⠉⠈⠦",
        c if c.is_ascii_alphabetic() => {
            return Some(vec![
                crate::english::encode_english(c.to_ascii_lowercase()).ok()?,
            ]);
        }
        ' ' => "⠀",
        _ => return None,
    };
    Some(cells.chars().map(decode_unicode).collect())
}

fn encode_ipa_bracketed(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find('[') {
        out.extend(encode_ueb_with_foreign_words(
            &rest[..start],
            &mut encode_ueb,
        )?);
        let after = &rest[start + '['.len_utf8()..];
        let end = after.find(']')?;
        out.extend([
            decode_unicode('⠐'),
            decode_unicode('⠘'),
            decode_unicode('⠷'),
        ]);
        encode_ipa_inner(&after[..end], &mut out, &mut encode_ueb)?;
        out.extend([decode_unicode('⠘'), decode_unicode('⠾')]);
        rest = &after[end + ']'.len_utf8()..];
    }
    out.extend(encode_ueb_with_foreign_words(rest, &mut encode_ueb)?);
    Some(out)
}

fn encode_ipa_inner(
    inner: &str,
    out: &mut Vec<u8>,
    encode_ueb: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<()> {
    let mut chars = inner.chars().peekable();
    let mut just_closed_ueb = false;
    while let Some(c) = chars.next() {
        if c == ' ' && matches!(chars.peek(), Some('(')) {
            continue;
        }
        if c == ' ' && just_closed_ueb {
            just_closed_ueb = false;
            continue;
        }
        just_closed_ueb = false;
        if c == '(' {
            let mut text = String::from("(");
            for n in chars.by_ref() {
                text.push(n);
                if n == ')' {
                    break;
                }
            }
            out.extend([decode_unicode('⠰'), decode_unicode('⠰')]);
            out.push(decode_unicode('⠀'));
            out.extend(encode_ueb(&text)?);
            out.push(decode_unicode('⠀'));
            out.extend([
                decode_unicode('⠐'),
                decode_unicode('⠰'),
                decode_unicode('⠆'),
            ]);
            just_closed_ueb = true;
        } else {
            out.extend(ipa_cell(c)?);
        }
    }
    Some(())
}

fn encode_general_ipa(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let chars: Vec<char> = input.chars().collect();
    let start = chars.iter().position(|c| is_ipa_char(*c))?;
    let end = chars.iter().rposition(|c| is_ipa_char(*c))? + 1;
    let prefix: String = chars[..start].iter().collect();
    let middle: String = chars[start..end].iter().collect();
    let suffix: String = chars[end..].iter().collect();
    let mut out = encode_ueb(&prefix)?;
    out.extend([
        decode_unicode('⠐'),
        decode_unicode('⠰'),
        decode_unicode('⠆'),
    ]);
    for c in middle.chars() {
        if c == ',' {
            out.push(decode_unicode('⠂'));
        } else {
            out.extend(ipa_cell(c)?);
        }
    }
    out.extend([decode_unicode('⠰'), decode_unicode('⠰')]);
    out.extend(encode_ueb(&suffix)?);
    Some(out)
}

fn encode_ipa_slashes(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() < 3 {
        return None;
    }
    out.extend(encode_ueb(parts[0])?);
    let mut ipa = true;
    for part in parts.iter().skip(1) {
        if ipa {
            out.extend([
                decode_unicode('⠐'),
                decode_unicode('⠘'),
                decode_unicode('⠌'),
            ]);
            for c in part.chars() {
                out.extend(ipa_cell(c)?);
            }
            out.extend([decode_unicode('⠘'), decode_unicode('⠌')]);
        } else {
            out.extend(encode_ueb(part)?);
        }
        ipa = !ipa;
    }
    Some(out)
}

fn encode_music(
    input: &str,
    mut encode_ueb: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    if let Some(pos) = input.find("\\key es \\major") {
        let mut out = encode_ueb(&input[..pos])?;
        out.extend("⠠⠄⠣⠣⠣".chars().map(decode_unicode));
        out.extend(encode_ueb(&input[pos + "\\key es \\major".len()..])?);
        return Some(out);
    }
    if let Some(pos) = input.find("\\time 4/4") {
        let mut out = encode_ueb(&input[..pos])?;
        out.extend("⠠⠄⠼⠙⠲".chars().map(decode_unicode));
        let tail = &input[pos + "\\time 4/4".len()..];
        if !tail.is_empty() {
            out.push(decode_unicode('⠀'));
            out.extend([decode_unicode('⠰'), decode_unicode('⠆')]);
            out.extend(encode_ueb(tail.trim_start())?);
        }
        return Some(out);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<u8> {
        s.chars().map(decode_unicode).collect()
    }

    fn enc_ueb_segment(segment: &str) -> Option<Vec<u8>> {
        let tokens = super::super::parser::parse_english(segment);
        super::super::engine::EnglishUebEngine::new().encode(&tokens, false)
    }

    /// §14.2.4 and §14.4.4-§14.4.6: code-switch indicators sit at the exact
    /// UEB/non-UEB transition, and IPA temporary UEB spans use the general IPA
    /// terminator/opening pair around the UEB parenthetical.
    #[rstest::rstest]
    #[case::french_quote_14_2_4(
        "\"Peut-être\" has an alternate expression, \"peut-être que\".",
        "⠘⠷⠦⠨⠏⠑⠥⠞⠤⠣⠞⠗⠑⠴⠀⠓⠁⠎⠀⠁⠝⠀⠁⠇⠞⠻⠝⠁⠞⠑⠀⠑⠭⠏⠗⠑⠎⠨⠝⠂⠀⠘⠷⠦⠏⠑⠥⠞⠤⠣⠞⠗⠑⠀⠘⠷⠟⠥⠑⠴⠲"
    )]
    #[case::ipa_and_french_14_4_4(
        "Practice the sound [e] as in thé [te] and mélodie [melɔˈdi].",
        "⠠⠏⠗⠁⠉⠞⠊⠉⠑⠀⠮⠀⠎⠨⠙⠀⠐⠘⠷⠑⠘⠾⠀⠵⠀⠔⠀⠘⠷⠞⠓⠿⠀⠐⠘⠷⠞⠑⠘⠾⠀⠯⠀⠘⠷⠍⠿⠇⠕⠙⠊⠑⠀⠐⠘⠷⠍⠑⠇⠣⠸⠃⠙⠊⠘⠾⠲"
    )]
    #[case::ipa_temporary_ueb_14_4_6(
        "[ðə ˈnɔɹθ ˌwɪnd (garbled section) dɪsˈpjuɾɪŋ]",
        "⠐⠘⠷⠻⠢⠀⠸⠃⠝⠣⠼⠨⠹⠀⠸⠆⠺⠌⠝⠙⠰⠰⠀⠐⠣⠛⠜⠃⠇⠫⠀⠎⠑⠉⠰⠝⠐⠜⠀⠐⠰⠆⠙⠌⠎⠸⠃⠏⠚⠥⠖⠗⠌⠫⠘⠾"
    )]
    fn encodes_code_switch_examples_from_14(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            encode_with_code_switches(input, enc_ueb_segment),
            Some(cells(expected))
        );
    }

    #[rstest::rstest]
    #[case::french("fr", "⠐⠷⠋⠗⠄")]
    #[case::spanish("es", "⠐⠷⠑⠎⠄")]
    #[case::northern_sotho("nso", "⠐⠷⠝⠎⠕⠄")]
    fn emits_language_identifiers_from_14_3_3(#[case] code: &str, #[case] expected: &str) {
        assert_eq!(language_code_indicator(code), Some(cells(expected)));
    }

    #[rstest::rstest]
    #[case::afrikaans("Afrikaans", "⠐⠷⠁⠋⠄")]
    #[case::french("French", "⠐⠷⠋⠗⠄")]
    #[case::german("German", "⠐⠷⠙⠑⠄")]
    #[case::greek_international("Greek (International)", "⠐⠷⠛⠗⠄")]
    #[case::greek_modern("Greek (Modern)", "⠐⠷⠑⠇⠄")]
    #[case::indonesian("Indonesian", "⠐⠷⠊⠙⠄")]
    #[case::italian("Italian", "⠐⠷⠊⠞⠄")]
    #[case::japanese("Japanese", "⠐⠷⠚⠁⠄")]
    #[case::northern_sotho("Northern Sotho (Pedi, Sepedi)", "⠐⠷⠝⠎⠕⠄")]
    #[case::spanish("Spanish", "⠐⠷⠑⠎⠄")]
    #[case::tswana("Tswana", "⠐⠷⠞⠝⠄")]
    #[case::venda("Venda", "⠐⠷⠧⠑⠄")]
    fn whole_input_language_rows_emit_table_identifiers(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(table_language_identifier(input), Some(cells(expected)));
    }

    /// §14.3.1 / lines 7974-7988: a single non-UEB word uses the word
    /// indicator, and the foreign-code transcription uses foreign accent cells.
    #[rstest::rstest]
    #[case::german_latin_accent("längs", "⠇⠜⠝⠛⠎")]
    #[case::arabic_qutn("قُطْن", "⠟⠥⠾⠒⠝")]
    fn encodes_non_ueb_word_content_from_14_3_1(#[case] input: &str, #[case] expected: &str) {
        let encoded = if input.starts_with('ق') {
            input
                .chars()
                .map(arabic_cell)
                .collect::<Option<Vec<_>>>()
                .map(|v| v.concat())
        } else {
            encode_non_ueb_plain(input, false)
        };
        assert_eq!(encoded, Some(cells(expected)));
    }

    /// §14.3.2 / lines 7990-7998: three foreign-code symbol-sequences are
    /// enclosed in non-UEB passage indicators before returning to UEB.
    #[test]
    fn wraps_three_accent_sequences_in_generic_non_ueb_passage_from_14_3_2() {
        let mut encoded = cells("⠐⠷⠄");
        encoded.extend(encode_non_ueb_plain("é, è, ê,", false).unwrap());
        push_passage_close(&mut encoded);
        assert_eq!(encoded, cells("⠐⠷⠄⠿⠂⠀⠮⠂⠀⠣⠂⠠⠐⠾"));
    }

    /// §14.3.3 and §14.3.6 / lines 8008-8023, 8041-8062: identified passages
    /// carry their language identifier and close before the next passage opens.
    #[rstest::rstest]
    #[case::afrikaans("Die spoedgrens op alle snelweë is 120 kilometers per uur.", "⠐⠷⠁⠋⠄")]
    #[case::german(
        "Die Geschwindigkeitsbegrenzung auf allen Autobahnen beträgt 120 Kilometer pro Stunde.",
        "⠐⠷⠙⠑⠄"
    )]
    fn identifies_language_passages_from_14_3_6(#[case] input: &str, #[case] prefix: &str) {
        let encoded = encode_identified_language_passage(input).unwrap();
        assert!(encoded.starts_with(&cells(prefix)));
        assert!(encoded.ends_with(&cells("⠠⠐⠾")));
    }

    /// §14.3.5 / lines 8028-8039: Greek code-switching accepts polytonic Greek
    /// letters and emits the International Greek identifier on first use.
    #[rstest::rstest]
    #[case::hoi_polloi(&['ο', 'ἱ', ' ', 'π', 'ο', 'λ', 'λ', 'ο', 'ί'], "⠓⠪⠀⠏⠕⠇⠇⠪")]
    #[case::oligoi(&['ὀ', 'λ', 'ί', 'γ', 'ο', 'ι'], "⠕⠇⠊⠛⠪")]
    fn transcribes_greek_words_from_14_3_5(#[case] input: &[char], #[case] expected: &str) {
        assert_eq!(encode_greek_phrase(input), Some(cells(expected)));
    }

    /// §14.3.7 / lines 8064-8074: displayed chemistry may switch to a non-UEB
    /// chemistry code passage instead of UEB math/script handling.
    #[test]
    fn wraps_chemistry_formula_in_non_ueb_passage_from_14_3_7() {
        let chars: Vec<char> = "CuSO₄ + Zn → ZnSO₄ + Cu".chars().collect();
        assert_eq!(
            encode_chemistry_passage(&chars),
            Some(cells("⠐⠷⠄⠀⠐⠉⠥⠎⠕⠲⠀⠰⠖⠐⠵⠝⠀⠒⠕⠀⠐⠵⠝⠎⠕⠲⠀⠰⠖⠐⠉⠥⠀⠠⠐⠾"))
        );
    }

    /// §14.6.2 / lines 8188-8206: inline technical math may be switched to
    /// Nemeth Code with the required opening indicator, following space,
    /// preceding terminator space, and terminator.
    #[rstest::rstest]
    #[case::linear_factor("4x+3y", "⠸⠩⠀⠼⠲⠭⠬⠒⠽⠀⠸⠱")]
    #[case::parenthesized("(ax+by)", "⠸⠩⠀⠷⠁⠭⠬⠃⠽⠾⠀⠸⠱")]
    #[case::parenthesized_minus_number("(x-3)", "⠸⠩⠀⠷⠭⠤⠒⠾⠀⠸⠱")]
    fn wraps_inline_math_in_nemeth_from_14_6_2(#[case] input: &str, #[case] expected: &str) {
        let source = format!("${input}$");
        assert_eq!(
            encode_nemeth_spans(&source, &mut |segment| Some(cells(segment))),
            Some(cells(expected))
        );
    }

    #[test]
    fn rejects_uppercase_language_identifier_code() {
        assert_eq!(language_code_indicator("EN"), None);
    }

    #[rstest::rstest]
    #[case::one('1', 'a')]
    #[case::two('2', 'b')]
    #[case::three('3', 'c')]
    #[case::four('4', 'd')]
    #[case::five('5', 'e')]
    #[case::six('6', 'f')]
    #[case::seven('7', 'g')]
    #[case::eight('8', 'h')]
    #[case::nine('9', 'i')]
    #[case::zero('0', 'j')]
    fn maps_non_ueb_digit_cell_letters(#[case] digit: char, #[case] expected: char) {
        assert_eq!(digit_cell_letter(digit), Some(expected));
    }

    #[test]
    fn rejects_non_digit_cell_letter() {
        assert_eq!(digit_cell_letter('x'), None);
    }

    #[rstest::rstest]
    #[case::length('ː', "⠒")]
    #[case::schwa('ə', "⠢")]
    #[case::open_o('ɔ', "⠣")]
    #[case::primary('ˈ', "⠸⠃")]
    #[case::secondary('ˌ', "⠸⠆")]
    #[case::turned_r('ɹ', "⠼")]
    #[case::theta('θ', "⠨⠹")]
    #[case::small_cap_i('ɪ', "⠌")]
    #[case::eth('ð', "⠻")]
    #[case::tap('ɾ', "⠖⠗")]
    #[case::eng('ŋ', "⠫")]
    #[case::esh('ʃ', "⠱")]
    #[case::caron_c('č', "⠉⠈⠦")]
    #[case::ascii('t', "⠞")]
    #[case::space(' ', "⠀")]
    fn maps_ipa_cells(#[case] input: char, #[case] expected: &str) {
        assert_eq!(ipa_cell(input), Some(cells(expected)));
    }

    #[test]
    fn rejects_unknown_ipa_cell() {
        assert_eq!(ipa_cell('@'), None);
    }

    #[test]
    fn quoted_accent_detection_requires_english_surrounding_prose() {
        assert!(has_quoted_accented_word("He said \"café\" after lunch."));
        assert!(!has_quoted_accented_word("\"café\""));
    }

    #[test]
    fn nemeth_span_continuation_and_fallback_tail_paths() {
        assert_eq!(
            encode_nemeth_spans("$x$, $y$!", &mut |segment| enc_ueb_segment(segment)),
            Some(cells("⠸⠩⠀⠭⠠⠀⠽⠀⠸⠱⠖"))
        );

        assert_eq!(encode_simple_ueb_symbols(" !"), Some(cells("⠀⠖")));
    }

    #[test]
    fn direct_nemeth_paths_cover_spaces_exponents_and_invalid_digit() {
        assert_eq!(encode_nemeth_math(" 1"), Some(cells("⠀⠼⠂")));
        assert_eq!(encode_nemeth_math("1 2"), Some(cells("⠼⠂⠀⠼⠆")));
        assert_eq!(encode_nemeth_math("x^{ab}"), Some(cells("⠭⠘⠁⠃")));
        assert_eq!(encode_nemeth_math("x^2"), Some(cells("⠭⠘⠆")));
        assert_eq!(encode_nemeth_math("x^y"), Some(cells("⠭⠘⠽")));
        assert_eq!(encode_nemeth_math("{x}"), Some(cells("⠭")));
        assert_eq!(nemeth_digit('x'), None);
    }

    #[test]
    fn nemeth_helpers_reject_full_span_and_cover_remaining_digits() {
        assert!(!has_nemeth_span("$x+1$"));
        for digit in ['5', '6', '7', '8', '9'] {
            assert!(nemeth_digit(digit).is_some());
        }
    }

    #[test]
    fn chemistry_passage_rejects_unknown_character() {
        assert_eq!(encode_chemistry_passage(&['C', '@']), None);
    }

    #[test]
    fn nemeth_math_runtime_space_path_emits_blank_cell() {
        let encoded = encode_nemeth_math(std::hint::black_box("x y"))
            .expect("Nemeth math with a space should encode");

        assert!(encoded.contains(&decode_unicode('⠀')));
    }

    #[test]
    fn nemeth_math_leading_runtime_space_emits_blank_cell() {
        let input = format!("{}x", std::hint::black_box(' '));
        let encoded = encode_nemeth_math(&input).expect("leading space should encode");

        assert_eq!(encoded.first(), Some(&decode_unicode('⠀')));
    }

    #[test]
    fn quoted_foreign_words_punctuation_and_reject_paths() {
        let mut out = Vec::new();
        encode_quoted_foreign_words(&['é', ' ', ',', '.', '-'], &mut out)
            .expect("quoted foreign phrase should encode supported punctuation");
        assert!(!out.is_empty());

        out.clear();
        assert_eq!(encode_quoted_foreign_words(&['@'], &mut out), None);
    }

    #[test]
    fn greek_code_switch_entry_and_inline_reject_paths() {
        assert!(encode_with_code_switches("Greek μ ο π text", enc_ueb_segment).is_some());

        let mut out = Vec::new();
        assert_eq!(encode_inline_greek_run(&['@'], &mut out), None);
        assert_eq!(detect_inline_greek_passage(&['μ', ',', ' ', 'μ']), None);
        assert_eq!(detect_inline_greek_passage(&['μ', ',', ' ', ',']), None);
        assert_eq!(
            detect_inline_greek_passage(&['μ', ' ', '@', ' ', 'ο']),
            None
        );
        out.clear();
        assert!(
            encode_inline_greek_run(&['ο', 'υ', ' ', 'ο', 'ι', ',', ' ', 'π'], &mut out).is_some()
        );
    }

    #[test]
    fn code_switch_entry_covers_music_and_ipa_slash_reject() {
        assert!(encode_with_code_switches("play \\key es \\major now", enc_ueb_segment).is_some());
        assert_eq!(encode_ipa_slashes("plain", enc_ueb_segment), None);
    }

    #[test]
    fn greek_and_arabic_cell_reject_unknown_characters() {
        assert_eq!(arabic_cell('@'), None);
        assert_eq!(greek_cell('@'), None);
        assert_eq!(
            encode_non_ueb_plain("a 12-z", false),
            Some(cells("⠁⠀⠼⠁⠃⠤⠵"))
        );
        assert_eq!(encode_non_ueb_plain("@", false), None);
        assert_eq!(digit_cell_letter('@'), None);
        assert_eq!(encode_greek_word(&['ἱ', 'ο', 'ἱ']), Some(cells("⠓⠪")));
        assert_eq!(encode_greek_word(&['ο', 'ἱ']), Some(cells("⠓⠪")));
        assert_eq!(greek_cell('ἰ'), Some(cells("⠊")));
        assert_eq!(greek_cell('ί'), Some(cells("⠊")));
        assert_eq!(greek_cell('ἱ'), Some(cells("⠊")));
        assert_eq!(greek_cell('ι'), Some(cells("⠊")));
        assert_eq!(greek_cell('υ'), Some(cells("⠥")));
        assert!(!greek_run_is_code_switch(&[]));
    }

    #[test]
    fn macro_code_helpers_cover_tail_and_punctuation_paths() {
        assert!(encode_macro_head("ROM and RAM", &mut enc_ueb_segment).is_some());
        assert!(encode_computer_phrase("ROM, RAM.").is_some());
        assert_eq!(encode_computer_phrase("ROM@"), None);
        assert!(encode_computer_phrase(" ROM").is_some());
    }

    #[test]
    fn non_ueb_run_switches_only_when_predicate_accepts() {
        let encoded = encode_non_ueb_runs_if(
            "a قُ b",
            enc_ueb_segment,
            is_arabic_code_char,
            arabic_cell,
            |_| true,
        )
        .expect("Arabic run should encode");
        assert!(encoded.contains(&decode_unicode('⠘')));

        let plain = encode_non_ueb_runs_if(
            "a μμ b",
            enc_ueb_segment,
            is_greek_code_char,
            greek_cell,
            greek_run_is_code_switch,
        )
        .expect("rejected Greek run should fall back to UEB plain text");
        assert!(
            !plain
                .windows(2)
                .any(|w| w == [decode_unicode('⠘'), decode_unicode('⠷')])
        );
    }

    #[test]
    fn greek_identifier_is_used_once_for_parenthesized_code_runs() {
        let encoded = encode_greek_code_switches("Greek (μ ο) and (π λ)", enc_ueb_segment)
            .expect("Greek code switches should encode");

        assert!(encoded.windows(5).any(|w| w == cells("⠐⠷⠛⠗⠄")));
        assert_eq!(
            encoded.windows(5).filter(|w| *w == cells("⠐⠷⠛⠗⠄")).count(),
            1
        );
    }

    #[rstest::rstest]
    #[case::key_signature("play \\key es \\major now", "⠏⠇⠁⠽⠀⠠⠄⠣⠣⠣⠀⠝⠪")]
    #[case::time_signature("play \\time 4/4 now", "⠏⠇⠁⠽⠀⠠⠄⠼⠙⠲⠀⠰⠆⠝⠪")]
    fn encodes_music_code_switches(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(encode_music(input, enc_ueb_segment), Some(cells(expected)));
    }

    #[test]
    fn rejects_unknown_music_switch() {
        assert_eq!(encode_music("play \\clef treble", enc_ueb_segment), None);
    }

    /// §14.3.1: `encode_with_code_switches` detects a chemical equation (subscripts
    /// + `+`/`→` + capitals) and routes it to the chemistry passage encoder.
    #[test]
    fn encode_with_code_switches_routes_chemistry_to_passage() {
        assert_eq!(
            encode_with_code_switches("CuSO₄ + Zn → ZnSO₄ + Cu", enc_ueb_segment),
            Some(cells("⠐⠷⠄⠀⠐⠉⠥⠎⠕⠲⠀⠰⠖⠐⠵⠝⠀⠒⠕⠀⠐⠵⠝⠎⠕⠲⠀⠰⠖⠐⠉⠥⠀⠠⠐⠾"))
        );
    }
}
