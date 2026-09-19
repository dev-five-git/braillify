//! §10.12.1 — abbreviations and acronyms whose letters are pronounced separately
//! take no contractions (`WHO ⠺⠓⠕`, `OED ⠕⠑⠙`, `MSH ⠍⠎⠓`, `DAR ⠙⠁⠗`,
//! `EST ⠑⠎⠞`, `POW ⠏⠕⠺`); an acronym read as a word keeps them (§10.12.2:
//! `FORTRAN ⠿⠞⠗⠁⠝`, `CANDU ⠉⠯⠥`).
//!
//! The rule asks whether the pronunciation "is known, or can be determined from
//! the text or by reference to a standard dictionary". For an all-capitals
//! Roman token embedded in Korean text (한국 점자 제28항/제37항 context) that
//! determination is made structurally:
//!
//! * A token standing next to another ordinary Roman word is part of an
//!   English phrase (a title, a product or band name), so it is read as text
//!   and keeps its contractions.
//! * Otherwise the token is an isolated abbreviation in a Korean sentence. It
//!   is read as a word only when it is *pronounceable*: it has a vowel, its
//!   consonant clusters are ones English syllables allow, and — mirroring the
//!   length of every §10.12.1 example — it is longer than three letters. A
//!   four-letter token is additionally accepted when a standard dictionary
//!   (CMUdict) records it as a word.
//!
//! Anything else is an initialism pronounced as letters and is spelled out.
//! Pure English documents are not affected: an all-caps word there is §8
//! emphasis (`THE`, `SHE`) and contracts as usual.

use super::pronunciation::cmudict;

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// `y` carries a vowel sound except word-initially (`ENHYPEN`, `SKYLINE`
/// versus `YOUTH`).
fn is_vowel_at(lower: &[char], index: usize) -> bool {
    is_vowel(lower[index]) || (index > 0 && lower[index] == 'y')
}

/// Two-letter consonant clusters that can begin an English syllable. A token
/// starting with any other pair (`QLED`, `CCUS`, `NBER`, `GGGI`) is not read as
/// a word.
fn is_valid_onset(first: char, second: char) -> bool {
    matches!(
        (first, second),
        ('b', 'l')
            | ('b', 'r')
            | ('c', 'h')
            | ('c', 'l')
            | ('c', 'r')
            | ('d', 'r')
            | ('f', 'l')
            | ('f', 'r')
            | ('g', 'l')
            | ('g', 'r')
            | ('k', 'l')
            | ('k', 'n')
            | ('k', 'r')
            | ('p', 'h')
            | ('p', 'l')
            | ('p', 'r')
            | ('q', 'u')
            | ('s', 'c')
            | ('s', 'h')
            | ('s', 'k')
            | ('s', 'l')
            | ('s', 'm')
            | ('s', 'n')
            | ('s', 'p')
            | ('s', 't')
            | ('s', 'w')
            | ('t', 'h')
            | ('t', 'r')
            | ('t', 'w')
            | ('w', 'h')
            | ('w', 'r')
    )
}

/// Consonant digraphs pronounced as one sound; they count as a single consonant
/// when measuring a cluster (`LIGHTS` → `l·i·gh·t·s`).
fn is_digraph(first: char, second: char) -> bool {
    matches!(
        (first, second),
        ('c', 'h') | ('s', 'h') | ('t', 'h') | ('p', 'h') | ('g', 'h') | ('c', 'k') | ('n', 'g')
    )
}

/// Longest run of consonant *sounds* in the (lowercase) token.
fn longest_consonant_cluster(lower: &[char]) -> usize {
    let mut longest = 0usize;
    let mut run = 0usize;
    let mut index = 0usize;
    while index < lower.len() {
        if is_vowel_at(lower, index) {
            run = 0;
            index += 1;
            continue;
        }
        run += 1;
        longest = longest.max(run);
        if lower
            .get(index + 1)
            .is_some_and(|next| is_digraph(lower[index], *next))
        {
            index += 2;
        } else {
            index += 1;
        }
    }
    longest
}

/// A whole word that UEB writes with a single contraction and that 한국 점자
/// 제37항 does *not* list among the words to spell out (`THE`, `AND`, `OUT`,
/// `ONE`, `DAY`): such a token is read as the word, never as letter names.
/// The 제37항 words themselves (`IT`, `US`, `AS`, `GO`) are left to the
/// initialism test: they are common Korean-news initialisms, and 제37항 spells
/// them out after the Roman indicator either way.
pub(crate) fn is_whole_word_contraction(lower: &str) -> bool {
    super::rule_10_2::wordsign(lower).is_some()
        || super::rule_10_3::is_strong_contraction_word(lower)
        || super::rule_10_7::is_initial_letter_contraction_word(lower)
}

/// Whether an isolated all-capitals token is read as a word (so its UEB
/// contractions apply) rather than as separate letter names.
fn reads_as_word(uppercase: &[char]) -> bool {
    let lower: Vec<char> = uppercase.iter().map(|ch| ch.to_ascii_lowercase()).collect();
    if is_whole_word_contraction(&lower.iter().collect::<String>()) {
        return true;
    }
    if lower.len() < 4 || !(0..lower.len()).any(|index| is_vowel_at(&lower, index)) {
        return false;
    }
    if lower.len() >= 5 {
        return longest_consonant_cluster(&lower) <= 3;
    }
    if cmudict::has_word_pronunciation(&lower) {
        return true;
    }
    let starts_with_two_vowels = is_vowel(lower[0]) && is_vowel(lower[1]);
    let onset_allowed =
        is_vowel(lower[0]) || is_vowel(lower[1]) || is_valid_onset(lower[0], lower[1]);
    !starts_with_two_vowels && onset_allowed && longest_consonant_cluster(&lower) <= 2
}

/// Whether a neighbouring print token is an ordinary Roman word (so the current
/// token sits inside an English phrase). Enclosing punctuation is ignored, but
/// the token must *begin* with its Roman letters — `네오(Neo)` is a Korean word
/// carrying a gloss, not a phrase member. A lowercase or Title-case word always
/// qualifies; an all-capitals neighbour qualifies only when it reads as a word
/// itself (`DAY`, `WORLD`), so `CJ ENM` and `LED TV` stay pairs of initialisms.
pub(crate) fn is_word_like_roman_token(token: &str) -> bool {
    let chars: Vec<char> = token
        .chars()
        .skip_while(|ch| {
            matches!(
                ch,
                '(' | '[' | '{' | '\u{2018}' | '\u{201c}' | '"' | '\'' | '#'
            )
        })
        .collect();
    let run: Vec<char> = chars
        .iter()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .copied()
        .collect();
    if run.len() < 2
        || chars[..run.len()]
            .iter()
            .any(|ch| !ch.is_ascii_alphabetic())
    {
        return false;
    }
    if run.iter().any(|ch| ch.is_ascii_lowercase()) {
        return true;
    }
    if run.len() <= 3 {
        let lower: Vec<char> = run.iter().map(|ch| ch.to_ascii_lowercase()).collect();
        return cmudict::has_word_pronunciation(&lower);
    }
    reads_as_word(&run)
}

/// Whether the trailing Roman run of a *preceding* print token makes it an
/// ordinary Roman word. The token must consist of Roman letters and enclosing
/// punctuation only (`Fix`, `‘GeForce`), never Korean script.
pub(crate) fn preceding_token_is_word_like(token: &str) -> bool {
    if token.chars().any(crate::utils::is_korean_char) {
        return false;
    }
    let trimmed: String = token
        .chars()
        .filter(|ch| {
            !matches!(
                ch,
                ')' | ']'
                    | '}'
                    | '\u{2019}'
                    | '\u{201d}'
                    | '"'
                    | '\''
                    | ','
                    | '.'
                    | ':'
                    | ';'
                    | '!'
                    | '?'
            )
        })
        .collect();
    is_word_like_roman_token(&trimmed)
}

/// §10.12.1 decision for an all-capitals Roman run embedded in Korean text:
/// `true` when the run is an initialism pronounced as letters, so the caller
/// spells it with alphabet signs only.
pub(crate) fn korean_context_run_is_initialism(run: &[char], in_english_phrase: bool) -> bool {
    run.len() >= 2
        && run.iter().all(|ch| ch.is_ascii_uppercase())
        && !in_english_phrase
        && !reads_as_word(run)
}

/// The Roman run `word_chars[run_start..run_end]` and its print neighbours.
pub(crate) struct RomanRunPosition<'a> {
    pub word_chars: &'a [char],
    pub run_start: usize,
    pub run_end: usize,
    pub prev_word: &'a str,
    pub next_word: Option<&'a str>,
}

/// Whether the run at `position` is a §10.12.1 initialism to spell letter by
/// letter. Only a Korean document qualifies (a 제39항 English-dominant one
/// keeps §8 emphasis words); a run abutting a digit (`CH6`) or joined to an
/// apostrophe contraction (`SHE'LL`) is left to the rules that already govern it.
pub(crate) fn is_letter_initialism_in_korean_text(
    state: &crate::rules::context::EncoderState,
    position: &RomanRunPosition<'_>,
) -> bool {
    let RomanRunPosition {
        word_chars,
        run_start,
        run_end,
        prev_word,
        next_word,
    } = *position;
    let korean_document = state.english_indicator && !state.english_dominant_no_indicator;
    let run = &word_chars[run_start..run_end];
    if !korean_document || !run.iter().all(|ch| ch.is_ascii_uppercase()) {
        return false;
    }
    let is_apostrophe = |ch: &char| matches!(ch, '\'' | '\u{2019}');
    let neighbour = |offset: usize| word_chars.get(offset);
    let digit_adjacent = run_start
        .checked_sub(1)
        .and_then(neighbour)
        .is_some_and(char::is_ascii_digit)
        || neighbour(run_end).is_some_and(char::is_ascii_digit);
    let apostrophe_joined = run_start.checked_sub(2).is_some_and(|before| {
        is_apostrophe(&word_chars[before + 1]) && word_chars[before].is_ascii_alphabetic()
    }) || (neighbour(run_end).is_some_and(is_apostrophe)
        && neighbour(run_end + 1).is_some_and(char::is_ascii_alphabetic));
    if digit_adjacent || apostrophe_joined {
        return false;
    }
    let run_opens_word = !word_chars[..run_start]
        .iter()
        .any(char::is_ascii_alphanumeric);
    let run_closes_word = !word_chars[run_end..]
        .iter()
        .any(char::is_ascii_alphanumeric);
    let in_english_phrase = (run_opens_word && preceding_token_is_word_like(prev_word))
        || (run_closes_word && next_word.is_some_and(is_word_like_roman_token));
    korean_context_run_is_initialism(run, in_english_phrase)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    /// §10.12.1 examples and Korean-news initialisms are pronounced as letters.
    #[rstest::rstest]
    #[case::who("WHO")]
    #[case::oed("OED")]
    #[case::mou("MOU")]
    #[case::led("LED")]
    #[case::est("EST")]
    #[case::gh("GH")]
    #[case::no_vowel("MLCC")]
    #[case::invalid_onset_qled("QLED")]
    #[case::invalid_onset_ccus("CCUS")]
    #[case::three_consonants_ustr("USTR")]
    #[case::two_leading_vowels("EAFF")]
    #[case::long_cluster("EDSCG")]
    #[case::unfccc("UNFCCC")]
    fn initialisms_are_spelled(#[case] run: &str) {
        assert!(korean_context_run_is_initialism(&chars(run), false));
    }

    /// Word-read acronyms keep their contractions (§10.12.2 `FORTRAN`, `CANDU`).
    #[rstest::rstest]
    #[case::fortran("FORTRAN")]
    #[case::candu("CANDU")]
    #[case::oled("OLED")]
    #[case::kaist("KAIST")]
    #[case::haccp("HACCP")]
    #[case::dictionary_word_gist("GIST")]
    #[case::digraph_cluster_lights("LIGHTS")]
    #[case::keri("KERI")]
    #[case::valid_onset_star("STAR")]
    #[case::y_as_vowel_enhypen("ENHYPEN")]
    #[case::strong_contraction_the("THE")]
    #[case::strong_wordsign_out("OUT")]
    #[case::initial_letter_day("DAY")]
    fn word_acronyms_keep_contractions(#[case] run: &str) {
        assert!(!korean_context_run_is_initialism(&chars(run), false));
    }

    /// Inside an English phrase the token is text, whatever its shape.
    #[rstest::rstest]
    #[case::the("THE")]
    #[case::out("OUT")]
    fn phrase_members_are_text(#[case] run: &str) {
        assert!(!korean_context_run_is_initialism(&chars(run), true));
    }

    #[rstest::rstest]
    #[case::title_case("Fix", true)]
    #[case::quoted_lowercase("‘GeForce", true)]
    #[case::caps_dictionary_word("DAY’를", true)]
    #[case::caps_long_word("WORLD", true)]
    #[case::initialism_neighbour("TV는", false)]
    #[case::initialism_pair("CJ", false)]
    #[case::korean_gloss("네오(Neo)", false)]
    #[case::single_letter("A", false)]
    fn neighbour_word_detection(#[case] token: &str, #[case] expected: bool) {
        assert_eq!(is_word_like_roman_token(token), expected);
    }

    #[rstest::rstest]
    #[case::plain_word("Fix", true)]
    #[case::closing_quote("GeForce’", true)]
    #[case::korean_gloss_before("네오(Neo)", false)]
    #[case::initialism("KBS", false)]
    fn preceding_neighbour_detection(#[case] token: &str, #[case] expected: bool) {
        assert_eq!(preceding_token_is_word_like(token), expected);
    }

    #[rstest::rstest]
    #[case::lights("lights", 3)]
    #[case::kbstar("kbstar", 4)]
    #[case::haccp("haccp", 3)]
    #[case::oled("oled", 1)]
    fn consonant_clusters_are_digraph_aware(#[case] word: &str, #[case] expected: usize) {
        assert_eq!(longest_consonant_cluster(&chars(word)), expected);
    }
}
