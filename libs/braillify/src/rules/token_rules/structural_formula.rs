use crate::english::encode_english;
use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::unicode::decode_unicode;

pub struct StructuralFormulaRule;

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "10",
    subsection: None,
    name: "science_structural_formula",
    standard_ref: "2024 Korean Braille Standard, 과학 제10항",
    description: "사슬 화합물의 기호 표기 형식",
};

/// 한 글자로 된 원소 기호. 과학 제4항이 대문자 구절표를 요구하는 대상이다.
const SINGLE_LETTER_ELEMENTS: &[char] = &[
    'H', 'B', 'C', 'N', 'O', 'F', 'P', 'S', 'K', 'V', 'Y', 'I', 'W', 'U',
];

/// 결합선. 과학 제10항 2 — ⠰을 먼저 적고 결합 수에 따라 1, 2, 3을 붙인다.
fn bond_cells(mark: char) -> Option<[u8; 2]> {
    let count = match mark {
        '-' => '⠂',
        '=' => '⠆',
        '≡' => '⠒',
        _ => return None,
    };
    Some([decode_unicode('⠰'), decode_unicode(count)])
}

/// 원소와 결합선이 번갈아 놓인 사슬인지 본다.
///
/// 원소를 한 글자짜리 실제 원소 기호로만 한정하고 셋 이상을 요구한다. 수학의
/// `A-B`는 A가 원소가 아니라서, 두 글자짜리 이름은 길이 때문에 걸리지 않는다.
fn chain_of_elements(text: &str) -> Option<Vec<char>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 5 || chars.len().is_multiple_of(2) {
        return None;
    }
    let elements = chars.iter().step_by(2).count();
    if elements < 3 {
        return None;
    }
    for (offset, mark) in chars.iter().enumerate() {
        let valid = if offset % 2 == 0 {
            SINGLE_LETTER_ELEMENTS.contains(mark)
        } else {
            bond_cells(*mark).is_some()
        };
        if !valid {
            return None;
        }
    }
    Some(chars)
}

fn encode_chain(chars: &[char]) -> Result<Vec<u8>, String> {
    let capital = decode_unicode('⠠');
    let mut out = vec![capital, capital, capital];
    for (offset, mark) in chars.iter().enumerate() {
        if offset % 2 == 0 {
            out.push(encode_english(mark.to_ascii_lowercase())?);
        } else {
            out.extend(bond_cells(*mark).ok_or("not a bond line")?);
        }
    }
    out.extend([capital, decode_unicode('⠄')]);
    Ok(out)
}

impl TokenRule for StructuralFormulaRule {
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
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let Some(chain) = chain_of_elements(word.text.as_ref()) else {
            return Ok(TokenAction::Noop);
        };
        Ok(TokenAction::Replace(Token::PreEncoded(encode_chain(
            &chain,
        )?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::single_bond("H-O-H", true)]
    #[case::double_bond("O=C=O", true)]
    #[case::mixed_bonds("H-C≡C-H", true)]
    #[case::not_elements("A-B", false)]
    #[case::two_elements("H-O", false)]
    #[case::plain_word("water", false)]
    fn recognises_only_element_chains(#[case] text: &str, #[case] expected: bool) {
        assert_eq!(chain_of_elements(text).is_some(), expected);
    }

    #[test]
    fn writes_the_chain_inside_a_capitals_passage() {
        let chain = chain_of_elements("H-O-H").expect("H-O-H is a chain");
        let cells = encode_chain(&chain).expect("chain encodes");
        let braille: String = cells
            .iter()
            .map(|cell| char::from_u32(0x2800 + u32::from(*cell)).expect("cell is braille"))
            .collect();
        assert_eq!(braille, "⠠⠠⠠⠓⠰⠂⠕⠰⠂⠓⠠⠄");
    }
}
