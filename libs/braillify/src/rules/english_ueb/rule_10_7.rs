//! §10.7 Initial-letter contractions.
//!
//! Two-cell strong contractions standing for whole words, formed from a prefix
//! cell (`⠐` dot-5, `⠨` dots-4-5, or `⠸` dots-4-5-6) plus the word's first
//! letter. Per RUEB 2024 §10.7 they may be used wherever the letters occur —
//! including word-initial and medial (`acknowledge` → `know`, `apartheid` →
//! `part`). Longest match wins so `character` (9) beats `ch` at the same spot.
//!
//! ONLY the rare, unambiguous letter sequences are applied mechanically here.
//! The contractions whose letters frequently occur INSIDE unrelated words with
//! a different sound (`one` in *component*, `here` in *adhered*, `ever` in
//! *persevere*, `time` in *centimetre*, `some`, `name`, `part`, `had`, `under`,
//! `day`, `there`, `where`, `know`, `these`, `those`, `lord`,
//! `mother`, `father`, `young`, `their`) are pronunciation- and
//! morphology-dependent (§10.7 "retain the pronunciation" + §10.11 bridging).
//! A naive longest-match over those mis-contracts ~120 words, so they are
//! deferred to a pronunciation/morphology-gated pass rather than guessed.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule};
use crate::unicode::decode_unicode;

/// Whole-word letter sequence → its two braille cells (prefix + first letter).
/// Restricted to sequences that essentially never occur spuriously, so plain
/// longest-match is safe (see module note for the deferred set).
static INITIAL_CONTRACTIONS: phf::Map<&'static str, [u8; 2]> = phf_map! {
    // ⠐ (dot 5) prefix.
    "right"     => [decode_unicode('⠐'), decode_unicode('⠗')],
    "question"  => [decode_unicode('⠐'), decode_unicode('⠟')],
    "character" => [decode_unicode('⠐'), decode_unicode('⠡')],
    "through"   => [decode_unicode('⠐'), decode_unicode('⠹')],
    "ought"     => [decode_unicode('⠐'), decode_unicode('⠳')],
    // ⠘ (dots 4-5) prefix. (`upon` omitted — it occurs inside `Dupont`,
    // `coupon` with a different sound; deferred to the morphology-gated pass.)
    "whose"     => [decode_unicode('⠘'), decode_unicode('⠱')],
    // ⠘ (dots 4-5) prefix. `word` is unambiguous: every corpus occurrence
    // (`sword`, `foreword`, `misworded`, `Wordsworth`, `word-for-word`) is the
    // `word` morpheme, so spelling-match is safe (no spell-out exception exists).
    "word"      => [decode_unicode('⠘'), decode_unicode('⠺')],
    // ⠸ (dots 4-5-6) prefix. `many` is likewise unambiguous (`Germany`,
    // `many-sided`) — the `m·a·n·y` run never occurs with a different sound.
    "cannot"    => [decode_unicode('⠸'), decode_unicode('⠉')],
    "spirit"    => [decode_unicode('⠸'), decode_unicode('⠎')],
    "world"     => [decode_unicode('⠸'), decode_unicode('⠺')],
    "many"      => [decode_unicode('⠸'), decode_unicode('⠍')],
};

/// §10.7 initial-letter contraction rule.
pub struct InitialContractionRule;

impl ContractionRule for InitialContractionRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let mut best: Option<(usize, [u8; 2])> = None;
        for (key, &cells) in INITIAL_CONTRACTIONS.entries() {
            let klen = key.chars().count();
            if pos + klen <= word.len()
                && key
                    .chars()
                    .zip(&word[pos..pos + klen])
                    .all(|(k, w)| k == *w)
                && best.is_none_or(|(bl, _)| klen > bl)
            {
                best = Some((klen, cells));
            }
        }
        best.map(|(klen, cells)| ContractionMatch {
            cells: cells.to_vec(),
            consumed: klen,
            priority: 55,
            protect_span: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::right_mid("brighten", 1, Some((vec![decode_unicode('⠐'), decode_unicode('⠗')], 5)))]
    #[case::spirit_mid("dispirited", 2, Some((vec![decode_unicode('⠸'), decode_unicode('⠎')], 6)))]
    #[case::cannot("cannot", 0, Some((vec![decode_unicode('⠸'), decode_unicode('⠉')], 6)))]
    #[case::character("characterise", 0, Some((vec![decode_unicode('⠐'), decode_unicode('⠡')], 9)))]
    #[case::through("throughout", 0, Some((vec![decode_unicode('⠐'), decode_unicode('⠹')], 7)))]
    #[case::no_match("cat", 0, None)]
    fn matches_initial_contractions(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(Vec<u8>, usize)>,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = InitialContractionRule
            .try_match(&chars, pos)
            .map(|m| (m.cells, m.consumed));
        assert_eq!(got, expected);
    }
}
