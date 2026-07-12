//! §16.2 Horizontal line mode.
//!
//! RUEB 2024 §16.2: a horizontal "line" drawn with box-drawing characters opens
//! with the horizontal line mode indicator `⠐⠒` and is then built from segment,
//! corner and crossing cells. The indicator's `⠒` is itself a simple segment, so
//! a line that *starts* with a solid segment (`─`) folds that first segment into
//! the indicator (§16.2.1, page 230 examples). Each subsequent box character maps
//! to one cell:
//!
//! | print            | cell | meaning (§16.2)                          |
//! |------------------|------|------------------------------------------|
//! | `─` U+2500       | `⠒`  | simple (solid single) segment            |
//! | spaced `─`       | `⠂`  | variant (dotted/dashed) segment          |
//! | `═` U+2550       | `⠶`  | double segment                           |
//! | `≡` U+2261       | `⠿`  | triple segment                           |
//! | `┴` U+2534       | `⠚`  | corner with upward vertical              |
//! | `┐` U+2510       | `⠲`  | corner with downward vertical            |
//! | `┼` U+253C       | `⠺`  | crossing with vertical line              |
//! | `╲` U+2572       | `⠣`  | crossing with left-leaning diagonal      |
//! | `╱` U+2571       | `⠜`  | crossing with right-leaning diagonal     |
//!
//! Only a *run* of two or more adjacent box characters is line mode; an isolated
//! one (a lone `─` rule, a mathematical `≡`) is left to the symbol/legacy path so
//! the math meaning is preserved.

use crate::unicode::decode_unicode;

/// The §16.2 horizontal-line cell for a box-drawing character, or `None`.
pub fn line_segment(c: char) -> Option<u8> {
    Some(match c {
        '\u{2500}' | '\u{250C}' | '\u{2514}' => decode_unicode('⠒'),
        '\u{2550}' => decode_unicode('⠶'),
        '\u{2261}' => decode_unicode('⠿'),
        '\u{2502}' | '\u{251C}' => decode_unicode('⠸'),
        '\u{250A}' => decode_unicode('⠘'),
        '\u{2534}' | '\u{2518}' => decode_unicode('⠚'),
        '\u{2510}' | '\u{252C}' => decode_unicode('⠲'),
        '\u{253C}' => decode_unicode('⠺'),
        '\u{2572}' => decode_unicode('⠣'),
        '\u{2571}' | '\u{2573}' => decode_unicode('⠜'),
        '▔' => decode_unicode('⠉'),
        '▁' => decode_unicode('⠤'),
        // §16.2.4 distinctive line features (multi-cell forms have their own
        // `line_marker_cells` path below; the fall-through `⠯` here is retained
        // as a single-cell approximation for pure `▭` glyphs outside a line).
        '▭' => decode_unicode('⠯'),
        _ => return None,
    })
}

/// §16.2.4: distinctive line-feature markers whose braille form is more than one
/// cell — the rectangle `▭` renders as `⠯⠭⠭⠭⠽` (open + fill + close) in the
/// middle of a horizontal line, per the PDF page 232 example
/// `⠐⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠯⠭⠭⠭⠽⠒⠒`.
pub fn line_marker_cells(c: char) -> Option<Vec<u8>> {
    Some(match c {
        '▭' => vec![
            decode_unicode('⠯'),
            decode_unicode('⠭'),
            decode_unicode('⠭'),
            decode_unicode('⠭'),
            decode_unicode('⠽'),
        ],
        _ => return None,
    })
}

/// §16.3–§16.4 spatial vertical/diagonal symbols outside horizontal line mode.
pub fn spatial_symbol(c: char) -> Option<Vec<u8>> {
    Some(match c {
        '\u{2502}' => vec![decode_unicode('⠸')],
        '\u{250A}' => vec![decode_unicode('⠘')],
        '\u{2572}' => vec![decode_unicode('⠣')],
        '\u{2571}' => vec![decode_unicode('⠜')],
        '←' => vec![decode_unicode('⠳'), decode_unicode('⠪')],
        '↙' => vec![decode_unicode('⠳'), decode_unicode('⠜')],
        '↗' => vec![decode_unicode('⠳'), decode_unicode('⠎')],
        _ => return None,
    })
}

/// Whether `c` is a §16.3 vertical or diagonal segment outside horizontal mode.
pub fn is_spatial_segment(c: char) -> bool {
    matches!(
        c,
        '\u{2502}' | '\u{250A}' | '\u{2572}' | '\u{2571}' | '\u{2573}'
    )
}

/// §16.4 arrow cells when an arrow is continuous with a line drawing.
pub fn line_arrow(c: char) -> Option<Vec<u8>> {
    Some(match c {
        '→' => vec![decode_unicode('⠳'), decode_unicode('⠕')],
        '↓' => vec![decode_unicode('⠳'), decode_unicode('⠩')],
        '←' => vec![decode_unicode('⠳'), decode_unicode('⠪')],
        '↙' => vec![decode_unicode('⠳'), decode_unicode('⠜')],
        '↗' => vec![decode_unicode('⠳'), decode_unicode('⠎')],
        _ => return None,
    })
}

/// Whether `c` is a §16.2 horizontal box-drawing character.
pub fn is_line_char(c: char) -> bool {
    line_segment(c).is_some()
}

/// The solid single horizontal segment `─`, folded into the `⠐⠒` indicator.
pub const SIMPLE_SEGMENT: char = '\u{2500}';

/// §16.2.2: print dashed horizontal lines can be represented as spaced dashes;
/// in line mode they become variant horizontal line segments (`⠂`).
pub const VARIANT_SPACED_SEGMENT: char = '\u{2500}';

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::simple('\u{2500}', '⠒')]
    #[case::upper_left_corner('\u{250C}', '⠒')]
    #[case::lower_left_corner('\u{2514}', '⠒')]
    #[case::double('\u{2550}', '⠶')]
    #[case::triple('\u{2261}', '⠿')]
    #[case::vertical('\u{2502}', '⠸')]
    #[case::left_tee('\u{251C}', '⠸')]
    #[case::dotted_vertical('\u{250A}', '⠘')]
    #[case::corner_up('\u{2534}', '⠚')]
    #[case::lower_right_corner('\u{2518}', '⠚')]
    #[case::corner_down('\u{2510}', '⠲')]
    #[case::top_tee('\u{252C}', '⠲')]
    #[case::crossing_vertical('\u{253C}', '⠺')]
    #[case::crossing_left('\u{2572}', '⠣')]
    #[case::crossing_right('\u{2571}', '⠜')]
    #[case::diagonal_cross('\u{2573}', '⠜')]
    #[case::overline('▔', '⠉')]
    #[case::underline('▁', '⠤')]
    #[case::rectangle_marker('▭', '⠯')]
    fn line_segments_map(#[case] c: char, #[case] cell: char) {
        assert_eq!(line_segment(c), Some(decode_unicode(cell)));
        assert!(is_line_char(c));
    }

    #[test]
    fn rectangle_marker_expands_inside_horizontal_line() {
        assert_eq!(
            line_marker_cells('▭'),
            Some("⠯⠭⠭⠭⠽".chars().map(decode_unicode).collect())
        );
        assert_eq!(line_marker_cells('─'), None);
    }

    #[rstest::rstest]
    #[case::vertical('\u{2502}', "⠸")]
    #[case::dotted_vertical('\u{250A}', "⠘")]
    #[case::left_diagonal('\u{2572}', "⠣")]
    #[case::right_diagonal('\u{2571}', "⠜")]
    #[case::left_arrow('←', "⠳⠪")]
    #[case::down_left_arrow('↙', "⠳⠜")]
    #[case::up_right_arrow('↗', "⠳⠎")]
    fn spatial_symbols_map(#[case] c: char, #[case] expected: &str) {
        assert_eq!(
            spatial_symbol(c),
            Some(expected.chars().map(decode_unicode).collect())
        );
    }

    #[test]
    fn spatial_symbol_accepts_runtime_vertical_segment() {
        let c = std::hint::black_box('\u{2502}');

        assert_eq!(spatial_symbol(c), Some(vec![decode_unicode('⠸')]));
    }

    #[rstest::rstest]
    #[case::right_arrow('→', "⠳⠕")]
    #[case::down_arrow('↓', "⠳⠩")]
    #[case::left_arrow('←', "⠳⠪")]
    #[case::down_left_arrow('↙', "⠳⠜")]
    #[case::up_right_arrow('↗', "⠳⠎")]
    fn line_arrows_map(#[case] c: char, #[case] expected: &str) {
        assert_eq!(
            line_arrow(c),
            Some(expected.chars().map(decode_unicode).collect())
        );
    }

    #[rstest::rstest]
    #[case::vertical('\u{2502}', true)]
    #[case::dotted_vertical('\u{250A}', true)]
    #[case::left_diagonal('\u{2572}', true)]
    #[case::right_diagonal('\u{2571}', true)]
    #[case::diagonal_cross('\u{2573}', true)]
    #[case::horizontal('\u{2500}', false)]
    #[case::letter('x', false)]
    fn spatial_segment_predicate_matches_vertical_and_diagonal_segments(
        #[case] c: char,
        #[case] expected: bool,
    ) {
        assert_eq!(is_spatial_segment(c), expected);
    }

    #[test]
    fn unknown_line_arrow_returns_none() {
        assert_eq!(line_arrow('x'), None);
    }

    #[rstest::rstest]
    #[case::letter('a')]
    #[case::digit('5')]
    #[case::space(' ')]
    #[case::equals('=')]
    fn non_line_chars(#[case] c: char) {
        assert_eq!(line_segment(c), None);
        assert!(!is_line_char(c));
    }
}
