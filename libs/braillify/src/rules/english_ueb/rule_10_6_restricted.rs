//! §10.6 restricted lower groupsigns `be` (⠆), `con` (⠒) and `dis` (⠲).
//!
//! Unlike the unrestricted `en`/`in` ([`super::rule_10_6`]), these may be used
//! only when the prefix forms the first *syllable* of the word — a
//! pronunciation/word-structure decision delegated to
//! [`super::pronunciation::classifier`]. The rule fires only at word start and
//! only on a [`Decision::Use`] verdict; `SpellOut` and `Unknown` leave the
//! letters to be spelled by the fallback path.

use super::contraction::{ContractionMatch, ContractionRule};
use super::pronunciation::PronunciationProvider;
use super::pronunciation::classifier::{Decision, Prefix, classify};
use crate::unicode::decode_unicode;

/// §10.6 restricted groupsign rule, backed by a pronunciation provider.
pub struct RestrictedLowerGroupsignRule {
    provider: Box<dyn PronunciationProvider>,
}

impl RestrictedLowerGroupsignRule {
    /// Build the rule with the pronunciation source used to judge syllables.
    pub fn new(provider: Box<dyn PronunciationProvider>) -> Self {
        Self { provider }
    }
}

impl ContractionRule for RestrictedLowerGroupsignRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        // Restricted groupsigns are word-initial only (§10.6.2).
        if pos != 0 {
            return None;
        }
        let (prefix, consumed, cell) = if word.starts_with(&['b', 'e']) {
            (Prefix::Be, 2, decode_unicode('⠆'))
        } else if word.starts_with(&['c', 'o', 'n']) {
            (Prefix::Con, 3, decode_unicode('⠒'))
        } else if word.starts_with(&['d', 'i', 's']) {
            (Prefix::Dis, 3, decode_unicode('⠲'))
        } else {
            return None;
        };
        match classify(word, prefix, self.provider.as_ref()) {
            Decision::Use => Some(ContractionMatch {
                cells: vec![cell],
                consumed,
                // Below §10.4 strong groupsigns (60) so a longer strong match
                // still wins; ties on length prefer the more specific prefix.
                priority: 65,
                // §10.10.1: syllable-validated, so the cell-minimiser must not
                // split it with a cheaper generic contraction (`con`/`dis`/`be`).
                protect_span: true,
            }),
            Decision::SpellOut | Decision::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::pronunciation::cmudict::CmuDictProvider;
    use super::*;

    fn chars(w: &str) -> Vec<char> {
        w.chars().collect()
    }

    fn rule() -> RestrictedLowerGroupsignRule {
        RestrictedLowerGroupsignRule::new(Box::new(CmuDictProvider::new()))
    }

    /// `become` → ⠆ (2 letters); `concept` → ⠒ (3); `dislike`/`dishonest` → ⠲
    /// (3, remainder is a word); `beckon`/`cone`/`dispirited`/`disc` produce no
    /// match (spelled out).
    #[rstest::rstest]
    #[case::become_word("become", Some((decode_unicode('⠆'), 2)))]
    #[case::concept("concept", Some((decode_unicode('⠒'), 3)))]
    #[case::dislike_rest_word("dislike", Some((decode_unicode('⠲'), 3)))]
    #[case::dishonest_rest_word("dishonest", Some((decode_unicode('⠲'), 3)))]
    #[case::beckon("beckon", None)]
    #[case::cone("cone", None)]
    #[case::dispirited("dispirited", None)]
    #[case::disc_monosyllable("disc", None)]
    #[case::plain_word("cat", None)]
    fn matches_restricted_groupsigns(#[case] word: &str, #[case] expected: Option<(u8, usize)>) {
        let got = rule()
            .try_match(&chars(word), 0)
            .map(|m| (m.cells[0], m.consumed));
        assert_eq!(got, expected);
    }

    #[test]
    fn runtime_be_prefix_uses_restricted_groupsign() {
        let word = chars(std::hint::black_box("become"));
        let matched = rule().try_match(&word, std::hint::black_box(0)).unwrap();

        assert_eq!(matched.cells, vec![decode_unicode('⠆')]);
        assert_eq!(matched.consumed, 2);
    }

    /// The groupsign is word-initial only — no match at a non-zero position.
    #[test]
    fn no_match_off_word_start() {
        assert!(rule().try_match(&chars("rebecome"), 2).is_none());
    }
}
