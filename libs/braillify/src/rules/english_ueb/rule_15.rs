//! §15 Scansion, Stress and Tone.

use crate::unicode::decode_unicode;

fn cells(s: &str) -> Vec<u8> {
    s.chars().map(decode_unicode).collect()
}

/// §15.1 scansion marks, §15.2 stress marks/schwa and §15.3 tone marks.
///
/// PDF cell mappings (RUEB 2024 §15, lines 8227-8413):
/// - §15.1.1 `|` → `⠸⠳`, `‖` → `⠸⠳⠸⠳`, `/` → `⠸⠌`
/// - §15.2 primary stress `ˈ`/`″` → `⠘⠨⠃`, secondary `ˌ`/`′` → `⠘⠨⠆`, schwa `ə` → `⠸⠢`
/// - §15.3 tone letters: `↑` up step → `⠘⠨⠫`, `↓` down step → `⠘⠨⠮`, `➘` fall → `⠘⠨⠴`,
///   `ˊ` high rising → `⠘⠨⠊`, `ˎ` low falling → `⠘⠨⠢`, `↺` fall-rise → `⠘⠨⠌`
pub fn encode_symbol(c: char) -> Option<Vec<u8>> {
    Some(match c {
        '|' => cells("⠸⠳"),
        '‖' => cells("⠸⠳⠸⠳"),
        '/' => cells("⠸⠌"),
        'ˈ' => cells("⠘⠨⠃"),
        '′' => cells("⠘⠨⠆"),
        'ˌ' => cells("⠘⠨⠆"),
        '″' => cells("⠘⠨⠃"),
        'ə' => cells("⠸⠢"),
        '➘' => cells("⠘⠨⠴"),
        // §15.3.2 example uses `↗` for low rising in prose (⠘⠨⠔). `ˊ` (modifier
        // acute) is the high-rising tone letter (⠘⠨⠊) — the two arrows share the
        // print glyph shape but differ in tone height per the PDF's own examples.
        '↗' => cells("⠘⠨⠔"),
        'ˊ' => cells("⠘⠨⠊"),
        '↺' => cells("⠘⠨⠌"),
        '↑' => cells("⠘⠨⠫"),
        '↓' => cells("⠘⠨⠮"),
        'ˎ' => cells("⠘⠨⠢"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::primary('ˈ', "⠘⠨⠃")]
    #[case::primary_double_prime('″', "⠘⠨⠃")]
    #[case::secondary('ˌ', "⠘⠨⠆")]
    #[case::secondary_prime('′', "⠘⠨⠆")]
    #[case::schwa('ə', "⠸⠢")]
    #[case::line('|', "⠸⠳")]
    #[case::double_line('‖', "⠸⠳⠸⠳")]
    #[case::scansion_solidus('/', "⠸⠌")]
    #[case::tone_down('↓', "⠘⠨⠮")]
    #[case::tone_fall('➘', "⠘⠨⠴")]
    #[case::tone_low_rising('↗', "⠘⠨⠔")]
    #[case::tone_high_rising('ˊ', "⠘⠨⠊")]
    #[case::tone_fall_rise('↺', "⠘⠨⠌")]
    #[case::tone_up('↑', "⠘⠨⠫")]
    #[case::tone_low_falling('ˎ', "⠘⠨⠢")]
    fn maps_scansion_stress_tone(#[case] c: char, #[case] expected: &str) {
        assert_eq!(encode_symbol(c), Some(cells(expected)));
    }

    #[test]
    fn unknown_symbol_returns_none() {
        assert_eq!(encode_symbol('x'), None);
    }
}
