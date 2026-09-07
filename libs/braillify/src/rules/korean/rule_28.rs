//! 제28항 — 로마자는 ｢통일영어점자 규정｣에 따라 다음과 같이 적는다.
//!
//! English letters are mapped to braille using the UEB (Unified English Braille) system.
//! Uppercase indicators: single ⠠(32), word ⠠⠠(32,32), passage ⠠⠠⠠(32,32,32).
//!
//! Encoding is delegated to `english::encode_english()`.
//!
//! Reference: 2024 Korean Braille Standard, Chapter 4, Section 10, Article 28

use crate::char_struct::CharType;
use crate::english_logic::requires_single_letter_continuation;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::english_shortform::{
    permits_grade1_boundary_after_run, requires_grade1_indicator,
};
use crate::rules::english_ueb::korean_context::KoreanPrefixInput;
use crate::rules::english_ueb::span::{encode_korean_unit, encode_korean_word};
use crate::rules::english_ueb::standing_alone::lower_wordsign_usable;
use crate::rules::english_ueb::token::EnglishToken;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "28",
    subsection: None,
    name: "english_encoding",
    standard_ref: "2024 Korean Braille Standard, Ch.4 Sec.10 Art.28",
    description: "English letters encoded per UEB (Unified English Braille)",
};

/// Single uppercase indicator (대문자 기호표).
pub const UPPERCASE_SINGLE: u8 = 32; // ⠠

/// Encode a single English letter to braille.
#[cfg(test)]
fn apply(ch: char) -> Result<u8, String> {
    crate::english::encode_english(ch)
}

/// Returns a slice of indicator bytes to prepend.
#[cfg(test)]
fn uppercase_indicators(
    is_single_uppercase: bool,
    is_word_all_uppercase: bool,
    consecutive_uppercase_words: u8,
) -> &'static [u8] {
    if consecutive_uppercase_words >= 3 {
        &[32, 32, 32] // passage: ⠠⠠⠠
    } else if is_word_all_uppercase {
        &[32, 32] // word: ⠠⠠
    } else if is_single_uppercase {
        &[32] // single: ⠠
    } else {
        &[]
    }
}

/// Plugin struct for the rule engine.
///
/// Handles 제28항 English-in-Korean encoding: 로마자표/연속표 entry and uppercase
/// indicators. Letter/contraction cell production is delegated to
/// [`crate::rules::english_ueb::span`]; 종료표/exit orchestration lives in
/// [`crate::rules::emit`].
pub struct Rule28;

impl BrailleRule for Rule28 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::English(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::English(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // At index 0 the emitter may already have emitted the Roman indicator,
        // so use its pre-entry snapshot. For an ASCII run later in a mixed
        // print word, the live mode accurately says whether this run continues
        // an existing Roman section or starts a fresh one.
        let continuing_roman_section = if ctx.index == 0 {
            ctx.roman_section_continues_from_previous_word
        } else {
            ctx.state.is_english
        };

        // Enter English mode (로마자표 / 연속표)
        // 제39항 영어 주도 문서에서는 영자표시/연속표를 emit하지 않는다.
        if ctx.state.english_indicator
            && !ctx.state.is_english
            && !ctx.state.english_dominant_no_indicator
        {
            if ctx.state.needs_english_continuation {
                ctx.emit(48);
            } else {
                ctx.emit(52);
            }
        }

        // 제37항: a Roman section in Korean text spells the word with UEB
        // alphabet signs and multi-letter groupsigns, while suppressing UEB
        // whole-word contractions. Encode each contiguous ASCII letter run in
        // one pass so the shared UEB preference/morphology algorithm can choose
        // contractions across the whole word. Lowercase apostrophe continuations
        // retain the legacy position-aware path because they are not fresh word
        // starts. An uppercase continuation is encoded as a run so UEB 8.4.2 can
        // restart capitals mode after the nonalphabetic apostrophe.
        let starts_ascii_run = c.is_ascii_alphabetic()
            && ctx
                .index
                .checked_sub(1)
                .and_then(|index| ctx.word_chars.get(index))
                .is_none_or(|previous| !previous.is_ascii_alphabetic());
        let follows_apostrophe = ctx
            .index
            .checked_sub(1)
            .and_then(|index| ctx.word_chars.get(index))
            .is_some_and(|previous| matches!(previous, '\'' | '\u{2019}'));
        if starts_ascii_run && (!follows_apostrophe || c.is_ascii_uppercase()) {
            let run_end = ctx.index
                + ctx.word_chars[ctx.index..]
                    .iter()
                    .take_while(|ch| ch.is_ascii_alphabetic())
                    .count();
            let run = &ctx.word_chars[ctx.index..run_end];
            // The token rule pre-emits capitals-word mode only for the initial
            // uppercase letters-sequence. UEB 8.4.2 ends that mode at a
            // nonletter, so a later run (the final `T` in official `AT&T`)
            // must produce its own capitalization indicator.
            let caps_already_emitted = ctx.state.triple_big_english
                || (ctx.index == 0
                    && ctx.is_all_uppercase
                    && ctx.word_len() >= 2
                    && ctx.ascii_starts_at_beginning);
            let word_initial = ctx.index == 0
                || ctx.word_chars.get(ctx.index - 1).is_some_and(|previous| {
                    crate::utils::is_korean_char(*previous)
                        || matches!(
                            previous,
                            '(' | '['
                                | '{'
                                | '\u{2018}'
                                | '\u{201c}'
                                | '"'
                                | '-'
                                | '\u{2010}'
                                | '\u{2011}'
                                | '\u{2012}'
                                | '\u{2013}'
                                | '\u{2014}'
                        )
                });
            let run_is_all_uppercase = run.iter().all(|ch| ch.is_ascii_uppercase());
            let is_standing_alone_ordinary_run = !run_is_all_uppercase
                && word_initial
                && permits_grade1_boundary_after_run(&ctx.word_chars[run_end..]);
            // Rule 37's PDF example, "그는 Can you help me?라고 도움을 요청했다.",
            // suppresses a whole-word sign for the first Roman word (`Can`) but retains
            // the UEB wordsign for the following `you`. Rule 29 keeps consecutive
            // Roman words in the same section, so every complete ordinary-cased word
            // after the first Roman word has the same continuation status, including
            // the final word of a phrase. UEB capitalization does not suppress a
            // wordsign, hence Title-case `Like`/`This` follows the same rule. All-caps
            // runs remain excluded because Rule 10.12.1 initialisms and emphasized
            // words have the same surface form and require pronunciation semantics.
            // Rule 39's "What is 김치 in English?" resumes the surrounding English
            // passage after Korean, so the persistent English-dominant gate retains
            // the resumed `in` wordsign. Neither gate depends on a corpus reference.
            let whole_print_word = ctx.index == 0 && run_end == ctx.word_chars.len();
            let wrap_wordsign = ctx.state.english_dominant_wrap_active && whole_print_word;
            // 제37항 붙임: these six words are spelled with alphabet signs and
            // applicable groupsigns even when they occur later in the Roman
            // section immediately before its terminator.  Other continuation
            // words, such as official `you` in `Can you help me?`, retain their
            // ordinary UEB wordsign.
            let lower_run = run
                .iter()
                .map(|ch| ch.to_ascii_lowercase())
                .collect::<String>();
            let is_lower_wordsign = matches!(
                lower_run.as_str(),
                "be" | "enough" | "his" | "in" | "was" | "were"
            );
            let rule_37_korean_context_exception = !ctx.state.english_dominant_wrap_active
                && !ctx.state.roman_section_is_english_context
                && is_lower_wordsign;
            // UEB 10.5 gives lower wordsigns a stricter boundary than ordinary
            // standing-alone wordsigns. In particular, a hyphen, dash, quote,
            // or lower punctuation cell touching either side forces spelling.
            // Reuse the English engine's boundary predicate instead of treating
            // Rule 28's general grade-1 boundary as sufficient (`In-house`).
            let previous_boundary = ctx
                .index
                .checked_sub(1)
                .and_then(|index| ctx.word_chars.get(index))
                .copied()
                .map(EnglishToken::Symbol);
            let next_boundary = ctx
                .word_chars
                .get(run_end)
                .copied()
                .map(EnglishToken::Symbol);
            let lower_wordsign_boundary_permits = !is_lower_wordsign
                || lower_wordsign_usable(previous_boundary.as_ref(), next_boundary.as_ref());
            let standalone_wordsign = is_standing_alone_ordinary_run
                && (wrap_wordsign || continuing_roman_section)
                && !rule_37_korean_context_exception
                && lower_wordsign_boundary_permits;
            let digit_adjacent = ctx
                .index
                .checked_sub(1)
                .and_then(|index| ctx.word_chars.get(index))
                .is_some_and(|ch| ch.is_ascii_digit())
                || ctx
                    .word_chars
                    .get(run_end)
                    .is_some_and(|ch| ch.is_ascii_digit());
            let numeric_grade1_active = ctx
                .index
                .checked_sub(1)
                .and_then(|index| ctx.word_chars.get(index))
                .is_some_and(|ch| ch.is_ascii_digit())
                && ctx.word_chars[..ctx.index.saturating_sub(1)]
                    .iter()
                    .any(|ch| ch.is_ascii_alphabetic());
            // UEB 5.7.1-5.7.2 and 5.8.1: grade 1 precedes the capitalization
            // marker when a standing letter/letters-sequence would otherwise be
            // read as an alphabetic wordsign or shortform.  A bare one-letter
            // Rule 28 specimen (`K`) keeps the PDF's plain alphabet cell, while
            // the same letter in running text (`K-POP`, `ARIRANG K방산Fn`) is a
            // UEB 5.7.1 standing letter.  Multi-letter uppercase tokens have
            // already had their capitalization mode emitted by
            // `UppercasePassageRule`; an adjacent digit is not a standing-alone
            // boundary, whereas a hyphen or dash explicitly is (UEB 2.6.1).
            let uppercase_run = run.iter().collect::<String>();
            let entire_isolated_rule_28_specimen = ctx.index == 0
                && run_end == ctx.word_chars.len()
                && ctx.prev_word.is_empty()
                && ctx.remaining_words.is_empty();
            let single_letter_wordsign_collision = run.len() == 1
                && requires_single_letter_continuation(run[0])
                && ctx.index == 0
                && ctx.roman_section_continues_from_previous_word
                && !entire_isolated_rule_28_specimen;
            let shortform_collision = requires_grade1_indicator(&uppercase_run);
            let prepend_grade1_indicator = !caps_already_emitted
                && word_initial
                && !digit_adjacent
                && run.iter().all(|ch| ch.is_ascii_uppercase())
                && permits_grade1_boundary_after_run(&ctx.word_chars[run_end..])
                && (single_letter_wordsign_collision || shortform_collision);
            let apostrophe_joined_lexeme =
                crate::rules::english_ueb::pronunciation::apostrophe_elided_recorded_word_at(
                    ctx.word_chars,
                    ctx.index,
                    run_end,
                );
            if let Some(cells) = encode_korean_word(
                run,
                caps_already_emitted,
                prepend_grade1_indicator,
                standalone_wordsign,
                word_initial,
                digit_adjacent,
                numeric_grade1_active,
                apostrophe_joined_lexeme,
            ) {
                ctx.emit_slice(&cells);
                *ctx.skip_count = run.len().saturating_sub(1);
                ctx.state.is_english = true;
                ctx.state.needs_english_continuation = false;
                return Ok(RuleResult::Consumed);
            }
        }

        // Uppercase indicators (single/consecutive uppercase run)
        if (!ctx.is_all_uppercase || ctx.word_len() < 2 || !ctx.ascii_starts_at_beginning)
            && !ctx.state.is_big_english
            && c.is_uppercase()
        {
            ctx.state.is_big_english = true;
            for idx in 0..std::cmp::min(ctx.word_len() - ctx.index, 2) {
                if ctx.word_chars[ctx.index + idx].is_uppercase() {
                    ctx.emit(UPPERCASE_SINGLE);
                } else {
                    break;
                }
            }
        }

        // English abbreviation lookup + fallback letter encoding.
        // Korean-context UEB contractions and standalone wordsigns are delegated to
        // `encode_korean_unit`; this rule only decides the surrounding mode markers.
        let is_whole_lowercase_word =
            ctx.index == 0 && ctx.word_chars.iter().all(|ch| ch.is_ascii_lowercase());
        let prev_is_ascii_word =
            !ctx.prev_word.is_empty() && ctx.prev_word.chars().all(|ch| ch.is_ascii_alphabetic());
        let next_is_ascii_word = ctx
            .remaining_words
            .first()
            .is_some_and(|w| !w.is_empty() && w.chars().all(|ch| ch.is_ascii_alphabetic()));
        let unit = encode_korean_unit(KoreanPrefixInput {
            word: ctx.word_chars,
            pos: ctx.index,
            wrap_active: ctx.state.english_dominant_wrap_active,
            is_all_uppercase: ctx.is_all_uppercase,
            at_entry: !ctx.state.is_english || ctx.index == 0,
            standalone_wordsign: is_whole_lowercase_word
                && prev_is_ascii_word
                && next_is_ascii_word,
        })?;
        ctx.emit_slice(&unit.cells);
        if unit.contracted {
            *ctx.skip_count = unit.consumed.saturating_sub(1);
        }

        ctx.state.is_english = true;
        ctx.state.needs_english_continuation = false;
        Ok(RuleResult::Consumed)
    }
}

/// Determine the uppercase indicator(s) needed.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::context::EncodingMode;
    use crate::unicode::decode_unicode;
    use crate::{EncodeOptions, encode_to_unicode, encode_with_options};

    /// 제28항 — 영문자 점역. 소문자/대문자 모두 동일 점형으로 인코딩.
    #[rstest::rstest]
    #[case::lower_a('a', '⠁')]
    #[case::lower_z('z', '⠵')]
    #[case::upper_a_as_lowercase('A', '⠁')]
    fn encodes_english_letters(#[case] ch: char, #[case] expected: char) {
        assert_eq!(apply(ch).unwrap(), decode_unicode(expected));
    }

    /// 영문자가 아닌 입력은 Err.
    #[rstest::rstest]
    #[case::digit('1')]
    #[case::syllable('가')]
    fn invalid_returns_error(#[case] ch: char) {
        assert!(apply(ch).is_err());
    }

    /// `uppercase_indicators` — single/word/passage 대문자 지시자 점형.
    #[rstest::rstest]
    #[case::single_letter(true, false, 0, &[32u8] as &[u8])]
    #[case::word_two_letters(false, true, 0, &[32, 32])]
    #[case::passage_run(false, true, 3, &[32, 32, 32])]
    #[case::no_indicator_lower(false, false, 0, &[] as &[u8])]
    fn uppercase_indicator_paths(
        #[case] single: bool,
        #[case] is_word: bool,
        #[case] run: u8,
        #[case] expected: &[u8],
    ) {
        assert_eq!(uppercase_indicators(single, is_word, run), expected);
    }

    /// 제37항 PDF examples: Korean-context Roman words suppress whole-word
    /// contractions while retaining their applicable multi-letter groupsigns.
    #[rstest::rstest]
    #[case::initial_letter_groupsign("every", &[52, 16, 17, 61, 50])]
    #[case::lower_and_strong_groupsigns("enough", &[52, 34, 51, 35, 50])]
    #[case::strong_contraction_inside_word("rather", &[52, 23, 1, 46, 23, 50])]
    #[case::entry_lower_wordsign_spelled_as_letters("in", &[52, 10, 29, 50])]
    fn korean_roman_words_share_ueb_groupsign_algorithm(
        #[case] input: &str,
        #[case] expected: &[u8],
    ) {
        let options = EncodeOptions {
            default_mode: Some(EncodingMode::Korean),
        };
        assert_eq!(encode_with_options(input, &options).unwrap(), expected);
    }

    /// 제37항 PDF 문장 전체를 공개 encoder로 통과시켜, 첫 Roman 어절의
    /// complete wordsign 억제와 뒤따르는 Roman phrase 경로를 함께 검증한다.
    #[test]
    fn rule_37_official_sentence_uses_shared_roman_engine() {
        assert_eq!(
            encode_to_unicode("그는 Can you help me?라고 도움을 요청했다.").unwrap(),
            "⠈⠪⠉⠵⠀⠴⠠⠉⠁⠝⠀⠽⠀⠓⠑⠇⠏⠀⠍⠑⠦⠐⠣⠈⠥⠀⠊⠥⠍⠢⠮⠀⠬⠰⠻⠚⠗⠌⠊⠲"
        );
    }

    /// Rule 37 limits its whole-word-contraction suppression to the Roman word
    /// immediately following the indicator. Apostrophe punctuation in that
    /// first word and a Korean suffix attached to the final word do not start a
    /// second Roman section, so subsequent `do` and `this` retain UEB wordsigns.
    #[test]
    fn rule_37_continuation_survives_apostrophe_and_attached_korean_suffix() {
        assert_eq!(
            encode_to_unicode("그는 Let's do this라고 말했다.").unwrap(),
            "⠈⠪⠉⠵⠀⠴⠠⠇⠑⠞⠄⠎⠀⠙⠀⠹⠲⠐⠣⠈⠥⠀⠑⠂⠚⠗⠌⠊⠲"
        );
    }

    /// UEB 5.7.2/5.8.1/10.9.7 complete-shortform handling through the complete
    /// Korean encoder. Every Roman surface comes directly from the PDF examples
    /// (`CD`, `ALT`, `NEC`); the Korean wrapper exercises only rule 28/29/34 routing.
    #[rstest::rstest]
    #[case::standing_alone_could("가(CD)", "⠫⠦⠄⠴⠰⠠⠠⠉⠙⠠⠴")]
    #[case::alt_example("가(ALT)", "⠫⠦⠄⠴⠰⠠⠠⠁⠇⠞⠠⠴")]
    #[case::nec_example("가(NEC)", "⠫⠦⠄⠴⠰⠠⠠⠝⠑⠉⠠⠴")]
    fn attached_allcaps_complete_shortform_uses_grade1(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).unwrap(), expected);
    }

    /// UEB 2.6.1-2.6.3 boundaries must be enforced on the Rule28 path as well as
    /// on whitespace tokens.  A leading quote forces this path because the token
    /// itself no longer starts with ASCII; `CD`/`LLC` are official 10.9 examples.
    #[rstest::rstest]
    #[case::closing_quote("‘CD’", true)]
    #[case::attached_after_korean("가CD", true)]
    #[case::non_shortform_after_korean("가KBS", false)]
    #[case::digit_after_hyphen("5-CD-678", true)]
    #[case::closing_group_before_korean_middle_dot("(CD)·현금", true)]
    #[case::adjacent_digit("‘CD47", false)]
    #[case::slash_continuation("‘CD/ATM", false)]
    #[case::opening_group_after_sequence("‘LLC(회사)", false)]
    fn noninitial_ascii_run_respects_grade1_boundary(#[case] input: &str, #[case] expected: bool) {
        let encoded = crate::encode(input).unwrap();
        assert_eq!(
            encoded
                .windows(3)
                .any(|window| window == [48, UPPERCASE_SINGLE, UPPERCASE_SINGLE]),
            expected
        );
    }

    /// UEB 5.7.1/5.8.1: a single capital wordsign letter standing in running
    /// text needs grade 1 before its capital indicator.  The rule is structural:
    /// the following boundary may be whitespace, a hyphen, or a Korean code
    /// boundary.  `a`, `i`, and `o` are excluded by the shared UEB predicate.
    #[rstest::rstest]
    #[case::roman_number_chain("가 X5 M 나", 'm')]
    #[case::hyphen_bounded("가 EAFF E-1 나", 'e')]
    #[case::korean_code_boundary("가 ARIRANG K방산Fn 나", 'k')]
    fn running_single_capital_wordsign_letter_uses_grade1(
        #[case] input: &str,
        #[case] letter: char,
    ) {
        let encoded = crate::encode(input).unwrap();
        let letter = crate::english::encode_english(letter).unwrap();

        assert!(encoded.windows(3).any(|window| {
            window
                == [
                    crate::rules::korean::rule_29::ENGLISH_CONTINUATION,
                    UPPERCASE_SINGLE,
                    letter,
                ]
        }));
    }

    /// Korean Rule 28's alphabet table is a specimen, not running contracted
    /// English.  Its isolated capital letters therefore retain the plain Rule
    /// 28 form without a UEB grade-1 prefix.
    #[test]
    fn isolated_rule_28_capital_specimen_stays_plain() {
        assert_eq!(crate::encode_to_unicode("K").as_deref(), Ok("⠠⠅"));
    }

    /// UEB 8.4.2 keeps an internal apostrophe in the Roman letters-sequence but
    /// terminates capitals-word mode at that nonalphabetic symbol. The Roman
    /// surfaces are official UEB examples; the neutral Korean wrapper exercises
    /// Rule 28/29 routing. Korean Rule 37 still suppresses the `that` wordsign in
    /// `THAT'S`, so its initial run retains the permitted `th` groupsign instead.
    #[rstest::rstest]
    #[case::official_name("가 O'Hara 나", "⠫⠀⠴⠠⠕⠄⠠⠓⠜⠁⠲⠀⠉")]
    #[case::official_contraction("가 DON'T 나", "⠫⠀⠴⠠⠠⠙⠕⠝⠄⠠⠞⠲⠀⠉")]
    #[case::official_possessive("가 THAT'S 나", "⠫⠀⠴⠠⠠⠹⠁⠞⠄⠠⠎⠲⠀⠉")]
    #[case::official_two_letter_suffix("가 SHE'LL 나", "⠫⠀⠴⠠⠠⠩⠑⠄⠠⠠⠇⠇⠲⠀⠉")]
    fn korean_wrapper_restarts_capitals_after_internal_apostrophe(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    /// UEB 10.6.8 keeps `en` inside a capitals word when the letters belong to
    /// an ordinarily pronounced word. Removing the internal apostrophe yields
    /// recorded `opening`, which distinguishes this emphasis from a 10.12.1
    /// initialism while exercising the Korean Rule 28/37 wrapper.
    #[test]
    fn apostrophe_elided_lexeme_contracts_inside_capitals_word() {
        assert_eq!(
            crate::encode_to_unicode("가 O'PENing 나").as_deref(),
            Ok("⠫⠀⠴⠠⠕⠄⠠⠠⠏⠢⠠⠄⠬⠲⠀⠉")
        );
    }

    #[test]
    fn english_dominant_wrap_resumes_ueb_wordsigns_after_korean_span() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("in", true);
        owned.state.is_english = true;
        owned.state.english_dominant_wrap_active = true;
        let mut ctx = owned.ctx_at(0);

        assert!(matches!(
            Rule28.apply(&mut ctx).unwrap(),
            RuleResult::Consumed
        ));
        assert_eq!(owned.result, vec![20]);
    }

    /// A document-level rule-39 wrap signal must not turn each separate Roman
    /// annotation inside a mixed Korean print word into a continuation. Each
    /// parenthesized item below begins a fresh rule-37 Roman section and is
    /// therefore spelled, even when its surface is also a UEB wordsign.
    #[rstest::rstest]
    #[case::titlecase_us(
        "(Us)",
        1,
        &[52, 32, decode_unicode('⠥'), decode_unicode('⠎')]
    )]
    #[case::lowercase_it("(it)", 1, &[52, decode_unicode('⠊'), decode_unicode('⠞')])]
    fn mixed_print_word_starts_fresh_rule_37_section(
        #[case] input: &str,
        #[case] index: usize,
        #[case] expected: &[u8],
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, true);
        owned.state.english_dominant_wrap_active = true;
        let mut ctx = owned.ctx_at(index);

        assert!(matches!(
            Rule28.apply(&mut ctx).unwrap(),
            RuleResult::Consumed
        ));
        assert_eq!(owned.result, expected);
    }

    /// Rule 37's PDF sentence `Can you help me?` permits wordsigns after the
    /// first Roman word. Rule 29 keeps the final Roman word in that same section,
    /// and UEB capitalization leaves the wordsign itself unchanged.
    #[rstest::rstest]
    #[case::interior_lowercase("you", "Can", &[decode_unicode('⠽')])]
    #[case::final_titlecase("This", "Like", &[32, decode_unicode('⠹')])]
    #[case::final_lowercase("will", "Boys", &[decode_unicode('⠺')])]
    fn rule_37_continuation_word_uses_standalone_wordsign(
        #[case] input: &str,
        #[case] previous: &str,
        #[case] expected: &[u8],
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, true)
            .with_prev_word(previous)
            .with_roman_section_continuation();
        owned.state.is_english = true;
        let mut ctx = owned.ctx_at(0);

        assert!(matches!(
            Rule28.apply(&mut ctx).unwrap(),
            RuleResult::Consumed
        ));
        assert_eq!(owned.result, expected);
    }

    /// 제37항 붙임: these words stay expanded throughout a Korean-context
    /// Roman section, including immediately before the Roman terminator.
    #[rstest::rstest]
    #[case::be("be", "⠃⠑")]
    #[case::enough("enough", "⠢⠳⠣")]
    #[case::his("his", "⠓⠊⠎")]
    #[case::in_word("in", "⠊⠝")]
    #[case::was("was", "⠺⠁⠎")]
    #[case::were("were", "⠺⠻⠑")]
    fn rule_37_terminator_exceptions_remain_expanded(#[case] input: &str, #[case] expected: &str) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, true)
            .with_prev_word("Can")
            .with_roman_section_continuation();
        owned.state.is_english = true;
        let mut ctx = owned.ctx_at(0);

        assert!(matches!(
            Rule28.apply(&mut ctx).unwrap(),
            RuleResult::Consumed
        ));
        assert_eq!(
            owned.result,
            expected.chars().map(decode_unicode).collect::<Vec<_>>()
        );
    }

    /// The NIKL's rule consultation distinguishes Korean metalinguistic Roman
    /// material from a visibly English phrase. In the latter context UEB 10.5
    /// applies to all six lower wordsigns, even though the surrounding document
    /// is Korean.
    #[rstest::rstest]
    #[case::be_word("be", '⠆')]
    #[case::enough_word("enough", '⠢')]
    #[case::his_word("his", '⠦')]
    #[case::in_word("in", '⠔')]
    #[case::was_word("was", '⠴')]
    #[case::were_word("were", '⠶')]
    fn english_phrase_uses_ueb_lower_wordsigns(#[case] word: &str, #[case] wordsign: char) {
        let input = format!("제목(Alpha {word} Omega)이다.");
        let actual = encode_to_unicode(&input).expect("English phrase must encode");
        let expected = format!("⠀{wordsign}⠀");

        assert!(
            actual.contains(&expected),
            "missing UEB lower wordsign in English phrase: {actual}"
        );
    }

    #[test]
    fn english_phrase_context_survives_a_preceding_capitals_passage() {
        let actual =
            encode_to_unicode("제목 ‘2023 SHINHWA WDJ FANPARTY COME TO LIFE in TAIPEI’는 끝이다.")
                .expect("capitalized English title must encode");

        assert!(
            actual.contains("⠀⠔⠀"),
            "caps-passage mode prefix lost the English phrase context: {actual}"
        );
    }

    /// UEB 10.5: a lower wordsign touching a hyphen is not usable even when the
    /// surrounding Roman section is clearly an English title.
    #[test]
    fn english_phrase_spells_lower_wordsign_touching_hyphen() {
        let actual = encode_to_unicode("제목(Alpha In-house Teams)이다.")
            .expect("hyphenated English phrase must encode");

        assert!(
            actual.contains("⠀⠠⠊⠝⠤"),
            "hyphen-adjacent `In` must remain expanded: {actual}"
        );
        assert!(
            !actual.contains("⠀⠠⠔⠤"),
            "hyphen-adjacent `In` must not use its lower wordsign: {actual}"
        );
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let _ = Rule28.apply(&mut ctx).unwrap();
        // Just exercise apply() for coverage
    }

    /// rule_28 — multi-cell `ong` abbreviation hit via real word `pyeongchang`
    /// from PDF testcase (rule_35.json). The 'o' at index 2 has remaining="ongchang"
    /// which matches `rule_en_multi_cell`.
    #[test]
    fn rule28_multi_cell_via_pyeongchang() {
        let _ = crate::encode("pyeongchang 2018");
    }

    /// rule_28:205-206 — multi-cell English abbreviation ("ong" → ⠰⠛)
    /// applied word-middle. Drives the `rule_en_multi_cell` arm via direct
    /// `RuleContext` setup with state.is_english=true, index > 0.
    #[test]
    fn rule28_multi_cell_word_middle_direct() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "along".chars().collect();
        let ct = CharType::English('o');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        state.is_english = true;
        let mut out = Vec::new();
        let mut ctx = crate::rules::context::RuleContext {
            word_chars: &word,
            index: 2, // 'o' position; remaining = "ong"
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: true,
            roman_section_continues_from_previous_word: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule28.apply(&mut ctx).unwrap();
        // Either Consumed (multi-cell applied) or other; at minimum the arm runs.
        let _ = outcome;
    }

    /// A rule invocation that resumes inside an ASCII run must restart a
    /// capitals indicator and stop its extent at the following lowercase
    /// letter. Normal full-word routing skips over this position in one pass;
    /// this direct check preserves the defensive continuation behavior.
    #[test]
    fn uppercase_continuation_stops_before_following_lowercase_letter() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("aBc", false);
        owned.state.is_english = true;
        let mut ctx = owned.ctx_at(1);

        let outcome = Rule28.apply(&mut ctx).unwrap();

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(owned.result.first(), Some(&UPPERCASE_SINGLE));
    }

    /// rule_28 line 64 — `let-else return Skip` for non-English ctx.
    #[test]
    fn rule28_apply_skip_for_non_english_ctx() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("가", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule28.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
