use crate::english::encode_english;
use crate::number::encode_number;
use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;

pub struct CellNotationRule;

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "21",
    subsection: None,
    name: "science_cell_notation",
    standard_ref: "2024 Korean Braille Standard, 과학 제21항",
    description: "전지의 구조를 나타낸 화학식",
};

const ELECTRODE: char = '\u{2223}';
const SALT_BRIDGE: char = '\u{2225}';

const ELEMENTS: &[&str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

/// 상태 기호. 과학 제18항 3 — 구별된 글자체를 나타내는 괄호는 한글 소괄호로 적는다.
const STATES: &[&str] = &["aq", "s", "l", "g"];

enum Item<'t> {
    Word(&'t str),
    Junction(char),
}

fn items(text: &str) -> Vec<Item<'_>> {
    let mut items = Vec::new();
    let mut start = None;
    for (offset, ch) in text.char_indices() {
        let breaks = ch.is_whitespace() || ch == ELECTRODE || ch == SALT_BRIDGE;
        if breaks {
            if let Some(from) = start.take() {
                items.push(Item::Word(&text[from..offset]));
            }
            if !ch.is_whitespace() {
                items.push(Item::Junction(ch));
            }
        } else if start.is_none() {
            start = Some(offset);
        }
    }
    if let Some(from) = start {
        items.push(Item::Word(&text[from..]));
    }
    items
}

fn subscript_digit(ch: char) -> Option<char> {
    ('\u{2080}'..='\u{2089}')
        .contains(&ch)
        .then(|| char::from(b'0' + (ch as u32 - 0x2080) as u8))
}

/// 과학 제7항 — 원소 기호마다 대문자표, 아래 첨자는 ⠰ 뒤에 수.
fn formula_cells(word: &str) -> Option<Vec<u8>> {
    let chars: Vec<char> = word.chars().collect();
    let mut out = Vec::new();
    let mut elements = 0;
    let mut at = 0;
    while at < chars.len() {
        let ch = chars[at];
        if ch.is_ascii_uppercase() {
            let two = chars
                .get(at + 1)
                .filter(|next| next.is_ascii_lowercase())
                .map(|next| format!("{ch}{next}"));
            let symbol = match two {
                Some(name) if ELEMENTS.contains(&name.as_str()) => name,
                Some(_) => return None,
                None if ELEMENTS.contains(&ch.to_string().as_str()) => ch.to_string(),
                None => return None,
            };
            out.push(decode_unicode('⠠'));
            for letter in symbol.chars() {
                out.push(encode_english(letter.to_ascii_lowercase()).ok()?);
            }
            at += symbol.len();
            elements += 1;
        } else if subscript_digit(ch).is_some() {
            out.extend([decode_unicode('⠰'), decode_unicode('⠼')]);
            while let Some(digit) = chars.get(at).copied().and_then(subscript_digit) {
                out.push(encode_number(digit).ok()?);
                at += 1;
            }
        } else if ch == '(' {
            let close = chars[at..].iter().position(|c| *c == ')')? + at;
            let state: String = chars[at + 1..close].iter().collect();
            if !STATES.contains(&state.as_str()) {
                return None;
            }
            out.extend([
                decode_unicode('⠦'),
                decode_unicode('⠄'),
                decode_unicode('⠴'),
            ]);
            for letter in state.chars() {
                out.push(encode_english(letter).ok()?);
            }
            out.extend([decode_unicode('⠠'), decode_unicode('⠴')]);
            at = close + 1;
        } else {
            return None;
        }
    }
    (elements > 0).then_some(out)
}

fn polarity_cells(word: &str) -> Option<[u8; 3]> {
    let sign = match word {
        "(-)" => '⠔',
        "(+)" => '⠢',
        _ => return None,
    };
    Some([
        decode_unicode('⠦'),
        decode_unicode(sign),
        decode_unicode('⠴'),
    ])
}

fn is_hangul_word(word: &str) -> bool {
    word.chars().all(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

/// 과학 제21항 — 전극은 ⠳, 염다리는 ⠳⠳으로 적고 앞뒤를 한 칸씩 띄운다. 식에
/// 포함된 한글은 한글표 ⠸⠷와 한글 종료표 ⠸⠾로 묶는다.
///
/// 전지 표기가 아니면 `None`. 염다리와 전극이 모두 있어야 하고, 식의 처음과 끝은
/// 화학식이나 극성이어야 하며, 한글은 바로 뒤에 전극이나 염다리가 올 때만 그
/// 전극의 설명으로 받는다. 그래서 문장 앞머리의 한글이 식으로 빨려 들지 않는다.
pub(crate) fn encode_cell(text: &str) -> Option<Vec<u8>> {
    if !text.contains(SALT_BRIDGE) || !text.contains(ELECTRODE) {
        return None;
    }
    let items = items(text);
    let is_edge = |item: Option<&Item<'_>>| match item {
        Some(Item::Word(word)) => polarity_cells(word).is_some() || formula_cells(word).is_some(),
        _ => false,
    };
    if !is_edge(items.first()) || !is_edge(items.last()) {
        return None;
    }
    let blank = decode_unicode('⠀');
    let mut out = Vec::new();
    let mut after_junction = true;
    for (position, item) in items.iter().enumerate() {
        match item {
            Item::Junction(mark) => {
                out.push(blank);
                out.push(decode_unicode('⠳'));
                if *mark == SALT_BRIDGE {
                    out.push(decode_unicode('⠳'));
                }
                out.push(blank);
                after_junction = true;
            }
            Item::Word(word) => {
                if !after_junction {
                    out.push(blank);
                }
                after_junction = false;
                if let Some(cells) = polarity_cells(word) {
                    out.extend(cells);
                } else if let Some(cells) = formula_cells(word) {
                    out.extend(cells);
                } else if is_hangul_word(word)
                    && matches!(items.get(position + 1), Some(Item::Junction(_)))
                {
                    out.extend([decode_unicode('⠸'), decode_unicode('⠷')]);
                    out.extend(crate::encode(word).ok()?);
                    out.extend([decode_unicode('⠸'), decode_unicode('⠾')]);
                } else {
                    return None;
                }
            }
        }
    }
    Some(out)
}

impl TokenRule for CellNotationRule {
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
        _state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        if !matches!(tokens.get(index), Some(Token::Word(_))) {
            return Ok(TokenAction::Noop);
        }
        let run = tokens[index..]
            .iter()
            .take_while(|token| matches!(token, Token::Word(_) | Token::Space(_)))
            .count();
        let has_bridge = tokens[index..index + run]
            .iter()
            .any(|token| matches!(token, Token::Word(word) if word.text.contains(SALT_BRIDGE)));
        if !has_bridge {
            return Ok(TokenAction::Noop);
        }
        for len in (1..=run).rev() {
            let text: String = tokens[index..index + len]
                .iter()
                .map(|token| match token {
                    Token::Word(word) => word.text.as_ref(),
                    _ => " ",
                })
                .collect();
            if let Some(cells) = encode_cell(&text) {
                return Ok(TokenAction::ReplaceRange(
                    len,
                    vec![Token::PreEncoded(cells)],
                ));
            }
        }
        Ok(TokenAction::Noop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(cells: &[u8]) -> String {
        cells
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("cell is braille"))
            .collect()
    }

    #[rstest::rstest]
    #[case::daniell(
        "Cu ∣CuSO₄(aq)∥ZnSO₄(aq)∣Zn",
        "⠠⠉⠥⠀⠳⠀⠠⠉⠥⠠⠎⠠⠕⠰⠼⠙⠦⠄⠴⠁⠟⠠⠴⠀⠳⠳⠀⠠⠵⠝⠠⠎⠠⠕⠰⠼⠙⠦⠄⠴⠁⠟⠠⠴⠀⠳⠀⠠⠵⠝"
    )]
    #[case::with_hangul_and_polarity(
        "(-) Zn∣NH₄Cl 포화용액∥MnO₂∣C (+)",
        "⠦⠔⠴⠀⠠⠵⠝⠀⠳⠀⠠⠝⠠⠓⠰⠼⠙⠠⠉⠇⠀⠸⠷⠙⠥⠚⠧⠬⠶⠗⠁⠸⠾⠀⠳⠳⠀⠠⠍⠝⠠⠕⠰⠼⠃⠀⠳⠀⠠⠉⠀⠦⠢⠴"
    )]
    fn writes_a_cell_diagram(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(braille(&encode_cell(text).expect("is a cell")), expected);
    }

    #[rstest::rstest]
    #[case::parallel_lines("AB∥CD")]
    #[case::bridge_without_electrode("Zn∥Cu")]
    #[case::leading_prose("전지는 Zn∣ZnSO₄∥CuSO₄∣Cu")]
    #[case::not_an_element("Qx∣Zn∥Cu∣Zn")]
    #[case::unknown_state("Zn(zz)∣Zn∥Cu∣Cu")]
    #[case::lone_non_element("Q∣Zn∥Cu∣Zn")]
    #[case::starts_with_electrode("∣Zn∥Cu∣Zn")]
    #[case::stray_word("Zn∣xyz∥Cu∣Cu")]
    fn leaves_everything_else_alone(#[case] text: &str) {
        assert!(encode_cell(text).is_none());
    }
}
