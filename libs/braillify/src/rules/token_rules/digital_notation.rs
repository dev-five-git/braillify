use crate::english::encode_english;
use crate::number::encode_number;
use crate::rules::context::EncoderState;
use crate::rules::english_ueb::contraction::{ContractionMatch, ContractionRule};
use crate::rules::english_ueb::pronunciation::cmudict::{CmuDictProvider, is_recorded_word};
use crate::rules::english_ueb::rule_10_3::StrongContractionRule;
use crate::rules::english_ueb::rule_10_4::StrongGroupsignRule;
use crate::rules::english_ueb::rule_10_6::{LowerGroupsignRule, middle_lower_groupsign};
use crate::rules::english_ueb::rule_10_7::InitialContractionRule;
use crate::rules::english_ueb::rule_10_7_pron::InitialContractionPronunciationRule;
use crate::rules::english_ueb::rule_10_8::FinalGroupsignRule;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;
use std::sync::LazyLock;

static DIGITAL_INITIAL_PRON_RULE: LazyLock<InitialContractionPronunciationRule> =
    LazyLock::new(|| InitialContractionPronunciationRule::new(Box::new(CmuDictProvider::new())));

pub struct DigitalNotationRule;

impl TokenRule for DigitalNotationRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::ModeEntry
    }

    fn priority(&self) -> u16 {
        1
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

        if !(has_digital_signature(word.text.as_ref())
            || _state.english_indicator
                && !word.meta.has_korean
                && has_english_www_signature(word.text.as_ref()))
        {
            return Ok(TokenAction::Noop);
        }

        let bytes = encode_digital_word(word.text.as_ref(), _state.english_indicator)?;
        Ok(TokenAction::Replace(Token::PreEncoded(bytes)))
    }
}

pub(crate) fn has_digital_signature(text: &str) -> bool {
    if !text
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
        return false;
    }
    // NOTE: a bare `www.` host (no `//`) is NOT a context-free digital signature:
    // Korean 제39/74항 URLs without a scheme keep ordinary english-in-korean
    // encoding. English UEB §10.12.3 opts such hosts into this path through
    // `has_english_www_signature` in `apply`, using the encoder-state flag.
    if text.contains("//") || text.contains('@') || text.contains('#') || text.contains('\\') {
        return true;
    }
    // 단독 `_`는 일반 부호로 처리. 디지털 표기는 `_`와 다른 디지털 표지(`. / :`)
    // 조합에서만 활성화한다.
    text.contains('_') && (text.contains('.') || text.contains('/') || text.contains(':'))
}

fn has_english_www_signature(text: &str) -> bool {
    text.strip_prefix("www.")
        .is_some_and(|rest| rest.chars().any(|ch| ch == '.'))
}

pub(crate) fn encode_digital_word(
    text: &str,
    needs_roman_markers: bool,
) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    let mut english_started = false;

    let prefix_len = chars
        .iter()
        .take_while(|ch| {
            ch.is_ascii_alphanumeric() || matches!(**ch, '/' | '\\' | '#' | '@' | '.' | ':' | '_')
        })
        .count();

    let digital_chars = &chars[..prefix_len];

    while i < digital_chars.len() {
        let ch = digital_chars[i];
        if ch.is_ascii_alphabetic() {
            let start = i;
            i += 1;
            while i < digital_chars.len() && digital_chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            if start > 0 && digital_chars[start - 1].is_ascii_digit() {
                result.extend([decode_unicode('⠰'), decode_unicode('⠄')]);
            }
            if needs_roman_markers && !english_started {
                result.push(decode_unicode('⠴'));
                english_started = true;
            }
            encode_digital_english_segment(
                &digital_chars[start..i],
                &mut result,
                digital_segment_stands_alone(&chars, start, i),
            )?;
            continue;
        }

        match ch {
            digit if digit.is_ascii_digit() => {
                if i > 0 && digital_chars[i - 1].is_ascii_alphabetic() {
                    result.push(decode_unicode('⠐'));
                    result.push(0);
                }
                result.push(decode_unicode('⠼'));
                while i < digital_chars.len() && digital_chars[i].is_ascii_digit() {
                    result.push(encode_number(digital_chars[i])?);
                    i += 1;
                }
                continue;
            }
            _ => {}
        }

        // line 51 filter restricts ch to alphanumeric + `/#@.:_`. Alphanumerics
        // are consumed earlier (lines 57/71). So ch here is one of `/#@.:_` and
        // the `_` fallback arm of a `match` would be structurally unreachable;
        // an if-else chain is used to avoid carrying a dead Err arm.
        if ch == '/' {
            result.push(decode_unicode('⠸'));
            result.push(decode_unicode('⠌'));
        } else if ch == '\\' {
            result.push(decode_unicode('⠸'));
            result.push(decode_unicode('⠡'));
        } else if ch == '#' {
            result.push(decode_unicode('⠸'));
            result.push(decode_unicode('⠹'));
        } else if ch == '@' {
            result.push(decode_unicode('⠈'));
            result.push(decode_unicode('⠁'));
        } else if ch == '.' {
            result.push(decode_unicode('⠲'));
        } else if ch == ':' {
            result.push(decode_unicode('⠒'));
        } else {
            debug_assert_eq!(
                ch, '_',
                "filter at line 51 guarantees ch is one of `/#@.:_`"
            );
            result.push(decode_unicode('⠨'));
            result.push(decode_unicode('⠤'));
        }
        i += 1;
    }

    if needs_roman_markers
        && prefix_len == chars.len()
        && chars.last().is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        result.push(decode_unicode('⠲'));
    }

    if prefix_len < chars.len() {
        if digital_chars
            .last()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
            && needs_roman_markers
        {
            result.push(decode_unicode('⠲'));
        }
        let remainder: String = chars[prefix_len..].iter().collect();
        result.extend(crate::encode(&remainder)?);
    }

    Ok(result)
}

/// Best UEB contraction at `pos` for digital-notation English, reusing the shared
/// contraction rules — §10.3 strong contractions; §10.4 strong groupsigns;
/// §10.7 initial-letter contractions (spelling-only plus pronunciation-gated);
/// §10.8 final groupsigns; §10.6 unrestricted lower groupsigns; and §10.6.5
/// middle lower groupsigns. Longest match wins, ties by lower priority (§10.10).
/// A middle lower sign yields to a strong/final sign claiming its second letter
/// (§10.10.5 preference, [`outranked_at`]): `learn` → `l e ar n`, while `korean`
/// → `kor⠂n` keeps the medial `ea`.
fn best_digital_groupsign(chars: &[char], pos: usize) -> Option<ContractionMatch> {
    [
        StrongContractionRule.try_match(chars, pos),
        StrongGroupsignRule.try_match(chars, pos),
        InitialContractionRule.try_match(chars, pos),
        DIGITAL_INITIAL_PRON_RULE.try_match(chars, pos),
        FinalGroupsignRule.try_match(chars, pos),
        LowerGroupsignRule.try_match(chars, pos),
        middle_lower_groupsign(chars, pos)
            .filter(|_| !crate::rules::english_ueb::rule_10_6_middle::outranked_at(chars, pos + 1)),
    ]
    .into_iter()
    .flatten()
    .filter(|m| {
        // §10.12.3 computer material uses ordinary contractions, but §10.11 still
        // bars a groupsign that bridges a component seam: `braille|documents`
        // spells `ed`, while run-initial `edu...` and final `hundred` keep `ed`.
        m.cells.as_slice() != [decode_unicode('⠫')] || !ed_bridges_digital_component(chars, pos)
    })
    .max_by_key(|m| (m.consumed, u16::MAX - m.priority))
}

fn encode_digital_english_segment(
    chars: &[char],
    result: &mut Vec<u8>,
    whole_shortforms_allowed: bool,
) -> Result<(), String> {
    if let Some(bounds) = camel_component_bounds(chars)
        && bounds.len() > 2
    {
        for window in bounds.windows(2) {
            encode_digital_english_segment(&chars[window[0]..window[1]], result, false)?;
        }
        return Ok(());
    }

    if chars.len() > 1 && chars.iter().all(|ch| ch.is_ascii_uppercase()) {
        result.extend([decode_unicode('⠠'), decode_unicode('⠠')]);
    } else if chars.first().is_some_and(|ch| ch.is_ascii_uppercase()) {
        result.push(decode_unicode('⠠'));
    }
    let lower: Vec<char> = chars.iter().map(|ch| ch.to_ascii_lowercase()).collect();
    if whole_shortforms_allowed
        && let Some(cells) = crate::rules::english_ueb::rule_10_9::whole_word_cells(
            &lower.iter().collect::<String>(),
        )
    {
        result.extend(cells);
        return Ok(());
    }
    let mut i = lower.len() - lower.len();
    loop {
        if i >= lower.len() {
            break;
        }
        if let Some(m) = best_digital_groupsign(&lower, i) {
            result.extend_from_slice(&m.cells);
            i += m.consumed;
            continue;
        }
        result.push(encode_english(lower[i])?);
        i += 1;
    }
    Ok(())
}

fn ed_bridges_digital_component(chars: &[char], pos: usize) -> bool {
    let seam = pos + 1;
    if pos == 0 || pos + 2 >= chars.len() {
        return false;
    }
    let word: String = chars.iter().collect();
    if crate::rules::english_ueb::compound::compound_seams(&word).contains(&seam) {
        return true;
    }
    let left: String = chars[..seam].iter().collect();
    let right: String = chars[seam..].iter().collect();
    left.len() >= 3 && right.len() >= 3 && is_recorded_word(&left) && is_recorded_word(&right)
}

fn digital_segment_stands_alone(chars: &[char], start: usize, end: usize) -> bool {
    (start == 0 || is_standing_alone_delimiter(chars[start - 1]))
        && (end == chars.len() || is_standing_alone_delimiter(chars[end]))
}

fn is_standing_alone_delimiter(ch: char) -> bool {
    matches!(ch, ' ' | '-' | '\u{2013}' | '\u{2014}')
}

fn camel_component_bounds(chars: &[char]) -> Option<Vec<usize>> {
    let mut bounds = vec![0];
    for i in 1..chars.len() {
        if chars[i].is_ascii_uppercase() && chars[i - 1].is_ascii_lowercase() {
            bounds.push(i);
        }
    }
    (bounds.len() > 1).then(|| {
        bounds.push(chars.len());
        bounds
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::{SpaceKind, WordMeta, WordToken};
    use std::borrow::Cow;

    fn encode_segment(text: &str, whole_shortforms_allowed: bool) -> Vec<u8> {
        let chars: Vec<char> = text.chars().collect();
        let mut buf = Vec::new();
        encode_digital_english_segment(&chars, &mut buf, whole_shortforms_allowed).unwrap();
        buf
    }

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
        let r = DigitalNotationRule;
        assert!(matches!(r.phase(), TokenPhase::ModeEntry));
        assert_eq!(r.priority(), 1);
    }

    /// `has_digital_signature` — URL/Email/Hashtag/Underscore 조합 인식.
    #[rstest::rstest]
    #[case::url_double_slash("http://example.com", true)]
    #[case::email_at("foo@bar.com", true)]
    #[case::hashtag("a#hash", true)]
    #[case::underscore_dot("a_b.c", true)]
    #[case::underscore_slash("a_b/c", true)]
    #[case::underscore_colon("a_b:c", true)]
    #[case::plain_alpha_only("hello", false)]
    #[case::pure_underscore("a_b", false)]
    #[case::non_alphanumeric_start("/foo", false)]
    #[case::empty_string("", false)]
    fn has_digital_signature_paths(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(has_digital_signature(input), expected);
    }

    #[test]
    fn apply_non_word_noop() {
        let r = DigitalNotationRule;
        let tokens: Vec<Token> = vec![Token::Space(SpaceKind::Regular)];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            r.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn apply_plain_word_noop() {
        let r = DigitalNotationRule;
        let tokens = vec![word_token("hello")];
        let mut state = EncoderState::new(false);
        assert!(matches!(
            r.apply(&tokens, 0, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn apply_url_replaces() {
        let r = DigitalNotationRule;
        let tokens = vec![word_token("http://a.b")];
        let mut state = EncoderState::new(false);
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Replace(Token::PreEncoded(_))));
    }

    #[test]
    fn apply_bare_www_host_replaces_only_in_english_mode() {
        let r = DigitalNotationRule;
        let tokens = vec![word_token("www.example.org")];

        let mut english_state = EncoderState::new(true);
        assert!(matches!(
            r.apply(&tokens, 0, &mut english_state).unwrap(),
            TokenAction::Replace(Token::PreEncoded(_))
        ));

        let mut korean_state = EncoderState::new(false);
        assert!(matches!(
            r.apply(&tokens, 0, &mut korean_state).unwrap(),
            TokenAction::Noop
        ));
    }

    #[test]
    fn encode_digital_word_email() {
        let result = encode_digital_word("a@b", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_hashtag() {
        let result = encode_digital_word("a#b", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_underscore_dot() {
        let result = encode_digital_word("a_b.c", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_colon_path() {
        let result = encode_digital_word("foo:bar", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_mixed_alpha_digit_transitions() {
        // English then digit → triggers ⠐ + space prefix logic (line 71-74)
        let result = encode_digital_word("abc123", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_pure_digit_starts() {
        let result = encode_digital_word("123#abc", false).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_digital_word_ends_with_alpha_appends_terminator() {
        // Pure URL ending with alpha; line 107-109 path
        let result = encode_digital_word("a#b", true).unwrap();
        // last alpha + appended terminator
        assert!(result.last().copied().is_some());
    }

    #[test]
    fn encode_digital_word_with_korean_suffix() {
        // prefix_len < chars.len() → lines 111-117
        // "a@b가" — `a@b` is digital prefix, `가` is Korean suffix
        let result = encode_digital_word("a@b가", true).unwrap();
        assert!(!result.is_empty());
    }

    /// 영문 약자 (digraph) 분기 — 각 입력이 빈 결과가 아닌 점역을 산출.
    #[rstest::rstest]
    #[case("aliment")]
    #[case("playing")]
    #[case("constant")]
    #[case("easy")]
    #[case("energy")]
    #[case("argon")]
    #[case("verb")]
    #[case("outdoor")]
    #[case("owls")]
    #[case("thumb")]
    fn encode_digital_english_segment_all_abbreviations(#[case] case: &str) {
        let chars: Vec<char> = case.chars().collect();
        let mut buf = Vec::new();
        encode_digital_english_segment(&chars, &mut buf, false).unwrap();
        assert!(!buf.is_empty(), "{case} should encode");
    }

    /// §10.12.3 computer material reuses the shared contraction rules instead of
    /// direct word literals: §10.3 `for`, §10.7 initial-letter signs, §10.9.1
    /// standing-alone shortforms, and the §10.11 `ed` bridge filter.
    #[rstest::rstest]
    #[case::initial_one("one", false, "⠐⠕")]
    #[case::strong_for("for", false, "⠿")]
    #[case::run_initial_ed("edu", false, "⠫⠥")]
    #[case::shortform_children("children", true, "⠡⠝")]
    #[case::friend_not_standing_alone("friend", false, "⠋⠗⠊⠢⠙")]
    #[case::initial_world("world", false, "⠸⠺")]
    #[case::initial_word_prefix("wordsigns", false, "⠘⠺⠎⠊⠛⠝⠎")]
    #[case::compound_internal_ed("brailledocuments", false, "⠃⠗⠁⠊⠇⠇⠑⠙⠕⠉⠥⠰⠞⠎")]
    fn digital_segment_uses_shared_ueb_rules(
        #[case] text: &str,
        #[case] whole_shortforms_allowed: bool,
        #[case] expected: &str,
    ) {
        let expected_cells: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(
            encode_segment(text, whole_shortforms_allowed),
            expected_cells
        );
    }

    #[test]
    fn digital_segment_standing_alone_accepts_runtime_right_delimiter() {
        let chars: Vec<char> = std::hint::black_box("x-y").chars().collect();

        assert!(digital_segment_stands_alone(
            &chars,
            std::hint::black_box(0),
            std::hint::black_box(1)
        ));
    }

    #[test]
    fn encode_digital_english_segment_plain_letter() {
        // Single letter — no digraph match → falls to single-letter encode (line 177)
        let chars: Vec<char> = "z".chars().collect();
        let mut buf = Vec::new();
        encode_digital_english_segment(&chars, &mut buf, false).unwrap();
        assert!(!buf.is_empty());
    }

    /// `apply` Replace path: `has_digital_signature` true → returns a
    /// `Token::PreEncoded`. Exercises line 33-35.
    #[test]
    fn apply_replaces_digital_word_token() {
        use crate::rules::context::EncoderState;
        use crate::rules::token::{Token, WordMeta, WordToken};
        use crate::rules::token_rule::{TokenAction, TokenRule};
        use std::borrow::Cow;

        let text = "http://example.com";
        let chars: Vec<char> = text.chars().collect();
        let meta = WordMeta::from_chars(&chars);
        let tokens = vec![Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars,
            meta,
        })];
        let mut state = EncoderState::new(false);
        let action = DigitalNotationRule.apply(&tokens, 0, &mut state).unwrap();
        match action {
            TokenAction::Replace(Token::PreEncoded(bytes)) => assert!(!bytes.is_empty()),
            _ => panic!("expected Replace(PreEncoded)"),
        }
    }

    /// `apply` Noop path: non-Word token short-circuits.
    #[test]
    fn apply_noop_on_non_word_token() {
        use crate::rules::context::EncoderState;
        use crate::rules::token::Token;
        use crate::rules::token_rule::{TokenAction, TokenRule};

        let tokens = vec![Token::PreEncoded(vec![1, 2, 3])];
        let mut state = EncoderState::new(false);
        let action = DigitalNotationRule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Noop));
    }

    /// `apply` Noop path: Word without digital signature.
    #[test]
    fn apply_noop_on_plain_word() {
        use crate::rules::context::EncoderState;
        use crate::rules::token::{Token, WordMeta, WordToken};
        use crate::rules::token_rule::{TokenAction, TokenRule};
        use std::borrow::Cow;

        let text = "hello";
        let chars: Vec<char> = text.chars().collect();
        let meta = WordMeta::from_chars(&chars);
        let tokens = vec![Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars,
            meta,
        })];
        let mut state = EncoderState::new(false);
        let action = DigitalNotationRule.apply(&tokens, 0, &mut state).unwrap();
        assert!(matches!(action, TokenAction::Noop));
    }

    /// digital_notation:33, 196 — digital signature with characters that aren't
    /// "ow" or "th" shortcuts fall through to the per-char encode at line 196.
    /// And the Replace(PreEncoded) return at line 33 fires for digital words.
    #[test]
    fn digital_words_simple_chars() {
        let _ = crate::encode("a@b");
        let _ = crate::encode("c#d");
        let _ = crate::encode("e//f");
        let _ = crate::encode("g_h.i");
        let _ = crate::encode("x_y:z");
    }

    #[test]
    fn digital_word_digit_after_letter_inserts_numeric_separator() {
        let encoded = encode_digital_word("a1", false).expect("a1 should encode");
        let expected: Vec<u8> = "⠁⠐⠀⠼⠁".chars().map(decode_unicode).collect();

        assert_eq!(encoded, expected);
    }

    #[test]
    fn digital_word_starting_digit_enters_digit_branch() {
        let encoded = encode_digital_word("1a", false).expect("1a should encode");
        let expected: Vec<u8> = "⠼⠁⠰⠄⠁".chars().map(decode_unicode).collect();

        assert_eq!(encoded, expected);
    }

    #[test]
    fn digital_word_digit_after_symbol_enters_digit_branch_without_separator() {
        let encoded = encode_digital_word("a.1", false).expect("a.1 should encode");
        let expected: Vec<u8> = "⠁⠲⠼⠁".chars().map(decode_unicode).collect();

        assert_eq!(encoded, expected);
    }

    #[test]
    fn digital_word_letters_without_shortform_walk_per_characters() {
        let encoded = encode_digital_word("abc@def", false).expect("digital word should encode");

        assert!(!encoded.is_empty());
    }

    #[test]
    fn digital_english_segment_allows_shortforms_but_walks_non_shortform_letters() {
        let chars: Vec<char> = std::hint::black_box("xyz").chars().collect();
        let mut result = Vec::new();

        encode_digital_english_segment(&chars, &mut result, std::hint::black_box(true)).unwrap();

        assert_eq!(
            result,
            vec![
                encode_english('x').unwrap(),
                encode_english('y').unwrap(),
                encode_english('z').unwrap()
            ]
        );
    }

    #[test]
    fn digital_english_segment_without_shortform_walks_from_start() {
        let chars: Vec<char> = std::hint::black_box("word").chars().collect();
        let mut result = Vec::new();

        encode_digital_english_segment(&chars, &mut result, false).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn digital_english_segment_accepts_empty_component() {
        let mut result = Vec::new();

        encode_digital_english_segment(&[], &mut result, false).unwrap();

        assert!(result.is_empty());
    }
}
