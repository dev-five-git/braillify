use super::*;

pub(super) fn foreign_en_spells_letters(
    prev: Option<&EnglishToken>,
    next: Option<&EnglishToken>,
) -> bool {
    matches!(
        prev,
        None | Some(EnglishToken::Space | EnglishToken::Symbol('-'))
    ) && matches!(
        next,
        None | Some(EnglishToken::Space | EnglishToken::Symbol('-'))
    )
}

pub(super) fn styled_word_count(tokens: &[EnglishToken]) -> usize {
    let mut count = 0usize;
    let mut in_word = false;
    for token in tokens {
        match token {
            EnglishToken::Styled(..) if !in_word => {
                count += 1;
                in_word = true;
            }
            EnglishToken::Styled(..) | EnglishToken::Symbol('-' | '\u{2019}' | '\'') => {}
            _ => in_word = false,
        }
    }
    count
}

pub(super) fn all_text_is_styled_or_punctuation(tokens: &[EnglishToken]) -> bool {
    tokens.iter().all(|token| {
        matches!(
            token,
            EnglishToken::Styled(..)
                | EnglishToken::Space
                | EnglishToken::Symbol(_)
                | EnglishToken::LineBreak
        )
    })
}

pub(super) fn styled_word_is_foreign(chars: &[char]) -> bool {
    if chars.iter().any(|c| {
        super::super::rule_13::is_foreign_letter(*c) && !super::super::rule_4::is_accented(*c)
    }) {
        return true;
    }
    let word: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    // §10.12.12: typeform does not block a contraction when the styled letters
    // themselves form a normal UEB groupsign (`tou𝐜𝐡ed`, `enoug̲h̲`). These short
    // digraphs are not CMUdict words, so keep them on the ordinary UEB path before
    // the foreign-word fallback below.
    if matches!(word.as_str(), "ch" | "gh" | "sh" | "th" | "wh") {
        return false;
    }
    if super::super::rule_10_1::wordsign(&word).is_some()
        || super::super::rule_10_2::wordsign(&word).is_some()
        || super::super::rule_10_5::wordsign(&word).is_some()
        || super::super::rule_10_9::whole_word_cells(&word).is_some()
    {
        return false;
    }
    if starts_with_ch_not_pronounced_ch(&word) {
        return true;
    }
    word.len() > 1 && !super::super::pronunciation::cmudict::is_recorded_word(&word)
}

/// §13: explicit foreign-script/pronunciation evidence in a single styled word.
/// Unknown ASCII vocabulary alone is not enough here: §9 typeform examples include
/// technical English words and URLs that still use ordinary UEB contractions.
pub(super) fn styled_word_has_foreign_signal(chars: &[char]) -> bool {
    if chars.iter().any(|c| {
        super::super::rule_13::is_foreign_letter(*c) && !super::super::rule_4::is_accented(*c)
    }) {
        return true;
    }
    let word: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    starts_with_ch_not_pronounced_ch(&word)
}

/// §13.2.1: whether a single styled word is foreign because it is not a
/// recorded English word AND is not itself a UEB wordsign/shortform. The
/// italic typeform per §13.1.2 marks the word as foreign; §13.2.1 then
/// suppresses contractions inside the styled span. Short digraphs
/// (`ch`/`gh`/`sh`/`th`/`wh`) which are themselves UEB groupsigns are
/// exempted so a styled emphatic digraph (`tou𝐜𝐡ed`) keeps its contraction.
pub(super) fn styled_single_word_is_foreign(chars: &[char]) -> bool {
    if styled_word_has_foreign_signal(chars) {
        return true;
    }
    let word: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    // A digraph groupsign (`ch`/`gh`/`sh`/`th`/`wh`) is 2 chars, so it is already
    // rejected by the `< 3` guard above — no separate digraph check is needed.
    if word.chars().count() < 3 {
        return false;
    }
    if super::super::rule_10_1::wordsign(&word).is_some()
        || super::super::rule_10_2::wordsign(&word).is_some()
        || super::super::rule_10_5::wordsign(&word).is_some()
        || super::super::rule_10_9::whole_word_cells(&word).is_some()
    {
        return false;
    }
    !super::super::pronunciation::cmudict::is_recorded_word(&word)
}

pub(super) fn starts_with_ch_not_pronounced_ch(word: &str) -> bool {
    if !word.starts_with("ch") {
        return false;
    }
    let provider = super::super::pronunciation::cmudict::CmuDictProvider::new();
    let pronunciations =
        super::super::pronunciation::PronunciationProvider::pronunciations(&provider, word);
    !pronunciations.is_empty()
        && pronunciations
            .iter()
            .all(|pronunciation| pronunciation.iter().all(|phoneme| phoneme.base != "CH"))
}

pub(super) fn styled_words_are_titlecase(words: &[Vec<char>]) -> bool {
    words.iter().all(|word| {
        word.first().is_some_and(|c| c.is_uppercase())
            && word
                .iter()
                .skip(1)
                .all(|c| !c.is_alphabetic() || c.is_lowercase())
    })
}

pub(super) fn styled_phrase_from_named_place(tokens: &[EnglishToken], phrase_end: usize) -> bool {
    if followed_by_from_named_place(tokens, phrase_end) {
        return true;
    }
    let mut k = phrase_end;
    if !matches!(tokens.get(k), Some(EnglishToken::Space))
        || !matches!(tokens.get(k + 1), Some(EnglishToken::Word(word)) if word.iter().collect::<String>().eq_ignore_ascii_case("and"))
        || !matches!(tokens.get(k + 2), Some(EnglishToken::Space))
    {
        return false;
    }
    k += 3;
    let mut saw_styled_title = false;
    loop {
        let mut saw_styled_word = false;
        while matches!(tokens.get(k), Some(EnglishToken::Styled(..))) {
            saw_styled_word = true;
            saw_styled_title = true;
            k += 1;
        }
        if !saw_styled_word {
            break;
        }
        if matches!(tokens.get(k), Some(EnglishToken::Space))
            && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(..)))
        {
            k += 1;
        } else {
            break;
        }
    }
    saw_styled_title && followed_by_from_named_place(tokens, k)
}

pub(super) fn followed_by_from_named_place(tokens: &[EnglishToken], phrase_end: usize) -> bool {
    matches!(tokens.get(phrase_end), Some(EnglishToken::Space))
        && matches!(tokens.get(phrase_end + 1), Some(EnglishToken::Word(word)) if word.iter().collect::<String>().eq_ignore_ascii_case("from"))
        && matches!(tokens.get(phrase_end + 2), Some(EnglishToken::Space))
        && matches!(tokens.get(phrase_end + 3), Some(EnglishToken::Word(word)) if word.first().is_some_and(|c| c.is_uppercase()))
}

pub(super) fn followed_by_word(tokens: &[EnglishToken], phrase_end: usize, expected: &str) -> bool {
    matches!(tokens.get(phrase_end), Some(EnglishToken::Space))
        && matches!(tokens.get(phrase_end + 1), Some(EnglishToken::Word(word)) if word.iter().collect::<String>().eq_ignore_ascii_case(expected))
}

pub(super) fn styled_words_are_lowercase(words: &[Vec<char>]) -> bool {
    words
        .iter()
        .all(|word| word.iter().all(|c| !c.is_alphabetic() || c.is_lowercase()))
}

pub(super) fn styled_words_spell(words: &[Vec<char>], index: usize, expected: &str) -> bool {
    words.get(index).is_some_and(|word| {
        word.iter()
            .flat_map(|c| c.to_lowercase())
            .eq(expected.chars())
    })
}

pub(super) fn styled_words_are_english_title(words: &[Vec<char>]) -> bool {
    words.len() >= 4
        && styled_words_spell(words, 0, "the")
        && words
            .iter()
            .enumerate()
            .skip(1)
            .take(words.len().saturating_sub(2))
            .any(|(index, _)| styled_words_spell(words, index, "of"))
}

pub(super) fn styled_word_in_english_title(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
) -> bool {
    let mut start = i;
    while start >= 2
        && matches!(tokens.get(start - 1), Some(EnglishToken::Space))
        && matches!(tokens.get(start - 2), Some(EnglishToken::Styled(_, f)) if *f == form)
    {
        start -= 2;
        while start > 0
            && matches!(tokens.get(start - 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            start -= 1;
        }
    }

    let mut words = Vec::new();
    let mut k = start;
    while k < tokens.len() {
        let mut word = Vec::new();
        while matches!(tokens.get(k), Some(EnglishToken::Styled(_, f)) if *f == form) {
            if let Some(EnglishToken::Styled(c, _)) = tokens.get(k) {
                word.push(*c);
            }
            k += 1;
        }
        if !word.is_empty() {
            words.push(word);
        }
        if matches!(tokens.get(k), Some(EnglishToken::Space))
            && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            k += 1;
        } else {
            break;
        }
    }
    styled_words_are_english_title(&words)
}

pub(super) fn styled_passage_ends_with_unrecorded_word(
    tokens: &[EnglishToken],
    start: usize,
    end: usize,
    form: super::super::token::Typeform,
) -> bool {
    let mut k = start;
    let mut last_word = Vec::new();
    while k < end {
        let mut word = Vec::new();
        while matches!(tokens.get(k), Some(EnglishToken::Styled(_, f)) if *f == form) {
            if let Some(EnglishToken::Styled(c, _)) = tokens.get(k) {
                word.push(*c);
            }
            k += 1;
        }
        if !word.is_empty() {
            last_word = word;
        }
        k += 1;
    }
    let word: String = last_word.iter().flat_map(|c| c.to_lowercase()).collect();
    word.len() >= 4 && !super::super::pronunciation::cmudict::is_recorded_word(&word)
}

pub(super) fn styled_titlecase_phrase_from_named_place(tokens: &[EnglishToken], i: usize) -> bool {
    let mut start = i;
    while start >= 2 && matches!(tokens.get(start - 1), Some(EnglishToken::Space)) {
        let mut previous = start - 2;
        if !matches!(tokens.get(previous), Some(EnglishToken::Styled(..))) {
            break;
        }
        while previous > 0 && matches!(tokens.get(previous - 1), Some(EnglishToken::Styled(..))) {
            previous -= 1;
        }
        start = previous;
    }

    let mut words = Vec::new();
    let mut k = start;
    while k < tokens.len() {
        let mut word = Vec::new();
        while matches!(tokens.get(k), Some(EnglishToken::Styled(..))) {
            if let Some(EnglishToken::Styled(c, _)) = tokens.get(k) {
                word.push(*c);
            }
            k += 1;
        }
        if !word.is_empty() {
            words.push(word);
        }
        if matches!(tokens.get(k), Some(EnglishToken::Space))
            && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(..)))
        {
            k += 1;
        } else {
            break;
        }
    }

    words.len() >= 2
        && styled_words_are_titlecase(&words)
        && styled_phrase_from_named_place(tokens, k)
}

pub(super) fn styled_word_in_lowercase_phrase_before_word(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
    expected: &str,
) -> bool {
    let mut start = i;
    while start >= 2
        && matches!(tokens.get(start - 1), Some(EnglishToken::Space))
        && matches!(tokens.get(start - 2), Some(EnglishToken::Styled(_, f)) if *f == form)
    {
        start -= 2;
        while start > 0
            && matches!(tokens.get(start - 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            start -= 1;
        }
    }

    let mut words = Vec::new();
    let mut k = start;
    while k < tokens.len() {
        let word_len = tokens[k..]
            .iter()
            .take_while(|token| matches!(token, EnglishToken::Styled(_, f) if *f == form))
            .count();
        let word = tokens[k..k + word_len]
            .iter()
            .filter_map(token_base_char)
            .collect::<Vec<_>>();
        k += word_len;
        if !word.is_empty() {
            words.push(word);
        }
        if matches!(tokens.get(k), Some(EnglishToken::Space))
            && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            k += 1;
        } else {
            break;
        }
    }

    words.len() >= 2 && styled_words_are_lowercase(&words) && followed_by_word(tokens, k, expected)
}

pub(super) fn styled_prose_double_space(tokens: &[EnglishToken], i: usize) -> bool {
    let has_typeform = tokens.iter().any(|t| matches!(t, EnglishToken::Styled(..)));
    let prev = i.checked_sub(1).and_then(|p| tokens.get(p));
    let sentence_end = matches!(
        prev,
        Some(EnglishToken::Symbol(
            '.' | '!' | '?' | '"' | '\u{201D}' | '\u{2019}'
        ))
    );
    // §9.7.2/§9.4.3: a printed double-space after `:` that introduces a styled
    // passage or quote (`word:  Maybe`, `dictum:  Pecunia`) is typography — it
    // collapses to the ordinary one-cell braille space so the passage indicator
    // sits directly after the colon-cell.
    let colon_before_passage = matches!(prev, Some(EnglishToken::Symbol(':')))
        && (styled_passage_starts_at_double_space(tokens, i)
            || colon_introduces_later_styled_text(tokens, i));
    let url_before_prose = styled_url_before(tokens, i)
        && matches!(tokens.get(i + 1), Some(EnglishToken::Space))
        && matches!(tokens.get(i + 2), Some(EnglishToken::Word(_)));
    !has_typeform || sentence_end || colon_before_passage || url_before_prose
}

pub(super) fn colon_introduces_later_styled_text(tokens: &[EnglishToken], i: usize) -> bool {
    let mut k = i;
    while matches!(
        tokens.get(k),
        Some(EnglishToken::Space | EnglishToken::LineBreak)
    ) {
        k += 1;
    }
    while let Some(token) = tokens.get(k) {
        match token {
            EnglishToken::Styled(..) => return true,
            EnglishToken::Symbol('.' | '!' | '?') => return false,
            _ => k += 1,
        }
    }
    false
}

pub(super) fn styled_passage_starts_at_double_space(tokens: &[EnglishToken], i: usize) -> bool {
    let mut k = i;
    while matches!(
        tokens.get(k),
        Some(EnglishToken::Space | EnglishToken::LineBreak)
    ) {
        k += 1;
    }
    matches!(tokens.get(k), Some(EnglishToken::Styled(..)))
}

pub(super) fn styled_passage_introduced_by_colon(tokens: &[EnglishToken], start: usize) -> bool {
    let mut k = start;
    while k > 0 {
        k -= 1;
        match tokens.get(k) {
            Some(EnglishToken::Space) => continue,
            Some(EnglishToken::Symbol(':')) => return true,
            _ => return false,
        }
    }
    false
}

pub(super) fn styled_phrase_foreign_scope(
    tokens: &[EnglishToken],
    i: usize,
    form: super::super::token::Typeform,
) -> Option<(super::super::rule_13::AccentCode, bool)> {
    let mut start = i;
    while start >= 2
        && matches!(tokens.get(start - 1), Some(EnglishToken::Space))
        && matches!(tokens.get(start - 2), Some(EnglishToken::Styled(_, f)) if *f == form)
    {
        start -= 2;
        while start > 0
            && matches!(tokens.get(start - 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            start -= 1;
        }
    }

    let mut words = Vec::new();
    let mut k = start;
    while k < tokens.len() {
        let mut word = Vec::new();
        while matches!(tokens.get(k), Some(EnglishToken::Styled(_, f)) if *f == form) {
            if let Some(EnglishToken::Styled(c, _)) = tokens.get(k) {
                word.push(*c);
            }
            k += 1;
        }
        if !word.is_empty() {
            words.push(word);
        }
        if matches!(tokens.get(k), Some(EnglishToken::Space))
            && matches!(tokens.get(k + 1), Some(EnglishToken::Styled(_, f)) if *f == form)
        {
            k += 1;
        } else {
            break;
        }
    }
    let phrase_end = k;

    let doc_letters = document_letters(tokens);
    let spanish = super::super::rule_13::spanish_context(&doc_letters);
    if styled_phrase_count(tokens) >= 2
        && !bibliography_entry_context(tokens)
        && super::super::rule_13::has_foreign_code_signal(&doc_letters)
        && words.iter().any(|w| {
            w.iter()
                .any(|c| super::super::rule_13::is_foreign_letter(*c))
        })
    {
        return Some((super::super::rule_13::AccentCode::Foreign, spanish));
    }

    // §13.7.2: when print typography (e.g. bold) identifies foreign vocabulary
    // and 2+ separate styled phrases appear with at least one carrying a foreign
    // accent letter (`𝐪𝐮𝐞́`, `𝐯��𝐲𝐚`), treat every styled phrase as foreign so
    // both `qué` and `vaya` take foreign-code accents even though `vaya` alone
    // has no accent evidence. Requires all styled phrases to be short foreign
    // vocabulary (all lowercase, ≤6 chars) — this excludes English titles and
    // proper-name runs from over-triggering. Absence of French-specific accents
    // (è, ê, ë, ç, à, ù) makes the passage Spanish (`⠮` for é) by default.
    if styled_phrase_count(tokens) >= 2
        && document_any_styled_phrase_has_foreign_letter(tokens)
        && document_all_styled_phrases_are_short_vocabulary(tokens)
    {
        let doc_has_french_accent = doc_letters
            .iter()
            .any(|c| matches!(c, 'è' | 'ê' | 'ë' | 'ç' | 'à' | 'ù'));
        return Some((
            super::super::rule_13::AccentCode::Foreign,
            spanish || !doc_has_french_accent,
        ));
    }

    if words.len() >= 2
        && !words.iter().any(|word| {
            let lower: String = word.iter().flat_map(|c| c.to_lowercase()).collect();
            super::super::rule_10_9::whole_word_cells(&lower).is_some()
        })
        && styled_phrase_from_named_place(tokens, phrase_end)
    {
        return Some((super::super::rule_13::AccentCode::Ueb, spanish));
    }

    if words.len() >= 2
        && styled_words_are_lowercase(&words)
        && followed_by_word(tokens, phrase_end, "of")
    {
        return None;
    }

    if styled_words_are_english_title(&words)
        || words.len() < 2
        || !words.iter().any(|w| styled_word_is_foreign(w))
    {
        return None;
    }
    Some((
        if super::super::rule_13::has_foreign_code_signal(&doc_letters)
            && !bibliography_entry_context(tokens)
        {
            super::super::rule_13::AccentCode::Foreign
        } else {
            super::super::rule_13::AccentCode::Ueb
        },
        spanish,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::italic_foreign_passage_en(
        "We ate \u{1D45D}\u{1D44E}\u{1D45D} \u{1D452}\u{1D45B} \u{1D464}\u{1D45C}\u{1D45F}\u{1D460}.",
        "⠠⠺⠑⠀⠁⠞⠑⠀⠨⠶⠏⠁⠏⠀⠑⠝⠀⠺⠕⠗⠎⠨⠄⠲"
    )]
    #[case::italic_japanese_newspaper_title(
        "\u{1D44C}\u{1D45C}\u{1D45A}\u{1D456}\u{1D462}\u{1D45F}\u{1D456} \u{1D446}\u{210E}\u{1D456}\u{1D45A}\u{1D44F}\u{1D462}\u{1D45B} from Japan",
        "⠨⠂⠠⠽⠕⠍⠊⠥⠗⠊⠀⠨⠂⠠⠎⠓⠊⠍⠃⠥⠝⠀⠋⠀⠠⠚⠁⠏⠁⠝"
    )]
    #[case::italic_japanese_newspaper_serial_title(
        "\u{1D44C}\u{1D45C}\u{1D45A}\u{1D456}\u{1D462}\u{1D45F}\u{1D456} \u{1D446}\u{210E}\u{1D456}\u{1D45A}\u{1D44F}\u{1D462}\u{1D45B} and \u{1D434}\u{1D460}\u{1D44E}\u{210E}\u{1D456} \u{1D446}\u{210E}\u{1D456}\u{1D45A}\u{1D44F}\u{1D462}\u{1D45B} from Japan",
        "⠨⠂⠠⠽⠕⠍⠊⠥⠗⠊⠀⠨⠂⠠⠎⠓⠊⠍⠃⠥⠝⠀⠯⠀⠨⠂⠠⠁⠎⠁⠓⠊⠀⠨⠂⠠⠎⠓⠊⠍⠃⠥⠝⠀⠋⠀⠠⠚⠁⠏⠁⠝"
    )]
    #[case::italic_anglicised_english_title_with_foreign_name(
        "\u{1D447}\u{210E}\u{1D452} \u{1D447}\u{1D44E}\u{1D459}\u{1D452} \u{1D45C}\u{1D453} \u{1D43A}\u{1D452}\u{1D45B}\u{1D457}\u{1D456}.",
        "⠨⠶⠠⠮⠀⠠⠞⠁⠇⠑⠀⠷⠀⠠⠛⠢⠚⠊⠨⠄⠲"
    )]
    #[case::bold_span_with_inverted_question(
        "\u{1D42D}\u{1D42E}\u{0301} \u{1D425}\u{1D41E} \u{1D41D}\u{1D422}\u{1D42C}\u{1D42D}\u{1D41E} \u{1D41E}\u{1D425} \u{00BF}\u{1D42F}\u{1D41E}\u{1D42B}\u{1D41D}\u{1D41A}\u{1D41D}?",
        "⠘⠶⠞⠾⠀⠇⠑⠀⠙⠊⠎⠞⠑⠀⠑⠇⠀⠢⠧⠑⠗⠙⠁⠙⠢⠘⠄"
    )]
    #[case::italic_span_with_inverted_exclamation_keeps_ueb_signs(
        "—¡\u{1D444}\u{1D462}\u{1D452}\u{0301} \u{1D456}\u{1D451}\u{1D452}\u{1D44E} \u{1D45A}\u{1D44E}\u{0301}\u{1D460} \u{1D44F}\u{1D462}\u{1D452}\u{1D45B}\u{1D44E}!—exclaimed Pedro's mother.",
        "⠠⠤⠨⠶⠘⠰⠖⠠⠟⠥⠘⠌⠑⠀⠊⠙⠑⠁⠀⠍⠘⠌⠁⠎⠀⠃⠥⠑⠝⠁⠖⠨⠄⠠⠤⠑⠭⠉⠇⠁⠊⠍⠫⠀⠠⠏⠫⠗⠕⠄⠎⠀⠐⠍⠲"
    )]
    #[case::lowercase_phrase_before_of_keeps_leading_word_contracted(
        "We went out for a \u{1D459}\u{1D452}\u{1D458}\u{1D458}\u{1D452}\u{1D45F} \u{1D44F}\u{1D45F}\u{1D44E}\u{1D44E}\u{1D456} of \u{1D45D}\u{1D44E}\u{1D45D} \u{1D452}\u{1D45B} \u{1D464}\u{1D45C}\u{1D45F}\u{1D460}.",
        "⠠⠺⠑⠀⠺⠢⠞⠀⠳⠀⠿⠀⠁⠀⠨⠂⠇⠑⠅⠅⠻⠀⠨⠂⠃⠗⠁⠁⠊⠀⠷⠀⠨⠶⠏⠁⠏⠀⠑⠝⠀⠺⠕⠗⠎⠨⠄⠲"
    )]
    #[case::quoted_french_phrase_uncontracted_13_2_1(
        "\"\u{1D443}\u{1D45F}\u{1D452}\u{1D45B}\u{1D45C}\u{1D45B}\u{1D460} \u{1D450}\u{1D45C}\u{1D462}\u{1D45F}\u{1D44E}\u{1D454}\u{1D452}, \u{1D440}\u{1D44E}\u{1D45F}\u{1D454}\u{1D462}\u{1D452}\u{1D45F}\u{1D456}\u{1D461}\u{1D452},\" Jeanne said",
        "⠦⠨⠶⠠⠏⠗⠑⠝⠕⠝⠎⠀⠉⠕⠥⠗⠁⠛⠑⠂⠀⠠⠍⠁⠗⠛⠥⠑⠗⠊⠞⠑⠂⠨⠄⠴⠀⠠⠚⠂⠝⠝⠑⠀⠎⠙"
    )]
    fn foreign_typeform_words_are_uncontracted_13_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §13.1.3 bibliography entries keep English UEB contraction decisions in
    /// author/publisher metadata while typeform-marked foreign titles are scoped
    /// as titles and use UEB §4.2 accent modifiers, not §14 foreign-code accents.

    #[test]
    fn encode_styled_span_uses_provided_foreign_scope() {
        use super::super::super::token::Typeform;
        // §13.2.1: when the caller supplies a foreign scope, the span's segments
        // are encoded uncontracted under that accent code.
        let tokens = [EnglishToken::Styled('a', Typeform::Italic)];
        let ctx = StyledContext {
            tokens: &tokens,
            suppress_caps: false,
            foreign_scope: Some((super::super::super::rule_13::AccentCode::Ueb, false)),
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_styled_span(0, 1, Typeform::Italic, ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }

    #[test]
    fn separate_spanish_vocabulary_phrases_use_foreign_accents() {
        let italic = super::super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Styled('q', italic),
            EnglishToken::Styled('u', italic),
            EnglishToken::Styled('é', italic),
            EnglishToken::Space,
            EnglishToken::Word(vec!['m', 'e', 'a', 'n', 's']),
            EnglishToken::Space,
            EnglishToken::Styled('v', italic),
            EnglishToken::Styled('a', italic),
            EnglishToken::Styled('y', italic),
            EnglishToken::Styled('á', italic),
        ];

        assert_eq!(
            styled_phrase_foreign_scope(&tokens, 0, italic),
            Some((super::super::super::rule_13::AccentCode::Foreign, true))
        );
    }
}
