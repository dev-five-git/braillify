//! Parse English source text into a flat `EnglishToken` stream.

use super::token::{EnglishToken, Typeform};

/// A word letter: ASCII alphabetic or a supported accented letter (§4.2), so
/// `crème` tokenizes as one word rather than `cr` + `è` + `me`.
fn is_word_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
        || super::rule_9::decode_small_cap(c).is_some()
        || super::rule_4::is_accented(c)
        || super::rule_12::is_early_letter(c)
        || super::rule_13::is_foreign_letter(c)
}

fn composed_word_letter(chars: &[char], i: usize) -> Option<(char, usize)> {
    let base = *chars.get(i)?;
    let mark = *chars.get(i + 1)?;
    super::rule_13::compose_combining(base, mark)
        .filter(|c| is_word_letter(*c))
        .map(|c| (c, i + 2))
}

fn combining_typeform(mark: char) -> Option<Typeform> {
    match mark {
        // §9 underline and §9.5 transcriber-defined print forms.
        '\u{0332}' => Some(Typeform::Underline),
        '\u{0333}' => Some(Typeform::Transcriber2),
        '\u{0336}' => Some(Typeform::Transcriber3),
        '\u{0323}' => Some(Typeform::Transcriber4),
        _ => None,
    }
}

fn combine_typeforms(base: Typeform, overlay: Typeform) -> Option<Typeform> {
    match (base, overlay) {
        (Typeform::Italic, Typeform::Underline) => Some(Typeform::ItalicUnderline),
        (Typeform::Bold, Typeform::Underline) => Some(Typeform::BoldUnderline),
        (Typeform::BoldItalic, Typeform::Underline) => Some(Typeform::BoldItalicUnderline),
        _ => None,
    }
}

fn stroke_overlay_ligature(chars: &[char], i: usize) -> bool {
    let run_len = struck_word_run_len(chars, i);
    if run_len >= 3 || struck_run_is_whole_word(chars, i, run_len) {
        return false;
    }
    chars.get(i + 1) == Some(&'\u{0336}')
        && (chars.get(i + 2).is_some_and(|c| is_word_letter(*c))
            || (i >= 2 && chars.get(i - 1) == Some(&'\u{0336}')))
}

/// UEB §9.5: crossed-out whole words, including two-letter words (`m̶y̶`,
/// `i̶s̶`), use a transcriber-defined typeform.  UEB §4.3 two-letter stroke
/// ligatures remain for embedded two-letter overlays within a larger word.
fn struck_run_is_whole_word(chars: &[char], i: usize, run_len: usize) -> bool {
    if run_len != 2 {
        return false;
    }
    let mut start = i;
    while start >= 2
        && chars.get(start - 1) == Some(&'\u{0336}')
        && is_word_letter(chars[start - 2])
    {
        start -= 2;
    }
    let end = start + run_len * 2;
    !start
        .checked_sub(1)
        .is_some_and(|p| is_word_letter(chars[p]))
        && !chars.get(end).is_some_and(|c| is_word_letter(*c))
}

fn struck_word_run_len(chars: &[char], i: usize) -> usize {
    let mut start = i;
    while start >= 2
        && chars.get(start - 1) == Some(&'\u{0336}')
        && is_word_letter(chars[start - 2])
    {
        start -= 2;
    }
    let mut len = 0;
    let mut k = start;
    while k + 1 < chars.len() && is_word_letter(chars[k]) && chars[k + 1] == '\u{0336}' {
        len += 1;
        k += 2;
    }
    len
}

fn small_cap_emphasis_word(chars: &[char], i: usize) -> bool {
    let mut k = i;
    while k > 0 {
        k -= 1;
        let c = chars[k];
        if c.is_ascii_uppercase() {
            return true;
        }
        if super::rule_9::decode_small_cap(c).is_some() || c.is_ascii_lowercase() {
            continue;
        }
        break;
    }
    false
}

/// A balanced `$...$` span is LaTeX/technical material only when the enclosed
/// text contains a technical signal.  UEB §3.10 also uses `$` as a currency sign,
/// including wordplay like `$hop for $aving$`; those dollar signs must stay as
/// ordinary symbols.
fn dollar_span_is_technical(span: &[char]) -> bool {
    span.iter().any(|c| {
        matches!(
            *c,
            '\\' | '^' | '_' | '{' | '}' | '+' | '=' | '<' | '>' | '−' | '×'
        )
    }) || span.iter().any(|c| c.is_ascii_digit())
}

/// Tokenize `text`: runs of word letters become `Word`, runs of ASCII digits
/// become `Number`, a single space becomes `Space`, anything else `Symbol`.
pub fn parse_english(text: &str) -> Vec<EnglishToken> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // §9: a Mathematical-Alphanumeric styled letter is its own token.
        if c.is_ascii_uppercase()
            && chars
                .get(i + 1)
                .is_some_and(|next| super::rule_9::decode_small_cap(*next).is_some())
        {
            tokens.push(EnglishToken::Styled(c, Typeform::Transcriber5));
            i += 1;
            continue;
        }
        if let Some(cap) = super::rule_9::decode_small_cap(c)
            && small_cap_emphasis_word(&chars, i)
        {
            // UEB §9.6.2: regular-height capitals mark the capitalised letters in
            // significant small-caps text; the small-cap glyphs themselves encode
            // as lowercase letters under the fifth transcriber-defined typeform.
            tokens.push(EnglishToken::Styled(
                cap.to_ascii_lowercase(),
                Typeform::Transcriber5,
            ));
            i += 1;
            continue;
        }
        if let Some((base, form)) = super::rule_9::decode_styled(c) {
            if let Some(&mark) = chars.get(i + 1)
                && let Some(composed) = super::rule_13::compose_combining(base, mark)
            {
                tokens.push(EnglishToken::Styled(composed, form));
                i += 2;
            } else if let Some(&mark) = chars.get(i + 1)
                && let Some(overlay) = combining_typeform(mark)
                && let Some(combined) = combine_typeforms(form, overlay)
            {
                tokens.push(EnglishToken::Styled(base, combined));
                i += 2;
            } else {
                tokens.push(EnglishToken::Styled(base, form));
                i += 1;
            }
            continue;
        }
        if c == '$'
            && let Some(end_rel) = chars[i + 1..].iter().position(|&x| x == '$')
        {
            let end = i + 1 + end_rel;
            if dollar_span_is_technical(&chars[i + 1..end]) {
                tokens.push(EnglishToken::Technical(chars[i + 1..end].to_vec()));
                i = end + 1;
                continue;
            }
        }
        // §9: any character (letter, digit, or symbol) immediately followed by a
        // combining low line (U+0332) is underlined → a Styled token that also
        // ends any current word. A space is excluded (an underlined space opens a
        // §9.x passage, handled separately) as is a lone combining mark, so a
        // styled digit (`3̲4̲`) or symbol (`.̲`, `%̲`) is captured alongside letters.
        if c != ' '
            && combining_typeform(c).is_none()
            && !stroke_overlay_ligature(&chars, i)
            && chars
                .get(i + 1)
                .and_then(|m| combining_typeform(*m))
                .is_some()
        {
            let Some(form) = combining_typeform(chars[i + 1]) else {
                unreachable!("guard checked combining typeform")
            };
            tokens.push(EnglishToken::Styled(c, form));
            i += 2;
            continue;
        }
        if is_word_letter(c) {
            // A styled letter is not a word letter (so it stops the run on its
            // own); stop one early before a letter that an upcoming U+0332 will
            // underline so it becomes its own Styled token.
            let mut word = Vec::new();
            while i < chars.len()
                && (chars
                    .get(i + 1)
                    .and_then(|m| combining_typeform(*m))
                    .is_none()
                    || stroke_overlay_ligature(&chars, i))
            {
                if let Some((composed, next)) = composed_word_letter(&chars, i) {
                    word.push(composed);
                    i = next;
                } else if let Some(cap) = super::rule_9::decode_small_cap(chars[i]) {
                    word.push(cap);
                    i += 1;
                } else if is_word_letter(chars[i]) {
                    word.push(chars[i]);
                    i += 1;
                } else {
                    break;
                }
            }
            if chars.get(i) == Some(&'\n')
                && chars.get(i + 1).is_some_and(|next| is_word_letter(*next))
            {
                i += 1;
                let break_at = word.len();
                while i < chars.len()
                    && is_word_letter(chars[i])
                    && (chars
                        .get(i + 1)
                        .and_then(|m| combining_typeform(*m))
                        .is_none()
                        || stroke_overlay_ligature(&chars, i))
                {
                    if let Some((composed, next)) = composed_word_letter(&chars, i) {
                        word.push(composed);
                        i = next;
                    } else if let Some(cap) = super::rule_9::decode_small_cap(chars[i]) {
                        word.push(cap);
                        i += 1;
                    } else {
                        word.push(chars[i]);
                        i += 1;
                    }
                }
                tokens.push(EnglishToken::WordDivision {
                    chars: word,
                    break_at,
                });
            } else {
                tokens.push(EnglishToken::Word(word));
            }
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && chars[i].is_ascii_digit()
                && chars
                    .get(i + 1)
                    .and_then(|m| combining_typeform(*m))
                    .is_none()
            {
                i += 1;
            }
            tokens.push(EnglishToken::Number(chars[start..i].to_vec()));
        } else if c == ' ' {
            tokens.push(EnglishToken::Space);
            i += if chars
                .get(i + 1)
                .and_then(|m| combining_typeform(*m))
                .is_some()
            {
                2
            } else {
                1
            };
        } else if c == '\n' {
            tokens.push(EnglishToken::LineBreak);
            i += 1;
        } else {
            tokens.push(EnglishToken::Symbol(c));
            i += 1;
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_words_spaces_numbers_symbols() {
        let toks = parse_english("ab 12!c");
        assert_eq!(
            toks,
            vec![
                EnglishToken::Word(vec!['a', 'b']),
                EnglishToken::Space,
                EnglishToken::Number(vec!['1', '2']),
                EnglishToken::Symbol('!'),
                EnglishToken::Word(vec!['c']),
            ]
        );
    }

    #[test]
    fn runtime_plain_word_parses_as_word_token() {
        let input = std::hint::black_box("hello");

        assert_eq!(
            parse_english(input),
            vec![EnglishToken::Word(input.chars().collect())]
        );
    }

    #[test]
    fn parses_ueb_word_division_marker_inside_word() {
        let toks = parse_english("ante\nroom");
        assert_eq!(
            toks,
            vec![EnglishToken::WordDivision {
                chars: vec!['a', 'n', 't', 'e', 'r', 'o', 'o', 'm'],
                break_at: 4,
            }]
        );
    }

    #[test]
    fn parses_ueb_line_break_marker_after_existing_hyphen() {
        let toks = parse_english("about-\nface");
        assert_eq!(
            toks,
            vec![
                EnglishToken::Word(vec!['a', 'b', 'o', 'u', 't']),
                EnglishToken::Symbol('-'),
                EnglishToken::LineBreak,
                EnglishToken::Word(vec!['f', 'a', 'c', 'e']),
            ]
        );
    }

    #[test]
    fn parses_dollar_delimited_technical_material() {
        assert_eq!(
            parse_english("a $x^{2}$ b"),
            vec![
                EnglishToken::Word(vec!['a']),
                EnglishToken::Space,
                EnglishToken::Technical("x^{2}".chars().collect()),
                EnglishToken::Space,
                EnglishToken::Word(vec!['b']),
            ]
        );
    }

    /// §3.10: dollar signs used as currency signs are not swallowed as a `$...$`
    /// technical span merely because a later currency dollar appears.
    #[test]
    fn leaves_currency_dollars_as_symbols() {
        assert_eq!(
            parse_english("$hop for $aving$"),
            vec![
                EnglishToken::Symbol('$'),
                EnglishToken::Word(vec!['h', 'o', 'p']),
                EnglishToken::Space,
                EnglishToken::Word(vec!['f', 'o', 'r']),
                EnglishToken::Space,
                EnglishToken::Symbol('$'),
                EnglishToken::Word(vec!['a', 'v', 'i', 'n', 'g']),
                EnglishToken::Symbol('$'),
            ]
        );
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert!(parse_english("").is_empty());
    }

    #[test]
    fn runtime_small_cap_and_accent_letters_stay_in_word() {
        let text = std::hint::black_box("ᴅé");

        assert_eq!(
            parse_english(text),
            vec![EnglishToken::Word(vec!['D', 'é'])]
        );
    }

    #[test]
    fn consumes_combining_low_line_after_underlined_space() {
        assert_eq!(
            parse_english("a\u{0332} \u{0332}b\u{0332}"),
            vec![
                EnglishToken::Styled('a', Typeform::Underline),
                EnglishToken::Space,
                EnglishToken::Styled('b', Typeform::Underline),
            ]
        );
    }

    /// UEB §4.3.1 and §9.5: two adjacent struck letters are a ligature mark;
    /// three or more struck letters are a transcriber-defined typeform passage/word
    /// signal, not a §4.3 two-letter ligature mark.
    #[test]
    fn parses_long_struck_run_as_transcriber_typeform() {
        assert_eq!(
            parse_english("m\u{0336}u\u{0336}t\u{0336}"),
            vec![
                EnglishToken::Styled('m', Typeform::Transcriber3),
                EnglishToken::Styled('u', Typeform::Transcriber3),
                EnglishToken::Styled('t', Typeform::Transcriber3),
            ]
        );
    }

    #[rstest::rstest]
    #[case::italic_underlined('\u{1D44E}', '\u{0332}', Typeform::ItalicUnderline)]
    #[case::bold_underlined('\u{1D41A}', '\u{0332}', Typeform::BoldUnderline)]
    #[case::bold_italic_underlined('\u{1D482}', '\u{0332}', Typeform::BoldItalicUnderline)]
    fn combines_styled_letters_with_underline(
        #[case] styled: char,
        #[case] mark: char,
        #[case] form: Typeform,
    ) {
        assert_eq!(
            parse_english(&format!("{styled}{mark}")),
            vec![EnglishToken::Styled('a', form)]
        );
    }

    #[test]
    fn two_letter_stroke_inside_word_stays_word_ligature() {
        assert_eq!(
            parse_english("ab\u{0336}c\u{0336}d"),
            vec![
                EnglishToken::Word(vec!['a', 'b']),
                EnglishToken::Symbol('\u{0336}'),
                EnglishToken::Word(vec!['c']),
                EnglishToken::Symbol('\u{0336}'),
                EnglishToken::Word(vec!['d']),
            ]
        );
    }

    #[test]
    fn word_division_second_line_handles_combining_and_small_caps() {
        assert_eq!(
            parse_english("caf\ne\u{0301}ᴅ"),
            vec![EnglishToken::WordDivision {
                chars: vec!['c', 'a', 'f', 'é', 'D'],
                break_at: 3,
            }]
        );
    }

    #[test]
    fn stroke_ligature_second_overlay_continues_word_run() {
        assert_eq!(
            parse_english("b\u{0336}c\u{0336}"),
            vec![
                EnglishToken::Styled('b', Typeform::Transcriber3),
                EnglishToken::Styled('c', Typeform::Transcriber3),
            ]
        );
    }

    #[test]
    fn stroke_overlay_ligature_keeps_middle_overlay_inside_word() {
        let chars: Vec<char> = "ab\u{0336}c\u{0336}d".chars().collect();

        assert!(stroke_overlay_ligature(&chars, 3));
    }

    #[test]
    fn stroke_overlay_ligature_continues_at_run_end_inside_word() {
        let chars: Vec<char> = "xa\u{0336}b\u{0336}".chars().collect();

        assert!(stroke_overlay_ligature(&chars, 3));
    }

    #[test]
    fn word_division_second_line_keeps_stroke_ligature_word_run() {
        assert_eq!(
            parse_english("ab\nc\u{0336}d"),
            vec![
                EnglishToken::WordDivision {
                    chars: vec!['a', 'b', 'c'],
                    break_at: 2,
                },
                EnglishToken::Symbol('\u{0336}'),
                EnglishToken::Word(vec!['d'])
            ]
        );
    }

    /// §9.5: the combining dot below (U+0323) is the fourth transcriber-defined
    /// print form; unrelated characters carry no typeform.
    #[test]
    fn combining_typeform_maps_dot_below_to_transcriber4() {
        assert_eq!(combining_typeform('\u{0323}'), Some(Typeform::Transcriber4));
        assert_eq!(combining_typeform('a'), None);
    }

    /// §4.2: a base letter followed by a combining accent composes into a single
    /// accented word letter (`a` + U+0301 → `á`) rather than splitting the word.
    #[test]
    fn parse_english_composes_letter_with_combining_accent() {
        assert_eq!(
            parse_english("a\u{0301}"),
            vec![EnglishToken::Word(vec!['á'])]
        );
    }
}
