//! §10.6.5 / §10.11 middle lower groupsigns `ea bb cc ff gg`, morpheme-gated
//! (feature `english_ueb_cmudict`).
//!
//! These one-cell signs are used in the *middle* of a word — a letter on both
//! sides ([`super::rule_10_6::middle_lower_groupsign`]) — and, per RUEB 2024,
//! CONTRACT BY DEFAULT. The standard does NOT reduce to a syllable test: UEB
//! deliberately keeps these signs across syllable breaks, diphthongs and most
//! suffix seams. A sign is spelled out only where a more specific rule applies:
//!  * §10.10 preference — a strong contraction (§10.3), strong groupsign (§10.4)
//!    or final-letter groupsign (§10.8) begins at the shared second letter and
//!    outranks it (`ear`→`ar`, `m·eander`→`and`, `aff·ord`→`for`, `bacc·h`→`ch`);
//!  * §10.11.4 — an `ea` bridging a productive PREFIX seam (`re·action`,
//!    `pre·amble`, `de·activate`, `fore·arm`);
//!  * §10.11.1 — any sign bridging an unhyphenated COMPOUND seam (`pine·apple`,
//!    `hide·away`, `lime·ade`, `dumb·bell`, `sub·basement`, `arc·cosine`).
//!
//! A derivational-SUFFIX seam is explicitly allowed (§10.11.7/§10.11.8):
//! `agree·able`, `Europe·an`, `line·age`, `line·al`, `mile·age`, `peace·able`;
//! and a consonant doubled by a productive suffix is intra-stem (`begg·ing`).
//!
//! Boundaries are classified from word *structure* (the CMUdict word list, not
//! phoneme identity) plus small CLOSED morphological resources. Conservative by
//! design: an unknown split defaults to *contract* (a missed contraction is far
//! better than a wrong one), except where the candidate plainly crosses a
//! real-word+real-word compound seam, which defaults to *spell out*.
//!
//! RUEB 2024 §10.6.5 contracts lexicalised doubled forms such as `doggone` even
//! when their spelling historically resembles a compound.

use super::contraction::{ContractionMatch, ContractionRule};
use super::pronunciation::PronunciationProvider;
use super::rule_10_3::StrongContractionRule;
use super::rule_10_4::StrongGroupsignRule;
use super::rule_10_6::middle_lower_groupsign;
use super::rule_10_8::FinalGroupsignRule;

/// §10.11.7: `a`-initial derivational suffixes whose seam an `ea` keeps
/// (`agree·able`, `change·ability`, `line·age`, `line·al`, `laure·ate`).
/// Deliberately MINIMAL and long-form: short ambiguous endings (`-an`, `-ant`) are
/// EXCLUDED so a root opening with them (`anticline` in `ge·anticline`, `action`
/// in `re·action`) is not misread as a suffix and correctly spells out. The roots
/// that need them (`Europe·an`, `ocean·ic`, `pea·nut`) reach the same "contract"
/// verdict via the root-seam test (a short or non-word remainder is not a seam).
const A_INITIAL_SUFFIXES: &[&str] = &["ability", "able", "ably", "age", "al", "ate"];

/// Productive suffixes that trigger final-consonant doubling. A doubled groupsign
/// over `stem+ing/ed/…` is intra-stem and keeps the sign (`begg·ing`).
const CONSONANT_DOUBLING_SUFFIXES: &[&str] = &["ing", "ed", "er", "est", "y", "ish", "able"];

/// §10.11.1: combining forms that read as a second component, so an `ea` into one
/// spells out though it is not a free-standing dictionary word (`lime·ade`).
const ROOTLIKE_A_FORMS: &[&str] = &["ade"];

/// §10.6.7 productive prefixes whose final `e` can be followed by an `a`-initial
/// remainder, so `ea` bridges the prefix boundary and spells out (`de·activate`,
/// `ge·anticline`, `pre·amble`, `re·action`).
const EA_BRIDGING_PREFIXES: &[&str] = &["de", "fore", "ge", "pre", "re"];

/// Morpheme-gated §10.6.5/§10.11 middle lower groupsign rule.
pub struct MiddleLowerGroupsignRule {
    provider: Box<dyn PronunciationProvider>,
}

impl MiddleLowerGroupsignRule {
    /// Build the rule with the word source used to classify boundaries.
    pub fn new(provider: Box<dyn PronunciationProvider>) -> Self {
        Self { provider }
    }

    /// `ea` (§10.6.5/§10.11): contract by default; spell out only on a §10.10
    /// overlap, or where the `a` opens a new root/component — a prefix seam
    /// (§10.11.4, `re·action`) or compound seam (§10.11.1, `pine·apple`). A
    /// derivational-suffix seam (§10.11.7) keeps the sign and is checked first.
    fn ea_allowed(&self, word: &[char], pos: usize) -> bool {
        let b = pos + 1;
        if self.is_ea_suffix_seam(word, b) {
            return true;
        }
        let derived_compound_seams = self.derived_ea_compound_seams(word, pos);
        if bridges_known_compound_seam(word, pos, 2)
            || bridges_any_seam(&derived_compound_seams, pos, 2)
        {
            return false;
        }
        if sign_outranks_ea(word, b, &derived_compound_seams) {
            return false;
        }
        if starts_at_known_compound_component(word, pos) {
            return true;
        }
        !self.is_ea_prefix_seam(word, b)
    }

    /// Doubled `bb cc ff gg` (§10.6.5/§10.11): contract by default; spell out on an
    /// unhyphenated-compound seam (§10.11.1). A doubling suffix (`begg·ing`) keeps
    /// the sign. §10.10.5 examples such as `egghead` and `effulgent` keep the
    /// doubled lower groupsign even though `gh`/`ful` also begin at the second letter.
    fn doubled_allowed(&self, word: &[char], pos: usize) -> bool {
        if strong_sign_outranks_doubled(word, pos + 1) {
            return false;
        }
        if self.is_doubled_before_suffix(word, pos) {
            return true;
        }
        !self.is_doubled_compound_seam(word, pos)
    }

    /// §10.11.7: `word[b..]` opens with an `a`-initial derivational suffix and the
    /// stem is recorded (the stem itself, or the whole word — `perme·able`, where
    /// only `permeable` is in the list) — keep the sign.
    fn is_ea_suffix_seam(&self, word: &[char], b: usize) -> bool {
        let right = collect(&word[b..]);
        A_INITIAL_SUFFIXES.iter().any(|s| right.starts_with(s))
            && (self.is_word(&word[..b]) || self.is_word(word))
    }

    /// §10.11.4: the `a` opens a remainder after a productive prefix whose final
    /// letter is the `e` of the candidate (`re·action`, `pre·amble`). Free-word
    /// compounds are handled by the compound-seam oracle before this check, so
    /// monomorphemes whose `a...` tail happens to be a CMUdict word (`speak·er`,
    /// `tore·ador`) keep the `ea` sign.
    fn is_ea_prefix_seam(&self, word: &[char], b: usize) -> bool {
        let right = &word[b..];
        // §10.6.5: `ea` is retained in ordinary roots such as `reach`/`reaching`.
        // The `re-` prefix seam restriction only applies when the `a` opens a new
        // root (`re·act`, `pre·amble`), not when `rea` is the root spelling.
        if right.starts_with(&['a', 'c', 'h']) {
            return false;
        }
        if ROOTLIKE_A_FORMS.contains(&collect(right).as_str()) {
            return true;
        }
        let prefix = collect(&word[..b]);
        if !EA_BRIDGING_PREFIXES.contains(&prefix.as_str()) {
            return false;
        }
        right.len() >= 4 && self.is_word(right)
    }

    /// Additional solid-compound seams recover obvious free-word compounds not in
    /// the static source table. The right component must carry its own primary
    /// stress in CMUdict; this rejects bound/foreign tails such as `tore·ador` while
    /// accepting compounds like `wise·acre` and `flea·ridden`.
    fn derived_ea_compound_seams(&self, word: &[char], pos: usize) -> Vec<usize> {
        let b = pos + 1;
        let mut seams = Vec::with_capacity(2);
        if b >= 3 && self.is_word(&word[..b]) && self.is_compound_component(&word[b..]) {
            seams.push(b);
        }
        if b + 1 < word.len()
            && self.is_word(&word[..=b])
            && self.is_compound_component(&word[b + 1..])
        {
            seams.push(b + 1);
        }
        seams
    }

    fn is_compound_component(&self, chars: &[char]) -> bool {
        let text = collect(chars);
        if ROOTLIKE_A_FORMS.contains(&text.as_str()) {
            return true;
        }
        if chars.len() < 4 {
            return false;
        }
        let pronunciations = self.provider.pronunciations(&text);
        !pronunciations.is_empty()
            && pronunciations.iter().all(|pron| {
                pron.iter()
                    .find(|phoneme| phoneme.is_vowel())
                    .is_some_and(|phoneme| phoneme.stress == Some(1))
            })
    }

    /// A doubled consonant added by a productive suffix is intra-stem: the stem
    /// through the first letter is a word and a doubling suffix follows the pair
    /// (`beg`+`g`+`ing`).
    fn is_doubled_before_suffix(&self, word: &[char], pos: usize) -> bool {
        let base = &word[..=pos];
        let after = collect(&word[pos + 2..]);
        self.is_word(base)
            && CONSONANT_DOUBLING_SUFFIXES
                .iter()
                .any(|s| after.starts_with(s))
    }

    /// §10.11.1: the doubled pair bridges an unhyphenated compound — a real head
    /// (≥3) plus EITHER a real root word (≥4: `dumb|bell`, `sub|basement`) OR a long
    /// (≥5) run outside the word list that reads as a technical/proper second element
    /// (`arc|cosine`, `arc|tangent`). The ≥3 head floor keeps the silent assimilated
    /// prefix of `ac·count` part of one word; the ≥4-real-root / ≥5-long-tail floors
    /// keep a monomorphemic short coda contracted (`cab·bage`, `bub·ble`, `rib·bon`).
    /// A recorded whole word whose later vowels are all unstressed is treated as a
    /// lexicalised doubled form, not a productive compound: RUEB 2024 §10.6.5's
    /// `doggone` example contracts `gg` because the second written `g` no longer
    /// opens an independently stressed component.
    fn is_doubled_compound_seam(&self, word: &[char], pos: usize) -> bool {
        let left = &word[..=pos];
        let right = &word[pos + 1..];
        if left.len() < 3 || !self.is_word(left) {
            return false;
        }
        if self.is_lexicalized_doubled_form(word) {
            return false;
        }
        (right.len() >= 4 && self.is_word(right)) || right.len() >= 5
    }

    fn is_lexicalized_doubled_form(&self, word: &[char]) -> bool {
        let pronunciations = self.provider.pronunciations(&collect(word));
        !pronunciations.is_empty()
            && pronunciations.iter().all(|pron| {
                let mut vowels = pron.iter().filter(|phoneme| phoneme.is_vowel());
                let _first_stressed_head = vowels.next();
                vowels.all(|phoneme| phoneme.stress == Some(0))
            })
    }

    /// True iff `chars` form a word recorded in the pronunciation source.
    fn is_word(&self, chars: &[char]) -> bool {
        !self.provider.pronunciations(&collect(chars)).is_empty()
    }
}

fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

fn compound_seams_for(word: &[char]) -> Vec<usize> {
    super::compound::compound_seams(&collect(word))
}

fn starts_at_known_compound_component(word: &[char], pos: usize) -> bool {
    compound_seams_for(word).contains(&pos)
}

fn bridges_known_compound_seam(word: &[char], pos: usize, consumed: usize) -> bool {
    bridges_any_seam(&compound_seams_for(word), pos, consumed)
}

fn bridges_any_seam(seams: &[usize], pos: usize, consumed: usize) -> bool {
    seams
        .iter()
        .any(|&seam| pos < seam && seam < pos + consumed)
}

/// §10.10 preference: whether a strong contraction (§10.3), strong groupsign
/// (§10.4) or final-letter groupsign (§10.8) begins at `at` — any of which
/// outranks a §10.6 lower groupsign and must claim the shared letter.
/// Shared with the 제74항 digital-notation path so URL/email words apply the
/// same preference (`learn` → `l e ar n`, never `l ea r n`).
pub(crate) fn outranked_at(word: &[char], at: usize) -> bool {
    StrongContractionRule.try_match(word, at).is_some()
        || StrongGroupsignRule.try_match(word, at).is_some()
        || FinalGroupsignRule.try_match(word, at).is_some()
}

fn sign_outranks_ea(word: &[char], at: usize, derived_compound_seams: &[usize]) -> bool {
    [
        StrongContractionRule.try_match(word, at),
        StrongGroupsignRule.try_match(word, at),
        FinalGroupsignRule.try_match(word, at),
    ]
    .into_iter()
    .flatten()
    .any(|m| {
        !bridges_known_compound_seam(word, at, m.consumed)
            && !bridges_any_seam(derived_compound_seams, at, m.consumed)
    })
}

/// §10.10.5: a strong contraction/groupsign beginning at the second letter of a
/// doubled lower groupsign normally wins (`a f·for d`, `ba c·chanal`). It does
/// not win when that competing sign would itself bridge a known compound seam —
/// then §10.11 has already barred it (`egg·head`, `chiff·orobe`) — and final-letter
/// groupsigns do not outrank doubled lower signs (`effulgent`).
fn strong_sign_outranks_doubled(word: &[char], at: usize) -> bool {
    [
        StrongContractionRule.try_match(word, at),
        StrongGroupsignRule.try_match(word, at),
    ]
    .into_iter()
    .flatten()
    .any(|m| !bridges_compound_seam(word, at, m.consumed))
}

fn bridges_compound_seam(word: &[char], pos: usize, consumed: usize) -> bool {
    compound_seams_for(word)
        .iter()
        .any(|&seam| pos < seam && seam < pos + consumed)
}

impl ContractionRule for MiddleLowerGroupsignRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let m = middle_lower_groupsign(word, pos)?;
        // `middle_lower_groupsign` only matches `ea` or a doubled letter
        // (bb/cc/ff/gg), so the pair is either `ea` or a doubling — there is no
        // other case to guard.
        let allowed = if (word[pos], word[pos + 1]) == ('e', 'a') {
            self.ea_allowed(word, pos)
        } else {
            self.doubled_allowed(word, pos)
        };
        allowed.then_some(ContractionMatch {
            protect_span: true,
            ..m
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::pronunciation::cmudict::CmuDictProvider;
    use super::*;
    use crate::unicode::decode_unicode;

    fn rule() -> MiddleLowerGroupsignRule {
        MiddleLowerGroupsignRule::new(Box::new(CmuDictProvider::new()))
    }

    fn try_at(word: &str, pos: usize) -> Option<(Vec<u8>, usize)> {
        let chars: Vec<char> = word.chars().collect();
        rule().try_match(&chars, pos).map(|m| (m.cells, m.consumed))
    }

    /// `ea` keeps its sound mid-component or across a suffix seam → contracted (⠂).
    #[rstest::rstest]
    #[case::oceanic("oceanic", 2)] // `oce` is not a word → not a compound seam
    #[case::head("head", 1)] // coda `ad` < 4 → one root
    #[case::beat("beat", 1)]
    #[case::peanut("peanut", 1)] // `anut` is not a root word
    #[case::agreeable("agreeable", 4)] // agree|able — §10.11.7 suffix seam
    #[case::european("european", 5)] // europe|an — suffix seam
    #[case::lineage("lineage", 3)] // line|age — suffix seam
    #[case::lineal("lineal", 3)] // line|al — suffix seam
    #[case::peaceable("peaceable", 4)] // peace|able — suffix seam (2nd ea)
    #[case::caveat("caveat", 3)] // cave + short coda `at` → one root
    #[case::seashore("seashore", 1)] // ea within `sea` (the `a` closes `sea`)
    #[case::genealogy("genealogy", 3)] // gene·alogy — no root seam
    #[case::read("read", 1)] // monomorpheme — `ad` < 4 → not a root seam
    #[case::ready("ready", 1)] // monomorpheme — `ady` < 4 → not a root seam
    #[case::leader("leader", 1)] // §10.6.5: `ea` keeps its sound in a monomorpheme
    #[case::motheaten("motheaten", 4)] // §10.6.5: `ea` keeps its sound in eaten
    #[case::toreador("toreador", 3)] // §10.6.5: `ea` keeps its sound
    #[case::flearidden("flearidden", 2)] // §10.11.1: flea·ridden keeps `ea`
    #[case::tearoom("tearoom", 1)] // §10.11.1: tea·room keeps `ea`
    fn ea_contracts(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), Some((vec![decode_unicode('⠂')], 2)));
    }

    /// `ea` where the `a` opens a new root (prefix/compound seam) or §10.10
    /// overlap → spelled out.
    #[rstest::rstest]
    #[case::pineapple("pineapple", 3)] // pine|apple — compound seam
    #[case::hideaway("hideaway", 3)] // hide|away — compound seam
    #[case::limeade("limeade", 3)] // lime|ade — combining form
    #[case::reaction("reaction", 1)] // re|action — the `a` opens `action`
    #[case::preamble("preamble", 3)] // pre|amble — the `a` opens `amble`
    #[case::geanticline("geanticline", 1)] // ge|anticline — the `a` opens `anticline`
    #[case::wiseacre("wiseacre", 3)] // wise|acre — compound seam
    #[case::bear("bear", 1)] // ear → strong groupsign `ar`
    #[case::meander("meander", 1)] // eand → strong contraction `and`
    #[case::vengeance("vengeance", 4)] // eance → final groupsign `ance`
    fn ea_spells_out(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    #[test]
    fn ea_prefix_seam_recognizes_rootlike_a_form() {
        let chars: Vec<char> = "limeade".chars().collect();

        assert!(rule().is_ea_prefix_seam(&chars, 4));
    }

    /// Doubled letters mid-stem or doubled-by-suffix → contracted.
    #[rstest::rstest]
    #[case::bubble("bubble", 2, '⠆')] // monomorphemic
    #[case::accept("accept", 1, '⠒')] // `ac` < 3 → one word
    #[case::account("account", 1, '⠒')] // `ac` < 3 → one word
    #[case::begging("begging", 2, '⠶')] // beg + g + ing — doubling suffix
    #[case::doggone("doggone", 2, '⠶')] // §10.6.5 lexicalised doubled form
    #[case::chifforobe("chifforobe", 3, '⠖')] // §10.6.5 keeps medial `ff`
    #[case::rabbi("rabbi", 2, '⠆')] // mid-stem
    #[case::abbe("abbé", 1, '⠆')] // accented neighbour, mid-stem
    fn doubled_contracts(#[case] word: &str, #[case] pos: usize, #[case] cell: char) {
        assert_eq!(try_at(word, pos), Some((vec![decode_unicode(cell)], 2)));
    }

    /// Doubled letters across a compound seam or §10.10 overlap → spelled out.
    #[rstest::rstest]
    #[case::dumbbell("dumbbell", 3)] // dumb|bell — compound (both ≥ floors)
    #[case::subbasement("subbasement", 2)] // sub|basement — compound
    #[case::arccosine("arccosine", 2)] // arc|cosine — trig head
    #[case::afford("afford", 1)] // ff|or → strong contraction `for`
    #[case::bacchanal("bacchanal", 2)] // cc|h → strong groupsign `ch`
    fn doubled_spells_out(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }
}
