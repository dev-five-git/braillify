//! English shortform collision detection (UEB 5.7.2 + 10.9).
//!
//! When an all-uppercase ASCII word is point-encoded as `⠠⠠xy...`, the trailing
//! cells are identical to the corresponding lowercase shortform abbreviation. To
//! prevent the contraction reading (e.g. `⠠⠠⠉⠙` could otherwise be read as the
//! capitalised word "COULD"), the Grade-1 indicator (`⠰`) must be inserted before
//! the capital indicator.
//!
//! Reference: 통일영어점자 규정 제3판
//! - §5.7.2: 약자(축어 포함)와의 혼동 방지를 위한 1급 점자 모드
//! - §10.9: 축어(shortform) 목록 (부록 1)
//!
//! Only **pure-letter** shortforms (whose braille cells map one-to-one to a-z)
//! can collide. Shortforms that embed contractions like `ch` (⠡), `sh` (⠩), `st`
//! (⠌), `th` (⠹), `ou` (⠳), or `con` (⠒) are NOT pure-letter, so their uppercase
//! acronyms (e.g. "MCH" → ⠠⠠⠍⠉⠓) cannot be confused with the shortform reading
//! and do not require the Grade-1 indicator.

use std::collections::HashSet;
use std::sync::OnceLock;

/// All pure-letter multi-letter shortforms from UEB Appendix 1 (lowercase form).
/// These cause collision with all-uppercase acronyms of the same letters.
const PURE_LETTER_SHORTFORMS: &[&str] = &[
    // a-series (10.9: about, above, according, ...)
    "ab", "abv", "ac", "acr", "af", "afn", "afw", "ag", "al", "alm", "alr", "alt",
    "alw", // b-series (10.9: because, before, behind, below, ...)
    "bc", "bf", "bh", "bl", "bn", "brl", "bs", "bt", "by", // c-series
    "cd", // could
    // d-series
    "dcl", "dclg", "dcv", "dcvg", // e-series
    "ei",   // either
    // f-series
    "fri", "fst", // g-series
    "gd", "grt", // h-series
    "hm", "hmf", "hrf", // i-series
    "imm", // l-series
    "ll", "lr",  // m-series
    "myf", // n-series
    "nec", "nei", // p-series
    "pd", "perh", // q-series
    "qk",   // r-series
    "rcv", "rcvg", "rjc", "rjcg", // s-series
    "sd",   // t-series
    "td", "tgr", "tm", "tn", // w-series
    "wd", // x-series
    "xf", "xs", // y-series
    "yr", "yrf", "yrvs",
];

fn shortform_set() -> &'static HashSet<&'static str> {
    static CACHE: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| PURE_LETTER_SHORTFORMS.iter().copied().collect())
}

/// Returns `true` if the given ASCII word (already verified all-uppercase) collides
/// with a multi-letter shortform when emitted as `⠠⠠letters`. The Grade-1 indicator
/// `⠰` must be inserted before the CapsWord/CapsPassage marker in that case.
///
/// Single-letter words are excluded (UEB §10.1 single-letter alphabetic word signs
/// require their own "독립적으로 사용된 경우" analysis handled elsewhere).
pub fn requires_grade1_indicator(uppercase_word: &str) -> bool {
    if uppercase_word.len() < 2 {
        return false;
    }
    if !uppercase_word.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    let lowered = uppercase_word.to_ascii_lowercase();
    shortform_set().contains(lowered.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cd_collides_with_could() {
        assert!(requires_grade1_indicator("CD"));
    }

    #[test]
    fn hm_collides_with_him() {
        assert!(requires_grade1_indicator("HM"));
    }

    #[test]
    fn td_collides_with_today() {
        assert!(requires_grade1_indicator("TD"));
    }

    #[test]
    fn wd_collides_with_would() {
        assert!(requires_grade1_indicator("WD"));
    }

    #[test]
    fn lp_does_not_collide() {
        // L = like, P = people are single-letter alphabetic wordsigns;
        // their concatenation is not a multi-letter shortform.
        assert!(!requires_grade1_indicator("LP"));
    }

    #[test]
    fn kbs_does_not_collide() {
        assert!(!requires_grade1_indicator("KBS"));
    }

    #[test]
    fn mp_does_not_collide() {
        assert!(!requires_grade1_indicator("MP"));
    }

    #[test]
    fn tv_does_not_collide() {
        assert!(!requires_grade1_indicator("TV"));
    }

    #[test]
    fn sns_does_not_collide() {
        assert!(!requires_grade1_indicator("SNS"));
    }

    #[test]
    fn single_letter_excluded() {
        assert!(!requires_grade1_indicator("C"));
        assert!(!requires_grade1_indicator("A"));
    }

    #[test]
    fn non_ascii_excluded() {
        assert!(!requires_grade1_indicator("É"));
        assert!(!requires_grade1_indicator("C1"));
    }

    #[test]
    fn case_insensitive_input() {
        // Function expects already-uppercase but should still match if lowercase given.
        assert!(requires_grade1_indicator("cd"));
    }

    #[test]
    fn runtime_shortform_lookup_uses_lowercase_key() {
        let word = std::hint::black_box("CD");

        assert!(requires_grade1_indicator(word));
    }
}
