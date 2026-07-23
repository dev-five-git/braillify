//! PDF 한국어 제33항 — 학술 인용 형식 (저자, 연도a, 연도b; ...).
//!
//! `1998a,`, `1998b;` 같이 4자리 연도 + 단일 알파벳 suffix + 구두점 토큰을 감지해
//! 영어 모드 점역(⠴ begin / ⠰ continue + letter + 영어 구두점)으로 emit한다.
//! 단어 시작 이외 위치의 영어 모드 진입은 char-level emit에서 미지원이므로 token-level
//! 으로 처리한다.

use crate::english::encode_english;
use crate::number::encode_number;
use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;

pub struct Rule33CitationYearSuffixRule;

/// Rule33가 emit한 PreEncoded인지 구조적으로 확인한다.
/// Pattern: `⠼(60)` + 4 digit bytes + (`⠴`(52) | `⠰`(48)) + letter byte + suffix.
fn is_rule33_emission(bytes: &[u8]) -> bool {
    // 최소 길이: 60 + 4 digits + marker + letter = 7. + suffix (1 or 2 bytes).
    if bytes.len() < 8 || bytes.len() > 9 {
        return false;
    }
    if bytes[0] != 60 {
        return false;
    }
    // bytes[1..5]는 number digits — `encode_number`가 emit하는 정확한 셀 값만 허용한다.
    // NUMBER_MAP 값: {1,3,9,10,11,17,19,25,26,27}
    let is_digit_byte = |b: &u8| matches!(*b, 1 | 3 | 9 | 10 | 11 | 17 | 19 | 25 | 26 | 27);
    if !bytes[1..5].iter().all(is_digit_byte) {
        return false;
    }
    // bytes[5] is marker: 48 (⠰ continue) or 52 (⠴ begin)
    if !matches!(bytes[5], 48 | 52) {
        return false;
    }
    // bytes[6] is encoded letter (1..=63)
    if !(1..=63).contains(&bytes[6]) {
        return false;
    }
    // suffix: bytes[7..]
    match &bytes[7..] {
        [2] => true,     // ⠂ comma
        [50] => true,    // ⠲ period
        [48, 6] => true, // ⠰⠆ semicolon
        _ => false,
    }
}

fn match_year_suffix(text: &str) -> Option<(&str, char, char)> {
    let chars: Vec<char> = text.chars().collect();
    // 정확히 6자: 4 digits + 1 lowercase + 1 punctuation (',' or ';' or '.')
    if chars.len() != 6 {
        return None;
    }
    if !chars[..4].iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !chars[4].is_ascii_lowercase() {
        return None;
    }
    if !matches!(chars[5], ',' | ';' | '.') {
        return None;
    }
    let year_end = text.char_indices().nth(4).map(|(i, _)| i)?;
    Some((&text[..year_end], chars[4], chars[5]))
}

impl TokenRule for Rule33CitationYearSuffixRule {
    fn phase(&self) -> TokenPhase {
        // Normalization 단계 — 다른 토큰 변환 전에 처리. 토큰 엔진은 Normalization
        // phase에서 Noop 시에도 다음 rule을 시도하므로 안전하다.
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        // LatexMergeRule(10) 이후, EmphasisRingRule(120)보다 먼저
        50
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let text = word.text.as_ref();
        let Some((year_str, letter, punct)) = match_year_suffix(text) else {
            return Ok(TokenAction::Noop);
        };

        // 직전 비공백 토큰이 동일 패턴(연속 인용)인지 확인 → ⠰ (continue) 마커.
        // 아니면 ⠴ (begin) 마커.
        let prev_is_same_pattern = check_prev_is_same_pattern(tokens, index);

        let mut bytes = Vec::new();
        // ⠼ + 연도 숫자
        bytes.push(decode_unicode('⠼'));
        for c in year_str.chars() {
            bytes.push(encode_number(c)?);
        }
        // ⠴ (begin) 또는 ⠰ (continue)
        bytes.push(if prev_is_same_pattern {
            decode_unicode('⠰')
        } else {
            decode_unicode('⠴')
        });
        // letter
        bytes.push(encode_english(letter)?);
        // 구두점 — 영어 모드 inside
        // `match_year_suffix` returns Some only when punct is `,`, `;`, or `.`
        // (see lines 62-64), so no defensive `_ =>` arm is needed.
        if punct == ',' {
            bytes.push(decode_unicode('⠂'));
        } else if punct == ';' {
            bytes.push(decode_unicode('⠰'));
            bytes.push(decode_unicode('⠆'));
        } else {
            // punct == '.'
            bytes.push(decode_unicode('⠲'));
        }

        Ok(TokenAction::Replace(Token::PreEncoded(bytes)))
    }
}

/// Walk backward through `tokens[..index]` looking for the nearest non-space/
/// non-PreEncoded token. Returns true if the previous Word matches a year-suffix
/// pattern or the previous PreEncoded looks like a Rule33 emission.
fn check_prev_is_same_pattern(tokens: &[Token<'_>], index: usize) -> bool {
    let mut i = index;
    while i > 0 {
        i -= 1;
        match tokens.get(i) {
            Some(Token::Space(_)) => continue,
            Some(Token::Word(w)) => return match_year_suffix(w.text.as_ref()).is_some(),
            Some(Token::PreEncoded(bytes)) => return is_rule33_emission(bytes),
            _ => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::{SpaceKind, WordMeta, WordToken};
    use std::borrow::Cow;

    fn word_token<'a>(text: &str) -> Token<'a> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Owned(text.to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    #[test]
    fn rule_phase_priority() {
        let r = Rule33CitationYearSuffixRule;
        assert!(matches!(r.phase(), TokenPhase::Normalization));
        assert_eq!(r.priority(), 50);
    }

    /// 제33항 인용 — 연도+소문자+구두점 형식 매칭.
    #[rstest::rstest]
    #[case::valid_with_comma("1998a,", true)]
    #[case::valid_with_semicolon("2024z;", true)]
    #[case::valid_with_period("1900b.", true)]
    #[case::missing_punctuation("1998a", false)]
    #[case::too_many_letters("1998abc,", false)]
    #[case::non_digit_in_year("199xa,", false)]
    #[case::uppercase_letter("1998A,", false)]
    #[case::wrong_punctuation("1998a!", false)]
    fn match_year_suffix_paths(#[case] input: &str, #[case] is_match: bool) {
        assert_eq!(match_year_suffix(input).is_some(), is_match);
    }

    #[test]
    fn is_rule33_emission_detects_own_output() {
        // ⠼(60) + 1998 + ⠴(52) + 'a'(1) + ⠂(2)
        let bytes = vec![60, 1, 11, 11, 27, 52, 1, 2];
        assert!(is_rule33_emission(&bytes));
        // With ⠰ continue marker
        let bytes2 = vec![60, 1, 11, 11, 27, 48, 1, 2];
        assert!(is_rule33_emission(&bytes2));
        // Period suffix
        let bytes3 = vec![60, 1, 11, 11, 27, 52, 1, 50];
        assert!(is_rule33_emission(&bytes3));
        // Semicolon suffix
        let bytes4 = vec![60, 1, 11, 11, 27, 52, 1, 48, 6];
        assert!(is_rule33_emission(&bytes4));
        // Wrong length
        assert!(!is_rule33_emission(&[]));
        assert!(!is_rule33_emission(&[60, 1, 11, 11, 27, 52, 1]));
        // Wrong prefix
        assert!(!is_rule33_emission(&[59, 1, 11, 11, 27, 52, 1, 2]));
        // Non-digit byte
        assert!(!is_rule33_emission(&[60, 1, 11, 11, 99, 52, 1, 2]));
        // Wrong marker
        assert!(!is_rule33_emission(&[60, 1, 11, 11, 27, 99, 1, 2]));
        // Out-of-range letter
        assert!(!is_rule33_emission(&[60, 1, 11, 11, 27, 52, 99, 2]));
        // Unknown suffix
        assert!(!is_rule33_emission(&[60, 1, 11, 11, 27, 52, 1, 99]));
    }

    #[test]
    fn apply_non_word_noop() {
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![Token::Space(SpaceKind::Regular)];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            r.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn apply_plain_word_noop() {
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![word_token("hello")];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            r.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn apply_year_suffix_comma() {
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![word_token("1998a,")];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_year_suffix_semicolon() {
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![word_token("1998a;")];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_year_suffix_period() {
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![word_token("1998a.")];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_continuation_after_year_word() {
        // Two year-suffix tokens — second should use ⠰ continuation marker
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![
            word_token("1998a,"),
            Token::Space(SpaceKind::Regular),
            word_token("1998b,"),
        ];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 2, &mut state).unwrap();
        if let TokenAction::Replace(Token::PreEncoded(bytes)) = action {
            // Marker byte at index 5 should be 48 (⠰ continue)
            assert_eq!(bytes[5], 48);
        } else {
            panic!("expected Replace");
        }
    }

    #[test]
    fn apply_continuation_after_preencoded() {
        // Previous PreEncoded matches rule33 pattern → continue
        let r = Rule33CitationYearSuffixRule;
        let preenc_bytes = vec![60u8, 1, 11, 11, 27, 52, 1, 2];
        let tokens = vec![
            Token::PreEncoded(preenc_bytes),
            Token::Space(SpaceKind::Regular),
            word_token("1998b,"),
        ];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 2, &mut state).unwrap();
        if let TokenAction::Replace(Token::PreEncoded(bytes)) = action {
            assert_eq!(bytes[5], 48); // ⠰ continue
        } else {
            panic!("expected Replace");
        }
    }

    /// rule_33_citation:117 — backward token traversal encounters a non-Word/Space/
    /// PreEncoded token (e.g. Mode) → break false. The year-suffix word at index
    /// has a Mode token before it; loop hits `_ => break false`.
    #[test]
    fn citation_with_mode_token_before_breaks_false() {
        use crate::rules::token::ModeEvent;
        let r = Rule33CitationYearSuffixRule;
        let tokens = vec![Token::Mode(ModeEvent::EnterEnglish), word_token("1998a,")];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 1, &mut state).unwrap();
        // prev_is_same_pattern = false (Mode → break false at line 117) → begin marker ⠴ at bytes[5].
        if let TokenAction::Replace(Token::PreEncoded(bytes)) = action {
            assert_eq!(bytes[5], 52); // ⠴ begin
        } else {
            panic!("expected Replace");
        }
    }
}
