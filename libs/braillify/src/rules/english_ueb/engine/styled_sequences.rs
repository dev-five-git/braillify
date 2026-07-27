use super::*;

/// §9.5: the exclusive end of a *word*-level typeform extent starting at `i` — the
/// index just past the last same-`form` styled token reachable through only
/// same-form styled tokens and attached symbols (no space). A styled word with
/// internal punctuation (`𝑜𝑓-𝑡ℎ𝑒`, `𝑙'𝑜𝑒𝑖𝑙-𝑑𝑒-𝑏𝑜𝑒𝑢𝑓`) is therefore one extent,
/// while a following space or plain word ends it (a trailing symbol like the `/`
/// in `a̲n̲d̲/` is excluded — the span ends at its last styled token).
pub(super) fn styled_word_span(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
) -> usize {
    let mut last_styled_end = i;
    let mut k = i;
    while let Some(t) = tokens.get(k) {
        match t {
            t if token_typeform(t) == Some(form) => {
                k += 1;
                last_styled_end = k;
            }
            EnglishToken::Symbol(_) | EnglishToken::LineBreak => k += 1,
            _ => break,
        }
    }
    last_styled_end
}

/// §9.3.2: a typeform word indicator over a numeric symbols-sequence is not
/// terminated by numeric spaces. Consume same-form styled digits/symbols and the
/// spaces that separate numeric groups (`1̲ ̲5̲0̲0̲`).
pub(super) fn styled_numeric_sequence_end(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
) -> usize {
    let mut saw_digit = false;
    let mut saw_separator = false;
    let mut last = i;
    let mut k = i;
    while let Some(token) = tokens.get(k) {
        match token {
            EnglishToken::Styled(c, f) if *f == form && c.is_ascii_digit() => {
                saw_digit = true;
                k += 1;
                last = k;
            }
            EnglishToken::Styled(c, f) if *f == form && matches!(c, ',' | '.' | '-') => {
                saw_separator = true;
                k += 1;
                last = k;
            }
            EnglishToken::Space
                if saw_digit
                    && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(c, f)) if *f == form && c.is_ascii_digit()) =>
            {
                saw_separator = true;
                k += 1;
                last = k;
            }
            _ => break,
        }
    }
    if saw_digit && saw_separator { last } else { i }
}

/// §9.3.1: a single styled capital can begin a larger symbols-sequence (for
/// example `𝑅.𝑆.` or `𝐍(𝑆)`). In that position the typeform applies to the
/// remainder of the symbols-sequence, so use the word indicator rather than the
/// symbol indicator for the first styled capital.
pub(super) fn styled_capital_starts_symbol_sequence(
    tokens: &[EnglishToken],
    i: usize,
    j: usize,
) -> bool {
    let Some(EnglishToken::Styled(c, _)) = tokens.get(i) else {
        return false;
    };
    if !c.is_ascii_uppercase() || !matches!(tokens.get(j), Some(EnglishToken::Symbol(_))) {
        return false;
    }
    let mut k = j + 1;
    while let Some(token) = tokens.get(k) {
        match token {
            EnglishToken::Styled(..)
            | EnglishToken::Word(_)
            | EnglishToken::Number(_)
            | EnglishToken::Technical(_) => return true,
            EnglishToken::Symbol(_) | EnglishToken::LineBreak => k += 1,
            EnglishToken::Space | EnglishToken::WordDivision { .. } => return false,
        }
    }
    false
}

/// UEB §9.3.1-§9.3.2: a typeform word indicator can cover a full
/// symbols-sequence, not just the first styled letter.  Initialisms such as
/// `𝑅.𝑆.𝑉.𝑃.` therefore get one word indicator before the sequence, and the
/// intervening punctuation/next same-form styled capitals stay within its scope
/// until a space terminates the typeform word mode.
pub(super) fn styled_symbol_sequence_end(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
) -> usize {
    let mut k = i;
    let mut styled_items = 0usize;
    let mut symbols = 0usize;
    let mut last_text_end = i;
    while let Some(token) = tokens.get(k) {
        match token {
            EnglishToken::Styled(c, f) if *f == form && c.is_ascii_alphanumeric() => {
                styled_items += 1;
                k += 1;
                last_text_end = k;
            }
            EnglishToken::Symbol('.') if styled_items > 0 => {
                symbols += 1;
                k += 1;
            }
            _ => break,
        }
    }
    if styled_items >= 2 && symbols > 0 {
        k
    } else {
        last_text_end
    }
}

pub(super) fn encode_styled_symbol_sequence(
    tokens: &[EnglishToken],
    start: usize,
    end: usize,
    form: super::super::token::Typeform,
    out: &mut Vec<u8>,
) -> Option<()> {
    let mut k = start;
    while k < end {
        match tokens.get(k)? {
            EnglishToken::Styled(c, f) if *f == form && c.is_ascii_alphabetic() => {
                push_literal_letter(*c, out)?;
            }
            EnglishToken::Styled(c, f) if *f == form && c.is_ascii_digit() => {
                out.extend(super::super::rule_6::encode_number(&[*c])?);
            }
            EnglishToken::Styled(c, f) if *f == form => {
                encode_styled_nonword_symbol(*c, out)?;
            }
            EnglishToken::Symbol(c) => {
                let cells = super::super::rule_7::encode_punctuation(*c)
                    .or_else(|| super::super::rule_3::encode_symbol(*c))?;
                out.extend(cells);
            }
            _ => return None,
        }
        k += 1;
    }
    Some(())
}

pub(super) fn encode_styled_numeric_sequence(
    tokens: &[EnglishToken],
    start: usize,
    end: usize,
    form: super::super::token::Typeform,
    out: &mut Vec<u8>,
) -> Option<()> {
    let mut numeric_mode = false;
    let mut k = start;
    while k < end {
        match &tokens[k] {
            EnglishToken::Styled(c, f) if *f == form && c.is_ascii_digit() => {
                if !numeric_mode {
                    out.push(decode_unicode('⠼'));
                    numeric_mode = true;
                }
                out.push(super::super::rule_6::digit_cell(*c)?);
                k += 1;
            }
            EnglishToken::Styled(c, f) if *f == form => {
                let cells = super::super::rule_7::encode_punctuation(*c)
                    .or_else(|| super::super::rule_3::encode_symbol(*c))?;
                out.extend(cells);
                numeric_mode = matches!(c, ',' | '.')
                    && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(n, nf)) if *nf == form && n.is_ascii_digit());
                k += 1;
            }
            EnglishToken::Space => {
                out.push(decode_unicode('⠐'));
                numeric_mode = true;
                k += 1;
            }
            _ => return None,
        }
    }
    Some(())
}

/// §9.3/§10.7 collision: whether the whole-word styled letters would encode to
/// a §10.7 initial-letter contraction whose two-cell form starts with the SAME
/// typeform prefix cell that a word indicator would emit — so `⠘⠂⠘⠺` (bold
/// word indicator + `word` contraction) collapses to just `⠘⠺` (reader still
/// sees a bold `word` cell, no ambiguity). Covers §10.7 words with dot-4-5 or
/// dot-4-5-6 prefixes matched against Bold/Underline typeforms; the ⠐ prefix
/// contractions have no matching typeform so they are excluded.
pub(super) fn styled_word_matches_typeform_prefix_contraction(
    chars: &[char],
    form: super::super::token::Typeform,
) -> bool {
    let lower: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    match form {
        super::super::token::Typeform::Bold => matches!(lower.as_str(), "word" | "whose"),
        super::super::token::Typeform::Underline => {
            matches!(lower.as_str(), "cannot" | "spirit" | "world" | "many")
        }
        _ => false,
    }
}

pub(super) fn continues_uppercase_word_across_typeform(tokens: &[EnglishToken], i: usize) -> bool {
    i.checked_sub(1).is_some_and(|p| {
        matches!(
            tokens.get(p),
            Some(EnglishToken::Word(prev))
                if prev.len() >= 2 && prev.iter().all(|c| c.is_ascii_uppercase())
        )
    })
}

/// §9.1.3 example note: if a document's only italicised items are repeated single
/// capital letters used as variable names, the typeform is not significant.
pub(super) fn insignificant_single_italic_capitals(tokens: &[EnglishToken]) -> bool {
    let mut count = 0usize;
    for (i, token) in tokens.iter().enumerate() {
        if let EnglishToken::Styled(c, super::super::token::Typeform::Italic) = token {
            if !c.is_uppercase() {
                return false;
            }
            if matches!(
                i.checked_sub(1).and_then(|p| tokens.get(p)),
                Some(EnglishToken::Symbol('.'))
            ) || matches!(tokens.get(i + 1), Some(EnglishToken::Symbol('.')))
            {
                return false;
            }
            count += 1;
        }
    }
    count >= 2
}

/// §9.3.2: a styled symbols-sequence that is URL-shaped may be followed in print
/// by extra visual spacing before prose. The URL itself keeps its typeform word
/// indicator, but the prose spacing collapses to the ordinary single blank cell.
pub(super) fn styled_url_before(tokens: &[EnglishToken], i: usize) -> bool {
    let Some(EnglishToken::Styled(_, form)) = i.checked_sub(1).and_then(|p| tokens.get(p)) else {
        return false;
    };
    // Walk left over the styled-URL run (styled letters of `form` plus `:` `/`
    // `.`), prepending each token's text. The walk itself validates the run, so a
    // separate re-scan (with an unreachable fallback arm) is unnecessary.
    let mut text = String::new();
    let mut start = i;
    while start > 0 {
        match tokens.get(start - 1) {
            Some(EnglishToken::Styled(c, f)) if *f == *form => {
                let lower: String = c.to_lowercase().collect();
                text.insert_str(0, &lower);
                start -= 1;
            }
            Some(EnglishToken::Symbol(c @ (':' | '/' | '.'))) => {
                text.insert(0, *c);
                start -= 1;
            }
            _ => break,
        }
    }
    text.starts_with("http://") || text.starts_with("https://") || text.starts_with("www.")
}

/// UEB §9.8.1 nested passage continuation: if text opens with a nested typeform
/// (`bold+italic`) and later drops only the inner form while retaining the outer
/// form (`italic`), keep the outer passage open and close only the inner one at
/// the first change.
pub(super) fn nested_typeform_continuation(
    tokens: &[EnglishToken],
    inner_end: usize,
    form: super::super::token::Typeform,
) -> Option<(
    usize,
    super::super::token::Typeform,
    super::super::token::Typeform,
)> {
    use super::super::token::Typeform::{Bold, BoldItalic, Italic};

    let (outer, inner) = match form {
        BoldItalic => (Italic, Bold),
        _ => return None,
    };
    let mut k = inner_end;
    while matches!(
        tokens.get(k),
        Some(EnglishToken::Space | EnglishToken::LineBreak)
    ) {
        k += 1;
    }
    if !matches!(tokens.get(k).and_then(token_typeform), Some(f) if f == outer) {
        return None;
    }
    let (words, outer_end) = styled_passage_extent(tokens, k, outer);
    (words > 0).then_some((outer_end, outer, inner))
}

/// UEB §9.1.3: underlining used only as a hyperlink print enhancement (URL-shaped
/// `http://…` or `www.…`) is not significant, unlike underlining that marks
/// embedded text; therefore its §9 typeform is omitted while the symbols-sequence
/// is still encoded normally.
pub(super) fn styled_underline_url_span(
    tokens: &[EnglishToken],
    start: usize,
    end: usize,
    form: super::super::token::Typeform,
) -> bool {
    if form != super::super::token::Typeform::Underline {
        return false;
    }
    let mut text = String::new();
    for token in &tokens[start..end] {
        match token {
            EnglishToken::Styled(c, f) if *f == form => {
                text.extend(c.to_lowercase());
            }
            EnglishToken::Symbol(c) => text.push(c.to_ascii_lowercase()),
            _ => return false,
        }
    }
    text.starts_with("http://") || text.starts_with("https://") || text.starts_with("www.")
}

/// §5.7.1/§9.1.3 support: whether a styled single-letter token at `tokens[i]`
/// (with post-run end `j`) needs a grade-1 indicator once its §9 typeform has
/// been stripped as insignificant. Mirrors the §2.6 boundary logic in
/// `rule_5_7::needs_grade1_indicator` — that helper is `Word`-token-only.
pub(super) fn styled_letter_needs_grade1(tokens: &[EnglishToken], i: usize, j: usize) -> bool {
    let is_boundary = |t: Option<&EnglishToken>| {
        matches!(
            t,
            None | Some(EnglishToken::Space)
                | Some(EnglishToken::Symbol('-' | '\u{2013}' | '\u{2014}'))
        )
    };
    let is_left_transparent =
        |c: char| matches!(c, '(' | '[' | '{' | '"' | '\u{201C}' | '\u{2018}' | '\'');
    let is_right_transparent = |c: char| {
        matches!(
            c,
            ')' | ']' | '}' | '"' | '\u{201D}' | '\u{2019}' | '.' | ',' | ':' | ';' | '\u{2026}'
        )
    };
    let mut l = i;
    while l > 0 && matches!(&tokens[l - 1], EnglishToken::Symbol(c) if is_left_transparent(*c)) {
        l -= 1;
    }
    if !is_boundary(l.checked_sub(1).map(|p| &tokens[p])) {
        return false;
    }
    let mut r = j.saturating_sub(1).max(i);
    while r + 1 < tokens.len()
        && matches!(&tokens[r + 1], EnglishToken::Symbol(c) if is_right_transparent(*c))
    {
        r += 1;
    }
    is_boundary(tokens.get(r + 1))
}

/// Identify §8.4 capitals passages: runs of three or more space-separated
/// all-caps "words". Returns per-token flags — emit `⠠⠠⠠` *before* a token,
/// emit the `⠠⠄` terminator *after* a token, and whether a token lies *inside*
/// a passage (so caps words drop their own indicator). Below the 3-word
/// threshold every flag stays false, so 1–2 caps-word inputs are untouched.
pub(super) fn caps_passages(
    tokens: &[EnglishToken],
    explicit_english: bool,
) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    let n = tokens.len();
    let (mut starts, mut terms, mut inside) = (vec![false; n], vec![false; n], vec![false; n]);

    // Space-separated groups, as inclusive token ranges. Opening punctuation is
    // included in the group but the passage indicator is placed immediately before
    // the first capitalised sequence (§8.5 placement inside opening quotes).
    // Lowercase groups are retained as barriers: §8.5.2 requires three or more
    // capitalised symbols-sequences in the *passage*, so intervening lowercase
    // words (`GO quickly and TAKE CARE`) must prevent the two later caps words
    // from being counted with the first.
    let mut groups: Vec<CapsGroup> = Vec::new();
    let mut g0: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        if matches!(
            t,
            EnglishToken::Space | EnglishToken::Symbol('–' | '—' | '―')
        ) {
            if let Some(s) = g0.take()
                && i > s
                && let Some(group) = caps_group_or_lower_barrier(tokens, s, i - 1)
            {
                groups.push(group);
            }
            // §8.5.4: a caps letters-sequence adjacent to an em-dash is not
            // necessarily part of a passage that starts after the dash. Insert
            // a synthetic barrier so the em-dash breaks passage merging (a plain
            // space keeps the passage cohesive as before).
            if matches!(t, EnglishToken::Symbol('–' | '—' | '―')) {
                groups.push(CapsGroup {
                    first_cap: i,
                    last_cap: i,
                    caps_sequences: 0,
                    has_lower: true,
                    single_letter_only: false,
                    ended_with_terminal_sentence_mark: false,
                });
            }
        } else if g0.is_none() {
            g0 = Some(i);
        }
    }
    if let Some(s) = g0
        && let Some(group) = caps_group_or_lower_barrier(tokens, s, n - 1)
    {
        groups.push(group);
    }

    let mut gi = 0;
    // §8.5.5 helper: whether the input consists of a single quoted all-caps
    // fragment (`"HE'S GETTING AWAY!"`). Such fragments are treated as one text
    // element inside a larger multi-element passage — they open with the passage
    // indicator ⠠⠠⠠ (even for a single caps word like "JUMP!") and elide the
    // ⠠⠄ terminator on the assumption that the passage continues.
    let input_is_quoted_all_caps = {
        let start_quote_at = tokens
            .first()
            .is_some_and(|t| matches!(t, EnglishToken::Symbol('"')));
        let end_quote_at = tokens
            .last()
            .is_some_and(|t| matches!(t, EnglishToken::Symbol('"')));
        let inner_has_lower = tokens
            .iter()
            .skip(1)
            .take(tokens.len().saturating_sub(2))
            .any(token_has_lower_sequence);
        let inner_has_upper = tokens
            .iter()
            .skip(1)
            .take(tokens.len().saturating_sub(2))
            .any(token_is_upper_sequence);
        !explicit_english
            && start_quote_at
            && end_quote_at
            && !inner_has_lower
            && inner_has_upper
            && !single_elongated_caps_word_in_quotes(tokens)
    };

    while gi < groups.len() {
        if groups[gi].is_caps() {
            let first = groups[gi].first_cap;
            let mut last = groups[gi].last_cap;
            let mut count = groups[gi].caps_sequences;
            let mut prev_ended_sentence = groups[gi].ended_with_terminal_sentence_mark;
            let mut gj = gi + 1;
            while gj < groups.len() && !groups[gj].has_lower {
                if groups[gj].is_caps() {
                    // §8.5.4: a single-letter capital immediately after a
                    // sentence-terminal caps group (`ABC.  A BBC`) is an
                    // article/pronoun, not a passage participant. It does not
                    // count toward the three-sequences threshold, and its
                    // position is not tracked as the passage extent.
                    let skip_single_letter = prev_ended_sentence && groups[gj].single_letter_only;
                    if !skip_single_letter {
                        last = groups[gj].last_cap;
                        count += groups[gj].caps_sequences;
                    }
                    prev_ended_sentence = groups[gj].ended_with_terminal_sentence_mark;
                }
                gj += 1;
            }
            // §8.5.2 threshold is normally three symbols-sequences, but §8.5.5
            // relaxes it when the whole input is a lone quoted all-caps
            // fragment — that fragment is one text element in a running passage,
            // so it opens with ⠠⠠⠠. It may be only one symbols-sequence (`"JUMP!"`).
            let passage_qualifies = count >= 3 || input_is_quoted_all_caps;
            if passage_qualifies {
                starts[first] = true;
                // §8.5.5 final element: a quoted all-caps passage that ends with
                // `!"` and spans multiple sentences (an internal `.  ` sentence
                // break) is the *final* element of a running passage — emit the
                // terminator immediately AFTER the closing quote (`⠴⠠⠄`), not
                // suppressed as for an intermediate element (L1/L2 end with `.`,
                // so keep the terminator dropped there).
                let is_final_element = input_is_quoted_all_caps
                    && tokens.len() >= 3
                    && matches!(
                        tokens.get(tokens.len() - 2),
                        Some(EnglishToken::Symbol('!'))
                    )
                    && tokens.windows(3).any(|w| {
                        matches!(w[0], EnglishToken::Symbol('.'))
                            && matches!(w[1], EnglishToken::Space)
                            && matches!(w[2], EnglishToken::Space)
                    });
                if is_final_element {
                    // Terminator placed on the CLOSING quote token (last token) so
                    // the emit path outputs `⠴⠠⠄` — the terminator falls after
                    // the closing `"` cell.
                    terms[tokens.len() - 1] = true;
                } else if !input_is_quoted_all_caps {
                    terms[last] = true;
                }
                for f in &mut inside[first..=last] {
                    *f = true;
                }
            }
            gi = gj;
        } else {
            gi += 1;
        }
    }
    (starts, terms, inside)
}

pub(super) fn caps_group_or_lower_barrier(
    tokens: &[EnglishToken],
    first: usize,
    last: usize,
) -> Option<CapsGroup> {
    caps_group(tokens, first, last).or_else(|| {
        tokens
            .iter()
            .take(last + 1)
            .skip(first)
            .any(token_has_lower_sequence)
            .then_some(CapsGroup {
                first_cap: first,
                last_cap: last,
                caps_sequences: 0,
                has_lower: true,
                single_letter_only: false,
                ended_with_terminal_sentence_mark: false,
            })
    })
}

impl CapsGroup {
    pub(super) fn is_caps(self) -> bool {
        !self.has_lower && self.caps_sequences > 0
    }
}

pub(super) fn caps_group(tokens: &[EnglishToken], first: usize, last: usize) -> Option<CapsGroup> {
    let mut first_cap = None;
    let mut last_cap = None;
    let mut caps_sequences = 0usize;
    let mut upper_tokens = 0usize;
    let mut has_lower = false;
    let mut has_styled_text = false;
    let mut terminal_punctuation = false;
    for (i, token) in tokens.iter().enumerate().take(last + 1).skip(first) {
        if first_cap.is_some() && matches!(token, EnglishToken::Symbol('/')) {
            break;
        }
        has_lower |= token_has_lower_sequence(token)
            || matches!(token, EnglishToken::Symbol('/' | '|' | '‖'));
        has_styled_text |= token_is_styled_text(token);
        if token_is_upper_sequence(token) {
            first_cap.get_or_insert(i);
            last_cap = Some(i);
            caps_sequences = 1;
            upper_tokens += 1;
        } else if first_cap.is_some()
            && matches!(
                token,
                EnglishToken::Symbol('!' | '?' | '.' | ',' | ':' | ';' | ')' | ']' | '}')
            )
        {
            terminal_punctuation = true;
            last_cap = Some(i);
        }
    }
    if has_styled_text {
        // Typeform passage handling opens any nested §8 caps passage after the §9
        // indicator. The document-level caps scan would otherwise emit a duplicate
        // `⠠⠠⠠` before the typeform indicator.
        return None;
    }
    let single_letter_sentence = terminal_punctuation
        && caps_sequences == 1
        && upper_tokens == 1
        && first_cap
            .is_some_and(|idx| matches!(&tokens[idx], EnglishToken::Word(w) if w.len() == 1));
    let single_letter_only = caps_sequences == 1
        && upper_tokens == 1
        && !terminal_punctuation
        && first_cap
            .is_some_and(|idx| matches!(&tokens[idx], EnglishToken::Word(w) if w.len() == 1));
    Some(CapsGroup {
        first_cap: first_cap?,
        last_cap: last_cap?,
        caps_sequences,
        has_lower: has_lower || single_letter_sentence,
        single_letter_only,
        ended_with_terminal_sentence_mark: terminal_punctuation
            && matches!(
                last_cap.and_then(|idx| tokens.get(idx)),
                Some(EnglishToken::Symbol('.' | '!' | '?'))
            ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::bold_digit_symbol("𝟖 chickens!", "⠘⠆⠼⠓⠀⠡⠊⠉⠅⠢⠎⠖")]
    #[case::bold_question_symbol("For help, click the 𝐀 icon?", "⠠⠿⠀⠓⠑⠇⠏⠂⠀⠉⠇⠊⠉⠅⠀⠮⠀⠘⠆⠠⠁⠀⠊⠉⠕⠝⠦")]
    #[case::script_y_letter("𝒴ou can do it!", "⠈⠆⠠⠽⠀⠉⠀⠙⠀⠭⠖")]
    #[case::script_letterlike_r("ℜ", "⠈⠆⠰⠠⠗")]
    #[case::small_caps_roman("ᴠɪɪɪ", "⠠⠠⠧⠊⠊⠊")]
    #[case::underlined_question_symbol("?\u{0332} icon", "⠸⠆⠰⠦⠀⠊⠉⠕⠝")]
    #[case::bold_italic_word("𝒕𝒔𝒖𝒏𝒂𝒎𝒊.", "⠘⠂⠨⠂⠞⠎⠥⠝⠁⠍⠊⠲")]
    fn encodes_rule9_typeform_extents(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §2.6/§10: standing-alone boundaries and typeform extents are semantic
    /// contexts, not literal word exceptions. These examples lock the failing
    /// seams where hyphen/dash/punctuation/typeform changes decide whether a
    /// wordsign, groupsign, or shortform may be used.

    #[rstest::rstest]
    #[case::italic_math_alpha("\u{1D45D}neumonia", "⠨⠆⠏⠝⠑⠥⠍⠕⠝⠊⠁")]
    #[case::bold_math_alpha("\u{1D41B}at", "⠘⠆⠃⠁⠞")]
    #[case::underline_combining("story\u{0332}", "⠌⠕⠗⠸⠆⠽")]
    #[case::italic_initial_wordsign("\u{1D440}other", "⠨⠆⠠⠐⠍")]
    #[case::bold_groupsign_start("mo\u{1D42D}her", "⠍⠕⠘⠆⠮⠗")]
    #[case::script_letterlike_r("\u{211C}", "⠈⠆⠰⠠⠗")]
    #[case::numeric_adjacent_italic_letter("31\u{1D459}", "⠼⠉⠁⠇")]
    fn typeform_symbol_indicator_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9.x: a run of 2+ styled letters takes a word indicator (`⠨⠂`) with the
    /// word contracted normally (`𝑅𝑎𝑑𝑎𝑟` → `⠨⠂⠠⠗⠁⠙⠜`, `ar` groupsign); a partial
    /// run ending mid-word adds a terminator (`𝐭𝐞𝐱𝐭book` → `⠘⠂⠞⠑⠭⠞⠘⠄…`).

    #[rstest::rstest]
    #[case::italic_whole_word("the \u{1D445}\u{1D44E}\u{1D451}\u{1D44E}\u{1D45F}", "⠮⠀⠨⠂⠠⠗⠁⠙⠜")]
    #[case::bold_partial_then_plain("\u{1D42D}\u{1D41E}\u{1D431}\u{1D42D}book", "⠘⠂⠞⠑⠭⠞⠘⠄⠃⠕⠕⠅")]
    #[case::bold_bracketed_symbol_sequence("\u{1D40D}(\u{1D446})", "⠘⠂⠠⠝⠐⠣⠨⠆⠠⠎⠐⠜")]
    fn typeform_word_indicator_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §13.2: a foreign phrase identified by typography/foreign context is written
    /// uncontracted under its typeform passage indicator, so English words inside
    /// the expression such as `en` do not use UEB contractions.

    #[rstest::rstest]
    #[case::underline_digit_run("3\u{0332}4\u{0332}", "⠸⠆⠼⠉⠙")]
    #[case::bold_digit_after_plain("5\u{1D7D3}", "⠼⠑⠘⠆⠼⠑")]
    #[case::underline_period_between_digits("27.\u{0332}9", "⠼⠃⠛⠸⠆⠲⠼⠊")]
    #[case::underline_percent("83%\u{0332}", "⠼⠓⠉⠸⠆⠨⠴")]
    fn typeform_styled_digits_symbols_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9 + §5.7.1: a single styled letter standing alone (§2.6) takes a grade-1
    /// indicator after its typeform symbol indicator when it is an alphabetic
    /// wordsign (`𝑦` italic → `⠨⠆⠰⠽`), but not for the a/i/o letters which carry no
    /// wordsign (`𝑖` → `⠨⠆⠊`).

    #[rstest::rstest]
    #[case::underline_word_then_slash_word("a\u{0332}n\u{0332}d\u{0332}/or", "⠸⠂⠯⠸⠄⠸⠌⠕⠗")]
    fn typeform_word_terminator_continues_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §9.x: a typeform *passage* keeps a trailing full stop inside its span, but a
    /// trailing dash separates it from following text, so the terminator falls
    /// after the stop and *before* the dash (`…𝑒𝑓.—` → `…⠑⠋⠲⠨⠄⠠⠤`).

    #[rstest::rstest]
    #[case::hyphen_joined_of_the(
        "out-\u{1D45C}\u{1D453}-\u{1D461}\u{210E}\u{1D452}-way",
        "⠳⠤⠨⠂⠷⠤⠮⠨⠄⠤⠺⠁⠽"
    )]
    // §10.12.2: the trailing lone wordsign letter `z` in running text takes grade-1 ⠰.
    #[case::apostrophe_single_first_segment("\u{1D459}'\u{1D45C} z", "⠨⠂⠇⠄⠕⠀⠰⠵")]
    fn typeform_multi_segment_word_9(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.12.12: punctuation, indicators, or terminators printed inside a word do
    /// not block the basic §10 groupsigns; the indicator encloses just the printed
    /// styled segment and terminates before any following plain letters.

    #[rstest::rstest]
    #[case::italic_th_medial("ra\u{1D461}\u{210E}er", "⠗⠁⠨⠂⠹⠨⠄⠻")]
    #[case::bold_ch_medial("tou\u{1D41C}\u{1D421}ed", "⠞⠳⠘⠂⠡⠘⠄⠫")]
    #[case::italic_ing_final("flow\u{1D456}\u{1D45B}\u{1D454}", "⠋⠇⠪⠨⠂⠬")]
    #[case::underlined_gh_final("enoug\u{0332}h\u{0332}", "⠢⠳⠸⠂⠣")]
    fn medial_typeform_keeps_groupsigns_10_12_12(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §10.12.3: embedded web addresses remain contracted, so dot-delimited words
    /// such as `one` and `in` can use their ordinary UEB contractions.

    #[test]
    fn styled_form_at_reports_typeform_only_for_styled_tokens() {
        use super::super::super::token::Typeform;
        assert_eq!(
            styled_form_at(&[EnglishToken::Styled('a', Typeform::Bold)], 0),
            Some(Typeform::Bold)
        );
        assert_eq!(styled_form_at(&[EnglishToken::Space], 0), None);
    }

    #[test]
    fn encodes_styled_lone_digit_as_typeform_symbol() {
        // §9/§11: a lone styled non-letter (bold digit `𝟏` = U+1D7CF) takes the
        // §9 typeform *symbol* indicator followed by the numeric symbol, not the
        // word indicator path used for styled letters.
        let mut expected = super::super::super::rule_9::symbol_indicator(
            super::super::super::token::Typeform::Bold,
        );
        encode_styled_nonword_symbol('1', &mut expected).unwrap();
        assert_eq!(enc("\u{1D7CF}").unwrap(), expected);
    }
}
