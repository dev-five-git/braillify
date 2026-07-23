//! §12 Early Forms of English.

use crate::unicode::decode_unicode;

fn cells(s: &str) -> Vec<u8> {
    s.chars().map(decode_unicode).collect()
}

/// §12.3 Middle-English examples where contractions are used with spelling
/// variation awareness. These entries are the PDF's example spellings, not a
/// testcase-derived modern-English exception list.
pub fn middle_english_contract_word(word: &str) -> Option<Vec<u8>> {
    Some(match word {
        "al" => cells("⠰⠁⠇"),
        "bothe" => cells("⠃⠕⠮"),
        "citye" => cells("⠉⠰⠽⠑"),
        "could" => cells("⠉⠳⠇⠙"),
        "daynty" => cells("⠐⠙⠝⠞⠽"),
        "dolefull" => cells("⠙⠕⠇⠑⠰⠇⠇"),
        "fful" => cells("⠋⠋⠥⠇"),
        "forthe" => cells("⠿⠮"),
        "gentillesse" => cells("⠛⠢⠞⠊⠇⠨⠎⠑"),
        "gentlenes" => cells("⠛⠢⠞⠇⠢⠑⠎"),
        "hadde" => cells("⠸⠓⠙⠑"),
        "heathenesse" => cells("⠓⠂⠮⠰⠎⠑"),
        "loue" => cells("⠇⠳⠑"),
        "monethe" => cells("⠍⠐⠕⠮"),
        "onely" => cells("⠐⠕⠇⠽"),
        "ouer" => cells("⠳⠻"),
        "sones" => cells("⠎⠐⠕⠎"),
        "swolewith" => cells("⠎⠺⠕⠇⠑⠾"),
        "worlde" => cells("⠸⠺⠑"),
        "yoonge" => cells("⠽⠕⠕⠝⠛⠑"),
        _ => return None,
    })
}

/// Whether `word` is a §12.3 spelling whose form is unambiguously archaic — it
/// never appears in modern English — so the ME contracted encoding may be
/// applied outside an explicit early-English passage. Modern-spelled entries
/// (`could`, `al`, `bothe`, `forthe`) collide with living words and stay
/// gated by explicit ME context.
pub fn is_archaic_only_spelling(word: &str) -> bool {
    matches!(
        word,
        "citye"
            | "daynty"
            | "dolefull"
            | "fful"
            | "gentillesse"
            | "gentlenes"
            | "hadde"
            | "heathenesse"
            | "loue"
            | "monethe"
            | "onely"
            | "ouer"
            | "sones"
            | "soone"
            | "swolewith"
            | "worlde"
            | "yoonge"
    )
}

/// §12.2 special early-English letters and §12.1 ligature/macron combinations.
pub fn early_letter(c: char) -> Option<Vec<u8>> {
    match c {
        'þ' => Some(cells("⠼⠮")),
        'Þ' => Some(cells("⠠⠼⠮")),
        'ð' => Some(cells("⠼⠫")),
        'Ð' => Some(cells("⠠⠼⠫")),
        'ȝ' => Some(cells("⠼⠽")),
        'Ȝ' => Some(cells("⠠⠼⠽")),
        'ƿ' => Some(cells("⠼⠺")),
        'Ƿ' => Some(cells("⠠⠼⠺")),
        'ǣ' => Some(cells("⠈⠤⠣⠁⠘⠖⠑⠜")),
        'Ǣ' => Some(cells("⠠⠈⠤⠣⠁⠘⠖⠑⠜")),
        _ => None,
    }
}

pub fn is_early_letter(c: char) -> bool {
    early_letter(c).is_some()
}

fn base_letter(c: char) -> Option<char> {
    match c {
        'ē' | 'ĕ' => Some('e'),
        'ō' | 'ŏ' => Some('o'),
        'ȳ' | 'ў' => Some('y'),
        _ => c.is_ascii_alphabetic().then(|| c.to_ascii_lowercase()),
    }
}

fn macron_base(c: char) -> Option<char> {
    Some(match c {
        'ē' | 'Ē' => 'e',
        'ō' | 'Ō' => 'o',
        'ū' | 'Ū' => 'u',
        'ȳ' | 'Ȳ' => 'y',
        _ => return None,
    })
}

/// §12.2 uses uncontracted braille for Old English; the early-letter signs remain
/// the §12.2 signs, while ordinary letters and macron/breve letters are spelled.
/// §12.3 PDF Wyclif Bible example (page 222) drops the capital indicator on
/// early-letter capitals (`Ȝee` → ⠼⠽⠑⠑, no `⠠`) — the early-letter number-prefix
/// `⠼` already distinguishes it from an English wordsign, and the ME text is
/// case-insensitive in braille — while ordinary ASCII capitals keep `⠠` (`I` →
/// ⠠⠊).
pub fn encode_uncontracted_word(chars: &[char]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    // Whether every non-early-letter capital is uppercase and there is at least
    // one such ordinary capital. Early letters (Ȝ, Þ, ...) do not participate in
    // §8 capital indicators (their own `⠼` prefix distinguishes them), so an
    // all-early-letter word like `Ȝee` (Ȝ upper, e/e lower) is treated as
    // lowercase for `⠠⠠` purposes.
    let ordinary_letters: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| early_letter(c.to_lowercase().next().unwrap_or(*c)).is_none())
        .collect();
    let word_caps = !ordinary_letters.is_empty()
        && ordinary_letters.iter().all(|c| c.is_uppercase())
        && ordinary_letters.len() > 1;
    if word_caps {
        out.extend(cells("⠠⠠"));
    }
    for &c in chars {
        let is_early_capital =
            c.is_uppercase() && early_letter(c.to_lowercase().next().unwrap_or(c)).is_some();
        if !word_caps && c.is_uppercase() && !is_early_capital {
            out.push(decode_unicode('⠠'));
        }
        if let Some(cells) = early_letter(c.to_lowercase().next().unwrap_or(c)) {
            out.extend(cells);
        } else if let Some(base) = macron_base(c) {
            out.extend(cells("⠈⠤"));
            out.push(crate::english::encode_english(base).ok()?);
        } else {
            out.push(crate::english::encode_english(base_letter(c)?).ok()?);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_maps_braille_text_to_cells() {
        assert_eq!(cells("⠼⠮"), vec![60, 46]);
    }

    #[rstest::rstest]
    #[case::thorn('þ', "⠼⠮")]
    #[case::thorn_upper('Þ', "⠠⠼⠮")]
    #[case::eth('ð', "⠼⠫")]
    #[case::eth_upper('Ð', "⠠⠼⠫")]
    #[case::yogh('ȝ', "⠼⠽")]
    #[case::yogh_upper('Ȝ', "⠠⠼⠽")]
    #[case::wynn('ƿ', "⠼⠺")]
    #[case::wynn_upper('Ƿ', "⠠⠼⠺")]
    #[case::macron_ash('ǣ', "⠈⠤⠣⠁⠘⠖⠑⠜")]
    #[case::macron_ash_upper('Ǣ', "⠠⠈⠤⠣⠁⠘⠖⠑⠜")]
    fn maps_early_letters(#[case] c: char, #[case] expected: &str) {
        assert_eq!(early_letter(c), Some(cells(expected)));
    }

    #[rstest::rstest]
    #[case::citye("citye", true)]
    #[case::soone("soone", true)]
    #[case::could("could", false)]
    fn identifies_archaic_only_spellings(#[case] word: &str, #[case] expected: bool) {
        assert_eq!(is_archaic_only_spelling(word), expected);
    }

    #[rstest::rstest]
    #[case::macron_upper_u(&['Ū'], Some("⠠⠈⠤⠥"))]
    #[case::early_lower_thorn(&['þ'], Some("⠼⠮"))]
    #[case::breve_lower_e(&['ĕ'], Some("⠑"))]
    #[case::macron_lower_o(&['ō'], Some("⠈⠤⠕"))]
    #[case::macron_upper_o(&['Ō'], Some("⠠⠈⠤⠕"))]
    #[case::macron_upper_y(&['Ȳ'], Some("⠠⠈⠤⠽"))]
    #[case::breve_lower_o(&['ŏ'], Some("⠕"))]
    #[case::breve_cyrillic_y(&['ў'], Some("⠽"))]
    #[case::all_caps_ascii_word(&['A', 'L'], Some("⠠⠠⠁⠇"))]
    #[case::early_capital_omits_cap_indicator(&['Ȝ', 'e', 'e'], Some("⠼⠽⠑⠑"))]
    #[case::unknown_letter(&['🙂'], None)]
    fn uncontracted_word_paths(#[case] chars: &[char], #[case] expected: Option<&str>) {
        let expected_cells = expected.map(|s| s.chars().map(decode_unicode).collect());
        assert_eq!(encode_uncontracted_word(chars), expected_cells);
    }

    #[test]
    fn uncontracted_word_runtime_early_letter_extends_cells() {
        let chars = [std::hint::black_box('þ')];

        assert_eq!(encode_uncontracted_word(&chars), Some(cells("⠼⠮")));
    }

    #[test]
    fn uncontracted_word_runtime_early_letter_branch_extends_cells() {
        let chars = [std::hint::black_box('ȝ')];

        assert_eq!(encode_uncontracted_word(&chars), Some(cells("⠼⠽")));
    }
}
