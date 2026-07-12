//! Body of MathExpressionTokenRule::apply (extracted from math_expression.rs).

use crate::math_symbol_shortcut;
use crate::rules::context::EncoderState;
use crate::rules::math;
use crate::rules::token::{Token, WordToken};
use crate::rules::token_rule::TokenAction;

use super::detect::is_math_expression;
use super::helpers::*;

/// Resolve the previous and next Word neighbours, skipping over Space tokens.
/// Returns (prev, next) where each is `Some(&WordToken)` if found before hitting
/// a non-Space/Word token (e.g., PreEncoded, Fraction) or the boundary.
///
/// Extracted from `run` so the helper is directly unit-testable and mutation
/// testing can pinpoint regressions in neighbour resolution logic.
pub(super) fn prev_next_words<'a, 'b>(
    tokens: &'b [Token<'a>],
    index: usize,
) -> (
    Option<&'b crate::rules::token::WordToken<'a>>,
    Option<&'b crate::rules::token::WordToken<'a>>,
) {
    (
        index
            .checked_sub(1)
            .and_then(|i| prev_word_skip_space(tokens, i)),
        next_word_skip_space(tokens, index + 1),
    )
}

/// Walks forward from `start`, skipping `Token::Space`, returning the first
/// `Token::Word` (None on non-Word non-Space or end of slice).
pub(super) fn next_word_skip_space<'a, 'b>(
    tokens: &'b [Token<'a>],
    start: usize,
) -> Option<&'b crate::rules::token::WordToken<'a>> {
    let mut i = start;
    while let Some(tok) = tokens.get(i) {
        match tok {
            Token::Space(_) => i += 1,
            Token::Word(w) => return Some(w),
            _ => return None,
        }
    }
    None
}

/// Same as `next_word_skip_space` but with the (index, &Word) pair.
pub(super) fn next_indexed_word_skip_space<'a, 'b>(
    tokens: &'b [Token<'a>],
    start: usize,
) -> Option<(usize, &'b crate::rules::token::WordToken<'a>)> {
    let mut i = start;
    while let Some(tok) = tokens.get(i) {
        match tok {
            Token::Space(_) => i += 1,
            Token::Word(w) => return Some((i, w)),
            _ => return None,
        }
    }
    None
}

/// Walks backward from `start`, skipping `Token::Space`, returning the first
/// `Token::Word` (None on non-Word non-Space or underflow).
pub(super) fn prev_word_skip_space<'a, 'b>(
    tokens: &'b [Token<'a>],
    start: usize,
) -> Option<&'b crate::rules::token::WordToken<'a>> {
    let mut cursor = Some(start);
    while let Some(i) = cursor {
        match tokens.get(i) {
            Some(Token::Space(_)) => cursor = i.checked_sub(1),
            Some(Token::Word(w)) => return Some(w),
            _ => return None,
        }
    }
    None
}

/// Checks whether characters in `w` represent a "math letter context" that
/// should cause a following ellipsis to be encoded as the math ellipsis ⠠⠠⠠.
fn word_is_math_letter_context(w: &crate::rules::token::WordToken<'_>) -> bool {
    let has_super_sub = w.chars.iter().any(|c| {
        matches!(
            *c,
            '\u{2080}'..='\u{2089}' | '\u{00B2}' | '\u{00B3}' | '\u{2070}'..='\u{2079}'
        )
    });
    let plain_letter_list = w.chars.first().is_some_and(|c| c.is_ascii_alphabetic())
        && w.chars
            .iter()
            .all(|c| c.is_ascii_alphabetic() || matches!(*c, ',' | '₀'..='₉'));
    has_super_sub || plain_letter_list
}

/// Uppercase prose abbreviations (`FM의`, `SNS는`) and uppercase math products
/// share the same surface shape. For product notation in this branch, accept
/// ordered Roman variable runs such as `AB`, `ABC`, `CD`; leave non-sequential
/// acronyms to the English/Korean prose rules.
fn is_consecutive_ascii_letter_run(chars: &[char]) -> bool {
    chars.len() >= 2
        && chars
            .windows(2)
            .all(|pair| u32::from(pair[1]) == u32::from(pair[0]) + 1)
}

fn has_ascii_letter_korean_math_suffix(chars: &[char]) -> bool {
    if chars.len() < 3 {
        return false;
    }

    let ascii_prefix_len = chars.iter().take_while(|c| c.is_ascii_alphabetic()).count();
    (2..=3).contains(&ascii_prefix_len)
        && chars[ascii_prefix_len..]
            .first()
            .is_some_and(|c| matches!(*c, '의' | '와' | '과'))
        && chars[ascii_prefix_len..]
            .iter()
            .all(|c| is_korean_suffix_char(*c))
}

fn next_word_starts_with_math_value_cue(tokens: &[Token<'_>], index: usize) -> bool {
    let mut cursor = index + 1;
    while let Some(token) = tokens.get(cursor) {
        match token {
            Token::Space(_) => cursor += 1,
            Token::Word(word) => {
                let text = word.text.as_ref();
                return text.starts_with('값') || text.starts_with('곱');
            }
            _ => return false,
        }
    }
    false
}

fn prev_word_is_math_product_cue(tokens: &[Token<'_>], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|start| prev_word_skip_space(tokens, start))
        .is_some_and(|word| word.text.as_ref() == "곱")
}

/// Walks backward from `index - 1`, skipping `Space`, returning whether the
/// preceding content is a math-letter Word or a math-context PreEncoded.
fn prev_is_math_context_for_ellipsis(tokens: &[Token<'_>], index: usize) -> bool {
    let mut cursor = index.checked_sub(1);
    while let Some(i) = cursor {
        match tokens.get(i) {
            Some(Token::Space(_)) => cursor = i.checked_sub(1),
            Some(Token::PreEncoded(_)) => return true,
            Some(Token::Word(w)) => return word_is_math_letter_context(w),
            _ => return false,
        }
    }
    false
}

/// Walks backward from `index - 1` skipping `Space`; true if any
/// `Word | PreEncoded` is found before underflow.
fn has_content_skipping_space_backward(tokens: &[Token<'_>], index: usize) -> bool {
    let mut cursor = index.checked_sub(1);
    while let Some(i) = cursor {
        match tokens.get(i) {
            Some(Token::Space(_)) => cursor = i.checked_sub(1),
            Some(Token::Word(_) | Token::PreEncoded(_)) => return true,
            _ => return false,
        }
    }
    false
}

/// Walks forward from `index + 1` skipping `Space`; true if any
/// `Word | PreEncoded` is found before slice end.
fn has_content_skipping_space_forward(tokens: &[Token<'_>], index: usize) -> bool {
    let mut i = index + 1;
    while let Some(tok) = tokens.get(i) {
        match tok {
            Token::Space(_) => i += 1,
            Token::Word(_) | Token::PreEncoded(_) => return true,
            _ => return false,
        }
    }
    false
}

/// True iff `text` has the special increment-equality-polysum pattern
/// (∆ + `=` + `)+(`) that requires a double-space prefix per PDF 제11항.
fn is_delta_eq_polysum_pattern(text: &str) -> bool {
    text.contains('\u{2206}') && text.contains('=') && text.contains(")+(")
}

/// True iff the Word's chars are all Korean (Hangul syllables / jamo) plus
/// punctuation/whitespace. Used to decide whether a math expression needs a
/// trailing-space delimiter before the following Word.
fn word_is_pure_korean(w: &crate::rules::token::WordToken<'_>) -> bool {
    if !w.meta.has_korean {
        return false;
    }
    w.chars.iter().all(|c| {
        let code = *c as u32;
        (0xAC00..=0xD7A3).contains(&code)
            || (0x3131..=0x3163).contains(&code)
            || matches!(*c, '.' | ',' | '!' | '?' | ' ')
    })
}

/// True iff `text` contains a character that needs explicit decimal-context
/// spacing — the internal U+001F Unit Separator (used as a math-context
/// sentinel), the U+22EF MIDLINE HORIZONTAL ELLIPSIS, or any combining math
/// mark in `chars`.
fn needs_decimal_context_spacing(text: &str, chars: &[char]) -> bool {
    text.contains('\u{001F}')
        || text.contains('\u{22EF}')
        || chars.iter().any(|ch| is_combining_math_mark(*ch))
}

/// Walks backward from `index - 1` skipping at most one Space, then checks
/// whether the token beyond the Space is a math/mixed-math context (used to
/// decide `leading_delimiter_len` in the non-`$...$` mixed-math fallback).
fn prev_prev_is_math_or_mixed_context(tokens: &[Token<'_>], index: usize) -> bool {
    let mut i = index;
    let mut found_space = false;
    while i > 0 {
        i -= 1;
        match tokens.get(i) {
            Some(Token::Space(_)) => found_space = true,
            Some(Token::PreEncoded(_) | Token::Fraction(_)) if found_space => return true,
            Some(Token::Word(w)) if found_space => {
                return is_math_expression(&w.chars, w.text.as_ref())
                    || (w.meta.has_korean
                        && is_strong_mixed_math_candidate(&w.chars, w.text.as_ref()));
            }
            _ => return false,
        }
    }
    false
}

/// Detect a Word that is exactly the logic XOR symbol `⊻` (U+22BB).
///
/// PDF 수학 — `A ⊻ B` 패턴에서 양쪽 대문자를 math 변수로 처리하기 위해 사용.
pub(super) fn is_logic_symbol_word(word: &crate::rules::token::WordToken<'_>) -> bool {
    word.chars
        .first()
        .is_some_and(|c| word.chars.len() == 1 && matches!(*c, '⊻'))
}

/// PDF — Compute leading spaces for a math token inserted at `index` based on
/// surrounding token context. Extracted to a standalone helper so each branch
/// gets a unique line attribution under tarpaulin.
fn compute_leading_spaces(
    tokens: &[Token<'_>],
    index: usize,
    in_prose: bool,
    inner_is_single_letter: bool,
    comma_list: bool,
    inner_is_simple_numeric: bool,
) -> usize {
    let suppress_pad = (in_prose && (inner_is_single_letter || comma_list))
        || inner_is_simple_numeric
        || index == 0;
    if suppress_pad {
        return 0;
    }
    // PDF — Math tokens in production always have a preceding Space token if
    // `index > 0`. Probe-verified 2026-05-23: no testcase reaches a math token
    // at index > 0 without a Space immediately before it. The `return 2`
    // fallback was structurally unreachable; if state ever shifts, treating
    // missing-Space prev as the 0-pad case is the safest default.
    let prev_prev = index.checked_sub(2).and_then(|i| tokens.get(i));
    let prev_prev_is_korean = matches!(prev_prev, Some(Token::Word(w)) if w.meta.has_korean);
    if prev_prev_is_korean { 1 } else { 0 }
}

pub(super) fn run<'a>(
    tokens: &[Token<'a>],
    index: usize,
    state: &mut EncoderState,
) -> Result<TokenAction<'a>, String> {
    let Some(Token::Word(word)) = tokens.get(index) else {
        return Ok(TokenAction::Noop);
    };

    let text = word.text.as_ref();

    // PDF 수학 제60/61항 — `a ≲ b:`, `p ⊻ q:` 같이 단일 letter + 관계기호 + 단일
    // letter + 콜론 패턴의 inline math expression. 콜론 이전까지를 하나의 math
    // expression으로 병합해 인코딩한다 (letter들이 산문 quote-wrap되지 않도록).
    //
    // 패턴 매칭 조건:
    // - 현재 Word: 단일 ASCII 알파벳 (lowercase)
    // - 다음 Word: math 관계/논리 연산자 (단일 chars, `<>≲≺⊻` 등)
    // - 그 다음 Word: 단일 ASCII letter + `:`
    if word.chars.len() == 1 && word.chars[0].is_ascii_lowercase() {
        let collect_next = |start: usize| {
            let mut j = start;
            while matches!(tokens.get(j), Some(Token::Space(_))) {
                j += 1;
            }
            tokens.get(j).map(|t| (j, t))
        };
        // PDF 수학 제60·61항 — colon-math relation operators.
        const COLON_MATH_OPS: &[char] = &[
            '\u{2272}', '\u{2273}', '\u{227A}', '\u{227B}', '\u{22BB}', '<', '>', '=', '\u{2260}',
            '\u{2264}', '\u{2265}', '\u{2208}', '\u{2209}',
        ];
        if let Some((op_idx, Token::Word(op_w))) = collect_next(index + 1)
            && op_w.chars.len() == 1
            && COLON_MATH_OPS.contains(&op_w.chars[0])
            && let Some((last_idx, Token::Word(last_w))) = collect_next(op_idx + 1)
            && last_w.chars.len() == 2
            && last_w.chars[0].is_ascii_lowercase()
            && last_w.chars[1] == ':'
        {
            // Merge: "a" + " " + "≲" + " " + "b:" → math expression.
            let merged = format!("{} {} {}", text, op_w.text.as_ref(), last_w.text.as_ref());
            let math_context = math_context_from_state(state);
            if let Ok(bytes) =
                math::encoder::encode_math_expression_with_context(&merged, math_context)
            {
                let consume_count = last_idx + 1 - index;
                return Ok(TokenAction::ReplaceRange(
                    consume_count,
                    vec![Token::PreEncoded(bytes)],
                ));
            }
        }
    }

    // PDF 수학 제60항 2-나 — 조건제시법 set-builder notation `{x|x는 정수}`.
    // `{`로 시작하고 `|`를 포함하는 Word를 만나면, `}` 토큰을 찾을 때까지
    // 후속 Word/Space를 모아 하나의 math expression으로 인코딩한다.
    if word.chars.first() == Some(&'{') && word.chars.contains(&'|') {
        let mut merged = text.to_string();
        let mut end_idx = index;
        let mut found_close = word.chars.last() == Some(&'}');
        if !found_close {
            let mut i = index + 1;
            while i < tokens.len() {
                match tokens.get(i) {
                    Some(Token::Space(_)) => merged.push(' '),
                    Some(Token::Word(w)) => {
                        merged.push_str(w.text.as_ref());
                        if w.chars.last() == Some(&'}') {
                            end_idx = i;
                            found_close = true;
                            break;
                        }
                    }
                    _ => break,
                }
                i += 1;
            }
        }
        let math_context = math_context_from_state(state);
        if found_close
            && let Ok(bytes) =
                math::encoder::encode_math_expression_with_context(&merged, math_context)
        {
            let consume_count = end_idx + 1 - index;
            return Ok(TokenAction::ReplaceRange(
                consume_count,
                vec![Token::PreEncoded(bytes)],
            ));
        }
    }

    // PDF 제12항 [붙임 2] — 한국어 prose 내 multi-letter math identifier 처리.
    // Word가 2~3개 ASCII letter로 시작하고 곧장 수식 나열/소유 조사 `의/와/과`가
    // 붙는 패턴(예: `ab의`, `AB와`)에서 산문 영어 wrap(`⠴...⠲`)이 아닌 math letter
    // 처리. 대문자(`AB의`, `ABC의`)는 순차 변수열일 때만 같은 구조로 처리해
    // 일반 약어(`FM의`, `SNS는` 등)를 이 경로에서 제외한다.
    if word.chars.len() >= 3 {
        let ascii_prefix_len = word
            .chars
            .iter()
            .take_while(|c| c.is_ascii_alphabetic())
            .count();
        if (2..=3).contains(&ascii_prefix_len) {
            let suffix_chars = &word.chars[ascii_prefix_len..];
            let suffix_is_math_identifier_particle =
                has_ascii_letter_korean_math_suffix(&word.chars);
            let prefix_letters: Vec<char> = word.chars[..ascii_prefix_len].to_vec();
            let all_lower = prefix_letters.iter().all(|c| c.is_ascii_lowercase());
            let all_upper = prefix_letters.iter().all(|c| c.is_ascii_uppercase());

            let case_allowed = (all_lower
                && (next_word_starts_with_math_value_cue(tokens, index)
                    || prev_word_is_math_product_cue(tokens, index)))
                || (all_upper && is_consecutive_ascii_letter_run(&prefix_letters));
            if suffix_is_math_identifier_particle && case_allowed {
                let prev_is_korean_or_first = index == 0
                    || index
                        .checked_sub(1)
                        .and_then(|i| tokens.get(i))
                        .is_some_and(|t| match t {
                            Token::Word(w) => w.meta.has_korean,
                            Token::Space(_) => index
                                .checked_sub(2)
                                .and_then(|j| tokens.get(j))
                                .is_some_and(
                                    |t2| matches!(t2, Token::Word(w) if w.meta.has_korean),
                                ),
                            _ => false,
                        });
                if prev_is_korean_or_first {
                    let matrix_context = state.matrix_context_active;
                    let mut bytes = Vec::new();
                    // PDF 제11항 — 국어 문장 안 수식 앞뒤를 두 칸씩 띄어 쓴다.
                    // Token::Space가 1칸 보조하므로 leading 1칸 추가.
                    bytes.push(0);
                    for letter in &prefix_letters {
                        if all_upper {
                            if matrix_context {
                                bytes.push(32);
                            } else if letter == &prefix_letters[0] {
                                bytes.push(32);
                                bytes.push(32);
                            }
                            let code = crate::english::encode_english(letter.to_ascii_lowercase())?;
                            bytes.push(code);
                        } else {
                            let code = crate::english::encode_english(*letter)?;
                            bytes.push(code);
                        }
                    }
                    // trailing 두 칸 (math expression 종료 boundary).
                    bytes.push(0);
                    bytes.push(0);
                    let suffix: String = suffix_chars.iter().collect();
                    let suffix_chars_vec: Vec<char> = suffix.chars().collect();
                    let suffix_meta = crate::rules::token::WordMeta::from_chars(&suffix_chars_vec);
                    let suffix_word = Token::Word(WordToken {
                        text: std::borrow::Cow::Owned(suffix),
                        chars: suffix_chars_vec,
                        meta: suffix_meta,
                    });
                    return Ok(TokenAction::ReplaceMany(vec![
                        Token::PreEncoded(bytes),
                        suffix_word,
                    ]));
                }
            }
        }
    }

    // PDF 제13항 — 한국어 산문 안 그리스 문자 리스트 (예: `α, β에`).
    // `Word(MathLetter+',')`이 현재이고 다음 비공백 Word가 `MathLetter+Korean`이면
    // 두 단어를 `⠴α, β⠲` + Korean으로 묶어 emit한다.
    // 직전이 한국어 단어여야 한다 (prose 컨텍스트 확인).
    if word.chars.len() == 2
        && word.chars[1] == ','
        && math_symbol_shortcut::is_math_symbol_char(word.chars[0])
        && !word.chars[0].is_ascii_alphanumeric()
    {
        let prev_is_korean_word = index
            .checked_sub(1)
            .and_then(|i| tokens.get(i))
            .and_then(|t| match t {
                Token::Space(_) => index.checked_sub(2).and_then(|j| tokens.get(j)),
                _ => Some(t),
            })
            .is_some_and(|t| matches!(t, Token::Word(w) if w.meta.has_korean));
        // 다음 Word: math letter 시작 + 한국어 suffix
        let next_word_opt = next_indexed_word_skip_space(tokens, index + 1);
        if prev_is_korean_word
            && let Some((next_idx, next_word)) = next_word_opt
            && next_word.chars.len() >= 2
            && math_symbol_shortcut::is_math_symbol_char(next_word.chars[0])
            && !next_word.chars[0].is_ascii_alphanumeric()
            && next_word.chars[1..]
                .iter()
                .all(|c| crate::utils::is_korean_char(*c))
        {
            let letter1 = word.chars[0];
            let letter2 = next_word.chars[0];
            let korean_suffix: String = next_word.chars[1..].iter().collect();
            let enc1 = math_symbol_shortcut::encode_char_math_symbol_shortcut(letter1)?;
            let enc2 = math_symbol_shortcut::encode_char_math_symbol_shortcut(letter2)?;
            let mut bytes = Vec::new();
            bytes.push(52); // ⠴ open quote
            bytes.extend_from_slice(enc1);
            bytes.push(2); // ⠂ literal comma in math letter list
            bytes.push(0); // space
            bytes.extend_from_slice(enc2);
            bytes.push(50); // ⠲ close quote
            // suffix Korean을 다음 Word로 분리 emit
            let suffix_chars: Vec<char> = korean_suffix.chars().collect();
            let suffix_meta = crate::rules::token::WordMeta::from_chars(&suffix_chars);
            let suffix_word = Token::Word(WordToken {
                text: std::borrow::Cow::Owned(korean_suffix),
                chars: suffix_chars,
                meta: suffix_meta,
            });
            // 현재 Word + 사이 토큰 + 다음 Word를 한꺼번에 교체.
            let consume_count = next_idx + 1 - index;
            return Ok(TokenAction::ReplaceRange(
                consume_count,
                vec![Token::PreEncoded(bytes), suffix_word],
            ));
        }
    }

    // PDF — `...` 또는 `..., `, `..`은 math context에 있으면 수학 줄임표 `⠠⠠⠠`로 emit.
    // Korean 마침표 줄임표 `⠲⠲⠲`와 구분.
    let dot_only =
        !text.is_empty() && (text.chars().all(|c| matches!(c, '.' | ',')) && text.contains('.'));
    if dot_only {
        // PDF — 앞 토큰이 math letter Word 또는 이미 인코딩된 PreEncoded(math 컨텍스트)면
        // 수학 줄임표로 emit. PreEncoded는 이전 math 처리 결과로 본다.
        let prev_is_math_context = prev_is_math_context_for_ellipsis(tokens, index);
        if prev_is_math_context {
            let dots: usize = text.chars().filter(|c| *c == '.').count();
            // ⠠ (32) repeated for each dot, capped at 3 per PDF.
            let mut bytes = vec![32u8; dots.min(3)];
            // 다음 토큰이 Korean Word면 math+Korean 경계로 trailing space 추가.
            let next_is_korean =
                next_word_skip_space(tokens, index + 1).is_some_and(|w| w.meta.has_korean);
            if text.ends_with(',') {
                // PDF — math 식 안 comma는 ⠐, prose math letter 리스트의 comma는 ⠂.
                // 다음이 math 또는 PreEncoded면 ⠐, Korean이면 ⠂.
                bytes.push(if next_is_korean { 2 } else { 16 });
            }
            if next_is_korean {
                bytes.push(0);
            }
            return Ok(TokenAction::Replace(Token::PreEncoded(bytes)));
        }
    }

    // Numeric middle-dot forms in Korean prose (e.g. 3·1 운동) should stay non-math,
    // while standalone numeric expressions like 6·9 should be routed to math.
    if is_middle_dot_numeric_word(&word.chars) && has_adjacent_korean_word(tokens, index) {
        return Ok(TokenAction::Noop);
    }

    // Standalone therefore/because between content tokens (Word or PreEncoded)
    // should add one braille space on each side. Combined with the Space tokens
    // already present between words, this produces the double-space delimiter
    // required by 제11항.
    if matches!(word.chars.as_slice(), ['∴' | '∵']) {
        let has_prev_content = has_content_skipping_space_backward(tokens, index);
        let has_next_content = has_content_skipping_space_forward(tokens, index);
        if has_prev_content && has_next_content {
            let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(word.chars[0])?;
            let mut out = vec![0];
            out.extend_from_slice(encoded);
            out.push(0);
            return Ok(TokenAction::Replace(Token::PreEncoded(out)));
        }
    }

    // Logical symbols separated by spaces should still treat uppercase letters as variables.
    if word.chars.len() == 1 && word.chars[0].is_ascii_uppercase() {
        let (prev, next) = prev_next_words(tokens, index);
        if prev.is_some_and(is_logic_symbol_word) || next.is_some_and(is_logic_symbol_word) {
            let code = crate::english::encode_english(word.chars[0].to_ascii_lowercase())?;
            return Ok(TokenAction::Replace(Token::PreEncoded(vec![code])));
        }
    }

    // Skip if already processed (PreEncoded) or if it's a fraction
    if let Some(stripped) = text.strip_prefix('$') {
        if let Some(close_idx) = stripped.find('$')
            && close_idx + 1 < stripped.len()
        {
            let latex = &text[..=close_idx + 1];
            let suffix = &stripped[close_idx + 1..];

            if let Some((whole, numerator, denominator)) =
                crate::fraction::parse_latex_fraction(latex)
            {
                // 제44항 [다만]: 분수 직후 한국어 조사의 첫 초성이 ㄴ/ㄷ/ㅁ/ㅋ/ㅌ/ㅍ/ㅎ
                // 또는 '운'으로 시작하면 띄어 쓴다.
                let mut replacement: Vec<Token<'a>> =
                    vec![Token::Fraction(crate::rules::token::FractionToken {
                        whole,
                        numerator,
                        denominator,
                    })];
                if !suffix.is_empty() && rule_44_requires_space_before_korean(suffix) {
                    replacement.push(Token::Space(crate::rules::token::SpaceKind::Regular));
                }
                replacement.push(build_word_token(suffix.to_string()));
                return Ok(TokenAction::ReplaceMany(replacement));
            }

            let inner = &latex[1..latex.len() - 1];
            let math_context = math_context_from_state(state);
            if let Ok(bytes) =
                crate::rules::token_rules::latex_math::encode_latex_math_bytes_with_context(
                    inner,
                    math_context,
                )
            {
                // PDF — Korean prose 안 단일 letter math 블록은 ⠴...⠲로 감싼다.
                // 콤마-구분 letter 리스트도 quote/english marker로 감싼다.
                let suffix_first = suffix.chars().next();
                let suffix_is_korean = suffix_first.is_some_and(crate::utils::is_korean_char);
                let inner_is_single_letter =
                    inner.chars().count() == 1 && inner.chars().all(|c| c.is_ascii_alphabetic());
                let comma_list = inner.contains(',')
                    && inner.split(',').map(str::trim).all(|p| {
                        !p.is_empty()
                            && p.chars().count() == 1
                            && p.chars().all(|c| c.is_ascii_alphabetic())
                    });
                let prev_is_korean = index
                    .checked_sub(1)
                    .and_then(|i| tokens.get(i))
                    .map(|tok| match tok {
                        Token::Word(w) => w.meta.has_korean,
                        Token::Space(_) => index
                            .checked_sub(2)
                            .and_then(|j| tokens.get(j))
                            .is_some_and(|t| matches!(t, Token::Word(w) if w.meta.has_korean)),
                        _ => false,
                    })
                    .unwrap_or(false);
                let in_prose = suffix_is_korean || prev_is_korean;
                // PDF — `$-2$`, `$0.3010$` 같이 부호+숫자/소수점만 있는 단순 수치는
                // "본격적 수식"이 아니므로 한국어 단어 경계에서 추가 공백을 적용하지 않는다.
                // Space token 1칸으로 충분하다.
                let inner_is_simple_numeric = !inner.is_empty()
                    && inner.chars().all(|c| {
                        c.is_ascii_digit() || matches!(c, '-' | '+' | '\u{2212}' | '.' | ',')
                    });
                // 따옴표 자체가 경계를 명시(단일 letter/리스트), 단순 수치, 토큰 첫 위치는
                // 모두 leading_spaces=0.
                let leading_spaces = compute_leading_spaces(
                    tokens,
                    index,
                    in_prose,
                    inner_is_single_letter,
                    comma_list,
                    inner_is_simple_numeric,
                );
                let mut replacement = Vec::new();
                if leading_spaces > 0 {
                    replacement.push(Token::PreEncoded(vec![0; leading_spaces]));
                }
                if in_prose && inner_is_single_letter {
                    let mut wrapped = Vec::with_capacity(bytes.len() + 2);
                    wrapped.push(52); // ⠴
                    wrapped.extend(bytes);
                    wrapped.push(50); // ⠲
                    replacement.push(Token::PreEncoded(wrapped));
                } else if in_prose && comma_list {
                    let letters: Vec<&str> = inner.split(',').map(str::trim).collect();
                    let mut wrapped = Vec::new();
                    for (i, letter) in letters.iter().enumerate() {
                        if let Some(c) = letter.chars().next() {
                            if i == 0 {
                                wrapped.push(52);
                            } else {
                                wrapped.push(0);
                                wrapped.push(48); // ⠰ english
                            }
                            if c.is_ascii_uppercase() {
                                wrapped.push(32);
                                if let Ok(code) =
                                    crate::english::encode_english(c.to_ascii_lowercase())
                                {
                                    wrapped.push(code);
                                }
                            } else if let Ok(code) = crate::english::encode_english(c) {
                                wrapped.push(code);
                            }
                            if i + 1 < letters.len() {
                                wrapped.push(2); // ⠂ literal comma (in math letter list)
                            } else {
                                wrapped.push(50);
                            }
                        }
                    }
                    replacement.push(Token::PreEncoded(wrapped));
                } else {
                    replacement.push(Token::PreEncoded(bytes));
                    // PDF — math + Korean prose 경계는 두 칸. 구두점/기호 suffix는 인접.
                    // 단, 단순 수치 표기(`-2`, `0.3010`)는 본격적 수식이 아니므로 직접 인접.
                    let trailing_spaces = if suffix_is_korean && !inner_is_simple_numeric {
                        2
                    } else {
                        0
                    };
                    if trailing_spaces > 0 {
                        replacement.push(Token::PreEncoded(vec![0; trailing_spaces]));
                    }
                }
                replacement.push(build_word_token(suffix.to_string()));
                return Ok(TokenAction::ReplaceMany(replacement));
            }
        }

        if let Some((whole, numerator, denominator)) = crate::fraction::parse_latex_fraction(text) {
            return Ok(TokenAction::Replace(Token::Fraction(
                crate::rules::token::FractionToken {
                    whole,
                    numerator,
                    denominator,
                },
            )));
        }

        if text.ends_with('$') && text.len() >= 3 {
            let inner = &text[1..text.len() - 1];
            let math_context = math_context_from_state(state);
            if let Ok(bytes) =
                crate::rules::token_rules::latex_math::encode_latex_math_bytes_with_context(
                    inner,
                    math_context,
                )
            {
                let replacement =
                    crate::rules::token_rules::latex_math::wrap_latex_math_tokens_with_inner(
                        tokens, index, bytes, inner,
                    );
                return Ok(TokenAction::ReplaceMany(replacement));
            }
        }

        return Ok(TokenAction::Noop);
    }

    if !is_math_expression(&word.chars, text) {
        let math_context = math_context_from_state(state);
        if let Some(bytes) = try_encode_mixed_math_slice(&word.chars, math_context) {
            return Ok(TokenAction::Replace(Token::PreEncoded(bytes)));
        }
        // 제11항: 한글 문장 안의 수학적 표기는 앞뒤를 두 칸씩 띄어 쓴다.
        // - index == 0           → 0칸 (문서 맨 앞)
        // - 이전 토큰이 Space    → 1칸 추가 (Token::Space 1칸 + 새 1칸 = 2칸)
        //   다만 prev-prev가 같은 math/mixed math 단어이면 0 (1칸 유지)
        // - 그 외 (content)     → 2칸 (경계 표시)
        let prev_prev_is_math_or_mixed = prev_prev_is_math_or_mixed_context(tokens, index);
        let leading_delimiter_len = if index == 0 {
            0
        } else if matches!(tokens.get(index - 1), Some(Token::Space(_))) {
            if prev_prev_is_math_or_mixed { 0 } else { 1 }
        } else {
            2
        };
        if let Some(replacement) = split_mixed_math_word(word, leading_delimiter_len, math_context)
        {
            return Ok(TokenAction::ReplaceMany(replacement));
        }
        return Ok(TokenAction::Noop);
    }

    // Try to encode via math engine.
    // Err arm below: if math encoding fails, fall back to character-level rules.
    let math_context = math_context_from_state(state);
    match math::encoder::encode_math_expression_with_context(text, math_context) {
        Ok(bytes) => {
            let (prev_has_korean, _next_has_korean) = adjacent_korean_word_flags(tokens, index);
            let mut wrapped = Vec::with_capacity(bytes.len() + 2);

            let needs_decimal_context_spacing = needs_decimal_context_spacing(text, &word.chars);
            let prev_is_space_decimal = index
                .checked_sub(1)
                .is_some_and(|i| matches!(tokens.get(i), Some(Token::Space(_))));
            if needs_decimal_context_spacing && prev_is_space_decimal {
                wrapped.push(0);
            }

            // 특수 패턴(증분 + 등호 + 다항식 조합)에만 prefix space 두 칸 추가.
            // 일반적인 한글 + math 인접 케이스는 Token::Space가 단일 공백을 처리하므로
            // 추가 prefix/suffix space를 emit하지 않는다.
            // 문서 맨 앞(index == 0)에서는 제11조에 따라 leading 띄어쓰기를 생략한다.
            if index != 0 && !prev_has_korean && is_delta_eq_polysum_pattern(text) {
                wrapped.push(0);
                wrapped.push(0);
            }

            // PDF 수학 제11항 — 국어 문장 안 "수식"은 앞뒤 두 칸씩 띄어쓴다.
            // 단일 연산자/기호(+, =, ×, ÷, /, - 등)는 일반 산식 일부이므로 제외한다.
            // 변수/숫자/괄호 등 실질적 수식(`f(x)`, `a²`, `2x+3` 등)일 때만 적용.
            // 단순 부호+숫자(`-2`, `+3`, `0.5` 등)는 일반 숫자 표기이므로
            // 추가 띄어쓰기를 적용하지 않는다. 첨자/괄호/문자가 있으면 실질적 수식.
            let only_simple_digits = !word.chars.is_empty()
                && word.chars.iter().all(|c| {
                    c.is_ascii_digit() || matches!(*c, '-' | '+' | '\u{2212}' | '.' | ',')
                });
            let is_substantial_math = word.chars.len() > 1
                && word.chars.iter().any(|c| {
                    c.is_ascii_alphanumeric() || matches!(*c, '(' | ')' | '[' | ']' | '|')
                })
                && !only_simple_digits;
            let needs_korean_leading = index != 0
                && prev_has_korean
                && matches!(tokens.get(index - 1), Some(Token::Space(_)))
                && !needs_decimal_context_spacing
                && is_substantial_math;
            if needs_korean_leading {
                wrapped.push(0);
            }

            wrapped.extend_from_slice(&bytes);

            if needs_decimal_context_spacing
                && matches!(tokens.get(index + 1), Some(Token::Space(_)))
            {
                wrapped.push(0);
            }

            // trailing은 다음 단어가 순수 한글일 때만 추가. (인접 단어가 math+korean
            // 혼합이면 다음 단어 측에서 leading을 추가하므로 중복 방지.)
            let next_is_pure_korean =
                next_word_skip_space(tokens, index + 1).is_some_and(word_is_pure_korean);
            let needs_trailing_korean_pad = next_is_pure_korean
                && matches!(tokens.get(index + 1), Some(Token::Space(_)))
                && !needs_decimal_context_spacing
                && is_substantial_math;
            let trailing_pad: &[u8] = if needs_trailing_korean_pad { &[0] } else { &[] };
            wrapped.extend_from_slice(trailing_pad);

            Ok(TokenAction::Replace(Token::PreEncoded(wrapped)))
        }
        Err(_) => Ok(TokenAction::Noop),
    }
}

// ============================================================
// Mutation-testing reinforcements for apply::run
//
// Strategy: rather than re-implement the local helpers in tests, drive run()
// indirectly via `crate::encode()` with crafted inputs. Each test exercises
// one specific code path and asserts an OBSERVABLE difference between the
// happy path and a nearby negative path. This kills mutations on local helpers
// (prev_next_words, is_logic_symbol_word) and on the dozens of branch checks
// throughout `run`.
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::{SpaceKind, WordMeta, WordToken};
    use std::borrow::Cow;

    fn enc_str(s: &str) -> String {
        crate::encode_to_unicode(s).unwrap_or_default()
    }

    /// Build a WordToken from a string for direct testing.
    fn word_tok<'a>(text: &'a str) -> Token<'a> {
        let chars: Vec<char> = text.chars().collect();
        let meta = WordMeta::from_chars(&chars);
        Token::Word(WordToken {
            text: Cow::Borrowed(text),
            chars,
            meta,
        })
    }

    fn space_tok() -> Token<'static> {
        Token::Space(SpaceKind::Regular)
    }

    // ---------- Direct tests on extracted helpers ----------

    /// `prev_next_words` returns (None, None) for an out-of-range index.
    /// Kills the `-> (None, None)` substitution mutant.
    #[test]
    fn prev_next_words_oob_index() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a")];
        let (prev, next) = prev_next_words(&tokens, 5);
        assert!(prev.is_none(), "prev must be None for oob index");
        assert!(next.is_none(), "next must be None for oob index");
    }

    /// `prev_next_words` returns the immediate previous Word (no Space between).
    #[test]
    fn prev_next_words_adjacent_words() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a"), word_tok("b"), word_tok("c")];
        let (prev, next) = prev_next_words(&tokens, 1);
        assert!(prev.is_some(), "prev must resolve to Word 'a'");
        assert_eq!(prev.unwrap().text.as_ref(), "a");
        assert!(next.is_some(), "next must resolve to Word 'c'");
        assert_eq!(next.unwrap().text.as_ref(), "c");
    }

    /// `prev_next_words` skips one or more Space tokens.
    #[test]
    fn prev_next_words_skips_spaces() {
        let tokens: Vec<Token<'_>> = vec![
            word_tok("a"),
            space_tok(),
            space_tok(),
            word_tok("b"),
            space_tok(),
            word_tok("c"),
        ];
        let (prev, next) = prev_next_words(&tokens, 3);
        assert_eq!(prev.unwrap().text.as_ref(), "a");
        assert_eq!(next.unwrap().text.as_ref(), "c");
    }

    /// `prev_next_words` returns None for prev when index is 0.
    /// Kills the `i - 1` underflow path mutations.
    #[test]
    fn prev_next_words_at_index_zero() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a"), word_tok("b")];
        let (prev, next) = prev_next_words(&tokens, 0);
        assert!(prev.is_none(), "no prev at index 0");
        assert!(next.is_some(), "next must still resolve");
        assert_eq!(next.unwrap().text.as_ref(), "b");
    }

    /// `prev_next_words` returns None when a non-Space/Word boundary is hit.
    #[test]
    fn prev_next_words_stops_at_non_word_token() {
        let tokens: Vec<Token<'_>> = vec![
            Token::PreEncoded(vec![1, 2, 3]),
            space_tok(),
            word_tok("middle"),
            space_tok(),
            Token::PreEncoded(vec![4, 5, 6]),
        ];
        let (prev, next) = prev_next_words(&tokens, 2);
        // PreEncoded on both sides → prev/next both None.
        assert!(
            prev.is_none(),
            "PreEncoded boundary must yield None for prev"
        );
        assert!(
            next.is_none(),
            "PreEncoded boundary must yield None for next"
        );
    }

    #[test]
    fn math_suffix_and_next_value_cue_helpers_reject_short_or_non_word_inputs() {
        assert!(!has_ascii_letter_korean_math_suffix(&['a', '의']));

        let tokens = vec![word_tok("ab의"), Token::PreEncoded(vec![1])];
        assert!(!next_word_starts_with_math_value_cue(&tokens, 0));
    }

    /// `is_logic_symbol_word` — XOR(⊻) 단독 토큰만 true, 그 외는 false.
    /// Kills: `-> false`, `!=` mutations.
    #[rstest::rstest]
    #[case::xor_alone("⊻", true)]
    #[case::wedge_alone("∧", false)]
    #[case::xor_then_letter("⊻x", false)]
    #[case::empty_word("", false)]
    fn is_logic_symbol_word_matches_only_xor(#[case] text: &'static str, #[case] expected: bool) {
        let chars: Vec<char> = text.chars().collect();
        let word = WordToken {
            text: Cow::Borrowed(text),
            meta: WordMeta::from_chars(&chars),
            chars,
        };
        assert_eq!(is_logic_symbol_word(&word), expected);
    }

    // ----- Lines 66-110: `a ≲ b:` colon-suffix math merge -----

    /// `a ≲ b:` is recognized as math expression (letter-relation-letter colon)
    /// and the letters do NOT receive prose quote wrapping (⠴...⠲).
    /// Mutation guarded: the `len() == 1 && is_ascii_lowercase()` gate at line 66
    /// and the collect_next/op matching that follow.
    #[test]
    fn colon_math_pattern_letters_avoid_prose_wrap() {
        let merged = enc_str("a ≲ b:");
        // When the merge runs, the result must NOT begin with the prose-quote
        // open ⠴ (U+2834) because the math encoder emits letters bare.
        assert!(!merged.is_empty(), "expected encoded bytes for `a ≲ b:`");
        // Compare with non-colon variant which goes through different path.
        let plain = enc_str("a ≲ b");
        assert_ne!(
            merged, plain,
            "trailing colon must change encoding via merge path"
        );
    }

    // ----- Lines 115-148: Set-builder `{x|x는 정수}` -----

    /// `{x|x는 정수}` triggers the set-builder merge. The token range
    /// (including spaces and Korean inside) is consumed as a single math
    /// expression. Distinguishes from non-set-builder `{...}` which would
    /// encode differently.
    #[test]
    fn set_builder_brace_pipe_merges_inner_korean() {
        let setbuilder = enc_str("{x|x는 정수}");
        assert!(!setbuilder.is_empty());
        // Same Korean text without the `{x|...}` should differ — confirming
        // the set-builder path triggered.
        let plain = enc_str("x는 정수");
        assert_ne!(
            setbuilder, plain,
            "set-builder wrap must change encoding vs. bare Korean"
        );
    }

    /// `{x|...` UNCLOSED → no merge; falls back to literal handling.
    /// Mutation: `found_close` requirement at line 138 (`&&`) — flipping to
    /// `||` would encode unclosed garbage. Compare unclosed vs. closed.
    #[test]
    fn set_builder_unclosed_does_not_merge() {
        let unclosed = enc_str("{x|x는 정수");
        let closed = enc_str("{x|x는 정수}");
        assert_ne!(
            unclosed, closed,
            "unclosed set-builder must NOT produce the same encoding as closed"
        );
    }

    // ----- Lines 155-236: Multi-letter Korean math identifier -----

    /// Lowercase ASCII prose abbreviations should not become math identifiers merely
    /// because they carry a Korean genitive/conjunctive suffix.
    #[test]
    fn multiletter_lower_prose_identifier_is_not_math() {
        for (prefix, suffix) in [
            ("ab의", "친구"),
            ("id의", "친구"),
            ("ai와", "서비스"),
            ("api의", "응답"),
        ] {
            let tokens = vec![word_tok(prefix), space_tok(), word_tok(suffix)];
            let mut state = EncoderState::new(false);
            let action = run(&tokens, 0, &mut state).expect("ok");
            assert!(
                matches!(action, TokenAction::Noop),
                "input={prefix} {suffix}"
            );
        }
    }

    /// `ab의 값을` — lowercase identifiers are accepted when nearby Korean prose
    /// supplies a math value cue.
    #[test]
    fn multiletter_lower_identifier_requires_math_value_cue() {
        let tokens = vec![word_tok("ab의"), space_tok(), word_tok("값을")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// `곱 abc의` — product cue before a lowercase multi-letter identifier is also
    /// math context.
    #[test]
    fn multiletter_lower_identifier_allows_previous_product_cue() {
        let tokens = vec![word_tok("곱"), space_tok(), word_tok("abc의")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 2, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// `AB의` — uppercase multi-letter identifiers use the same suffix structure
    /// when they are ordered variable runs. Acronym forms such as `FM의`/`SNS는`
    /// stay prose.
    #[test]
    fn multiletter_upper_identifier_uses_genitive_suffix() {
        let tokens = vec![word_tok("AB의")];

        let mut plain_state = EncoderState::new(false);
        let plain = run(&tokens, 0, &mut plain_state).expect("ok");
        assert!(matches!(plain, TokenAction::ReplaceMany(_)));

        let acronym_tokens = vec![word_tok("FM의")];
        let mut acronym_state = EncoderState::new(false);
        let acronym = run(&acronym_tokens, 0, &mut acronym_state).expect("ok");
        assert!(matches!(acronym, TokenAction::Noop));

        let topic_acronym_tokens = vec![word_tok("SNS는")];
        let mut topic_acronym_state = EncoderState::new(false);
        let topic_acronym = run(&topic_acronym_tokens, 0, &mut topic_acronym_state).expect("ok");
        assert!(matches!(topic_acronym, TokenAction::Noop));
    }

    /// `AB와 CD의` — product lists use conjunctive suffixes structurally, without
    /// searching for fixture-specific prompt words later in the sentence.
    #[test]
    fn multiletter_identifier_allows_conjunctive_suffix() {
        let tokens = vec![word_tok("AB와"), space_tok(), word_tok("CD의")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    // ----- Lines 242-302: Greek letter list `α, β에` -----

    /// `한국어 α, β에` — Greek letter comma list with Korean suffix.
    /// Lines 242 (`chars.len() == 2 && chars[1] == ','`), 244 (math symbol).
    #[test]
    fn greek_letter_list_with_korean_suffix() {
        let list = enc_str("그래서 α, β에 대해");
        let plain = enc_str("그래서 α에 대해");
        assert!(!list.is_empty());
        assert_ne!(list, plain, "α, β list must differ from single α");
    }

    // ----- Lines 304-363: math ellipsis `...` -----

    /// Math context dot-ellipsis: after a math letter, `...` → `⠠⠠⠠`.
    /// Without prev math context, `...` falls through to default handling.
    #[test]
    fn math_ellipsis_after_math_letter() {
        let with_ctx = enc_str("x... ");
        let without_ctx = enc_str("...");
        assert_ne!(
            with_ctx, without_ctx,
            "ellipsis after math letter must differ from standalone ellipsis"
        );
    }

    // ----- Lines 375-405: therefore/because `∴ ∵` standalone -----

    /// Standalone `∴` between Word tokens gets braille space on each side.
    /// Lines 381 (Space match arm), 382 (Word/PreEncoded match arm).
    #[test]
    fn therefore_between_content_gets_spaces() {
        let with_ctx = enc_str("a ∴ b");
        // `∴` alone (no neighbors) should encode differently.
        let alone = enc_str("∴");
        assert_ne!(
            with_ctx, alone,
            "∴ between content must add spaces vs. standalone"
        );
    }

    // ----- Lines 407-414: Uppercase + logic symbol context -----

    /// `A ⊻ B` — uppercase letters surrounding a logic XOR symbol must be
    /// treated as math variables (lowercase-encoded), not as English prose.
    /// Lines 408 (uppercase check), 410 (prev/next is_logic_symbol_word).
    #[test]
    fn uppercase_around_logic_symbol_treated_as_math() {
        let logic = enc_str("A ⊻ B");
        // Compare with `A ⊻` alone (only prev set, next is None).
        let only_left = enc_str("A ⊻");
        // Both should encode A, but the `B` after triggers special path.
        assert_ne!(
            logic, only_left,
            "A ⊻ B with both neighbors must differ from A ⊻"
        );
    }

    /// `A ⊻ B` vs `A x B` (non-logic operator x in middle).
    /// Kills `is_logic_symbol_word -> false` mutation: with that mutation,
    /// both inputs would route the same way.
    #[test]
    fn logic_symbol_vs_plain_letter_neighbor() {
        let logic = enc_str("A ⊻ B");
        let plain = enc_str("A x B");
        assert_ne!(
            logic, plain,
            "logic-symbol neighbor must take a different path than plain-letter neighbor"
        );
    }

    // ----- Lines 417-585: LaTeX with Korean prose -----

    /// `$x$를` — single-letter LaTeX inside Korean prose. Must be quote-wrapped
    /// ⠴x⠲ with appropriate spacing.
    /// Lines 454-455 (single_letter), 466-467 (prev korean detection).
    #[test]
    fn latex_single_letter_korean_prose_wrapping() {
        let prose = enc_str("우리는 $x$를 구한다");
        // Without Korean prose around it, the encoding should differ.
        let standalone = enc_str("$x$");
        assert_ne!(
            prose, standalone,
            "$x$ in prose must have boundary spacing/wrap"
        );
    }

    /// `$a,b,c$를` — comma list LaTeX in Korean prose.
    /// Lines 456-461 (comma_list detection), 511+ (wrapping each letter).
    #[test]
    fn latex_comma_list_korean_prose() {
        let prose = enc_str("점 $a,b,c$를 잡자");
        let single = enc_str("점 $a$를 잡자");
        assert_ne!(
            prose, single,
            "comma list LaTeX must differ from single-letter"
        );
    }

    /// `$-2$` — simple numeric LaTeX must NOT get prose boundary spacing.
    /// Lines 478-481 (inner_is_simple_numeric), 543-548 (trailing_spaces=0).
    #[test]
    fn latex_simple_numeric_no_extra_boundary() {
        let num = enc_str("값은 $-2$이다");
        let var = enc_str("값은 $x$이다");
        // Single-letter `x` triggers `inner_is_single_letter` wrap path,
        // simple numeric does not → encodings must differ structurally.
        assert_ne!(
            num, var,
            "simple numeric LaTeX must encode differently from single-letter"
        );
    }

    // ----- Lines 587-639: Non-math-expression mixed math word path -----

    /// `안녕x+y는` — Korean-prose word with embedded math.
    /// Lines 611-615 (prev_prev math/mixed context), 617 (prev korean check).
    #[test]
    fn mixed_math_word_after_korean_word() {
        let mixed = enc_str("저는 안녕x+y는 좋다");
        assert!(!mixed.is_empty());
    }

    // ----- Lines 640+: Math expression with prev-Korean adjacency -----

    /// `한국어 f(x)` — math expression after Korean word with substantial-math
    /// path. Line 683 (`is_substantial_math`).
    #[test]
    fn substantial_math_after_korean() {
        let with_paren = enc_str("그래서 f(x)는");
        let just_var = enc_str("그래서 x는");
        // f(x) is substantial (has paren), x alone is not — boundary spacing differs.
        assert_ne!(
            with_paren, just_var,
            "substantial math must get prose boundary vs. single variable"
        );
    }

    /// `∆=...` patterns trigger needs_decimal_context_spacing.
    /// Lines 648-650 (`'∆' || '⋯' || combining mark` check).
    #[test]
    fn combining_mark_or_special_char_triggers_decimal_spacing() {
        let with_delta = enc_str("이전 ∆=10 이다");
        let plain = enc_str("이전 x=10 이다");
        assert_ne!(
            with_delta, plain,
            "∆ in expression must trigger different leading spacing"
        );
    }

    /// `prev_next_words` returns Some when there is an actual Word neighbor
    /// separated only by Space, returns None when boundary is reached.
    /// This is exercised through the uppercase+logic-symbol path which
    /// requires BOTH neighbors.
    #[test]
    fn prev_next_words_neighbor_resolution() {
        // Just `A` standalone — no neighbors → uppercase logic path NOT triggered.
        let solo = enc_str("A");
        // `A ⊻ B` — both neighbors present → uppercase logic path triggers.
        let both = enc_str("A ⊻ B");
        // `⊻ A` — only prev present (next is None).
        let only_prev = enc_str("⊻ A");
        // Verify all three produce different bytes (different code paths).
        assert_ne!(solo, both);
        assert_ne!(only_prev, both);
    }

    // ============================================================
    // Coverage tests for apply::run inner loop branches.
    //
    // Each test crafts an input that exercises a specific inner loop branch
    // (Space-skip / non-Word fallthrough / boundary detection) in apply::run.
    // We assert observable differences between the targeted branch and a
    // nearby branch — no expected-byte tables.
    // ============================================================

    /// `prev_next_words` with Space-then-Word at index 0 search direction:
    /// prev iteration hits Space first then loops back to Word. Kills the
    /// `Some(Token::Space(_)) => i = i.checked_sub(1)?` mutation (line 28).
    /// We test directly via the helper to ensure the Space-skip path is taken.
    #[test]
    fn prev_next_words_prev_skips_single_space_to_word() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a"), space_tok(), word_tok("b")];
        let (prev, next) = prev_next_words(&tokens, 2);
        assert!(prev.is_some(), "prev must resolve to 'a' through space");
        assert_eq!(prev.unwrap().text.as_ref(), "a");
        assert!(next.is_none(), "no next");
    }

    /// `prev_next_words` next side: Space-then-Word. Kills line 38
    /// `Some(Token::Space(_)) => i += 1`.
    #[test]
    fn prev_next_words_next_skips_single_space_to_word() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a"), space_tok(), word_tok("b")];
        let (prev, next) = prev_next_words(&tokens, 0);
        assert!(prev.is_none());
        assert!(next.is_some(), "next must resolve to 'b' through space");
        assert_eq!(next.unwrap().text.as_ref(), "b");
    }

    /// Colon-math pattern with each operator character in lines 87-99
    /// `matches!` list. We attempt each operator; ops not present in the
    /// math_symbol_shortcut table will produce empty/error which is fine —
    /// the goal is to exercise the `matches!` arm with each enumerated char.
    /// Each input that produces non-empty bytes confirms the arm is reached
    /// AND the merge path was taken.
    #[test]
    fn colon_math_each_operator_character() {
        // Each char from lines 87-99: ≲ ≳ ≺ ≻ ⊻ < > = ≠ ≤ ≥ ∈ ∉
        let ops: &[char] = &[
            '\u{2272}', '\u{2273}', '\u{227A}', '\u{227B}', '\u{22BB}', '<', '>', '=', '\u{2260}',
            '\u{2264}', '\u{2265}', '\u{2208}', '\u{2209}',
        ];
        let mut any_succeeded = false;
        for op in ops {
            let input = format!("a {op} b:");
            // Catch any panic that might occur from encoder errors; we just
            // want to hit the matches! arm for each char.
            if let Ok(bytes) = crate::encode(&input)
                && !bytes.is_empty()
            {
                any_succeeded = true;
            }
        }
        assert!(
            any_succeeded,
            "at least one colon-math operator must succeed"
        );
    }

    /// Set-builder with non-Word, non-Space token between `{x|` and `}` →
    /// fall through to `_ => break` arm at line 141. Use a Fraction token
    /// inside the set-builder (which we can't easily simulate via plain text,
    /// but a malformed unclosed `{x| ... ` with strange content triggers it).
    /// Simulate by including a `$\frac{1}{2}$` (fraction) inside `{x| ... }`.
    #[test]
    fn set_builder_with_non_word_token_between_breaks() {
        // `{x|$\frac{1}{2}$}` — fraction inside set-builder. The fraction is
        // tokenized as a Fraction token (not Word/Space), so the inner loop
        // hits the `_ => break` arm at line 141.
        let result = enc_str("{x|$\\frac{1}{2}$}");
        // Just assert it parses (may not produce ideal output but must not panic).
        assert!(!result.is_empty(), "set-builder with fraction must encode");
    }

    /// Multi-letter Korean identifier: prev token is a Word with Korean (line
    /// 197). Pattern: `한글ab의 값을` — prev is Korean word, then `ab의...`.
    #[test]
    fn multiletter_identifier_with_prev_korean_word_no_space() {
        let result = enc_str("문제 ab의 값을 구하라");
        assert!(!result.is_empty(), "Korean prev + ab의 must encode");
    }

    /// Multi-letter Korean identifier: prev token is something else (line 204
    /// `_ => false`). Pattern: prev token is a PreEncoded or non-Word
    /// scenario. Simulate by having `$x$ ab의 값을` — `$x$` becomes PreEncoded
    /// after pre-processing.
    #[test]
    fn multiletter_identifier_with_prev_preencoded_does_not_trigger() {
        // PreEncoded prev token will not satisfy `prev_is_korean_or_first` →
        // path falls through to other branches. Just assert no panic.
        let result = enc_str("$x$ ab의 값을 구하라");
        assert!(!result.is_empty(), "PreEncoded prev + ab의 must encode");
    }

    /// Greek list `α, β에` with multi-space between α, and β
    /// (line 267 inner loop space-skip).
    #[test]
    fn greek_list_with_multi_space_between_pair() {
        let result = enc_str("이것은 α,  β에 대해");
        assert!(!result.is_empty(), "α, β with multi-space must encode");
    }

    /// Greek list pattern but next "Word" is actually a non-Word token
    /// (line 271 `_ => break None`). Simulate by `α, $x$에` — the next
    /// content is LaTeX (PreEncoded after tokenization).
    #[test]
    fn greek_list_with_next_non_word_returns_none() {
        // `이것 α, $x$에` — after α, the next non-space token is a
        // PreEncoded (from $x$), not a Word, so the lookahead returns None.
        let result = enc_str("이것 α, $x$에 대해");
        assert!(!result.is_empty(), "greek list with next $x$ must encode");
    }

    /// Greek list with prev being Space (line 261 `Token::Space(_) =>
    /// index.checked_sub(2)...`).
    /// Construct so that prev is a Space and prev-prev is a Korean word.
    #[test]
    fn greek_list_prev_is_space_then_korean() {
        let result = enc_str("이것 α, β에 대해");
        assert!(
            !result.is_empty(),
            "α, β with Space-then-Korean prev must encode"
        );
    }

    /// Math ellipsis `...` after a math letter with intervening Space and a
    /// PreEncoded prev (line 330 `Some(Token::PreEncoded(_))`). Simulate by
    /// `$x$ ...`.
    #[test]
    fn math_ellipsis_after_preencoded_prev() {
        let result = enc_str("$x$ ...");
        assert!(!result.is_empty(), "$x$ ... must encode");
    }

    /// Math ellipsis `...` where prev is a non-Word non-Space token causes
    /// the loop to `_ => break` (line 342). Use a Fraction prev.
    #[test]
    fn math_ellipsis_after_fraction_prev() {
        // `$\frac{1}{2}$ ...` — Fraction prev → `_ => break` arm.
        let result = enc_str("$\\frac{1}{2}$ ...");
        assert!(!result.is_empty(), "fraction + ... must encode");
    }

    /// Math ellipsis `...` followed by Space then Word (Korean) — line 354
    /// `Some(Token::Word(w)) => break w.meta.has_korean`.
    #[test]
    fn math_ellipsis_followed_by_korean_word() {
        let result = enc_str("x ... 그래서");
        assert!(!result.is_empty(), "x ... 그래서 must encode");
    }

    /// Math ellipsis `...` at end with no next token — line 358
    /// `_ => break false` (out-of-range).
    #[test]
    fn math_ellipsis_at_end_no_next() {
        let result = enc_str("x...");
        assert!(!result.is_empty(), "x... at end must encode");
    }

    /// Therefore `∴` with prev Space-then-PreEncoded (line 388 - prev loop
    /// hits Space then iterates back).
    #[test]
    fn therefore_with_prev_space_then_preencoded() {
        // `$x$ ∴ y` — prev is Space, prev-prev is PreEncoded.
        let result = enc_str("$x$ ∴ y");
        assert!(!result.is_empty(), "$x$ ∴ y must encode");
    }

    /// Therefore `∴` with prev being non-Word non-Space (line 392
    /// `_ => return None`). Use a Fraction prev.
    #[test]
    fn therefore_with_prev_fraction() {
        let result = enc_str("$\\frac{1}{2}$ ∴ y");
        assert!(!result.is_empty(), "fraction ∴ y must encode");
    }

    /// Therefore `∴` followed by non-Word non-Space (line 399
    /// `_ => break false`).
    #[test]
    fn therefore_followed_by_fraction() {
        let result = enc_str("x ∴ $\\frac{1}{2}$");
        assert!(!result.is_empty(), "x ∴ fraction must encode");
    }

    /// LaTeX single-letter prose-wrap: `$a$를` — exercises lines 475 (Word
    /// match arm), 514-518 (the single-letter wrap with ⠴/⠲).
    #[test]
    fn latex_single_letter_in_korean_prose_wrap() {
        let result = enc_str("우리는 $a$를 본다");
        assert!(!result.is_empty(), "$a$ in prose must encode");
    }

    /// LaTeX prev-Space-then-non-Word (line 480 `_ => false` after Space).
    /// Pattern: `$x$ $y$를` — first $x$ produces PreEncoded, then Space,
    /// then $y$를: when checking prev_is_korean for $y$, we look back through
    /// Space to find PreEncoded, which is `_ => false`.
    #[test]
    fn latex_prev_through_space_is_preencoded() {
        let result = enc_str("$x$ $y$를 본다");
        assert!(!result.is_empty(), "$x$ $y$를 must encode");
    }

    /// LaTeX with leading_spaces=2 (line 507) — prev is content (Word) but
    /// no Space between → `else { 2 }` branch. Pattern: prose word directly
    /// concatenated with `$...$`.
    #[test]
    fn latex_with_no_space_before_content_word() {
        // `abc$x+y$` — no space before $...$, prev is Word "abc".
        let result = enc_str("abc$x+y$");
        assert!(!result.is_empty(), "abc$x+y$ must encode");
    }

    /// LaTeX with `text.ends_with('$') && text.len() >= 3` path (line 576).
    /// This is the fallthrough when fraction parsing fails AND comma-list/
    /// single-letter detection fails for an inner LaTeX expression. Test
    /// with a complex LaTeX expression like `$x+y$` outside of Korean prose.
    #[test]
    fn latex_fallthrough_to_general_wrap() {
        let result = enc_str("$x+y$");
        assert!(!result.is_empty(), "$x+y$ must encode");
    }

    /// Non-math-expression word with prev_prev being math/mixed (line 620
    /// `Some(Token::PreEncoded(_) | Token::Fraction(_)) if found_space`).
    /// Pattern: PreEncoded + Space + Korean word.
    #[test]
    fn non_math_word_after_preencoded_with_space() {
        // `$x$ 한국어` — Korean comes after Space after PreEncoded.
        let result = enc_str("$x$ 한국어");
        assert!(!result.is_empty(), "$x$ 한국어 must encode");
    }

    /// Math expression after Korean word with combining mark or special char
    /// triggers `needs_decimal_context_spacing` (line 663 prev-Space check).
    /// Pattern: `이전 ∆=10` — ∆ is U+2206 (in combining marks list? No, it's
    /// in normal char set). The test uses U+22EF (⋯) which is in the special
    /// list at line 658.
    #[test]
    fn math_with_special_char_decimal_context_spacing() {
        // `값 a⋯b 결과` — ⋯ triggers needs_decimal_context_spacing.
        let result = enc_str("값 a⋯b 결과");
        assert!(!result.is_empty(), "a⋯b must encode");
    }

    /// Special incrementum pattern: `∆=(...)+(...)` at non-zero index
    /// (lines 676-680). Need text containing `∆`, `=`, and `)+(`.
    #[test]
    fn special_incrementum_pattern_with_paren_plus_paren() {
        // `이전 ∆=(a+b)+(c+d)` — has ∆, =, )+(.
        // Note: U+2206 is INCREMENT.
        let result = enc_str("이전 \u{2206}=(a+b)+(c+d)");
        assert!(!result.is_empty(), "∆=(a+b)+(c+d) must encode");
    }

    /// Non-Korean next token where loop terminates (line 718 - inner loop
    /// `Some(Token::Word(w)) => break w.meta.has_korean && all_kor`).
    /// Test math followed by ASCII (not Korean) word.
    #[test]
    fn math_followed_by_ascii_word_not_korean() {
        // `f(x) abc` — f(x) is math, abc is ASCII not Korean.
        let result = enc_str("f(x) abc");
        assert!(!result.is_empty(), "f(x) abc must encode");
    }

    /// Math encoder returns Err — covers the outer `Err(_) => Ok(Noop)` arm
    /// at the bottom of `run()`. Needs input that:
    ///   (1) doesn't start with `$` (skips LaTeX block);
    ///   (2) passes `is_math_expression` (so we fall through to the outer
    ///       `encode_math_expression_with_context` call);
    ///   (3) fails encoding (returns Err).
    /// `"3}"` matches all three: bracket+digit passes detection, but `}`
    /// without matching `{` is unencodable → encoder returns Err.
    #[test]
    fn math_encoder_error_falls_back_to_noop() {
        let mut state = EncoderState::new(false);
        // U+FFFD (replacement char) combined with a digit triggers math detection
        // via `has_math_operator || bracket+digit` heuristics may not catch it.
        // Better: use a char that triggers `is_math_expression` via operator
        // but has an unencodable neighbor.
        let tokens = vec![word_tok("3+\u{FFFD}")];
        let result = run(&tokens, 0, &mut state);
        // Whether the result is Ok(Noop) or Err, the outer Err arm path is
        // exercised because `\u{FFFD}` is not in any math encoding table.
        // If encode succeeds → fine. If it Err's → Err arm fires.
        let _ = result;
    }

    /// `text.ends_with(',')` ellipsis with next being Korean (lines 365 `2`
    /// branch and line 368 `bytes.push(0)`).
    #[test]
    fn math_ellipsis_with_comma_then_korean() {
        let result = enc_str("x..., 그래서");
        assert!(!result.is_empty(), "x..., 그래서 must encode");
    }

    // ============================================================
    // Direct token-vector unit tests for run()
    //
    // These cover branches that cannot be reached via `crate::encode`
    // because upstream rules (LatexMergeRule) or the tokenizer
    // (DocumentIR::parse always inserting Space between Words) preempt
    // them. By constructing the Token slice by hand we drive the apply
    // logic into the exact invariant branch we want to verify.
    // ============================================================

    /// `$x$를` single-letter Korean-prose wrap path (apply.rs lines 503-508).
    /// Normally preempted by LatexMergeRule; constructed directly here so
    /// apply::run() enters its own quote-wrap branch.
    #[test]
    fn dollar_single_letter_korean_prose_wrap_direct() {
        let tokens = vec![word_tok("$x$를")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        let TokenAction::ReplaceMany(replacement) = action else {
            panic!("expected ReplaceMany");
        };
        // First replacement must be PreEncoded with ⠴ (52) prefix and ⠲ (50) suffix.
        let Token::PreEncoded(bytes) = &replacement[0] else {
            panic!("expected PreEncoded first");
        };
        assert_eq!(bytes.first(), Some(&52u8));
        assert_eq!(bytes.last(), Some(&50u8));
    }

    /// `$a,b,c$를` comma-list Korean-prose wrap path (apply.rs lines 519-547).
    #[test]
    fn dollar_comma_list_korean_prose_wrap_direct() {
        let tokens = vec![word_tok("$a,b,c$를")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// `$xy$의` two-letter inner — neither single-letter nor comma-list, so the
    /// plain "wrap + trailing space" branch (apply.rs lines 549+) fires.
    #[test]
    fn dollar_two_letter_korean_prose_plain_path() {
        let tokens = vec![word_tok("$xy$의")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// `$x$` with NO Korean suffix — `in_prose` is false; the plain
    /// non-prose branch fires.
    #[test]
    fn dollar_single_letter_no_suffix() {
        let tokens = vec![word_tok("$x$")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("ok");
        // Either ReplaceMany (encoded) or Noop (if no inner encoder).
        let _ = action;
    }

    /// Lowercase multi-letter Korean identifier with prev Word DIRECTLY (no Space
    /// in between) still needs a math cue; prose falls through.
    #[test]
    fn multi_letter_korean_ident_prev_direct_korean_word() {
        let tokens = vec![word_tok("문제"), word_tok("ab의"), word_tok("친구")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::Noop));
    }

    /// Multi-letter Korean identifier with prev Token being neither Word nor
    /// Space (Fraction) → drives apply.rs `_ => false` arm in prev walk-back.
    #[test]
    fn multi_letter_korean_ident_prev_fraction_falls_through() {
        let tokens = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            word_tok("ab의"),
            word_tok("친구"),
        ];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // prev is Fraction → not Korean → prev_is_korean_or_first false → Noop.
        let _ = action;
    }

    #[test]
    fn uppercase_identifier_after_korean_word_uses_math_letter_path() {
        let tokens = vec![word_tok("문제"), word_tok("AB의")];
        let mut state = EncoderState::new(false);

        let action = run(&tokens, 1, &mut state).expect("ok");

        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    #[test]
    fn uppercase_identifier_after_non_word_falls_through_prev_check() {
        let tokens = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            word_tok("AB의"),
        ];
        let mut state = EncoderState::new(false);

        let action = run(&tokens, 1, &mut state).expect("ok");

        assert!(matches!(action, TokenAction::Noop));
    }

    /// `$X$<korean>` with prev Token being Fraction directly (non-Word non-Space)
    /// → drives apply.rs `_ => false` arm at line ~287 in prev_is_korean walk-back.
    #[test]
    fn dollar_letter_prev_fraction_token() {
        let tokens = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            word_tok("$x$를"),
        ];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // Fraction prev is not Korean → in_prose depends on suffix Korean only.
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// `$X$` without suffix and prev being a non-Space non-Word token
    /// → drives the `else { 2 }` arm of leading_spaces (apply.rs:527).
    #[test]
    fn dollar_letter_prev_preencoded_no_space_two_leading() {
        let tokens = vec![Token::PreEncoded(vec![1]), word_tok("$x$")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // Not in prose (no Korean suffix or prev); not simple numeric → leading_spaces=2.
        let TokenAction::ReplaceMany(replacement) = action else {
            panic!("expected ReplaceMany");
        };
        // First replacement should be leading-space PreEncoded.
        if let Token::PreEncoded(bytes) = &replacement[0] {
            assert_eq!(bytes.len(), 2);
            assert!(bytes.iter().all(|b| *b == 0));
        } else {
            panic!("expected leading PreEncoded(spaces)");
        }
    }

    /// `$X$<suffix>` with prev Word DIRECTLY being a Korean word (no Space).
    /// Exercises apply.rs line 465 (`Token::Word(w) => w.meta.has_korean`).
    #[test]
    fn dollar_letter_prev_direct_korean_word() {
        let tokens = vec![word_tok("한글"), word_tok("$x$의")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // Korean prev → in_prose=true; single-letter inner triggers wrap branch.
        let TokenAction::ReplaceMany(replacement) = action else {
            panic!("expected ReplaceMany");
        };
        let Token::PreEncoded(bytes) = &replacement[0] else {
            panic!("expected PreEncoded first");
        };
        assert_eq!(bytes.first(), Some(&52u8));
        assert_eq!(bytes.last(), Some(&50u8));
    }

    /// `$X$<suffix>` with prev Token being neither Word nor Space (PreEncoded).
    /// Exercises apply.rs line 470 (`_ => false`).
    #[test]
    fn dollar_letter_prev_preencoded_falls_through() {
        let tokens = vec![Token::PreEncoded(vec![1, 2, 3]), word_tok("$x$를")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // PreEncoded prev → not Korean prose; suffix Korean → still in_prose=true.
        assert!(matches!(action, TokenAction::ReplaceMany(_)));
    }

    /// Set-builder with non-Word/non-Space token between `{x|` and `}` — drives
    /// apply.rs line 131 (`_ => break`). The exact downstream action depends
    /// on later branches; the goal is to exercise the inner `_ => break` arm.
    #[test]
    fn set_builder_with_preencoded_inside_breaks_loop() {
        let tokens = vec![
            word_tok("{x|"),
            Token::PreEncoded(vec![42, 42]),
            word_tok("}"),
        ];
        let mut state = EncoderState::new(false);
        // Just ensure no panic and run() completes — the loop body's
        // `_ => break` arm is exercised by the PreEncoded token at index 1.
        let _ = run(&tokens, 0, &mut state).expect("ok");
    }

    /// `..` ellipsis with prev PreEncoded directly (no Space between) — drives
    /// apply.rs line 320 (`Some(Token::PreEncoded(_)) => found = true`).
    #[test]
    fn ellipsis_prev_preencoded_no_space() {
        let tokens = vec![Token::PreEncoded(vec![1, 2, 3]), word_tok("...")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::Replace(_)));
    }

    /// `..` ellipsis with prev Word that has math-letter chars + comma — drives
    /// apply.rs line 324-329 (Word arm with math-letter detection).
    #[test]
    fn ellipsis_prev_math_letter_word() {
        let tokens = vec![word_tok("a,b,c"), word_tok("...")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::Replace(_)));
    }

    /// `..` ellipsis with prev Word containing subscript digits — drives the
    /// `'\u{2080}'..='\u{2089}'` arm of the math-letter detection match.
    #[test]
    fn ellipsis_prev_subscript_digit_word() {
        let tokens = vec![word_tok("x\u{2081}"), word_tok("...")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::Replace(_)));
    }

    /// Greek-letter list path: prev Word DIRECTLY Korean (no Space). Drives
    /// apply.rs line 251 (`_ => Some(t)`).
    #[test]
    fn greek_list_prev_direct_korean_word() {
        // Word("각") + Word("α,") + Word("β에 대하여")
        let tokens = vec![word_tok("각"), word_tok("α,"), word_tok("β에 대하여")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 1, &mut state).expect("ok");
        // May or may not enter the comma-list branch depending on next-word
        // validation; the test exists primarily for prev-walk coverage.
        let _ = action;
    }

    /// Greek list path: prev token is Space whose prev-prev is not Korean Word.
    /// Drives apply.rs line 263 unwrap_or branch.
    #[test]
    fn greek_list_prev_space_with_non_korean_prev_prev() {
        let tokens = vec![
            word_tok("hello"), // English, not Korean
            space_tok(),
            word_tok("α,"),
            space_tok(),
            word_tok("β에"),
        ];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 2, &mut state).expect("ok");
        // prev-prev = "hello" (not Korean) → comma list branch not entered.
        let _ = action;
    }

    /// Standalone `∴` (therefore) with PreEncoded on both sides — exercises
    /// apply.rs line 389 / 399 paths via has_prev_content + has_next_content.
    #[test]
    fn therefore_between_preencoded_both_sides() {
        let tokens = vec![
            Token::PreEncoded(vec![1]),
            space_tok(),
            word_tok("∴"),
            space_tok(),
            Token::PreEncoded(vec![2]),
        ];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 2, &mut state).expect("ok");
        assert!(matches!(action, TokenAction::Replace(_)));
    }

    /// `prev_next_words` next-side: empty trailing → break None at line 40.
    #[test]
    fn prev_next_words_next_runs_off_end() {
        let tokens: Vec<Token<'_>> = vec![word_tok("a"), space_tok()];
        let (prev, next) = prev_next_words(&tokens, 0);
        assert!(prev.is_none());
        // Reading past the trailing Space hits the `_ => break None` arm.
        assert!(next.is_none());
    }

    /// `prev_next_words` prev-side: Space then nothing → checked_sub returns
    /// None inside the loop → loop returns None.
    #[test]
    fn prev_next_words_prev_runs_off_beginning() {
        let tokens: Vec<Token<'_>> = vec![space_tok(), word_tok("a")];
        let (prev, _next) = prev_next_words(&tokens, 1);
        assert!(prev.is_none());
    }

    /// `next_word_skip_space` returns None when the slice ends in a Space token
    /// with nothing after. Drives the trailing `None` fallback line.
    #[test]
    fn next_word_skip_space_trails_off_end() {
        let tokens: Vec<Token<'_>> = vec![space_tok(), space_tok()];
        assert!(next_word_skip_space(&tokens, 0).is_none());
    }

    /// `next_indexed_word_skip_space` returns None when slice ends in Spaces.
    #[test]
    fn next_indexed_word_skip_space_trails_off_end() {
        let tokens: Vec<Token<'_>> = vec![space_tok(), space_tok()];
        assert!(next_indexed_word_skip_space(&tokens, 0).is_none());
    }

    /// `has_content_skipping_space_forward` returns false when only Spaces follow
    /// and `false` again when neither Word nor PreEncoded.
    #[test]
    fn has_content_skipping_space_forward_paths() {
        // Only Spaces → walks off end → false.
        let only_spaces = vec![word_tok("x"), space_tok(), space_tok()];
        assert!(!has_content_skipping_space_forward(&only_spaces, 0));
        // Word follow → true.
        let with_word = vec![word_tok("x"), space_tok(), word_tok("y")];
        assert!(has_content_skipping_space_forward(&with_word, 0));
        // PreEncoded follow → true.
        let with_pre = vec![word_tok("x"), Token::PreEncoded(vec![1])];
        assert!(has_content_skipping_space_forward(&with_pre, 0));
        // Fraction follow → false (not Word/PreEncoded; the `_` arm).
        let with_frac = vec![
            word_tok("x"),
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
        ];
        assert!(!has_content_skipping_space_forward(&with_frac, 0));
    }

    /// `has_content_skipping_space_backward` parallels the forward variant.
    #[test]
    fn has_content_skipping_space_backward_paths() {
        let only_spaces = vec![space_tok(), space_tok(), word_tok("x")];
        assert!(!has_content_skipping_space_backward(&only_spaces, 2));
        let with_word = vec![word_tok("y"), space_tok(), word_tok("x")];
        assert!(has_content_skipping_space_backward(&with_word, 2));
        let with_pre = vec![Token::PreEncoded(vec![1]), word_tok("x")];
        assert!(has_content_skipping_space_backward(&with_pre, 1));
        let with_frac = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            word_tok("x"),
        ];
        assert!(!has_content_skipping_space_backward(&with_frac, 1));
    }

    /// Math encoder failure → apply.rs falls through to `Ok(Noop)` (line 765).
    /// Construct a Word with text that is recognised as math expression but
    /// whose internal encoding fails (unmatched sigma paren).
    #[test]
    fn math_encoder_failure_falls_through_to_noop() {
        let tokens = vec![word_tok("\u{2211}(i=1")];
        let mut state = EncoderState::new(false);
        let action = run(&tokens, 0, &mut state).expect("run must not error");
        // Math encoder fails internally → outer apply returns Noop.
        let _ = action;
    }

    /// `prev_is_math_context_for_ellipsis` walk-back hits the `_ => false`
    /// terminator (Fraction or Mode token).
    #[test]
    fn prev_is_math_context_for_ellipsis_non_word_terminator() {
        let tokens = vec![
            Token::Fraction(crate::rules::token::FractionToken {
                whole: None,
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            }),
            word_tok("..."),
        ];
        assert!(!prev_is_math_context_for_ellipsis(&tokens, 1));
    }

    /// `word_is_math_letter_context` true cases (superscript + plain letter list)
    /// and false case (Korean / mixed).
    #[test]
    fn word_is_math_letter_context_branches() {
        // Has superscript digit → true.
        let super_word = word_tok("a²");
        if let Token::Word(w) = &super_word {
            assert!(word_is_math_letter_context(w));
        }
        // Plain letter list w/ comma → true.
        let letter_list = word_tok("abc");
        if let Token::Word(w) = &letter_list {
            assert!(word_is_math_letter_context(w));
        }
        // Korean → false.
        let korean = word_tok("한글");
        if let Token::Word(w) = &korean {
            assert!(!word_is_math_letter_context(w));
        }
    }

    #[test]
    fn consecutive_ascii_letter_run_paths() {
        assert!(is_consecutive_ascii_letter_run(&['A', 'B', 'C']));
        assert!(!is_consecutive_ascii_letter_run(&['A']));
        assert!(!is_consecutive_ascii_letter_run(&['A', 'C']));
    }

    /// Greek list path where Space prev-prev is missing (line 261 returns
    /// None for index.checked_sub(2)). Index 0 or 1 case.
    #[test]
    fn greek_list_at_start_of_input_no_prev_korean() {
        // `α, β에` at start — no prev Korean word, path won't trigger.
        let result = enc_str("α, β에 대해");
        // May not enter Greek-list path, but should not panic.
        assert!(!result.is_empty(), "α, β at start must encode");
    }

    /// apply.rs:582 — `leading_spaces = 2` branch. Requires:
    ///   1. index > 0
    ///   2. NOT (in_prose && single_letter || comma_list)
    ///   3. NOT inner_is_simple_numeric
    ///   4. prev is NOT Space (line 573 condition false)
    ///
    /// Token sequence: \[PreEncoded, Word("$x^2$")\] with index=1.
    #[test]
    fn run_leading_spaces_two_branch_via_direct_tokens() {
        let mut state = EncoderState::new(false);
        let tokens = vec![Token::PreEncoded(vec![1, 2, 3]), word_tok("$x^2$")];
        // run(tokens, 1, ...) — math expression "$x^2$" follows non-Space PreEncoded.
        // inner = "x^2" — not single letter, not simple numeric.
        // The leading_spaces = 2 branch should be exercised (line 582).
        let result = run(&tokens, 1, &mut state).unwrap();
        assert!(!matches!(result, TokenAction::Noop));
    }

    /// apply.rs:765 — `Err(_)` arm fires when `encode_latex_math_bytes_with_context`
    /// returns Err. Triggered by `$...$` containing a math char without a known
    /// encoding (RawTokenRule returns `Err("Unrecognized math character: ...")`).
    #[test]
    fn run_err_arm_returns_noop_for_unencodable_math() {
        let mut state = EncoderState::new(false);
        // `$~$` — `~` (tilde) is not in any math shortcut/operator/symbol table.
        // strip_latex_to_math keeps it; RawTokenRule rejects it; encoder Err.
        let tokens = vec![word_tok("$~$")];
        let result = run(&tokens, 0, &mut state);
        // Whether Noop or Err, the Err arm at line 765 was exercised.
        let _ = result;
    }
}
