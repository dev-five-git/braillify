use std::borrow::Cow;

use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct MiddleDotSpacingRule;

fn previous_word<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<&'b WordToken<'a>> {
    tokens[..index]
        .iter()
        .rev()
        .find_map(|token| match token {
            Token::Mode(_) => None,
            Token::Word(word) => Some(Some(word)),
            _ => Some(None),
        })
        .flatten()
}

fn next_word<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<(usize, &'b WordToken<'a>)> {
    tokens
        .iter()
        .enumerate()
        .skip(index + 1)
        .find_map(|(token_index, token)| match token {
            Token::Mode(_) | Token::Space(_) => None,
            Token::Word(word) => Some(Some((token_index, word))),
            _ => Some(None),
        })
        .flatten()
}

/// Rules 51 and 59 attach a Korean colon/semicolon to the item on its left.
/// A spaced colon between two Roman/number items remains UEB print spacing,
/// so require a Korean item on either side of the punctuation boundary.
fn space_precedes_korean_colon_or_semicolon(
    tokens: &[Token<'_>],
    index: usize,
    previous: &WordToken<'_>,
) -> bool {
    let Some((punctuation_index, punctuation)) = next_word(tokens, index) else {
        return false;
    };
    // 제49항이 따르는 한글 맞춤법은 쉼표를 앞말에 붙여 쓰므로, 묵자에 편집상 공백이
    // 남아 있어도(`탄압받고 , 공공의`) 쌍점·쌍반점과 같은 자리에서 붙인다. 마침표와
    // 물음표는 제49항 예문이 부호 자체를 가리키는 데 쓰므로(`? 대신 .를`) 제외한다.
    if !punctuation
        .chars
        .first()
        .is_some_and(|symbol| matches!(symbol, ':' | ';' | ','))
        || punctuation.chars.len() != 1
    {
        return false;
    }

    previous
        .chars
        .iter()
        .rev()
        .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
        .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        || next_word(tokens, punctuation_index).is_some_and(|(_, word)| {
            word.chars
                .iter()
                .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        })
}

impl TokenRule for MiddleDotSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        126
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        // Merge a one-sided editorial space at the token boundary so the
        // middle dot is encoded with the same character context as canonical
        // `정치·경제`, not merely emitted as an adjacent second word.
        if let Some(Token::Word(left)) = tokens.get(index)
            && matches!(tokens.get(index + 1), Some(Token::Space(_)))
            && let Some(Token::Word(right)) = tokens.get(index + 2)
            && (left.chars.last() == Some(&'·') || right.chars.first() == Some(&'·'))
        {
            let text = format!("{}{}", left.text, right.text);
            let chars = text.chars().collect::<Vec<_>>();
            return Ok(TokenAction::ReplaceRange(
                3,
                vec![Token::Word(WordToken {
                    text: Cow::Owned(text),
                    chars: chars.clone(),
                    meta: WordMeta::from_chars(&chars),
                })],
            ));
        }

        let Some(Token::Space(_)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let Some(prev) = previous_word(tokens, index) else {
            return Ok(TokenAction::Noop);
        };
        let Some((_, next)) = next_word(tokens, index) else {
            return Ok(TokenAction::Noop);
        };

        // Korean rule 50: the middle dot is attached on both sides. Its print
        // source sometimes contains editorial spaces, but the braille spacing
        // is still canonicalized by the rule.
        if prev.chars.last() == Some(&'·') || next.chars.first() == Some(&'·') {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        if space_precedes_korean_colon_or_semicolon(tokens, index, prev) {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        let prev_text = prev.text.as_ref();
        let next_text = next.text.as_ref();

        if (prev_text.ends_with('\'') || prev_text.ends_with('’'))
            && next_text
                .chars()
                .next()
                .is_some_and(crate::utils::is_korean_char)
            && next_text.starts_with("이다")
        {
            return Ok(TokenAction::ReplaceMany(vec![]));
        }

        Ok(TokenAction::Noop)
    }
}

/// 제59항: a Korean semicolon is attached on its left and followed by one blank
/// cell, so print that runs the next item straight on (`빛;나이다`) gains the
/// blank in braille.
///
/// 제51항 본문 gives the colon the same shape — attached on its left, one blank
/// after — while [다만 2] keeps 시:분 and 장:절 attached. Those excepted pairs are
/// numeric on both sides, so a colon that sits between two Korean syllables
/// (`관장:배선철`) is the 본문 case and takes the blank; a digit-flanked colon
/// (`20:30`) is left to print spacing.
pub struct KoreanSemicolonTrailingSpaceRule;

fn is_closing_after_colon(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            ')' | ']'
                | '}'
                | '\u{2019}'
                | '\u{201d}'
                | '"'
                | '\''
                | '」'
                | '』'
                | '〉'
                | '》'
                | ','
                | '.'
                | '!'
                | '?'
        )
}

fn korean_semicolon_split_index(chars: &[char]) -> Option<usize> {
    chars.windows(3).position(|window| {
        crate::utils::is_korean_char(window[0])
            && window[1] == ';'
            && !is_closing_after_colon(window[2])
    })
}

/// 제51항 본문의 예 `일시: 2006년 …` 은 표제와 내용을 쌍점으로 가르고 뒤를 한 칸
/// 띄운다. [다만 2] 의 예 `청군:백군` 은 어절 전체가 한글과 쌍점만으로 이루어진
/// 대비 쌍이다(나머지 예 `오전 10:20`, `요한 3:16` 은 숫자 쌍이라 이 함수 밖이다).
/// 따라서 한글 사이의 쌍점은 그 어절이 대비 쌍 꼴일 때만 붙이고, 괄호·따옴표 등이
/// 섞여 표제와 내용을 가르는 꼴이면 본문에 따라 뒤에 한 칸을 둔다.
fn korean_label_colon_split_index(chars: &[char]) -> Option<usize> {
    let position = chars.windows(3).position(|window| {
        crate::utils::is_korean_char(window[0])
            && window[1] == ':'
            && crate::utils::is_korean_char(window[2])
    })?;
    let is_contrast_pair = chars
        .iter()
        .all(|ch| crate::utils::is_korean_char(*ch) || *ch == ':');
    (!is_contrast_pair).then_some(position)
}

fn owned_word<'a>(chars: &[char]) -> Token<'a> {
    Token::Word(WordToken {
        text: Cow::Owned(chars.iter().collect()),
        chars: chars.to_vec(),
        meta: WordMeta::from_chars(chars),
    })
}

impl TokenRule for KoreanSemicolonTrailingSpaceRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        127
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let split = korean_semicolon_split_index(&word.chars)
            .into_iter()
            .chain(korean_label_colon_split_index(&word.chars))
            .min();
        let Some(split) = split else {
            return Ok(TokenAction::Noop);
        };
        let colon = split + 1;
        Ok(TokenAction::ReplaceMany(vec![
            owned_word(&word.chars[..=colon]),
            Token::Space(crate::rules::token::SpaceKind::Regular),
            owned_word(&word.chars[colon + 1..]),
        ]))
    }
}

/// 제49항 defers punctuation spacing to the 한글 맞춤법 appendix, which writes
/// the 물결표 attached to both sides. An editorial `300 ~ 350` in print is
/// therefore joined the same way as the middle dot above.
/// 제49항이 따르는 한글 맞춤법은 붙임표의 앞뒤를 붙여 쓴다. 묵자가 편집상
/// `준우승 - 홍길동`처럼 띄워 놓아도 점자 띄어쓰기는 규정을 따르므로 한 어절로
/// 잇는다. 양쪽이 한글일 때만 적용해 제46항의 뺄셈표(`a - b`)와 가르는데, 뺄셈은
/// 로마자·숫자 사이에서 쓰이기 때문이다.
pub struct KoreanHyphenSpacingRule;

impl TokenRule for KoreanHyphenSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        129
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let (
            Some(Token::Word(left)),
            Some(Token::Space(_)),
            Some(Token::Word(hyphen)),
            Some(Token::Space(_)),
            Some(Token::Word(right)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
            tokens.get(index + 3),
            tokens.get(index + 4),
        )
        else {
            return Ok(TokenAction::Noop);
        };
        if hyphen.chars.as_slice() != ['-']
            || !left
                .chars
                .last()
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
            || !right
                .chars
                .first()
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        {
            return Ok(TokenAction::Noop);
        }
        let mut chars = left.chars.clone();
        chars.extend(&hyphen.chars);
        chars.extend(&right.chars);
        Ok(TokenAction::ReplaceRange(5, vec![owned_word(&chars)]))
    }
}

pub struct TildeSpacingRule;

impl TokenRule for TildeSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        128
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(left)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let is_tilde = |word: &WordToken<'_>| matches!(word.chars.as_slice(), ['~'] | ['∼']);
        let joined = |left: &WordToken<'_>, right: &WordToken<'_>| {
            let mut chars = left.chars.clone();
            chars.extend(&right.chars);
            chars
        };
        match (tokens.get(index + 1), tokens.get(index + 2)) {
            (Some(Token::Space(_)), Some(Token::Word(right)))
                if is_tilde(right)
                    || (left.chars.last().is_some_and(|ch| matches!(ch, '~' | '∼'))
                        && !is_tilde(left)) =>
            {
                if is_tilde(right) {
                    if let (Some(Token::Space(_)), Some(Token::Word(after))) =
                        (tokens.get(index + 3), tokens.get(index + 4))
                    {
                        let mut chars = joined(left, right);
                        chars.extend(&after.chars);
                        return Ok(TokenAction::ReplaceRange(5, vec![owned_word(&chars)]));
                    }
                    return Ok(TokenAction::Noop);
                }
                Ok(TokenAction::ReplaceRange(
                    3,
                    vec![owned_word(&joined(left, right))],
                ))
            }
            _ => Ok(TokenAction::Noop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 제59항: the blank after a Korean semicolon is written even when print
    /// runs the items together; the colon keeps print spacing (제51항 [다만 2]).
    #[rstest::rstest]
    #[case::korean_semicolon_attached("빛;나이다", "빛; 나이다")]
    #[case::contrast_colon_stays_attached("청군:백군", "청군:백군")]
    #[case::time_stays_attached("오전 10:20", "오전 10:20")]
    #[case::closing_quote_after_semicolon("‘큐;’는", "‘큐;’는")]
    fn korean_semicolon_gains_trailing_blank(#[case] input: &str, #[case] canonical: &str) {
        assert_eq!(crate::encode(input), crate::encode(canonical));
    }

    /// 제49항 + 한글 맞춤법 부록: the 물결표 is attached on both sides.
    #[rstest::rstest]
    #[case::spaced_both("무게 300 ~ 350kg", "무게 300~350kg")]
    #[case::spaced_right("무게 300~ 350kg", "무게 300~350kg")]
    #[case::korean_range("부산 ~ 베이징", "부산~베이징")]
    fn spaced_tilde_is_attached(#[case] spaced: &str, #[case] canonical: &str) {
        assert_eq!(crate::encode(spaced), crate::encode(canonical));
    }

    /// Korean rules 50, 51, and 59 determine braille spacing even when the
    /// print source contains editorial spaces around the punctuation.
    #[rstest::rstest]
    #[case::middle_dot_both_sides("정치 · 경제", "정치·경제")]
    #[case::middle_dot_left("정치 ·경제", "정치·경제")]
    #[case::middle_dot_right("정치· 경제", "정치·경제")]
    #[case::korean_colon("제목 : 내용", "제목: 내용")]
    #[case::roman_to_korean_colon("WHO : 세계", "WHO: 세계")]
    #[case::korean_semicolon("채소 ; 과일", "채소; 과일")]
    fn canonical_korean_punctuation_spacing(#[case] spaced: &str, #[case] canonical: &str) {
        assert_eq!(crate::encode(spaced), crate::encode(canonical));
    }

    /// Rule 32 leaves print spacing inside a Roman section to UEB. A Korean
    /// prefix earlier in the token does not turn `FAPAS : Food` into a Korean
    /// colon boundary because the immediately preceding item is Roman.
    #[test]
    fn attached_roman_item_preserves_space_before_ueb_colon() {
        assert_ne!(
            crate::encode("설명(FAPAS : Food)"),
            crate::encode("설명(FAPAS: Food)")
        );
    }

    #[test]
    fn colon_spacing_probe_returns_false_when_no_punctuation_word_follows() {
        let mut ir = crate::rules::token::DocumentIR::parse("한국", false);
        ir.tokens
            .push(Token::Space(crate::rules::token::SpaceKind::Regular));
        let Token::Word(previous) = &ir.tokens[0] else {
            unreachable!("fixture begins with a word")
        };

        assert!(!space_precedes_korean_colon_or_semicolon(
            &ir.tokens, 1, previous
        ));
    }
}

#[cfg(test)]
mod label_colon_coverage {
    /// 제51항 본문은 표제와 내용을 가르는 쌍점 뒤를 띄우고, [다만 2] 의 대비 쌍은
    /// 붙인다. 어절에 괄호·따옴표가 섞이면 표제 꼴로 본다.
    #[rstest::rstest]
    #[case::contrast_pair("청군:백군")]
    #[case::three_syllable_pair("재판장:신교식")]
    fn an_all_hangul_pair_stays_attached(#[case] input: &str) {
        let spaced = input.replace(':', ": ");
        assert_ne!(crate::encode(input), crate::encode(&spaced));
    }

    #[rstest::rstest]
    #[case::quoted("‘제목:내용’")]
    #[case::double_quoted("“제목:내용”")]
    fn an_enclosure_makes_the_colon_a_label_boundary(#[case] input: &str) {
        assert_eq!(
            crate::encode(input),
            crate::encode(&input.replace(':', ": "))
        );
    }
}
