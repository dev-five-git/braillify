//! 제18항 — 다음 단어들은 약어를 사용하여 적는다.
//! (그래서, 그러나, 그러면, 그러므로, 그런데, 그리고, 그리하여)
//!
//! Word-level abbreviations: entire words are replaced with short braille sequences.
//! Lookup is delegated to `word_shortcut::split_word_shortcut()`.
//!
//! [다만] 약어 앞에 다른 글자가 붙어 나올 때에는 약어를 사용하지 않는다.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 2, Section 7, Article 18

use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::word_shortcut;

pub static META: RuleMeta = RuleMeta {
    section: "18",
    subsection: None,
    name: "word_abbreviation",
    standard_ref: "2024 Korean Braille Standard, Ch.2 Sec.7 Art.18",
    description: "Word abbreviations: 그래서,그러나,그러면,그러므로,그런데,그리고,그리하여",
};

/// Try to match a word against the abbreviation table.
/// Returns Some((matched_str, braille_codes, remaining_str)) if matched.
#[cfg(test)]
fn apply(text: &str) -> Option<(&'static str, &'static [u8], String)> {
    word_shortcut::split_word_shortcut(text)
}

/// Check whether `word_chars` starts with any entry in the word-shortcut table,
/// returning the matching braille code slice without materializing a `String`.
///
/// Hot-path alternative to `word_shortcut::split_word_shortcut(&word)` when the
/// caller only needs the matched codes (제18항 약어). PHF table has 7 short
/// Korean keys, so the linear scan is trivial.
fn match_word_shortcut(word_chars: &[char]) -> Option<&'static [u8]> {
    for (key, codes) in word_shortcut::SHORTCUT_MAP.entries() {
        let mut key_chars = key.chars();
        let mut matched = true;
        let mut consumed = 0usize;
        loop {
            match key_chars.next() {
                None => break,
                Some(kc) => match word_chars.get(consumed) {
                    Some(wc) if *wc == kc => {
                        consumed += 1;
                    }
                    _ => {
                        matched = false;
                        break;
                    }
                },
            }
        }
        if matched {
            return Some(*codes);
        }
    }
    None
}

/// Plugin struct for the rule engine.
///
/// Handles word-level abbreviations (제18항): 그래서, 그러나, 그러면, etc.
/// Runs in the WordShortcut phase at index 0 (word start).
/// When matched, emits the abbreviated braille codes and Consumes.
///
/// Note: Handling the "rest" (suffix after abbreviation, e.g., "그래서인지" → "인지")
/// requires re-entering the encoding pipeline. In Phase 3, the engine-driven
/// encode_word() will handle this recursion.
pub struct Rule18;

impl BrailleRule for Rule18 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::WordShortcut
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        // Word shortcuts only apply at the beginning of a word
        if ctx.index != 0 {
            return false;
        }
        match_word_shortcut(ctx.word_chars).is_some()
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if let Some(codes) = match_word_shortcut(ctx.word_chars) {
            ctx.emit_slice(codes);
            // TODO(Phase 3): handle `rest` by re-entering encoding pipeline
            // For now, the remaining characters are handled by the caller.
            Ok(RuleResult::Consumed)
        } else {
            Ok(RuleResult::Skip)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 제18항 — 7개 약어 단어가 모두 약자 매칭된다.
    #[rstest::rstest]
    #[case("그래서")]
    #[case("그러나")]
    #[case("그러면")]
    #[case("그러므로")]
    #[case("그런데")]
    #[case("그리고")]
    #[case("그리하여")]
    fn matches_all_word_abbreviations(#[case] word: &str) {
        let result = apply(word);
        assert!(result.is_some(), "Expected abbreviation for: {word}");
        let (matched, codes, rest) = result.unwrap();
        assert_eq!(matched, word);
        assert!(!codes.is_empty());
        assert!(rest.is_empty());
    }

    #[test]
    fn matches_with_suffix() {
        // 그래서인지 → matches 그래서, rest = "인지"
        let result = apply("그래서인지").unwrap();
        assert_eq!(result.0, "그래서");
        assert_eq!(result.2, "인지");
    }

    /// 약어 사전 미등록 단어 → None.
    #[rstest::rstest]
    #[case::korean_unknown("안녕하세요")]
    #[case::english("hello")]
    fn no_match_for_non_abbreviation(#[case] input: &str) {
        assert!(apply(input).is_none());
    }

    /// Direct tests for `match_word_shortcut` — covers lines 31-55.
    #[rstest::rstest]
    #[case("그래서")]
    #[case("그러나")]
    #[case("그러면")]
    #[case("그러므로")]
    #[case("그런데")]
    #[case("그리고")]
    #[case("그리하여")]
    fn match_word_shortcut_finds_each_abbreviation(#[case] word: &str) {
        let chars: Vec<char> = word.chars().collect();
        let result = match_word_shortcut(&chars);
        assert!(result.is_some(), "should match {word}");
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn match_word_shortcut_returns_none_for_unknown() {
        let chars: Vec<char> = "안녕".chars().collect();
        assert!(match_word_shortcut(&chars).is_none());
    }

    #[test]
    fn match_word_shortcut_matches_prefix_only() {
        // 그래서인지 — prefix matches 그래서
        let chars: Vec<char> = "그래서인지".chars().collect();
        let result = match_word_shortcut(&chars);
        assert!(result.is_some());
    }

    #[test]
    fn match_word_shortcut_short_word_no_match() {
        // Single char — shorter than any key in shortcut map
        let chars: Vec<char> = "가".chars().collect();
        assert!(match_word_shortcut(&chars).is_none());
    }

    /// BrailleRule trait surface tests.
    #[test]
    fn rule18_meta_and_phase() {
        let rule = Rule18;
        let meta = rule.meta();
        assert_eq!(meta.section, "18");
        assert!(matches!(rule.phase(), Phase::WordShortcut));
    }

    fn make_ctx<'a>(
        word_chars: &'a [char],
        index: usize,
        char_type: &'a crate::char_struct::CharType,
        skip_count: &'a mut usize,
        state: &'a mut crate::rules::context::EncoderState,
        result: &'a mut Vec<u8>,
    ) -> RuleContext<'a> {
        RuleContext {
            word_chars,
            index,
            char_type,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: true,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count,
            state,
            result,
        }
    }

    #[test]
    fn rule18_matches_at_word_start_only() {
        use crate::char_struct::CharType;
        use crate::rules::context::EncoderState;
        let word_chars: Vec<char> = "그래서".chars().collect();
        let ct0 = CharType::new(word_chars[0]).unwrap();
        let mut skip = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let ctx_start = make_ctx(&word_chars, 0, &ct0, &mut skip, &mut state, &mut result);
        assert!(Rule18.matches(&ctx_start));

        let ct1 = CharType::new(word_chars[1]).unwrap();
        let mut skip2 = 0usize;
        let mut state2 = EncoderState::new(false);
        let mut result2 = Vec::new();
        let ctx_mid = make_ctx(&word_chars, 1, &ct1, &mut skip2, &mut state2, &mut result2);
        assert!(!Rule18.matches(&ctx_mid));
    }

    #[test]
    fn rule18_apply_emits_codes_on_match() {
        use crate::char_struct::CharType;
        use crate::rules::context::EncoderState;
        let word_chars: Vec<char> = "그래서".chars().collect();
        let ct = CharType::new(word_chars[0]).unwrap();
        let mut skip = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = make_ctx(&word_chars, 0, &ct, &mut skip, &mut state, &mut result);
        let outcome = Rule18.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!result.is_empty());
    }

    #[test]
    fn rule18_apply_skips_on_no_match() {
        use crate::char_struct::CharType;
        use crate::rules::context::EncoderState;
        let word_chars: Vec<char> = "안녕".chars().collect();
        let ct = CharType::new(word_chars[0]).unwrap();
        let mut skip = 0usize;
        let mut state = EncoderState::new(false);
        let mut result = Vec::new();
        let mut ctx = make_ctx(&word_chars, 0, &ct, &mut skip, &mut state, &mut result);
        let outcome = Rule18.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
        assert!(result.is_empty());
    }
}
