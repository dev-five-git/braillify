//! English shortform collision detection (UEB 5.7.2 + 10.9).
//!
//! When an all-uppercase ASCII letters-sequence is point-encoded as
//! `⠠⠠xy...`, its cells can be identical to a shortform or to the beginning of
//! a longer word containing one. To prevent that reading (for example `CD` as
//! "could", or the official `LLC` as "little" + `c`), the Grade-1 indicator
//! (`⠰`) must be inserted before the capital indicator.
//!
//! Reference: 통일영어점자 규정 제3판
//! - §5.7.2: 약자(축어 포함)와의 혼동 방지를 위한 1급 점자 모드
//! - §10.9: 축어(shortform) 목록과 10.9.2-10.9.5의 긴 단어 조건

/// Returns `true` if the given initial ASCII letters-sequence collides with a
/// shortform under UEB 10.9.7 or with a permitted longer shortform reading under
/// 10.9.8. The Grade-1 indicator `⠰` must precede the capitalization marker.
///
/// Single-letter words are excluded (UEB §10.1 single-letter alphabetic word signs
/// require their own "독립적으로 사용된 경우" analysis handled elsewhere).
pub fn requires_grade1_indicator(uppercase_word: &str) -> bool {
    super::english_ueb::rule_10_9::requires_grade1_at_word_start(uppercase_word)
}

/// UEB 2.6.1-2.6.3 boundary after a letters-sequence.
///
/// A grade-1 symbol used for shortform disambiguation is valid only when the
/// letters-sequence is standing alone (10.9.7), or is the initial sequence of a
/// longer alphabetic word (10.9.8).  Callers use this after an all-capitals ASCII
/// run, so any nonletter suffix must satisfy the standing-alone boundary.  A
/// Korean syllable starts the next code span and is likewise a hard boundary for
/// the embedded Roman sequence.  Digits, slash, plus, and an opening grouping
/// sign are deliberately excluded (`CD47`, `CD/ATM`, `NEIS+`, `LLM(SLM)`).
pub fn permits_grade1_boundary_after_run(suffix: &[char]) -> bool {
    for &ch in suffix {
        // UEB 2.6.1 makes a hyphen or dash a boundary in its own right.  Do
        // not scan through it into the next segment: the official `CD-ROM`
        // requires grade 1 for `CD` even though another Roman segment follows.
        if matches!(
            ch,
            '-' | '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}'
        ) {
            return true;
        }
        if matches!(ch as u32, 0x3131..=0x3163 | 0xAC00..=0xD7A3)
            || matches!(ch, '\u{00b7}' | '\u{30fb}')
        {
            return true;
        }
        if !matches!(
            ch,
            ',' | ';'
                | ':'
                | '.'
                | '\u{2026}'
                | '!'
                | '?'
                | ')'
                | ']'
                | '}'
                | '\''
                | '"'
                | '\u{2019}'
                | '\u{201d}'
        ) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::empty_boundary("", true)]
    #[case::official_cd_rom_boundary("-ROM", true)]
    #[case::closing_then_korean(")은", true)]
    #[case::adjacent_digit("47", false)]
    #[case::slash_continuation("/ATM", false)]
    #[case::plus_continuation("+", false)]
    #[case::opening_group("(SLM)", false)]
    #[case::comma_before_attached_letters(",ABC", false)]
    fn grade1_boundary_follows_ueb_standing_alone_rules(
        #[case] suffix: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(
            permits_grade1_boundary_after_run(&suffix.chars().collect::<Vec<_>>()),
            expected
        );
    }

    #[rstest::rstest]
    #[case::complete_cd("CD", true)]
    #[case::complete_hm("HM", true)]
    #[case::groupsign_fst("FST", true)]
    #[case::groupsign_shd("SHD", true)]
    #[case::official_llc_prefix("LLC", true)]
    #[case::good_prefix("GDP", true)]
    #[case::added_s("SDS", true)]
    #[case::because_needs_be("BC", false)]
    #[case::about_unlisted_suffix("ABBA", false)]
    #[case::little_before_vowel("LLAMA", false)]
    #[case::plain_initialism("KBS", false)]
    #[case::single_letter("C", false)]
    #[case::non_ascii("É", false)]
    #[case::alphanumeric("C1", false)]
    fn detects_complete_and_word_initial_shortform_confusion(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(requires_grade1_indicator(input), expected);
    }

    #[test]
    fn case_insensitive_runtime_input() {
        let word = std::hint::black_box("cd");

        assert!(requires_grade1_indicator(word));
    }
}
