//! §4.2 Accented and modified letters.
//!
//! A letter bearing a diacritic is written as a two-cell accent indicator
//! (`⠘…` or `⠈…`) followed by the base letter (RUEB 2024 §4.2): `è` → `⠘⠡⠑`,
//! `é` → `⠘⠌⠑`, `û` → `⠘⠩⠥`. The base letter still belongs to the surrounding
//! word, so contractions elsewhere are unaffected (`Frühling` keeps its `ing`
//! groupsign).

use crate::english::encode_english;
use crate::unicode::decode_unicode;

const GRAVE: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠡')];
const ACUTE: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠌')];
const CIRCUMFLEX: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠩')];
const DIAERESIS: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠒')];
const CARON: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠬')];
const RING: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠫')];
const CEDILLA: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠯')];
const TILDE: &[u8] = &[decode_unicode('⠘'), decode_unicode('⠻')];
const STROKE: &[u8] = &[decode_unicode('⠈'), decode_unicode('⠡')];
const MACRON: &[u8] = &[decode_unicode('⠈'), decode_unicode('⠤')];
const BREVE: &[u8] = &[decode_unicode('⠈'), decode_unicode('⠬')];
/// §4.2 comma-below (Romanian `ț`, `ș`) — a three-cell indicator.
const COMMA_BELOW: &[u8] = &[
    decode_unicode('⠘'),
    decode_unicode('⠸'),
    decode_unicode('⠂'),
];
/// §4.2 letter stroke through H (Maltese `Ħ`, `ħ`).
const H_STROKE: &[u8] = &[decode_unicode('⠈'), decode_unicode('⠒')];
/// §4.2 dot above (Maltese `Ġ`, `ġ`) — a three-cell indicator.
const DOT_ABOVE: &[u8] = &[
    decode_unicode('⠘'),
    decode_unicode('⠸'),
    decode_unicode('⠆'),
];

/// Map an accented letter to (accent indicator cells, base ASCII letter).
/// Matches on the lowercased character so an uppercase accented letter (`É`,
/// `Ö`) maps to the same indicator + base; the §8 capital is added by
/// [`accent_cells`].
fn accent_of(c: char) -> Option<(&'static [u8], char)> {
    let m = match c.to_lowercase().next()? {
        'à' => (GRAVE, 'a'),
        'è' => (GRAVE, 'e'),
        'ì' => (GRAVE, 'i'),
        'ò' => (GRAVE, 'o'),
        'ù' => (GRAVE, 'u'),
        'á' => (ACUTE, 'a'),
        'é' => (ACUTE, 'e'),
        'í' => (ACUTE, 'i'),
        'ó' => (ACUTE, 'o'),
        'ú' => (ACUTE, 'u'),
        'ý' => (ACUTE, 'y'),
        'ć' => (ACUTE, 'c'),
        'ń' => (ACUTE, 'n'),
        'â' => (CIRCUMFLEX, 'a'),
        'ê' => (CIRCUMFLEX, 'e'),
        'î' => (CIRCUMFLEX, 'i'),
        'ô' => (CIRCUMFLEX, 'o'),
        'û' => (CIRCUMFLEX, 'u'),
        'ä' => (DIAERESIS, 'a'),
        'ë' => (DIAERESIS, 'e'),
        'ü' => (DIAERESIS, 'u'),
        'ÿ' => (DIAERESIS, 'y'),
        'ï' => (DIAERESIS, 'i'),
        'ö' => (DIAERESIS, 'o'),
        'č' => (CARON, 'c'),
        'š' => (CARON, 's'),
        'ž' => (CARON, 'z'),
        'ě' => (CARON, 'e'),
        'ř' => (CARON, 'r'),
        'å' => (RING, 'a'),
        'ç' => (CEDILLA, 'c'),
        'ã' => (TILDE, 'a'),
        'ñ' => (TILDE, 'n'),
        'õ' => (TILDE, 'o'),
        'ø' => (STROKE, 'o'),
        'ā' => (MACRON, 'a'),
        'ē' => (MACRON, 'e'),
        'ī' => (MACRON, 'i'),
        'ō' => (MACRON, 'o'),
        'ū' => (MACRON, 'u'),
        'ȳ' => (MACRON, 'y'),
        'ă' => (BREVE, 'a'),
        'ĕ' => (BREVE, 'e'),
        'ĭ' => (BREVE, 'i'),
        'ŏ' => (BREVE, 'o'),
        'ŭ' => (BREVE, 'u'),
        'ł' => (STROKE, 'l'),
        'ț' => (COMMA_BELOW, 't'),
        'ș' => (COMMA_BELOW, 's'),
        'ħ' => (H_STROKE, 'h'),
        'ġ' => (DOT_ABOVE, 'g'),
        _ => return None,
    };
    Some(m)
}

/// §4.2 ligatured letters: the two base letters joined by the ligature sign ⠘⠖
/// (`æ` → ⠁⠘⠖⠑, `œ` → ⠕⠘⠖⠑). Returns the (first, second) ASCII base letters.
fn ligature_bases(c: char) -> Option<(char, char)> {
    match c {
        'æ' | 'Æ' => Some(('a', 'e')),
        'œ' | 'Œ' => Some(('o', 'e')),
        _ => None,
    }
}

/// §4.6 the German eszett (sharp s) `ß`/`ẞ` → ⠨⠮, a fixed two-cell sign with no
/// base letter; the uppercase form carries the §8 capital indicator.
fn eszett_cells(c: char) -> Option<Vec<u8>> {
    matches!(c, 'ß' | 'ẞ').then(|| {
        let mut cells = Vec::with_capacity(3);
        if c.is_uppercase() {
            cells.push(decode_unicode('⠠'));
        }
        cells.extend([decode_unicode('⠨'), decode_unicode('⠮')]);
        cells
    })
}

/// Whether `c` is a supported accented or ligatured letter (so the parser keeps
/// it in a word).
pub fn is_accented(c: char) -> bool {
    accent_of(c).is_some() || ligature_bases(c).is_some() || matches!(c, 'ß' | 'ẞ')
}

/// Whether `c` is a §4.2 modifier-bearing letter (not a ligature or eszett).
pub fn is_modified_letter(c: char) -> bool {
    accent_of(c).is_some()
}

/// Braille cells for an accented or ligatured letter — `[§8 capital] + …`.
/// An uppercase letter (`É`, `Æ`) carries the capital indicator ⠠ first. `None`
/// if `c` is not a supported accented/ligatured letter.
pub fn accent_cells(c: char) -> Option<Vec<u8>> {
    if let Some(cells) = eszett_cells(c) {
        return Some(cells);
    }
    if let Some((first, second)) = ligature_bases(c) {
        let mut cells = Vec::with_capacity(5);
        if c.is_uppercase() {
            cells.push(decode_unicode('⠠'));
        }
        cells.push(encode_english(first).ok()?);
        if c.is_uppercase() {
            cells.push(decode_unicode('⠠'));
        }
        cells.extend([decode_unicode('⠘'), decode_unicode('⠖')]);
        cells.push(encode_english(second).ok()?);
        return Some(cells);
    }
    let (indicator, base) = accent_of(c)?;
    let mut cells = Vec::with_capacity(indicator.len() + 2);
    if c.is_uppercase() {
        cells.push(decode_unicode('⠠'));
    }
    cells.extend_from_slice(indicator);
    cells.push(encode_english(base).ok()?);
    Some(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::u_grave('ù', "⠘⠡⠥")]
    #[case::y_acute('ý', "⠘⠌⠽")]
    #[case::c_acute('ć', "⠘⠌⠉")]
    #[case::n_acute('ń', "⠘⠌⠝")]
    #[case::e_circumflex('ê', "⠘⠩⠑")]
    #[case::y_diaeresis('ÿ', "⠘⠒⠽")]
    #[case::s_caron('š', "⠘⠬⠎")]
    #[case::e_caron('ě', "⠘⠬⠑")]
    #[case::r_caron('ř', "⠘⠬⠗")]
    #[case::o_tilde('õ', "⠘⠻⠕")]
    #[case::i_macron('ī', "⠈⠤⠊")]
    #[case::u_breve('ŭ', "⠈⠬⠥")]
    #[case::e_grave('è', "⠘⠡⠑")]
    #[case::e_acute('é', "⠘⠌⠑")]
    #[case::u_circumflex('û', "⠘⠩⠥")]
    #[case::e_diaeresis('ë', "⠘⠒⠑")]
    #[case::u_diaeresis('ü', "⠘⠒⠥")]
    #[case::i_diaeresis('ï', "⠘⠒⠊")]
    #[case::a_ring('å', "⠘⠫⠁")]
    #[case::c_cedilla('ç', "⠘⠯⠉")]
    #[case::a_tilde('ã', "⠘⠻⠁")]
    #[case::o_stroke('ø', "⠈⠡⠕")]
    #[case::l_stroke('ł', "⠈⠡⠇")]
    // Uppercase accented letters carry the §8 capital indicator ⠠ before the accent.
    #[case::e_acute_upper('É', "⠠⠘⠌⠑")]
    #[case::o_diaeresis_upper('Ö', "⠠⠘⠒⠕")]
    #[case::u_circumflex_upper('Û', "⠠⠘⠩⠥")]
    // §4.2 ligatures æ/Æ and œ/Œ → first base + ligature sign ⠘⠖ + second base.
    #[case::ae_ligature('æ', "⠁⠘⠖⠑")]
    #[case::ae_ligature_upper('Æ', "⠠⠁⠠⠘⠖⠑")]
    #[case::oe_ligature('œ', "⠕⠘⠖⠑")]
    #[case::oe_ligature_upper('Œ', "⠠⠕⠠⠘⠖⠑")]
    // §4.6 the German eszett ß/ẞ → ⠨⠮ (uppercase form carries the §8 capital).
    #[case::eszett('ß', "⠨⠮")]
    #[case::eszett_upper('ẞ', "⠠⠨⠮")]
    // §4.2 three-cell indicators: comma-below (`ț`/`ș`), dot-above (`ġ`), and the
    // two-cell H-stroke (`ħ`/`Ħ`).
    #[case::t_comma_below('ț', "⠘⠸⠂⠞")]
    #[case::s_comma_below('ș', "⠘⠸⠂⠎")]
    #[case::h_stroke('ħ', "⠈⠒⠓")]
    #[case::h_stroke_upper('Ħ', "⠠⠈⠒⠓")]
    #[case::g_dot_above('ġ', "⠘⠸⠆⠛")]
    fn accent_cells_match_indicator_plus_base(#[case] c: char, #[case] expected: &str) {
        let want: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(accent_cells(c), Some(want));
    }

    #[test]
    fn plain_letter_is_not_accented() {
        assert!(!is_accented('e'));
        assert!(!is_modified_letter('e'));
        assert!(accent_cells('e').is_none());
    }

    #[test]
    fn ligature_and_eszett_are_accented_but_not_modified_letters() {
        assert!(is_accented('æ'));
        assert!(is_accented('ß'));
        assert!(!is_modified_letter('æ'));
        assert!(!is_modified_letter('ß'));
    }

    #[test]
    fn accent_cells_runtime_ligature_allocates_cells() {
        let letter = std::hint::black_box('æ');

        assert_eq!(
            accent_cells(letter),
            Some(vec![
                decode_unicode('⠁'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠑')
            ])
        );
    }

    #[test]
    fn accent_cells_runtime_eszett_and_upper_ligature_paths() {
        assert_eq!(
            accent_cells(std::hint::black_box('ß')),
            Some(vec![decode_unicode('⠨'), decode_unicode('⠮')])
        );
        assert_eq!(
            accent_cells(std::hint::black_box('Æ')),
            Some(vec![
                decode_unicode('⠠'),
                decode_unicode('⠁'),
                decode_unicode('⠠'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠑')
            ])
        );
    }
}
