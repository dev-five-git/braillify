use crate::char_struct::{CharType, KoreanChar};
use crate::english_logic;
use crate::fraction;
use crate::rules::context::{EncoderState, RuleContext};
use crate::rules::engine::RuleEngine;
use crate::rules::korean::rule_69::parse_numeric_ascii_unit_prefix;
use crate::rules::roman_mode;
use crate::rules::traits::Phase;

use super::token::{DocumentIR, ModeEvent, SpaceKind, Token, WordToken};

/// 제39항 한글표 점형 (⠸⠷). 영어 어절 사이에 끼인 한글 어절을 감싼다.
pub(crate) const HANGUL_WRAP_START_BYTES: [u8; 2] = [56, 55];
/// 제39항 한글 종료표 점형 (⠸⠾).
pub(crate) const HANGUL_WRAP_END_BYTES: [u8; 2] = [56, 62];

struct WordContext<'a> {
    prev_word: &'a str,
    remaining_words: &'a [&'a str],
}

/// Rule 29/35: a following print word which begins with Roman text or a number
/// continues the same Roman section across its intervening print space.
fn next_word_starts_roman_or_number(remaining_words: &[&str]) -> bool {
    remaining_words
        .first()
        .and_then(|word| word.chars().next())
        .is_some_and(|ch| ch.is_ascii_alphanumeric())
}

fn is_opening_english_phrase_enclosure(ch: char) -> bool {
    matches!(ch, '(' | '[' | '{' | '‘' | '“' | '"')
}

fn is_closing_english_phrase_enclosure(ch: char) -> bool {
    matches!(ch, ')' | ']' | '}' | '’' | '”' | '"')
}

/// Decide whether the Roman section beginning in `tokens[start_index]` has an
/// independently visible English-phrase context.
///
/// Korean rule 37 expands the six UEB lower wordsigns when Roman material is
/// mentioned inside Korean prose. The NIKL's rule consultation distinguishes
/// that case from an English title or sentence, where UEB 10.5 applies. We use
/// only print structure available to a plain-text encoder: at least two Roman
/// print words plus either sentence/title capitalization or a paired-enclosure
/// boundary. A lowercase metalinguistic list such as the rule-37 attachment
/// (`be, his, was, were의 ...`) therefore remains Korean context.
fn roman_section_has_english_phrase_context(tokens: &[Token<'_>], start_index: usize) -> bool {
    let mut started = false;
    let mut roman_word_count = 0usize;
    let mut has_uppercase = false;
    let mut has_enclosure_boundary = false;

    for token in tokens.iter().skip(start_index) {
        let Token::Word(word) = token else {
            if matches!(token, Token::Space(_) | Token::Mode(_)) {
                continue;
            }
            if started {
                break;
            }
            continue;
        };

        let scan_start = if started {
            let Some(first_script) = word
                .chars
                .iter()
                .position(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(*ch))
            else {
                continue;
            };
            if crate::utils::is_korean_char(word.chars[first_script]) {
                break;
            }
            let Some(first_roman) = word.chars[first_script..]
                .iter()
                .position(|ch| ch.is_ascii_alphabetic())
                .map(|offset| first_script + offset)
            else {
                continue;
            };
            first_roman
        } else {
            let Some(first_roman) = word.chars.iter().position(|ch| ch.is_ascii_alphabetic())
            else {
                continue;
            };
            first_roman
        };

        if !started {
            has_enclosure_boundary |= word.chars[..scan_start]
                .iter()
                .rev()
                .take_while(|ch| !ch.is_ascii_alphanumeric() && !crate::utils::is_korean_char(**ch))
                .any(|ch| is_opening_english_phrase_enclosure(*ch));
        }

        let section_end = word.chars[scan_start..]
            .iter()
            .position(|ch| crate::utils::is_korean_char(*ch))
            .map_or(word.chars.len(), |offset| scan_start + offset);
        let roman_slice = &word.chars[scan_start..section_end];
        // `scan_start` is selected from an ASCII alphabetic position above, so
        // this slice necessarily contains at least that Roman letter.
        let last_roman = roman_slice
            .iter()
            .rposition(|ch| ch.is_ascii_alphabetic())
            .expect("Roman section starts at an ASCII alphabetic character");

        started = true;
        roman_word_count += 1;
        has_uppercase |= roman_slice.iter().any(|ch| ch.is_ascii_uppercase());
        has_enclosure_boundary |= roman_slice[last_roman + 1..]
            .iter()
            .any(|ch| is_closing_english_phrase_enclosure(*ch));

        if section_end < word.chars.len() {
            break;
        }
    }

    roman_word_count >= 2 && (has_uppercase || has_enclosure_boundary)
}

/// 토큰의 byte 슬라이스가 한글표(⠸⠷) 점형과 일치하는지.
fn is_hangul_wrap_start(token: &Token<'_>) -> bool {
    matches!(token, Token::PreEncoded(bytes) if bytes.as_slice() == HANGUL_WRAP_START_BYTES)
}

/// 토큰의 byte 슬라이스가 한글 종료표(⠸⠾) 점형과 일치하는지.
fn is_hangul_wrap_end(token: &Token<'_>) -> bool {
    matches!(token, Token::PreEncoded(bytes) if bytes.as_slice() == HANGUL_WRAP_END_BYTES)
}

/// 어떤 토큰 직후, 공백/PreEncoded(non-wrap)을 건너뛰고 만나는 첫 토큰이
/// 한글표 시작이면 true. 한글 wrap이 영어 모드 유지를 위한 신호이므로,
/// 단어 끝의 종료표 emit을 건너뛰는 데 사용된다.
fn next_non_space_is_hangul_wrap_start<'a>(tokens: &'a [Token<'a>], after_index: usize) -> bool {
    for token in tokens.iter().skip(after_index + 1) {
        match token {
            Token::Space(_) => continue,
            t => return is_hangul_wrap_start(t),
        }
    }
    false
}

/// 어떤 토큰 직전에, 공백을 건너뛰고 만나는 첫 비공백 토큰이 한글 종료표면 true.
/// 한글 wrap 종료 후 영어 컨텍스트가 자동 재개되는 점을 알리는 데 사용한다.
fn prev_non_space_is_hangul_wrap_end<'a>(tokens: &'a [Token<'a>], before_index: usize) -> bool {
    for token in tokens[..before_index].iter().rev() {
        match token {
            Token::Space(_) => continue,
            t => return is_hangul_wrap_end(t),
        }
    }
    false
}

/// Single-line predicate for math-context Unicode chars — extracted so
/// tarpaulin attributes coverage to one line per call site (the multi-line
/// `matches!()` form suffered attribution loss on lines 68-71).
fn is_math_context_char(c: char) -> bool {
    c.is_ascii_alphabetic()
        || ('\u{2080}'..='\u{2089}').contains(&c)
        || c == '\u{00B2}'
        || c == '\u{00B3}'
        || ('\u{2070}'..='\u{2079}').contains(&c)
        || matches!(c, '∇' | '∂' | '∞' | '∫')
        || ('α'..='ω').contains(&c)
        || ('Α'..='Ω').contains(&c)
}

/// True iff `token` is a math-context Word (non-Korean with math/paren/slash chars)
/// or any PreEncoded token. Extracted as a free function so coverage is attributed
/// per-call-site instead of being lost inside a nested function.
fn token_is_math_word(token: Option<&Token<'_>>) -> bool {
    let Some(tok) = token else {
        return false;
    };
    match tok {
        Token::Word(w) => {
            !w.meta.has_korean
                && (w.chars.iter().any(|c| is_math_context_char(*c))
                    || w.chars.contains(&'(')
                    || w.chars.contains(&')')
                    || w.chars.contains(&'/'))
        }
        Token::PreEncoded(_) => true,
        _ => false,
    }
}

/// Find the word governed by a run of UEB grade-1/capital mode markers.
///
/// Korean rule 29 requires the roman indicator before the roman text, while
/// rule 28 appendix places UEB capitalization indicators immediately before
/// the capitalized roman word. Token rewriting may discover capitalization
/// before the character emitter discovers a new roman section, so the emitter
/// must establish roman mode before it emits these UEB prefix markers.
fn roman_word_after_prefix<'a>(
    tokens: &'a [Token<'a>],
    prefix_index: usize,
) -> Option<&'a WordToken<'a>> {
    for token in tokens.iter().skip(prefix_index + 1) {
        match token {
            Token::Mode(
                ModeEvent::Grade1Indicator | ModeEvent::CapsWord | ModeEvent::CapsPassageStart,
            ) => continue,
            Token::Word(word) => return Some(word),
            _ => return None,
        }
    }
    None
}

fn current_word_at_or_after<'a>(
    tokens: &'a [Token<'a>],
    index: usize,
) -> Option<&'a WordToken<'a>> {
    for token in tokens.iter().skip(index) {
        match token {
            Token::Mode(_) => continue,
            Token::Word(word) => return Some(word),
            _ => return None,
        }
    }
    None
}

fn is_separated_from_previous_word(tokens: &[Token<'_>], index: usize) -> bool {
    let mut saw_space = false;
    for token in tokens[..index].iter().rev() {
        match token {
            Token::Mode(_) => {}
            Token::Space(_) => saw_space = true,
            Token::Word(_) => return saw_space,
            _ => return false,
        }
    }
    false
}

fn previous_word_index_before(tokens: &[Token<'_>], index: usize) -> Option<usize> {
    tokens[..index]
        .iter()
        .rposition(|token| matches!(token, Token::Word(_)))
}

fn matching_group_open(close: char) -> Option<char> {
    match close {
        ')' => Some('('),
        ']' => Some('['),
        '}' => Some('{'),
        '’' => Some('‘'),
        '”' => Some('“'),
        '〉' => Some('〈'),
        '》' => Some('《'),
        '」' => Some('「'),
        '』' => Some('『'),
        '】' => Some('【'),
        '〕' => Some('〔'),
        '〗' => Some('〖'),
        '〙' => Some('〘'),
        '〛' => Some('〚'),
        _ => None,
    }
}

fn closed_enclosure_before_contains_ascii(tokens: &[Token<'_>], index: usize) -> bool {
    let Some(previous_index) = previous_word_index_before(tokens, index) else {
        return false;
    };
    let Token::Word(previous) = &tokens[previous_index] else {
        unreachable!("previous_word_index_before returns a Word token");
    };

    let mut end = previous.chars.len();
    while end > 0 && matches!(previous.chars[end - 1], ',' | ':' | ';' | '.' | '!' | '?') {
        end -= 1;
    }
    let Some(&closer) = previous.chars.get(end.saturating_sub(1)) else {
        return false;
    };
    let Some(opener) = matching_group_open(closer) else {
        return false;
    };

    let mut nesting = 1usize;
    let mut contains_ascii = false;
    for token_index in (0..=previous_index).rev() {
        let Token::Word(word) = &tokens[token_index] else {
            if matches!(tokens[token_index], Token::Space(_) | Token::Mode(_)) {
                continue;
            }
            return false;
        };
        let word_end = if token_index == previous_index {
            end - 1
        } else {
            word.chars.len()
        };
        for &ch in word.chars[..word_end].iter().rev() {
            if ch == closer {
                nesting += 1;
            } else if ch == opener {
                nesting -= 1;
                if nesting == 0 {
                    return contains_ascii;
                }
            } else if ch.is_ascii_alphabetic() {
                contains_ascii = true;
            }
        }
    }
    false
}

/// Rule 34's closing enclosure ends the enclosed Roman item without a Roman
/// terminator.  If the next whitespace-delimited item starts directly with
/// Roman text, rule 29 opens a new Roman section.  A following enclosure is
/// excluded so the official rule-32 list `(a), (e), (i)` remains one Roman
/// section and may use UEB grade-1 indicators for its single letters.
fn starts_new_roman_section_after_closed_enclosure(tokens: &[Token<'_>], index: usize) -> bool {
    if !is_separated_from_previous_word(tokens, index) {
        return false;
    }
    let Some(current) = current_word_at_or_after(tokens, index) else {
        return false;
    };

    let current_starts_roman = current
        .chars
        .iter()
        .copied()
        .find(|ch| !matches!(ch, '‘' | '“' | '\'' | '"'))
        .is_some_and(|ch| ch.is_ascii_alphabetic());
    if !current_starts_roman {
        return false;
    }
    closed_enclosure_before_contains_ascii(tokens, index)
}

fn enter_roman_before_ueb_prefix(
    tokens: &[Token<'_>],
    prefix_index: usize,
    event: ModeEvent,
    state: &mut EncoderState,
    result: &mut Vec<u8>,
) {
    let is_ueb_prefix = matches!(
        event,
        ModeEvent::Grade1Indicator | ModeEvent::CapsWord | ModeEvent::CapsPassageStart
    );
    let roman_word =
        roman_word_after_prefix(tokens, prefix_index).filter(|word| word.meta.starts_with_ascii);

    if is_ueb_prefix
        && state.english_indicator
        && !state.is_english
        && let Some(word) = roman_word
    {
        // Use the shared rule-29/35 transition so a capital word after a number
        // in the same roman section (`KBS 1 TV`) resumes without a second
        // roman indicator, while a genuinely new section receives one.
        roman_mode::enter_english_if_starting(
            state,
            &word.chars,
            word.meta.has_ascii_alphabetic,
            result,
        );
    }
}

/// PDF 수학 — `Word(math)+Space+Word(=/==/관계)+Space+Word(math)` 패턴에서
/// 등호 양옆 Space 토큰을 묵음 처리한다. 점역 결과는 `expr⠒⠒expr`로 인접한다.
fn is_math_operator_space_suppression<'a>(tokens: &'a [Token<'a>], space_idx: usize) -> bool {
    fn token_is_relation_operator_word(token: Option<&Token<'_>>) -> bool {
        match token {
            Some(Token::Word(w)) => {
                w.chars.len() <= 2
                    && w.chars.iter().all(|c| {
                        matches!(*c, '=' | '<' | '>' | '\u{2260}' | '\u{2264}' | '\u{2265}')
                    })
            }
            // PDF — MathExpressionTokenRule이 관계연산자 Word를 PreEncoded로 변환한 결과.
            // 등호/부등호/관계기호의 점역 결과는 다음과 같다 (소스: rule_3, rule_4, math_symbol_shortcut).
            // 셀 시퀀스가 정확히 일치하면 관계연산자로 본다.
            // 향후 Token 메타데이터로 의미를 보존하는 방향이 더 안전하지만, 현 구조에서는
            // 점형이 짧고 충돌 가능성이 낮은 셀들만 골라 매칭한다.
            Some(Token::PreEncoded(bytes)) => matches!(
                bytes.as_slice(),
                [18, 18]                  // ⠒⠒ : =
                | [40, 18, 18]            // ⠨⠒⠒ : ≠
                | [16, 16]                // ⠐⠐ : ≤  
                | [16, 18]                // ⠐⠒ : <
                | [18, 16] // ⠒⠐ : >
            ),
            _ => false,
        }
    }
    // 케이스 1: Space 다음이 관계 연산자 Word, 이전이 math Word/PreEncoded.
    if space_idx + 1 < tokens.len()
        && token_is_relation_operator_word(tokens.get(space_idx + 1))
        && space_idx > 0
        && token_is_math_word(tokens.get(space_idx - 1))
    {
        return true;
    }
    // 케이스 2: Space 이전이 관계 연산자 Word, 다음이 math Word/PreEncoded.
    if space_idx > 0
        && token_is_relation_operator_word(tokens.get(space_idx - 1))
        && space_idx + 1 < tokens.len()
        && token_is_math_word(tokens.get(space_idx + 1))
    {
        return true;
    }
    false
}

pub fn emit(ir: &mut DocumentIR, char_engine: &mut RuleEngine) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let word_texts = if ir.tokens.len() > 1 {
        collect_word_texts(&ir.tokens)
    } else {
        Vec::new()
    };
    let mut word_index = 0usize;

    for (idx, token) in ir.tokens.iter().enumerate() {
        match token {
            Token::Word(word) => {
                let context = if word_texts.is_empty() {
                    WordContext {
                        prev_word: "",
                        remaining_words: &[],
                    }
                } else {
                    word_context(&word_texts, word_index)
                };
                emit_word(
                    word,
                    idx,
                    &mut ir.state,
                    char_engine,
                    &ir.tokens,
                    context,
                    &mut result,
                )?;
                word_index += 1;
            }
            Token::Space(SpaceKind::Regular) => {
                if !is_math_operator_space_suppression(&ir.tokens, idx) {
                    result.push(0);
                }
            }
            Token::Mode(event) => {
                let starts_new_roman_section =
                    starts_new_roman_section_after_closed_enclosure(&ir.tokens, idx);
                let event = if *event == ModeEvent::EnterEnglishContinue && starts_new_roman_section
                {
                    ModeEvent::EnterEnglish
                } else {
                    *event
                };
                if starts_new_roman_section {
                    ir.state.needs_english_continuation = false;
                }
                let opens_fresh_roman_section = !ir.state.is_english
                    && !ir.state.roman_number_chain
                    && !ir.state.needs_english_continuation
                    && matches!(
                        event,
                        ModeEvent::EnterEnglish
                            | ModeEvent::Grade1Indicator
                            | ModeEvent::CapsWord
                            | ModeEvent::CapsPassageStart
                    );
                if opens_fresh_roman_section {
                    ir.state.roman_section_is_english_context =
                        roman_section_has_english_phrase_context(&ir.tokens, idx);
                }
                enter_roman_before_ueb_prefix(&ir.tokens, idx, event, &mut ir.state, &mut result);
                emit_mode_event(event, &mut ir.state, &mut result);
            }
            Token::Fraction(frac) => {
                if let Some(ref w) = frac.whole {
                    result.extend(fraction::encode_mixed_fraction(
                        w,
                        &frac.numerator,
                        &frac.denominator,
                    )?);
                } else {
                    result.extend(fraction::encode_fraction(
                        &frac.numerator,
                        &frac.denominator,
                    )?);
                }
                ir.state.is_number = true;
            }
            Token::PreEncoded(bytes) => {
                // 제39항 한글 wrap 점형은 영어 모드를 자동으로 휴면(⠸⠷)·재개(⠸⠾)시킨다.
                // 이렇게 하면 wrap 사이의 한글 어절은 한국어 인코더로 처리되고,
                // wrap 종료 후 이어지는 영어 어절은 영자표시(⠴) 없이 모드를 이어간다.
                if bytes.as_slice() == HANGUL_WRAP_START_BYTES {
                    ir.state.is_english = false;
                    ir.state.needs_english_continuation = false;
                    ir.state.roman_number_chain = false;
                } else if bytes.as_slice() == HANGUL_WRAP_END_BYTES {
                    ir.state.is_english = true;
                    ir.state.needs_english_continuation = false;
                }
                result.extend(bytes);
            }
        }
    }

    // End-of-stream: close triple uppercase if active (Encoder::finish).
    // 모든 production input은 word loop 내에서 triple_big_english를 close하므로
    // 이 분기는 fallback safety net. probe-verified 2026-05-24.
    if ir.state.triple_big_english {
        result.push(32);
        result.push(4);
    }

    Ok(result)
}

fn collect_word_texts<'tokens, 'source>(tokens: &'tokens [Token<'source>]) -> Vec<&'tokens str> {
    let mut word_texts = Vec::with_capacity(tokens.len().div_ceil(2));

    for token in tokens {
        if let Token::Word(word) = token {
            word_texts.push(word.text.as_ref());
        }
    }

    word_texts
}

fn word_context<'a>(word_texts: &'a [&'a str], word_index: usize) -> WordContext<'a> {
    let prev_word = word_index
        .checked_sub(1)
        .map_or("", |prev_index| word_texts[prev_index]);
    let remaining_words = &word_texts[word_index + 1..];

    WordContext {
        prev_word,
        remaining_words,
    }
}

/// Whether the next word token is separated from the current word by print
/// whitespace. Token rewrites can insert mode/pre-encoded tokens between the
/// two words, so inspect the whole intervening token span rather than only the
/// immediate successor.
fn has_space_before_next_word(tokens: &[Token<'_>], token_index: usize) -> bool {
    let mut saw_space = false;
    for token in tokens.iter().skip(token_index + 1) {
        match token {
            Token::Space(_) => saw_space = true,
            Token::Word(_) => return saw_space,
            _ => {}
        }
    }
    false
}

fn matching_group_close(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '〈' => Some('〉'),
        '《' => Some('》'),
        '「' => Some('」'),
        '『' => Some('』'),
        '【' => Some('】'),
        '〔' => Some('〕'),
        '〖' => Some('〗'),
        '〘' => Some('〙'),
        '〚' => Some('〛'),
        '‘' => Some('’'),
        '“' => Some('”'),
        _ => None,
    }
}

/// Rule 33's printed `Umm ...이라고` example treats a whitespace-separated
/// ellipsis as the punctuation ending the Roman run, so the preceding Roman
/// terminator is still omitted. Accept the Unicode ellipsis forms and the
/// three-full-stop print spelling as the same punctuation grammar.
fn word_starts_with_rule_33_ellipsis(word: &WordToken<'_>) -> bool {
    matches!(word.chars.first(), Some('…' | '⋯')) || word.chars.starts_with(&['.', '.', '.'])
}

/// Korean rules 29, 32, and 35: print whitespace around a colon does not split
/// a Roman section when the colon is followed by another Roman/number item.
/// The tokenizer represents `Alpha : Beta` as three words, so prove the item
/// after the standalone colon before treating the colon as UEB punctuation.
fn spaced_colon_connects_roman_items(tokens: &[Token<'_>], colon_index: usize) -> bool {
    let Some(Token::Word(colon)) = tokens.get(colon_index) else {
        return false;
    };
    if colon.chars.as_slice() != [':'] {
        return false;
    }

    tokens
        .iter()
        .skip(colon_index + 1)
        .find_map(|token| match token {
            Token::Space(_) | Token::Mode(_) => None,
            Token::Word(word) => Some(
                word.chars
                    .iter()
                    .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
                    .is_some_and(|ch| ch.is_ascii_alphanumeric()),
            ),
            _ => Some(false),
        })
        .unwrap_or(false)
}

/// UEB 3.1.1 and Korean rule 29 keep an ampersand inside a spaced Roman name
/// or phrase (`Marks & Spencer`, `Scan & Solution`).  The tokenizer makes the
/// ampersand its own word, so prove a Roman word on both sides before allowing
/// it to bridge the current Roman section.  A right-hand word may be attached
/// directly to the sign (`Mining &Development`).
fn spaced_ampersand_connects_roman_words(tokens: &[Token<'_>], ampersand_index: usize) -> bool {
    let Some(Token::Word(ampersand)) = tokens.get(ampersand_index) else {
        return false;
    };
    if ampersand.chars.first() != Some(&'&') {
        return false;
    }

    let left_is_roman = tokens[..ampersand_index]
        .iter()
        .rev()
        .find_map(|token| match token {
            Token::Space(_) | Token::Mode(_) => None,
            Token::Word(word) => Some(
                word.chars
                    .iter()
                    .rev()
                    .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
                    .is_some_and(|ch| ch.is_ascii_alphanumeric()),
            ),
            _ => Some(false),
        })
        .unwrap_or(false);
    if !left_is_roman {
        return false;
    }

    if ampersand
        .chars
        .get(1)
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return true;
    }
    if ampersand.chars.len() != 1 {
        return false;
    }

    tokens
        .iter()
        .skip(ampersand_index + 1)
        .find_map(|token| match token {
            Token::Space(_) | Token::Mode(_) => None,
            Token::Word(word) => Some(
                word.chars
                    .iter()
                    .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
                    .is_some_and(|ch| ch.is_ascii_alphabetic()),
            ),
            _ => Some(false),
        })
        .unwrap_or(false)
}

/// Rule 29 keeps consecutive Roman/number text in one section even across
/// print spaces. A separated enclosure continues that section only when the
/// *complete* enclosure is Roman/number text. This distinguishes
/// `GRI (Global Reporting Initiative)` from `Poison (모래성)` and from a mixed
/// gloss such as `TVB (Television - 전시광파유한공사)`.
fn separated_symbol_continues_roman_section(tokens: &[Token<'_>], token_index: usize) -> bool {
    let next_word = tokens
        .iter()
        .enumerate()
        .skip(token_index + 1)
        .find_map(|(index, token)| match token {
            Token::Word(word) => Some((index, word)),
            _ => None,
        });
    let Some((next_word_index, next_word)) = next_word else {
        return false;
    };

    if next_word.chars.first() == Some(&'&')
        && spaced_ampersand_connects_roman_words(tokens, next_word_index)
    {
        return true;
    }

    if word_starts_with_rule_33_ellipsis(next_word) {
        return true;
    }

    if spaced_colon_connects_roman_items(tokens, next_word_index) {
        return true;
    }

    // Rule 35: punctuation may introduce a numeric continuation (`'23`).
    if next_word
        .chars
        .iter()
        .find(|ch| ch.is_ascii_alphanumeric() || crate::utils::is_korean_char(**ch))
        .is_some_and(char::is_ascii_digit)
    {
        return true;
    }

    let Some(opening) = next_word.chars.first().copied() else {
        return false;
    };
    let Some(closing) = matching_group_close(opening) else {
        return false;
    };

    let mut depth = 0usize;
    let mut saw_roman_or_number = false;
    let mut saw_korean = false;
    for token in tokens.iter().skip(token_index + 1) {
        match token {
            Token::Space(_) | Token::Mode(_) => continue,
            Token::Word(word) => {
                for ch in word.chars.iter().copied() {
                    if ch == opening {
                        depth += 1;
                        continue;
                    }
                    if ch == closing {
                        // The first scanned character is the matching opener,
                        // and the function returns as soon as that level closes.
                        depth -= 1;
                        if depth == 0 {
                            return saw_roman_or_number && !saw_korean;
                        }
                        continue;
                    }
                    if depth > 0 {
                        saw_roman_or_number |= ch.is_ascii_alphanumeric();
                        saw_korean |= crate::utils::is_korean_char(ch);
                    }
                }
            }
            _ => return false,
        }
    }
    false
}

fn emit_mode_event(event: ModeEvent, state: &mut EncoderState, result: &mut Vec<u8>) {
    match event {
        ModeEvent::EnterEnglish => {
            // Korean rule 29 uses one Roman section for consecutive Roman
            // text. Token-level capitalization can discover a later word and
            // request entry again after the character emitter has already kept
            // that section open; make the explicit event idempotent at the
            // authoritative emit-state boundary.
            if !state.is_english {
                result.push(52);
            }
            state.is_english = true;
            state.needs_english_continuation = false;
            state.roman_number_chain = false;
        }
        ModeEvent::EnterEnglishContinue => {
            result.push(48);
            state.is_english = true;
            state.needs_english_continuation = false;
            state.roman_number_chain = false;
        }
        ModeEvent::CapsWord => {
            result.push(32);
            result.push(32);
        }
        ModeEvent::Grade1Indicator => {
            // ⠰ (dots 5+6, byte 48): UEB Grade-1 indicator that forces literal letter
            // reading and prevents shortform/contraction collision (UEB 5.7.2 + 10.9).
            result.push(48);
        }
        ModeEvent::CapsPassageStart => {
            result.push(32);
            result.push(32);
            result.push(32);
            state.triple_big_english = true;
        }
        ModeEvent::CapsPassageEnd => {
            result.push(32);
            result.push(4);
            state.triple_big_english = false;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_core_encoding_rules(
    engine: &mut RuleEngine,
    char_type: &CharType,
    word_chars: &[char],
    index: usize,
    is_all_uppercase: bool,
    has_korean_char: bool,
    ascii_starts_at_beginning: bool,
    roman_section_continues_from_previous_word: bool,
    state: &mut EncoderState,
    skip_count: &mut usize,
    remaining_words: &[&str],
    prev_word: &str,
    result: &mut Vec<u8>,
) -> Result<crate::rules::traits::RuleResult, String> {
    let mut ctx = RuleContext {
        word_chars,
        index,
        char_type,
        prev_word,
        remaining_words,
        has_korean_char,
        is_all_uppercase,
        ascii_starts_at_beginning,
        roman_section_continues_from_previous_word,
        skip_count,
        state,
        result,
    };
    engine.apply_phase(Phase::CoreEncoding, &mut ctx)
}

#[allow(clippy::too_many_arguments)]
fn apply_inter_character_rules(
    engine: &mut RuleEngine,
    char_type: &CharType,
    word_chars: &[char],
    index: usize,
    is_all_uppercase: bool,
    has_korean_char: bool,
    ascii_starts_at_beginning: bool,
    roman_section_continues_from_previous_word: bool,
    state: &mut EncoderState,
    skip_count: &mut usize,
    remaining_words: &[&str],
    prev_word: &str,
    result: &mut Vec<u8>,
) -> Result<crate::rules::traits::RuleResult, String> {
    let mut ctx = RuleContext {
        word_chars,
        index,
        char_type,
        prev_word,
        remaining_words,
        has_korean_char,
        is_all_uppercase,
        ascii_starts_at_beginning,
        roman_section_continues_from_previous_word,
        skip_count,
        state,
        result,
    };
    engine.apply_phase(Phase::InterCharacter, &mut ctx)
}

fn emit_word(
    word: &WordToken,
    token_index: usize,
    state: &mut EncoderState,
    char_engine: &mut RuleEngine,
    all_tokens: &[Token],
    context: WordContext<'_>,
    result: &mut Vec<u8>,
) -> Result<(), String> {
    let prev_word = context.prev_word;
    let remaining_words = context.remaining_words;
    let next_word_is_separated = has_space_before_next_word(all_tokens, token_index);
    // 다음 비공백 토큰이 한글표(⠸⠷)이면 영어 모드를 끊지 않는다 (제39항).
    let next_is_hangul_wrap = next_non_space_is_hangul_wrap_start(all_tokens, token_index);
    // 직전 비공백 토큰이 한글 종료표(⠸⠾)이면 이 토큰의 시작 문장부호도
    // 영어 컨텍스트의 일부로 본다 (제39항 wrap 재개 직후).
    let prev_is_hangul_wrap_end = prev_non_space_is_hangul_wrap_end(all_tokens, token_index);

    // ── [D] Per-character loop (encoder.rs:201-409) ──
    let word_chars = word.chars.as_slice();
    let word_len = word_chars.len();

    if word_len > 0 {
        let meta = word.meta;
        let is_all_uppercase = meta.is_all_uppercase;
        let has_korean_char = meta.has_korean;
        let has_ascii_alphabetic = meta.has_ascii_alphabetic;

        if word_chars.first().is_some_and(|ch| ch.is_ascii_digit())
            && !state.is_english
            && let Some((numeric, mut unit, consumed)) = parse_numeric_ascii_unit_prefix(word_chars)
            && consumed == word_chars.len()
        {
            let continues_roman_section = next_word_starts_roman_or_number(remaining_words);
            if continues_roman_section && unit.last() == Some(&crate::unicode::decode_unicode('⠲'))
            {
                unit.pop();
            }
            let mut encoded = crate::encode(&numeric)?;
            encoded.extend(unit);
            result.extend(encoded);
            state.is_english = continues_roman_section;
            state.needs_english_continuation = false;
            return Ok(());
        }

        if starts_new_roman_section_after_closed_enclosure(all_tokens, token_index) {
            state.needs_english_continuation = false;
        }

        // Korean Rule 35 keeps a Roman-led alphanumeric chain in the same
        // Roman section across whitespace (`MP4 Player`).  While the final
        // digit temporarily leaves `is_english` false, `roman_number_chain`
        // records that the next Roman word is a continuation rather than a new
        // Rule-37 entry word.
        let roman_section_continues_from_previous_word =
            state.is_english || state.roman_number_chain;
        let starts_fresh_roman_section = !roman_section_continues_from_previous_word
            && !state.needs_english_continuation
            && has_ascii_alphabetic;
        if starts_fresh_roman_section {
            state.roman_section_is_english_context =
                roman_section_has_english_phrase_context(all_tokens, token_index);
        }

        // English entry (제28/35/39항) — 로마자표/연속표 emit + 영어 모드 전환.
        roman_mode::enter_english_if_starting(state, word_chars, has_ascii_alphabetic, result);

        let first_ascii_index = word_chars.iter().position(|c| c.is_ascii_alphabetic());
        let ascii_starts_at_beginning = matches!(first_ascii_index, Some(0));

        let mut is_number = false;
        let mut is_big_english = false;
        let mut skip_count = 0usize;

        // Per-char loop (encoder.rs:251-409)
        for (i, c) in word_chars.iter().enumerate() {
            if skip_count > 0 {
                skip_count -= 1;
                continue;
            }

            let char_type = CharType::new(*c)?;

            // English exit state machine (encoder.rs:259-294)
            if state.english_indicator && state.is_english {
                match &char_type {
                    CharType::English(_) => {}
                    CharType::Number(_) => {
                        roman_mode::exit_english_for_roman_number_chain(state);
                    }
                    CharType::MathSymbol('+')
                        if crate::rules::token_rules::math_expression::is_roman_plus_identifier(
                            word_chars,
                        ) => {}
                    CharType::Symbol(sym) => {
                        // 한글 wrap 직후의 첫 디지털 표기 기호(. / @ # _ : -)는
                        // 영어 컨텍스트의 연속으로 본다. 예) "www.대통령.kr"에서
                        // wrap 종료 직후의 '.'는 ".kr" 영어 도메인 일부.
                        let prev_wrap_eng_continuation = i == 0
                            && prev_is_hangul_wrap_end
                            && matches!(*sym, '.' | '/' | '@' | '#' | '_' | ':' | '-')
                            && english_logic::next_ascii_letter_or_digit(
                                word_chars,
                                i,
                                remaining_words,
                            );

                        // 단어 끝의 영어 모드 유지 가능 기호(. , : ;) 직후 한글표(⠸⠷)가
                        // 이어지면, 그 기호도 영어 컨텍스트의 연속으로 본다 (제39항 wrap
                        // 직전). 예) "(Korean:" 끝의 ':'은 다음 wrap된 한글에 이어지므로
                        // 영어 점자(⠒)로 처리.
                        let next_wrap_eng_continuation = i == word_chars.len() - 1
                            && next_is_hangul_wrap
                            && matches!(*sym, '.' | ',' | ':' | ';');

                        if prev_wrap_eng_continuation
                            || next_wrap_eng_continuation
                            || (*sym == '&'
                                && spaced_ampersand_connects_roman_words(all_tokens, token_index))
                            || english_logic::should_render_symbol_as_english(
                                state.english_indicator,
                                state.is_english,
                                state.doc_summary.is_english_majority,
                                &state.parenthesis_stack,
                                *sym,
                                word_chars,
                                i,
                                remaining_words,
                            )
                            || english_logic::should_keep_english_mode_for_symbol(
                                *sym,
                                word_chars,
                                i,
                                remaining_words,
                            )
                        {
                        } else if english_logic::should_force_terminator_before_symbol(*sym)
                            || !english_logic::should_skip_terminator_for_symbol(*sym)
                        {
                            result.push(50);
                            roman_mode::exit_english(state, false);
                        } else {
                            roman_mode::exit_english(
                                state,
                                *sym != ')' && english_logic::should_request_continuation(*sym),
                            );
                        }
                    }
                    _ => {
                        result.push(50);
                        roman_mode::exit_english(state, false);
                    }
                }
            }

            // Pre-engine type-specific checks (encoder.rs:296-327)
            if state.roman_number_chain && !state.is_english {
                match &char_type {
                    CharType::English(_) => {
                        // Korean rule 35 keeps adjacent Roman letters and digits in
                        // one Roman section. Under UEB 6.5.2, lowercase a-j still
                        // need grade 1 after a digit because their cells are numeric;
                        // a capital indicator or a lowercase k-z cell is sufficient
                        // for every other Roman letter class.
                        if matches!(*c, 'a'..='j') {
                            result.push(crate::rules::korean::rule_29::ENGLISH_CONTINUATION);
                        }
                        roman_mode::resume_english_from_roman_number_chain(state);
                    }
                    CharType::Number(_) => {}
                    CharType::MathSymbol('+')
                        if crate::rules::token_rules::math_expression::is_roman_plus_identifier(
                            word_chars,
                        ) =>
                    {
                        roman_mode::resume_english_from_roman_number_chain(state);
                    }
                    CharType::Symbol(symbol)
                        if crate::rules::korean::rule_69::is_compatibility_unit_presentation(
                            *symbol,
                        ) || (*symbol == '-'
                            && word_chars
                                .get(i + 1)
                                .is_some_and(|next| next.is_ascii_alphanumeric()))
                            || (*symbol == '*'
                                && english_logic::is_attached_ascii_roman_asterisk(
                                    word_chars, i,
                                )) => {}
                    _ => {
                        state.roman_number_chain = false;
                    }
                }
            }

            match &char_type {
                CharType::Korean(_) | CharType::KoreanPart(_) => {
                    state.needs_english_continuation = false;
                }
                CharType::Number(_) => {}
                _ => {}
            }

            // CoreEncoding via engine (encoder.rs:330-360)
            state.is_number = is_number;
            state.is_big_english = is_big_english;
            apply_core_encoding_rules(
                char_engine,
                &char_type,
                word_chars,
                i,
                is_all_uppercase,
                has_korean_char,
                ascii_starts_at_beginning,
                roman_section_continues_from_previous_word,
                state,
                &mut skip_count,
                remaining_words,
                prev_word,
                result,
            )?;
            is_number = state.is_number;
            is_big_english = state.is_big_english;

            // InterCharacter via engine (encoder.rs:362-402)
            if let CharType::Korean(ref korean) = char_type
                && i < word_len - 1
            {
                let recon_type = CharType::Korean(KoreanChar {
                    cho: korean.cho,
                    jung: korean.jung,
                    jong: korean.jong,
                });
                state.is_number = is_number;
                state.is_big_english = is_big_english;
                apply_inter_character_rules(
                    char_engine,
                    &recon_type,
                    word_chars,
                    i,
                    is_all_uppercase,
                    has_korean_char,
                    ascii_starts_at_beginning,
                    roman_section_continues_from_previous_word,
                    state,
                    &mut skip_count,
                    remaining_words,
                    prev_word,
                    result,
                )?;
                is_number = state.is_number;
                is_big_english = state.is_big_english;
            }

            // Post-char state reset (encoder.rs:403-408)
            if !c.is_numeric() {
                is_number = false;
            }
            if c.is_ascii_alphabetic() && !c.is_uppercase() {
                is_big_english = false;
            }
        }
    }

    // ── [F] Post-loop: English termination for next word (encoder.rs:424-482) ──
    // Space between words is handled by Token::Space, NOT emitted here.
    // 제39항: 다음 토큰이 한글표(⠸⠷)이면 영어 모드를 끊지 않는다.
    // 한글표 emit 시점에 영어 모드가 자동 휴면되고, 한글 종료표(⠸⠾)에서 재개된다.
    if state.english_indicator && state.is_english && next_is_hangul_wrap {
        // 한글 wrap이 영어 모드 전환을 책임지므로 여기서는 아무 것도 emit하지 않는다.
    } else if state.english_dominant_no_indicator && state.english_indicator && state.is_english {
        // 영어 주도 문서: 영어 단어 사이의 종료표 ⠲ 모두 생략하고 영어 모드를 유지.
    } else if state.english_indicator && state.is_english {
        if remaining_words.is_empty() {
            result.push(50);
            roman_mode::exit_english(state, false);
        } else if let Some(next_word) = remaining_words.first() {
            let ascii_letters = next_word
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<Vec<_>>();
            let has_invalid_symbol = next_word.chars().any(|ch| {
                !(ch.is_ascii_alphabetic()
                    || english_logic::is_english_symbol(ch)
                    || crate::symbol_shortcut::is_symbol_char(ch)
                    || crate::utils::is_korean_char(ch))
            });
            let starts_with_roman_letter = next_word
                .chars()
                .find(|ch| ch.is_ascii_alphabetic() || crate::utils::is_korean_char(*ch))
                .is_some_and(|ch| ch.is_ascii_alphabetic());
            let is_single_letter_word = ascii_letters.len() == 1
                && !next_word.chars().any(|ch| ch.is_ascii_digit())
                && starts_with_roman_letter
                && !has_invalid_symbol;

            if is_single_letter_word
                && english_logic::requires_single_letter_continuation(ascii_letters[0])
            {
                roman_mode::exit_english(state, true);
            } else if let Some(next_char) = next_word.chars().next() {
                if let Ok(next_type) = CharType::new(next_char) {
                    match next_type {
                        CharType::English(_) | CharType::Number(_) => {}
                        CharType::Symbol(sym) => {
                            let separated_continuation = next_word_is_separated
                                && separated_symbol_continues_roman_section(
                                    all_tokens,
                                    token_index,
                                );
                            // Rule 33/34 terminator omission applies when the
                            // punctuation is attached to the Roman run. If the
                            // print has whitespace first (`Poison (모래성)`),
                            // Rule 29 closes the Roman run before that space.
                            if next_word_is_separated && !separated_continuation {
                                result.push(50);
                                roman_mode::exit_english(state, false);
                            } else if separated_continuation && sym == '&' {
                                // A standalone ampersand joining Roman words is
                                // itself part of the current Roman section.
                            } else if state.english_indicator
                                && state.is_english
                                && english_logic::is_english_symbol(sym)
                            {
                                // 연속되는 영어 구절 사이에 오는 영어 문장 부호는
                                // 로마자 구간을 유지한다.
                            } else if english_logic::should_force_terminator_before_symbol(sym)
                                || !english_logic::should_skip_terminator_for_symbol(sym)
                            {
                                result.push(50);
                                roman_mode::exit_english(state, false);
                            } else {
                                roman_mode::exit_english(
                                    state,
                                    english_logic::should_request_continuation(sym),
                                );
                            }
                        }
                        _ => {
                            result.push(50);
                            roman_mode::exit_english(state, false);
                        }
                    }
                } else {
                    result.push(50);
                    roman_mode::exit_english(state, false);
                }
            }
        }
    }

    // ── [G] has_processed_word (encoder.rs:501-504) ──
    if !state.has_processed_word {
        state.has_processed_word = true;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::encode;
    use crate::rules::korean::rule_1::Rule1;
    use crate::utils;

    use super::*;

    fn english_indicator(text: &str) -> bool {
        text.split(' ')
            .filter(|word| !word.is_empty())
            .any(|word| word.chars().any(utils::is_korean_char))
    }

    fn make_char_engine() -> RuleEngine {
        let mut engine = RuleEngine::new();
        engine.register(Box::new(crate::rules::korean::rule_53::Rule53));
        engine.register(Box::new(crate::rules::korean::rule_18::Rule18));
        engine.register(Box::new(crate::rules::korean::rule_44::Rule44));
        engine.register(Box::new(crate::rules::korean::rule_16::Rule16));
        engine.register(Box::new(crate::rules::korean::rule_14::Rule14));
        engine.register(Box::new(crate::rules::korean::rule_13::Rule13));
        engine.register(Box::new(crate::rules::korean::rule_korean::RuleKorean));
        engine.register(Box::new(crate::rules::korean::rule_28::Rule28));
        engine.register(Box::new(crate::rules::korean::rule_40::Rule40));
        engine.register(Box::new(crate::rules::korean::rule_8::Rule8));
        engine.register(Box::new(Rule1));
        engine.register(Box::new(crate::rules::korean::rule_2::Rule2));
        engine.register(Box::new(crate::rules::korean::rule_3::Rule3));
        engine.register(Box::new(
            crate::rules::korean::rule_english_symbol::RuleEnglishSymbol,
        ));
        engine.register(Box::new(crate::rules::korean::rule_61::Rule61));
        engine.register(Box::new(crate::rules::korean::rule_41::Rule41));
        engine.register(Box::new(crate::rules::korean::rule_56::Rule56));
        engine.register(Box::new(crate::rules::korean::rule_57::Rule57));
        engine.register(Box::new(crate::rules::korean::rule_58::Rule58));
        engine.register(Box::new(crate::rules::korean::rule_60::Rule60));
        engine.register(Box::new(crate::rules::korean::rule_49::Rule49));
        engine.register(Box::new(crate::rules::korean::rule_space::RuleSpace));
        engine.register(Box::new(crate::rules::korean::rule_math::RuleMath));
        engine.register(Box::new(crate::rules::korean::rule_fraction::RuleFraction));
        engine.register(Box::new(crate::rules::korean::rule_11::Rule11));
        engine.register(Box::new(crate::rules::korean::rule_12::Rule12));
        engine
    }

    fn make_token_engine() -> crate::rules::token_engine::TokenRuleEngine {
        let mut engine = crate::rules::token_engine::TokenRuleEngine::new();
        engine.register(Box::new(
            crate::rules::token_rules::normalize::NormalizeEllipsis,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::emphasis_ring::EmphasisRingRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::latex_fraction::LatexFractionRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::inline_fraction::InlineFractionRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::word_shortcut::WordShortcutRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::uppercase_passage::UppercasePassageRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::middle_dot_spacing::MiddleDotSpacingRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::quote_attachment::QuoteAttachmentRule,
        ));
        engine.register(Box::new(
            crate::rules::token_rules::spacing::AsteriskSpacingRule,
        ));
        engine
    }

    fn word_token(text: &'static str) -> Token<'static> {
        let chars = text.chars().collect::<Vec<_>>();
        Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars: chars.clone(),
            meta: super::super::token::WordMeta::from_chars(&chars),
        })
    }

    /// Helper: round-trip test via emit(parse(text)) == encode(text)
    fn assert_round_trip(text: &str) {
        let mut ir = DocumentIR::parse(text, english_indicator(text));
        let mut engine = make_char_engine();
        let mut token_engine = make_token_engine();
        let state_before_token_rules = ir.state.clone();
        token_engine
            .apply_all(&mut ir.tokens, &mut ir.state)
            .unwrap();
        ir.state = state_before_token_rules;
        let emitted = emit(&mut ir, &mut engine).unwrap();
        let expected = encode(text).unwrap();
        assert_eq!(
            emitted, expected,
            "round-trip mismatch for {:?}\n  emit:   {:?}\n  encode: {:?}",
            text, emitted, expected
        );
    }

    #[rstest::rstest]
    #[case::capitalized_parenthetical("논문(Frontiers in Drug Delivery)에", "Frontiers", true)]
    #[case::capitalized_unenclosed("Nuclear Week in Parliament에 참석했다.", "Nuclear", true)]
    #[case::lowercase_enclosed("제목(plain words in context)이다.", "plain", true)]
    #[case::rule_37_metalinguistic_list("be, his, was, were의 약자를 바르게 쓰시오.", "be,", false)]
    #[case::single_roman_annotation("논문(Cell)이 발표됐다.", "논문(Cell)이", false)]
    #[case::numeric_word_before_phrase("123 Alpha Beta", "123", true)]
    #[case::numeric_word_inside_phrase("Alpha 123 Beta", "Alpha", true)]
    #[case::korean_word_ends_phrase("Alpha 한국 Beta", "Alpha", false)]
    fn recognizes_structural_english_phrase_context(
        #[case] input: &str,
        #[case] first_roman_word: &str,
        #[case] expected: bool,
    ) {
        let ir = DocumentIR::parse(input, true);
        let start_index = ir
            .tokens
            .iter()
            .position(
                |token| matches!(token, Token::Word(word) if word.text.contains(first_roman_word)),
            )
            .expect("test phrase must have a first Roman word");

        assert_eq!(
            roman_section_has_english_phrase_context(&ir.tokens, start_index),
            expected
        );
    }

    #[test]
    fn english_phrase_scan_handles_non_text_boundaries() {
        let tokens = vec![
            word_token("Alpha"),
            Token::Space(SpaceKind::Regular),
            word_token("Beta"),
            Token::PreEncoded(vec![1]),
        ];
        let leading_boundary = vec![
            Token::PreEncoded(vec![1]),
            word_token("Alpha"),
            Token::Space(SpaceKind::Regular),
            word_token("Beta"),
        ];

        assert!(roman_section_has_english_phrase_context(&tokens, 0));
        assert!(roman_section_has_english_phrase_context(
            &leading_boundary,
            0
        ));
    }

    #[test]
    fn previous_word_separation_scan_crosses_mode_markers() {
        let tokens = vec![
            word_token("Alpha"),
            Token::Space(SpaceKind::Regular),
            Token::Mode(ModeEvent::CapsWord),
        ];

        assert!(is_separated_from_previous_word(&tokens, tokens.len()));
        assert!(!is_separated_from_previous_word(&tokens, 1));
    }

    #[test]
    fn current_word_lookup_handles_hard_boundaries_and_end_of_stream() {
        assert!(current_word_at_or_after(&[Token::PreEncoded(vec![1])], 0).is_none());
        assert!(current_word_at_or_after(&[], 0).is_none());
    }

    #[rstest::rstest]
    #[case::parenthesis('(', ')')]
    #[case::square_bracket('[', ']')]
    #[case::curly_brace('{', '}')]
    #[case::single_quote('‘', '’')]
    #[case::double_quote('“', '”')]
    #[case::single_angle('〈', '〉')]
    #[case::double_angle('《', '》')]
    #[case::corner_bracket('「', '」')]
    #[case::white_corner_bracket('『', '』')]
    #[case::lenticular_bracket('【', '】')]
    #[case::tortoise_shell_bracket('〔', '〕')]
    #[case::white_lenticular_bracket('〖', '〗')]
    #[case::white_tortoise_shell_bracket('〘', '〙')]
    #[case::white_square_bracket('〚', '〛')]
    fn enclosure_delimiter_pairs_are_bidirectional(#[case] opening: char, #[case] closing: char) {
        assert_eq!(matching_group_open(closing), Some(opening));
        assert_eq!(matching_group_close(opening), Some(closing));
    }

    #[rstest::rstest]
    #[case::no_previous_word(None, false)]
    #[case::empty_previous(Some(""), false)]
    #[case::punctuation_only_previous(Some("..."), false)]
    #[case::nested_enclosure(Some("((A))"), true)]
    #[case::missing_opener(Some("A)"), false)]
    fn closed_enclosure_scans_only_a_complete_ascii_group(
        #[case] previous: Option<&'static str>,
        #[case] expected: bool,
    ) {
        let tokens = previous.map_or_else(
            || vec![Token::Space(SpaceKind::Regular)],
            |text| vec![word_token(text)],
        );

        assert_eq!(
            closed_enclosure_before_contains_ascii(&tokens, tokens.len()),
            expected
        );
    }

    #[test]
    fn new_section_probe_handles_a_mode_prefix_without_a_current_word() {
        let tokens = vec![
            word_token("(A)"),
            Token::Space(SpaceKind::Regular),
            Token::Mode(ModeEvent::CapsWord),
        ];

        assert!(!starts_new_roman_section_after_closed_enclosure(&tokens, 2));
    }

    #[test]
    fn spaced_colon_rejects_non_words_and_non_textual_right_boundaries() {
        assert!(!spaced_colon_connects_roman_items(
            &[Token::Space(SpaceKind::Regular)],
            0
        ));
        let tokens = vec![
            word_token(":"),
            Token::Space(SpaceKind::Regular),
            Token::PreEncoded(vec![1]),
        ];
        assert!(!spaced_colon_connects_roman_items(&tokens, 0));
    }

    #[rstest::rstest]
    #[case::non_word_at_index("non_word")]
    #[case::non_textual_left_boundary("non_text_left")]
    #[case::non_roman_left_word("korean_left")]
    #[case::malformed_attached_suffix("long_ampersand")]
    #[case::non_textual_right_boundary("non_text_right")]
    fn spaced_ampersand_rejects_incomplete_roman_neighbors(#[case] scenario: &str) {
        let (tokens, index) = match scenario {
            "non_word" => (vec![Token::Space(SpaceKind::Regular)], 0),
            "non_text_left" => (
                vec![
                    Token::PreEncoded(vec![1]),
                    Token::Space(SpaceKind::Regular),
                    word_token("&"),
                ],
                2,
            ),
            "korean_left" => (
                vec![
                    word_token("한국"),
                    Token::Space(SpaceKind::Regular),
                    word_token("&"),
                ],
                2,
            ),
            "long_ampersand" => (
                vec![
                    word_token("Alpha"),
                    Token::Space(SpaceKind::Regular),
                    word_token("&?"),
                ],
                2,
            ),
            "non_text_right" => (
                vec![
                    word_token("Alpha"),
                    Token::Space(SpaceKind::Regular),
                    word_token("&"),
                    Token::Space(SpaceKind::Regular),
                    Token::PreEncoded(vec![1]),
                ],
                2,
            ),
            _ => unreachable!("unknown fixture"),
        };

        assert!(!spaced_ampersand_connects_roman_words(&tokens, index));
    }

    #[rstest::rstest]
    #[case::no_following_word("no_next", false)]
    #[case::empty_following_word("empty", false)]
    #[case::nested_complete_group("nested", true)]
    #[case::non_textual_group_body("non_text", false)]
    #[case::unclosed_group("unclosed", false)]
    fn separated_symbol_requires_a_complete_roman_group(
        #[case] scenario: &str,
        #[case] expected: bool,
    ) {
        let tokens = match scenario {
            "no_next" => vec![word_token("Alpha")],
            "empty" => vec![
                word_token("Alpha"),
                Token::Space(SpaceKind::Regular),
                word_token(""),
            ],
            "nested" => vec![
                word_token("Alpha"),
                Token::Space(SpaceKind::Regular),
                word_token("((Beta))"),
            ],
            "non_text" => vec![
                word_token("Alpha"),
                Token::Space(SpaceKind::Regular),
                word_token("("),
                Token::PreEncoded(vec![1]),
            ],
            "unclosed" => vec![
                word_token("Alpha"),
                Token::Space(SpaceKind::Regular),
                word_token("(Beta"),
            ],
            _ => unreachable!("unknown fixture"),
        };

        assert_eq!(
            separated_symbol_continues_roman_section(&tokens, 0),
            expected
        );
    }

    #[test]
    fn slash_forces_a_terminator_before_leaving_roman_mode() {
        let mut ir = DocumentIR::parse("ABC/한글", true);
        let mut engine = make_char_engine();

        let output = emit(&mut ir, &mut engine).expect("mixed Roman/Korean word must encode");

        assert!(output.contains(&crate::unicode::decode_unicode('⠲')));
        assert!(!ir.state.is_english);
    }

    #[test]
    fn forced_symbol_between_adjacent_word_tokens_terminates_roman_mode() {
        let tokens = vec![word_token("ABC"), word_token("/")];
        let Token::Word(word) = &tokens[0] else {
            unreachable!("fixture begins with a word")
        };
        let remaining_words = ["/"];
        let mut state = EncoderState::new(true);
        state.is_english = true;
        let mut engine = make_char_engine();
        let mut result = Vec::new();

        emit_word(
            word,
            0,
            &mut state,
            &mut engine,
            &tokens,
            WordContext {
                prev_word: "",
                remaining_words: &remaining_words,
            },
            &mut result,
        )
        .expect("Roman word must encode");

        assert_eq!(result.last(), Some(&50));
        assert!(!state.is_english);
    }

    // ── Step 1-3: Basic token tests ──

    /// `emit` 결과가 `encode()` 와 byte-identical 한지 (round-trip) 다양한
    /// 입력에 대해 일관되게 통과하는지 검증한다. 각 case는 다른 점역 규칙
    /// 경로를 통과한다 — 한글/영어/대문자/숫자/약어/LaTeX/전화번호/괄호 등.
    #[rstest::rstest]
    #[case::korean_greeting("안녕하세요")]
    #[case::english_words("hello world는")]
    #[case::triple_uppercase_passage("WELCOME TO KOREA")]
    #[case::english_indicator_sns("SNS에서")]
    #[case::english_indicator_atm("ATM 기기")]
    #[case::english_indicator_bmi_paren("BMI(지수)")]
    #[case::mixed_upper_atm("ATM")]
    #[case::mixed_upper_capitalized("Contents는")]
    #[case::mixed_upper_title("Table of Contents는")]
    #[case::number_with_comma("1,000")]
    #[case::number_decimal("0.48")]
    #[case::multi_word_korean("상상이상의 ")]
    #[case::korean_with_newline("안녕\n반가워")]
    #[case::word_shortcut_geuraeseo("그래서")]
    #[case::word_shortcut_geureona("그러나")]
    #[case::latex_fraction_half("$\\frac{1}{2}$")]
    #[case::math_symbols_korean_sentence("나루 + 배 = 나룻배")]
    #[case::phone_number_range("02-2669-9775~6")]
    #[case::parenthesized_english_bmi("지수(BMI)")]
    #[case::parenthesized_english_chejilryang_bmi("체질량 지수(BMI)")]
    #[case::standalone_jamo("삼각형 ㄱㄴㄷ")]
    #[case::kg_parenthesized("(kg)는")]
    #[case::kg_bare("kg")]
    #[case::roma_bracket("Roma [ㄹㄹ로마]")]
    fn emit_round_trip(#[case] text: &str) {
        assert_round_trip(text);
    }

    #[test]
    fn mode_events_emit_expected_bytes() {
        let mut ir = DocumentIR {
            tokens: vec![
                Token::Mode(ModeEvent::EnterEnglish),
                Token::Mode(ModeEvent::EnterEnglishContinue),
                Token::Mode(ModeEvent::CapsWord),
                Token::Mode(ModeEvent::CapsPassageStart),
                Token::Mode(ModeEvent::CapsPassageEnd),
                Token::Mode(ModeEvent::Grade1Indicator),
            ],
            state: EncoderState::new(false),
        };
        let mut engine = make_char_engine();
        let out = emit(&mut ir, &mut engine).unwrap();
        assert_eq!(out, vec![52, 48, 32, 32, 32, 32, 32, 32, 4, 48]);
    }

    /// Korean rules 28 appendix and 29: in Korean prose the roman indicator
    /// precedes the UEB capital-word indicator (`0,,KTX`, not `,,0KTX`).
    #[test]
    fn roman_indicator_precedes_capital_prefix_without_explicit_entry_token() {
        let chars = "KTX".chars().collect::<Vec<_>>();
        let mut ir = DocumentIR {
            tokens: vec![
                Token::Mode(ModeEvent::CapsWord),
                Token::Word(WordToken {
                    text: Cow::Borrowed("KTX"),
                    chars: chars.clone(),
                    meta: super::super::token::WordMeta::from_chars(&chars),
                }),
            ],
            state: EncoderState::new(true),
        };
        let mut engine = make_char_engine();

        let out = emit(&mut ir, &mut engine).unwrap();

        assert!(out.starts_with(&[52, 32, 32]));
    }

    #[test]
    fn explicit_roman_entry_is_not_duplicated_before_capital_prefix() {
        let chars = "KTX".chars().collect::<Vec<_>>();
        let mut ir = DocumentIR {
            tokens: vec![
                Token::Mode(ModeEvent::EnterEnglish),
                Token::Mode(ModeEvent::CapsWord),
                Token::Word(WordToken {
                    text: Cow::Borrowed("KTX"),
                    chars: chars.clone(),
                    meta: super::super::token::WordMeta::from_chars(&chars),
                }),
            ],
            state: EncoderState::new(true),
        };
        let mut engine = make_char_engine();

        let out = emit(&mut ir, &mut engine).unwrap();

        assert!(out.starts_with(&[52, 32, 32]));
        assert_eq!(out.iter().filter(|byte| **byte == 52).count(), 1);
    }

    /// Korean rule 29: consecutive Roman text shares one Roman section. A
    /// token-level rediscovery of capitalization must not emit a second entry
    /// when the final character emitter is still in that section.
    #[test]
    fn repeated_explicit_roman_entry_is_idempotent_in_active_section() {
        let new_chars = "NEW".chars().collect::<Vec<_>>();
        let york_chars = "YORK".chars().collect::<Vec<_>>();
        let mut ir = DocumentIR {
            tokens: vec![
                Token::Mode(ModeEvent::EnterEnglish),
                Token::Mode(ModeEvent::CapsWord),
                Token::Word(WordToken {
                    text: Cow::Borrowed("NEW"),
                    chars: new_chars.clone(),
                    meta: super::super::token::WordMeta::from_chars(&new_chars),
                }),
                Token::Space(SpaceKind::Regular),
                Token::Mode(ModeEvent::EnterEnglish),
                Token::Mode(ModeEvent::CapsWord),
                Token::Word(WordToken {
                    text: Cow::Borrowed("YORK"),
                    chars: york_chars.clone(),
                    meta: super::super::token::WordMeta::from_chars(&york_chars),
                }),
            ],
            state: EncoderState::new(true),
        };
        let mut engine = make_char_engine();

        let out = emit(&mut ir, &mut engine).unwrap();

        assert_eq!(out.iter().filter(|byte| **byte == 52).count(), 1);
    }

    /// Rules 29 and 33: whitespace closes the Roman run before a following
    /// parenthetical; Rule 34's omission is only for an attached enclosure.
    #[rstest::rstest]
    #[case::ordinary_word("Poison (모래성)", "⠝⠲⠀")]
    #[case::mixed_roman_korean_gloss("그룹 TVB (Television - 전시광파유한공사)", "⠃⠲⠀")]
    #[case::roman_number_chain("8PM (최초)", "⠍⠲⠀")]
    fn spaced_parenthetical_follows_a_closed_roman_run(
        #[case] input: &str,
        #[case] expected_boundary: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains(expected_boundary),
            "missing Rule 29 terminator at spaced boundary: {actual}"
        );
    }

    /// Rules 29 and 34: an enclosure closes its own Roman item without a
    /// terminator, but a following unenclosed Roman item starts a new section.
    #[rstest::rstest]
    #[case::capitalized_after_comma("가는 설명(ABC), Next 나다", "⠠⠴⠐⠀⠴⠠⠝")]
    #[case::all_caps_after_parenthesis("가는 설명(ABC) XYZ 나다", "⠠⠴⠀⠴⠠⠠⠭")]
    #[case::quoted_title_after_parenthesis("가는 설명(ABC) ‘Title’ 나다", "⠠⠴⠀⠠⠦⠴⠠⠞")]
    #[case::multiword_parenthesis("가는 설명(Alpha Beta) XYZ 나다", "⠠⠴⠀⠴⠠⠠⠭")]
    #[case::multiword_quote("가는 ‘Alpha Beta’, ‘Title’ 나다", "⠄⠐⠀⠠⠦⠴⠠⠞")]
    #[case::mixed_quote("가는 ‘설명 ABC’, XYZ 나다", "⠴⠄⠐⠀⠴⠠⠠⠭")]
    fn unenclosed_roman_after_closed_enclosure_starts_a_new_section(
        #[case] input: &str,
        #[case] expected_boundary: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains(expected_boundary),
            "missing new Rule-29 Roman section: {actual}"
        );
    }

    /// Rule 32's official sequence keeps the successively enclosed single
    /// letters in one Roman section; `e` still takes its UEB grade-1 marker.
    #[test]
    fn successive_enclosed_single_letters_remain_one_roman_section() {
        let actual = crate::encode_to_unicode("모음에는 (a), (e), (i)가 있다.")
            .expect("official Rule-32 example must encode");

        assert_eq!(
            actual
                .chars()
                .filter(|cell| *cell == crate::unicode::encode_unicode(52))
                .count(),
            1
        );
        assert!(actual.contains("⠐⠣⠰⠑⠐⠜"));
    }

    /// Rules 29, 34, and 35 keep a pure Roman enclosure or a following number
    /// inside the active Roman section.
    #[rstest::rstest]
    #[case::pure_roman_expansion("기준 GRI (Global Reporting Initiative) Standards", "⠊⠲⠀")]
    #[case::roman_parenthetical("노래 Back for More (with Anitta)", "⠍⠲⠀")]
    #[case::number_continuation("대회 May Circuit '23에서", "⠞⠲⠀")]
    #[case::ascii_ellipsis("머뭇거리며 Umm ...이라고 말했다", "⠍⠍⠲⠀")]
    #[case::unicode_ellipsis("머뭇거리며 Umm …이라고 말했다", "⠍⠍⠲⠀")]
    #[case::midline_ellipsis("머뭇거리며 Umm ⋯이라고 말했다", "⠍⠍⠲⠀")]
    fn separated_roman_or_number_continuation_stays_in_the_section(
        #[case] input: &str,
        #[case] forbidden_boundary: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            !actual.contains(forbidden_boundary),
            "Roman section closed too early: {actual}"
        );
    }

    /// Korean rules 29, 32, and 35: a standalone print colon between Roman
    /// items is UEB punctuation inside one Roman section, even when spaces
    /// surround it.
    #[rstest::rstest]
    #[case::capitalized_words("가 Alpha : Beta 나")]
    #[case::uppercase_and_number("가 URL : 393 나")]
    #[case::mixed_case_and_number("가 Id : 7 나")]
    fn spaced_colon_between_roman_items_remains_inside_section(#[case] input: &str) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains("⠀⠒⠀"),
            "colon was not rendered as UEB punctuation: {actual}"
        );
        assert!(
            !actual.contains("⠲⠀⠐⠂⠀⠴"),
            "colon split one Roman section: {actual}"
        );
    }

    /// UEB 3.1.1 and Korean rule 29: a spaced ampersand connecting Roman words
    /// neither closes the section before itself nor starts a new one after it.
    #[rstest::rstest]
    #[case::official_name("가 Marks & Spencer 나")]
    #[case::technical_phrase("가 3D Scan & Solution 나")]
    #[case::attached_right_word("가 EV Mining &Development 나")]
    fn spaced_ampersand_bridges_one_roman_section(#[case] input: &str) {
        let ir = DocumentIR::parse(input, true);
        let ampersand_index = ir
            .tokens
            .iter()
            .position(
                |token| matches!(token, Token::Word(word) if word.chars.first() == Some(&'&')),
            )
            .expect("ampersand token");
        assert!(
            spaced_ampersand_connects_roman_words(&ir.tokens, ampersand_index),
            "test input must contain a structurally Roman ampersand"
        );

        let actual = crate::encode_to_unicode(input).expect("Roman phrase must encode");
        assert!(actual.contains("⠈⠯"), "ampersand missing: {actual}");
        assert!(
            !actual.contains("⠲⠀⠈⠯") && !actual.contains("⠈⠯⠀⠴"),
            "ampersand split the Roman section: {actual}"
        );
    }

    /// Rule 29: a Korean word containing one embedded Roman letter is not a
    /// standalone one-letter Roman continuation.
    #[rstest::rstest]
    #[case::parenthesized_letter("KODEX 골드선물(H)", "⠭⠲⠀")]
    #[case::korean_word_with_letter("ABB FIA 포뮬러E", "⠁⠲⠀")]
    #[case::following_model_name("SUV 모델X", "⠧⠲⠀")]
    fn korean_word_with_one_roman_letter_closes_the_previous_section(
        #[case] input: &str,
        #[case] expected_boundary: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains(expected_boundary),
            "missing Roman terminator before Korean word: {actual}"
        );
    }

    /// Rule 29: a separated one-letter Roman name remains part of the same
    /// Roman section when it starts the next print word. A directly attached
    /// Korean suffix or gloss does not change that Roman-first boundary.
    #[rstest::rstest]
    #[case::korean_particle("Global X가")]
    #[case::korean_classifier("WBC B조")]
    #[case::korean_gloss("DAY6 Young K(영케이)")]
    fn roman_initial_single_letter_with_korean_suffix_continues_section(#[case] input: &str) {
        let actual = crate::encode_to_unicode(input).expect("input must encode");
        assert!(
            actual.contains("⠀⠰"),
            "separated Roman initial did not continue the active section: {actual}"
        );
    }

    /// Korean rule 35 PDF example: numbers do not split a roman section, so
    /// the later capital word resumes without another roman indicator.
    #[test]
    fn capital_prefix_after_roman_number_chain_does_not_reenter_roman_mode() {
        let out = encode("KBS 1 TV 좀 켜 주세요.").unwrap();

        assert_eq!(out.iter().filter(|byte| **byte == 52).count(), 1);
    }

    /// UEB 5.6.1/6.5.1-6.5.2 through the rule-29/35 character route. `A` is only
    /// the Roman-chain routing scaffold for the PDF's exact `3b`, `3B`, and `3m`
    /// suffixes; those cases directly cover lowercase a-j grade 1, capitalization,
    /// and unmarked lowercase k-z. Comparison starts immediately after the route's
    /// single Roman indicator, so another occurrence cannot satisfy the assertion.
    #[rstest::rstest]
    #[case::braille4all("Braille4All", "⠠⠃⠗⠁⠊⠇⠇⠑⠼⠙⠠⠁⠇⠇")]
    #[case::m4g("M4G", "⠠⠍⠼⠙⠠⠛")]
    #[case::w1n("W1N", "⠠⠺⠼⠁⠠⠝")]
    #[case::lower_a_to_j("A3b", "⠠⠁⠼⠉⠰⠃")]
    #[case::uppercase("A3B", "⠠⠁⠼⠉⠠⠃")]
    #[case::lower_k_to_z("A3m", "⠠⠁⠼⠉⠍")]
    fn numeric_grade1_mode_continues_into_pdf_roman_examples(
        #[case] surface: &str,
        #[case] expected_ueb: &str,
    ) {
        let output = encode(&format!("가({surface})")).unwrap();
        let expected_ueb = expected_ueb
            .chars()
            .map(crate::unicode::decode_unicode)
            .collect::<Vec<_>>();
        let roman_start = output
            .iter()
            .position(|cell| *cell == crate::rules::korean::rule_29::ROMAN_INDICATOR)
            .expect("Korean wrapper must enter one Roman section")
            + 1;

        assert_eq!(
            output.get(roman_start..roman_start + expected_ueb.len()),
            Some(expected_ueb.as_slice())
        );
    }

    /// UEB 6.5.2 full-encoder controls for the three Roman letter classes after
    /// a digit: lowercase a-j needs grade 1, capitals use their capital indicator,
    /// and lowercase k-z needs no additional indicator.
    #[rstest::rstest]
    #[case::lower_a_to_j("3b", "⠼⠉⠰⠃")]
    #[case::uppercase("3B", "⠼⠉⠠⠃")]
    #[case::lower_k_to_z("3m", "⠼⠉⠍")]
    fn numeric_grade1_letter_class_pdf_controls(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    #[test]
    fn fraction_token_encodes() {
        let mut ir = DocumentIR {
            tokens: vec![
                Token::Fraction(super::super::token::FractionToken {
                    whole: None,
                    numerator: "1".to_string(),
                    denominator: "2".to_string(),
                }),
                Token::Space(SpaceKind::Regular),
                Token::Fraction(super::super::token::FractionToken {
                    whole: Some("3".to_string()),
                    numerator: "1".to_string(),
                    denominator: "4".to_string(),
                }),
            ],
            state: EncoderState::new(false),
        };
        let mut engine = make_char_engine();
        let out = emit(&mut ir, &mut engine).unwrap();

        let mut expected = fraction::encode_fraction("1", "2").unwrap();
        expected.push(0);
        expected.extend(fraction::encode_mixed_fraction("3", "1", "4").unwrap());
        assert_eq!(out, expected);
    }

    #[test]
    fn extract_context_uses_prev_and_remaining_words() {
        let words = ["A", "B", "C"];
        let tokens = words
            .iter()
            .map(|w| {
                let chars: Vec<char> = w.chars().collect();
                Token::Word(WordToken {
                    text: Cow::Borrowed(w),
                    chars: chars.clone(),
                    meta: super::super::token::WordMeta::from_chars(&chars),
                })
            })
            .collect::<Vec<_>>();

        let word_texts = collect_word_texts(&tokens);
        let context = word_context(&word_texts, 1);
        assert_eq!(context.prev_word, "A");
        assert_eq!(context.remaining_words, ["C"]);
    }

    /// emit:85 (extracted helper) — `token_is_math_word` returns false for None
    /// and for tokens that aren't Word/PreEncoded (Space, Mode, Fraction).
    #[test]
    fn token_is_math_word_returns_false_for_non_word_non_preencoded() {
        use super::token_is_math_word;
        use crate::rules::token::{ModeEvent, SpaceKind};
        assert!(!token_is_math_word(None));
        assert!(!token_is_math_word(Some(&Token::Space(SpaceKind::Regular))));
        assert!(!token_is_math_word(Some(&Token::Mode(
            ModeEvent::EnterEnglish
        ))));
        // Korean Word also returns false (meta.has_korean = true).
        let chars: Vec<char> = "한국".chars().collect();
        let kw = Token::Word(crate::rules::token::WordToken {
            text: std::borrow::Cow::Borrowed("한국"),
            chars: chars.clone(),
            meta: crate::rules::token::WordMeta::from_chars(&chars),
        });
        assert!(!token_is_math_word(Some(&kw)));
        // PreEncoded → true.
        assert!(token_is_math_word(Some(&Token::PreEncoded(vec![1, 2, 3]))));
    }

    /// emit.rs lines 155-156 - end-of-stream triple_big_english cleanup arm.
    /// 모든 production input은 word loop 내에서 triple_big_english를 close하므로
    /// 이 fallback은 도달하지 않는다. 직접 DocumentIR을 구성해 상태를 강제 주입한
    /// 뒤 emit을 호출해 분기를 cover한다.
    #[test]
    fn emit_end_of_stream_triple_big_english_safety_net() {
        use crate::rules::engine::RuleEngine;
        use crate::rules::token::DocumentIR;
        let mut ir = DocumentIR::parse("", false);
        ir.state.triple_big_english = true;
        let mut engine = RuleEngine::new();
        let result = emit(&mut ir, &mut engine).unwrap();
        assert_eq!(
            result,
            vec![32, 4],
            "expected safety-net close bytes, got {result:?}"
        );
    }
}
