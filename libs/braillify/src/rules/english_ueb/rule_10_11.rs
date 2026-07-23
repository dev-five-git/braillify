//! §10.11 — strong groupsigns must not bridge a compound-word boundary
//! (feature `english_ueb_cmudict`).
//!
//! In a closed compound, a strong digraph groupsign whose two letters fall in
//! different components spells out: `cart·horse` (t ends *cart*, h starts
//! *horse*) → `⠉⠜⠞⠓⠕⠗⠎⠑`, not `⠉⠜⠹⠕⠗⠎⠑`. Single-morpheme words keep the
//! contraction: `father`, `panther`, `heathen` all contract `th`.
//!
//! The boundary is detected with Knuth-Liang hyphenation — the same mechanism
//! the reference UEB translator liblouis uses (`nocross`): a syllable break
//! between the two letters marks the morpheme boundary. Restricted to the
//! digraphs `th`/`wh`/`sh`, whose English realisation is a single sound, so
//! hyphenation splits them essentially only at a morpheme boundary
//! (`father`→`fa·ther` keeps `th` together; `sweet·heart`, `mis·handle` split).
//! `th`/`wh` measured zero false positives across CMUdict; `sh` splits are
//! overwhelmingly compounds/prefixed forms (`mis-`, `dis-`), the rare obscure
//! surname falling on the safe spell-out side. Consonant clusters such as `st`
//! break at ordinary syllable boundaries (`mas·ter`) and are deliberately NOT
//! gated.

use std::sync::OnceLock;

use hyphenation::{Hyphenator, Language, Load, Standard};

use super::contraction::{ContractionMatch, ContractionRule};
use super::rule_10_4::StrongGroupsignRule;

/// Lazily-loaded embedded en-US hyphenation dictionary, shared across calls.
fn dictionary() -> Option<&'static Standard> {
    static DICT: OnceLock<Option<Standard>> = OnceLock::new();
    DICT.get_or_init(|| Standard::from_embedded(Language::EnglishUS).ok())
        .as_ref()
}

/// Whether hyphenation places a syllable break between `word[pos]` and
/// `word[pos + 1]` — i.e. the two letters cross a morpheme boundary. The crate
/// returns byte offsets; for an all-ASCII letter word those equal char indices.
fn splits_between(word: &[char], pos: usize) -> bool {
    let s: String = word.iter().collect();
    if !s.is_ascii() {
        return false;
    }
    dictionary().is_some_and(|d| d.hyphenate(&s).breaks.contains(&(pos + 1)))
}

/// A strong digraph whose two letters represent a single sound, so a
/// hyphenation break between them reliably marks a compound boundary (§10.11).
/// `gh` joins `th`/`wh`/`sh` on the same basis: within a morpheme it is
/// a single sound (`high`/`night`, `power`/`vowel` — hyphenation keeps the pair
/// together), so a break between the two letters marks a compound seam
/// (`shang·hai`). `ow` is deliberately excluded: hyphenation splits legitimate
/// mid-morpheme `to·ward`, but RUEB 2024 §10.4.1 explicitly contracts it.
/// `ar` was likewise rejected: it suppresses legitimate mid-morpheme contractions.
fn is_bridging_digraph(a: char, b: char) -> bool {
    matches!((a, b), ('t', 'h') | ('w', 'h') | ('s', 'h') | ('g', 'h'))
}

/// §10.11-aware strong groupsign rule: identical to [`StrongGroupsignRule`]
/// except a `th`/`wh` digraph that bridges a compound boundary (hyphenation
/// splits its two letters) is left to spell out.
pub struct BridgeAwareStrongGroupsignRule;

impl ContractionRule for BridgeAwareStrongGroupsignRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let m = StrongGroupsignRule.try_match(word, pos)?;
        if m.consumed == 2
            && is_bridging_digraph(word[pos], word[pos + 1])
            && splits_between(word, pos)
        {
            return None;
        }
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    fn try_at(word: &str, pos: usize) -> Option<(Vec<u8>, usize)> {
        let chars: Vec<char> = word.chars().collect();
        BridgeAwareStrongGroupsignRule
            .try_match(&chars, pos)
            .map(|m| (m.cells, m.consumed))
    }

    /// Compound boundary → `th`/`wh`/`sh` spells out (suppressed).
    #[rstest::rstest]
    #[case::sweetheart("sweetheart", 4)] // sweet|heart (th)
    #[case::lighthouse("lighthouse", 4)] // light|house (th)
    #[case::mishandle("mishandle", 2)] // mis|handle (sh)
    fn bridging_digraph_suppressed(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    /// §10.4.1: `toward` keeps `ow`; a hyphenation break alone is not enough for
    /// `ow` because the rulebook contracts this exact mid-morpheme case.
    #[test]
    fn ow_contracts_in_toward() {
        assert_eq!(try_at("toward", 1), Some((vec![decode_unicode('⠪')], 2)));
    }

    /// `sh` in a single morpheme still contracts (⠩): `bish·op` keeps it together.
    #[test]
    fn sh_contracts_in_single_morpheme() {
        assert_eq!(try_at("bishop", 2), Some((vec![decode_unicode('⠩')], 2)));
    }

    /// Single morpheme → `th` still contracts (⠹), even though `pant`/`heat`/
    /// `fat` + `her`/`hen` are coincidental word splits.
    #[rstest::rstest]
    #[case::father("father", 2)]
    #[case::panther("panther", 3)]
    #[case::heathen("heathen", 3)]
    fn digraph_contracts_in_single_morpheme(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), Some((vec![decode_unicode('⠹')], 2)));
    }

    /// Non-digraph strong groupsigns are unaffected even at a syllable break:
    /// `st` in `mas·ter` splits but is not bridging-gated, so it still contracts.
    #[test]
    fn cluster_groupsign_unaffected() {
        assert_eq!(try_at("master", 2), Some((vec![decode_unicode('⠌')], 2)));
    }

    #[test]
    fn non_ascii_word_cannot_split_as_compound_bridge() {
        assert!(!splits_between(&['c', 'a', 'f', 'é'], 2));
    }
}
