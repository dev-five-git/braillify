//! Korean-context English *cell* production (제28/37항) — single source of truth.
//!
//! Produces the braille cells for English letters embedded in Korean text. The
//! Korean engine (`emit.rs` + 제29항 orchestration) owns the 로마자표 ⠴ / 종료표 ⠲
//! / 연속표 ⠰ markers and mode transitions; THIS module owns only the
//! letter/contraction cells, so every English point is produced by the UEB rule
//! modules (`rule_10_*`, §28 alphabet via [`crate::english`]) rather than a
//! parallel legacy table.
//!
//! Contractions are 제37항-RESTRICTED (not full UEB Grade-2 standing-alone
//! wordsigns): the gate in [`super::korean_context`] decides which §10.x
//! contractions apply in Korean context. A position that starts no restricted
//! contraction falls back to the §28 single-letter cell.

use super::korean_context::{KoreanPrefixInput, match_korean_prefix};
use crate::english::encode_english;

/// One unit of Korean-context English output: a 제37항-restricted UEB contraction
/// when one begins at `input.pos`, otherwise the single §28 letter cell.
pub(crate) struct KoreanSpanUnit {
    /// Braille cells for this unit.
    pub(crate) cells: Vec<u8>,
    /// Source characters consumed (always ≥ 1).
    pub(crate) consumed: usize,
    /// `true` when a 제37항 contraction matched (may span multiple chars); `false`
    /// for the §28 single-letter fallback. The caller advances its skip counter
    /// only for contractions — the single-letter case leaves the counter as-is,
    /// matching the legacy [`super::super::korean::rule_28`] branch structure.
    pub(crate) contracted: bool,
}

/// Encode the English unit beginning at `input.pos` to UEB cells.
///
/// Returns `Err` only when the position is not an encodable English letter
/// (mirrors [`encode_english`]); callers in Korean context guarantee
/// `input.word[input.pos]` is ASCII alphabetic.
pub(crate) fn encode_korean_unit(input: KoreanPrefixInput<'_>) -> Result<KoreanSpanUnit, String> {
    let letter = input.word[input.pos];
    match match_korean_prefix(input) {
        Some(matched) => Ok(KoreanSpanUnit {
            cells: matched.cells,
            consumed: matched.consumed,
            contracted: true,
        }),
        None => Ok(KoreanSpanUnit {
            cells: vec![encode_english(letter)?],
            consumed: 1,
            contracted: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    fn unit(word: &str, pos: usize, wrap_active: bool) -> KoreanSpanUnit {
        let chars: Vec<char> = word.chars().collect();
        encode_korean_unit(KoreanPrefixInput {
            word: &chars,
            pos,
            wrap_active,
            is_all_uppercase: false,
            at_entry: pos == 0,
            standalone_wordsign: false,
        })
        .unwrap()
    }

    /// A position with no restricted contraction falls back to the §28 letter cell.
    #[rstest::rstest]
    #[case::plain_a("cat", 0, decode_unicode('⠉'), 1)]
    #[case::plain_t("cat", 2, decode_unicode('⠞'), 1)]
    fn falls_back_to_single_letter(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected_cell: u8,
        #[case] consumed: usize,
    ) {
        let u = unit(word, pos, false);
        assert_eq!(u.cells, vec![expected_cell]);
        assert_eq!(u.consumed, consumed);
    }

    /// A 제37항-restricted contraction is produced via `korean_context` — e.g. the
    /// §10.8 final groupsign `ong` (⠰⠛) inside `pyeongchang`.
    #[test]
    fn restricted_contraction_ong() {
        let u = unit("pyeongchang", 3, false);
        assert_eq!(u.cells, vec![decode_unicode('⠰'), decode_unicode('⠛')]);
        assert_eq!(u.consumed, 3);
    }
}
