//! §10.8 Final-letter groupsigns.
//!
//! Two-cell contractions for word-final letter clusters: a prefix cell (`⠨`
//! dots 4-6, or `⠰` dots 5-6) followed by the cluster's final letter. Per RUEB
//! 2024 §10.8 they are used medially or finally, never at the start of a word,
//! so the rule offers no match at position 0. Longest match wins, so `tion`
//! (4) beats any shorter cluster at the same position.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule};
use crate::unicode::decode_unicode;

/// Cluster → its two braille cells (`⠨`/`⠰` prefix + final letter).
static FINAL_GROUPSIGNS: phf::Map<&'static str, [u8; 2]> = phf_map! {
    // ⠨ (dots 4-6) prefix.
    "ound" => [decode_unicode('⠨'), decode_unicode('⠙')],
    "ance" => [decode_unicode('⠨'), decode_unicode('⠑')],
    "sion" => [decode_unicode('⠨'), decode_unicode('⠝')],
    "less" => [decode_unicode('⠨'), decode_unicode('⠎')],
    "ount" => [decode_unicode('⠨'), decode_unicode('⠞')],
    // ⠰ (dots 5-6) prefix.
    "ence" => [decode_unicode('⠰'), decode_unicode('⠑')],
    "ong"  => [decode_unicode('⠰'), decode_unicode('⠛')],
    "ful"  => [decode_unicode('⠰'), decode_unicode('⠇')],
    "tion" => [decode_unicode('⠰'), decode_unicode('⠝')],
    "ness" => [decode_unicode('⠰'), decode_unicode('⠎')],
    "ment" => [decode_unicode('⠰'), decode_unicode('⠞')],
    "ity"  => [decode_unicode('⠰'), decode_unicode('⠽')],
};

/// Return the cells for a final-letter groupsign cluster.
pub(crate) fn final_groupsign_cells(cluster: &str) -> Option<[u8; 2]> {
    FINAL_GROUPSIGNS.get(cluster).copied()
}

/// RUEB 2024 §10.8.3 printed exceptions for the `ity` final-letter groupsign.
fn ity_exception(word: &[char]) -> bool {
    matches!(
        word,
        ['b', 'i', 's', 'c', 'u', 'i', 't', 'y']
            | ['d', 'a', 'c', 'o', 'i', 't', 'y']
            | ['f', 'r', 'u', 'i', 't', 'y']
            | ['p', 'i', 't', 'y', 'a', 'r', 'd']
            | ['r', 'a', 'b', 'b', 'i', 't', 'y']
    )
}

fn ness_exception(word: &[char]) -> bool {
    matches!(word, ['h', 'e', 'a', 't', 'h', 'e', 'n', 'e', 's', 's'])
}

/// §10.8 final-letter groupsign rule.
pub struct FinalGroupsignRule;

impl ContractionRule for FinalGroupsignRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        // §10.8: never used at the start of a word.
        if pos == 0 {
            return None;
        }
        let mut best: Option<(usize, [u8; 2])> = None;
        for (key, &cells) in FINAL_GROUPSIGNS.entries() {
            let klen = key.chars().count();
            if pos + klen <= word.len()
                && key
                    .chars()
                    .zip(&word[pos..pos + klen])
                    .all(|(k, w)| k == *w)
                && !(*key == "ity" && ity_exception(word))
                && !(*key == "ness" && ness_exception(word))
                && best.is_none_or(|(bl, _)| klen > bl)
            {
                best = Some((klen, cells));
            }
        }
        best.map(|(klen, cells)| ContractionMatch {
            cells: cells.to_vec(),
            consumed: klen,
            priority: 80,
            protect_span: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::tion_final("bastion", 3, Some((vec![decode_unicode('⠰'), decode_unicode('⠝')], 4)))]
    #[case::ment_final("comment", 3, Some((vec![decode_unicode('⠰'), decode_unicode('⠞')], 4)))]
    #[case::ount_mid("amount", 2, Some((vec![decode_unicode('⠨'), decode_unicode('⠞')], 4)))]
    #[case::ness_final("baroness", 4, Some((vec![decode_unicode('⠰'), decode_unicode('⠎')], 4)))]
    #[case::ity_final("circuity", 5, Some((vec![decode_unicode('⠰'), decode_unicode('⠽')], 3)))]
    #[case::no_match_at_start("tion", 0, None)]
    #[case::no_cluster("cat", 1, None)]
    fn matches_final_groupsigns(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(Vec<u8>, usize)>,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = FinalGroupsignRule
            .try_match(&chars, pos)
            .map(|m| (m.cells, m.consumed));
        assert_eq!(got, expected);
    }

    #[test]
    fn runtime_start_position_never_uses_final_groupsign() {
        let word: Vec<char> = std::hint::black_box("tion").chars().collect();
        let pos = std::hint::black_box(0);

        assert!(FinalGroupsignRule.try_match(&word, pos).is_none());
    }
}
