//! §10.7 initial-letter contractions, pronunciation-gated (feature
//! `english_ueb_cmudict`).
//!
//! Extends the safe spelling-only set in [`super::rule_10_7`] with the
//! contractions that module defers as pronunciation-dependent. A contraction is
//! applied at a position ONLY when the whole word's CMUdict pronunciation
//! contains the standalone contraction word's pronunciation as a contiguous
//! phoneme run (comparing ARPABET base symbols, ignoring stress) — i.e. the
//! letters keep their normal sound there: `someone` /-wʌn/ → use `one`;
//! `money` /-ʌni/ and `component` → spell out. All recorded pronunciations of
//! the word must agree, mirroring the conservative §10.6 classifier: a wrong
//! contraction is far worse than a missed one, so unknown words or non-matching
//! pronunciations spell out.
//!
//! Spelling-based occurrences whose sound differs (`acknowledge`→`know` /-nɒl-/,
//! `Germany`→`many` /-ʌnɪ/) are *safely missed* by this gate (no contraction),
//! never mis-encoded.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule};
use super::pronunciation::{Phoneme, PronunciationProvider};
use crate::unicode::decode_unicode;

/// §10.7 deferred initial-letter contractions → (prefix cell, first-letter cell),
/// taken from the RUEB 2024 §10.7 examples.
/// The unambiguous ones (`right`, `cannot`, `world`, …) already live ungated in
/// [`super::rule_10_7`]; this table holds only the pronunciation-gated remainder.
static PRON_CONTRACTIONS: phf::Map<&'static str, [u8; 2]> = phf_map! {
    // ⠐ (dot 5) prefix.
    "day"    => [decode_unicode('⠐'), decode_unicode('⠙')],
    "ever"   => [decode_unicode('⠐'), decode_unicode('⠑')],
    "father" => [decode_unicode('⠐'), decode_unicode('⠋')],
    "here"   => [decode_unicode('⠐'), decode_unicode('⠓')],
    "know"   => [decode_unicode('⠐'), decode_unicode('⠅')],
    // `lord` deferred: its phonemes appear contiguously across a morpheme
    // boundary in `chlordane` (chlor|dane), a §10.11 bridging case this gate
    // cannot detect without syllabification — a wrong contraction is worse than
    // the few missed `lordship`/`Gaylord` wins.
    "mother" => [decode_unicode('⠐'), decode_unicode('⠍')],
    "part"   => [decode_unicode('⠐'), decode_unicode('⠏')],
    "some"   => [decode_unicode('⠐'), decode_unicode('⠎')],
    "under"  => [decode_unicode('⠐'), decode_unicode('⠥')],
    "where"  => [decode_unicode('⠐'), decode_unicode('⠱')],
    "work"   => [decode_unicode('⠐'), decode_unicode('⠺')],
    "name"   => [decode_unicode('⠐'), decode_unicode('⠝')],
    "one"    => [decode_unicode('⠐'), decode_unicode('⠕')],
    "there"  => [decode_unicode('⠐'), decode_unicode('⠮')],
    "time"   => [decode_unicode('⠐'), decode_unicode('⠞')],
    "young"  => [decode_unicode('⠐'), decode_unicode('⠽')],
    // ⠘ (dots 4-5) prefix. (`word` moved to the ungated spelling-only §10.7 set.)
    "these"  => [decode_unicode('⠘'), decode_unicode('⠮')],
    "those"  => [decode_unicode('⠘'), decode_unicode('⠹')],
    "upon"   => [decode_unicode('⠘'), decode_unicode('⠥')],
    // ⠸ (dots 4-5-6) prefix. (`many` moved to the ungated spelling-only §10.7 set.)
    // `had` is pronunciation-gated (unlike the ungated `many`/`world`), so it is not
    // taken where the letters take a different sound (`shadow` /SH AE D/, not
    // /HH AE D/) — `haddock`, `Galahad`, `hadn't` contract, `shadow` spells out.
    "had"    => [decode_unicode('⠸'), decode_unicode('⠓')],
    "their"  => [decode_unicode('⠸'), decode_unicode('⠮')],
};

/// Pronunciation-gated §10.7 initial-letter contraction rule.
pub struct InitialContractionPronunciationRule {
    provider: Box<dyn PronunciationProvider>,
}

impl InitialContractionPronunciationRule {
    /// Build the rule with the pronunciation source used to gate each contraction.
    pub fn new(provider: Box<dyn PronunciationProvider>) -> Self {
        Self { provider }
    }

    /// True iff EVERY recorded pronunciation of `full_word` contains SOME recorded
    /// pronunciation of `contraction` as a contiguous phoneme run.
    fn pronunciation_supports(&self, full_word: &str, contraction: &str) -> bool {
        let full = self.provider.pronunciations(full_word);
        let sub = self.provider.pronunciations(contraction);
        if full.is_empty() || sub.is_empty() {
            return false;
        }
        full.iter()
            .all(|fp| sub.iter().any(|sp| contains_contiguous(fp, sp)))
    }

    /// §10.7: a SUFFICIENT acceptance for a contraction whose letters end in a silent
    /// `e` (`one`, `some`, `name`, `time`, `here`, `there`, `where`, `these`,
    /// `those`). The letters form the `…one`/`…ere` unit — word-final OR medial —
    /// when the trailing `e` is silent or `ey`-merged in every recorded pronunciation
    /// (`cone`, `atonement`, `honey`); a `e` voicing its own vowel splits the unit
    /// (`krone` /…ə/, `Monet` /…eɪ/, `phonetic` /…ɛ…/) — see [`super::pronunciation::aligner`].
    /// A preceding vowel letter is rejected so the first letter never continues a
    /// vowel digraph (`B[oo]ne`, `R[oo]ney`).
    fn silent_final_e_supports(&self, word: &[char], pos: usize, key: &str, full: &str) -> bool {
        if !key.ends_with('e') {
            return false;
        }
        // The preceding-vowel guard stops a VOWEL-initial contraction (`one`, the
        // `ere` of `here`/`there`/`where`) from continuing a vowel digraph
        // (`B[oo]ne`, `R[oo]ney`). A CONSONANT-initial contraction (`some`, `name`,
        // `time`, `here`, `there`, `where`) cannot form a digraph with a preceding
        // vowel, so the guard does not apply — `blithe·some`, `chromo·some` keep it.
        let key_starts_vowel = key.starts_with(['a', 'e', 'i', 'o', 'u']);
        if key_starts_vowel && pos > 0 && matches!(word[pos - 1], 'a' | 'e' | 'i' | 'o' | 'u') {
            return false;
        }
        let e_idx = pos + key.chars().count() - 1;
        // §10.10: a following `ed`/`er` strong groupsign claims the trailing `e` with
        // the next letter (`stone·d`→⠫, `ton·er`→⠻, `on·er·ous`→⠻), so the silent `e`
        // must not be swallowed into the contraction: `stoned`=st·o·n·ed,
        // `toner`=t·o·n·er — not st·one·d / t·one·r.
        if matches!(word.get(e_idx + 1), Some('d' | 'r')) {
            return false;
        }
        let prons = self.provider.pronunciations(full);
        super::pronunciation::aligner::trailing_letter_is_silent_or_merged(word, e_idx, &prons)
    }

    /// §10.7 `ever`: used where the letters `e·v·e·r` form the unstressed `-er` unit
    /// (`fever`, `several`, `beverage`) — the trailing `er` carrying no stress — but
    /// not a stressed `e·VER·sion`/`se·VER·ity`, nor across a preceding vowel digraph
    /// (`belie·ver`'s `ie`). Restricted to `ever`; `father`/`mother`/`under` are
    /// deferred (no test corpus, and `under` risks coincidental substrings like
    /// `th·under`).
    fn ever_shape_supports(&self, word: &[char], pos: usize, key: &str, full: &str) -> bool {
        if key != "ever" {
            return false;
        }
        if pos > 0 && matches!(word[pos - 1], 'a' | 'e' | 'i' | 'o' | 'u') {
            return false;
        }
        let er_e_idx = pos + 2; // the `e` of the trailing `er` in `e·v·e·r`
        let prons = self.provider.pronunciations(full);
        super::pronunciation::aligner::trailing_er_is_unstressed(word, er_e_idx, &prons)
    }

    /// True iff `chars` is a recorded CMUdict headword.
    fn is_word(&self, chars: &[char]) -> bool {
        !self
            .provider
            .pronunciations(&chars.iter().collect::<String>())
            .is_empty()
    }

    /// §10.7 danger zone: decide a final-`e` contraction `key` at `pos` whose next
    /// letter is the `r`/`d` of an -er/-ed suffix. The contraction is used only when
    /// it is a morpheme exposed by word structure, not merely by sound.
    fn danger_zone_use(&self, word: &[char], pos: usize, key: &str, end: usize) -> bool {
        // The suffix must be a single trailing `r`/`d`; a longer tail (`some|rsault`,
        // `some|rs`) means the letters do not sit on the -er/-ed boundary.
        if end + 1 != word.len() {
            return false;
        }
        // The base IS the contraction: `time`+r, `name`+d. There is no word
        // `somer`/`somed`, so a bare `some`+suffix never carries the -some morpheme.
        if word[..end].iter().copied().eq(key.chars()) {
            return key != "some";
        }
        // If the base WITHOUT the contraction's final `e` is itself a word, the word
        // is (that word)+ed/er and the `e` is the suffix vowel — not part of the
        // -some/-name morpheme (`ransom`+ed, `blossom`+ed). `handsom` (from
        // `handsome`+r) is not a word, so `handsomer` survives this guard.
        if self.is_word(&word[..end - 1]) {
            return false;
        }
        // Otherwise the contraction must be an exposed suffix of the base: a known
        // word or a closed bound prefix precedes it, AND the base's pronunciation
        // carries the contraction's sound. `one` is excluded from known-word
        // recovery so `toner`/`sooner`/`crooner` keep the `er` groupsign (§10.7).
        // A closed bound prefix (`mis`·time·d, `re`·name·d) is reliable evidence on
        // its own — skip phonology, since the base (`mistime`, `rename`) is often
        // absent from CMUdict. With the §10.10 path-priority tie-break the offered
        // contraction is now actually selected over the overlapping `st`/`en`.
        let left_str: String = word[..pos].iter().collect();
        if bound_prefix_exposes(key, &left_str) {
            return true;
        }
        // A known word before the contraction still needs phonology on the base
        // (`hand`·some → `handsome` carries /S AH M/) to avoid a coincidence.
        matches!(key, "some" | "time" | "name")
            && self.is_word(&word[..pos])
            && self.pronunciation_supports(&word[..end].iter().collect::<String>(), key)
    }

    /// §10.7 morphology recovery for CMUdict GAPS: when a word is absent from the
    /// dictionary (so phonology can neither confirm nor deny) but its structure
    /// exposes the contraction as a clean morpheme — `blithe·some`, `some·such`,
    /// `tea·time`, `time·ously` — the sign is safe. Restricted to the
    /// consonant-initial final-`e` keys whose `e` is reliably silent as a
    /// suffix/compound unit (`one` is excluded: its vowel onset risks a
    /// digraph/diphthong with a coincidental neighbour). A known word is left to
    /// the phonology gates above; unknown words with no clean split spell out.
    fn morphology_recovers(&self, word: &[char], pos: usize, key: &str, end: usize) -> bool {
        // Consonant-initial final-`e` keys (silent-`e` compounds) plus selected
        // vowel-initial `one` compounds (`stone·work`, `lone·some`) and `upon`, whose
        // `here·upon` compound is a clean word-edge morpheme (`there·upon`/
        // `where·upon` are already in CMUdict; `coupon` is, so it never reaches here).
        if !matches!(
            key,
            "some" | "name" | "time" | "here" | "there" | "where" | "upon" | "one"
        ) || self.is_word(word)
        {
            return false;
        }
        // The contraction must sit at a WORD EDGE. A fragment on BOTH sides
        // (`ga`|some|`ter` in `gasometer`, where the dict happens to list `ga`) is a
        // coincidence, not a morpheme boundary — so exactly one side must be empty.
        let after = &word[end..];
        let after_ok = after.is_empty()
            || self.is_word(after)
            || is_safe_suffix(&after.iter().collect::<String>());
        if key == "one" {
            // §10.7.6: use `one` in compounds/derivatives (`stonework`,
            // `demonetise`, `lonesomest`) when not preceded by `o`; require a real
            // following component/suffix so monomorphemes like `anemone` still spell.
            pos > 0 && word[pos - 1] != 'o' && !after.is_empty() && after_ok
        } else if pos == 0 {
            // Word-initial unit: `some`·such, `time`·ously, `name`·able, `where`·of.
            // A bare key alone is a known word (handled by phonology), so a valid
            // non-empty remainder is required.
            !after.is_empty() && after_ok
        } else if after.is_empty() {
            // Word-final unit: `blithe`·some, `tea`·time, `your`·name.
            self.is_word(&word[..pos])
        } else if matches!(key, "some" | "time")
            && is_safe_suffix(&after.iter().collect::<String>())
        {
            self.is_word(&word[..pos])
        } else {
            false
        }
    }
}

impl ContractionRule for InitialContractionPronunciationRule {
    fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
        let full: String = word.iter().collect();
        let mut best: Option<(usize, [u8; 2])> = None;
        for (key, &cells) in PRON_CONTRACTIONS.entries() {
            let klen = key.chars().count();
            let end = pos + klen;
            if end > word.len()
                || !key.chars().zip(&word[pos..end]).all(|(k, w)| k == *w)
                || best.is_some_and(|(bl, _)| klen <= bl)
            {
                continue;
            }
            if *key == "where"
                && (word.get(end) == Some(&'\'')
                    || word.get(end..).is_some_and(|tail| {
                        tail.starts_with(&['e', 'v', 'e', 'r'])
                            || tail.starts_with(&['v', 'e', 'r'])
                    }))
            {
                continue;
            }
            if *key == "time"
                && pos > 0
                && word[pos - 1] == 'n'
                && !matches!(word.get(..pos), Some(['u', 'n']))
            {
                continue;
            }
            if *key == "day"
                && end != word.len()
                && word
                    .get(end)
                    .is_some_and(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            {
                continue;
            }
            if *key == "ever" && pos > 0 && word[pos - 1] == 'i' {
                continue;
            }
            if *key == "one" && pos > 0 && matches!(word[pos - 1], 'a' | 'e' | 'i' | 'o' | 'u') {
                continue;
            }
            if *key == "one" && word.get(end..) == Some(&['s', 's']) {
                continue;
            }
            if *key == "one" && word.get(end..) == Some(&['g', 'a', 'l']) {
                continue;
            }
            // The letters keep their sound here when EITHER the whole word's
            // pronunciation contains the contraction's run (`part`/`work`), OR — for
            // a silent-`e` contraction standing word-finally — the trailing `e` is
            // proven silent (`cone`, `phone`, `adhere`). Note: these §10.7
            // initial-letter contractions are whole-unit signs (priority 55), exempt
            // from §10.11 compound-seam bridging — `up·on`/`there·up·on` legitimately
            // span their own etymological seams.
            // §10.7 danger zone: a final-`e` contraction immediately followed by
            // `r`/`d` (the -er/-ed suffix) cannot be judged by phonology alone —
            // `time+r` (USE) shares the /… ER/ / /… D/ shape with `some+rsault`,
            // interior `tone`+`r`, and `ransom+ed`. Require an exposed morpheme.
            let danger =
                key.ends_with('e') && word.get(end).is_some_and(|c| matches!(c, 'r' | 'd'));
            let accept = if (*key == "had" && pos == 0 && !matches!(word.get(3), Some('e' | 'r')))
                || (*key == "day"
                    && (end == word.len()
                        || word
                            .get(end)
                            .is_some_and(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))))
                || (*key == "under" && pos > 0 && word[pos - 1] == 'w')
                || (*key == "ever" && word.get(pos..pos + 5) == Some(&['e', 'v', 'e', 'r', 'y']))
                || (*key == "ever" && ever_morpheme_exposed(word, pos, end))
                || (*key == "one"
                    && (word.get(pos..pos + 5) == Some(&['o', 'n', 'e', 's', 't'])
                        || word.get(end..end + 4) == Some(&['s', 'o', 'm', 'e'])
                        || word.get(end..) == Some(&['t', 'i', 's', 'e'])
                        || word.get(end) == Some(&'y')))
                || (*key == "time"
                    && pos > 0
                    && matches!(word.get(..pos), Some(['u', 'n']))
                    && word.get(end..) == Some(&['l', 'y']))
            {
                true
            } else if danger {
                self.danger_zone_use(word, pos, key, end)
            } else {
                self.pronunciation_supports(&full, key)
                    || self.silent_final_e_supports(word, pos, key, &full)
                    || self.ever_shape_supports(word, pos, key, &full)
                    || self.morphology_recovers(word, pos, key, end)
            };
            if accept {
                best = Some((klen, cells));
            }
        }
        best.map(|(klen, cells)| ContractionMatch {
            cells: cells.to_vec(),
            consumed: klen,
            // Same band as the spelling-only §10.7 set so longest-match drives
            // the choice; below §10.4 strong groupsigns (60) on equal length.
            priority: 55,
            // §10.10.1: pronunciation-validated (`part` in `apartheid`), so the
            // cell-minimiser must not split it with a cheaper generic contraction.
            protect_span: true,
        })
    }
}

/// Closed set of bound prefixes that expose a final-`e` contraction as a morpheme
/// in an -er/-ed word (`mis·time·d`, `re·name·d`). Kept deliberately small so the
/// danger-zone override never over-accepts.
fn bound_prefix_exposes(key: &str, left: &str) -> bool {
    matches!((key, left), ("time", "mis") | ("name", "re"))
}

/// Closed set of safe derivational/inflectional suffixes that may follow a
/// recovered final-`e` contraction (`time·ously`, `lone·some·st`, `some`·`s`).
fn is_safe_suffix(s: &str) -> bool {
    matches!(
        s,
        "s" | "es" | "st" | "ly" | "ness" | "ous" | "ously" | "able" | "ment"
    )
}

/// §10.7 `ever` in transparent bound forms not reliably covered by CMUdict.
fn ever_morpheme_exposed(word: &[char], pos: usize, end: usize) -> bool {
    matches!(word.get(end..), Some([] | ['a', 't', 'e']))
        && matches!(word.get(..pos), Some(['a', 's', 's'] | [.., 's', 'o']))
}

/// Whether `needle` occurs as a contiguous run inside `haystack`, comparing only
/// ARPABET base symbols (lexical stress shifts in context: `one` is W AH1 N alone
/// but W AH0 N in `anyone`).
fn contains_contiguous(haystack: &[Phoneme], needle: &[Phoneme]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|w| w.iter().zip(needle).all(|(a, b)| a.base == b.base))
}

#[cfg(test)]
mod tests {
    use super::super::pronunciation::cmudict::CmuDictProvider;
    use super::*;

    fn rule() -> InitialContractionPronunciationRule {
        InitialContractionPronunciationRule::new(Box::new(CmuDictProvider::new()))
    }

    fn try_at(word: &str, pos: usize) -> Option<(Vec<u8>, usize)> {
        let chars: Vec<char> = word.chars().collect();
        rule().try_match(&chars, pos).map(|m| (m.cells, m.consumed))
    }

    /// Pronunciation MATCHES → contraction used (the letters keep their sound).
    #[rstest::rstest]
    // partake = P AA R T EY K — `part` (P AA R T) at pos 0.
    #[case::partake("partake", 0, Some((vec![decode_unicode('⠐'), decode_unicode('⠏')], 4)))]
    // apartheid = AH P AA R T … — `part` (P AA R T) at pos 1.
    #[case::apartheid("apartheid", 1, Some((vec![decode_unicode('⠐'), decode_unicode('⠏')], 4)))]
    // network = N EH T W ER K — `work` (W ER K) at pos 3.
    #[case::network("network", 3, Some((vec![decode_unicode('⠐'), decode_unicode('⠺')], 4)))]
    fn applies_when_pronunciation_matches(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(Vec<u8>, usize)>,
    ) {
        assert_eq!(try_at(word, pos), expected);
    }

    /// §10.7 `ever`: the unstressed `-er` unit contracts (`fever` /…V ER0/,
    /// `several` /…V R AH0…/); a stressed `e·VER` (`eversion`) or a full vowel at the
    /// `e` (`severity`) does not.
    #[rstest::rstest]
    #[case::fever("fever", 1, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::several("several", 1, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::beverage("beverage", 1, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::whosesoever("whosesoever", 7, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::asseverate("asseverate", 3, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::eversion("eversion", 0, None)] // e·VER stressed → use `er`
    #[case::severity("severity", 1, None)] // se·VER·ity — e is a full vowel
    fn applies_ever_when_unstressed(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: Option<(Vec<u8>, usize)>,
    ) {
        assert_eq!(try_at(word, pos), expected);
    }

    /// The letters split across the contraction's sound → safely spelled out.
    /// `component` voices a schwa at the `e` (`…N AH0 N T`), so the `one` unit is
    /// split; `acknowledge`'s `know` is /N AA…/, not /N OW/.
    #[rstest::rstest]
    #[case::component_not_one("component", 4)] // K AH M P OW N AH0 N T — e voices schwa
    #[case::acknowledge_not_know("acknowledge", 2)] // AE K N AA L … — `know` is N OW
    fn spells_out_when_pronunciation_differs(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    /// Silent word-final `e` → the e-ending contraction is used even though the
    /// in-word vowel differs from the standalone sound (`cone` /…N/ ≠ `one` /wʌn/).
    #[rstest::rstest]
    #[case::cone("cone", 1, decode_unicode('⠕'), 3)] // c·one — word-final
    #[case::done("done", 1, decode_unicode('⠕'), 3)]
    #[case::phone("phone", 2, decode_unicode('⠕'), 3)]
    #[case::everyone("everyone", 5, decode_unicode('⠕'), 3)] // y precedes — not a digraph
    #[case::adhere("adhere", 2, decode_unicode('⠓'), 4)] // ad·here
    #[case::atonement("atonement", 2, decode_unicode('⠕'), 3)] // medial: at·one·ment
    #[case::dishonesty("dishonesty", 4, decode_unicode('⠕'), 3)] // dis·h·one·sty
    #[case::honey("honey", 1, decode_unicode('⠕'), 3)] // medial: h·one·y (ey merge)
    #[case::money("money", 1, decode_unicode('⠕'), 3)] // m·one·y — identical to honey
    fn applies_silent_final_e(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] letter: u8,
        #[case] consumed: usize,
    ) {
        assert_eq!(
            try_at(word, pos),
            Some((vec![decode_unicode('⠐'), letter], consumed))
        );
    }

    /// The silent-`e` path stays conservative: a SOUNDED final `e` (`krone` /…ə/,
    /// `anemone` /…i/), a preceding vowel digraph (`b[oo]ne`), or a medial occurrence
    /// (`money`) all spell out rather than risk a wrong contraction.
    #[rstest::rstest]
    #[case::krone("krone", 2)] // K R OW N AH0 — sounded final e
    #[case::anemone("anemone", 4)] // … N IY0 — sounded final e
    #[case::boone("boone", 2)] // b·oo·ne — o starts inside the `oo` digraph
    #[case::abalone("abalone", 4)] // … OW N IY0 — sounded final e
    #[case::colonel("colonel", 3)] // irregular K ER1 N AH0 L — e voices schwa
    #[case::stoned("stoned", 2)] // st·o·n·ed — `ed` groupsign claims e-d
    #[case::toner("toner", 1)] // t·o·n·er — `er` groupsign claims e-r
    fn silent_final_e_stays_conservative(#[case] word: &str, #[case] pos: usize) {
        assert_eq!(try_at(word, pos), None);
    }

    /// The contiguous-run helper compares ARPABET base only (stress-insensitive)
    /// and requires a true contiguous run.
    #[test]
    fn contiguous_run_compares_base_only() {
        use super::super::pronunciation::parse_phoneme;
        let hay: Vec<Phoneme> = "S AH0 M W AH1 N"
            .split_whitespace()
            .map(parse_phoneme)
            .collect();
        // `W AH N` matches despite stress differing (AH1 needle vs AH1 hay here).
        let need: Vec<Phoneme> = "W AH0 N".split_whitespace().map(parse_phoneme).collect();
        assert!(contains_contiguous(&hay, &need));
        // `M AH N` is present as letters but not as a contiguous phoneme run.
        let nope: Vec<Phoneme> = "M AH1 N".split_whitespace().map(parse_phoneme).collect();
        assert!(!contains_contiguous(&hay, &nope));
    }

    /// §10.7 danger zone (a final-`e` contraction immediately followed by `r`/`d`).
    /// The sign is used only when it is an EXPOSED morpheme — the whole base
    /// (`time`+r, `name`+d), or a known word / bound prefix + key (`hand`·some·r,
    /// `re`·name·d) — never a coincidental run (`somer`sault), an interior substring
    /// (`t`·one within `tone`+r), or the `-ed`/`-er` suffix vowel (`ransom`+ed,
    /// `blossom`+ed). `one` is excluded from known-word recovery.
    #[rstest::rstest]
    #[case::timer("timer", 0, true)] // time + r
    #[case::named("named", 0, true)] // name + d
    #[case::renamed("renamed", 2, true)] // re + name + d (bound prefix)
    #[case::mistimed("mistimed", 3, true)] // mis + time + d (bound prefix)
    #[case::handsomer("handsomer", 4, true)] // hand + some + r
    #[case::toner("toner", 1, false)] // tone interior; keep `er`
    #[case::somersault("somersault", 0, false)] // somer… not a morpheme
    #[case::somerset("somerset", 0, false)]
    #[case::somers("somers", 0, false)] // som + er + s (tail not a single r)
    #[case::ransomed("ransomed", 3, false)] // ransom + ed
    #[case::blossomed("blossomed", 4, false)] // blossom + ed
    fn danger_zone_er_ed(#[case] word: &str, #[case] pos: usize, #[case] use_it: bool) {
        assert_eq!(try_at(word, pos).is_some(), use_it, "danger zone: {word}");
    }

    /// §10.7 morphology recovery for CMUdict GAPS: an unknown word whose structure
    /// exposes the contraction at a WORD EDGE (`some`·such, `tea`·time, `blithe`·some,
    /// `time`·ously, `name`·able) uses the sign; a fragment on both sides
    /// (`ga`·some·`ter`) is a coincidence and spells out.
    #[rstest::rstest]
    #[case::somesuch("somesuch", 0, true)] // some + such
    #[case::teatime("teatime", 3, true)] // tea + time
    #[case::blithesome("blithesome", 6, true)] // blithe + some
    #[case::nameable("nameable", 0, true)] // name + able
    #[case::hereupon("hereupon", 4, true)] // here + upon (word-edge, dict gap)
    #[case::timeously("timeously", 0, true)] // time + ously
    #[case::gasometer("gasometer", 2, false)] // ga | some | ter — coincidence
    fn morphology_recovery_for_dict_gaps(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] use_it: bool,
    ) {
        assert_eq!(try_at(word, pos).is_some(), use_it, "recovery: {word}");
    }

    #[test]
    fn silent_final_e_support_rejects_vowel_bridge_and_ed_er_claims() {
        let rule = rule();
        let boone: Vec<char> = "boone".chars().collect();
        assert!(!rule.silent_final_e_supports(&boone, 2, "one", "boone"));

        let stoned: Vec<char> = "stoned".chars().collect();
        assert!(!rule.silent_final_e_supports(&stoned, 2, "one", "stoned"));
    }

    #[test]
    fn morphology_recovery_direct_paths_for_word_edges() {
        let rule = rule();
        let somesuch: Vec<char> = "somesuch".chars().collect();
        assert!(rule.morphology_recovers(&somesuch, 0, "some", 4));

        let teatime: Vec<char> = "teatime".chars().collect();
        assert!(rule.morphology_recovers(&teatime, 3, "time", 7));

        let nameable: Vec<char> = "nameable".chars().collect();
        assert!(rule.morphology_recovers(&nameable, 0, "name", 4));

        let blithesome: Vec<char> = "blithesome".chars().collect();
        assert!(rule.morphology_recovers(&blithesome, 6, "some", 10));

        let coincidental_middle: Vec<char> = "gasometer".chars().collect();
        assert!(!rule.morphology_recovers(&coincidental_middle, 2, "some", 6));
    }

    #[test]
    fn morphology_recovery_runtime_initial_unit_checks_suffix() {
        let rule = rule();
        let word: Vec<char> = std::hint::black_box("whereof").chars().collect();

        assert!(rule.morphology_recovers(&word, 0, "where", 5));
    }

    #[test]
    fn morphology_recovery_directly_covers_empty_after_word_final_unit() {
        let rule = rule();
        let teatime: Vec<char> = "teatime".chars().collect();

        assert!(rule.morphology_recovers(&teatime, 3, "time", 7));
    }

    #[test]
    fn morphology_recovery_directly_covers_initial_and_one_paths() {
        let rule = rule();
        let timeously: Vec<char> = "timeously".chars().collect();
        assert!(rule.morphology_recovers(&timeously, 0, "time", 4));

        let stonework: Vec<char> = "stonework".chars().collect();
        assert!(rule.morphology_recovers(&stonework, 2, "one", 5));

        let alone: Vec<char> = "alone".chars().collect();
        assert!(!rule.morphology_recovers(&alone, 2, "one", 5));
    }

    #[test]
    fn morphology_recovery_directly_rejects_known_word_and_accepts_initial_suffix() {
        let rule = rule();
        let some: Vec<char> = "some".chars().collect();
        assert!(!rule.morphology_recovers(&some, 0, "some", 4));

        let somewise: Vec<char> = "somewise".chars().collect();
        assert!(rule.morphology_recovers(&somewise, 0, "some", 4));
    }
}
