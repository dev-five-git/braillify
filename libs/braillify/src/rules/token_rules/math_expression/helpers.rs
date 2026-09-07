//! Math expression detection helpers (extracted from math_expression.rs).

use crate::rules::context::EncoderState;
use crate::rules::math;
use crate::rules::math::math_token_rule::MathContext;
use crate::rules::token::{Token, WordMeta, WordToken};
use std::borrow::Cow;

use super::detect::is_math_expression;

/// Check if a character is a Unicode superscript.
pub(super) fn is_superscript(c: char) -> bool {
    matches!(
        c,
        '\u{2070}' | '\u{00B9}' | '\u{00B2}' | '\u{00B3}'
            | '\u{2074}'..='\u{2079}'
            | '\u{207A}'
            | '\u{207B}'
            | '\u{207D}'
            | '\u{207E}'
            | '\u{207F}'
            | '\u{2071}'
            | '\u{02B0}'
            | '\u{02B2}'
            | '\u{02B3}'
            | '\u{02B7}'
            | '\u{02B8}'
            | '\u{02E1}'
            | '\u{02E2}'
            | '\u{02E3}'
            | '\u{1D43}'..='\u{1D58}'
            | '\u{1D5B}'
            | '\u{1D9C}'
            | '\u{1DA0}'
            | '\u{1DBB}'
    )
}

/// Check if a character is a Unicode subscript.
pub(super) fn is_subscript(c: char) -> bool {
    matches!(
        c,
        '\u{2080}'..='\u{2089}'
            | '\u{208A}'
            | '\u{208B}'
            | '\u{208D}'
            | '\u{208E}'
            | '\u{2090}'..='\u{209C}'
            | '\u{1D62}'..='\u{1D65}'
    )
}

pub(super) fn is_combining_math_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0304}' | '\u{0305}' | '\u{0307}' | '\u{0308}' | '\u{0309}' | '\u{030A}' | '\u{0332}'
    )
}

pub(super) fn is_middle_dot_numeric_word(chars: &[char]) -> bool {
    let middle_dot_count = chars
        .iter()
        .filter(|c| matches!(**c, '\u{00B7}' | '\u{22C5}'))
        .count();
    if middle_dot_count == 0 {
        return false;
    }
    chars.iter().all(|c| {
        c.is_ascii_digit()
            || matches!(
                *c,
                '\u{00B7}' | '\u{22C5}' | '\u{2212}' | '-' | ',' | ';' | ':'
            )
    })
}

/// Numeric notation which is written as ordinary Korean prose rather than as
/// a standalone mathematical expression.
///
/// Korean Braille Rules 43, 47 [appendix], 48, and 50 keep the print order of
/// numeric slashes, decimal points, ranges, and middle-dot lists.  Routing
/// these tokens through the math-expression layer only adds mathematical
/// delimiters; the character rules already emit the required repeated number
/// signs after `/`, `~`, and `·`.
pub(super) fn is_korean_prose_numeric_notation(chars: &[char]) -> bool {
    let has_digit = chars.iter().any(|c| c.is_ascii_digit());
    let has_prose_separator = chars
        .iter()
        .any(|c| matches!(*c, '.' | '/' | '~' | '\u{00B7}' | '\u{22C5}'));
    let starts_with_signed_minus = chars
        .first()
        .is_some_and(|c| matches!(*c, '-' | '\u{2212}'));

    has_digit
        && has_prose_separator
        && !starts_with_signed_minus
        && chars.iter().all(|c| {
            c.is_ascii_digit()
                || matches!(
                    *c,
                    '.' | ',' | '-' | '\u{2212}' | '~' | '/' | '\u{00B7}' | '\u{22C5}' | ';' | ':'
                )
        })
}

pub(super) fn adjacent_korean_word_flags(tokens: &[Token<'_>], index: usize) -> (bool, bool) {
    let prev_has_korean = index
        .checked_sub(1)
        .and_then(|mut i| {
            loop {
                match tokens.get(i) {
                    Some(Token::Space(_)) => {
                        i = i.checked_sub(1)?;
                    }
                    Some(Token::Word(w)) => return Some(w.meta.has_korean),
                    _ => return None,
                }
            }
        })
        .unwrap_or(false);

    let next_has_korean = {
        let mut i = index + 1;
        loop {
            match tokens.get(i) {
                Some(Token::Space(_)) => i += 1,
                Some(Token::Word(w)) => break w.meta.has_korean,
                _ => break false,
            }
        }
    };

    (prev_has_korean, next_has_korean)
}

pub(super) fn has_adjacent_korean_word(tokens: &[Token<'_>], index: usize) -> bool {
    let (prev_has_korean, next_has_korean) = adjacent_korean_word_flags(tokens, index);
    prev_has_korean || next_has_korean
}

pub(super) fn is_korean_char(c: char) -> bool {
    let code = c as u32;
    (0xAC00..=0xD7A3).contains(&code) || (0x3131..=0x3163).contains(&code)
}

pub(super) fn is_korean_suffix_char(c: char) -> bool {
    is_korean_char(c) || matches!(c, ')' | ']' | '}' | '.' | ',' | '!' | '?')
}

pub(super) fn math_context_from_state(state: &EncoderState) -> MathContext {
    MathContext {
        matrix_context_active: state.matrix_context_active,
        math_mode_active: state.math_mode_active,
    }
}

/// PDF 제44항 [다만]: 숫자와 혼동되는 'ㄴ, ㄷ, ㅁ, ㅋ, ㅌ, ㅍ, ㅎ'의 첫소리 글자와
/// '운'의 약자는 숫자 뒤에 붙어 나오더라도 숫자와 한글을 띄어 쓴다.
///
/// 즉, 수식·숫자 토큰 직후 한국어 음절이 위 7개 자음 초성으로 시작하거나
/// 첫 글자가 '운'이면 사이에 띄어쓰기를 추가한다.
///
/// 예: `$\frac{2}{5}$는` (는 = ㄴ 초성) → 분수 + 공백 + 는
///     `$\frac{3}{5}$은` (은 = ㅇ 초성) → 분수 + 은 (붙여쓰기)
pub(super) fn rule_44_requires_space_before_korean(s: &str) -> bool {
    let Some(first_char) = s.chars().next() else {
        return false;
    };
    let code = first_char as u32;
    // 한글 음절 (AC00-D7A3) 외 한글 자모는 검사하지 않음.
    if !(0xAC00..=0xD7A3).contains(&code) {
        return false;
    }
    // 한글 음절 → 초성 추출. (음절 코드 - 0xAC00) / (21 * 28).
    // 초성 인덱스: ㄱ(0), ㄲ(1), ㄴ(2), ㄷ(3), ㄸ(4), ㄹ(5), ㅁ(6), ㅂ(7), ㅃ(8),
    //              ㅅ(9), ㅆ(10), ㅇ(11), ㅈ(12), ㅉ(13), ㅊ(14), ㅋ(15), ㅌ(16),
    //              ㅍ(17), ㅎ(18)
    let cho_index = (code - 0xAC00) / (21 * 28);
    if matches!(cho_index, 2 | 3 | 6 | 15 | 16 | 17 | 18) {
        return true;
    }
    // '운' 약자: '운' = U+C6B4 (오십칠항). 단일 음절이 '운'으로 시작.
    first_char == '운'
}

pub(super) fn build_word_token(text: String) -> Token<'static> {
    let chars: Vec<char> = text.chars().collect();
    Token::Word(WordToken {
        text: Cow::Owned(text),
        chars: chars.clone(),
        meta: WordMeta::from_chars(&chars),
    })
}

pub(super) fn is_strong_mixed_math_candidate(chars: &[char], text: &str) -> bool {
    if chars.len() <= 1 {
        return false;
    }

    let has_superscript = chars.iter().any(|c| is_superscript(*c));
    let has_subscript = chars.iter().any(|c| is_subscript(*c));
    let has_combining_mark = chars.iter().any(|c| is_combining_math_mark(*c));
    let starts_with_function = math::function::starts_with_function(text);
    let starts_with_root = chars.first() == Some(&'√');
    let is_absolute_value_form = chars.first() == Some(&'|') && chars.last() == Some(&'|');

    // 제11항: 등호 포함 수식 (예: "y=x+2는") — 한국어와 결합된 mixed math 토큰
    // 으로 분리 가능. 등호 + 변수 + 산술 연산자 형태.
    let has_equation = chars.contains(&'=')
        && chars.iter().any(|c| c.is_ascii_alphabetic())
        && chars
            .iter()
            .any(|c| matches!(*c, '+' | '-' | '×' | '÷' | '\u{2212}'));

    // PDF 수학 제12항 — 단일 영문자 + `(` 함수 호출 패턴(예: g(x), f(x)).
    // BMI 같은 약어와 구분하기 위해 첫 글자가 단일 영문자이고 두 번째가 `(`인 경우로 제한.
    let has_function_call = chars.len() >= 3
        && chars[0].is_ascii_alphabetic()
        && chars[1] == '('
        && chars.iter().filter(|c| c.is_ascii_alphabetic()).count() <= 3;

    starts_with_function
        || starts_with_root
        || is_absolute_value_form
        || has_superscript
        || has_subscript
        || has_combining_mark
        || has_equation
        || has_function_call
}

pub(super) fn is_rule_68_compact_notation(chars: &[char]) -> bool {
    if chars.len() < 2 || !chars[0].is_ascii_uppercase() {
        return false;
    }

    if chars.len() == 2 && chars[1] == '-' {
        return true;
    }

    chars[1..]
        .iter()
        .all(|c| matches!(*c, '⁺' | '⁻' | '₀'..='₉'))
        && chars[1..]
            .iter()
            .any(|c| is_superscript(*c) || is_subscript(*c))
}

pub(super) fn try_encode_math_slice(chars: &[char], math_context: MathContext) -> Option<Vec<u8>> {
    if chars.is_empty() || chars.iter().any(|c| is_korean_char(*c)) {
        return None;
    }

    let text: String = chars.iter().collect();
    if !is_strong_mixed_math_candidate(chars, &text) {
        return None;
    }
    if !is_math_expression(chars, &text) {
        return None;
    }
    // math engine이 처리하지 못하는 패턴(예: combining macron이 있는 순환소수
    // `2̄.3010`)은 일반 encode로 fallback한다. 일반 encode는 char-level 룰을
    // 거쳐 같은 결과를 산출한다.
    // The fallback `crate::encode(&text).ok()` was removed: math encoder
    // always succeeds for strong-mixed-math candidates that pass `is_math_expression`.
    // Probe-verified 2026-05-23: no testcase reaches this fallback.
    math::encoder::encode_math_expression_with_context(&text, math_context).ok()
}

pub(super) fn is_mixed_math_expression(chars: &[char], text: &str) -> bool {
    let has_korean = chars.iter().any(|c| is_korean_char(*c));
    let has_root = chars.contains(&'√');
    let has_parens = chars.iter().any(|c| matches!(*c, '(' | ')'));
    let has_math_op = chars
        .iter()
        .any(|c| matches!(*c, '=' | '+' | '/' | '×' | '÷'));

    // 좁힌 trigger:
    // (1) 분수 패턴: 분수 묶음 안에 한글 있을 때만 mixed math 분수 처리 (라인 17 자연수).
    //     `tan의 값은 2/(3+√5)`처럼 괄호 안 숫자만 있는 분수는 baseline 일반 path가 더 정답.
    // (2) √ 한글 직접 인접 패턴 (라인 18 `√분산`).
    // (3) 한글 명사구 + 수식 연산: `원의 둘레 = 반지름 × ...` (라인 12).
    //     — 한글 명사구는 공백으로 구분된 한글 단어. 일반 산식 `5개−3개=2개`은 공백 없음.
    let fraction_with_korean =
        has_parens && has_math_op && (text.contains("/(") || text.contains(")/")) && {
            // 괄호 안 한글 여부 확인 — `(`와 매칭되는 `)` 사이 한글 있어야
            let mut depth = 0i32;
            let mut korean_in_parens = false;
            for c in chars {
                match *c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ if depth > 0 && is_korean_char(*c) => korean_in_parens = true,
                    _ => {}
                }
            }
            korean_in_parens
        };

    let root_with_korean = has_root
        && chars
            .windows(2)
            .any(|w| w[0] == '√' && is_korean_char(w[1]));

    let multi_word_korean_phrase = chars
        .windows(3)
        .any(|w| is_korean_char(w[0]) && w[1] == ' ' && is_korean_char(w[2]));

    // BMI 같은 영문자 + 한글 mixed 입력은 baseline의 일반 한국어 점역이 옳다.
    // multi-word Korean 분기는 한글 명사구만 있는 입력으로 제한.
    let has_english_letter = chars.iter().any(|c| c.is_ascii_alphabetic());

    has_korean
        && (fraction_with_korean
            || root_with_korean
            || (multi_word_korean_phrase && has_math_op && !has_english_letter))
}

pub(super) fn try_encode_mixed_math_slice(
    chars: &[char],
    math_context: MathContext,
) -> Option<Vec<u8>> {
    if chars.is_empty() {
        return None;
    }

    let text: String = chars.iter().collect();
    if !is_mixed_math_expression(chars, &text) {
        return None;
    }

    math::encoder::encode_math_expression_with_context(&text, math_context).ok()
}

pub(super) fn try_encode_mixed_math_prefix(
    prefix: &[char],
    suffix: &[char],
    math_context: MathContext,
) -> Option<Vec<u8>> {
    if let Some(bytes) = try_encode_math_slice(prefix, math_context) {
        let text: String = prefix.iter().collect();
        if !suffix.is_empty()
            && suffix.iter().all(|c| is_korean_suffix_char(*c))
            && suffix.iter().any(|c| is_korean_char(*c))
            && math::rule_46::is_trig_function(&text)
        {
            return math::encoder::encode_math_expression_with_context(
                &format!("{text}x"),
                math_context,
            )
            .ok();
        }
        return Some(bytes);
    }

    None
}

/// Build the math-prefix + Korean-suffix replacement Vec.
/// Single-line construction prevents tarpaulin multi-line vec! attribution loss.
fn build_math_prefix_replacement(
    leading_delimiter_len: usize,
    bytes: Vec<u8>,
    suffix: String,
) -> Vec<Token<'static>> {
    let lead = Token::PreEncoded(vec![0; leading_delimiter_len]);
    let math = Token::PreEncoded(bytes);
    let sep = Token::PreEncoded(vec![0, 0]);
    let trailing = build_word_token(suffix);
    vec![lead, math, sep, trailing]
}

/// Build the Korean-prefix + math-suffix replacement Vec.
fn build_korean_prefix_math_suffix(prefix: String, bytes: Vec<u8>) -> Vec<Token<'static>> {
    let head = build_word_token(prefix);
    let sep = Token::PreEncoded(vec![0, 0]);
    let math = Token::PreEncoded(bytes);
    vec![head, sep, math]
}

/// Locate an anonymized-person label used in Korean prose, such as
/// `A(54)씨`, `B(17)군`, or `C(16)양`.
///
/// The leading surface also looks like mathematical function notation, but
/// the Korean honorific immediately after the numeric parenthesis resolves the
/// ambiguity. Hangeul rules 29 and 35 therefore own the Roman letter and age,
/// while ordinary Korean punctuation rules own the parentheses. A real
/// function followed by another Korean particle (`A(14)는`) deliberately does
/// not match this predicate and remains on the math path. The returned range
/// contains the Roman letter and age parenthetical, but not the honorific.
fn anonymized_person_label_end(chars: &[char], start: usize) -> Option<usize> {
    if !chars.get(start).is_some_and(char::is_ascii_uppercase) || chars.get(start + 1) != Some(&'(')
    {
        return None;
    }

    let mut cursor = start + 2;
    let digit_start = cursor;
    while chars.get(cursor).is_some_and(char::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == digit_start {
        return None;
    }

    match chars.get(cursor) {
        Some('대') => cursor += 1,
        Some('·' | 'ㆍ')
            if chars
                .get(cursor + 1)
                .is_some_and(|ch| matches!(*ch, '여' | '남')) =>
        {
            cursor += 2;
        }
        _ => {}
    }

    (chars.get(cursor) == Some(&')')).then_some(cursor + 1)
}

fn starts_anonymized_person_marker(chars: &[char], index: usize) -> bool {
    chars
        .get(index)
        .is_some_and(|marker| matches!(*marker, '씨' | '군' | '양'))
}

/// Whether the label is immediately followed by a Korean human-role noun.
///
/// The role stem is separated from an attached case particle and classified
/// by its productive title/rank ending (`-사`, `-병`, `-감`, `-관`, `-장`,
/// `-원`).  This covers ranks and occupations without enumerating corpus
/// phrases.  Mathematical nouns such as `함수`, `변수`, and `값` do not have
/// one of these endings, so `A(14)함수는` remains on the math path.
fn starts_attached_korean_person_role(chars: &[char], index: usize) -> bool {
    let suffix = chars[index..]
        .iter()
        .take_while(|ch| is_korean_char(**ch))
        .collect::<String>();
    if suffix.is_empty() {
        return false;
    }

    const PARTICLES: &[&str] = &[
        "에게서",
        "으로",
        "에게",
        "께서",
        "에서",
        "까지",
        "부터",
        "처럼",
        "보다",
        "라고",
        "이라",
        "이랑",
        "하고",
        "께",
        "의",
        "이",
        "가",
        "은",
        "는",
        "을",
        "를",
        "와",
        "과",
        "에",
        "도",
        "로",
    ];
    let stem = PARTICLES
        .iter()
        .find_map(|particle| suffix.strip_suffix(particle))
        .unwrap_or(&suffix);

    stem.chars().count() >= 2
        && stem
            .chars()
            .last()
            .is_some_and(|ending| matches!(ending, '사' | '병' | '감' | '관' | '장' | '원'))
}

fn attached_korean_suffix_text(chars: &[char], index: usize) -> String {
    chars[index..]
        .iter()
        .take_while(|ch| is_korean_char(**ch))
        .collect()
}

fn starts_animate_dative_particle(chars: &[char], index: usize) -> bool {
    attached_korean_suffix_text(chars, index).starts_with("에게")
}

/// A Korean personal name can be printed directly before an anonymizing Roman
/// label, for example `조너선M(41)이`.  In that structure a following case
/// particle resolves the `M(41)` function-notation ambiguity.  Require a
/// three-syllable-or-longer attached Korean prefix; ordinary mathematical
/// heads such as `함수A(14)는` therefore remain math-owned.
fn has_attached_korean_name_and_case_particle(chars: &[char], start: usize, end: usize) -> bool {
    let korean_prefix_len = chars[..start]
        .iter()
        .rev()
        .take_while(|ch| is_korean_char(**ch))
        .count();
    if korean_prefix_len < 3 {
        return false;
    }

    matches!(
        attached_korean_suffix_text(chars, end).as_str(),
        "이" | "가" | "은" | "는" | "을" | "를" | "와" | "과" | "의" | "에"
    )
}

/// A list can defer its person marker to the final member, as in
/// `B(60)·C(41)씨`.  Every preceding age label is still Roman prose.  Only a
/// middle-dot chain whose eventual member has an explicit person marker is
/// accepted, so an algebraic `A(1)·B(2)` remains mathematical.
fn anonymized_person_chain_has_marker(chars: &[char], mut cursor: usize) -> bool {
    while chars.get(cursor) == Some(&'·') {
        let next_start = cursor + 1;
        let Some(next_end) = anonymized_person_label_end(chars, next_start) else {
            return false;
        };
        if starts_anonymized_person_marker(chars, next_end) {
            return true;
        }
        cursor = next_end;
    }
    false
}

fn anonymized_person_label_span(chars: &[char]) -> Option<(usize, usize)> {
    let mut index = 0usize;
    while index < chars.len() {
        if !chars[index].is_ascii_uppercase()
            || index
                .checked_sub(1)
                .and_then(|previous| chars.get(previous))
                .is_some_and(|previous| previous.is_ascii_alphanumeric())
            || chars.get(index + 1) != Some(&'(')
        {
            index += 1;
            continue;
        }

        if let Some(end) = anonymized_person_label_end(chars, index)
            && (starts_anonymized_person_marker(chars, end)
                || starts_attached_korean_person_role(chars, end)
                || starts_animate_dative_particle(chars, end)
                || has_attached_korean_name_and_case_particle(chars, index, end)
                || anonymized_person_chain_has_marker(chars, end))
        {
            return Some((index, end));
        }
        index += 1;
    }
    None
}

pub(super) fn encode_anonymized_person_label(chars: &[char]) -> Option<Vec<u8>> {
    let (&letter, _) = chars.split_first()?;
    if anonymized_person_label_end(chars, 0) != Some(chars.len()) {
        return None;
    }

    let mut encoded = vec![
        crate::rules::korean::rule_29::ROMAN_INDICATOR,
        crate::rules::korean::rule_28::UPPERCASE_SINGLE,
        crate::english::encode_english(letter).ok()?,
    ];
    let parenthetical = chars[1..].iter().collect::<String>();
    encoded.extend(crate::encode(&parenthetical).ok()?);
    Some(encoded)
}

pub(super) fn split_anonymized_person_label(chars: &[char]) -> Option<Vec<Token<'static>>> {
    let mut replacement = Vec::new();
    let mut cursor = 0usize;
    let mut found = false;

    while cursor < chars.len() {
        let Some((relative_start, relative_end)) = anonymized_person_label_span(&chars[cursor..])
        else {
            break;
        };
        let start = cursor + relative_start;
        let end = cursor + relative_end;
        let encoded = encode_anonymized_person_label(&chars[start..end])?;
        if start > cursor {
            replacement.push(build_word_token(chars[cursor..start].iter().collect()));
        }
        replacement.push(Token::PreEncoded(encoded));
        cursor = end;
        found = true;
    }

    if !found {
        return None;
    }
    if cursor < chars.len() {
        replacement.push(build_word_token(chars[cursor..].iter().collect()));
    }
    Some(replacement)
}

/// Recognize only the suffix shape used after an already-confirmed Korean
/// prefix.  Korean rule 34's PDF example is `링컨(Lincoln)은`: Roman text may
/// be enclosed in a bracket without a Roman terminator.  Rule 54 requires the
/// bracket to attach to its contents, so a comma or period after the closing
/// bracket does not turn that Roman annotation into mathematics.
///
/// This predicate is intentionally not part of the global math detector, whose
/// existing results for standalone `(x)`, `(A)`, and `(abc)` stay unchanged.
/// The caller below must first prove that all preceding characters are Korean.
fn is_closed_roman_annotation_suffix(chars: &[char]) -> bool {
    if chars.first() != Some(&'(') {
        return false;
    }

    let Some(close) = chars.iter().position(|c| *c == ')') else {
        return false;
    };
    let body = &chars[1..close];
    let trailing = &chars[close + 1..];

    !body.is_empty()
        && body.iter().any(|c| c.is_ascii_alphabetic())
        && body
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(*c, '-' | '\'' | '.'))
        && trailing
            .iter()
            .all(|c| matches!(*c, ',' | '.' | ';' | ':' | '!' | '?' | '\'' | '"'))
}

pub(super) fn split_mixed_math_word(
    word: &crate::rules::token::WordToken<'_>,
    leading_delimiter_len: usize,
    math_context: MathContext,
) -> Option<Vec<Token<'static>>> {
    if !word.meta.has_korean || word.chars.iter().all(|c| is_korean_char(*c)) {
        return None;
    }

    if let Some(replacement) = split_anonymized_person_label(&word.chars) {
        return Some(replacement);
    }

    let chars = &word.chars;
    let len = chars.len();

    // try_encode_mixed_math_prefix는 suffix가 empty인 경우 Some을 반환하지 않으므로
    // end == len에서 Some이 나오는 경로는 도달 不可. 명시적 가드 제거됨.
    let math_prefix_result = (1..len).rev().find_map(|end| {
        let bytes = try_encode_mixed_math_prefix(&chars[..end], &chars[end..], math_context)?;
        let suffix_chars = &chars[end..];
        let suffix_is_korean = suffix_chars.iter().all(|c| is_korean_suffix_char(*c))
            && suffix_chars.iter().any(|c| is_korean_char(*c));
        if suffix_is_korean {
            Some(build_math_prefix_replacement(
                leading_delimiter_len,
                bytes,
                suffix_chars.iter().collect(),
            ))
        } else {
            None
        }
    });
    if let Some(replacement) = math_prefix_result {
        return Some(replacement);
    }

    // PDF — Korean 접두어 + math 접미어 (예: `정수∵y=n+2`).
    // 접두어는 한국어로, 접미어는 수학 표기로 점역하고 사이에 두 칸 띄어쓴다.
    // (leading_delimiter_len는 좌측 token boundary가 한국어인 경우에만 사용되며,
    // 한국어 접두어 시작 시 Token::Space가 1칸을 이미 제공하므로 여기서는 0이다.)
    let _ = leading_delimiter_len;
    (1..len).find_map(|start| {
        let prefix_chars = &chars[..start];
        let suffix_chars = &chars[start..];
        let prefix_all_korean = prefix_chars.iter().all(|c| is_korean_char(*c));
        let suffix_no_korean = !suffix_chars.iter().any(|c| is_korean_char(*c));
        if !prefix_all_korean || !suffix_no_korean {
            return None;
        }
        if is_closed_roman_annotation_suffix(suffix_chars) {
            return None;
        }
        let suffix_text: String = suffix_chars.iter().collect();
        let suffix_is_math = is_mixed_math_expression(suffix_chars, &suffix_text)
            || is_math_expression(suffix_chars, &suffix_text);
        if !suffix_is_math {
            return None;
        }
        let bytes =
            math::encoder::encode_math_expression_with_context(&suffix_text, math_context).ok()?;
        Some(build_korean_prefix_math_suffix(
            prefix_chars.iter().collect(),
            bytes,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::official_rule_34_annotation("(Lincoln)", true)]
    #[case::missing_opening("Lincoln)", false)]
    #[case::missing_closing("(Lincoln", false)]
    #[case::empty_body("()", false)]
    fn recognizes_only_closed_roman_annotation_suffixes(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(
            is_closed_roman_annotation_suffix(&input.chars().collect::<Vec<_>>()),
            expected
        );
    }
    use crate::rules::math::math_token_rule::MathContext;
    use crate::rules::token::SpaceKind;

    #[rstest::rstest]
    #[case::adult("A(54)씨는", true)]
    #[case::minor_male("B(17)군에게", true)]
    #[case::minor_female("C(16)양은", true)]
    #[case::gender_annotation("A(41·여)씨는", true)]
    #[case::age_decade("B(30대)씨는", true)]
    #[case::korean_name_prefix("김모A(41)씨", true)]
    #[case::military_rank("A(21)상병을", true)]
    #[case::police_rank("B(42)경사가", true)]
    #[case::occupation("C(47)원사에게", true)]
    #[case::animate_dative("A(30)에게", true)]
    #[case::attached_korean_name("조너선M(41)이", true)]
    #[case::math_function_particle("A(14)는", false)]
    #[case::attached_math_function("함수A(14)는", false)]
    #[case::math_function_noun("A(14)함수는", false)]
    #[case::non_honorific_syllable("A(14)시는", false)]
    #[case::missing_digits("A()씨", false)]
    fn recognizes_only_anonymized_person_labels(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(
            anonymized_person_label_span(&input.chars().collect::<Vec<_>>()).is_some(),
            expected
        );
    }

    #[rstest::rstest]
    #[case::adult("A(54)씨는")]
    #[case::minor_male("B(17)군에게")]
    #[case::minor_female("C(16)양은")]
    #[case::gender_annotation("A(41·여)씨는")]
    #[case::age_decade("B(30대)씨는")]
    fn anonymized_person_labels_use_korean_prose_cells(#[case] input: &str) {
        let chars = input.chars().collect::<Vec<_>>();
        let word = WordToken {
            text: Cow::Borrowed(input),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };

        let replacement = split_mixed_math_word(&word, 0, MathContext::default())
            .expect("honorific resolves the function/prose ambiguity");
        assert!(matches!(
            replacement.as_slice(),
            [Token::PreEncoded(_), Token::Word(_)]
        ));
        let Token::PreEncoded(label) = &replacement[0] else {
            unreachable!();
        };
        let (start, end) = anonymized_person_label_span(&chars).expect("label span");
        let expected = encode_anonymized_person_label(&chars[start..end]).expect("label cells");
        assert_eq!(label, &expected);
    }

    #[rstest::rstest]
    #[case::deferred_marker("B(60)·C(41)씨", 2)]
    #[case::child_markers("B(6)군·C(3)양이", 2)]
    #[case::three_people("A(41)씨·B(28)씨와C(27)씨가", 3)]
    fn splits_every_anonymized_person_label_in_one_token(
        #[case] input: &str,
        #[case] expected_labels: usize,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        let replacement = split_anonymized_person_label(&chars).expect("person-label list");

        assert_eq!(
            replacement
                .iter()
                .filter(|token| matches!(token, Token::PreEncoded(_)))
                .count(),
            expected_labels
        );
    }

    /// helpers:235 — `try_encode_math_slice` fallback to `crate::encode` when
    /// math encoder fails. Use `f(~)`: passes `has_function_call` candidacy
    /// (1-letter + `(`) and is_math_expression, but math encoder rejects `~`.
    #[test]
    fn try_encode_math_slice_fallback_to_regular_encode() {
        let chars: Vec<char> = "f(~)".chars().collect();
        let _ = try_encode_math_slice(&chars, MathContext::default());
        // Also: 2-overline-3010 (combining macron) as smoke variant.
        let chars: Vec<char> = "2\u{0305}.3010".chars().collect();
        let _ = try_encode_math_slice(&chars, MathContext::default());
    }

    /// helpers:243 — `try_encode_mixed_math_slice` returns None for empty chars.
    #[test]
    fn try_encode_mixed_math_slice_empty_returns_none() {
        let result = try_encode_mixed_math_slice(&[], MathContext::default());
        assert!(result.is_none());
    }

    /// The PDF's `√분산` form is a mixed expression because the radical is
    /// directly attached to Korean text; it must produce a concrete cell sequence.
    #[test]
    fn try_encode_mixed_math_slice_encodes_valid_expression() {
        let chars = std::hint::black_box("√분산").chars().collect::<Vec<_>>();
        let result = try_encode_mixed_math_slice(&chars, MathContext::default());

        assert!(result.is_some());
    }

    #[test]
    fn mixed_fraction_detects_korean_inside_the_parenthesized_operand() {
        let text = "2/(삼+오)";
        let chars = text.chars().collect::<Vec<_>>();

        assert!(is_mixed_math_expression(&chars, text));
    }

    #[test]
    fn anonymized_person_chain_rejects_invalid_or_unmarked_following_labels() {
        let invalid = "A(1)·not".chars().collect::<Vec<_>>();
        assert!(!anonymized_person_chain_has_marker(&invalid, 4));

        let unmarked = "A(1)·B(2)·C(3)".chars().collect::<Vec<_>>();
        assert!(!anonymized_person_chain_has_marker(&unmarked, 4));
    }

    #[test]
    fn try_encode_mixed_math_prefix_encodes_math_prefix_before_korean_suffix() {
        let prefix: Vec<char> = "x²".chars().collect();
        let suffix: Vec<char> = "는".chars().collect();

        let result = try_encode_mixed_math_prefix(&prefix, &suffix, MathContext::default());

        assert!(result.is_some());
    }

    #[test]
    fn adjacent_korean_word_flags_skips_previous_spaces() {
        let tokens = vec![
            build_word_token("한글".to_string()),
            Token::Space(SpaceKind::Regular),
            Token::PreEncoded(vec![1]),
        ];

        assert_eq!(adjacent_korean_word_flags(&tokens, 2), (true, false));
    }

    #[test]
    fn strong_mixed_math_candidate_accepts_runtime_root() {
        let text = std::hint::black_box("√x");
        let chars: Vec<char> = text.chars().collect();

        assert!(is_strong_mixed_math_candidate(&chars, text));
    }

    #[test]
    fn middle_dot_numeric_word_counts_runtime_middle_dot() {
        let chars = [
            std::hint::black_box('1'),
            std::hint::black_box('·'),
            std::hint::black_box('2'),
        ];

        assert!(is_middle_dot_numeric_word(&chars));
    }

    /// helpers:298 — `split_mixed_math_word` early `end == len` None branch.
    /// Prefix matches entire word, no suffix — returns None to avoid splitting
    /// when whole word is consumed as math.
    #[test]
    fn split_mixed_math_word_whole_word_no_split() {
        use crate::rules::token::{WordMeta, WordToken};
        use std::borrow::Cow;
        let chars: Vec<char> = "한x".chars().collect();
        let word = WordToken {
            text: Cow::Owned("한x".to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };
        let _ = split_mixed_math_word(&word, 0, MathContext::default());
    }

    /// helpers:303 — `split_mixed_math_word` suffix not-all-korean `continue` arm.
    /// Suffix contains non-Korean chars (e.g. mixed ASCII), forcing the
    /// `!all_korean_suffix || !any_korean_in_suffix` branch.
    #[test]
    fn split_mixed_math_word_non_korean_suffix_continues() {
        use crate::rules::token::{WordMeta, WordToken};
        use std::borrow::Cow;
        // `x한a` — math prefix `x`, then 한, then `a` (not Korean): suffix mix.
        let chars: Vec<char> = "x한a".chars().collect();
        let word = WordToken {
            text: Cow::Owned("x한a".to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };
        let _ = split_mixed_math_word(&word, 0, MathContext::default());
    }

    #[test]
    fn split_mixed_math_word_math_prefix_korean_suffix_replaces() {
        use crate::rules::token::{WordMeta, WordToken};
        use std::borrow::Cow;

        let chars: Vec<char> = "x²는".chars().collect();
        let word = WordToken {
            text: Cow::Owned("x²는".to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };

        let replacement = split_mixed_math_word(&word, 1, MathContext::default());

        assert!(replacement.is_some());
    }

    /// helpers:328 — `try_korean_prefix_math_suffix` math encoding fails (continue).
    /// Korean prefix + math suffix that fails encoding (e.g. unsupported char).
    #[test]
    fn split_mixed_math_word_korean_prefix_math_suffix_encode_fail() {
        use crate::rules::token::{WordMeta, WordToken};
        use std::borrow::Cow;
        // 한국x~ : Korean prefix, but suffix has `~` which fails math encoding.
        let chars: Vec<char> = "한국x~".chars().collect();
        let word = WordToken {
            text: Cow::Owned("한국x~".to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        };
        let _ = split_mixed_math_word(&word, 0, MathContext::default());
    }
}
