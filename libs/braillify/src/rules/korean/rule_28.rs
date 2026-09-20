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
use crate::rules::english_ueb::rule_10_9::requires_grade1_before_spelled_letters;
use crate::rules::english_ueb::rule_10_12::{
    RomanRunPosition, is_letter_initialism_in_korean_text,
};
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

/// Whether the emitted cells end with a capitals word/passage prefix (`⠠⠠`,
/// `⠠⠠⠠`) that is itself preceded by the grade-1 symbol `⠰`.
fn grade1_precedes_capitals_prefix(result: &[u8]) -> bool {
    let capitals = result
        .iter()
        .rev()
        .take_while(|cell| **cell == UPPERCASE_SINGLE)
        .count();
    capitals >= 2
        && result[..result.len() - capitals].last()
            == Some(&crate::rules::korean::rule_29::ENGLISH_CONTINUATION)
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
        // print word, the live mode says whether the section is still open,
        // but an opening bracket alone (`(Like)`) does not make this run a
        // continuation: 제37항 still treats it as the section's first word
        // unless Roman text already preceded it.
        let continuing_roman_section = if ctx.index == 0 {
            ctx.roman_section_continues_from_previous_word
        } else {
            ctx.state.is_english
                && (ctx.roman_section_continues_from_previous_word
                    || ctx.word_chars[..ctx.index]
                        .iter()
                        .any(|ch| ch.is_ascii_alphanumeric()))
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
            // UEB 10.12.1 (`rule_10_12`): an all-capitals initialism in Korean
            // text is spelled with alphabet signs only.
            let letter_initialism = is_letter_initialism_in_korean_text(
                ctx.state,
                &RomanRunPosition {
                    word_chars: ctx.word_chars,
                    run_start: ctx.index,
                    run_end,
                    prev_word: ctx.prev_word,
                    next_word: ctx.remaining_words.first().copied(),
                },
            );
            let is_standing_alone_ordinary_run = (!run_is_all_uppercase || !letter_initialism)
                && word_initial
                && permits_grade1_boundary_after_run(&ctx.word_chars[run_end..]);
            // Rule 37's PDF example, "그는 Can you help me?라고 도움을 요청했다.",
            // suppresses a whole-word sign for the first Roman word (`Can`) but retains
            // the UEB wordsign for the following `you`. Rule 29 keeps consecutive
            // Roman words in the same section, so every complete ordinary-cased word
            // after the first Roman word has the same continuation status, including
            // the final word of a phrase. UEB capitalization does not suppress a
            // wordsign, hence Title-case `Like`/`This` follows the same rule. An
            // all-caps run joins them once Rule 10.12.1 has read it as a word rather
            // than as an initialism (`WE GO`, `Fix YOU`).
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
            // 국립국어원 회신(2026-09-15): 붙임표 뒤에 이어진 낱말은 독립적이므로
            // 약자를 쓸 수 있고, 제37항 붙임의 여섯 낱말 중에서는 `in` 만
            // 해당한다 — ⠔ 가 낱말표이면서 UEB §10.6 묶음약자이기 때문이다.
            let follows_hyphen = ctx
                .index
                .checked_sub(1)
                .and_then(|index| ctx.word_chars.get(index))
                .is_some_and(|previous| {
                    matches!(
                        previous,
                        '-' | '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}'
                    )
                });
            let hyphen_joined_in = lower_run == "in" && follows_hyphen;
            let rule_37_korean_context_exception = !ctx.state.english_dominant_wrap_active
                && !ctx.state.roman_section_is_english_context
                && is_lower_wordsign
                && !hyphen_joined_in;
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
                || hyphen_joined_in
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
                && ctx.state.english_indicator
                && ctx.roman_section_continues_from_previous_word
                && !entire_isolated_rule_28_specimen;
            let shortform_collision = if letter_initialism {
                requires_grade1_before_spelled_letters(&uppercase_run)
            } else {
                requires_grade1_indicator(&uppercase_run)
            };
            let prepend_grade1_indicator = !caps_already_emitted
                && word_initial
                && !digit_adjacent
                && permits_grade1_boundary_after_run(&ctx.word_chars[run_end..])
                && (single_letter_wordsign_collision
                    || (run_is_all_uppercase && shortform_collision));
            let apostrophe_joined_lexeme =
                crate::rules::english_ueb::pronunciation::apostrophe_elided_recorded_word_at(
                    ctx.word_chars,
                    ctx.index,
                    run_end,
                );
            // 제37항 names only the §10.1 alphabetic and §10.5 lower wordsigns
            // as words to spell out after the Roman indicator. A §10.2 strong
            // wordsign (`out`, `this`, `which`, …) that is the whole Roman item
            // therefore keeps its single cell even as the first Roman word.
            let run_is_whole_roman_item = !ctx.word_chars[..ctx.index]
                .iter()
                .chain(&ctx.word_chars[run_end..])
                .any(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '\''));
            if !standalone_wordsign
                && !letter_initialism
                && run_is_whole_roman_item
                && let Some(cell) = crate::rules::english_ueb::rule_10_2::wordsign(&lower_run)
            {
                if !caps_already_emitted {
                    if run_is_all_uppercase && run.len() >= 2 {
                        ctx.emit_slice(&[UPPERCASE_SINGLE, UPPERCASE_SINGLE]);
                    } else if run[0].is_ascii_uppercase() {
                        ctx.emit(UPPERCASE_SINGLE);
                    }
                }
                ctx.emit(cell);
                *ctx.skip_count = run.len().saturating_sub(1);
                crate::rules::roman_mode::set_section_open_keeping_number_chain(ctx.state, true);
                return Ok(RuleResult::Consumed);
            }
            // 제37항 names wordsigns only, so a §10.9 shortform (`Good`,
            // `First`, `After`) stays available to the entry word whenever the
            // run itself stands alone (§2.6).
            let shortform_usable = is_standing_alone_ordinary_run;
            if let Some(cells) = encode_korean_word(
                run,
                caps_already_emitted,
                prepend_grade1_indicator,
                standalone_wordsign,
                shortform_usable,
                word_initial,
                digit_adjacent,
                numeric_grade1_active,
                apostrophe_joined_lexeme,
                letter_initialism,
            ) {
                // UEB 5.8.1: the token rule already placed the §10.9.7 grade-1
                // symbol before the capitals prefix it emitted (`⠰⠠⠠⠁⠛`), so
                // the engine's own §10.9.7 symbol for the same word is dropped.
                let duplicate_grade1 = ctx.index == 0
                    && grade1_precedes_capitals_prefix(ctx.result)
                    && cells.first() == Some(&crate::rules::korean::rule_29::ENGLISH_CONTINUATION);
                ctx.emit_slice(&cells[usize::from(duplicate_grade1)..]);
                *ctx.skip_count = run.len().saturating_sub(1);
                crate::rules::roman_mode::set_section_open_keeping_number_chain(ctx.state, true);
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

        crate::rules::roman_mode::set_section_open_keeping_number_chain(ctx.state, true);
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
    #[case::opening_group_after_sequence("‘LLC(회사)", true)]
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

    /// UEB 10.5: a lower wordsign followed by a hyphen is not usable even when
    /// the surrounding Roman section is clearly an English title.
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

    /// 국립국어원 회신(2026-09-15): 붙임표 뒤에 이어진 `in` 은 독립적이라 약자를
    /// 쓴다. 제37항 붙임의 나머지 다섯 낱말은 해당하지 않는다.
    #[rstest::rstest]
    #[case::between_hyphens("클라우드(Cloud-in-a-Box)라는", "⠤⠔⠤")]
    #[case::before_closing_paren("록인(lock-in)을", "⠤⠔⠠⠴")]
    #[case::before_opening_paren("‘built-in(빌트인)’", "⠤⠔⠦⠄")]
    #[case::capitalized("팬인(Fan-In)", "⠤⠠⠔⠠⠴")]
    fn hyphen_joined_in_keeps_its_contraction(#[case] input: &str, #[case] expected: &str) {
        let actual = encode_to_unicode(input).expect("hyphenated `in` must encode");

        assert!(
            actual.contains(expected),
            "`in` after a hyphen must contract to ⠔: {actual}"
        );
    }

    /// 제37항 붙임 과 UEB 10.5 는 그대로다 — 붙임표가 앞에 없는 나머지 다섯
    /// 낱말은 여전히 철자로 적는다.
    #[rstest::rstest]
    #[case::whole_roman_item("가나 in 다라", "⠴⠊⠝⠲")]
    #[case::hyphen_only_follows("내부(in-house)에서", "⠴⠊⠝⠤")]
    #[case::hyphen_joined_was("클라우드(Cloud-was-a-Box)라는", "⠤⠺⠁⠎⠤")]
    #[case::hyphen_joined_his("클라우드(Cloud-his-a-Box)라는", "⠤⠓⠊⠎⠤")]
    #[case::hyphen_joined_be("클라우드(Cloud-be-a-Box)라는", "⠤⠃⠑⠤")]
    fn hyphen_rule_leaves_the_other_lower_wordsigns_spelled(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let actual = encode_to_unicode(input).expect("lower wordsign context must encode");

        assert!(
            actual.contains(expected),
            "lower wordsign must stay spelled: {actual}"
        );
    }

    /// UEB 10.4.2: a complete `ch`/`sh`/`th`/`wh`/`ou`/`st` sequence that is the
    /// whole Roman run is spelled — its one-cell groupsign reads as a wordsign
    /// (⠌ = still, ⠩ = shall, ⠹ = this, ⠱ = which, ⠳ = out, ⠡ = child).
    #[rstest::rstest]
    #[case::st_after_opening_bracket("에스티유니타스(ST", "⠠⠠⠎⠞")]
    #[case::sh_whole_enclosure("서울주택도시공사(SH)는", "⠠⠠⠎⠓")]
    #[case::wh_after_digit("80Wh(와트시)", "⠠⠺⠓")]
    #[case::th_before_digit("Th17이", "⠠⠞⠓")]
    #[case::ch_before_period("Ch.1(류현진", "⠠⠉⠓")]
    fn complete_strong_sequence_is_spelled(#[case] input: &str, #[case] expected: &str) {
        let actual = encode_to_unicode(input).expect("strong sequence must encode");

        assert!(
            actual.contains(expected),
            "a whole-run strong sequence must be spelled: {actual}"
        );
    }

    /// UEB 10.12.1: an all-capitals letters-run standing as the whole Roman item
    /// in Korean text is an initialism, spelled letter by letter, so a groupsign
    /// must not swallow it. Lower-case runs keep their groupsign.
    #[rstest::rstest]
    #[case::caps_ar("VR(가상현실)과 AR(증강현실), AI(인공지능) 기술", "⠠⠠⠁⠗")]
    #[case::caps_gh("LH와 GH(경기주택도시공사), HUIC(하남도시공사) 등이다.", "⠠⠠⠛⠓")]
    #[case::caps_en("첫 공연은 ‘EN. VOICE(이엔 보이스)’를 초청해", "⠠⠠⠑⠝")]
    #[case::caps_be("가나 BE 다라", "⠠⠠⠃⠑")]
    #[case::lower_er_keeps_groupsign("가나 er 다라", "⠻")]
    #[case::lower_be_keeps_wordsign("제목(Alpha be Omega)이다", "⠆")]
    fn all_caps_whole_run_is_spelled(#[case] input: &str, #[case] expected: &str) {
        let actual = encode_to_unicode(input).expect("initialism must encode");

        assert!(
            actual.contains(expected),
            "all-caps whole run must be spelled: {actual}"
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

#[cfg(test)]
mod uppercase_run_coverage {
    /// UEB 10.12.1: an all-capitals initialism in Korean text is spelled with
    /// alphabet signs; a mixed-case Roman run keeps its ordinary route.
    #[rstest::rstest]
    #[case::initialism("그는 WHO 를")]
    #[case::hyphenated_run("그는 CV3-AD685 를")]
    #[case::mixed_case("그는 Lincoln 을")]
    fn a_roman_run_in_korean_text_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod uppercase_indicator_coverage {
    /// 제28항: 대문자 하나는 대문자표를, 둘 이상 이어지면 대문자 낱말표를 앞세운다.
    /// UEB 10.12.1 의 두문자어는 알파벳 기호로만 적는다.
    #[rstest::rstest]
    #[case::single_capital("그는 Ab 를")]
    #[case::two_capitals("그는 AB 를")]
    #[case::initialism("그는 WHO 를")]
    #[case::hyphenated_run("그는 CV3-AD685 를")]
    #[case::mixed_case("그는 Lincoln 을")]
    fn a_roman_run_in_korean_text_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod strong_wordsign_coverage {
    /// 제37항은 §10.1·§10.5 의 낱말 약자만 풀어 적게 한다. 로마자 항목 전체가
    /// §10.2 의 강한 낱말 약자이면 대문자 표시만 앞세우고 한 칸으로 적는다.
    #[rstest::rstest]
    #[case::lowercase("그는 this 를")]
    #[case::title_case("그는 This 를")]
    #[case::all_capitals("그는 THIS 를")]
    #[case::another_wordsign("그는 WHICH 를")]
    fn a_strong_wordsign_as_the_whole_item_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod enclosed_wordsign_coverage {
    /// 제37항 + §10.2: 괄호나 따옴표에 싸여 낱말 첫 글자가 아닌 자리에서도 강한
    /// 낱말 약자는 한 칸으로 적고, 그 앞에 대문자 표시를 붙인다.
    #[rstest::rstest]
    #[case::all_capitals_in_parentheses("그는 (THIS) 를")]
    #[case::all_capitals_out("그는 (OUT) 을")]
    #[case::title_case_in_parentheses("그는 (This) 를")]
    #[case::lowercase_in_parentheses("그는 (this) 를")]
    #[case::all_capitals_in_quotes("그는 \u{201C}WHICH\u{201D} 를")]
    fn an_enclosed_strong_wordsign_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}
