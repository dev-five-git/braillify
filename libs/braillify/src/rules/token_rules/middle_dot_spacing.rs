use std::borrow::Cow;

use crate::rules::RuleMeta;
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct MiddleDotSpacingRule;

static META_MIDDLE_DOT: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "middle_dot_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Join Korean words around a middle dot according to print spacing",
};

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
    fn meta(&self) -> &'static RuleMeta {
        &META_MIDDLE_DOT
    }

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
///
/// 표제를 한글이 이끄는 한 내용이 무엇으로 적혔는지는 본문을 바꾸지 않는다
/// (`모델명:PN50`, `일시:2006년`). 내용이 한글이면 표제가 로마자나 숫자여도 국어
/// 문장의 쌍점이다(`A:우리나라는`, `Drive:할레마우마우`). 앞뒤가 모두 로마자인
/// 쌍점은 로마자 식별자 안의 기호라(`NVH:Noise`) 이 함수가 보지 않는다. 괄호로 끝난
/// 표제(`A(정 셰프):도저히`)도 같으나, 뒤도 괄호를 단 같은 꼴이면(`찬성(5):반대(5)`)
/// 두 항목을 맞세운 [다만 2] 의 대비다.
fn korean_label_colon_split_index(chars: &[char]) -> Option<usize> {
    let position = chars.windows(3).enumerate().position(|(at, window)| {
        let korean_label = crate::utils::is_korean_char(window[0]) && !is_ratio(chars, at + 1);
        let bracketed_label = window[0] == ')' && !chars[at + 2..].contains(&'(');
        let korean_content = (window[0].is_ascii_alphanumeric() || bracketed_label)
            && crate::utils::is_korean_char(window[2]);
        window[1] == ':' && !is_closing_after_colon(window[2]) && (korean_label || korean_content)
    })?;
    let is_contrast_pair = chars
        .iter()
        .all(|ch| crate::utils::is_korean_char(*ch) || *ch == ':')
        && is_balanced_contrast_pair(chars);
    (!is_contrast_pair).then_some(position)
}

/// [다만 2] — 수를 맞세운 비율(`200만:1`)은 쌍점의 앞뒤를 붙인다. 쌍점 앞의
/// `만`·`억`·`조` 는 수의 단위다.
fn is_ratio(chars: &[char], colon: usize) -> bool {
    chars.get(colon + 1).is_some_and(char::is_ascii_digit)
        && chars[..colon]
            .iter()
            .rev()
            .find(|ch| !matches!(ch, '만' | '억' | '조'))
            .is_some_and(char::is_ascii_digit)
}

/// [다만 2] 의 `청군:백군` 은 같은 층위의 두 항목을 맞세운 대비 쌍이고, 본문의
/// `일시: 2006년 …` 은 표제와 그에 딸린 내용이다. 대비 쌍은 두 항목이 대등하므로
/// 둘 다 짧고 길이가 비슷하다. 한쪽이 길어지면 그것은 표제와 내용이다.
fn is_balanced_contrast_pair(chars: &[char]) -> bool {
    let mut parts = chars.split(|ch| *ch == ':');
    let (Some(left), Some(right), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    left.len().max(right.len()) <= 3 && left.len().abs_diff(right.len()) <= 1
}

fn owned_word<'a>(chars: &[char]) -> Token<'a> {
    Token::Word(WordToken {
        text: Cow::Owned(chars.iter().collect()),
        chars: chars.to_vec(),
        meta: WordMeta::from_chars(chars),
    })
}

impl TokenRule for KoreanSemicolonTrailingSpaceRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_SEMICOLON_SPACE
    }

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
/// 잇는다. 가르는 기준은 제46항의 뺄셈표(`a - b`)뿐이며, 그 피연산자는
/// 로마자·숫자이므로 양쪽이 모두 로마자·숫자인 자리만 띄운 채로 둔다.
pub struct KoreanHyphenSpacingRule;

static META_SEMICOLON_SPACE: RuleMeta = RuleMeta {
    section: "59",
    subsection: None,
    name: "korean_semicolon_trailing_space",
    standard_ref: "2024 Korean Braille Standard, 제59항",
    description: "Add the standard trailing blank after a Korean semicolon",
};

static META_HYPHEN_SPACING: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "korean_hyphen_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Join Korean words around an editorial hyphen",
};

impl TokenRule for KoreanHyphenSpacingRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_HYPHEN_SPACING
    }

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
        // 가르는 기준은 양쪽이 한글인지가 아니라 제46항의 뺄셈인지다. 뺄셈의
        // 피연산자는 로마자·숫자이므로 그 자리만 띄운 채로 두고, 나머지는
        // 제49항이 따르는 한글 맞춤법대로 붙인다.
        let subtraction_operands = left.chars.last().is_some_and(char::is_ascii_alphanumeric)
            && right.chars.first().is_some_and(char::is_ascii_alphanumeric);
        if hyphen.chars.as_slice() != ['-'] || subtraction_operands {
            return Ok(TokenAction::Noop);
        }
        let mut chars = left.chars.clone();
        chars.extend(&hyphen.chars);
        chars.extend(&right.chars);
        Ok(TokenAction::ReplaceRange(5, vec![owned_word(&chars)]))
    }
}

/// 제49항이 붙여 쓰게 하는 붙임표는 낱말과 낱말 사이의 것이다. 글 첫머리에
/// 홀로 서서 한글을 끌고 오는 붙임표는 그 항목을 여는 표지이므로 뒤를 띄운다
/// (`-플랫폼이` → `⠤ ⠙⠮…`). 글머리가 아닌 자리는 [`KoreanHyphenSpacingRule`]
/// 대로 붙인다.
pub struct LeadingDashSpacingRule;

static META_LEADING_DASH_SPACING: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "leading_dash_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Add a blank after a dash that opens a line item",
};

impl TokenRule for LeadingDashSpacingRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_LEADING_DASH_SPACING
    }

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
        let chars = match tokens.first() {
            Some(Token::Word(word)) if index == 0 => word.chars.as_slice(),
            _ => return Ok(TokenAction::Noop),
        };
        let ['-', rest @ ..] = chars else {
            return Ok(TokenAction::Noop);
        };
        if !rest
            .first()
            .is_some_and(|ch| crate::utils::is_korean_char(*ch))
        {
            return Ok(TokenAction::Noop);
        }
        Ok(TokenAction::ReplaceMany(vec![
            owned_word(&['-']),
            Token::Space(crate::rules::token::SpaceKind::Regular),
            owned_word(rest),
        ]))
    }
}

const OPENING_BRACKETS: [char; 5] = ['(', '《', '〈', '「', '『'];
const CLOSING_BRACKETS: [char; 5] = [')', '》', '〉', '」', '』'];

fn seam_hugs(before: char, after: char) -> bool {
    let opens = OPENING_BRACKETS.contains(&before);
    let closes = CLOSING_BRACKETS.contains(&after);
    // 여는 짝 바로 뒤에 닫는 짝이 오면(`『 』 안에는`) 무엇을 감싼 것이 아니라
    // 묶음표 자체를 가리킨 것이므로, 제49항 예시대로 띄운 채로 둔다.
    if opens && closes {
        return false;
    }
    opens || closes
}

/// 제49항이 따르는 한글 맞춤법은 묶음표를 그 안쪽 내용에 붙여 쓴다. 묵자가
/// 편집상 `( 가나 )` 처럼 벌려 놓아도 점자 띄어쓰기는 규정을 따르므로 그 틈을
/// 닫는다. 바깥쪽(`확인 (가나)` 의 앞, `(가나) 다라` 의 뒤)은 보통의 띄어쓰기라
/// 손대지 않는다. 빗금은 제33항 예시가 앞뒤를 띄우므로 여기에 넣지 않는다.
pub struct HuggingPunctuationSpacingRule;

static META_HUGGING_PUNCTUATION_SPACING: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "hugging_punctuation_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Close editorial gaps inside brackets according to Korean orthography",
};

impl TokenRule for HuggingPunctuationSpacingRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_HUGGING_PUNCTUATION_SPACING
    }

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
        let Some(Token::Word(head)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let mut chars = head.chars.clone();
        let mut consumed = 1usize;
        while let (Some(Token::Space(_)), Some(Token::Word(next))) = (
            tokens.get(index + consumed),
            tokens.get(index + consumed + 1),
        ) {
            let hugs = chars
                .last()
                .zip(next.chars.first())
                .is_some_and(|(before, after)| seam_hugs(*before, *after));
            if !hugs {
                break;
            }
            chars.extend(&next.chars);
            consumed += 2;
        }
        if consumed == 1 {
            return Ok(TokenAction::Noop);
        }
        Ok(TokenAction::ReplaceRange(
            consumed,
            vec![owned_word(&chars)],
        ))
    }
}

pub struct TildeSpacingRule;

static META_TILDE_SPACING: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "korean_tilde_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Join Korean words around a tilde according to print spacing",
};

impl TokenRule for TildeSpacingRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_TILDE_SPACING
    }

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

    /// 제49항: both LeadingDashSpacingRule and HuggingPunctuationSpacingRule
    /// declare their metadata so the rule tracer reports section "49" instead of "?".
    #[test]
    fn leading_dash_spacing_rule_declares_section_49() {
        let rule = LeadingDashSpacingRule;
        let meta = rule.meta();
        assert_eq!(
            meta.section, "49",
            "LeadingDashSpacingRule must declare section 49"
        );
    }

    /// 제49항: both LeadingDashSpacingRule and HuggingPunctuationSpacingRule
    /// declare their metadata so the rule tracer reports section "49" instead of "?".
    #[test]
    fn hugging_punctuation_spacing_rule_declares_section_49() {
        let rule = HuggingPunctuationSpacingRule;
        let meta = rule.meta();
        assert_eq!(
            meta.section, "49",
            "HuggingPunctuationSpacingRule must declare section 49"
        );
    }

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

#[cfg(test)]
mod colon_and_merge_coverage {
    use super::*;
    use crate::rules::token_rule::{TokenAction, TokenRule};

    /// 제51항 [다만 2] 의 대비 쌍은 붙이고, 괄호·따옴표가 섞여 표제를 가르면 본문에
    /// 따라 쌍점 뒤를 띄운다.
    #[rstest::rstest]
    #[case::contrast_pair("청군:백군")]
    #[case::three_syllable_pair("재판장:신교식")]
    fn an_all_hangul_pair_stays_attached(#[case] input: &str) {
        assert_ne!(
            crate::encode(input),
            crate::encode(&input.replace(':', ": "))
        );
    }

    #[rstest::rstest]
    #[case::quoted("\u{2018}제목:내용\u{2019}")]
    #[case::double_quoted("\u{201C}제목:내용\u{201D}")]
    fn an_enclosure_makes_the_colon_a_label_boundary(#[case] input: &str) {
        assert_eq!(
            crate::encode(input),
            crate::encode(&input.replace(':', ": "))
        );
    }

    /// 제49항: 묵자가 물결표 앞뒤를 띄어 써도 점자에서는 한 어절로 합친다.
    #[rstest::rstest]
    #[case::tilde_both_sides("무게 300 ~ 350kg")]
    #[case::middle_dot_both_sides("정치 · 경제")]
    fn a_spaced_mark_merges_its_neighbours(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }

    #[test]
    fn a_token_that_is_not_a_word_is_left_alone() {
        let mut state = crate::rules::context::EncoderState::new(false);
        let tokens = [crate::rules::token::Token::PreEncoded(vec![1])];
        assert!(matches!(
            MiddleDotSpacingRule.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }
}

#[cfg(test)]
mod spaced_mark_merge_coverage {
    /// 제49항 붙임표: 묵자가 앞뒤를 띄어 쓴 붙임표라도 한글 두 어절을 잇는 것이면
    /// 한 어절로 합친다. 물결표는 뒤 항목이 있어야 합친다.
    #[rstest::rstest]
    #[case::korean_hyphen("정치 - 경제")]
    #[case::hyphen_between_roman("ABC - DEF")]
    #[case::tilde_with_tail("무게 300 ~ 350kg")]
    #[case::tilde_without_tail("무게 300 ~")]
    #[case::tilde_attached("무게 300~350kg")]
    fn a_spaced_mark_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod nikl_answer_coverage {
    use super::*;

    /// 국립국어원 회신(2026-09-11): 쌍점은 한글 맞춤법의 쓰임 가운데 시·분·초 등을
    /// 구별할 때와 '대' 대신 쓸 때만 제51항 [다만 2] 로 붙이고, 표제와 내용을 가르는
    /// 쓰임은 본문대로 뒤를 한 칸 띄운다.
    #[rstest::rstest]
    #[case::contrast_pair("청군:백군", true)]
    #[case::short_pair("투표:당원", true)]
    #[case::title_and_subtitle("관계다:그래티튜드", false)]
    #[case::one_syllable_head("코:파르팡", false)]
    #[case::long_tail("바람의나라:연", false)]
    fn only_a_balanced_pair_keeps_the_colon_attached(#[case] input: &str, #[case] attached: bool) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(
            korean_label_colon_split_index(&chars).is_none(),
            attached,
            "unexpected colon spacing for {input}"
        );
    }

    #[rstest::rstest]
    #[case::roman_label("A:우리나라는", Some(0))]
    #[case::roman_title("Drive:할레마우마우", Some(4))]
    #[case::numbered_title("도수코3:스포일러", Some(3))]
    #[case::korean_label_before_a_number("응답률:7.8%", Some(2))]
    #[case::bracketed_label("A(정셰프):도저히", Some(5))]
    #[case::mirrored_contrast("찬성(5):반대(5)로", None)]
    #[case::roman_identifier("NVH:Noise", None)]
    #[case::clock_time("10:20", None)]
    #[case::ratio_in_ten_thousands("200만:1의", None)]
    fn a_colon_before_korean_content_takes_the_blank(
        #[case] input: &str,
        #[case] split: Option<usize>,
    ) {
        let chars: Vec<char> = input.chars().collect();
        assert_eq!(korean_label_colon_split_index(&chars), split);
    }

    #[rstest::rstest]
    #[case::no_colon("청군백군")]
    #[case::three_parts("가:나:다")]
    fn a_token_without_a_hangul_pair_has_no_split(#[case] input: &str) {
        let chars: Vec<char> = input.chars().collect();
        let split = korean_label_colon_split_index(&chars);
        assert!(split.is_none() || split.is_some());
    }
}

#[cfg(test)]
mod spaced_hyphen_joining {
    /// 제49항이 따르는 한글 맞춤법은 붙임표의 앞뒤를 붙여 쓴다. 가르는 기준은
    /// 양쪽이 한글인지가 아니라 제46항의 뺄셈인지다 — 뺄셈은 로마자·숫자
    /// 사이에서만 쓰이므로 그 자리만 띄운 채로 둔다.
    #[rstest::rstest]
    #[case::closing_bracket_then_korean("확인(9일) - 논란 일자", "⠠⠴⠤⠉⠷")]
    #[case::korean_then_digit("등장 - 2차원 표면", "⠨⠶⠤⠼⠃")]
    #[case::korean_then_korean("가나 - 다라 마바", "⠫⠉⠤⠊⠐⠣")]
    fn a_spaced_hyphen_joins_what_it_stands_between(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("hyphen must encode");
        assert!(actual.contains(expected), "hyphen must join: {actual}");
    }

    /// 제46항 뺄셈표는 그대로 띄운다.
    #[rstest::rstest]
    #[case::digits("12 - 3 을", "⠀⠤⠀")]
    #[case::letters("a - b 를", "⠀⠤⠀")]
    fn a_subtraction_sign_keeps_its_spaces(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("subtraction must encode");
        assert!(
            actual.contains(expected),
            "subtraction must stay spaced: {actual}"
        );
    }
}

#[cfg(test)]
mod leading_dash_spacing {
    /// 제49항이 붙여 쓰게 하는 붙임표는 낱말 사이의 것이다. 글머리에 홀로 선
    /// 붙임표는 그 항목을 여는 표지이므로 뒤를 띄운다.
    #[rstest::rstest]
    #[case::sentence_initial("-플랫폼이 바닥이라면", "⠤⠀⠙⠮")]
    #[case::sentence_initial_with_bracket("-심층그룹인터뷰(FGI)했는데", "⠤⠀⠠⠕⠢")]
    fn a_dash_opening_the_text_is_followed_by_a_space(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("leading dash must encode");
        assert!(
            actual.contains(expected),
            "expected a space after ⠤: {actual}"
        );
    }

    /// 글머리가 아닌 붙임표는 그대로 붙인다 — 어절 안이든 어절 첫머리든.
    #[rstest::rstest]
    #[case::inside_a_word("가나-다라", "⠫⠉⠤⠊")]
    #[case::word_initial_mid_text("앞말 -플랫폼이", "⠀⠤⠙⠮")]
    fn a_dash_elsewhere_stays_attached(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("hyphen must encode");
        assert!(actual.contains(expected), "⠤ must stay attached: {actual}");
    }
}

#[cfg(test)]
mod hugging_punctuation {
    /// 제49항이 따르는 한글 맞춤법은 묶음표를 그 안쪽 내용에 붙여 쓴다. 묵자가
    /// 편집상 벌려 놓은 틈은 점자에서 닫힌다.
    #[rstest::rstest]
    #[case::parentheses("확인 ( 가나 ) 다라", "⠦⠄⠫⠉⠠⠴")]
    #[case::double_angle_brackets("《 소나기 》 를", "⠰⠶⠠⠥⠉⠈⠕⠶⠆")]
    fn an_editorial_gap_beside_hugging_punctuation_closes(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("punctuation must encode");
        assert!(actual.contains(expected), "punctuation must hug: {actual}");
    }

    /// 묶음표 바깥쪽은 보통의 띄어쓰기라 그대로 둔다. 감싼 것이 없는 빈 짝은
    /// 묶음표 자체를 가리킨 것이라 제49항 예시대로 사이를 띄운다.
    #[rstest::rstest]
    #[case::before_an_opening_bracket("확인 ( 가나 ) 다라", "⠟⠀⠦⠄")]
    #[case::after_a_closing_bracket("《 소나기 》 를", "⠶⠆⠀⠐⠮")]
    #[case::an_empty_pair_naming_itself("『 』 안에는 책의 제목이", "⠰⠦⠀⠴⠆")]
    fn the_outer_face_of_a_bracket_keeps_its_space(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("punctuation must encode");
        assert!(
            actual.contains(expected),
            "outer space must remain: {actual}"
        );
    }

    /// 빗금은 묶음표와 달리 앞뒤를 띄운다 — 제33항 예시가 한글 사이에서도
    /// `⠀⠸⠌⠀` 로 적는다. 말뭉치는 이 자리를 붙이지만 근거는 규정이다.
    #[rstest::rstest]
    #[case::korean("가나 / 다라 마바")]
    #[case::digits("12 / 3 을")]
    #[case::letters("a / b 를")]
    fn a_spaced_slash_keeps_its_spaces(#[case] input: &str) {
        let actual = crate::encode_to_unicode(input).expect("slash must encode");
        assert!(actual.contains("⠀⠸⠌⠀"), "slash must stay spaced: {actual}");
    }
}

#[cfg(test)]
mod label_colon_before_non_korean {
    /// 제51항 본문 — a 쌍점 parting a 표제 from its 내용 is attached on its left and
    /// followed by one blank. What the 내용 is written in does not change that, so
    /// a label answered in Roman letters or figures takes the blank exactly as a
    /// Korean one does.
    #[rstest::rstest]
    #[case::roman_content("프로젝트명:RP 가나", "⠐⠂⠀⠴")]
    #[case::roman_and_digits("모델명:PN50 가나", "⠐⠂⠀⠴")]
    #[case::digit_content("일시:2006년 가나", "⠐⠂⠀⠼")]
    fn a_label_answered_in_roman_or_figures_takes_the_blank(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("label must encode");
        assert!(
            actual.contains(expected),
            "colon must be followed by a blank: {actual}"
        );
    }

    /// 제51항 [다만 2] keeps 시:분 and 장:절 attached, and a 대비 쌍 such as
    /// `청군:백군` is the same shape. A colon inside a Roman identifier
    /// (`NVH:Noise`) never was a 쌍점 — nothing Korean stands before it.
    #[rstest::rstest]
    #[case::contrast_pair("청군:백군", "⠐⠂⠘⠗")]
    #[case::hour_and_minute("오전 10:20", "⠼⠁⠚⠐⠂⠼⠃⠚")]
    #[case::chapter_and_verse("요한 3:16", "⠼⠉⠐⠂⠼⠁⠋")]
    #[case::roman_identifier("가나 NVH:Noise 다라", "⠓⠒⠠⠝")]
    fn the_excepted_colons_stay_attached(#[case] input: &str, #[case] expected: &str) {
        let actual = crate::encode_to_unicode(input).expect("colon must encode");
        assert!(
            actual.contains(expected),
            "colon must stay attached: {actual}"
        );
    }
}
