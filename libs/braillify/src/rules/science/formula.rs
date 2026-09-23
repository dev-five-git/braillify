//! 과학 제1~8·18항 — 화학식과 화학 반응식.
//!
//! 원소 기호는 모두 1급 점자로 적고(제4항 1) 원소 기호마다 대문자표를 붙인다
//! (제7항 1). 로마자 하나로 된 원소 기호가 셋 이상 이어지면 대문자 구절표
//! ⠠⠠⠠ 로 묶고, 마지막 한 글자 원소 기호와 그 첨자 뒤에 대문자 종료표 ⠠⠄ 를
//! 적는다(제4항, [붙임 1]). 구절 안에서 숫자 뒤에 붙는 H·B·C·F·I 앞에는 ⠐ 을
//! 적는다(제5항).

use crate::english::encode_english;
use crate::number::encode_number;
use crate::rules::english_ueb::rule_9::decode_styled;
use crate::rules::english_ueb::token::Typeform;
use crate::unicode::decode_unicode;

use super::elements::{is_single_letter_element, is_two_letter_element};

/// 상태 기호. 과학 제18항 3 — 한글 소괄호로 묶는다.
const STATES: &[&str] = &["s", "l", "g", "aq"];

/// 과학 제7항 5 — 화학식 안에서 강조된 문자는 통일영어점자 §9 에 따른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Style {
    Italic,
    Bold,
    Underline,
}

impl Style {
    fn lead(self) -> u8 {
        decode_unicode(match self {
            Style::Italic => '⠨',
            Style::Bold => '⠘',
            Style::Underline => '⠸',
        })
    }
}

/// 식을 이루는 낱낱.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// 원소 기호, 또는 원소가 아닌 로마자 대문자(`R`, `NAD` 의 `A`·`D`).
    Capital {
        symbol: String,
        element: bool,
        style: Option<Style>,
    },
    /// 아래 첨자. 제7항 3 — ⠰ 뒤에 첨자 내용.
    Sub(String),
    /// 위 첨자. 제2항 — ⠘ 뒤에 수, `+` 는 ⠢, `-` 는 ⠔.
    Sup(String),
    /// 계수나 식 속의 수.
    Number(String),
    Open(char),
    Close(char),
    /// 상태 기호 `(s)`·`(aq)` — 제18항 3.
    State(String),
    /// 괄호 안 한글 설명 `(피루브산)` — 제18항 4.
    Gloss(String),
    /// 식 사이에 든 한글 낱말 — 제18항 4, 한글표로 묶는다.
    Hangul(String),
    /// 가운뎃점 `·`.
    Dot,
    /// 비율의 쌍점 `:`.
    Ratio,
    Equals,
    Comma,
    /// 반응·비교 기호 `+ → ← ⇄ > <`.
    Op(char),
    /// 기체 발생·침전 `↑ ↓` — 제18항 2.
    Arrow(char),
    /// 분수의 묶음 괄호 — 분수는 분모를 먼저 적는다.
    Group(bool),
    FractionBar,
    Space,
    /// 단위 앞머리의 로마자 소문자(`mH₂O` 의 `m`) — 과학 제30항 [붙임].
    Unit(String),
    /// 전자 점식의 전자 — 제16항 2, 전자 하나를 ⠔ 로 적는다.
    Electrons(usize),
    /// 염기 서열 끝의 프라임 `′` — 제24항.
    Prime,
    /// 줄표 `―` — 제24항.
    Dash,
    /// 줄임표 `…` — 제24항.
    Ellipsis,
    /// 결합선 — 제10항 2, ⠰ 뒤에 결합 수 1·2·3.
    Bond(u8),
    /// 측쇄의 방향 표지 — 제10항 3·제16항 3·4. 적을 점형을 담는다.
    Branch(char),
}

impl Item {
    fn single_capital(&self) -> Option<bool> {
        match self {
            Item::Capital {
                symbol, element, ..
            } if symbol.len() == 1 => Some(*element),
            _ => None,
        }
    }

    fn is_script(&self) -> bool {
        matches!(self, Item::Sub(_) | Item::Sup(_))
    }

    /// 항(項)과 항을 가르는 자리.
    fn separates_terms(&self) -> bool {
        matches!(
            self,
            Item::Space
                | Item::Op(_)
                | Item::Equals
                | Item::Ratio
                | Item::Comma
                | Item::Group(_)
                | Item::FractionBar
                | Item::Dash
                | Item::Ellipsis
        )
    }
}

#[derive(Clone, Copy)]
struct Glyph {
    ch: char,
    style: Option<Style>,
}

/// 문자와 그 강조를 짝지어 푼다. 굵은·기울인 로마자는 수학 영숫자 기호로,
/// 밑줄은 결합 밑줄(U+0332)로 들어온다.
fn glyphs(text: &str) -> Option<Vec<Glyph>> {
    let mut out: Vec<Glyph> = Vec::new();
    for ch in text.chars() {
        if ch == '\u{0332}' {
            out.last_mut()?.style = Some(Style::Underline);
            continue;
        }
        let glyph = match decode_styled(ch) {
            Some((base, Typeform::Italic)) => Glyph {
                ch: base,
                style: Some(Style::Italic),
            },
            Some((base, Typeform::Bold)) => Glyph {
                ch: base,
                style: Some(Style::Bold),
            },
            Some(_) => return None,
            None => Glyph { ch, style: None },
        };
        out.push(glyph);
    }
    Some(out)
}

const SUBSCRIPTS: [(char, char); 22] = [
    ('₀', '0'),
    ('₁', '1'),
    ('₂', '2'),
    ('₃', '3'),
    ('₄', '4'),
    ('₅', '5'),
    ('₆', '6'),
    ('₇', '7'),
    ('₈', '8'),
    ('₉', '9'),
    ('ₐ', 'a'),
    ('ₑ', 'e'),
    ('ₒ', 'o'),
    ('ₓ', 'x'),
    ('ₕ', 'h'),
    ('ₖ', 'k'),
    ('ₗ', 'l'),
    ('ₘ', 'm'),
    ('ₙ', 'n'),
    ('ₚ', 'p'),
    ('ₛ', 's'),
    ('ₜ', 't'),
];

const SUPERSCRIPTS: [(char, char); 14] = [
    ('⁰', '0'),
    ('¹', '1'),
    ('²', '2'),
    ('³', '3'),
    ('⁴', '4'),
    ('⁵', '5'),
    ('⁶', '6'),
    ('⁷', '7'),
    ('⁸', '8'),
    ('⁹', '9'),
    ('⁺', '+'),
    ('⁻', '-'),
    ('ᵐ', 'm'),
    ('ⁿ', 'n'),
];

fn lookup(table: &[(char, char)], ch: char) -> Option<char> {
    table
        .iter()
        .find(|(key, _)| *key == ch)
        .map(|(_, value)| *value)
}

fn reverse(table: &[(char, char)], ch: char) -> Option<char> {
    table
        .iter()
        .find(|(_, value)| *value == ch)
        .map(|(key, _)| *key)
}

fn subscript(ch: char) -> Option<char> {
    lookup(&SUBSCRIPTS, ch)
}

fn superscript(ch: char) -> Option<char> {
    lookup(&SUPERSCRIPTS, ch)
}

fn is_hangul(ch: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&ch)
}

/// 괄호 안이 상태 기호나 한글 설명이면 그 낱낱과 소비한 글자 수를 돌려준다.
fn parenthesised(glyphs: &[Glyph]) -> Option<(Item, usize)> {
    let close = glyphs.iter().position(|g| g.ch == ')')?;
    let inner: String = glyphs[1..close].iter().map(|g| g.ch).collect();
    if STATES.contains(&inner.as_str()) {
        return Some((Item::State(inner), close + 1));
    }
    if !inner.is_empty() && inner.chars().all(is_hangul) {
        return Some((Item::Gloss(inner), close + 1));
    }
    None
}

fn collect(glyphs: &[Glyph], at: &mut usize, map: fn(char) -> Option<char>) -> String {
    let mut run = String::new();
    while let Some(ch) = glyphs.get(*at).and_then(|g| map(g.ch)) {
        run.push(ch);
        *at += 1;
    }
    run
}

/// 글자를 낱낱으로 가른다. 식에 쓰이지 않는 글자가 있으면 `None`.
pub(crate) fn parse(text: &str) -> Option<Vec<Item>> {
    let glyphs = glyphs(text)?;
    let mut items = Vec::new();
    let mut at = 0;
    while let Some(&Glyph { ch, style }) = glyphs.get(at) {
        if ch.is_ascii_uppercase() {
            let lower = glyphs
                .get(at + 1)
                .filter(|g| g.style == style && is_two_letter_element(ch, g.ch));
            let symbol = match lower {
                Some(g) => format!("{ch}{}", g.ch),
                None => ch.to_string(),
            };
            at += symbol.len();
            let element = symbol.len() == 2 || is_single_letter_element(ch);
            items.push(Item::Capital {
                symbol,
                element,
                style,
            });
            continue;
        }
        if subscript(ch).is_some() {
            items.push(Item::Sub(collect(&glyphs, &mut at, subscript)));
            continue;
        }
        if superscript(ch).is_some() {
            items.push(Item::Sup(collect(&glyphs, &mut at, superscript)));
            continue;
        }
        if ch.is_ascii_digit() {
            let digits = collect(&glyphs, &mut at, |c| c.is_ascii_digit().then_some(c));
            items.push(Item::Number(digits));
            continue;
        }
        if is_hangul(ch) {
            let word = collect(&glyphs, &mut at, |c| is_hangul(c).then_some(c));
            items.push(Item::Hangul(word));
            continue;
        }
        if ch.is_ascii_lowercase() && items.last().is_none_or(Item::separates_terms) {
            let letters = collect(&glyphs, &mut at, |c| c.is_ascii_lowercase().then_some(c));
            if !glyphs.get(at).is_some_and(|g| g.ch.is_ascii_uppercase()) {
                return None;
            }
            items.push(Item::Unit(letters));
            continue;
        }
        if ch == '('
            && let Some((item, len)) = parenthesised(&glyphs[at..])
        {
            items.push(item);
            at += len;
            continue;
        }
        items.push(match ch {
            ' ' => Item::Space,
            '(' | '[' => Item::Open(ch),
            ')' | ']' => Item::Close(ch),
            '·' => Item::Dot,
            ':' => Item::Ratio,
            '=' => Item::Equals,
            ',' => Item::Comma,
            '+' | '>' | '<' | '→' | '←' | '⇄' | '⇌' => Item::Op(ch),
            '↑' | '↓' => Item::Arrow(ch),
            '⋮' => Item::Electrons(3),
            '′' => Item::Prime,
            '―' => Item::Dash,
            '…' => Item::Ellipsis,
            _ => return None,
        });
        at += 1;
    }
    Some(reorder_prescripts(electron_pairs(items)))
}

/// 전자 점식에서는 쌍점이 비율이 아니라 전자 두 개다(`:N⋮⋮N:`). 약어 뒤의
/// 쌍점(`ATM:`)과 가르려면 세 점 전자가 있거나 식 양끝이 쌍점이어야 한다.
fn electron_pairs(items: Vec<Item>) -> Vec<Item> {
    let dotted = items.iter().any(|item| matches!(item, Item::Electrons(_)))
        || (matches!(items.first(), Some(Item::Ratio))
            && matches!(items.last(), Some(Item::Ratio)));
    if !dotted {
        return items;
    }
    items
        .into_iter()
        .map(|item| match item {
            Item::Ratio => Item::Electrons(2),
            other => other,
        })
        .collect()
}

/// 과학 제3항 — 원자 번호와 질량수는 원소 기호 뒤에 아래·위 첨자로 적는다.
/// 묵자에서 원소 앞에 선 첨자를 원소 뒤로 옮긴다(`²³⁵₉₂U` → U, ₉₂, ²³⁵).
fn reorder_prescripts(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    let mut at = 0;
    while at < items.len() {
        let term_start = out.last().is_none_or(Item::separates_terms)
            || matches!(out.last(), Some(Item::Open(_)));
        let scripts = items[at..].iter().take_while(|i| i.is_script()).count();
        let base = items.get(at + scripts);
        if term_start && scripts > 0 && matches!(base, Some(Item::Capital { element: true, .. })) {
            let (mut subs, sups): (Vec<Item>, Vec<Item>) = items[at..at + scripts]
                .iter()
                .cloned()
                .partition(|i| matches!(i, Item::Sub(_)));
            out.push(items[at + scripts].clone());
            subs.extend(sups);
            out.extend(subs);
            at += scripts + 1;
            continue;
        }
        out.push(items[at].clone());
        at += 1;
    }
    out
}

/// 식이 화학식이라는 신호. 한 글자 원소 기호는 수학 변수와 글자가 겹치므로
/// 원소 기호만으로는 화학식이라 하지 않는다.
fn has_formula_signal(items: &[Item]) -> bool {
    let scripted = items.windows(2).any(|pair| {
        matches!(
            pair[0],
            Item::Capital { element: true, .. } | Item::Close(_)
        ) && pair[1].is_script()
    });
    // 반응 화살표만으로는 화학식이라 하지 않는다(`BSI(98 → 88)`). 기체·침전
    // 표시, 상태 기호, 전자는 화학식에만 쓰인다.
    let chemical_marks = items
        .iter()
        .any(|item| matches!(item, Item::Arrow(_) | Item::State(_) | Item::Electrons(_)));
    scripted || chemical_marks || is_element_ratio(items)
}

/// 제24항 — 염기 서열. `5′―ATAAT…―3′` 처럼 끝 표시와 이음표가 있어야 한다.
fn is_base_sequence(items: &[Item]) -> bool {
    let bases = items
        .iter()
        .filter(|item| {
            matches!(item, Item::Capital { symbol, style: None, .. }
                if matches!(symbol.as_str(), "A" | "C" | "G" | "T" | "U"))
        })
        .count();
    let only_sequence = items.iter().all(|item| {
        matches!(
            item,
            Item::Number(_) | Item::Prime | Item::Dash | Item::Ellipsis | Item::Capital { .. }
        )
    });
    only_sequence
        && bases >= 4
        && bases
            == items
                .iter()
                .filter(|i| matches!(i, Item::Capital { .. }))
                .count()
        && items.contains(&Item::Prime)
        && items
            .iter()
            .any(|item| matches!(item, Item::Dash | Item::Ellipsis))
}

/// 앞뒤를 두 칸씩 띄우고 로마자표를 적지 않는 식인가(제6·24항).
pub(crate) fn stands_apart(items: &[Item]) -> bool {
    is_base_sequence(items)
        || items.iter().any(|item| {
            matches!(
                item,
                Item::Op(_) | Item::Equals | Item::Ratio | Item::FractionBar
            )
        })
}

/// `C:H:O` 처럼 쌍점으로 이은 한 글자 원소 기호가 셋 이상인가.
fn is_element_ratio(items: &[Item]) -> bool {
    let lead: Vec<&Item> = items
        .iter()
        .take_while(|item| !matches!(item, Item::Equals))
        .collect();
    lead.len() >= 5
        && lead.iter().enumerate().all(|(at, item)| {
            if at % 2 == 0 {
                item.single_capital() == Some(true)
            } else {
                matches!(item, Item::Ratio)
            }
        })
}

fn element_count(items: &[Item]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, Item::Capital { element: true, .. }))
        .count()
}

/// 과학 제1항 — 쉼표로 늘어놓은 원소 기호 목록(`Li, Na, K`). 두 글자 원소
/// 기호가 하나라도 있어야 로마자 낱말 나열과 구별된다.
pub(crate) fn is_element_list(items: &[Item]) -> bool {
    let mut elements = 0;
    let mut two_letter = false;
    let mut expect_element = true;
    let mut at = 0;
    while let Some(item) = items.get(at) {
        if expect_element {
            let Item::Capital {
                symbol,
                element: true,
                style: None,
            } = item
            else {
                return false;
            };
            elements += 1;
            two_letter |= symbol.len() == 2;
            expect_element = false;
            at += 1;
        } else {
            if !matches!(item, Item::Comma) || !matches!(items.get(at + 1), Some(Item::Space)) {
                return false;
            }
            expect_element = true;
            at += 2;
        }
    }
    !expect_element && elements >= 2 && two_letter
}

/// 식 사이의 한글 낱말은 기호 뒤, 식 앞에만 선다(`→ 아세틸 CoA`).
fn hangul_is_embedded(items: &[Item]) -> bool {
    items.iter().enumerate().all(|(at, item)| {
        if !matches!(item, Item::Hangul(_)) {
            return true;
        }
        let before = items[..at].iter().rev().find(|i| !matches!(i, Item::Space));
        let after = items[at + 1..].iter().find(|i| !matches!(i, Item::Space));
        matches!(before, Some(Item::Op(_)))
            && matches!(after, Some(Item::Capital { .. } | Item::Number(_)))
    })
}

/// 원소가 아닌 로마자 대문자는 반응식 안(`ROH`, `NAD⁺`)이나, 화학식 앞에 선
/// 로마자(제7항 2, `PETCO₂`)로만 받는다.
fn capitals_are_chemical(items: &[Item]) -> bool {
    let reaction = items
        .iter()
        .any(|item| matches!(item, Item::Op('→' | '←' | '⇄' | '⇌') | Item::Arrow(_)));
    if reaction {
        return true;
    }
    let mut roman_until = 0;
    items.iter().enumerate().all(|(at, item)| {
        if at >= roman_until && starts_term(items, at) {
            roman_until = at + roman_prefix(items, at);
        }
        !matches!(item, Item::Capital { element: false, .. }) || at < roman_until
    })
}

fn brackets_balance(items: &[Item]) -> bool {
    let mut open = Vec::new();
    for item in items {
        match item {
            Item::Open(ch) => open.push(*ch),
            Item::Close(ch) => {
                let expected = if *ch == ']' { '[' } else { '(' };
                if open.pop() != Some(expected) {
                    return false;
                }
            }
            _ => {}
        }
    }
    open.is_empty()
}

/// 화학식으로 적을 식인가. `single_element` 은 원소 하나에 첨자가 붙은 식
/// (`O₂`, `H⁺`)도 받을지 정한다. 국어 문장 밖에서는 그런 식이 비타민
/// `B₆`(한글 제68항) 같은 로마자 표기와 구별되지 않는다.
pub(crate) fn is_formula(items: &[Item], single_element: bool) -> bool {
    if is_element_list(items) || is_base_sequence(items) {
        return true;
    }
    // 제3항 — 원자 번호와 질량수를 함께 단 원소(`²³⁵₉₂U`, `ₐZnᵐ`).
    let isotope = element_count(items) == 1
        && items.iter().any(|item| matches!(item, Item::Sub(_)))
        && items.iter().any(|item| matches!(item, Item::Sup(_)));
    // 원소 하나에 붙은 첨자는 수(`H₂`)나 전하(`H⁺`)일 때만 화학식이다. 글자
    // 첨자(`Sₙ`)는 수열 같은 수식의 첨자다. 원자 하나는 수를 적지 않으므로
    // `V₀`·`P₁` 의 0·1 도 변수의 첨자다.
    let charged_or_counted = items.iter().any(Item::is_script)
        && items.iter().all(|item| match item {
            Item::Sub(content) => content.parse::<u32>().is_ok_and(|count| count >= 2),
            Item::Sup(content) => content.contains(['+', '-']),
            _ => true,
        });
    let enough = element_count(items) >= 2
        || isotope
        || (single_element && element_count(items) == 1 && charged_or_counted);
    let well_formed = hangul_is_embedded(items)
        && brackets_balance(items)
        && capitals_are_chemical(items)
        && !items.iter().any(|item| matches!(item, Item::Comma))
        && !matches!(items.first(), Some(Item::Space | Item::Op(_)))
        && !matches!(items.last(), Some(Item::Space | Item::Op(_)));
    enough && well_formed && has_formula_signal(items)
}

/// 글 전체가 화학식인가. LaTeX 로 적힌 식(`$...$`)도 받는다. `science` 는 과학
/// 문맥이라 [`science_reading`] 도 식으로 받을지 정한다.
pub(crate) fn owns_text(text: &str, science: bool) -> bool {
    if configuration(text).is_some() || super::genotype::encode_genotype(text).is_some() {
        return true;
    }
    let items = match text.strip_prefix('$').and_then(|t| t.strip_suffix('$')) {
        Some(latex) => parse_latex(latex),
        None => parse(text),
    };
    items.is_some_and(|items| {
        is_formula(&items, false) || science && science_reading(&items).is_some()
    })
}

/// 과학 문맥에서만 식으로 읽는 것. 묵자 모양만으로는 로마자 낱말과 구별되지 않는다.
///
/// - 원소 기호만 이어지고 두 글자 원소 기호가 든 것(`NaCl`) — 제7항 1. 한 글자
///   원소 기호만 이어진 것은 단위(`HP`, 제30항)와 모양이 같아 받지 않는다.
/// - 단위 앞머리 뒤에 원소 기호만 이어진 것(`pOH`) — 단위 속 화학식(제30항 [붙임]).
/// - 수와 대문자를 `+` 로 이은 것(`44+XX`) — 염색체 구성(제23항). 대문자는 원소가
///   아닌 유전자라 대문자 단어표로 적는다.
pub(crate) fn science_reading(items: &[Item]) -> Option<Vec<Item>> {
    let is_element = |item: &Item| {
        matches!(
            item,
            Item::Capital {
                element: true,
                style: None,
                ..
            }
        )
    };
    let element_word = items.len() >= 2
        && items.iter().all(is_element)
        && items
            .iter()
            .any(|item| matches!(item, Item::Capital { symbol, .. } if symbol.len() == 2));
    let unit_formula = matches!(items, [Item::Unit(_), rest @ ..]
        if rest.len() >= 2 && rest.iter().all(is_element));
    if element_word || unit_formula {
        return Some(items.to_vec());
    }
    let mut numbers = 0;
    let mut genes = 0;
    for term in items.split(|item| *item == Item::Op('+')) {
        if matches!(term, [Item::Number(_)]) {
            numbers += 1;
        } else if !term.is_empty()
            && term
                .iter()
                .all(|item| matches!(item, Item::Capital { style: None, .. }))
        {
            genes += 1;
        } else {
            return None;
        }
    }
    (numbers > 0 && genes > 0).then(|| {
        items
            .iter()
            .map(|item| match item {
                Item::Capital { symbol, .. } => Item::Capital {
                    symbol: symbol.clone(),
                    element: false,
                    style: None,
                },
                other => other.clone(),
            })
            .collect()
    })
}

/// 대문자 구절 하나: 여는 원소의 자리와 종료표를 적을 마지막 낱낱의 자리.
struct Phrase {
    open: usize,
    close: usize,
}

/// 항의 첫 글자인가. 제4항 3 — 원소 기호 앞에 원소 기호가 아닌 숫자나 문자가
/// 오면 대문자 구절표를 쓸 수 없다. 괄호는 사이에 둘 수 있다.
fn starts_term(items: &[Item], at: usize) -> bool {
    items[..at]
        .iter()
        .rev()
        .find(|item| !matches!(item, Item::Open(_)))
        .is_none_or(Item::separates_terms)
}

/// 제7항 2 — 로마자 대문자에 화학식이 이어진 항(`PETCO₂`)의 로마자 부분.
/// 항 첫머리 대문자열에 원소가 아닌 글자가 있으면 그 글자까지가 로마자다.
fn roman_prefix(items: &[Item], at: usize) -> usize {
    let run = items[at..]
        .iter()
        .take_while(|item| item.single_capital().is_some())
        .count();
    items[at..at + run]
        .iter()
        .rposition(|item| item.single_capital() == Some(false))
        .map_or(0, |last| last + 1)
}

/// 대문자 구절이 끊기는 자리 — 두 글자 원소 기호, 등호, 식 사이의 한글.
fn breaks_phrase(item: &Item) -> bool {
    matches!(
        item,
        Item::Capital { symbol, .. } if symbol.len() == 2
    ) || matches!(item, Item::Equals | Item::Hangul(_) | Item::Comma)
}

fn phrases(items: &[Item]) -> Vec<Phrase> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < items.len() {
        let opens = items[at].single_capital() == Some(true)
            && starts_term(items, at)
            && roman_prefix(items, at) == 0;
        if !opens {
            at += 1;
            continue;
        }
        let mut elements = 0;
        let mut last = at;
        for (offset, item) in items[at..].iter().enumerate() {
            if breaks_phrase(item) {
                break;
            }
            if let Some(element) = item.single_capital() {
                elements += usize::from(element);
                last = at + offset;
            }
        }
        if elements < 3 {
            at += 1;
            continue;
        }
        let close = last
            + items[last + 1..]
                .iter()
                .take_while(|item| item.is_script())
                .count();
        found.push(Phrase { open: at, close });
        at = close + 1;
    }
    found
}

fn cells(pattern: &str) -> impl Iterator<Item = u8> + '_ {
    pattern.chars().map(decode_unicode)
}

/// 첨자 내용. 수는 수표를 앞세우고, `+` 는 ⠢, `-` 는 ⠔ 로 적는다(제2항).
fn script_cells(content: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let mut in_number = false;
    for ch in content.chars() {
        match ch {
            '0'..='9' => {
                if !in_number {
                    out.push(decode_unicode('⠼'));
                }
                out.push(encode_number(ch)?);
                in_number = true;
                continue;
            }
            '+' => out.push(decode_unicode('⠢')),
            '-' => out.push(decode_unicode('⠔')),
            letter => out.push(encode_english(letter)?),
        }
        in_number = false;
    }
    Ok(())
}

fn korean_cells(word: &str) -> Result<Vec<u8>, String> {
    crate::encode(word)
}

/// 반응·비교 기호 — 과학 제8·18항, 수학 제4항.
const OPERATORS: [(char, &str); 7] = [
    ('+', "⠢"),
    ('>', "⠢⠢"),
    ('<', "⠔⠔"),
    ('→', "⠒⠕"),
    ('←', "⠪⠒"),
    ('⇄', "⠪⠶⠕"),
    ('⇌', "⠪⠶⠕"),
];

fn operator_cells(op: char) -> &'static str {
    OPERATORS
        .iter()
        .find(|(key, _)| *key == op)
        .map_or("", |(_, cells)| cells)
}

/// 강조 표시자. 같은 강조가 뒤 글자로 이어지면 단어 표시자, 아니면 기호 표시자.
/// 표시자를 적었으면 `true` — 표시자가 수와 글자 사이를 가르므로 제5항의 ⠐ 은
/// 필요 없어진다.
fn emphasis_before(items: &[Item], at: usize, out: &mut Vec<u8>) -> bool {
    let Some(Item::Capital {
        style: Some(style), ..
    }) = items.get(at)
    else {
        return false;
    };
    let continues = |item: Option<&Item>| matches!(item, Some(Item::Capital { style: Some(s), .. }) if s == style);
    if at > 0 && continues(items.get(at - 1)) {
        return false;
    }
    let word = continues(items.get(at + 1));
    out.push(style.lead());
    out.push(decode_unicode(if word { '⠂' } else { '⠆' }));
    true
}

/// 식을 점자로 적는다.
pub(crate) fn encode(items: &[Item]) -> Result<Vec<u8>, String> {
    let phrases = phrases(items);
    let capital = decode_unicode('⠠');
    let mut out = Vec::new();
    let mut open_until: Option<usize> = None;
    let mut numeric = false;
    let mut at = 0;
    while at < items.len() {
        let item = &items[at];
        let is_capital = matches!(item, Item::Capital { .. });
        if is_capital && emphasis_before(items, at, &mut out) {
            numeric = false;
        }
        if let Some(phrase) = phrases.iter().find(|p| p.open == at) {
            out.extend([capital; 3]);
            open_until = Some(phrase.close);
        }
        let prefix = if is_capital && open_until.is_none() && starts_term(items, at) {
            roman_prefix(items, at)
        } else {
            0
        };
        if prefix >= 2 {
            out.extend([capital, capital]);
            for letter in &items[at..at + prefix] {
                write_letters(letter, &mut out)?;
            }
            at += prefix;
            numeric = false;
            continue;
        }
        numeric = match item {
            Item::Capital { symbol, .. } if open_until.is_some() => {
                if numeric && symbol.starts_with(|c: char| ('A'..='J').contains(&c)) {
                    out.push(decode_unicode('⠐'));
                }
                write_letters(item, &mut out)?;
                false
            }
            _ => encode_item(item, &mut out)?,
        };
        if open_until == Some(at) {
            out.extend([capital, decode_unicode('⠄')]);
            open_until = None;
            numeric = false;
        }
        at += 1;
    }
    Ok(out)
}

fn write_letters(item: &Item, out: &mut Vec<u8>) -> Result<(), String> {
    if let Item::Capital { symbol, .. } = item {
        for letter in symbol.chars() {
            out.push(encode_english(letter.to_ascii_lowercase())?);
        }
    }
    Ok(())
}

/// 대문자 구절 밖의 낱낱을 적고, 적은 끝이 수인지 돌려준다.
fn encode_item(item: &Item, out: &mut Vec<u8>) -> Result<bool, String> {
    match item {
        Item::Capital { .. } => {
            out.push(decode_unicode('⠠'));
            write_letters(item, out)?;
        }
        Item::Sub(content) => {
            out.push(decode_unicode('⠰'));
            script_cells(content, out)?;
            return Ok(content.ends_with(|c: char| c.is_ascii_digit()));
        }
        Item::Sup(content) => {
            out.push(decode_unicode('⠘'));
            script_cells(content, out)?;
            return Ok(content.ends_with(|c: char| c.is_ascii_digit()));
        }
        Item::Number(digits) => {
            script_cells(digits, out)?;
            return Ok(true);
        }
        Item::Open('[') => out.extend(cells("⠷⠄")),
        Item::Open(_) => out.extend(cells("⠦")),
        Item::Close(']') => out.extend(cells("⠠⠾")),
        Item::Close(_) => out.extend(cells("⠴")),
        Item::State(state) => {
            out.extend(cells("⠦⠄⠴"));
            for letter in state.chars() {
                out.push(encode_english(letter)?);
            }
            out.extend(cells("⠠⠴"));
        }
        Item::Gloss(word) => {
            out.extend(cells("⠦⠄"));
            out.extend(korean_cells(word)?);
            out.extend(cells("⠠⠴"));
        }
        Item::Hangul(word) => {
            out.extend(cells("⠸⠷"));
            out.extend(korean_cells(word)?);
            out.extend(cells("⠸⠾"));
        }
        Item::Dot => out.extend(cells("⠐")),
        Item::Ratio => out.extend(cells("⠐⠂")),
        Item::Equals => out.extend(cells("⠒⠒")),
        Item::Comma => out.extend(cells("⠂")),
        Item::Op(op) => out.extend(cells(operator_cells(*op))),
        Item::Arrow('↑') => out.extend(cells("⠰⠒⠕")),
        Item::Arrow(_) => out.extend(cells("⠘⠒⠕")),
        Item::Group(true) => out.extend(cells("⠷")),
        Item::Group(false) => out.extend(cells("⠾")),
        Item::FractionBar => out.extend(cells("⠌")),
        Item::Space => out.extend(cells("⠀")),
        Item::Unit(letters) => {
            for letter in letters.chars() {
                out.push(encode_english(letter)?);
            }
        }
        Item::Electrons(count) => out.extend(std::iter::repeat_n(decode_unicode('⠔'), *count)),
        Item::Prime => out.extend(cells("⠤")),
        Item::Dash => out.extend(cells("⠠⠤")),
        Item::Ellipsis => out.extend(cells("⠄⠄⠄")),
        Item::Bond(1) => out.extend(cells("⠰⠂")),
        Item::Bond(2) => out.extend(cells("⠰⠆")),
        Item::Bond(_) => out.extend(cells("⠰⠒")),
        Item::Branch(mark) => out.push(decode_unicode(*mark)),
    }
    Ok(false)
}

/// 과학 제19항 — 전자 배치(`1s²2s²2p⁶`). 주 양자수와 방위 양자수 기호 사이는
/// 혼동이 있을 때(a~j 의 글자)에만 ⠐ 을 적는다.
pub(crate) fn configuration(text: &str) -> Option<Vec<u8>> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let digits: String = chars[at..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        at += digits.len();
        let orbital = *chars
            .get(at)
            .filter(|c| matches!(c, 's' | 'p' | 'd' | 'f'))?;
        at += 1;
        let electrons: String = chars[at..].iter().map_while(|c| superscript(*c)).collect();
        if digits.is_empty()
            || electrons.is_empty()
            || !electrons.chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }
        at += electrons.chars().count();
        script_cells(&digits, &mut out).ok()?;
        if ('a'..='j').contains(&orbital) {
            out.push(decode_unicode('⠐'));
        }
        out.push(encode_english(orbital).ok()?);
        out.push(decode_unicode('⠘'));
        script_cells(&electrons, &mut out).ok()?;
    }
    (!out.is_empty()).then_some(out)
}

/// 식이 대문자 구절로 끝나는가. 제7항 6 다만 — 그때는 로마자 종료표를 적는다.
pub(crate) fn ends_in_phrase(items: &[Item]) -> bool {
    phrases(items)
        .last()
        .is_some_and(|phrase| phrase.close + 1 == items.len())
}

/// LaTeX 로 적힌 화학식(`$K_1=\frac{[H_3O^+]}{[H_2CO_3]}$`)을 낱낱으로 푼다.
/// 분수는 분모를 먼저 적고, 둘 이상의 낱덩이로 된 쪽은 묶음 괄호로 묶는다.
pub(crate) fn parse_latex(source: &str) -> Option<Vec<Item>> {
    let (head, rest) = match source.find("\\frac") {
        Some(at) => (&source[..at], Some(&source[at + "\\frac".len()..])),
        None => (source, None),
    };
    let mut items = parse(&latex_scripts(head)?)?;
    let Some(rest) = rest else {
        return Some(items);
    };
    let (numerator, rest) = braced(rest)?;
    let (denominator, rest) = braced(rest)?;
    if !rest.is_empty() {
        return None;
    }
    items.extend(grouped(parse(&latex_scripts(denominator)?)?));
    items.push(Item::FractionBar);
    items.extend(grouped(parse(&latex_scripts(numerator)?)?));
    Some(items)
}

fn braced(source: &str) -> Option<(&str, &str)> {
    let body = source.strip_prefix('{')?;
    let mut depth = 1;
    for (at, ch) in body.char_indices() {
        depth += match ch {
            '{' => 1,
            '}' => -1,
            _ => 0,
        };
        if depth == 0 {
            return Some((&body[..at], &body[at + 1..]));
        }
    }
    None
}

fn grouped(items: Vec<Item>) -> Vec<Item> {
    let units = items
        .iter()
        .enumerate()
        .filter(|(at, item)| match item {
            Item::Open(_) => {
                items[..*at]
                    .iter()
                    .filter(|i| matches!(i, Item::Open(_)))
                    .count()
                    == items[..*at]
                        .iter()
                        .filter(|i| matches!(i, Item::Close(_)))
                        .count()
            }
            _ => false,
        })
        .count();
    if units <= 1 {
        return items;
    }
    let mut out = vec![Item::Group(true)];
    out.extend(items);
    out.push(Item::Group(false));
    out
}

/// LaTeX 첨자 `_x`·`_{..}`·`^x`·`^{..}` 를 유니코드 첨자로 바꾼다.
fn latex_scripts(source: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        let table: fn(char) -> Option<char> = match ch {
            '_' => to_subscript,
            '^' => to_superscript,
            '\\' | '{' | '}' => return None,
            // LaTeX 수식 안의 빈칸은 조판에 나타나지 않는다(`$Ca Cl_{2}$`).
            ' ' => continue,
            _ => {
                out.push(ch);
                continue;
            }
        };
        let body: String = if chars.peek() == Some(&'{') {
            chars.next();
            let body: String = chars.by_ref().take_while(|c| *c != '}').collect();
            body
        } else {
            chars.next()?.to_string()
        };
        for c in body.chars() {
            out.push(table(c)?);
        }
    }
    Some(out)
}

fn to_subscript(ch: char) -> Option<char> {
    reverse(&SUBSCRIPTS, ch)
}

fn to_superscript(ch: char) -> Option<char> {
    reverse(&SUPERSCRIPTS, ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(text: &str) -> String {
        let items = parse(text).expect("parses");
        encode(&items)
            .expect("encodes")
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect()
    }

    #[rstest::rstest]
    #[case::acetic_acid("CH₃COOH", "⠠⠠⠠⠉⠓⠰⠼⠉⠐⠉⠕⠕⠓⠠⠄")]
    #[case::phrase_starts_after_coefficient("2H₂ + O₂ → 2H₂O", "⠼⠃⠠⠓⠰⠼⠃⠀⠢⠀⠠⠠⠠⠕⠰⠼⠃⠀⠒⠕⠀⠼⠃⠐⠓⠰⠼⠃⠕⠠⠄")]
    #[case::two_letter_element_closes("H₂S > SO₂ > Cl₂", "⠠⠠⠠⠓⠰⠼⠃⠎⠀⠢⠢⠀⠎⠕⠰⠼⠃⠠⠄⠀⠢⠢⠀⠠⠉⠇⠰⠼⠃")]
    #[case::no_phrase_after_a_letter("ZnSO₄ + H₂↑", "⠠⠵⠝⠠⠎⠠⠕⠰⠼⠙⠀⠢⠀⠠⠓⠰⠼⠃⠰⠒⠕")]
    #[case::brackets_start_with_a_metal("[Cu(NH₃)₄](OH)₂", "⠷⠄⠠⠉⠥⠦⠠⠝⠠⠓⠰⠼⠉⠴⠰⠼⠙⠠⠾⠦⠠⠕⠠⠓⠴⠰⠼⠃")]
    #[case::isotope("²³⁵₉₂U", "⠠⠥⠰⠼⠊⠃⠘⠼⠃⠉⠑")]
    #[case::ion_charge("⁹₄Be²⁺", "⠠⠃⠑⠰⠼⠙⠘⠼⠊⠘⠼⠃⠢")]
    #[case::ratio("C:H:O=1:2:1", "⠠⠠⠠⠉⠐⠂⠓⠐⠂⠕⠠⠄⠒⠒⠼⠁⠐⠂⠼⠃⠐⠂⠼⠁")]
    #[case::roman_before_formula("PETCO₂", "⠠⠠⠏⠑⠞⠠⠉⠠⠕⠰⠼⠃")]
    #[case::bold_opener("𝐂₂H₅OH", "⠘⠆⠠⠠⠠⠉⠰⠼⠃⠐⠓⠰⠼⠑⠕⠓⠠⠄")]
    #[case::italic_word("C₂H₅𝑂𝐻", "⠠⠠⠠⠉⠰⠼⠃⠐⠓⠰⠼⠑⠨⠂⠕⠓⠠⠄")]
    #[case::underlined_symbol("C₂H̲₅̲OH", "⠠⠠⠠⠉⠰⠼⠃⠸⠆⠓⠰⠼⠑⠕⠓⠠⠄")]
    #[case::element_list("F, Cl, Br, I", "⠠⠋⠂⠀⠠⠉⠇⠂⠀⠠⠃⠗⠂⠀⠠⠊")]
    #[case::electron_dots(":N⋮⋮N:", "⠔⠔⠠⠝⠔⠔⠔⠔⠔⠔⠠⠝⠔⠔")]
    #[case::base_sequence("5′―ATAATG―3′", "⠼⠑⠤⠠⠤⠠⠠⠁⠞⠁⠁⠞⠛⠠⠤⠼⠉⠤")]
    #[case::unit_prefix("mH₂O", "⠍⠠⠓⠰⠼⠃⠠⠕")]
    fn writes_formulas(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(braille(text), expected);
    }

    #[rstest::rstest]
    #[case::vitamin_outside_korean("B₆", false, false)]
    #[case::vitamin_inside_korean("B₆", true, true)]
    #[case::sequence_index("Sₙ", true, false)]
    #[case::initial_value_index("V₀", true, false)]
    #[case::first_value_index("P₁", true, false)]
    #[case::bare_letters("NHK", false, false)]
    #[case::spelled_out_word("WATER", false, false)]
    #[case::matrix_names("AB", false, false)]
    #[case::two_elements("H₂O", false, true)]
    #[case::unknown_capital_without_reaction("QO₂", false, false)]
    #[case::words_list("I, O, U", false, false)]
    #[case::abbreviation_before_colon("ATM:", true, false)]
    #[case::arrow_between_numbers("BSI(98 → 88)", true, false)]
    fn recognises_only_formulas(#[case] text: &str, #[case] single: bool, #[case] expected: bool) {
        let formula = parse(text).is_some_and(|items| is_formula(&items, single));
        assert_eq!(formula, expected);
    }

    #[rstest::rstest]
    #[case::shell_filling("1s²2s²2p⁶", "⠼⠁⠎⠘⠼⠃⠼⠃⠎⠘⠼⠃⠼⠃⠏⠘⠼⠋")]
    #[case::d_orbital("3d⁵", "⠼⠉⠐⠙⠘⠼⠑")]
    fn writes_electron_configurations(#[case] text: &str, #[case] expected: &str) {
        let braille: String = configuration(text)
            .expect("is a configuration")
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect();
        assert_eq!(braille, expected);
    }

    #[test]
    fn writes_a_latex_equilibrium_constant() {
        let items = parse_latex("K_1=\\frac{[H_3O^+][HCO_3^-]}{[H_2CO_3]}").expect("parses");
        let braille: String = encode(&items)
            .expect("encodes")
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect();
        assert_eq!(
            braille,
            "⠠⠅⠰⠼⠁⠒⠒⠷⠄⠠⠠⠠⠓⠰⠼⠃⠐⠉⠕⠰⠼⠉⠠⠾⠌⠷⠷⠄⠓⠰⠼⠉⠕⠘⠢⠠⠾⠷⠄⠓⠉⠕⠰⠼⠉⠘⠔⠠⠄⠠⠾⠾"
        );
    }

    #[rstest::rstest]
    #[case::closed_phrase("C₆H₁₂O₆", true)]
    #[case::open_letters("H₂O", false)]
    fn knows_when_a_formula_ends_in_a_phrase(#[case] text: &str, #[case] expected: bool) {
        let items = parse(text).expect("parses");
        assert_eq!(ends_in_phrase(&items), expected);
    }

    #[rstest::rstest]
    #[case::reaction_with_rest_group("HNCO + ROH → NH₂·CO·OR", "⠠⠠⠠⠓⠝⠉⠕⠀⠢⠀⠗⠕⠓⠀⠒⠕⠀⠝⠓⠰⠼⠃⠐⠉⠕⠐⠕⠗⠠⠄")]
    #[case::state_symbols(
        "2S(s) + 3O₂(g) → 2SO₃(g)",
        "⠼⠃⠠⠎⠦⠄⠴⠎⠠⠴⠀⠢⠀⠼⠉⠠⠕⠰⠼⠃⠦⠄⠴⠛⠠⠴⠀⠒⠕⠀⠼⠃⠠⠎⠠⠕⠰⠼⠉⠦⠄⠴⠛⠠⠴"
    )]
    #[case::precipitate("Ag⁺ + Cl⁻ → AgCl↓", "⠠⠁⠛⠘⠢⠀⠢⠀⠠⠉⠇⠘⠔⠀⠒⠕⠀⠠⠁⠛⠠⠉⠇⠘⠒⠕")]
    #[case::gloss_and_hangul_word(
        "C₃H₄O₃(피루브산) + NAD⁺ + CoA → 아세틸 CoA",
        "⠠⠠⠠⠉⠰⠼⠉⠐⠓⠰⠼⠙⠕⠰⠼⠉⠦⠄⠙⠕⠐⠍⠘⠪⠇⠒⠠⠴⠀⠢⠀⠝⠁⠙⠘⠢⠠⠄⠀⠢⠀⠠⠉⠕⠠⠁⠀⠒⠕⠀⠸⠷⠣⠠⠝⠓⠕⠂⠸⠾⠀⠠⠉⠕⠠⠁"
    )]
    fn writes_reactions(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(braille(text), expected);
    }

    #[test]
    fn writes_every_kind_of_item() {
        let items = vec![
            Item::Open('['),
            Item::Capital {
                symbol: "Q".into(),
                element: false,
                style: None,
            },
            Item::Close(']'),
            Item::Open('('),
            Item::Close(')'),
            Item::State("aq".into()),
            Item::Gloss("가".into()),
            Item::Hangul("가".into()),
            Item::Dot,
            Item::Ratio,
            Item::Equals,
            Item::Comma,
            Item::Op('<'),
            Item::Op('←'),
            Item::Op('⇌'),
            Item::Arrow('↑'),
            Item::Arrow('↓'),
            Item::Group(true),
            Item::Group(false),
            Item::FractionBar,
            Item::Space,
            Item::Unit("m".into()),
            Item::Electrons(2),
            Item::Prime,
            Item::Dash,
            Item::Ellipsis,
            Item::Number("12".into()),
            Item::Sub("a".into()),
            Item::Sup("2+".into()),
        ];
        let braille: String = encode(&items)
            .expect("encodes")
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect();
        assert_eq!(
            braille,
            "⠷⠄⠠⠟⠠⠾⠦⠴⠦⠄⠴⠁⠟⠠⠴⠦⠄⠫⠠⠴⠸⠷⠫⠸⠾⠐⠐⠂⠒⠒⠂⠔⠔⠪⠒⠪⠶⠕⠰⠒⠕⠘⠒⠕⠷⠾⠌⠀⠍⠔⠔⠤⠠⠤⠄⠄⠄⠼⠁⠃⠰⠁⠘⠼⠃⠢"
        );
    }

    #[test]
    fn parses_the_symbols_a_formula_can_hold() {
        let items = parse("[H]·=,+>→⇄↑′―… ").expect("parses");
        assert_eq!(
            items,
            vec![
                Item::Open('['),
                Item::Capital {
                    symbol: "H".into(),
                    element: true,
                    style: None,
                },
                Item::Close(']'),
                Item::Dot,
                Item::Equals,
                Item::Comma,
                Item::Op('+'),
                Item::Op('>'),
                Item::Op('→'),
                Item::Op('⇄'),
                Item::Arrow('↑'),
                Item::Prime,
                Item::Dash,
                Item::Ellipsis,
                Item::Space,
            ]
        );
    }

    #[rstest::rstest]
    #[case::punctuation_outside_formulas("H₂O!")]
    #[case::bold_italic_letter("𝑨")]
    #[case::underline_with_nothing_before("\u{0332}H")]
    #[case::lowercase_word("mx")]
    fn refuses_what_a_formula_cannot_contain(#[case] text: &str) {
        assert!(parse(text).is_none());
    }

    #[rstest::rstest]
    #[case::list_without_space("Li,Na", false)]
    #[case::list_starting_with_a_letter("A, Na", false)]
    #[case::hangul_after_a_formula("H₂O 물", false)]
    #[case::crossed_brackets("[H₂O)", false)]
    #[case::stray_closing_bracket("H₂O)", false)]
    #[case::electron_pairs_at_both_ends(":H:N:", true)]
    fn checks_the_shape_of_a_formula(#[case] text: &str, #[case] expected: bool) {
        let formula = parse(text).is_some_and(|items| is_formula(&items, false));
        assert_eq!(formula, expected);
    }

    #[rstest::rstest]
    #[case::configuration("1s²2s²", true)]
    #[case::genotype("RRyy", true)]
    #[case::latex_formula("$[H_2O]$", true)]
    #[case::latex_variable("$A_1$", false)]
    #[case::plain_word("water", false)]
    fn owns_only_scientific_text(#[case] text: &str, #[case] expected: bool) {
        assert_eq!(owns_text(text, false), expected);
    }

    #[rstest::rstest]
    #[case::two_letter_element_word("NaCl", false, true)]
    #[case::unit_holding_a_formula("pOH", false, true)]
    #[case::chromosome_count("44+XX", false, true)]
    #[case::sum_of_terms("2+3", false, false)]
    #[case::single_letter_elements_only("CO", false, false)]
    fn owns_what_only_science_reads(
        #[case] text: &str,
        #[case] outside_science: bool,
        #[case] in_science: bool,
    ) {
        assert_eq!(owns_text(text, false), outside_science);
        assert_eq!(owns_text(text, true), in_science);
    }

    #[rstest::rstest]
    #[case::two_letter_element_word("NaOH", "⠠⠝⠁⠠⠕⠠⠓")]
    #[case::unit_holding_a_formula("pOH", "⠏⠠⠕⠠⠓")]
    #[case::chromosome_count("44+XX", "⠼⠙⠙⠢⠠⠠⠭⠭")]
    #[case::gene_that_is_also_an_element("44+XY", "⠼⠙⠙⠢⠠⠠⠭⠽")]
    #[case::single_gene("22+X", "⠼⠃⠃⠢⠠⠭")]
    fn writes_what_only_science_reads(#[case] text: &str, #[case] expected: &str) {
        let items = science_reading(&parse(text).expect("parses")).expect("science reads it");
        let braille: String = encode(&items)
            .expect("encodes")
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("braille cell"))
            .collect();
        assert_eq!(braille, expected);
    }

    #[rstest::rstest]
    #[case::unit_with_one_element("pH")]
    #[case::unit_with_a_two_letter_element("hPa")]
    #[case::roman_letter_with_an_element("CoA")]
    #[case::number_joined_to_a_word("DF1+DF8")]
    #[case::genes_without_a_count("XX+XY")]
    #[case::styled_gene("44+𝐗")]
    #[case::trailing_operator("44+")]
    fn leaves_ordinary_text_to_other_rules(#[case] text: &str) {
        assert!(parse(text).is_none_or(|items| science_reading(&items).is_none()));
    }

    #[rstest::rstest]
    #[case::unknown_orbital("1x²")]
    #[case::missing_shell("s²")]
    #[case::missing_electrons("1s")]
    #[case::charged_orbital("1s⁺")]
    fn rejects_malformed_configurations(#[case] text: &str) {
        assert!(configuration(text).is_none());
    }

    #[rstest::rstest]
    #[case::without_fraction("K_{12}", true)]
    #[case::single_character_script("H^+", true)]
    #[case::text_after_fraction("\\frac{A}{B}C", false)]
    #[case::unclosed_fraction("\\frac{A", false)]
    #[case::other_command("\\alpha", false)]
    #[case::script_without_body("H_", false)]
    #[case::unmapped_script("H_{q}", false)]
    fn reads_the_latex_it_supports(#[case] source: &str, #[case] parsed: bool) {
        assert_eq!(parse_latex(source).is_some(), parsed);
    }

    #[rstest::rstest]
    #[case::known('<', "⠔⠔")]
    #[case::unknown('?', "")]
    fn looks_up_operators(#[case] op: char, #[case] expected: &str) {
        assert_eq!(operator_cells(op), expected);
    }
}
