//! UEB §10.13 word division at braille line endings.

use crate::unicode::decode_unicode;

/// Braille line-break division metadata for a word originally containing `\n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordDivision {
    /// Character index in the lowercased logical word where the braille line breaks.
    pub index: usize,
}

/// §10.13 Note: the braille line ends at the division point, so the output
/// carries a literal line break (cell 255 → `\n`) after the hyphen.
pub const LINE_BREAK_CELLS: [u8; 1] = [255];

/// §10.13.1: an originally unhyphenated divided word gets a hyphen before EOL.
pub const ADDED_HYPHEN: u8 = decode_unicode('⠤');

impl WordDivision {
    /// §10.13.1: contractions and groupsigns must not span this division point.
    pub const fn blocks_span(self, pos: usize, consumed: usize) -> bool {
        pos < self.index && self.index < pos + consumed
    }

    /// §10.13.4: `ing` is spelled `in`+`g` at the start of the second line.
    pub fn blocks_initial_ing(self, word: &[char], pos: usize, consumed: usize) -> bool {
        pos == self.index && consumed == 3 && word.get(pos..pos + 3) == Some(&['i', 'n', 'g'])
    }

    /// §10.13.9: `be`/`con`/`dis` are not contracted immediately before the added
    /// hyphen or at the start of the second line.
    pub fn blocks_restricted_lower(self, word: &[char], pos: usize, consumed: usize) -> bool {
        (pos + consumed == self.index || pos == self.index)
            && matches_letters(word, pos, consumed, &["be", "con", "dis"])
    }

    /// §10.13.10: `ea`/`bb`/`cc`/`ff`/`gg` are not contracted immediately before
    /// the added hyphen or at the start of the second line.
    pub fn blocks_middle_lower(self, word: &[char], pos: usize, consumed: usize) -> bool {
        (pos + consumed == self.index || pos == self.index)
            && matches_letters(word, pos, consumed, &["ea", "bb", "cc", "ff", "gg"])
    }

    /// §10.13.11: final-letter groupsigns are not used at the start of line two.
    pub const fn blocks_final_letter(self, pos: usize, consumed: usize) -> bool {
        pos == self.index && consumed >= 3
    }

    /// §10.13.5: if a lower-sign sequence beside the division has no upper-dot sign,
    /// the final lower groupsign in that sequence is not used.
    pub fn blocks_lower_sequence(
        self,
        word: &[char],
        pos: usize,
        consumed: usize,
        cells: &[u8],
        first_line_has_upper_prefix: bool,
    ) -> bool {
        if !cells.iter().all(|cell| is_lower_cell(*cell)) {
            return false;
        }
        if pos + consumed == self.index {
            let prefix_is_lower_contraction = word
                .get(..pos)
                .is_some_and(|prefix| matches!(prefix, ['d', 'i', 's'] | ['e', 'n'] | ['i', 'n']));
            return !first_line_has_upper_prefix && (pos == 0 || prefix_is_lower_contraction);
        }
        if pos > self.index {
            if pos + consumed == word.len()
                && word
                    .get(self.index..pos)
                    .is_some_and(|prefix| prefix == ['e', 'n'])
            {
                return true;
            }
            let second_line_has_upper = word[self.index..pos]
                .iter()
                .chain(word.get(pos + consumed..).into_iter().flatten())
                .any(|c| letter_has_upper_dot(*c));
            if !second_line_has_upper {
                return true;
            }
        }
        pos == self.index
            && !word
                .get(pos + consumed)
                .is_some_and(|c| letter_has_upper_dot(*c))
    }
}

/// §10.13.1 Note: emit the line break, optionally with the added hyphen.
pub fn append_break(out: &mut Vec<u8>, add_hyphen: bool) {
    if add_hyphen {
        out.push(ADDED_HYPHEN);
    }
    out.extend(LINE_BREAK_CELLS);
}

fn matches_letters(word: &[char], pos: usize, consumed: usize, needles: &[&str]) -> bool {
    needles.iter().any(|needle| {
        needle.chars().count() == consumed
            && needle
                .chars()
                .zip(&word[pos..])
                .all(|(expected, actual)| expected == *actual)
    })
}

const fn is_lower_cell(cell: u8) -> bool {
    cell & 0b1001 == 0
}

fn letter_has_upper_dot(ch: char) -> bool {
    crate::english::encode_english(ch).is_ok_and(|cell| !is_lower_cell(cell))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::spanning(2, 1, 3, true)]
    #[case::ends_at_break(2, 0, 2, false)]
    #[case::starts_at_break(2, 2, 2, false)]
    fn blocks_only_spanning_matches(
        #[case] break_at: usize,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] expected: bool,
    ) {
        assert_eq!(
            WordDivision { index: break_at }.blocks_span(pos, consumed),
            expected
        );
    }

    #[rstest::rstest]
    #[case::ing_at_line_start("nightingale", 5, 3, true)]
    #[case::ing_before_line_start("nightingale", 4, 3, false)]
    #[case::plain_in_at_line_start("in", 0, 2, false)]
    fn section_10_13_4_blocks_initial_ing(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            WordDivision { index: pos }.blocks_initial_ing(&chars, pos, consumed),
            expected
        );
    }

    #[rstest::rstest]
    #[case::dis_before_added_hyphen("disgusting", 0, 3, 3, true)]
    #[case::con_after_added_hyphen("bacon", 2, 3, 2, true)]
    #[case::ordinary_pair("bacon", 0, 2, 2, false)]
    fn section_10_13_9_blocks_restricted_lower_groupsigns(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] break_at: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            WordDivision { index: break_at }.blocks_restricted_lower(&chars, pos, consumed),
            expected
        );
    }

    #[rstest::rstest]
    #[case::ea_before_added_hyphen("peanut", 1, 2, 3, true)]
    #[case::bb_after_added_hyphen("abbot", 1, 2, 1, true)]
    #[case::ordinary_pair("peanut", 0, 2, 2, false)]
    fn section_10_13_10_blocks_middle_lower_groupsigns(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] break_at: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            WordDivision { index: break_at }.blocks_middle_lower(&chars, pos, consumed),
            expected
        );
    }

    #[rstest::rstest]
    #[case::long_match_at_second_line_start(3, 3, true)]
    #[case::short_match_at_second_line_start(3, 2, false)]
    #[case::long_match_before_second_line(2, 3, false)]
    fn section_10_13_11_blocks_final_letter_at_line_start(
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] expected: bool,
    ) {
        assert_eq!(
            WordDivision { index: 3 }.blocks_final_letter(pos, consumed),
            expected
        );
    }

    #[rstest::rstest]
    #[case::with_added_hyphen(true, "⠤\n")]
    #[case::without_added_hyphen(false, "\n")]
    fn append_break_emits_optional_added_hyphen(#[case] add_hyphen: bool, #[case] expected: &str) {
        let mut cells = Vec::new();
        append_break(&mut cells, add_hyphen);
        let expected_cells: Vec<u8> = expected
            .chars()
            .map(|c| if c == '\n' { 255 } else { decode_unicode(c) })
            .collect();
        assert_eq!(cells, expected_cells);
    }

    #[rstest::rstest]
    #[case::shortenin_final_in("shortenin", 7, 2, 5, &[decode_unicode('⠔')], true)]
    #[case::linen_initial_en("linen", 3, 2, 3, &[decode_unicode('⠢')], true)]
    #[case::disinherit_has_upper_after("disinherit", 0, 3, 5, &[decode_unicode('⠲')], false)]
    fn section_10_13_5_blocks_lower_sequence_without_upper_dot(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] break_at: usize,
        #[case] cells: &[u8],
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            WordDivision { index: break_at }
                .blocks_lower_sequence(&chars, pos, consumed, cells, false,),
            expected
        );
    }

    #[rstest::rstest]
    #[case::non_lower_cell("about", 0, 2, 2, &[decode_unicode('⠁')], false)]
    #[case::first_line_has_upper_prefix("shortenin", 7, 2, 9, &[decode_unicode('⠔')], false)]
    #[case::line_start_followed_by_upper_dot_letter("begin", 2, 2, 2, &[decode_unicode('⠢')], false)]
    #[case::second_line_without_upper_dot("a--", 2, 1, 1, &[decode_unicode('⠆')], true)]
    #[case::second_line_with_upper_dot("abbbc", 3, 2, 1, &[decode_unicode('⠆')], false)]
    fn lower_sequence_non_blocking_paths(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] consumed: usize,
        #[case] break_at: usize,
        #[case] cells: &[u8],
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            WordDivision { index: break_at }
                .blocks_lower_sequence(&chars, pos, consumed, cells, true,),
            expected
        );
    }
}
