use phf::phf_map;

use crate::unicode::decode_unicode;

static SHORTCUT_MAP: phf::Map<char, &'static [u8]> = phf_map! {
    '"' => &[decode_unicode('⠦')],
    // '"' => &[decode_unicode('⠴')],
    '\'' => &[decode_unicode('⠠'), decode_unicode('⠦')],
    // '\'' => &[decode_unicode('⠴'), decode_unicode('⠄')],
    '~' => &[decode_unicode('⠈'), decode_unicode('⠔')],
    // PDF 제73항 붙임 1 — 빈칸 채우기 placeholder (U+F000, Private Use)
    // `⠸⠦⠦⠄` 형태로 점역한다 (4셀: 빈칸 표지 + 묶음 마커).
    '\u{F000}' => &[decode_unicode('⠸'), decode_unicode('⠦'), decode_unicode('⠦'), decode_unicode('⠄')],
    '…' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')],
    '⋯' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')],
    '!' => &[decode_unicode('⠖')],
    '.' => &[decode_unicode('⠲')],
    ',' => &[decode_unicode('⠐')],
    '?' => &[decode_unicode('⠦')],
    // PDF 제56항 — 드러냄표/굵은글자/점역자글자체 sentinels (expand_emphasis_marks가 삽입).
    '\u{E000}' => &[decode_unicode('⠠'), decode_unicode('⠤')], // 드러냄표 시작 (= 밑줄)
    '\u{E001}' => &[decode_unicode('⠤'), decode_unicode('⠄')], // 드러냄표 종료
    '\u{E002}' => &[decode_unicode('⠰'), decode_unicode('⠤')], // 굵은 글자 시작
    '\u{E003}' => &[decode_unicode('⠤'), decode_unicode('⠆')], // 굵은 글자 종료
    '\u{E004}' => &[decode_unicode('⠐'), decode_unicode('⠤')], // 점역자1 글자체 시작
    '\u{E005}' => &[decode_unicode('⠤'), decode_unicode('⠂')], // 점역자1 글자체 종료
    '\u{E006}' => &[decode_unicode('⠈'), decode_unicode('⠤')], // 점역자2 글자체 시작
    '\u{E007}' => &[decode_unicode('⠤'), decode_unicode('⠁')], // 점역자2 글자체 종료
    '“' => &[decode_unicode('⠦')],
    '”' => &[decode_unicode('⠴')],
    ':' => &[decode_unicode('⠐'), decode_unicode('⠂')],
    ';' => &[decode_unicode('⠰'), decode_unicode('⠆')],
    '_' => &[decode_unicode('⠤')],
    '*' => &[decode_unicode('⠐'), decode_unicode('⠔')],
    '(' => &[decode_unicode('⠦'), decode_unicode('⠄')],
    ')' => &[decode_unicode('⠠'), decode_unicode('⠴')],
    '{' => &[decode_unicode('⠦'), decode_unicode('⠂')],
    '}' => &[decode_unicode('⠐'), decode_unicode('⠴')],
    '[' => &[decode_unicode('⠦'), decode_unicode('⠆')],
    ']' => &[decode_unicode('⠰'), decode_unicode('⠴')],
    '〔' => &[decode_unicode('⠦'), decode_unicode('⠆')],
    '〕' => &[decode_unicode('⠰'), decode_unicode('⠴')],
    '·' => &[decode_unicode('⠐'), decode_unicode('⠆')],
    '：' => &[decode_unicode('⠐'), decode_unicode('⠂')],
    '「' => &[decode_unicode('⠐'), decode_unicode('⠦')],
    '」' => &[decode_unicode('⠴'), decode_unicode('⠂')],
    '『' => &[decode_unicode('⠰'), decode_unicode('⠦')],
    '』' => &[decode_unicode('⠴'), decode_unicode('⠆')],
    '/' => &[decode_unicode('⠸'), decode_unicode('⠌')],
    '〈' => &[decode_unicode('⠐'), decode_unicode('⠶')],
    '〉' => &[decode_unicode('⠶'), decode_unicode('⠂')],
    '《' => &[decode_unicode('⠰'), decode_unicode('⠶')],
    '》' => &[decode_unicode('⠶'), decode_unicode('⠆')],
    '―' => &[decode_unicode('⠤'), decode_unicode('⠤')],
    '-' => &[decode_unicode('⠤')],
    // En-dash / em-dash → the dash sign `⠠⠤` (distinct from the hyphen `⠤`); a tight
    // number range (`1914–18`, `4—7`) reaches this table for the dash glyph.
    '\u{2013}' => &[decode_unicode('⠠'), decode_unicode('⠤')],
    '\u{2014}' => &[decode_unicode('⠠'), decode_unicode('⠤')],
    '∼' => &[decode_unicode('⠈'), decode_unicode('⠔')],
    '‘' => &[decode_unicode('⠠'), decode_unicode('⠦')],
    '’' => &[decode_unicode('⠴'), decode_unicode('⠄')],
    '○' => &[decode_unicode('⠸'),decode_unicode('⠴'), decode_unicode('⠇')],
    // '×' => &[decode_unicode('⠸'),decode_unicode('⠭'), decode_unicode('⠇')],
    '△' => &[decode_unicode('⠸'),decode_unicode('⠬'), decode_unicode('⠇')],
    '☆' => &[decode_unicode('⠸'),decode_unicode('⠔'), decode_unicode('⠇')],
    '◇' => &[decode_unicode('⠸'),decode_unicode('⠢'), decode_unicode('⠇')],
    '◆' => &[decode_unicode('⠸'),decode_unicode('⠕'), decode_unicode('⠇')],
    '□' => &[decode_unicode('⠸'),decode_unicode('⠶'), decode_unicode('⠇')],
    '•' => &[decode_unicode('⠸'),decode_unicode('⠲')],
    'ː' => &[decode_unicode('⠠'), decode_unicode('⠄')],
    '〃' => &[decode_unicode('⠴'), decode_unicode('⠴')],
    // PDF 제60항 [붙임 1] — 참조 기호 ※ (U+203B).
    '※' => &[decode_unicode('⠸'), decode_unicode('⠔')],
};

/// Symbols that take UEB English (로마자) point shapes inside a Korean Roman
/// section (제28/33-39항): parentheses, comma, hyphen, colon. Their cells are
/// produced by the UEB §7 punctuation rule — a single source —
/// ([`crate::rules::english_ueb::rule_7::encode_punctuation`]), so this set only
/// gates *which* symbols are English-eligible and does NOT duplicate the point
/// shapes. (Whether a given `:`/`,` is actually rendered English in 제39항 영-한
/// wrap context is decided by `english_logic::should_render_symbol_as_english`.)
const ENGLISH_SYMBOL_CHARS: [char; 5] = ['(', ')', ',', '-', ':'];

pub fn encode_char_symbol_shortcut(text: char) -> Result<&'static [u8], String> {
    if let Some(code) = SHORTCUT_MAP.get(&text) {
        Ok(code)
    } else {
        Err("Invalid symbol character".to_string())
    }
}

pub fn is_symbol_char(text: char) -> bool {
    SHORTCUT_MAP.contains_key(&text)
        || crate::rules::korean::rule_64::is_enclosed_symbol(text)
        || crate::rules::korean::rule_65::is_currency_symbol(text)
        || crate::rules::korean::rule_23::is_historical_letter_symbol(text)
        || crate::rules::korean::rule_25::is_rule_25_symbol(text)
        || crate::rules::korean::rule_31::is_greek_letter(text)
        || crate::rules::korean::rule_68::is_rule_68_symbol(text)
        || crate::rules::korean::rule_69::is_rule_69_symbol(text)
        || crate::rules::korean::rule_70::is_arrow_symbol(text)
        || crate::rules::korean::rule_71::is_rule_71_symbol(text)
        || crate::rules::korean::rule_72::is_rule_72_symbol(text)
}

pub fn encode_english_char_symbol_shortcut(text: char) -> Option<Vec<u8>> {
    if !is_english_symbol_char(text) {
        return None;
    }
    // Single source: the §7 UEB punctuation cell. The gate above keeps Korean
    // context to the 제28/33-39항 subset (`( ) , - :`).
    crate::rules::english_ueb::rule_7::encode_punctuation(text)
}

pub fn is_english_symbol_char(text: char) -> bool {
    ENGLISH_SYMBOL_CHARS.contains(&text)
}

#[cfg(test)]
mod test {
    use super::*;

    /// `is_symbol_char` — PHF 사전 등록 기호는 true.
    #[rstest::rstest]
    #[case('"')]
    #[case('\'')]
    #[case('~')]
    #[case('…')]
    #[case('!')]
    #[case('.')]
    #[case(',')]
    #[case('?')]
    #[case(':')]
    #[case(';')]
    #[case('_')]
    #[case('*')]
    #[case('(')]
    #[case(')')]
    #[case('{')]
    #[case('}')]
    #[case('①')]
    #[case('ⓐ')]
    #[case('￦')]
    pub fn test_is_symbol_char(#[case] ch: char) {
        assert!(is_symbol_char(ch));
    }

    #[rstest::rstest]
    #[case::enclosed_jamo('㉠')]
    #[case::currency_dollar('$')]
    #[case::historical_letter('ㅸ')]
    #[case::middle_korean_vowel('ㆍ')]
    #[case::greek_delta('Δ')]
    #[case::rule_68_square_meter('㎡')]
    #[case::rule_69_micro('μ')]
    #[case::arrow_right('→')]
    #[case::information_at('@')]
    #[case::placeholder_double_circle('◎')]
    fn delegated_symbol_predicates_are_symbols(#[case] ch: char) {
        assert!(is_symbol_char(std::hint::black_box(ch)));
    }

    /// `encode_char_symbol_shortcut` — 기호별 점역 점형 매핑.
    #[rstest::rstest]
    #[case::double_quote('"', vec!['⠦'])]
    #[case::single_quote('\'', vec!['⠠', '⠦'])]
    #[case::tilde('~', vec!['⠈', '⠔'])]
    #[case::horizontal_ellipsis('…', vec!['⠠', '⠠', '⠠'])]
    #[case::midline_ellipsis('⋯', vec!['⠠', '⠠', '⠠'])]
    #[case::exclamation('!', vec!['⠖'])]
    #[case::period('.', vec!['⠲'])]
    #[case::comma(',', vec!['⠐'])]
    #[case::question('?', vec!['⠦'])]
    #[case::colon(':', vec!['⠐', '⠂'])]
    #[case::semicolon(';', vec!['⠰', '⠆'])]
    #[case::underscore('_', vec!['⠤'])]
    #[case::asterisk('*', vec!['⠐', '⠔'])]
    #[case::open_paren('(', vec!['⠦', '⠄'])]
    #[case::close_paren(')', vec!['⠠', '⠴'])]
    pub fn test_encode_char_symbol_shortcut(#[case] ch: char, #[case] expected_unicode: Vec<char>) {
        let expected: Vec<u8> = expected_unicode.into_iter().map(decode_unicode).collect();
        assert_eq!(
            encode_char_symbol_shortcut(ch).unwrap(),
            expected.as_slice()
        );
    }

    #[test]
    fn test_encode_english_char_symbol_shortcut_variants() {
        assert_eq!(
            encode_english_char_symbol_shortcut('(').unwrap(),
            vec![decode_unicode('⠐'), decode_unicode('⠣')]
        );
        assert_eq!(
            encode_english_char_symbol_shortcut(')').unwrap(),
            vec![decode_unicode('⠐'), decode_unicode('⠜')]
        );
        assert_eq!(encode_english_char_symbol_shortcut('?'), None);
    }
}
