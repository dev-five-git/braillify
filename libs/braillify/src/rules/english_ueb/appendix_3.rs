//! Appendix 3 Symbols List.
//!
//! RUEB 2024 Appendix 3 lists every braille symbol with its print symbol. The
//! symbols here are those no section of the rules assigns: mostly the technical
//! signs of the Guidelines for Technical Material, whose part the list cites in
//! square brackets. "When not otherwise indicated, symbols are assumed to take a
//! grade 1 meaning" (the list's usage column), so each entry is the symbol's full
//! grade 1 form and the list is also the reading of a symbol standing on its own.

use crate::unicode::decode_unicode;

fn cells(s: &str) -> Vec<u8> {
    s.chars().map(decode_unicode).collect()
}

/// The Appendix 3 form of `c`, or `None` if the list gives it nowhere else than a
/// section the other tables already cover.
pub fn encode_symbol(c: char) -> Option<Vec<u8>> {
    let braille = match c {
        '∫' => "⠮",   // integral sign [11]
        '∥' => "⠼⠇",  // parallel to [11]
        '∞' => "⠼⠿",  // infinity sign [11]
        '⟂' => "⠼⠤",  // perpendicular to [3]
        '⦀' => "⠼⠸⠇", // triple vertical bar [3]
        '⊾' => "⠼⠸⠪", // measured right angle sign [11]
        '∂' => "⠈⠙",  // partial derivative [11]
        '∅' => "⠈⠚",  // null set [10]
        '∮' => "⠈⠮",  // closed line integral [11]
        '¬' => "⠈⠹",  // "not" sign [10]
        '∨' => "⠈⠖",  // or [10]
        '∧' => "⠈⠦",  // and [10]
        '∵' => "⠈⠌",  // "since" [11]
        '∋' => "⠈⠘⠑", // contains as an element [10]
        '⊲' => "⠈⠸⠣", // is a normal subgroup of [10]
        '⊣' => "⠈⠸⠒", // reverse assertion [10]
        '⊳' => "⠈⠸⠜", // inverse "is normal subgroup" [10]
        '∀' => "⠘⠁",  // "for all" [11]
        '∇' => "⠘⠙",  // del, nabla [11]
        '∈' => "⠘⠑",  // is an element of [10]
        '⊂' => "⠘⠣",  // is a subset of [10]
        '∃' => "⠘⠢",  // "there exists" [11]
        '≈' => "⠘⠔",  // approximately equal to [3]
        '⊃' => "⠘⠜",  // is a superset of [10]
        '⊨' => "⠘⠸⠒", // "is valid" sign [10]
        '⇌' => "⠘⠸⠶", // equilibrium arrow [16]
        '≏' => "⠘⠐⠶", // difference between [3]
        '≡' => "⠸⠿",  // equivalent to [3]
        '∠' => "⠸⠪",  // angle sign [11]
        '⊦' => "⠸⠒",  // assertion [10]
        '±' => "⠸⠖",  // plus-or-minus [3]
        '≃' => "⠸⠔",  // approximately equal to, tilde over line [3]
        '∓' => "⠸⠤",  // minus-or-plus [3]
        '≤' => "⠸⠈⠣", // less than or equal to [3]
        '≥' => "⠸⠈⠜", // greater than or equal to [3]
        '⊆' => "⠸⠘⠣", // contained in or equal to [10]
        '⊇' => "⠸⠘⠜", // contains or equal to [10]
        '⊴' => "⠸⠸⠣", // normal subgroup of or equal [10]
        '⊵' => "⠸⠸⠜", // inverse "normal subgroup or equal" [10]
        '∝' => "⠸⠐⠶", // is proportional to [3, 11]
        '√' => "⠐⠩",  // radical without vinculum [8]
        '∗' => "⠐⠔",  // asterisk operator [3]
        '∘' => "⠐⠴",  // "hollow dot" [11]
        '≅' => "⠐⠸⠔", // tilde over equals sign [3]
        '`' => "⠨⠡",  // grave accent alone
        '¦' => "⠨⠳",  // broken vertical bar [11]
        '∪' => "⠨⠖",  // union [10]
        '∩' => "⠨⠦",  // intersection [10]
        '≪' => "⠨⠈⠣", // is much less than [3]
        '≫' => "⠨⠈⠜", // is much greater than [3]
        '⊊' => "⠨⠘⠣", // proper subset [10]
        '⊋' => "⠨⠘⠜", // proper superset [10]
        '∡' => "⠨⠸⠪", // measured angle sign [11]
        '⫤' => "⠨⠸⠒", // reverse "is valid" sign [10]
        '≑' => "⠨⠐⠶", // equals sign dotted above and below [3]
        '∴' => "⠠⠡",  // "therefore" [11]
        _ => return None,
    };
    Some(cells(braille))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::integral('∫', "⠮")]
    #[case::perpendicular('⟂', "⠼⠤")]
    #[case::element_of('∈', "⠘⠑")]
    #[case::less_or_equal('≤', "⠸⠈⠣")]
    #[case::radical('√', "⠐⠩")]
    #[case::grave_alone('`', "⠨⠡")]
    #[case::much_greater('≫', "⠨⠈⠜")]
    #[case::therefore('∴', "⠠⠡")]
    fn lists_the_grade1_form(#[case] c: char, #[case] expected: &str) {
        assert_eq!(encode_symbol(c), Some(cells(expected)));
    }

    #[test]
    fn symbols_of_other_sections_are_not_listed_here() {
        assert_eq!(encode_symbol('+'), None);
    }
}
