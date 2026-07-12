//! §9 Typeforms (italic, bold, underline, script).
//!
//! §9.x: emphasised text carries a typeform indicator. This module covers the
//! *symbol* level — a single styled letter takes a symbol indicator (`⠨⠆`
//! italic, `⠘⠆` bold, `⠸⠆` underline, `⠈⠆` script) before its base cell. The
//! style signal comes from Unicode: the Mathematical Alphanumeric Symbols block
//! (U+1D400–1D7FF) encodes bold/italic/… letters, and a combining low line
//! (U+0332) underlines its preceding letter. The parser turns each styled letter
//! into an [`EnglishToken::Styled`] that acts as a contraction boundary, so the
//! plain neighbours still contract (`𝐛right` → bold-`b` then the `right`
//! groupsign).
//!
//! NFC (not NFKC/NFKD) is used upstream so these styled code points survive — a
//! compatibility fold would collapse them back to plain ASCII and erase the
//! typeform.

use crate::unicode::decode_unicode;

use super::token::Typeform;

/// Decode a styled Latin letter to its plain base char and typeform, or `None`.
///
/// Handles the contiguous bold (U+1D400) and italic (U+1D434) letter blocks
/// (each laid out as `A`–`Z` then `a`–`z`) plus the one Letterlike-block gap
/// (`ℎ` U+210E = italic h, whose math-alphanumeric slot U+1D455 is unassigned).
pub fn decode_styled(c: char) -> Option<(char, Typeform)> {
    let cp = c as u32;
    if cp == 0x210E {
        return Some(('h', Typeform::Italic)); // ℎ fills the italic-h gap
    }
    for (base, form) in [(0x1D400u32, Typeform::Bold), (0x1D434u32, Typeform::Italic)] {
        if (base..base + 52).contains(&cp) {
            let off = (cp - base) as u8;
            let letter = if off < 26 {
                (b'A' + off) as char
            } else {
                (b'a' + off - 26) as char
            };
            return Some((letter, form));
        }
    }
    // Mathematical bold-italic letters (U+1D468–U+1D49B) carry two nested §9
    // typeforms.  The indicator helpers below emit both in print order.
    if (0x1D468..0x1D49B).contains(&cp) {
        let off = (cp - 0x1D468) as u8;
        let letter = if off < 26 {
            (b'A' + off) as char
        } else {
            (b'a' + off - 26) as char
        };
        return Some((letter, Typeform::BoldItalic));
    }
    // Bold digits 𝟎–𝟗 (U+1D7CE–1D7D7) — the styled-digit block in the corpus.
    if (0x1D7CE..=0x1D7D7).contains(&cp) {
        return Some(((b'0' + (cp - 0x1D7CE) as u8) as char, Typeform::Bold));
    }
    // §9.5 first transcriber-defined typeform: mathematical monospace letters and
    // digits represent the typewriter font used in the examples.
    if (0x1D670..=0x1D6A3).contains(&cp) {
        let off = (cp - 0x1D670) as u8;
        let letter = if off < 26 {
            (b'A' + off) as char
        } else {
            (b'a' + off - 26) as char
        };
        return Some((letter, Typeform::Transcriber1));
    }
    if (0x1D7F6..=0x1D7FF).contains(&cp) {
        return Some((
            (b'0' + (cp - 0x1D7F6) as u8) as char,
            Typeform::Transcriber1,
        ));
    }
    if let Some(letter) = decode_script_letter(cp) {
        return Some((letter, Typeform::Script));
    }
    None
}

/// §9.6.1 small capitals used for abbreviations/Roman numerals are transcribed as
/// ordinary capitals, not as a distinct typeform.
pub fn decode_small_cap(c: char) -> Option<char> {
    match c {
        'ᴀ' => Some('A'),
        'ʙ' => Some('B'),
        'ᴄ' => Some('C'),
        'ᴅ' => Some('D'),
        'ᴇ' => Some('E'),
        'ꜰ' => Some('F'),
        'ɢ' => Some('G'),
        'ʜ' => Some('H'),
        'ɪ' => Some('I'),
        'ᴊ' => Some('J'),
        'ᴋ' => Some('K'),
        'ʟ' => Some('L'),
        'ᴍ' => Some('M'),
        'ɴ' => Some('N'),
        'ᴏ' => Some('O'),
        'ᴘ' => Some('P'),
        'ꞯ' => Some('Q'),
        'ʀ' => Some('R'),
        'ꜱ' => Some('S'),
        'ᴛ' => Some('T'),
        'ᴜ' => Some('U'),
        'ᴠ' => Some('V'),
        'ᴡ' => Some('W'),
        'ʏ' => Some('Y'),
        'ᴢ' => Some('Z'),
        _ => None,
    }
}

fn decode_script_letter(cp: u32) -> Option<char> {
    if (0x1D49C..=0x1D4B5).contains(&cp) {
        return script_upper(cp).map(|off| (b'A' + off) as char);
    }
    if (0x1D4B6..=0x1D4CF).contains(&cp) {
        return script_lower(cp).map(|off| (b'a' + off) as char);
    }
    match cp {
        0x210A => Some('g'),
        0x210B => Some('H'),
        0x2110 => Some('I'),
        0x2112 => Some('L'),
        0x211C => Some('R'),
        0x212C => Some('B'),
        0x212F => Some('e'),
        0x2130 => Some('E'),
        0x2131 => Some('F'),
        0x2133 => Some('M'),
        0x2134 => Some('o'),
        _ => None,
    }
}

fn script_upper(cp: u32) -> Option<u8> {
    match cp {
        0x1D49C => Some(0),
        0x212C => Some(1),
        0x1D49E..=0x1D49F => Some((cp - 0x1D49D) as u8),
        0x2130 => Some(4),
        0x2131 => Some(5),
        0x1D4A2 => Some(6),
        0x210B => Some(7),
        0x2110 => Some(8),
        0x1D4A5..=0x1D4A6 => Some((cp - 0x1D49E) as u8),
        0x2112 => Some(11),
        0x2133 => Some(12),
        0x1D4A9..=0x1D4AC => Some((cp - 0x1D4A0) as u8),
        0x211B | 0x211C => Some(17),
        0x1D4AE..=0x1D4B5 => Some((cp - 0x1D49C) as u8),
        _ => None,
    }
}

fn script_lower(cp: u32) -> Option<u8> {
    match cp {
        0x1D4B6..=0x1D4B9 => Some((cp - 0x1D4B6) as u8),
        0x212F => Some(4),
        0x1D4BB => Some(5),
        0x210A => Some(6),
        0x1D4BD..=0x1D4C3 => Some((cp - 0x1D4B6) as u8),
        0x2134 => Some(14),
        0x1D4C5..=0x1D4CF => Some((cp - 0x1D4B6) as u8),
        _ => None,
    }
}

/// The dot-4/5/6 prefix that selects a typeform (`⠨` italic, `⠘` bold, `⠸`
/// underline). The §9 indicators are this prefix plus a level cell.
fn prefixes(form: Typeform) -> &'static [char] {
    match form {
        Typeform::Italic => &['⠨'],
        Typeform::Bold => &['⠘'],
        // UEB §9.8.1 leaves multiple typeform order to the transcriber; the §9.8
        // examples nest italic outside bold (`⠨⠶⠘⠶ … ⠘⠄ … ⠨⠄`).
        Typeform::BoldItalic => &['⠨', '⠘'],
        Typeform::Underline => &['⠸'],
        Typeform::ItalicUnderline => &['⠨', '⠸'],
        Typeform::BoldUnderline => &['⠘', '⠸'],
        Typeform::BoldItalicUnderline => &['⠨', '⠘', '⠸'],
        Typeform::Script => &['⠈'],
        Typeform::Transcriber1 => &['⠈', '⠼'],
        Typeform::Transcriber2 => &['⠘', '⠼'],
        Typeform::Transcriber3 => &['⠸', '⠼'],
        Typeform::Transcriber4 => &['⠐', '⠼'],
        Typeform::Transcriber5 => &['⠨', '⠼'],
    }
}

/// The bare typeform prefix cells for a typeform-marked character that is part of
/// a larger print word whose indicator level is supplied by context.
pub fn prefix_cells(form: Typeform) -> Vec<u8> {
    prefixes(form).iter().map(|c| decode_unicode(*c)).collect()
}

/// Whether `form` nests two separate §9 typeforms — each gets its own root
/// cell (`⠘⠂⠨⠂` bold+italic word). A transcriber-defined typeform has a
/// multi-cell prefix that acts as a single indicator: all prefix cells followed
/// by ONE root cell (`⠈⠼⠶` first-transcriber passage).
fn is_nested_typeform(form: Typeform) -> bool {
    matches!(
        form,
        Typeform::BoldItalic
            | Typeform::ItalicUnderline
            | Typeform::BoldUnderline
            | Typeform::BoldItalicUnderline
    )
}

fn indicator(form: Typeform, root: char) -> Vec<u8> {
    if is_nested_typeform(form) {
        if form == Typeform::BoldItalic && root != '⠶' {
            return ['⠘', '⠨']
                .iter()
                .flat_map(|prefix| [decode_unicode(*prefix), decode_unicode(root)])
                .collect();
        }
        return prefixes(form)
            .iter()
            .flat_map(|prefix| [decode_unicode(*prefix), decode_unicode(root)])
            .collect();
    }
    let mut cells: Vec<u8> = prefixes(form).iter().map(|c| decode_unicode(*c)).collect();
    cells.push(decode_unicode(root));
    cells
}

/// The §9 *symbol* typeform indicator cells (prefix + `⠆`) for `form` — used
/// before a single styled letter (`𝑝neumonia` → `⠨⠆⠏…`).
pub fn symbol_indicator(form: Typeform) -> Vec<u8> {
    indicator(form, '⠆')
}

/// The §9.x *word* typeform indicator cells (prefix + `⠂`) for `form` — used
/// before a run of two or more styled letters (`𝑅𝑎𝑑𝑎𝑟` → `⠨⠂…`).
pub fn word_indicator(form: Typeform) -> Vec<u8> {
    indicator(form, '⠂')
}

/// The §9.x *passage* typeform indicator cells (prefix + `⠶`) for `form` — opens
/// a run of three or more styled words (`𝑂𝑙𝑖𝑣𝑒𝑟 𝑇𝑤𝑖𝑠𝑡, 𝐺𝑟𝑒𝑎𝑡 …` → `⠨⠶…⠨⠄`).
pub fn passage_indicator(form: Typeform) -> Vec<u8> {
    indicator(form, '⠶')
}

/// The §9.x typeform terminator cells (prefix + `⠄`) for `form` — closes a word
/// indicator when the emphasis ends mid-word (`𝐭𝐞𝐱𝐭book` → `⠘⠂⠞⠑⠭⠞⠘⠄⠃…`).
pub fn terminator(form: Typeform) -> Vec<u8> {
    if is_nested_typeform(form) {
        return prefixes(form)
            .iter()
            .rev()
            .flat_map(|prefix| [decode_unicode(*prefix), decode_unicode('⠄')])
            .collect();
    }
    let mut cells: Vec<u8> = prefixes(form).iter().map(|c| decode_unicode(*c)).collect();
    cells.push(decode_unicode('⠄'));
    cells
}

#[cfg(test)]
fn decode_cells(s: &str) -> Vec<u8> {
    s.chars().map(decode_unicode).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::small_cap_a('ᴀ', 'A')]
    #[case::small_cap_b('ʙ', 'B')]
    #[case::small_cap_c('ᴄ', 'C')]
    #[case::small_cap_d('ᴅ', 'D')]
    #[case::small_cap_f('ꜰ', 'F')]
    #[case::small_cap_g('ɢ', 'G')]
    #[case::small_cap_j('ᴊ', 'J')]
    #[case::small_cap_m('ᴍ', 'M')]
    #[case::small_cap_w('ᴡ', 'W')]
    #[case::small_cap_y('ʏ', 'Y')]
    fn small_caps_decode_as_capitals(#[case] c: char, #[case] expected: char) {
        assert_eq!(decode_small_cap(c), Some(expected));
    }

    #[test]
    fn plain_letter_is_not_small_cap() {
        assert_eq!(decode_small_cap('A'), None);
    }

    #[rstest::rstest]
    #[case::bold_upper_a('\u{1D400}', 'A', Typeform::Bold)]
    #[case::bold_lower_b('\u{1D41B}', 'b', Typeform::Bold)]
    #[case::bold_lower_z('\u{1D433}', 'z', Typeform::Bold)]
    #[case::italic_upper_a('\u{1D434}', 'A', Typeform::Italic)]
    #[case::italic_lower_p('\u{1D45D}', 'p', Typeform::Italic)]
    #[case::italic_h_gap('\u{210E}', 'h', Typeform::Italic)]
    #[case::italic_lower_z('\u{1D467}', 'z', Typeform::Italic)]
    #[case::bold_italic_lower_t('\u{1D495}', 't', Typeform::BoldItalic)]
    #[case::bold_digit_zero('\u{1D7CE}', '0', Typeform::Bold)]
    #[case::monospace_upper_a('\u{1D670}', 'A', Typeform::Transcriber1)]
    #[case::monospace_lower_z('\u{1D6A3}', 'z', Typeform::Transcriber1)]
    #[case::monospace_digit_nine('\u{1D7FF}', '9', Typeform::Transcriber1)]
    #[case::script_capital_a_math_alpha('\u{1D49C}', 'A', Typeform::Script)]
    #[case::script_capital_b_letterlike('\u{212C}', 'B', Typeform::Script)]
    #[case::script_capital_b_range('\u{1D49E}', 'B', Typeform::Script)]
    #[case::script_capital_c_range('\u{1D49F}', 'C', Typeform::Script)]
    #[case::script_capital_e_letterlike('\u{2130}', 'E', Typeform::Script)]
    #[case::script_capital_f_letterlike('\u{2131}', 'F', Typeform::Script)]
    #[case::script_capital_g_math_alpha('\u{1D4A2}', 'G', Typeform::Script)]
    #[case::script_capital_h_letterlike('\u{210B}', 'H', Typeform::Script)]
    #[case::script_capital_i_letterlike('\u{2110}', 'I', Typeform::Script)]
    #[case::script_capital_h_range('\u{1D4A5}', 'H', Typeform::Script)]
    #[case::script_capital_i_range('\u{1D4A6}', 'I', Typeform::Script)]
    #[case::script_capital_l_letterlike('\u{2112}', 'L', Typeform::Script)]
    #[case::script_capital_m_letterlike('\u{2133}', 'M', Typeform::Script)]
    #[case::script_capital_j_range('\u{1D4A9}', 'J', Typeform::Script)]
    #[case::script_capital_m_range('\u{1D4AC}', 'M', Typeform::Script)]
    #[case::script_capital_r_letterlike('\u{211C}', 'R', Typeform::Script)]
    #[case::script_capital_s_math_alpha('\u{1D4AE}', 'S', Typeform::Script)]
    #[case::script_capital_z_math_alpha('\u{1D4B5}', 'Z', Typeform::Script)]
    #[case::script_lower_a_math_alpha('\u{1D4B6}', 'a', Typeform::Script)]
    #[case::script_lower_d_math_alpha('\u{1D4B9}', 'd', Typeform::Script)]
    #[case::script_lower_o_letterlike('\u{2134}', 'o', Typeform::Script)]
    #[case::script_lower_e_letterlike('\u{212F}', 'e', Typeform::Script)]
    #[case::script_lower_f_math_alpha('\u{1D4BB}', 'f', Typeform::Script)]
    #[case::script_lower_g_letterlike('\u{210A}', 'g', Typeform::Script)]
    #[case::script_lower_h_math_alpha('\u{1D4BD}', 'h', Typeform::Script)]
    #[case::script_lower_n_math_alpha('\u{1D4C3}', 'n', Typeform::Script)]
    #[case::script_lower_y_math_alpha('\u{1D4CE}', 'y', Typeform::Script)]
    #[case::script_lower_z_math_alpha('\u{1D4CF}', 'z', Typeform::Script)]
    fn decodes_styled_letters(#[case] c: char, #[case] base: char, #[case] form: Typeform) {
        assert_eq!(decode_styled(c), Some((base, form)));
    }

    #[test]
    fn decodes_runtime_bold_uppercase_letter() {
        let c = std::hint::black_box('\u{1D400}');

        assert_eq!(decode_styled(c), Some(('A', Typeform::Bold)));
    }

    #[rstest::rstest]
    #[case::plain_ascii('a')]
    #[case::digit('5')]
    #[case::combining_low_line('\u{0332}')]
    fn plain_chars_are_not_styled(#[case] c: char) {
        assert_eq!(decode_styled(c), None);
    }

    #[rstest::rstest]
    #[case::italic(Typeform::Italic, '⠨')]
    #[case::bold(Typeform::Bold, '⠘')]
    #[case::underline(Typeform::Underline, '⠸')]
    #[case::script(Typeform::Script, '⠈')]
    fn symbol_indicator_uses_the_right_prefix(#[case] form: Typeform, #[case] prefix: char) {
        assert_eq!(
            symbol_indicator(form),
            vec![decode_unicode(prefix), decode_unicode('⠆')]
        );
    }

    #[test]
    fn symbol_indicator_accepts_runtime_typeform() {
        let form = std::hint::black_box(Typeform::Italic);

        assert_eq!(symbol_indicator(form), super::decode_cells("⠨⠆"));
    }

    #[test]
    fn bold_italic_indicators_are_nested() {
        assert_eq!(
            word_indicator(Typeform::BoldItalic),
            super::decode_cells("⠘⠂⠨⠂")
        );
        assert_eq!(
            passage_indicator(Typeform::BoldItalic),
            super::decode_cells("⠨⠶⠘⠶")
        );
        assert_eq!(
            terminator(Typeform::BoldItalic),
            super::decode_cells("⠘⠄⠨⠄")
        );
    }

    #[rstest::rstest]
    #[case::italic_underline(Typeform::ItalicUnderline, "⠨⠆⠸⠆")]
    #[case::bold_underline(Typeform::BoldUnderline, "⠘⠆⠸⠆")]
    #[case::bold_italic_underline(Typeform::BoldItalicUnderline, "⠨⠆⠘⠆⠸⠆")]
    #[case::transcriber_2(Typeform::Transcriber2, "⠘⠼⠆")]
    #[case::transcriber_3(Typeform::Transcriber3, "⠸⠼⠆")]
    #[case::transcriber_4(Typeform::Transcriber4, "⠐⠼⠆")]
    #[case::transcriber_5(Typeform::Transcriber5, "⠨⠼⠆")]
    fn symbol_indicators_cover_all_typeforms(#[case] form: Typeform, #[case] expected: &str) {
        assert_eq!(symbol_indicator(form), super::decode_cells(expected));
    }

    #[rstest::rstest]
    #[case::italic(Typeform::Italic, "⠨⠄")]
    #[case::transcriber_1(Typeform::Transcriber1, "⠈⠼⠄")]
    #[case::bold_underline(Typeform::BoldUnderline, "⠸⠄⠘⠄")]
    fn terminators_cover_single_and_nested_typeforms(
        #[case] form: Typeform,
        #[case] expected: &str,
    ) {
        assert_eq!(terminator(form), super::decode_cells(expected));
    }

    #[rstest::rstest]
    #[case::letterlike_b(0x212C, Some(1))]
    #[case::letterlike_e(0x2130, Some(4))]
    #[case::letterlike_f(0x2131, Some(5))]
    #[case::math_g(0x1D4A2, Some(6))]
    #[case::letterlike_h(0x210B, Some(7))]
    #[case::letterlike_i(0x2110, Some(8))]
    #[case::range_h(0x1D4A5, Some(7))]
    #[case::range_i(0x1D4A6, Some(8))]
    #[case::letterlike_l(0x2112, Some(11))]
    #[case::letterlike_m(0x2133, Some(12))]
    #[case::range_j(0x1D4A9, Some(9))]
    #[case::range_m(0x1D4AC, Some(12))]
    #[case::letterlike_r_roundhand(0x211B, Some(17))]
    #[case::letterlike_r_blackletter(0x211C, Some(17))]
    #[case::range_s(0x1D4AE, Some(18))]
    #[case::range_z(0x1D4B5, Some(25))]
    #[case::plain_a('A' as u32, None)]
    fn script_upper_offset_paths(#[case] cp: u32, #[case] expected: Option<u8>) {
        assert_eq!(script_upper(cp), expected);
    }

    #[rstest::rstest]
    #[case::range_a(0x1D4B6, Some(0))]
    #[case::range_d(0x1D4B9, Some(3))]
    #[case::letterlike_e(0x212F, Some(4))]
    #[case::math_f(0x1D4BB, Some(5))]
    #[case::letterlike_g(0x210A, Some(6))]
    #[case::range_h(0x1D4BD, Some(7))]
    #[case::range_n(0x1D4C3, Some(13))]
    #[case::letterlike_o(0x2134, Some(14))]
    #[case::range_p(0x1D4C5, Some(15))]
    #[case::range_z(0x1D4CF, Some(25))]
    #[case::plain_a('a' as u32, None)]
    fn script_lower_offset_paths(#[case] cp: u32, #[case] expected: Option<u8>) {
        assert_eq!(script_lower(cp), expected);
    }
}
