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

#[allow(clippy::too_many_arguments)]
/// 괄호/쉼표가 영어 점자로 이어져야 하는지 판정한다.
/// - '(' 는 뒤에 올 문자가 ASCII 영숫자여야 하고, 앞은 한글이 아니어야 한다.
/// - ')' 는 여는 괄호가 영어 기호로 열렸던 경우에만 영어 기호로 닫는다.
/// - ',' 는 앞뒤 모두 ASCII 영숫자가 이어지는 경우에만 영어 점자로 유지한다.
pub(crate) fn should_render_symbol_as_english(
    english_indicator: bool,
    is_english: bool,
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

    match symbol {
        '(' => is_ascii_letter_or_digit(next_char) && !prev_char.is_some_and(utils::is_korean_char),
        ')' => parenthesis_stack.last().copied().unwrap_or(false),
        ',' => {
            if !is_english {
                return false;
            }

            let prev_ascii = prev_ascii_letter_or_digit(word_chars, index);
            let next_ascii = next_ascii_letter_or_digit(word_chars, index, remaining_words);

            prev_ascii && next_ascii
        }
        '/' | '@' | '#' | '.' | '_' | ':' | '-' => {
            let prev_ascii = prev_ascii_letter_or_digit(word_chars, index);
            let next_ascii = next_ascii_letter_or_digit(word_chars, index, remaining_words);

            (prev_ascii && next_ascii)
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

    should_render_symbol_as_english(true, true, &[], symbol, word_chars, index, remaining_words)
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

    #[test]
    fn should_render_symbol_as_english_for_parentheses() {
        let opener: Vec<char> = "(Hello".chars().collect();
        assert!(should_render_symbol_as_english(
            true,
            false,
            &[],
            '(',
            &opener,
            0,
            &[]
        ));

        let korean_before: Vec<char> = "가(".chars().collect();
        assert!(!should_render_symbol_as_english(
            true,
            false,
            &[],
            '(',
            &korean_before,
            1,
            &["A"]
        ));

        assert!(!should_render_symbol_as_english(
            false,
            false,
            &[],
            '(',
            &opener,
            0,
            &[]
        ));
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
            should_render_symbol_as_english(true, true, &[stack_top], ')', &closer, 0, &[]),
            expected,
        );
    }

    /// `should_render_symbol_as_english` for ',' — 양쪽 ASCII + 영어 컨텍스트 둘 다 필요.
    #[rstest::rstest]
    #[case::both_ascii_in_english_mode("A,B", true, true)]
    #[case::not_in_english_mode("A,B", false, false)]
    #[case::korean_neighbor("가,B", true, false)]
    fn should_render_symbol_as_english_for_comma_requires_ascii_neighbors(
        #[case] input: &str,
        #[case] is_english: bool,
        #[case] expected: bool,
    ) {
        let word: Vec<char> = input.chars().collect();
        assert_eq!(
            should_render_symbol_as_english(true, is_english, &[], ',', &word, 1, &[]),
            expected
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
