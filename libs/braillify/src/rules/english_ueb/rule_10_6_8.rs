//! §10.6.8 `en`/`in` vs the `ness` final groupsign (pronunciation-gated).
//!
//! `en` (⠢) and `in` (⠔) may be used anywhere (§10.6), but where they overlap a
//! following `ness` at the shared `n` (`busi⟨in⟩ess`), §10.10 prefers the `ness`
//! final groupsign — *but only when the `n` onsets the `ness` syllable*, i.e. the
//! word is pronounced with a final `N <vowel> S` (`business` /…N AH0 S/, `finesse`
//! /…N EH1 S/, `happiness`, `friendliness`). When the `n` instead *closes* a
//! syllable the `en`/`in` is kept (`citi·zen·ess`, `captain·ess` — the base ends
//! in /n/, the suffix is `-ess`). Those rarer base+`ess` words are absent from
//! CMUdict, so an unknown word defaults to KEEP: a missed suppression is far
//! safer than a wrong contraction. This mirrors the `con` coda/onset test
//! (`con·cept` uses it, `co·ney` does not) but locates the `n` from the word
//! ending rather than a fixed phoneme index.

use super::contraction::{ContractionMatch, ContractionRule};
use super::pronunciation::{Phoneme, PronunciationProvider};
use super::rule_10_6::LowerGroupsignRule;

/// Pronunciation-gated §10.6.8 `en`/`in` rule.
pub struct EnInBeforeNessRule {
    provider: Box<dyn PronunciationProvider>,
}

impl EnInBeforeNessRule {
    /// Build the rule with the pronunciation source used to judge the `n`.
    pub fn new(provider: Box<dyn PronunciationProvider>) -> Self {
        Self { provider }
    }

    /// Whether the shared `n` onsets a `ness` syllable — the whole word is
    /// pronounced with a final `N <vowel> S`, every recorded variant agreeing.
    /// An unknown word yields `false` (keep `en`/`in`).
    fn n_onsets_ness(&self, word: &[char]) -> bool {
        let spelling: String = word.iter().collect();
        let prons = self.provider.pronunciations(&spelling);
        !prons.is_empty() && prons.iter().all(|p| ends_n_vowel_s(p))
    }
}

impl ContractionRule for EnInBeforeNessRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let mut m = LowerGroupsignRule.try_match(word, pos)?;
        // §10.6.8: where `en`/`in` overlaps a following `ness` at the shared `n`,
        // pronunciation decides which is kept.
        if ness_overlaps(word, pos) {
            if self.n_onsets_ness(word) {
                // The `n` onsets the `ness` syllable (`busi·ness`, `fi·ness·e`) →
                // suppress `en`/`in` so the cheaper `ness` final groupsign wins.
                return None;
            }
            // The `n` closes the base syllable (`captain·ess`, `citizen·ess`) →
            // keep `en`/`in` AND protect its span (§10.10.1), so the §10.10.2
            // cell-minimiser cannot pick the overlapping `ness` (`captain·ess`,
            // not `captai·ness`).
            m.protect_span = true;
        }
        Some(m)
    }
}

/// The `en`/`in` at `pos` shares its `n` with a following `ness` (`n e s s`).
fn ness_overlaps(word: &[char], pos: usize) -> bool {
    word.get(pos + 1..pos + 5)
        .is_some_and(|s| s == ['n', 'e', 's', 's'])
}

/// Whether a pronunciation ends `N <vowel> S` — the `/nVs/` of a `ness` suffix.
fn ends_n_vowel_s(p: &[Phoneme]) -> bool {
    let n = p.len();
    n >= 3 && p[n - 3].base == "N" && p[n - 2].is_vowel() && p[n - 1].base == "S"
}

#[cfg(test)]
mod tests {
    use super::super::pronunciation::cmudict::CmuDictProvider;
    use super::*;

    fn rule() -> EnInBeforeNessRule {
        EnInBeforeNessRule::new(Box::new(CmuDictProvider::new()))
    }

    /// `en`/`in` is suppressed (→ `None`, so the `ness` groupsign wins) when the
    /// `n` onsets the `ness` syllable per CMUdict (`busi·ness`, `fi·ness·e`).
    #[rstest::rstest]
    #[case::business("business", 3)]
    #[case::finesse("finesse", 1)]
    #[case::happiness("happiness", 4)]
    #[case::friendliness("friendliness", 7)]
    fn suppresses_en_in_before_onset_ness(#[case] word: &str, #[case] pos: usize) {
        let chars: Vec<char> = word.chars().collect();
        assert!(rule().try_match(&chars, pos).is_none());
    }

    /// `en`/`in` is kept (→ `Some`) when the `n` is a coda — the base ends in /n/
    /// (`citi·zen·ess`, `captain·ess`); these base+`ess` words are absent from
    /// CMUdict, so KEEP by default.
    #[rstest::rstest]
    #[case::citizeness("citizeness", 5)]
    #[case::captainess("captainess", 5)]
    #[case::chieftainess("chieftainess", 7)]
    #[case::inessential("inessential", 0)]
    fn keeps_en_in_when_coda_or_unknown(#[case] word: &str, #[case] pos: usize) {
        let chars: Vec<char> = word.chars().collect();
        assert!(rule().try_match(&chars, pos).is_some());
    }

    /// `en`/`in` with no overlapping `ness` is always kept (`engine`, `arena`).
    #[rstest::rstest]
    #[case::engine("engine", 0)]
    #[case::arena("arena", 2)]
    fn keeps_en_in_without_ness_overlap(#[case] word: &str, #[case] pos: usize) {
        let chars: Vec<char> = word.chars().collect();
        assert!(rule().try_match(&chars, pos).is_some());
    }
}
