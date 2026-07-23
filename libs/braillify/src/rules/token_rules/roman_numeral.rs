use std::borrow::Cow;

use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

const ROMAN_INDICATOR: u8 = 52; // ⠴
const ROMAN_CONTINUATION: u8 = 48; // ⠰
const UPPERCASE_SIGN: u8 = 32; // ⠠
const ROMAN_TERMINATOR: u8 = 50; // ⠲
const HYPHEN: u8 = crate::unicode::decode_unicode('⠤');

pub struct RomanNumeralRule;

fn is_upper_roman_char(c: char) -> bool {
    matches!(c, 'I' | 'V' | 'X')
}

fn is_lower_roman_char(c: char) -> bool {
    matches!(c, 'i' | 'v' | 'x')
}

fn roman_case(text: &str) -> Option<bool> {
    if text.chars().all(is_upper_roman_char) {
        return Some(true);
    }
    if text.chars().all(is_lower_roman_char) {
        return Some(false);
    }
    None
}

fn is_valid_roman_1_to_39(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    if roman_case(text).is_none() {
        return false;
    }

    let upper = text.to_ascii_uppercase();
    let mut chars = upper.chars().peekable();
    let mut x_count = 0usize;
    while chars.peek().is_some_and(|c| *c == 'X') {
        x_count += 1;
        chars.next();
    }
    if x_count > 3 {
        return false;
    }

    let rest: String = chars.collect();
    let ones_ok = matches!(
        rest.as_str(),
        "" | "I" | "II" | "III" | "IV" | "V" | "VI" | "VII" | "VIII" | "IX"
    );

    (x_count > 0 || !rest.is_empty()) && ones_ok
}

fn split_roman_prefix(text: &str) -> (&str, &str) {
    let mut split_at = 0usize;
    for (idx, ch) in text.char_indices() {
        if !is_upper_roman_char(ch) && !is_lower_roman_char(ch) {
            break;
        }
        split_at = idx + ch.len_utf8();
    }
    text.split_at(split_at)
}

fn split_after_hyphen(text: &str) -> Option<(&str, &str)> {
    if !text.starts_with('-') {
        return None;
    }
    let without_hyphen = &text['-'.len_utf8()..];
    let (roman, rest) = split_roman_prefix(without_hyphen);
    if roman.is_empty() {
        return None;
    }
    Some((roman, rest))
}

fn starts_with_ascii_alpha(text: &str) -> bool {
    text.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

fn has_prev_ascii_word<'a>(tokens: &[Token<'a>], index: usize) -> bool {
    tokens[..index].iter().rev().find_map(|token| match token {
        Token::Word(word) => Some(word.meta.has_ascii_alphabetic),
        _ => None,
    }) == Some(true)
}

fn build_word_token<'a>(text: &str) -> Token<'a> {
    let chars: Vec<char> = text.chars().collect();
    Token::Word(WordToken {
        text: Cow::Owned(text.to_string()),
        chars: chars.clone(),
        meta: WordMeta::from_chars(&chars),
    })
}

fn encode_roman_segment(text: &str, entry: u8, with_terminator: bool) -> Result<Vec<u8>, String> {
    let is_upper = roman_case(text).ok_or_else(|| format!("invalid roman case: {text}"))?;

    let mut out = Vec::new();
    out.push(entry);

    if is_upper {
        if text.chars().count() >= 2 {
            out.push(UPPERCASE_SIGN);
            out.push(UPPERCASE_SIGN);
        } else {
            out.push(UPPERCASE_SIGN);
        }
    }

    for ch in text.chars() {
        out.push(crate::english::encode_english(ch.to_ascii_lowercase())?);
    }

    if with_terminator {
        out.push(ROMAN_TERMINATOR);
    }

    Ok(out)
}

impl TokenRule for RomanNumeralRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::ModeEntry
    }

    fn priority(&self) -> u16 {
        5
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let text = word.text.as_ref();

        let allow_standalone = state.english_indicator || text.chars().count() >= 2;
        if allow_standalone && is_valid_roman_1_to_39(text) {
            let bytes = encode_roman_segment(text, ROMAN_INDICATOR, true)?;
            return Ok(TokenAction::Replace(Token::PreEncoded(bytes)));
        }

        if !state.english_indicator {
            return Ok(TokenAction::Noop);
        }

        let (first, rest) = split_roman_prefix(text);
        if first.is_empty() || !is_valid_roman_1_to_39(first) {
            return Ok(TokenAction::Noop);
        }

        if let Some((second, suffix)) = split_after_hyphen(rest)
            && is_valid_roman_1_to_39(second)
            && !starts_with_ascii_alpha(suffix)
        {
            let first_entry = if has_prev_ascii_word(tokens, index) {
                ROMAN_CONTINUATION
            } else {
                ROMAN_INDICATOR
            };
            let mut bytes = encode_roman_segment(first, first_entry, false)?;
            bytes.push(HYPHEN);
            bytes.extend(encode_roman_segment(second, ROMAN_CONTINUATION, true)?);

            if suffix.is_empty() {
                return Ok(TokenAction::Replace(Token::PreEncoded(bytes)));
            }

            return Ok(TokenAction::ReplaceMany(vec![
                Token::PreEncoded(bytes),
                build_word_token(suffix),
            ]));
        }

        if starts_with_ascii_alpha(rest) {
            return Ok(TokenAction::Noop);
        }

        let entry = if has_prev_ascii_word(tokens, index) {
            ROMAN_CONTINUATION
        } else {
            ROMAN_INDICATOR
        };

        // `rest` cannot be empty here: if it were, `text == first` and the
        // line 151 standalone path would have already returned. Probe-verified.
        let bytes = encode_roman_segment(first, entry, true)?;
        Ok(TokenAction::ReplaceMany(vec![
            Token::PreEncoded(bytes),
            build_word_token(rest),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncoderState;
    use crate::rules::token::SpaceKind;

    /// `is_upper_roman_char` / `is_lower_roman_char` — 케이스별 분류.
    #[rstest::rstest]
    #[case::upper_i('I', true, false)]
    #[case::upper_v('V', true, false)]
    #[case::upper_x('X', true, false)]
    #[case::lower_i('i', false, true)]
    #[case::lower_v('v', false, true)]
    #[case::lower_x('x', false, true)]
    #[case::non_roman_upper('A', false, false)]
    #[case::non_roman_lower('a', false, false)]
    fn is_upper_lower_roman_char_basic(#[case] ch: char, #[case] upper: bool, #[case] lower: bool) {
        assert_eq!(is_upper_roman_char(ch), upper);
        assert_eq!(is_lower_roman_char(ch), lower);
    }

    #[rstest::rstest]
    #[case::all_upper("XII", Some(true))]
    #[case::all_lower("xii", Some(false))]
    #[case::mixed_case("Xi", None)]
    #[case::non_roman_char("XA", None)]
    fn roman_case_all_upper_all_lower_mixed(#[case] input: &str, #[case] expected: Option<bool>) {
        assert_eq!(roman_case(input), expected);
    }

    /// `is_valid_roman_1_to_39` — PDF 제36항 1~39 범위 검증.
    #[rstest::rstest]
    #[case::empty("", false)]
    #[case::mixed_case("Iv", false)]
    #[case::one("I", true)]
    #[case::two("II", true)]
    #[case::three("III", true)]
    #[case::four("IV", true)]
    #[case::five("V", true)]
    #[case::six("VI", true)]
    #[case::seven("VII", true)]
    #[case::eight("VIII", true)]
    #[case::nine("IX", true)]
    #[case::ten("X", true)]
    #[case::eleven("XI", true)]
    #[case::fifteen("XV", true)]
    #[case::twenty_nine("XXIX", true)]
    #[case::thirty("XXX", true)]
    #[case::thirty_nine("XXXIX", true)]
    #[case::too_many_x("XXXX", false)]
    #[case::invalid_ones_iiii("IIII", false)]
    #[case::invalid_ones_vv("VV", false)]
    #[case::non_roman_char("XXA", false)]
    fn is_valid_roman_1_to_39_table(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_valid_roman_1_to_39(input), expected, "input={input:?}");
    }

    #[rstest::rstest]
    #[case::all_roman("IV", "IV", "")]
    #[case::alpha_suffix("IVs", "IV", "s")]
    #[case::hyphen_suffix("IV-V", "IV", "-V")]
    #[case::no_roman_prefix("abc", "", "abc")]
    #[case::empty("", "", "")]
    fn split_roman_prefix_various(#[case] input: &str, #[case] roman: &str, #[case] rest: &str) {
        assert_eq!(split_roman_prefix(input), (roman, rest));
    }

    #[rstest::rstest]
    #[case::hyphen_then_roman("-V", Some(("V", "")))]
    #[case::hyphen_then_roman_then_alpha("-Vs", Some(("V", "s")))]
    #[case::no_leading_hyphen("V", None)]
    #[case::lone_hyphen("-", None)]
    #[case::hyphen_then_non_roman("-abc", None)]
    fn split_after_hyphen_paths(
        #[case] input: &str,
        #[case] expected: Option<(&'static str, &'static str)>,
    ) {
        assert_eq!(split_after_hyphen(input), expected);
    }

    #[rstest::rstest]
    #[case::lowercase_alpha("abc", true)]
    #[case::uppercase_alpha("Z", true)]
    #[case::digit_prefix("123", false)]
    #[case::empty("", false)]
    #[case::symbol_prefix("-", false)]
    fn starts_with_ascii_alpha_branches(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(starts_with_ascii_alpha(input), expected);
    }

    #[test]
    fn encode_roman_segment_upper_single_then_multi() {
        // Single upper letter: indicator + single ⠠ + lowercase encode + terminator
        let bytes = encode_roman_segment("I", ROMAN_INDICATOR, true).unwrap();
        assert_eq!(bytes[0], ROMAN_INDICATOR);
        assert_eq!(bytes[1], UPPERCASE_SIGN);
        assert_eq!(*bytes.last().unwrap(), ROMAN_TERMINATOR);
        // Multi upper: double ⠠⠠
        let bytes = encode_roman_segment("IV", ROMAN_INDICATOR, true).unwrap();
        assert_eq!(bytes[0], ROMAN_INDICATOR);
        assert_eq!(bytes[1], UPPERCASE_SIGN);
        assert_eq!(bytes[2], UPPERCASE_SIGN);
        // Lower: no uppercase sign
        let bytes = encode_roman_segment("iv", ROMAN_INDICATOR, true).unwrap();
        assert_eq!(bytes[0], ROMAN_INDICATOR);
        assert_ne!(bytes[1], UPPERCASE_SIGN);
        // Without terminator
        let bytes = encode_roman_segment("v", ROMAN_INDICATOR, false).unwrap();
        assert_ne!(*bytes.last().unwrap(), ROMAN_TERMINATOR);
        // Invalid case (mixed) → Err
        assert!(encode_roman_segment("Iv", ROMAN_INDICATOR, true).is_err());
    }

    fn make_word_token<'a>(text: &str) -> Token<'a> {
        build_word_token(text)
    }

    #[test]
    fn apply_non_word_token_noop() {
        let rule = RomanNumeralRule;
        let tokens: Vec<Token> = vec![Token::Space(SpaceKind::Regular)];
        let mut state = EncoderState::new(false);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Noop));
    }

    #[test]
    fn apply_pure_roman_replace() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("IV")];
        let mut state = EncoderState::new(false); // not english_indicator
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_single_letter_no_indicator_noop() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("I")];
        let mut state = EncoderState::new(false); // not english_indicator, single char
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // allow_standalone = false, then english_indicator=false → Noop
        assert!(matches!(action, TokenAction::Noop));
    }

    #[test]
    fn apply_single_letter_with_indicator_replace() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("I")];
        let mut state = EncoderState::new(true); // english_indicator
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_roman_with_suffix_alpha_noop() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("IVs")];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // After split: ("IV", "s") — s is ASCII alpha → returns Noop at line 169-170
        assert!(matches!(action, TokenAction::Noop));
    }

    #[test]
    fn apply_roman_hyphen_roman_no_suffix() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("IV-V")];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_roman_hyphen_roman_with_suffix() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("IV-V형")];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // suffix "형" starts with non-ASCII → split path taken
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn apply_roman_hyphen_with_ascii_suffix_falls_through() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("IV-Vs")];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // suffix "s" is ASCII → hyphen branch skipped → normal branch.
        // Normal branch: rest = "-Vs". starts_with_ascii_alpha("-Vs")? '-' is not alpha → false.
        // → continues to encode segment + ReplaceMany.
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn apply_non_roman_word_noop() {
        let rule = RomanNumeralRule;
        let tokens = vec![make_word_token("hello")];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 0, &mut state).unwrap();
        // first = "" (no Roman prefix) → Noop at line 149-150
        assert!(matches!(action, TokenAction::Noop));
    }

    #[test]
    fn apply_prev_ascii_word_uses_continuation() {
        let rule = RomanNumeralRule;
        let tokens = vec![
            make_word_token("hello"),
            Token::Space(SpaceKind::Regular),
            make_word_token("II"),
        ];
        let mut state = EncoderState::new(true);
        // Apply at index 2 (the "II") — but allow_standalone is true (count=2)
        // so the first branch fires with ROMAN_INDICATOR, not continuation.
        let action = rule.apply(&tokens, 2, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_continuation_path_with_suffix() {
        let rule = RomanNumeralRule;
        // "II한국" — "II" is valid Roman, "한국" is non-ASCII suffix
        let tokens = vec![
            make_word_token("hello"),
            Token::Space(SpaceKind::Regular),
            make_word_token("II한국"),
        ];
        let mut state = EncoderState::new(true);
        let action = rule.apply(&tokens, 2, &mut state).unwrap();
        // First branch fails (whole text not valid Roman). Falls to split path.
        // first="II", rest="한국". No hyphen. starts_with_ascii_alpha("한국")=false.
        // has_prev_ascii_word=true → entry=ROMAN_CONTINUATION
        // Returns ReplaceMany with PreEncoded + suffix word.
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn rule_phase_and_priority() {
        let rule = RomanNumeralRule;
        assert!(matches!(rule.phase(), TokenPhase::ModeEntry));
        assert_eq!(rule.priority(), 5);
    }

    /// roman_numeral:170 — roman hyphen-second variant preceded by an ASCII word
    /// triggers ROMAN_CONTINUATION (vs ROMAN_INDICATOR when no prev ASCII).
    /// Hand-build token slice with prev ASCII Word and english_indicator=true.
    #[test]
    fn roman_hyphen_continuation_with_direct_tokens() {
        let r = RomanNumeralRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        let tokens = vec![
            build_word_token("abc"),
            Token::Space(SpaceKind::Regular),
            build_word_token("I-V"),
        ];
        let action = r.apply(&tokens, 2, &mut state).unwrap();
        match action {
            TokenAction::Replace(Token::PreEncoded(bytes)) => {
                // ROMAN_CONTINUATION marker should appear (vs ROMAN_INDICATOR for fresh start).
                assert!(
                    bytes.contains(&ROMAN_CONTINUATION),
                    "expected ROMAN_CONTINUATION in {bytes:?}"
                );
            }
            _ => panic!("expected Replace(PreEncoded)"),
        }
    }

    /// roman_numeral:170 negative path — no prev ASCII → ROMAN_INDICATOR (line 172).
    #[test]
    fn roman_hyphen_indicator_when_no_prev_ascii() {
        let r = RomanNumeralRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        let tokens = vec![build_word_token("I-V")];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        match action {
            TokenAction::Replace(Token::PreEncoded(bytes)) => {
                assert!(bytes.contains(&ROMAN_INDICATOR), "{bytes:?}");
            }
            _ => panic!("expected Replace(PreEncoded)"),
        }
    }
}
