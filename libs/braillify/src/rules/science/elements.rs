//! 원소 기호 — 과학 점자의 모든 규칙이 함께 쓰는 단일 목록.
//!
//! 한 글자짜리 원소 기호는 수학 변수와 글자가 겹치므로(`P`, `V`, `B`, `C`), 어떤
//! 대문자가 원소인지는 이 목록으로만 판정한다. 화학식이라는 신호(첨자, 반응
//! 화살표 등)를 따로 보는 것은 각 규칙의 몫이다.

use phf::{Set, phf_set};

/// 주기율표의 118개 원소 기호.
static ELEMENTS: Set<&'static str> = phf_set! {
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
};

/// `symbol` 이 원소 기호인가.
pub fn is_element(symbol: &str) -> bool {
    ELEMENTS.contains(symbol)
}

/// 대문자 하나가 그대로 원소 기호인가. 과학 제4항 대문자 구절표의 대상이다.
pub fn is_single_letter_element(letter: char) -> bool {
    let mut buf = [0u8; 4];
    ELEMENTS.contains(letter.encode_utf8(&mut buf))
}

/// 대문자와 소문자가 이어진 두 글자 원소 기호인가(`Na`, `Cl`).
pub fn is_two_letter_element(upper: char, lower: char) -> bool {
    upper.is_ascii_uppercase()
        && lower.is_ascii_lowercase()
        && ELEMENTS.contains(format!("{upper}{lower}").as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::hydrogen("H", true)]
    #[case::sodium("Na", true)]
    #[case::oganesson("Og", true)]
    #[case::not_an_element("A", false)]
    #[case::lowercase("na", false)]
    fn knows_the_periodic_table(#[case] symbol: &str, #[case] expected: bool) {
        assert_eq!(is_element(symbol), expected);
    }

    #[rstest::rstest]
    #[case::hydrogen('H', true)]
    #[case::uranium('U', true)]
    #[case::rest_group('R', false)]
    #[case::lowercase('h', false)]
    fn knows_single_letter_elements(#[case] letter: char, #[case] expected: bool) {
        assert_eq!(is_single_letter_element(letter), expected);
    }

    #[rstest::rstest]
    #[case::chlorine('C', 'l', true)]
    #[case::cobalt('C', 'o', true)]
    #[case::not_an_element('C', 'x', false)]
    #[case::wrong_case('c', 'l', false)]
    fn knows_two_letter_elements(#[case] upper: char, #[case] lower: char, #[case] expected: bool) {
        assert_eq!(is_two_letter_element(upper, lower), expected);
    }

    #[test]
    fn holds_every_element_once() {
        assert_eq!(ELEMENTS.len(), 118);
    }
}
