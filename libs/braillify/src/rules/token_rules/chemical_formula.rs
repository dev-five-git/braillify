//! 과학 제1~8·18항 — 화학식과 화학 반응식을 문장 안에서 적는다.
//!
//! 식 자체의 점형은 [`crate::rules::science::formula`] 가 정하고, 이 규칙은 식이
//! 국어 문장과 만나는 자리를 다룬다.
//!
//! - 식 하나(`H₂O와`)는 로마자표 ⠴ 를 앞세우고(한글 제29항), 끝이 숫자 첨자면
//!   로마자 종료표를 적지 않는다(과학 제7항 6, 한글 제68항). 대문자 구절로 끝나면
//!   대문자 종료표 뒤에 로마자 종료표를 적는다(제7항 6 다만).
//! - 이온 표시 뒤에는 로마자 종료표 없이 한 칸 띄운다(제2항 [붙임]).
//! - 연산·비교 기호나 화살표가 든 식은 앞뒤를 두 칸씩 띄우고 로마자표를 적지
//!   않는다(제6항).

use std::borrow::Cow;

use crate::rules::context::EncoderState;
use crate::rules::korean::rule_44::is_number_confusable_korean_char;
use crate::rules::science::formula::{self, Item};
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;

pub struct ChemicalFormulaRule;

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "7",
    subsection: None,
    name: "science_chemical_formula",
    standard_ref: "2024 Korean Braille Standard, 과학 제1~8·18항",
    description: "화학식과 화학 반응식",
};

/// 식이 시작될 수 있는 글자.
fn may_start(ch: char) -> bool {
    ch.is_ascii_uppercase()
        || ch.is_ascii_digit()
        || matches!(ch, '(' | '[' | '$' | '\u{2080}'..='\u{209C}' | '⁰'..='⁹' | '¹' | '²' | '³')
        || crate::rules::english_ueb::rule_9::decode_styled(ch).is_some()
}

fn is_hangul(ch: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&ch)
}

fn word<'a>(text: String) -> Token<'a> {
    let chars: Vec<char> = text.chars().collect();
    Token::Word(WordToken {
        meta: WordMeta::from_chars(&chars),
        chars,
        text: Cow::Owned(text),
    })
}

/// 첫 낱말의 머리. 한글에 붙은 여는 괄호는 한글 괄호라 머리에 넣는다(`분압(`).
fn prefix_len(chars: &[char]) -> Option<usize> {
    let start = chars.iter().position(|c| may_start(*c))?;
    if !chars[..start].iter().copied().any(is_hangul) {
        return Some(0);
    }
    (chars[start] == '(').then_some(start + 1)
}

/// 식 뒤에 붙은 꼬리로 받을 수 있는가 — 한글, 닫는 괄호, 문장 부호.
fn is_tail(tail: &[char]) -> bool {
    tail.first()
        .is_none_or(|c| is_hangul(*c) || matches!(c, ')' | '.' | ',' | '?' | '!'))
}

/// 식을 이룬 뒤 꼬리 앞에 적을 로마자 종료표와 빈칸.
fn closing(items: &[Item], tail: &[char], followed: bool) -> Vec<u8> {
    let blank = decode_unicode('⠀');
    let terminator = decode_unicode('⠲');
    // 닫는 괄호·마침표 앞에서는 로마자 종료표를 적지 않는다(한글 제33·35항).
    if tail
        .first()
        .is_some_and(|c| *c == ')' || crate::english_logic::should_skip_terminator_for_symbol(*c))
    {
        return Vec::new();
    }
    if formula::ends_in_phrase(items) {
        return vec![terminator];
    }
    let attached_korean = tail.first().copied().filter(|c| is_hangul(*c));
    match items.last() {
        Some(Item::Sup(_)) => attached_korean.map(|_| vec![blank]).unwrap_or_default(),
        Some(Item::Sub(content)) if content.ends_with(|c: char| c.is_ascii_digit()) => {
            attached_korean
                .filter(|c| is_number_confusable_korean_char(*c))
                .map(|_| vec![blank])
                .unwrap_or_default()
        }
        _ if followed || !tail.is_empty() => vec![terminator],
        _ => Vec::new(),
    }
}

/// 식을 이루는 토큰 범위와 그 점형.
struct Span {
    tokens: usize,
    prefix: String,
    cells: Vec<u8>,
    tail: String,
}

fn read(items: Vec<Item>, korean: bool, science: bool) -> Option<Vec<Item>> {
    if formula::is_formula(&items, korean) {
        return Some(items);
    }
    science.then(|| formula::science_reading(&items)).flatten()
}

/// 과학 제30항 — 단위는 「한글 점자」 제69항에 따라 로마자표를 앞세운다.
fn sits_in_korean(items: &[Item], korean: bool, science: bool) -> bool {
    korean || science && matches!(items.first(), Some(Item::Unit(_)))
}

fn span_at(
    tokens: &[Token<'_>],
    index: usize,
    first: &WordToken<'_>,
    korean: bool,
    science: bool,
) -> Option<Span> {
    let head = prefix_len(&first.chars)?;
    let run = tokens[index..]
        .iter()
        .take_while(|token| matches!(token, Token::Word(_) | Token::Space(_)))
        .count();
    for count in (1..=run).rev() {
        let Some(Token::Word(last)) = tokens.get(index + count - 1) else {
            continue;
        };
        let mut body: Vec<char> = Vec::new();
        for token in &tokens[index..index + count - 1] {
            match token {
                Token::Word(w) => body.extend(&w.chars),
                _ => body.push(' '),
            }
        }
        let lead = body.len();
        body.extend(&last.chars);
        let body = &body[head..];
        // 여러 낱말이면 마지막 낱말에서 한 글자라도 식에 들어야 한다.
        let shortest = if count == 1 { 1 } else { lead - head + 1 };
        for end in (shortest..=body.len()).rev() {
            let tail = &body[end..];
            if !is_tail(tail) {
                continue;
            }
            let text: String = body[..end].iter().collect();
            let Some(items) = formula::parse(&text).and_then(|items| read(items, korean, science))
            else {
                continue;
            };
            if head > 0 && formula::stands_apart(&items) {
                continue;
            }
            let (preceded, followed) = neighbours(tokens, index, count);
            // 원소 기호 목록(과학 제1항)은 목록이 끝나는 자리까지 통째로 받는다.
            // `K, Ca Mg` 처럼 로마자가 뒤로 이어지면 목록이 아니다.
            if formula::is_element_list(&items)
                && tail.is_empty()
                && matches!(tokens.get(index + count + 1), Some(Token::Word(next))
                    if next.chars.first().is_some_and(char::is_ascii_alphabetic))
            {
                continue;
            }
            let korean = sits_in_korean(&items, korean, science);
            let cells = layout(&items, tail, korean, preceded, followed).ok()?;
            return Some(Span {
                tokens: count,
                prefix: first.chars[..head].iter().collect(),
                cells,
                tail: tail.iter().collect(),
            });
        }
    }
    None
}

/// 국어 문장 안에서의 앞뒤 표지와 빈칸을 붙인다.
fn layout(
    items: &[Item],
    tail: &[char],
    korean: bool,
    preceded: bool,
    followed: bool,
) -> Result<Vec<u8>, String> {
    let body = formula::encode(items)?;
    if !korean {
        return Ok(body);
    }
    let blank = decode_unicode('⠀');
    if formula::stands_apart(items) {
        let mut out = Vec::new();
        if preceded {
            out.push(blank);
        }
        out.extend(body);
        if !tail.is_empty() && tail.first().copied().is_some_and(is_hangul) {
            out.extend([blank, blank]);
        } else if followed {
            out.push(blank);
        }
        return Ok(out);
    }
    let mut out = vec![decode_unicode('⠴')];
    out.extend(body);
    out.extend(closing(items, tail, followed));
    Ok(out)
}

/// 식 앞에 빈칸이 있는가, 식 뒤에 빈칸을 두고 낱말이 이어지는가.
fn neighbours(tokens: &[Token<'_>], index: usize, count: usize) -> (bool, bool) {
    let preceded = index > 0 && matches!(tokens.get(index - 1), Some(Token::Space(_)));
    let followed = matches!(tokens.get(index + count), Some(Token::Space(_)))
        && matches!(tokens.get(index + count + 1), Some(Token::Word(_)));
    (preceded, followed)
}

/// 낱말 하나가 통째로 전자 배치(제19항)나 유전자형(제23항)이거나, 조사·문장
/// 부호가 붙은 LaTeX 화학식(`$H_{2}PO_{4}^{-}$가`)인 경우.
fn whole_word<'a>(
    tokens: &[Token<'a>],
    index: usize,
    first: &WordToken<'_>,
    korean: bool,
    science: bool,
) -> Option<TokenAction<'a>> {
    let whole = formula::configuration(&first.text)
        .or_else(|| crate::rules::science::genotype::encode_genotype(&first.text));
    if let Some(cells) = whole {
        return Some(TokenAction::Replace(Token::PreEncoded(cells)));
    }
    let body = first.text.strip_prefix('$')?;
    let (latex, tail) = body.split_at(body.find('$')?);
    let tail: Vec<char> = tail.chars().skip(1).collect();
    if !is_tail(&tail) {
        return None;
    }
    let items = read(formula::parse_latex(latex)?, korean, science)?;
    let (preceded, followed) = neighbours(tokens, index, 1);
    let korean = sits_in_korean(&items, korean, science);
    let cells = layout(&items, &tail, korean, preceded, followed).ok()?;
    let mut replacement = vec![Token::PreEncoded(cells)];
    if !tail.is_empty() {
        replacement.push(word(tail.iter().collect()));
    }
    Some(TokenAction::ReplaceMany(replacement))
}

impl TokenRule for ChemicalFormulaRule {
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &META
    }

    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        1
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(first)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        if !state.reads_science_shapes()
            || (!first.chars.iter().any(|c| c.is_ascii_alphabetic())
                && !first.text.starts_with('$'))
        {
            return Ok(TokenAction::Noop);
        }
        let korean = state.english_indicator || state.korean_context_active;
        let science = state.science_context_active;
        if let Some(action) = whole_word(tokens, index, first, korean, science) {
            return Ok(action);
        }
        let Some(span) = span_at(tokens, index, first, korean, science) else {
            return Ok(TokenAction::Noop);
        };
        let mut replacement = Vec::new();
        if !span.prefix.is_empty() {
            replacement.push(word(span.prefix));
        }
        replacement.push(Token::PreEncoded(span.cells));
        if !span.tail.is_empty() {
            replacement.push(word(span.tail));
        }
        Ok(TokenAction::ReplaceRange(span.tokens, replacement))
    }
}

#[cfg(test)]
mod tests {
    fn braille(text: &str) -> String {
        crate::encode_to_unicode(text).expect("encodes")
    }

    #[rstest::rstest]
    #[case::letter_end_at_sentence_end("물은 H₂O", "⠑⠯⠵⠀⠴⠠⠓⠰⠼⠃⠠⠕")]
    #[case::ion_before_a_space("H⁺ 수는", "⠴⠠⠓⠘⠢⠀⠠⠍⠉⠵")]
    #[case::expression_before_a_word("식 C + O₂ → CO₂ 이다", "⠠⠕⠁⠀⠀⠠⠠⠠⠉⠀⠢⠀⠕⠰⠼⠃⠀⠒⠕⠀⠉⠕⠰⠼⠃⠠⠄⠀⠀⠕⠊")]
    #[case::expression_in_korean_parentheses("값(H₂ + O₂)은", "⠫⠃⠄⠦⠄⠴⠠⠓⠰⠼⠃⠀⠢⠀⠴⠠⠕⠰⠼⠃⠠⠴⠵")]
    #[case::hangul_glued_before_a_formula("물H₂O", "⠑⠯⠀⠀⠠⠓⠰⠼⠃⠠⠕")]
    #[case::roman_tail("값 H₂Ox", "⠫⠃⠄⠀⠀⠠⠓⠰⠼⠃⠠⠕⠭")]
    #[case::letter_end("H₂O와 물", "⠴⠠⠓⠰⠼⠃⠠⠕⠲⠧⠀⠑⠯")]
    #[case::element_list("Li, Na, K는 알칼리", "⠴⠠⠇⠊⠂⠀⠠⠝⠁⠂⠀⠠⠅⠲⠉⠵⠀⠣⠂⠋⠂⠐⠕")]
    #[case::list_that_runs_on("양이온(K, Ca Mg), 전기", "⠜⠶⠕⠷⠦⠄⠴⠠⠅⠂⠀⠠⠉⠁⠀⠠⠍⠛⠠⠴⠐⠀⠨⠾⠈⠕")]
    #[case::subscript_before_confusable("H₂는 수소", "⠴⠠⠓⠰⠼⠃⠀⠉⠵⠀⠠⠍⠠⠥")]
    #[case::subscript_before_plain("O₂이다.", "⠴⠠⠕⠰⠼⠃⠕⠊⠲")]
    #[case::ion("H⁺가 된다.", "⠴⠠⠓⠘⠢⠀⠫⠀⠊⠽⠒⠊⠲")]
    #[case::closed_phrase("C₆H₁₂O₆이다.", "⠴⠠⠠⠠⠉⠰⠼⠋⠐⠓⠰⠼⠁⠃⠕⠰⠼⠋⠠⠄⠲⠕⠊⠲")]
    #[case::korean_parenthesis("분압(PO₂)과", "⠘⠛⠣⠃⠦⠄⠴⠠⠏⠠⠕⠰⠼⠃⠠⠴⠈⠧")]
    #[case::expression("식은 C + O₂ → CO₂이다.", "⠠⠕⠁⠵⠀⠀⠠⠠⠠⠉⠀⠢⠀⠕⠰⠼⠃⠀⠒⠕⠀⠉⠕⠰⠼⠃⠠⠄⠀⠀⠕⠊⠲")]
    #[case::latex_formula_with_a_particle(
        "수용액의 $H_{2}PO_{4}^{-}$가 있다",
        "⠠⠍⠬⠶⠗⠁⠺⠀⠴⠠⠠⠠⠓⠰⠼⠃⠏⠕⠰⠼⠙⠘⠔⠠⠄⠲⠫⠀⠕⠌⠊"
    )]
    fn places_formulas_in_korean_sentences(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(braille(text), expected);
    }

    #[rstest::rstest]
    #[case::configuration("1s²2s²2p⁶", "⠼⠁⠎⠘⠼⠃⠼⠃⠎⠘⠼⠃⠼⠃⠏⠘⠼⠋")]
    #[case::genotype("RRyy", "⠠⠠⠗⠗⠠⠄⠽⠽")]
    #[case::latex_formula(
        "$K_1=\\frac{[H_3O^+][HCO_3^-]}{[H_2CO_3]}$",
        "⠠⠅⠰⠼⠁⠒⠒⠷⠄⠠⠠⠠⠓⠰⠼⠃⠐⠉⠕⠰⠼⠉⠠⠾⠌⠷⠷⠄⠓⠰⠼⠉⠕⠘⠢⠠⠾⠷⠄⠓⠉⠕⠰⠼⠉⠘⠔⠠⠄⠠⠾⠾"
    )]
    #[case::unit_glyphs_without_korean("㎜Hg", "⠴⠍⠍⠠⠓⠛⠲")]
    #[case::latex_spaces_do_not_print("$Ca Cl_{2}$", "⠠⠉⠁⠠⠉⠇⠰⠼⠃")]
    fn writes_whole_scientific_words(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            crate::encode_to_unicode_in_context(text, "science"),
            Ok(expected.to_string())
        );
    }

    #[test]
    fn leaves_a_latex_formula_with_a_roman_tail_to_english() {
        assert_eq!(braille("$H_{2}O$x"), "⠠⠓⠰⠢⠼⠃⠠⠕⠭");
    }

    #[rstest::rstest]
    #[case::latex_variable("$A_1$", true)]
    #[case::formula_outside_korean_and_science_text("H₂O", false)]
    fn leaves_the_word_to_other_rules(#[case] text: &str, #[case] korean_text: bool) {
        let tokens = vec![super::word(text.into())];
        let mut state = crate::rules::context::EncoderState::new(korean_text);
        let action = super::TokenRule::apply(&super::ChemicalFormulaRule, &tokens, 0, &mut state)
            .expect("applies");
        assert!(matches!(action, super::TokenAction::Noop));
    }

    #[test]
    fn attributes_the_roman_markers_it_wraps_around_a_unit() {
        let options = crate::EncodeOptions {
            default_mode: Some(crate::rules::context::EncodingMode::Science),
        };
        let (cells, trace) =
            crate::encode_with_options_and_trace("㎜Hg", &options).expect("encodes");
        assert_eq!(cells.len() as u32, trace.output_len());
        assert_eq!(trace.unattributed_cells(), 0);
    }
}
