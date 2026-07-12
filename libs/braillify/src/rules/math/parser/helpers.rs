//! Math parser helpers: char predicates and normalizers (extracted from parser.rs).

use super::BracketKind;
use super::MathToken;

pub(super) fn is_korean_char(c: char) -> bool {
    let code = c as u32;
    (0xAC00..=0xD7A3).contains(&code) || (0x3131..=0x3163).contains(&code)
}

/// Check if a character is a Unicode superscript digit.
pub(super) fn is_superscript_char(c: char) -> bool {
    matches!(
        c,
        '\u{2070}' | '\u{00B9}' | '\u{00B2}' | '\u{00B3}' | '\u{2074}'
            ..='\u{2079}'
                | '\u{207A}'
                | '\u{207B}'
                | '\u{207D}'
                | '\u{207E}'
                | '\u{207F}'
                | '\u{1D43}' // ᵃ (latin superscript small a)
                | '\u{1D47}' // ᵇ
                | '\u{1D9C}' // ᶜ
                | '\u{1D48}' // ᵈ
                | '\u{1D49}' // ᵉ
                | '\u{1DA0}' // ᶠ
                | '\u{1D4D}' // ᵍ
                | '\u{02B0}' // ʰ
                | '\u{2071}' // ⁱ
                | '\u{02B2}' // ʲ
                | '\u{1D4F}' // ᵏ
                | '\u{02E1}' // ˡ
                | '\u{1D50}' // ᵐ
                | '\u{1D52}' // ᵒ
                | '\u{1D56}' // ᵖ
                | '\u{02B3}' // ʳ
                | '\u{02E2}' // ˢ
                | '\u{1D57}' // ᵗ
                | '\u{1D58}' // ᵘ
                | '\u{1D5B}' // ᵛ
                | '\u{02B7}' // ʷ
                | '\u{02E3}' // ˣ
                | '\u{02B8}' // ʸ
                | '\u{1DBB}' // ᶻ
    )
}

/// Check if a character is a Unicode subscript.
pub(super) fn is_subscript_char(c: char) -> bool {
    matches!(
        c,
        '\u{2080}'..='\u{2089}' | '\u{208A}' | '\u{208B}' | '\u{208D}' | '\u{208E}'
            | '\u{2090}'..='\u{209C}' // ₐ ₑ ₒ ₓ ... ₜ
            | '\u{1D62}'..='\u{1D65}' // ᵢ ᵣ ᵤ ᵥ (phonetic extensions used as subscript)
    )
}

/// Normalize a superscript character to its base form.
pub(super) fn normalize_superscript(c: char) -> Option<MathToken> {
    match c {
        '\u{2070}' => Some(MathToken::Number("0".into())),
        '\u{00B9}' => Some(MathToken::Number("1".into())),
        '\u{00B2}' => Some(MathToken::Number("2".into())),
        '\u{00B3}' => Some(MathToken::Number("3".into())),
        '\u{2074}' => Some(MathToken::Number("4".into())),
        '\u{2075}' => Some(MathToken::Number("5".into())),
        '\u{2076}' => Some(MathToken::Number("6".into())),
        '\u{2077}' => Some(MathToken::Number("7".into())),
        '\u{2078}' => Some(MathToken::Number("8".into())),
        '\u{2079}' => Some(MathToken::Number("9".into())),
        '\u{207A}' => Some(MathToken::Operator('+')),
        '\u{207B}' => Some(MathToken::Operator('\u{2212}')),
        '\u{207D}' => Some(MathToken::OpenParen(BracketKind::MathParen)),
        '\u{207E}' => Some(MathToken::CloseParen(BracketKind::MathParen)),
        '\u{207F}' => Some(MathToken::Variable('n')),
        // Latin superscript small letters (modifier letters & phonetic extensions)
        '\u{1D43}' => Some(MathToken::Variable('a')),
        '\u{1D47}' => Some(MathToken::Variable('b')),
        '\u{1D9C}' => Some(MathToken::Variable('c')),
        '\u{1D48}' => Some(MathToken::Variable('d')),
        '\u{1D49}' => Some(MathToken::Variable('e')),
        '\u{1DA0}' => Some(MathToken::Variable('f')),
        '\u{1D4D}' => Some(MathToken::Variable('g')),
        '\u{02B0}' => Some(MathToken::Variable('h')),
        '\u{2071}' => Some(MathToken::Variable('i')),
        '\u{02B2}' => Some(MathToken::Variable('j')),
        '\u{1D4F}' => Some(MathToken::Variable('k')),
        '\u{02E1}' => Some(MathToken::Variable('l')),
        '\u{1D50}' => Some(MathToken::Variable('m')),
        '\u{1D52}' => Some(MathToken::Variable('o')),
        '\u{1D56}' => Some(MathToken::Variable('p')),
        '\u{02B3}' => Some(MathToken::Variable('r')),
        '\u{02E2}' => Some(MathToken::Variable('s')),
        '\u{1D57}' => Some(MathToken::Variable('t')),
        '\u{1D58}' => Some(MathToken::Variable('u')),
        '\u{1D5B}' => Some(MathToken::Variable('v')),
        '\u{02B7}' => Some(MathToken::Variable('w')),
        '\u{02E3}' => Some(MathToken::Variable('x')),
        '\u{02B8}' => Some(MathToken::Variable('y')),
        '\u{1DBB}' => Some(MathToken::Variable('z')),
        _ => None,
    }
}

/// Normalize a subscript character to its base form.
pub(super) fn normalize_subscript(c: char) -> Option<MathToken> {
    match c {
        '\u{2080}' => Some(MathToken::Number("0".into())),
        '\u{2081}' => Some(MathToken::Number("1".into())),
        '\u{2082}' => Some(MathToken::Number("2".into())),
        '\u{2083}' => Some(MathToken::Number("3".into())),
        '\u{2084}' => Some(MathToken::Number("4".into())),
        '\u{2085}' => Some(MathToken::Number("5".into())),
        '\u{2086}' => Some(MathToken::Number("6".into())),
        '\u{2087}' => Some(MathToken::Number("7".into())),
        '\u{2088}' => Some(MathToken::Number("8".into())),
        '\u{2089}' => Some(MathToken::Number("9".into())),
        '\u{208A}' => Some(MathToken::Operator('+')),
        '\u{208B}' => Some(MathToken::Operator('\u{2212}')),
        '\u{208D}' => Some(MathToken::OpenParen(BracketKind::MathParen)),
        '\u{208E}' => Some(MathToken::CloseParen(BracketKind::MathParen)),
        '\u{2090}' => Some(MathToken::Variable('a')),
        '\u{2091}' => Some(MathToken::Variable('e')),
        '\u{2092}' => Some(MathToken::Variable('o')),
        '\u{2093}' => Some(MathToken::Variable('x')),
        '\u{2095}' => Some(MathToken::Variable('h')),
        '\u{2096}' => Some(MathToken::Variable('k')),
        '\u{2097}' => Some(MathToken::Variable('l')),
        '\u{2098}' => Some(MathToken::Variable('m')),
        '\u{2099}' => Some(MathToken::Variable('n')),
        '\u{209A}' => Some(MathToken::Variable('p')),
        '\u{209B}' => Some(MathToken::Variable('s')),
        '\u{209C}' => Some(MathToken::Variable('t')),
        // Phonetic extensions used as subscript: ᵢ ᵣ ᵤ ᵥ
        '\u{1D62}' => Some(MathToken::Variable('i')),
        '\u{1D63}' => Some(MathToken::Variable('r')),
        '\u{1D64}' => Some(MathToken::Variable('u')),
        '\u{1D65}' => Some(MathToken::Variable('v')),
        _ => None,
    }
}

/// PDF 수학 — Unicode Mathematical Alphanumeric Symbols(U+1D400–U+1D7FF)와
/// 첨자 라틴 문자(U+2071, U+2095–U+209C 등)를 ASCII 라틴 문자로 정규화한다.
/// 이는 PDF 규정에서 italic/bold/script/fraktur 변형을 일반 변수로 본다는 원칙을
/// 따른다. 한국 점자 수학 규정은 글꼴 변형을 별도로 표기하지 않으며,
/// `𝑃`(MATH ITALIC CAPITAL P) ≡ `P`로 취급한다.
pub(super) fn normalize_math_alphanumeric(c: char) -> char {
    let cp = c as u32;
    // Mathematical Italic small h는 U+1D455 자리 비고 U+210E (Planck) 사용.
    if cp == 0x210E {
        return 'h';
    }
    // Mathematical Alphanumeric Symbols: 5 letter-shape ranges (bold, italic, bold italic,
    // script, fraktur, double-struck, sans-serif, sans-serif bold, sans-serif italic,
    // sans-serif bold italic, monospace). Each block is 26 capitals + 26 smalls.
    // 정규화: cp가 해당 블록의 capital A 또는 small a 위치 기준 0~25 오프셋이면 변환.
    const BLOCKS: &[(u32, char)] = &[
        (0x1D400, 'A'),
        (0x1D41A, 'a'), // bold
        (0x1D434, 'A'),
        (0x1D44E, 'a'), // italic
        (0x1D468, 'A'),
        (0x1D482, 'a'), // bold italic
        (0x1D49C, 'A'),
        (0x1D4B6, 'a'), // script
        (0x1D4D0, 'A'),
        (0x1D4EA, 'a'), // bold script
        (0x1D504, 'A'),
        (0x1D51E, 'a'), // fraktur
        (0x1D538, 'A'),
        (0x1D552, 'a'), // double-struck
        (0x1D56C, 'A'),
        (0x1D586, 'a'), // bold fraktur
        (0x1D5A0, 'A'),
        (0x1D5BA, 'a'), // sans-serif
        (0x1D5D4, 'A'),
        (0x1D5EE, 'a'), // sans-serif bold
        (0x1D608, 'A'),
        (0x1D622, 'a'), // sans-serif italic
        (0x1D63C, 'A'),
        (0x1D656, 'a'), // sans-serif bold italic
        (0x1D670, 'A'),
        (0x1D68A, 'a'), // monospace
    ];
    for &(start, base) in BLOCKS {
        if cp >= start && cp < start + 26 {
            return char::from_u32(base as u32 + (cp - start)).unwrap_or(c);
        }
    }
    // Mathematical Bold/Sans-serif Digits U+1D7CE-U+1D7FF (5 sets of 0-9).
    const DIGIT_BLOCKS: &[u32] = &[0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6];
    for &start in DIGIT_BLOCKS {
        if cp >= start && cp < start + 10 {
            return char::from_u32(b'0' as u32 + (cp - start)).unwrap_or(c);
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::hangul_syllable('가', true)]
    #[case::compat_jamo('ㄱ', true)]
    #[case::latin('a', false)]
    fn detects_korean_char_ranges(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(is_korean_char(ch), expected);
    }

    /// `normalize_superscript` returns None for non-superscript chars.
    #[rstest::rstest]
    #[case('a')]
    #[case('1')]
    fn normalize_superscript_none_for_non_superscript(#[case] ch: char) {
        assert!(normalize_superscript(ch).is_none());
    }

    /// `normalize_subscript` returns None for non-subscript chars.
    #[rstest::rstest]
    #[case('a')]
    #[case('Z')]
    fn normalize_subscript_none_for_non_subscript(#[case] ch: char) {
        assert!(normalize_subscript(ch).is_none());
    }

    /// `normalize_math_alphanumeric` maps Math Alphanumeric Symbols to ASCII base.
    /// - U+210E PLANCK CONSTANT → 'h'
    /// - U+1D400 MATHEMATICAL BOLD CAPITAL A → 'A'
    /// - U+1D434 MATHEMATICAL ITALIC CAPITAL A → 'A'
    /// - U+1D7CE MATHEMATICAL BOLD DIGIT ZERO → '0'
    /// - U+1D7D8 MATHEMATICAL DOUBLE-STRUCK DIGIT ZERO → '0'
    #[rstest::rstest]
    #[case('\u{210E}', 'h')]
    #[case('\u{1D400}', 'A')]
    #[case('\u{1D434}', 'A')]
    #[case('\u{1D7CE}', '0')]
    #[case('\u{1D7D8}', '0')]
    fn normalize_math_alphanumeric_table(#[case] input: char, #[case] expected: char) {
        assert_eq!(
            normalize_math_alphanumeric(std::hint::black_box(input)),
            expected
        );
    }
}
