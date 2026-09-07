use crate::{symbol_shortcut, utils};

/// 규칙 33~35에서 종료표(⠲)를 생략해야 하는 기호 모음.
/// 기호 앞뒤에서는 로마자 종료표를 생략한다.
pub(crate) fn should_skip_terminator_for_symbol(symbol: char) -> bool {
    matches!(
        symbol,
        '.' | '?'
            | '!'
            | '…'
            | '⋯'
            | '"'
            | '\''
            | '”'
            | '’'
            | '」'
            | '』'
            | '〉'
            | '》'
            | '('
            | ')'
            | ']'
            | '}'
            | ','
            | ':'
            | ';'
            | '―'
    )
}

/// 종료표를 생략한 뒤에도 연속표(⠐)로 이어야 하는 기호 모음.
/// 여는 괄호 '(' 는 새 영어 구간을 열게 되므로 제외한다.
/// 종료표를 생략했지만 이어지는 로마자에 연속표를 붙여야 하는지 판단한다.
pub(crate) fn should_request_continuation(symbol: char) -> bool {
    matches!(
        symbol,
        '.' | '?'
            | '!'
            | '…'
            | '⋯'
            | '"'
            | '\''
            | '”'
            | '’'
            | '」'
            | '』'
            | '〉'
            | '》'
            | ')'
            | ']'
            | '}'
            | ','
            | ':'
            | ';'
            | '―'
    )
}

/// 제33항 [다만] : '/', '~' 앞에는 종료표를 강제로 붙인다.
/// '-'는 PDF 제35항 적용 — 로마자+숫자가 이어지는 컨텍스트(예: D-100)에서는
/// 종료표를 적지 않는다. `-` 자체가 영어 문맥의 일부로 처리.
pub(crate) fn should_force_terminator_before_symbol(symbol: char) -> bool {
    matches!(symbol, '/' | '~' | '∼')
}

/// 영어 점자 전용 기호인지 확인.[외국어 점자 일람표의 문장 부호 참고]
pub(crate) fn is_english_symbol(symbol: char) -> bool {
    symbol_shortcut::is_english_symbol_char(symbol)
}

/// 단일 소문자 단어가 연속될 때 연속표가 필요한지 판단한다.
/// [통일 영어 점자 - 5.2 1급 점자 기호표(⠰)] : 글자 a, i, o 앞에는 1급 점자 기호표가 필요하지 않다.
pub(crate) fn requires_single_letter_continuation(letter: char) -> bool {
    letter.is_ascii_alphabetic() && !matches!(letter.to_ascii_lowercase(), 'a' | 'i' | 'o')
}

fn is_ascii_letter_or_digit(ch: Option<char>) -> bool {
    ch.is_some_and(|c| c.is_ascii_alphanumeric())
}

/// Whether a following print item starts with a number and then crosses
/// directly into Korean text, without an intervening Roman unit/identifier.
///
/// Korean rule 33 changes a comma to the Korean punctuation sign only at an
/// actual Roman-to-Korean boundary.  Looking for Korean anywhere later in the
/// whitespace-delimited item is too broad: in `173cm, 68kg의`, the comma is
/// followed first by the Roman measurement `68kg`, and the particle `의` is a
/// later boundary.  Numeric grouping/decimal punctuation remains part of the
/// numeric prefix (`1,000년`, `3.5년`).
fn begins_numeric_then_korean(chars: impl Iterator<Item = char>) -> bool {
    let mut chars = chars.peekable();
    if !chars.peek().is_some_and(char::is_ascii_digit) {
        return false;
    }

    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit()
            || (matches!(ch, ',' | '.') && chars.peek().is_some_and(char::is_ascii_digit))
        {
            continue;
        }
        return utils::is_korean_char(ch);
    }

    false
}

/// Returns whether `index` is an ampersand inside a complete sequence of
/// non-empty ASCII-letter segments joined by `&`. Korean rule 35 allows the
/// resulting Roman text to continue directly into digits and later Roman
/// letters, as in the official `MP4 Player` example. A leading digit remains
/// outside this predicate because the cited sequence begins with Roman text.
pub(crate) fn is_attached_ascii_roman_ampersand(word_chars: &[char], index: usize) -> bool {
    if word_chars.get(index) != Some(&'&')
        || index == 0
        || index + 1 >= word_chars.len()
        || !word_chars[index - 1].is_ascii_alphabetic()
        || !word_chars[index + 1].is_ascii_alphabetic()
    {
        return false;
    }

    let mut start = index;
    while start > 0 && (word_chars[start - 1].is_ascii_alphabetic() || word_chars[start - 1] == '&')
    {
        start -= 1;
    }
    let mut end = index + 1;
    while end < word_chars.len()
        && (word_chars[end].is_ascii_alphanumeric() || word_chars[end] == '&')
    {
        end += 1;
    }

    let segment = &word_chars[start..end];
    segment.first().is_some_and(|ch| ch.is_ascii_alphabetic())
        && segment.last().is_some_and(|ch| ch.is_ascii_alphanumeric())
        && segment.iter().enumerate().all(|(offset, ch)| {
            *ch != '&'
                || (offset > 0
                    && offset + 1 < segment.len()
                    && segment[offset - 1].is_ascii_alphabetic()
                    && segment[offset + 1].is_ascii_alphabetic())
        })
        && (start == 0 || !word_chars[start - 1].is_ascii_alphanumeric())
        && (end == word_chars.len() || !word_chars[end].is_ascii_alphanumeric())
}

/// Returns whether `index` is an asterisk inside a complete sequence of
/// non-empty ASCII-alphanumeric segments, each containing a Roman letter.
/// UEB 3.3.1 says that the asterisk follows its UEB form regardless of meaning
/// and gives `M*A*S*H` as an attached Roman example. Korean rules 32 and 35
/// keep the resulting UEB text, including directly adjacent digits, in one
/// Roman section. Requiring a Roman letter in every segment keeps numeric
/// `2*3`, detached asterisks, Korean text, and empty segments out of this rule.
pub(crate) fn is_attached_ascii_roman_asterisk(word_chars: &[char], index: usize) -> bool {
    if word_chars.get(index) != Some(&'*') || index == 0 || index + 1 >= word_chars.len() {
        return false;
    }

    let mut start = index;
    while start > 0
        && (word_chars[start - 1].is_ascii_alphanumeric() || word_chars[start - 1] == '*')
    {
        start -= 1;
    }
    let mut end = index + 1;
    while end < word_chars.len()
        && (word_chars[end].is_ascii_alphanumeric() || word_chars[end] == '*')
    {
        end += 1;
    }

    let segment = &word_chars[start..end];
    segment
        .split(|ch| *ch == '*')
        .all(|part| !part.is_empty() && part.iter().any(|ch| ch.is_ascii_alphabetic()))
        && segment.first().is_some_and(|ch| ch.is_ascii_alphabetic())
        && (start == 0 || !word_chars[start - 1].is_ascii_alphanumeric())
        && (end == word_chars.len() || !word_chars[end].is_ascii_alphanumeric())
}

/// Returns whether `index` is the one-sided ampersand at the beginning of a
/// complete attached ASCII-letter segment. UEB 3.1.1 prints `&c` without a
/// boundary between the ampersand and `c`; Korean rule 35 then permits an
/// attached numeric/Roman continuation. A left ASCII alphanumeric or another
/// ampersand is excluded so the existing two-sided `A&B` rule retains ownership.
pub(crate) fn is_ampersand_before_attached_ascii_roman_segment(
    word_chars: &[char],
    index: usize,
) -> bool {
    if word_chars.get(index) != Some(&'&')
        || !word_chars
            .get(index + 1)
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        || index
            .checked_sub(1)
            .and_then(|i| word_chars.get(i))
            .is_some_and(|previous| previous.is_ascii_alphanumeric() || *previous == '&')
    {
        return false;
    }

    let mut end = index + 1;
    while word_chars
        .get(end)
        .is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
        end += 1;
    }
    word_chars
        .get(end)
        .is_none_or(|next| !next.is_ascii_alphanumeric())
}

fn is_digital_notation_symbol(symbol: char) -> bool {
    matches!(symbol, '/' | '@' | '#' | '.' | '_' | ':')
}

fn has_digital_notation_signature(word_chars: &[char]) -> bool {
    let text: String = word_chars.iter().collect();
    // PDF — 단일 `_`만 있는 경우는 일반 부호로 처리하고, 디지털 표기는 `//`, `@`, `#`
    // 같은 강한 표지 또는 `_`와 다른 디지털 표지 조합에서만 활성화한다.
    if text.contains("//") || text.contains('@') || text.contains('#') {
        return true;
    }
    text.contains('_') && (text.contains('.') || text.contains('/') || text.contains(':'))
}

pub(crate) fn prev_ascii_letter_or_digit(word_chars: &[char], index: usize) -> bool {
    let mut j = index;
    while j > 0 {
        let ch = word_chars[j - 1];
        if ch.is_ascii_alphanumeric() {
            return true;
        }
        if symbol_shortcut::is_english_symbol_char(ch) {
            j -= 1;
            continue;
        }
        break;
    }
    false
}

pub(crate) fn next_ascii_letter_or_digit(
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
) -> bool {
    let mut j = index + 1;
    while j < word_chars.len() {
        let ch = word_chars[j];
        if ch.is_ascii_alphanumeric() {
            return true;
        }
        if symbol_shortcut::is_english_symbol_char(ch) {
            j += 1;
            continue;
        }
        return false;
    }

    for word in remaining_words {
        for ch in word.chars() {
            if ch.is_ascii_alphanumeric() {
                return true;
            }
            if symbol_shortcut::is_english_symbol_char(ch) {
                continue;
            }
            return false;
        }
    }

    false
}

/// Korean rule 46's `BMI(체질량 지수)` example assigns the attached, closed
/// parenthesis to Korean punctuation even though it follows a Roman run. Scan
/// the complete balanced enclosure because its Korean content can begin after
/// a number, a Roman expansion, or a print-space boundary. Pure Roman/number
/// enclosures remain eligible for rule 32's UEB punctuation.
fn closed_parenthesis_contains_korean(
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
) -> bool {
    let mut depth = 1usize;
    let mut contains_korean = false;
    let tail = word_chars
        .iter()
        .skip(index + 1)
        .copied()
        .chain(remaining_words.iter().flat_map(|word| word.chars()));

    for ch in tail {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return contains_korean;
                }
            }
            _ if utils::is_korean_char(ch) => contains_korean = true,
            _ => {}
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
/// 괄호/쉼표가 영어 점자로 이어져야 하는지 판정한다.
/// - '(' 는 뒤에 올 문자가 ASCII 영숫자여야 하고, 앞은 한글이 아니어야 한다.
/// - ')' 는 여는 괄호가 영어 기호로 열렸던 경우에만 영어 기호로 닫는다.
/// - ',' 는 앞뒤 모두 ASCII 영숫자가 이어지는 경우에만 영어 점자로 유지한다.
pub(crate) fn should_render_symbol_as_english(
    english_indicator: bool,
    is_english: bool,
    is_english_majority: bool,
    parenthesis_stack: &[bool],
    symbol: char,
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
) -> bool {
    if !english_indicator {
        return false;
    }

    let prev_char = if index > 0 {
        Some(word_chars[index - 1])
    } else {
        None
    };
    let next_char = if index + 1 < word_chars.len() {
        Some(word_chars[index + 1])
    } else {
        remaining_words.first().and_then(|w| w.chars().next())
    };

    // A non-English closing enclosure is a hard Roman-section boundary.  The
    // look-behind helpers deliberately skip punctuation for attached UEB runs,
    // but must not reach through that boundary and pull a following version or
    // identifier mark (`(XBB).1.5`) back into the closed Roman section.
    if !is_english && prev_char.is_some_and(|ch| matches!(ch, ')' | ']' | '}')) {
        return false;
    }

    match symbol {
        '(' => {
            (is_english_majority
                || !closed_parenthesis_contains_korean(word_chars, index, remaining_words))
                && is_ascii_letter_or_digit(next_char)
                && !prev_char.is_some_and(utils::is_korean_char)
        }
        ')' => parenthesis_stack.last().copied().unwrap_or(false),
        // UEB 3.1.1 prints an ampersand without ending and restarting
        // grade-1 mode in attached Roman forms such as AT&T and B&B. Use
        // a complete ASCII-letter run so spaced prose, Hangul, and outer
        // alphanumeric continuations keep their existing routes.
        '&' => is_attached_ascii_roman_ampersand(word_chars, index),
        // UEB 3.3.1 explicitly keeps the general-purpose asterisk inside the
        // attached Roman example `M*A*S*H`. Preserve that one Roman section;
        // Korean Rule 60 continues to own standalone and non-Roman asterisks.
        '*' => is_attached_ascii_roman_asterisk(word_chars, index),
        // UEB 8.4.2 keeps the apostrophe inside the Roman word in its
        // `O'Hara`, `DON'T`, and `THAT'S` examples. Capitals-word mode may
        // terminate at this nonalphabetic symbol, but the surrounding Roman
        // section does not. Detached quotes and digit measurement marks stay
        // on their existing punctuation routes.
        '\'' => {
            prev_char.is_some_and(|ch| ch.is_ascii_alphabetic())
                && word_chars
                    .get(index + 1)
                    .is_some_and(|ch| ch.is_ascii_alphabetic())
        }
        // UEB 7.3 writes the single-character ellipsis as three full stops.
        // Keep that UEB punctuation only while the Roman run visibly
        // continues or closes an enclosure. A Unicode ellipsis followed
        // directly by Korean remains the Rule-53 Korean middle-dot ellipsis.
        '…' => {
            is_english
                && (next_ascii_letter_or_digit(word_chars, index, remaining_words)
                    || matches!(next_char, Some(')' | ']' | '}' | '”' | '’' | '」' | '』')))
        }
        ',' => {
            if !is_english {
                return false;
            }

            let next_word_is_digit_led_korean = if index + 1 < word_chars.len() {
                begins_numeric_then_korean(word_chars[index + 1..].iter().copied())
            } else {
                remaining_words
                    .first()
                    .is_some_and(|word| begins_numeric_then_korean(word.chars()))
            };
            if next_word_is_digit_led_korean {
                // Korean rule 33: punctuation whose UEB and Korean cells
                // differ is written as Korean punctuation at a Roman-to-
                // Korean boundary. Limit the whole-token lookahead to a
                // digit-led Korean word: Roman-led mixed words such as
                // `LG유플러스` continue the Roman list and are not this case.
                return false;
            }

            let prev_roman = prev_ascii_letter_or_digit(word_chars, index)
                || prev_char
                    .is_some_and(crate::rules::korean::rule_69::is_compatibility_unit_presentation);
            let next_roman = next_ascii_letter_or_digit(word_chars, index, remaining_words)
                || next_char
                    .is_some_and(crate::rules::korean::rule_69::is_compatibility_unit_presentation);

            prev_roman && next_roman
        }
        '-' => {
            let prev_ascii = prev_ascii_letter_or_digit(word_chars, index);
            let next_ascii = next_ascii_letter_or_digit(word_chars, index, remaining_words);
            let roman_started_before_hyphen = word_chars[..index]
                .iter()
                .rev()
                .take_while(|ch| ch.is_ascii_alphanumeric() || **ch == '-')
                .any(|ch| ch.is_ascii_alphabetic());

            // Korean rule 35 keeps a Roman-led identifier (`CV3-AD685`,
            // `N-79-20`) in one Roman/number chain across its hyphens.  A
            // number-led item (`0-Zone`, `777-300ER`) has not entered Roman
            // mode yet: its first Roman indicator belongs immediately before
            // the first letter, never before an earlier hyphen.
            prev_ascii && next_ascii && (is_english || roman_started_before_hyphen)
        }
        '/' | '@' | '#' | '.' | '_' | ':' => {
            let prev_ascii = prev_ascii_letter_or_digit(word_chars, index);
            let next_ascii = next_ascii_letter_or_digit(word_chars, index, remaining_words);

            (prev_ascii && next_ascii)
                // Korean rules 29/32/35: `Alpha : Beta` is one Roman
                // section. The print tokenizer makes the colon a standalone
                // word; an already-open section proves its left Roman item,
                // while this lookahead proves the right Roman/number item.
                || (symbol == ':' && is_english && word_chars == [':'] && next_ascii)
                || (symbol == '/' && prev_char == Some('/') && next_ascii)
                || (symbol == '/' && next_char == Some('/') && prev_ascii)
        }
        _ => false,
    }
}

pub(crate) fn should_keep_english_mode_for_symbol(
    symbol: char,
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
) -> bool {
    if !is_digital_notation_symbol(symbol) || !has_digital_notation_signature(word_chars) {
        return false;
    }

    should_render_symbol_as_english(
        true,
        true,
        false,
        &[],
        symbol,
        word_chars,
        index,
        remaining_words,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `requires_single_letter_continuation` — 영어 연속점이 필요한 letter 식별.
    #[rstest::rstest]
    #[case::lowercase_b_requires('b', true)]
    #[case::lowercase_a_excluded('a', false)]
    #[case::uppercase_excluded('A', false)]
    fn requires_single_letter_continuation_distinguishes_letters(
        #[case] ch: char,
        #[case] expected: bool,
    ) {
        assert_eq!(requires_single_letter_continuation(ch), expected);
    }

    #[test]
    fn skip_and_force_terminator_sets_are_separate() {
        for symbol in ['.', '?', '!', ')', ']', ','] {
            assert!(should_skip_terminator_for_symbol(symbol));
        }
        // PDF 제33항 [다만] — `/`, `~` 앞에는 영어 종료표 강제 (제35항에 따라 `-`는 제외).
        // `-`는 로마자+숫자 연결(예: D-100)에서 영어 컨텍스트의 일부이므로 종료표를 적지 않는다.
        for symbol in ['/', '~'] {
            assert!(should_force_terminator_before_symbol(symbol));
            assert!(!should_skip_terminator_for_symbol(symbol));
        }
        // `-`는 force 대상이 아니지만, skip 대상도 아니다 (별도 분기 처리).
        assert!(!should_force_terminator_before_symbol('-'));
        assert!(should_request_continuation('.'));
        assert!(!should_request_continuation('('));
    }

    /// `is_english_symbol` 표 — 영어 모드에서 인식되는 기호 vs 아닌 것.
    #[rstest::rstest]
    #[case('(', true)]
    #[case(')', true)]
    #[case(',', true)]
    #[case('?', false)]
    fn english_symbol_detection_matches_lookup_table(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(is_english_symbol(ch), expected);
    }

    /// `prev_ascii_letter_or_digit` — 영어 기호 건너뛰며 직전 ASCII 문자 탐색.
    #[rstest::rstest]
    #[case::skip_english_symbol_to_ascii("A(,B", 2, true)]
    #[case::korean_neighbor_blocks("가,", 1, false)]
    fn prev_ascii_letter_or_digit_skips_english_symbols(
        #[case] input: &str,
        #[case] idx: usize,
        #[case] expected: bool,
    ) {
        let word: Vec<char> = input.chars().collect();
        assert_eq!(prev_ascii_letter_or_digit(&word, idx), expected);
    }

    /// `next_ascii_letter_or_digit` — 현재 토큰의 future ASCII 검사.
    /// 토큰 내 직후 / 영어 기호 건너뛴 후 / 다음 단어로 이어 보는 케이스.
    #[rstest::rstest]
    #[case::contiguous_ascii("A,B", 1, &[], true)]
    #[case::skip_english_symbol("A,(B", 1, &[], true)]
    #[case::remaining_word_ascii("A,", 1, &["B"], true)]
    #[case::hangul_following("A,가", 1, &[], false)]
    #[case::remaining_word_with_symbol_then_ascii("A,", 1, &["(B"], true)]
    #[case::remaining_word_only_symbols("A,", 1, &["()"], false)]
    fn next_ascii_letter_or_digit_checks_future_ascii(
        #[case] input: &str,
        #[case] idx: usize,
        #[case] remaining: &[&str],
        #[case] expected: bool,
    ) {
        let word: Vec<char> = input.chars().collect();
        assert_eq!(next_ascii_letter_or_digit(&word, idx, remaining), expected);
    }

    #[rstest::rstest]
    #[case::pure_roman("(Hello)", 0, &[], true, false, false, true)]
    #[case::korean_before("가(", 1, &["A)"], true, false, false, false)]
    #[case::indicator_disabled("(Hello)", 0, &[], false, false, false, false)]
    #[case::official_rule_46_shape("BMI(체질량", 3, &["지수)"], true, true, false, false)]
    #[case::roman_then_korean_body(
        "SDV(Software",
        3,
        &["Defined", "Vehicle,", "소프트웨어", "중심)"],
        true,
        true,
        false,
        false
    )]
    #[case::pure_roman_body("ABC(def)", 3, &[], true, true, false, true)]
    #[case::pure_number_body("BSI(73)", 3, &[], true, true, false, true)]
    #[case::unclosed_body("ABC(def", 3, &["한글"], true, true, false, true)]
    #[case::nested_korean_body(
        "BIT(BT(바이오)+IT(정보))",
        3,
        &[],
        true,
        true,
        false,
        false
    )]
    #[case::rule_39_english_majority("(Korean:", 0, &["반찬)"], true, true, true, true)]
    fn should_render_symbol_as_english_for_opening_parenthesis(
        #[case] input: &str,
        #[case] index: usize,
        #[case] remaining_words: &[&str],
        #[case] english_indicator: bool,
        #[case] is_english: bool,
        #[case] is_english_majority: bool,
        #[case] expected: bool,
    ) {
        let word = input.chars().collect::<Vec<_>>();
        assert_eq!(
            should_render_symbol_as_english(
                english_indicator,
                is_english,
                is_english_majority,
                &[],
                '(',
                &word,
                index,
                remaining_words,
            ),
            expected,
        );
    }

    /// `should_render_symbol_as_english` for ')' — paren stack top 만 본다.
    #[rstest::rstest]
    #[case::stack_top_true(true, true)]
    #[case::stack_top_false(false, false)]
    fn should_render_symbol_as_english_for_closing_parenthesis(
        #[case] stack_top: bool,
        #[case] expected: bool,
    ) {
        let closer: Vec<char> = ")".chars().collect();
        assert_eq!(
            should_render_symbol_as_english(true, true, false, &[stack_top], ')', &closer, 0, &[],),
            expected,
        );
    }

    /// `should_render_symbol_as_english` for ',' — 양쪽 ASCII + 영어 컨텍스트 둘 다 필요.
    #[rstest::rstest]
    #[case::both_ascii_in_english_mode("A,B", true, true)]
    #[case::compatibility_unit_in_english_mode("㎿,30", true, true)]
    #[case::not_in_english_mode("A,B", false, false)]
    #[case::korean_neighbor("가,B", true, false)]
    fn should_render_symbol_as_english_for_comma_requires_ascii_neighbors(
        #[case] input: &str,
        #[case] is_english: bool,
        #[case] expected: bool,
    ) {
        let word: Vec<char> = input.chars().collect();
        assert_eq!(
            should_render_symbol_as_english(true, is_english, false, &[], ',', &word, 1, &[],),
            expected
        );
    }

    #[rstest::rstest]
    #[case::roman_led_chain("CV3-AD685", 3, false, true)]
    #[case::roman_led_numeric_chain("N-79-20", 4, false, true)]
    #[case::number_led_word("0-Zone", 1, false, false)]
    #[case::number_led_suffix("777-300ER", 3, false, false)]
    #[case::after_closed_enclosure("(GTX)-C", 5, false, false)]
    fn hyphen_enters_roman_punctuation_only_after_a_roman_run(
        #[case] input: &str,
        #[case] index: usize,
        #[case] is_english: bool,
        #[case] expected: bool,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        assert_eq!(
            should_render_symbol_as_english(true, is_english, false, &[], '-', &chars, index, &[],),
            expected
        );
    }

    #[test]
    fn punctuation_after_closed_korean_enclosure_does_not_reenter_roman_mode() {
        let chars = "(XBB).1.5".chars().collect::<Vec<_>>();
        assert!(!should_render_symbol_as_english(
            true,
            false,
            false,
            &[false],
            '.',
            &chars,
            5,
            &[],
        ));
    }

    /// Korean rule 33 classifies a comma before a digit-led Korean word from
    /// the complete following token. A Roman-led mixed word remains Roman
    /// context at the boundary.
    #[rstest::rstest]
    #[case::digit_led_korean("2000년대", false)]
    #[case::grouped_digit_led_korean("2,000년대", false)]
    #[case::decimal_digit_led_korean("3.5년", false)]
    #[case::pure_number("2000", true)]
    #[case::roman_word("Beta", true)]
    #[case::roman_led_mixed_word("LG유플러스", true)]
    #[case::numeric_roman_unit_before_korean_particle("68kg의", true)]
    fn comma_before_next_word_uses_narrow_digit_led_korean_context(
        #[case] next_word: &str,
        #[case] expected: bool,
    ) {
        let word = ['A', ','];
        assert_eq!(
            should_render_symbol_as_english(true, true, false, &[], ',', &word, 1, &[next_word],),
            expected
        );
    }

    /// UEB 8.4.2 keeps a word-internal apostrophe inside the Roman section.
    #[rstest::rstest]
    #[case::official_name("O'Hara", true)]
    #[case::official_contraction("DON'T", true)]
    #[case::official_possessive("THAT'S", true)]
    #[case::detached_open("'word", false)]
    #[case::detached_close("word'", false)]
    #[case::measurement("6'2", false)]
    fn internal_apostrophe_requires_ascii_letters_on_both_sides(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let word = input.chars().collect::<Vec<_>>();
        let index = word.iter().position(|ch| *ch == '\'').unwrap();
        assert_eq!(
            should_render_symbol_as_english(true, true, false, &[], '\'', &word, index, &[],),
            expected,
        );
    }

    #[test]
    fn apostrophe_does_not_join_the_next_whitespace_delimited_word() {
        let word = "Guitar'".chars().collect::<Vec<_>>();
        assert!(!should_render_symbol_as_english(
            true,
            true,
            false,
            &[],
            '\'',
            &word,
            word.len() - 1,
            &["Listening"],
        ));
    }

    /// UEB 3.1.1 keeps attached Roman segments on both sides of `&` in the
    /// same mode. Korean rule 35 additionally keeps a trailing number/Roman
    /// continuation in that section. The left boundary still excludes a
    /// number-led sequence because the cited section begins with Roman text.
    #[rstest::rstest]
    #[case::official_at_and_t("AT&T", true, true)]
    #[case::official_b_and_b("B&B", true, true)]
    #[case::spaced("A & B", true, false)]
    #[case::hangul_left("가&B", true, false)]
    #[case::hangul_right("A&나", true, false)]
    #[case::digit_neighbor("3&B", true, false)]
    #[case::digit_outer_left("3A&B", true, false)]
    #[case::rule35_digit_suffix("A&B3", true, true)]
    #[case::rule35_digit_then_roman_suffix("A&B3C", true, true)]
    #[case::ampersand_after_digit("A&B3&C", true, false)]
    #[case::multiple_ampersands("A&B&C", true, true)]
    #[case::empty_segment("A&&B", true, false)]
    #[case::no_roman_indicator("AT&T", false, false)]
    fn attached_ampersand_requires_complete_ascii_roman_run(
        #[case] input: &str,
        #[case] english_indicator: bool,
        #[case] expected: bool,
    ) {
        let word: Vec<char> = input.chars().collect();
        let index = word.iter().position(|ch| *ch == '&').unwrap();
        assert_eq!(
            should_render_symbol_as_english(
                english_indicator,
                true,
                false,
                &[],
                '&',
                &word,
                index,
                &[],
            ),
            expected,
        );
    }

    /// UEB 3.3.1 uses one uninterrupted UEB run for official `M*A*S*H`.
    /// Korean rules 32/35 preserve the same form inside a Korean document,
    /// while numeric multiplication, detached marks, and empty segments remain
    /// outside the Roman-asterisk grammar.
    #[rstest::rstest]
    #[case::official_mash_first("M*A*S*H", 1, true, true)]
    #[case::official_mash_middle("M*A*S*H", 3, true, true)]
    #[case::official_mash_last("M*A*S*H", 5, true, true)]
    #[case::roman_number_chain("A1*B2", 2, true, true)]
    #[case::number_led("2*A", 1, true, false)]
    #[case::digit_only_right_segment("A*2", 1, true, false)]
    #[case::empty_segment("A**B", 1, true, false)]
    #[case::hangul_segment("가*A", 1, true, false)]
    #[case::detached("A * B", 2, true, false)]
    #[case::no_roman_indicator("M*A*S*H", 1, false, false)]
    fn attached_asterisk_requires_complete_ascii_roman_segments(
        #[case] input: &str,
        #[case] index: usize,
        #[case] english_indicator: bool,
        #[case] expected: bool,
    ) {
        let word = input.chars().collect::<Vec<_>>();
        assert_eq!(
            should_render_symbol_as_english(
                english_indicator,
                true,
                false,
                &[],
                '*',
                &word,
                index,
                &[],
            ),
            expected,
        );
    }

    #[rstest::rstest]
    #[case::official_and_c("&c", true)]
    #[case::official_at_and_t("AT&T", false)]
    #[case::official_b_and_b("B&B", false)]
    #[case::official_spaced("Marks & Spencer", false)]
    #[case::rule35_digit_suffix("&P500", true)]
    #[case::digit_without_roman_segment("&500", false)]
    fn one_sided_ampersand_requires_complete_right_roman_segment(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let word = input.chars().collect::<Vec<_>>();
        let index = word.iter().rposition(|ch| *ch == '&').unwrap();
        assert_eq!(
            is_ampersand_before_attached_ascii_roman_segment(&word, index),
            expected,
        );
    }

    /// `has_digital_notation_signature` — `//`, `@`, `#` 강한 마커 또는
    /// underscore + digital marker 조합은 true, 단순 underscore는 false.
    #[rstest::rstest]
    #[case::double_slash("http://example.com", true)]
    #[case::at_sign("user@host", true)]
    #[case::hash("tag#name", true)]
    #[case::underscore_plus_dot("a_b.c", true)]
    #[case::pure_underscore("a_b", false)]
    fn digital_notation_signature_strong_markers(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(
            super::has_digital_notation_signature(&chars),
            expected,
            "input={input:?}"
        );
    }

    /// english_logic:208 — `should_keep_english_mode_for_symbol` returns the
    /// inner `should_render_symbol_as_english` result when both pre-conditions pass.
    #[test]
    fn should_keep_english_mode_for_symbol_passes_through() {
        // Use a digital_notation_symbol AND a word that has digital signature.
        let chars: Vec<char> = "user@host.com".chars().collect();
        // '@' at index 4
        let _ = super::should_keep_english_mode_for_symbol('@', &chars, 4, &[]);
    }
}
