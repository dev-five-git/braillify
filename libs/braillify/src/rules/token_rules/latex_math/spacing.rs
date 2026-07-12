//! Token spacing helpers for LaTeX math encoding (extracted from latex_math.rs).

use crate::rules::token::Token;

/// Whether the immediate neighbour (direct or via single Space) is a Korean Word.
fn neighbor_is_korean(tokens: &[Token<'_>], index: usize, dir: NeighborDir) -> bool {
    let neighbor_idx = match dir {
        NeighborDir::Prev => index.checked_sub(1),
        NeighborDir::Next => Some(index + 1),
    };
    let Some(idx) = neighbor_idx else {
        return false;
    };
    let direct_is_korean_word =
        matches!(tokens.get(idx), Some(Token::Word(w)) if w.meta.has_korean);
    if direct_is_korean_word {
        return true;
    }
    let direct_is_space = matches!(tokens.get(idx), Some(Token::Space(_)));
    if !direct_is_space {
        return false;
    }
    let beyond_idx = match dir {
        NeighborDir::Prev => idx.checked_sub(1),
        NeighborDir::Next => Some(idx + 1),
    };
    beyond_idx
        .and_then(|j| tokens.get(j))
        .is_some_and(|t| matches!(t, Token::Word(w) if w.meta.has_korean))
}

#[derive(Clone, Copy)]
enum NeighborDir {
    Prev,
    Next,
}

pub(super) fn previous_content_needs_math_spacing(tokens: &[Token<'_>], index: usize) -> usize {
    let Some(previous_index) = index.checked_sub(1) else {
        return 0;
    };

    match tokens.get(previous_index) {
        Some(Token::Space(_)) => {
            let left = previous_index
                .checked_sub(1)
                .and_then(|left_index| tokens.get(left_index));
            match left {
                Some(Token::Word(word)) if word.meta.has_korean => 1,
                _ => 0,
            }
        }
        Some(Token::Word(_) | Token::PreEncoded(_) | Token::Fraction(_) | Token::Mode(_)) => 2,
        None => 0,
    }
}

pub(super) fn next_content_needs_math_spacing(tokens: &[Token<'_>], index: usize) -> usize {
    match tokens.get(index + 1) {
        Some(Token::Space(_)) => {
            let right = tokens.get(index + 2);
            match right {
                Some(Token::Word(word)) if word.meta.has_korean => 1,
                _ => 0,
            }
        }
        Some(Token::Word(_) | Token::PreEncoded(_) | Token::Fraction(_) | Token::Mode(_)) => 2,
        None => 0,
    }
}

/// PDF — 한국어 산문 내 단일 math letter는 따옴표(⠴...⠲)로 감싼다.
/// 본문이 1~2자 ASCII letter이고 좌우가 한국어 컨텍스트일 때 적용한다.
pub(crate) fn wrap_latex_math_tokens_with_inner<'a>(
    tokens: &[Token<'a>],
    index: usize,
    bytes: Vec<u8>,
    inner: &str,
) -> Vec<Token<'a>> {
    let mut replacement = Vec::new();
    // PDF — `$-2$`, `$0.3010$` 같이 부호+숫자/소수점만 있는 단순 수치 표기는
    // "본격적 수식"이 아니므로 한국어 단어 경계에서 추가 두칸 띄어쓰기를 적용하지 않는다.
    // Space token 1칸으로 충분하다.
    let inner_is_simple_numeric = !inner.is_empty()
        && inner
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '\u{2212}' | '.' | ','));

    // PDF — 한국어 산문 내 단일/소수 math letter는 따옴표(⠴...⠲)로 감싼다.
    // 검출 조건:
    // 1. inner가 모두 ASCII letter 또는 짧은 letter+숫자 첨자 패턴
    // 2. 좌측이 한국어 단어로 끝나거나 우측이 한국어 단어로 시작(또는 한국어 particle)
    let is_short_prose_letter = !inner.is_empty()
        && inner.chars().count() <= 2
        && inner.chars().all(|c| c.is_ascii_alphabetic());
    // 콤마-구분 letter 리스트 (예: `a, b, c`, `A, B, C`)
    let comma_separated_letter_list = !inner.is_empty()
        && inner.contains(',')
        && inner.split(',').map(str::trim).all(|part| {
            !part.is_empty()
                && part.chars().count() == 1
                && part.chars().all(|c| c.is_ascii_alphabetic())
        });
    let in_korean_prose = if is_short_prose_letter || comma_separated_letter_list {
        let prev_is_korean = neighbor_is_korean(tokens, index, NeighborDir::Prev);
        let next_is_korean = neighbor_is_korean(tokens, index, NeighborDir::Next);
        prev_is_korean || next_is_korean
    } else {
        false
    };

    // 따옴표 wrap 경우 자체적으로 경계 명시 → 추가 leading 공백 불필요.
    // 단순 수치 또한 leading 공백 없음.
    let leading_spaces = if inner_is_simple_numeric || in_korean_prose {
        0
    } else {
        previous_content_needs_math_spacing(tokens, index)
    };
    if leading_spaces > 0 {
        replacement.push(Token::PreEncoded(vec![0; leading_spaces]));
    }

    if in_korean_prose && comma_separated_letter_list {
        // 콤마-구분 letter 리스트: 각 letter를 quote/english marker로 감싼다.
        // 예: `a, b, c` → ⠴a⠂ ⠰b⠂ ⠰c⠲
        let letters: Vec<&str> = inner.split(',').map(str::trim).collect();
        let mut wrapped = Vec::new();
        for (i, letter) in letters.iter().enumerate() {
            if let Some(c) = letter.chars().next() {
                if i == 0 {
                    wrapped.push(52); // ⠴ open quote
                } else {
                    wrapped.push(0); // space
                    wrapped.push(48); // ⠰ english indicator
                }
                if c.is_ascii_uppercase() {
                    wrapped.push(32); // ⠠ capital marker
                    if let Ok(code) = crate::english::encode_english(c.to_ascii_lowercase()) {
                        wrapped.push(code);
                    }
                } else if let Ok(code) = crate::english::encode_english(c) {
                    wrapped.push(code);
                }
                if i + 1 < letters.len() {
                    wrapped.push(16); // ⠐ comma
                } else {
                    wrapped.push(50); // ⠲ close quote
                }
            }
        }
        replacement.push(Token::PreEncoded(wrapped));
    } else if in_korean_prose {
        // ⠴ (open quote, 52) + 본문 + ⠲ (close quote, 50)
        // 따옴표가 math/Korean 경계를 명시하므로 추가 trailing/leading 공백 불필요.
        let mut wrapped = Vec::with_capacity(bytes.len() + 2);
        wrapped.push(52);
        wrapped.extend(bytes);
        wrapped.push(50);
        replacement.push(Token::PreEncoded(wrapped));
        // Korean prose context에서는 trailing 공백을 emit하지 않는다 (Space token이 분리).
    } else {
        replacement.push(Token::PreEncoded(bytes));
        let trailing_spaces = if inner_is_simple_numeric {
            0
        } else {
            next_content_needs_math_spacing(tokens, index)
        };
        if trailing_spaces > 0 {
            replacement.push(Token::PreEncoded(vec![0; trailing_spaces]));
        }
    }
    replacement
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::{SpaceKind, WordMeta, WordToken};
    use crate::unicode::decode_unicode;
    use std::borrow::Cow;

    fn word(text: &str) -> Token<'static> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Owned(text.to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    /// `neighbor_is_korean`: direct neighbour is a Korean Word (no Space between).
    /// Drives the `direct_is_korean_word = true → return true` branch.
    #[test]
    fn neighbor_is_korean_direct_korean_word() {
        // [한국, MATH_TOKEN_AT_INDEX_1] — direct Prev neighbour is Korean Word.
        let tokens = vec![word("한국"), Token::PreEncoded(vec![])];
        assert!(neighbor_is_korean(&tokens, 1, NeighborDir::Prev));
        // [MATH_TOKEN_AT_INDEX_0, 한국] — direct Next neighbour is Korean Word.
        let tokens = vec![Token::PreEncoded(vec![]), word("한국")];
        assert!(neighbor_is_korean(&tokens, 0, NeighborDir::Next));
    }

    /// Negative case: direct neighbour is non-Korean Word → return false.
    #[test]
    fn neighbor_is_korean_direct_english_word() {
        let tokens = vec![word("hello"), Token::PreEncoded(vec![])];
        assert!(!neighbor_is_korean(&tokens, 1, NeighborDir::Prev));
    }

    /// Space + Korean Word beyond → returns true via beyond_idx path.
    #[test]
    fn neighbor_is_korean_space_then_korean() {
        let tokens = vec![
            word("한국"),
            Token::Space(SpaceKind::Regular),
            Token::PreEncoded(vec![]),
        ];
        assert!(neighbor_is_korean(&tokens, 2, NeighborDir::Prev));
    }

    /// No neighbour at all (Prev at index 0) → false.
    #[test]
    fn neighbor_is_korean_no_neighbour() {
        let tokens = vec![Token::PreEncoded(vec![])];
        assert!(!neighbor_is_korean(&tokens, 0, NeighborDir::Prev));
    }

    /// spacing.rs - `*_content_needs_math_spacing` None arms when index points past tokens.
    #[rstest::rstest]
    #[case::prev_oob(true, 2)]
    #[case::next_oob(false, 2)]
    fn content_needs_math_spacing_none_arm_returns_zero(
        #[case] is_prev: bool,
        #[case] index: usize,
    ) {
        let tokens: Vec<Token<'_>> = vec![Token::PreEncoded(vec![])];
        let result = if is_prev {
            previous_content_needs_math_spacing(&tokens, index)
        } else {
            next_content_needs_math_spacing(&tokens, index)
        };
        assert_eq!(result, 0);
    }

    /// spacing.rs - `next_content_needs_math_spacing` direct Word/PreEncoded arm → 2.
    #[test]
    fn next_content_needs_math_spacing_word_or_preencoded_returns_2() {
        let tokens: Vec<Token<'_>> = vec![Token::PreEncoded(vec![]), Token::PreEncoded(vec![])];
        assert_eq!(next_content_needs_math_spacing(&tokens, 0), 2);
    }

    #[test]
    fn wraps_comma_separated_letters_inside_korean_prose() {
        let tokens = vec![word("값"), Token::PreEncoded(vec![]), word("이다")];

        let replacement = wrap_latex_math_tokens_with_inner(&tokens, 1, vec![], "a, B, c");

        let [Token::PreEncoded(cells)] = replacement.as_slice() else {
            panic!("expected one pre-encoded wrapped token: {replacement:?}");
        };
        assert_eq!(
            cells,
            &vec![
                52,
                decode_unicode('⠁'),
                16,
                0,
                48,
                32,
                decode_unicode('⠃'),
                16,
                0,
                48,
                decode_unicode('⠉'),
                50,
            ]
        );
    }
}
