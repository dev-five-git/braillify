use crate::rules::english_shortform::{
    permits_grade1_boundary_after_run, requires_grade1_indicator,
};
use crate::rules::token::{ModeEvent, Token, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct UppercasePassageRule;

fn prev_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<&'a WordToken<'a>> {
    tokens[..index].iter().rev().find_map(|t| {
        if let Token::Word(w) = t {
            Some(w)
        } else {
            None
        }
    })
}

/// Return the next two Word tokens after `index`, in order, lazily.
///
/// The caller only needs to know whether the next 1 and 2 upcoming Word
/// tokens exist and whether they look like ASCII passages. We never need
/// the full tail of upcoming words, so avoid materializing a `Vec`. This
/// turns what was previously an O(N²) scan (one full tail-collect per
/// token application) into O(1) amortized lookahead.
fn next_two_words<'a>(
    tokens: &'a [Token<'a>],
    index: usize,
) -> (Option<&'a WordToken<'a>>, Option<&'a WordToken<'a>>) {
    let mut iter = tokens.iter().skip(index + 1).filter_map(|t| {
        if let Token::Word(w) = t {
            Some(w)
        } else {
            None
        }
    });
    let first = iter.next();
    let second = iter.next();
    (first, second)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CapitalizedGroup {
    /// First affected ASCII capital.  Opening punctuation before this position
    /// is outside capitals mode (UEB §8.5 placement).
    start: usize,
    /// First character outside the affected symbols-sequence.  A Korean gloss
    /// or closing quote attached to the final word begins here.
    end: usize,
}

fn is_opening_passage_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '\'' | '"' | '\u{2018}' | '\u{201c}' | '(' | '[' | '{' | '〈' | '《' | '「' | '『'
    )
}

fn is_closing_passage_quote(ch: char) -> bool {
    matches!(
        ch,
        '\'' | '"' | '\u{2019}' | '\u{201d}' | '〉' | '》' | '」' | '』'
    )
}

/// Locate one whitespace-delimited capitalised symbols-sequence.
///
/// `DocumentIR` deliberately preserves print whitespace and therefore keeps
/// punctuation and a Korean gloss attached to the same `WordToken` (`‘BET`,
/// `ME’이다`, `COSMO(코스모)`).  UEB §8.5 places the passage indicators inside
/// opening/closing punctuation and before a following non-Roman gloss, so the
/// token rule needs the precise affected slice instead of asking whether the
/// entire token is ASCII.
fn capitalized_group(word: &WordToken<'_>) -> Option<CapitalizedGroup> {
    if word.chars.iter().any(char::is_ascii_lowercase) {
        return None;
    }

    let start = word.chars.iter().position(char::is_ascii_uppercase)?;
    if !word.chars[..start]
        .iter()
        .copied()
        .all(is_opening_passage_punctuation)
    {
        return None;
    }

    let last_capital = word.chars.iter().rposition(char::is_ascii_uppercase)?;
    if word.chars[start..=last_capital]
        .iter()
        .any(|ch| crate::utils::is_korean_char(*ch))
    {
        return None;
    }

    let mut end = word.chars.len();
    for index in last_capital + 1..word.chars.len() {
        let ch = word.chars[index];
        let opens_attached_korean_gloss = matches!(ch, '(' | '[' | '{')
            && word.chars[index + 1..]
                .iter()
                .any(|next| crate::utils::is_korean_char(*next));
        if crate::utils::is_korean_char(ch)
            || is_closing_passage_quote(ch)
            || opens_attached_korean_gloss
        {
            end = index;
            break;
        }
    }

    Some(CapitalizedGroup { start, end })
}

fn owned_word<'a>(chars: &[char]) -> Token<'a> {
    let text = chars.iter().collect::<String>();
    Token::Word(WordToken {
        text: std::borrow::Cow::Owned(text),
        chars: chars.to_vec(),
        meta: crate::rules::token::WordMeta::from_chars(chars),
    })
}

/// A separated uppercase unit is emitted atomically by Korean rule 69,
/// including its Roman and capitalization indicators.  Do not pre-emit the
/// generic UEB word prefix for the same letters.
fn is_separated_rule_69_unit(tokens: &[Token<'_>], index: usize, word: &WordToken<'_>) -> bool {
    let Some(previous) = prev_word(tokens, index) else {
        return false;
    };
    let previous_is_number = previous.chars.iter().any(char::is_ascii_digit)
        && previous
            .chars
            .iter()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'));
    previous_is_number
        && crate::rules::korean::rule_69::complete_ascii_unit_len(&word.chars, 0).is_some()
}

fn is_single_capital_comma_item(word: &WordToken<'_>) -> bool {
    word.chars.first().is_some_and(char::is_ascii_uppercase)
        && word.chars.get(1) == Some(&',')
        && word
            .chars
            .iter()
            .filter(|ch| ch.is_ascii_uppercase())
            .count()
            == 1
}

/// 수학 제12항 [붙임 1]의 국어 문장 안 로마자 변수 나열은 각 변수를
/// 독립된 로마자 항목으로 점역한다 (`세 점 A, B, C가 있다.`). 겉모양만
/// 보면 UEB 8.5의 세 대문자 symbols-sequence와 같으므로, 앞의 국어 문맥과
/// 마지막 변수에 붙은 국어 조사를 함께 확인해 대문자 구절로 오인하지 않는다.
/// 문자의 이름은 열거하지 않고 동일한 단일 대문자 콤마 나열 전체에 적용한다.
fn is_korean_math_letter_list_start(
    tokens: &[Token<'_>],
    index: usize,
    word: &WordToken<'_>,
    upcoming_first: Option<&WordToken<'_>>,
    upcoming_second: Option<&WordToken<'_>>,
) -> bool {
    let previous_is_korean =
        prev_word(tokens, index).is_some_and(|previous| previous.meta.has_korean);
    let Some(first) = upcoming_first else {
        return false;
    };
    let Some(second) = upcoming_second else {
        return false;
    };
    let second_group = capitalized_group(second);
    let second_has_one_capital = second
        .chars
        .iter()
        .filter(|ch| ch.is_ascii_uppercase())
        .count()
        == 1;
    let second_has_attached_korean = second_group.is_some_and(|group| {
        second.chars[group.end..]
            .iter()
            .any(|ch| crate::utils::is_korean_char(*ch))
    });

    previous_is_korean
        && is_single_capital_comma_item(word)
        && is_single_capital_comma_item(first)
        && second_has_one_capital
        && second_has_attached_korean
}

impl TokenRule for UppercasePassageRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::UppercasePassage
    }

    fn priority(&self) -> u16 {
        100
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

        let mut prefix = Vec::new();
        let mut suffix = Vec::new();

        let (upcoming_first, upcoming_second) = next_two_words(tokens, index);
        let word_len = word.chars.len();
        let ascii_starts_at_beginning = word.meta.starts_with_ascii;
        let capitalized = capitalized_group(word);

        let needs_inline_entry = state.english_indicator
            && !state.is_english
            && word.meta.has_ascii_alphabetic
            && capitalized.is_some();

        let upcoming_first_group = upcoming_first.and_then(capitalized_group);
        let upcoming_second_group = upcoming_second.and_then(capitalized_group);
        let is_korean_math_letter_list =
            is_korean_math_letter_list_start(tokens, index, word, upcoming_first, upcoming_second);
        let can_start_passage = capitalized.is_some_and(|group| group.end == word_len)
            && upcoming_first
                .zip(upcoming_first_group)
                .is_some_and(|(next, group)| group.start == 0 && group.end == next.chars.len())
            && upcoming_second_group.is_some_and(|group| group.start == 0)
            && !is_korean_math_letter_list
            && !is_separated_rule_69_unit(tokens, index, word);

        if can_start_passage && !state.triple_big_english {
            let group = capitalized.expect("passage start has a capitalized group");
            let mut replacement = Vec::new();
            if group.start > 0 {
                replacement.push(owned_word(&word.chars[..group.start]));
            }
            if needs_inline_entry {
                let entry = if state.needs_english_continuation {
                    ModeEvent::EnterEnglishContinue
                } else {
                    ModeEvent::EnterEnglish
                };
                replacement.push(Token::Mode(entry));
                state.is_english = true;
                state.needs_english_continuation = false;
            }

            // UEB §5.7.2 + §10.9: inspect the initial maximal ASCII-capital
            // letters-sequence rather than the entire whitespace token. Korean
            // text or punctuation attached after that run is its boundary, not
            // part of the UEB shortform-collision decision (`AC밀란`, `CD,`).
            let uppercase_run_len = word
                .chars
                .iter()
                .skip(group.start)
                .take_while(|ch| ch.is_ascii_uppercase())
                .count();
            let uppercase_run_end = group.start + uppercase_run_len;
            let uppercase_run = word.chars[group.start..uppercase_run_end]
                .iter()
                .collect::<String>();
            let needs_grade1 = permits_grade1_boundary_after_run(&word.chars[uppercase_run_end..])
                && requires_grade1_indicator(&uppercase_run);
            if needs_grade1 {
                replacement.push(Token::Mode(ModeEvent::Grade1Indicator));
            }
            replacement.push(Token::Mode(ModeEvent::CapsPassageStart));
            replacement.push(owned_word(&word.chars[group.start..]));
            state.triple_big_english = true;
            state.has_processed_word = true;
            return Ok(TokenAction::ReplaceMany(replacement));
        }

        if word.meta.is_all_uppercase
            && !state.triple_big_english
            && ascii_starts_at_beginning
            && !is_separated_rule_69_unit(tokens, index, word)
        {
            if needs_inline_entry {
                let entry = if state.needs_english_continuation {
                    ModeEvent::EnterEnglishContinue
                } else {
                    ModeEvent::EnterEnglish
                };
                prefix.push(Token::Mode(entry));
                state.is_english = true;
                state.needs_english_continuation = false;
            }

            let uppercase_run_len = word
                .chars
                .iter()
                .take_while(|ch| ch.is_ascii_uppercase())
                .count();
            let uppercase_run = word.chars[..uppercase_run_len].iter().collect::<String>();
            let needs_grade1 = permits_grade1_boundary_after_run(&word.chars[uppercase_run_len..])
                && requires_grade1_indicator(&uppercase_run);
            if word_len >= 2 {
                if needs_grade1 {
                    prefix.push(Token::Mode(ModeEvent::Grade1Indicator));
                }
                prefix.push(Token::Mode(ModeEvent::CapsWord));
            }
        }

        let next_continues_passage = upcoming_first_group.is_some_and(|group| group.start == 0);
        if state.triple_big_english && !next_continues_passage {
            state.triple_big_english = false;

            if let Some(group) = capitalized
                && group.start == 0
                && group.end < word_len
            {
                let replacement = vec![
                    owned_word(&word.chars[..group.end]),
                    Token::Mode(ModeEvent::CapsPassageEnd),
                    owned_word(&word.chars[group.end..]),
                ];
                state.has_processed_word = true;
                return Ok(TokenAction::ReplaceMany(replacement));
            }
            suffix.push(Token::Mode(ModeEvent::CapsPassageEnd));
        }

        if !state.has_processed_word {
            state.has_processed_word = true;
        }

        if prefix.is_empty() && suffix.is_empty() {
            return Ok(TokenAction::Noop);
        }

        let mut replacement = Vec::with_capacity(prefix.len() + 1 + suffix.len());
        replacement.extend(prefix);
        replacement.push(Token::Word(word.clone()));
        replacement.extend(suffix);
        Ok(TokenAction::ReplaceMany(replacement))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncoderState;
    use crate::rules::token::{SpaceKind, WordMeta};
    use std::borrow::Cow;

    fn word(text: &str) -> Token<'static> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Owned(text.to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    fn spaced_words(words: &[&str]) -> Vec<Token<'static>> {
        let mut tokens = Vec::with_capacity(words.len().saturating_mul(2).saturating_sub(1));
        for (index, value) in words.iter().enumerate() {
            if index > 0 {
                tokens.push(Token::Space(SpaceKind::Regular));
            }
            tokens.push(word(value));
        }
        tokens
    }

    fn replacement_words<'a>(tokens: &'a [Token<'_>]) -> Vec<&'a str> {
        tokens
            .iter()
            .filter_map(|token| match token {
                Token::Word(word) => Some(word.text.as_ref()),
                _ => None,
            })
            .collect()
    }

    /// uppercase_passage:78 — `EnterEnglishContinue` arm fires when
    /// `state.needs_english_continuation` is true at the moment of inline entry.
    /// Direct apply with hand-crafted state.
    #[test]
    fn uppercase_passage_enter_english_continue_direct() {
        let r = UppercasePassageRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        state.is_english = false; // needs_inline_entry requires this
        state.needs_english_continuation = true; // selects EnterEnglishContinue arm
        // 3 uppercase words: first triggers entry, next two satisfy passage start.
        let tokens = vec![
            word("ABC"),
            Token::Space(SpaceKind::Regular),
            word("DEF"),
            Token::Space(SpaceKind::Regular),
            word("GHI"),
        ];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        // The replacement must contain Mode::EnterEnglishContinue.
        let found = matches!(action, TokenAction::ReplaceMany(ref ts)
            if ts.iter().any(|t| matches!(t, Token::Mode(ModeEvent::EnterEnglishContinue))));
        assert!(found, "expected EnterEnglishContinue Mode token");
    }

    /// uppercase_passage:80 — `EnterEnglish` arm fires when
    /// `state.needs_english_continuation` is false.
    #[test]
    fn uppercase_passage_enter_english_direct() {
        let r = UppercasePassageRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        state.is_english = false;
        state.needs_english_continuation = false;
        let tokens = vec![
            word("ABC"),
            Token::Space(SpaceKind::Regular),
            word("DEF"),
            Token::Space(SpaceKind::Regular),
            word("GHI"),
        ];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        let found = matches!(action, TokenAction::ReplaceMany(ref ts)
            if ts.iter().any(|t| matches!(t, Token::Mode(ModeEvent::EnterEnglish))));
        assert!(found, "expected EnterEnglish Mode token");
    }

    /// UEB 2.6 + 5.7.2 + 10.9.7-10.9.8: a shortform-confusable sequence gets
    /// grade 1 only at a permitted standing-alone/code boundary.  `CD` and `LLC`
    /// are the rulebook's official shortform-confusion examples.
    #[rstest::rstest]
    #[case::bare_cd("CD", true)]
    #[case::llc_before_closing_group("LLC)", true)]
    #[case::llc_before_korean_code_span("LLC회사", true)]
    #[case::cd_before_digit("CD47", false)]
    #[case::cd_before_slash("CD/ATM", false)]
    #[case::neither_s_before_plus("NEIS+", false)]
    #[case::little_m_before_opening_group("LLM(SLM)", false)]
    fn uppercase_passage_grade1_respects_letters_sequence_boundary(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let r = UppercasePassageRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        state.is_english = false;
        let tokens = vec![word(input)];
        let action = r.apply(&tokens, 0, &mut state).unwrap();
        let found = matches!(action, TokenAction::ReplaceMany(ref ts)
            if ts.iter().any(|t| matches!(t, Token::Mode(ModeEvent::Grade1Indicator))));
        assert_eq!(found, expected);
    }

    #[test]
    fn capitalized_group_rejects_korean_inside_the_capital_extent() {
        let Token::Word(word) = word("A한B") else {
            unreachable!("helper always builds a word")
        };

        assert_eq!(capitalized_group(&word), None);
    }

    #[test]
    fn shortform_collision_before_capitals_passage_gets_grade1() {
        let rule = UppercasePassageRule;
        let tokens = spaced_words(&["CD", "EF", "GH"]);
        let mut state = EncoderState::new(false);
        state.english_indicator = true;

        let TokenAction::ReplaceMany(replacement) =
            rule.apply(&tokens, 0, &mut state).expect("passage starts")
        else {
            panic!("expected a passage-start replacement")
        };

        assert!(replacement.windows(2).any(|window| matches!(
            window,
            [
                Token::Mode(ModeEvent::Grade1Indicator),
                Token::Mode(ModeEvent::CapsPassageStart)
            ]
        )));
    }

    #[rstest::rstest]
    #[case::pdf_gigabyte("GB")]
    #[case::petabyte("PB")]
    #[case::terabyte("TB")]
    fn separated_uppercase_units_do_not_preemit_ueb_modes(#[case] unit: &str) {
        let r = UppercasePassageRule;
        let mut state = EncoderState::new(false);
        state.english_indicator = true;
        let tokens = vec![word("5"), Token::Space(SpaceKind::Regular), word(unit)];

        assert!(matches!(
            r.apply(&tokens, 2, &mut state).unwrap(),
            TokenAction::Noop
        ));
    }

    /// UEB 8.5.2-8.5.3: three or more capitalised symbols-sequences use one
    /// passage indicator, and the terminator immediately follows the final
    /// affected sequence. These are official 2024 UEB examples with Korean
    /// boundary punctuation/glosses attached to exercise the mixed-script
    /// tokenisation used by `DocumentIR`.
    #[rstest::rstest]
    #[case::caution_with_quote(
        &["‘CAUTION:", "WET", "PAINT!’이다."],
        &["‘", "CAUTION:"],
        &["PAINT!", "’이다."]
    )]
    #[case::bbc_news_with_quote(
        &["“THE", "BBC", "AFRICA", "NEWS”이다."],
        &["“", "THE"],
        &["NEWS", "”이다."]
    )]
    #[case::self_made_man_with_gloss(
        &["A", "SELF-MADE", "MAN(남자)이다."],
        &["A"],
        &["MAN", "(남자)이다."]
    )]
    fn capitalized_passage_respects_attached_mixed_script_boundaries(
        #[case] words: &[&str],
        #[case] expected_start_words: &[&str],
        #[case] expected_end_words: &[&str],
    ) {
        let rule = UppercasePassageRule;
        let tokens = spaced_words(words);
        let mut state = EncoderState::new(false);
        state.english_indicator = true;

        let TokenAction::ReplaceMany(start) = rule
            .apply(&tokens, 0, &mut state)
            .expect("official capitalised passage must start")
        else {
            panic!("expected a passage-start replacement");
        };
        assert_eq!(replacement_words(&start), expected_start_words);
        assert_eq!(
            start
                .iter()
                .filter(|token| matches!(token, Token::Mode(ModeEvent::CapsPassageStart)))
                .count(),
            1
        );
        assert!(
            !start
                .iter()
                .any(|token| matches!(token, Token::Mode(ModeEvent::CapsWord)))
        );
        assert!(state.triple_big_english);

        let last_index = tokens.len() - 1;
        let TokenAction::ReplaceMany(end) = rule
            .apply(&tokens, last_index, &mut state)
            .expect("official capitalised passage must terminate")
        else {
            panic!("expected a passage-end replacement");
        };
        assert_eq!(replacement_words(&end), expected_end_words);
        assert!(matches!(
            end.get(1),
            Some(Token::Mode(ModeEvent::CapsPassageEnd))
        ));
        assert!(!state.triple_big_english);
    }

    /// 수학 제12항 [붙임 1]: 국어 문장 안에서 콤마로 나열한 단일 대문자
    /// 변수는 UEB 대문자 구절이 아니라 각각의 로마자 항목으로 유지한다.
    #[test]
    fn korean_math_letter_list_does_not_start_capitals_passage() {
        let ir = crate::rules::token::DocumentIR::parse("세 점 A, B, C가 있다.", true);
        let index = ir
            .tokens
            .iter()
            .position(|token| matches!(token, Token::Word(word) if word.text == "A,"))
            .expect("official example must contain its first Roman variable");
        let mut state = EncoderState::new(true);

        assert!(matches!(
            UppercasePassageRule
                .apply(&ir.tokens, index, &mut state)
                .expect("letter list classification must succeed"),
            TokenAction::Noop
        ));
        assert!(!state.triple_big_english);
    }
}
