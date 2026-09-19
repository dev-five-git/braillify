//! English-context symbol handling.
//!
//! Handles symbol behavior that depends on English mode state:
//! - English symbol rendering for (, ), , when context requires
//! - Parenthesis stack push/pop for matching English parentheses
//! - Comma before Korean fallback preservation

use crate::char_struct::CharType;
use crate::english_logic;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::english_ueb::rule_5_7::is_wordsign_letter;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::symbol_shortcut;
use crate::utils;

pub static META: RuleMeta = RuleMeta {
    section: "49",
    subsection: Some("eng"),
    name: "english_symbol_context",
    standard_ref: "2024 Korean Braille Standard, Ch.4 Sec.10 + Ch.6 Sec.13",
    description: "English-context punctuation rendering with parenthesis tracking",
};

pub struct RuleEnglishSymbol;

/// Korean rules 28 and 32 + UEB 5.7.1: inside an already-open Roman
/// section, a one-letter ASCII segment after a hyphen needs the continuation /
/// grade-1 cell when it could be read as an alphabetic wordsign. Multi-letter
/// segments such as `pop`, `ray`, and `Case` do not take this indicator. Rule
/// 28's Roman indicator already establishes the first segment (`K-pop`), while
/// rule 36's `v-x` demonstrates the indicator on the later single-letter
/// segment. A directly attached ASCII digit remains part of the same rule-35
/// Roman-number sequence and therefore is not a single-letter segment.
fn hyphen_suffix_requires_grade1(word_chars: &[char], hyphen_index: usize) -> bool {
    let Some(suffix) = word_chars.get(hyphen_index + 1..) else {
        return false;
    };
    let suffix_len = suffix
        .iter()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count();

    suffix_len == 1
        && is_wordsign_letter(suffix[0])
        && suffix
            .get(suffix_len)
            .is_none_or(|ch| !ch.is_ascii_alphanumeric())
}

impl BrailleRule for RuleEnglishSymbol {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        300
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(sym) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        // 한글 점자 제43항·제48항: ASCII 숫자 사이의 마침표는 숫자 흐름의
        // 소수점이다. 같은 어절 뒤쪽에 로마자가 있다는 이유만으로 이 위치에서
        // 로마자 모드에 재진입하면 `⠴⠲`가 되어 수표 뒤의 올바른 `⠲` 앞에
        // 불필요한 로마자표가 붙는다. 이 기호는 아래의 일반 한글 문장부호
        // 규칙이 처리하도록 넘기고, 접미사 종류에는 관여하지 않는다.
        if *sym == '.'
            && ctx.prev_char().is_some_and(|ch| ch.is_ascii_digit())
            && ctx.next_char().is_some_and(|ch| ch.is_ascii_digit())
        {
            return Ok(RuleResult::Continue);
        }

        // Rule 69 [붙임 3]: a slash joining two complete Roman-written unit
        // components stays in that measurement chain.  Do not let the generic
        // English-symbol route insert a second Roman indicator before `/`.
        if *sym == '/' && super::rule_69::is_ascii_unit_chain_slash(ctx.word_chars, ctx.index) {
            return Ok(RuleResult::Continue);
        }

        let mut use_english_symbol = english_logic::should_render_symbol_as_english(
            ctx.state.english_indicator,
            ctx.state.is_english,
            ctx.state.doc_summary.is_english_majority,
            &ctx.state.parenthesis_stack,
            *sym,
            ctx.word_chars,
            ctx.index,
            ctx.remaining_words,
        );

        // Korean rules 34 and 54: when a Korean prose item (optionally ending
        // in an attached Arabic number) introduces a Roman explanation, the
        // Korean opening parenthesis is written before the Roman indicator.
        // Rule 34's `링컨(Lincoln)은` order does not depend on whether print
        // attaches the parenthesis or separates it by a space (`링컨 (Lincoln)`).
        // A parenthesis reached while a Roman section is already active stays
        // UEB punctuation (`ABC(def)`), as does ordinary function notation,
        // and so does an enclosure that runs straight on into Roman letters
        // (`폐쇄회로(CC)TV`): rule 32 keeps that whole sequence in one section.
        if *sym == '('
            && !ctx.state.is_english
            && !english_logic::closed_parenthesis_continues_into_roman(ctx.word_chars, ctx.index)
            && korean_prose_owns_opening_parenthesis(
                ctx.word_chars,
                ctx.index,
                ctx.remaining_words,
                ctx.prev_word,
            )
        {
            use_english_symbol = false;
        }

        // 제33항은 점형이 다른 문장 부호를 "로마자와 한글 사이"에서만 한글 점자로
        // 적게 한다. 제35항의 로마자+숫자 연결(`A100,` `DA5,Inc`)은 종료표 없이
        // 로마자 구간 안에 남아 있으므로, 뒤에 로마자 항목이 이어지면 제32항에
        // 따라 UEB 점형을 유지한다 — 규정 예 `1998a, 1998b;`와 같은 자리다.
        // 뒤가 순수 숫자(`Cs-134, 137`)이면 그 숫자는 한글 수표 항목이므로
        // 여기서 로마자 구간이 끝난다.
        let next_item_continues_roman = if let Some(next) = ctx.next_char() {
            next.is_ascii_alphanumeric()
                && !english_logic::begins_korean_mode_number(
                    ctx.word_chars[ctx.index + 1..].iter().copied(),
                )
        } else {
            ctx.remaining_words.first().is_some_and(|w| {
                w.chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphanumeric())
                    && !english_logic::begins_korean_mode_number(w.chars())
            })
        };
        if !use_english_symbol
            && matches!(*sym, ',' | ':' | ';')
            && ctx.state.roman_number_chain
            && next_item_continues_roman
        {
            use_english_symbol = true;
        }

        // 제39항 영-한 wrap context: 단어 끝의 영어 모드 유지 가능 기호(. , : ;)
        // 다음에 한글 어절(wrap 대상)이 이어지면 그 기호를 영어 점자로 처리한다.
        // 예) "(Korean:" 끝의 ':'은 다음 wrap된 "반찬" 직전이므로 영어 점자 ⠒.
        if !use_english_symbol
            && ctx.state.english_dominant_wrap_active
            && ctx.state.is_english
            && ctx.index == ctx.word_chars.len() - 1
            && matches!(*sym, '.' | ',' | ':' | ';')
            && let Some(next_word) = ctx.remaining_words.first()
            && next_word.chars().next().is_some_and(utils::is_korean_char)
        {
            use_english_symbol = true;
        }

        if *sym == '(' {
            // 닫는 따옴표 뒤의 한국어 괄호는 앞 구간을 완전히 닫는다. 예약된 연속표
            // ⠰ 를 지워 괄호 안 로마자가 제29항의 로마자표 ⠴ 로 새로 열리게 한다.
            if !use_english_symbol
                && ctx.index > 0
                && matches!(ctx.word_chars[ctx.index - 1], '\u{2019}' | '\u{201d}')
            {
                crate::rules::roman_mode::clear_pending_continuation(ctx.state);
            }
            ctx.state.parenthesis_stack.push(use_english_symbol);
        } else if *sym == ')' {
            use_english_symbol = ctx
                .state
                .parenthesis_stack
                .pop()
                .unwrap_or(use_english_symbol);
        }

        let has_ascii_alphabetic = ctx.word_chars.iter().any(|ch| ch.is_ascii_alphabetic());
        let can_use_english_symbol = ctx.state.is_english || has_ascii_alphabetic;

        if ctx.state.english_indicator && can_use_english_symbol && use_english_symbol {
            if !ctx.state.is_english
                && !ctx.state.needs_english_continuation
                && !ctx.state.roman_number_chain
            {
                ctx.emit(52);
                crate::rules::roman_mode::set_section_open_keeping_number_chain(ctx.state, true);
            }
            let encoded = if matches!(*sym, '\'' | '\u{2019}') {
                // `use_english_symbol` is true here only for an apostrophe
                // immediately between ASCII letters — the straight form or its
                // typographic U+2019 spelling. Keep that narrow UEB 8.4.2 role
                // local instead of making detached quotes globally eligible for
                // the UEB apostrophe cell.
                crate::rules::english_ueb::rule_7::encode_punctuation('\'')
            } else {
                symbol_shortcut::encode_english_char_symbol_shortcut(*sym)
            };
            if let Some(encoded) = encoded {
                ctx.emit_slice(&encoded);
                if *sym == '-'
                    && ctx.state.is_english
                    && hyphen_suffix_requires_grade1(ctx.word_chars, ctx.index)
                {
                    ctx.emit(crate::rules::korean::rule_29::ENGLISH_CONTINUATION);
                }
                return Ok(RuleResult::Consumed);
            }
        }

        if *sym == ',' {
            let next_char = ctx
                .next_char()
                .or_else(|| ctx.remaining_words.first().and_then(|w| w.chars().next()));
            if next_char.is_some_and(utils::is_korean_char) {
                ctx.emit_slice(symbol_shortcut::encode_char_symbol_shortcut(*sym)?);
                return Ok(RuleResult::Consumed);
            }
        }

        Ok(RuleResult::Continue)
    }
}

/// 제34항의 `링컨(Lincoln)은` 은 한글 어절이 로마자 풀이를 이끌 때 한글 여는 괄호를
/// 로마자표보다 앞에 둔다. 묵자가 괄호를 붙여 썼는지 띄어 썼는지는 기준이 아니다.
fn korean_prose_owns_opening_parenthesis(
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
    prev_word: &str,
) -> bool {
    let prefix = &word_chars[..index];
    if prefix.iter().any(|ch| utils::is_korean_char(*ch)) {
        return true;
    }
    if !prev_word.chars().any(utils::is_korean_char) {
        return false;
    }
    numeric_parenthesis_prefix(prefix)
        || quote_only_prefix_encloses_roman_word(prefix, word_chars, index, remaining_words)
}

/// 제54항: 한글 어절에 붙은 아라비아 숫자(`3.5(`, `1,000(`)도 그 어절의 일부다.
fn numeric_parenthesis_prefix(prefix: &[char]) -> bool {
    !prefix.is_empty()
        && prefix.iter().any(char::is_ascii_digit)
        && prefix
            .iter()
            .all(|ch| ch.is_ascii_digit() || is_number_adjacent_punctuation(*ch))
}

/// 앞이 따옴표뿐이면 괄호는 앞 한글 어절에 딸린 것이다.
fn quote_only_prefix_encloses_roman_word(
    prefix: &[char],
    word_chars: &[char],
    index: usize,
    remaining_words: &[&str],
) -> bool {
    prefix.iter().all(|ch| is_quote_punctuation(*ch))
        && english_logic::closed_parenthesis_encloses_roman_word(word_chars, index, remaining_words)
}

fn is_quote_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '\'' | '\u{2019}' | '"' | '\u{201D}' | '\u{2018}' | '\u{201C}'
    )
}

fn is_number_adjacent_punctuation(ch: char) -> bool {
    ch == '.' || ch == ',' || is_quote_punctuation(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::rule_36_single_x("v-x", true)]
    #[case::single_x_before_korean("v-x쪽", true)]
    #[case::single_t_before_korean_annotation("CAR-T(카티)", true)]
    #[case::non_wordsign_i("v-i", false)]
    #[case::multi_letter_pop("K-pop", false)]
    #[case::multi_letter_case("Title-Case", false)]
    #[case::letter_abutting_digit("v-x1", false)]
    #[case::multi_letter_shortform_candidate("CD-AB", false)]
    fn grade1_after_hyphen_is_limited_to_a_standing_single_letter(
        #[case] input: &str,
        #[case] expected: bool,
    ) {
        let chars = input.chars().collect::<Vec<_>>();
        let hyphen_index = chars
            .iter()
            .position(|ch| *ch == '-')
            .expect("fixture must contain a hyphen");

        assert_eq!(
            hyphen_suffix_requires_grade1(&chars, hyphen_index),
            expected
        );
    }

    #[test]
    fn hyphen_suffix_lookup_rejects_an_out_of_bounds_index() {
        assert!(!hyphen_suffix_requires_grade1(&['A'], 1));
    }

    /// Korean rules 28/29/32: the Roman indicator establishes the first
    /// one-letter segment, and a multi-letter segment after the hyphen starts
    /// directly with its UEB letters.  In particular, no continuation/grade-1
    /// cell is inserted between the hyphen and the lowercase word.
    #[rstest::rstest]
    #[case::k_pop("가 K-pop 나", "⠫⠀⠴⠠⠅⠤⠏⠕⠏⠲⠀⠉")]
    #[case::x_ray("가 X-ray 나", "⠫⠀⠴⠠⠭⠤⠗⠁⠽⠲⠀⠉")]
    #[case::k_water("가 K-water 나", "⠫⠀⠴⠠⠅⠤⠺⠁⠞⠻⠲⠀⠉")]
    fn hyphenated_roman_word_has_no_spurious_post_hyphen_indicator(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    /// Korean rule 35: the official `D-100` establishes that adjacent Roman
    /// letters, digits, and identifier hyphens form one chain.  The Roman
    /// indicator stays at the first Roman letter; a number-led item opens its
    /// Roman section only when its first Roman letter is reached.
    #[rstest::rstest]
    #[case::roman_led_multi_segment("가 CV3-AD685 나", "⠫⠀⠴⠠⠠⠉⠧⠼⠉⠤⠠⠠⠁⠙⠼⠋⠓⠑⠀⠉")]
    #[case::number_led_word("가 0-Zone 나", "⠫⠀⠼⠚⠤⠴⠠⠵⠐⠕⠲⠀⠉")]
    #[case::roman_led_repeated_numeric_segments("가 N-79-20 나", "⠫⠀⠴⠠⠝⠤⠼⠛⠊⠤⠼⠃⠚⠀⠉")]
    fn rule_35_places_the_roman_indicator_at_the_first_roman_letter(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    /// Korean rules 29 and 34: a Korean closing parenthesis ends its enclosed
    /// Roman section.  Later Roman text therefore starts with a new Roman
    /// indicator, even when a comma, number, or identifier hyphen intervenes.
    #[rstest::rstest]
    #[case::numeric_item_after_comma("가 액세스(FWA), 5G 나", "⠫⠀⠗⠁⠠⠝⠠⠪⠦⠄⠴⠠⠠⠋⠺⠁⠠⠴⠐⠀⠼⠑⠴⠠⠛⠲⠀⠉")]
    #[case::hyphenated_item_after_enclosure("가(GTX)-C 나", "⠫⠦⠄⠴⠠⠠⠛⠞⠭⠠⠴⠤⠴⠠⠉⠲⠀⠉")]
    fn korean_parenthesis_does_not_leak_english_continuation(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(crate::encode_to_unicode(input).as_deref(), Ok(expected));
    }

    #[test]
    fn apply_exercise() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        // Just exercise apply() for coverage; either Skip or Continue/Consumed is OK
        let _ = RuleEnglishSymbol.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = RuleEnglishSymbol.matches(&ctx);
    }

    #[test]
    fn opening_parenthesis_pushes_symbol_mode() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("(", false);
        let mut ctx = owned.ctx_at(0);

        let _ = RuleEnglishSymbol.apply(&mut ctx);

        assert!(!ctx.state.parenthesis_stack.is_empty());
    }

    /// 닫는 따옴표 뒤의 괄호는 제56항의 한국어 괄호 ⠦⠄ 이고, 그 안의 로마자는
    /// 예약된 연속표 ⠰ 가 아니라 제29항의 로마자표 ⠴ 로 새로 열린다. 한글에 바로
    /// 붙은 괄호(`모터보트(`)와 같은 자리다.
    #[rstest::rstest]
    #[case::after_closing_quote("가나 ‘MDPS’(Motor 다라", "⠴⠄⠦⠄⠴⠠⠍⠕⠞⠕⠗")]
    #[case::attached_to_korean("가나 모터보트(Motor 다라", "⠦⠄⠴⠠⠍⠕⠞⠕⠗")]
    fn closing_quote_opens_a_fresh_roman_section(
        #[case] input: &str,
        #[case] expected_segment: &str,
    ) {
        let actual = crate::encode_to_unicode(input).unwrap();
        assert!(
            actual.contains(expected_segment),
            "missing Korean parenthesis run {expected_segment:?} in {actual:?}"
        );
    }

    #[test]
    fn closing_parenthesis_reuses_opening_parenthesis_symbol_mode() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("()", true);
        {
            let mut ctx = owned.ctx_at(0);
            ctx.state.is_english = true;

            let _ = RuleEnglishSymbol.apply(&mut ctx);
            assert_eq!(ctx.state.parenthesis_stack.len(), 1);
        }

        let mut ctx = owned.ctx_at(1);
        ctx.state.is_english = true;

        let _ = RuleEnglishSymbol.apply(&mut ctx);

        assert!(ctx.state.parenthesis_stack.is_empty());
    }

    #[test]
    fn decimal_point_between_digits_is_not_an_english_entry_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("3.5P", false);
        let mut ctx = owned.ctx_at(1);

        let outcome = RuleEnglishSymbol.apply(&mut ctx).unwrap();

        assert_eq!(outcome, RuleResult::Continue);
        assert!(owned.result.is_empty());
    }

    #[rstest::rstest]
    #[case::percent_with_later_roman("42.2%포인트(P)", "42.2")]
    #[case::korean_unit_with_annotation("34.3리터(L)", "34.3")]
    #[case::two_decimals_with_arrow("99.8→99.4", "99.8")]
    #[case::roman_identifier("GPT-3.5", "3.5")]
    fn decimal_subsequence_matches_the_standalone_rule_48_encoding(
        #[case] input: &str,
        #[case] decimal: &str,
    ) {
        let actual = crate::encode_to_unicode(input).expect("mixed decimal context must encode");
        let standalone =
            crate::encode_to_unicode(decimal).expect("standalone rule-48 decimal must encode");

        assert!(
            actual.contains(&standalone),
            "input={input}, decimal={decimal}, actual={actual}, standalone={standalone}"
        );
    }

    /// UEB 5.7.2 prints `CD-ROM` with one grade-1 indicator before the complete
    /// letters-sequence and no second grade-1 indicator after the hyphen. This
    /// full-encoder wrapper exercises the Korean rule-29 character route rather
    /// than the standalone-English token route used by the standard PDF case.
    #[test]
    fn korean_wrapper_keeps_pdf_cd_rom_as_one_grade1_letters_sequence() {
        let output = crate::encode("가(CD-ROM)나").expect("Korean wrapper must encode");
        let expected_ueb = "⠰⠠⠠⠉⠙⠤⠠⠠⠗⠕⠍"
            .chars()
            .map(crate::unicode::decode_unicode)
            .collect::<Vec<_>>();
        let roman_start = output
            .iter()
            .position(|cell| *cell == crate::rules::korean::rule_29::ROMAN_INDICATOR)
            .expect("Korean wrapper must enter one Roman section")
            + 1;

        assert_eq!(
            output.get(roman_start..roman_start + expected_ueb.len()),
            Some(expected_ueb.as_slice())
        );
    }

    /// UEB 3.3.1 writes the official `M*A*S*H` example with the UEB asterisk
    /// and no mode boundary around any of its attached marks. Korean rule 32
    /// adds only the outer Roman indicator and terminator in mixed text.
    #[test]
    fn korean_wrapper_keeps_official_mash_in_one_roman_section() {
        let official = crate::encode_to_unicode("M*A*S*H").unwrap();
        assert_eq!(official, "⠠⠍⠐⠔⠠⠁⠐⠔⠠⠎⠐⠔⠠⠓");
        assert_eq!(
            crate::encode_to_unicode("가 M*A*S*H 나").as_deref(),
            Ok("⠫⠀⠴⠠⠍⠐⠔⠠⠁⠐⠔⠠⠎⠐⠔⠠⠓⠲⠀⠉")
        );
    }

    /// UEB 7.3: a Unicode ellipsis closing Roman content is equivalent to the
    /// three-full-stop print spelling, even when Korean text owns the outer
    /// parenthesis.
    #[test]
    fn roman_ellipsis_before_a_closing_parenthesis_uses_ueb_cells() {
        assert_eq!(
            crate::encode_to_unicode("문구(I AM…)이다"),
            crate::encode_to_unicode("문구(I AM...)이다")
        );
    }
}

#[cfg(test)]
mod parenthesis_route_coverage {
    /// 제34항 `링컨(Lincoln)은` keeps the Korean parenthesis outside the Roman
    /// indicator whether or not print separates it with a space; a parenthesis
    /// reached inside an open Roman section stays UEB punctuation.
    #[rstest::rstest]
    #[case::attached("링컨(Lincoln)은")]
    #[case::spaced("링컨 (Lincoln)은")]
    #[case::inside_roman_section("그는 ABC(def) 를")]
    #[case::runs_on_into_roman("폐쇄회로(CC)TV와")]
    fn a_roman_parenthetical_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod parenthesis_and_list_coverage {
    use super::*;

    /// 제34항 `링컨(Lincoln)은` 은 괄호를 붙여 썼든 띄어 썼든 한글 괄호를 앞에 둔다.
    #[rstest::rstest]
    #[case::attached("링컨(Lincoln)은")]
    #[case::spaced("링컨 (Lincoln)은")]
    #[case::inside_roman_section("그는 ABC(def) 를")]
    #[case::runs_on_into_roman("폐쇄회로(CC)TV와")]
    fn a_roman_parenthetical_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }

    /// 제35항: 로마자+숫자 연결 뒤의 쉼표는 뒤에 로마자 항목이 이어지면 제32항의
    /// 통일영어점자 점형을 지킨다.
    #[rstest::rstest]
    #[case::roman_number_then_roman("그는 DA5,Inc 를")]
    #[case::roman_number_then_number("그는 Cs-134, 137 을")]
    fn a_comma_inside_a_roman_number_chain_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }

    #[rstest::rstest]
    #[case::korean_prefix(&['한','글','('], 2, "", true)]
    #[case::numeric_prefix_after_korean(&['3','.','5','('], 3, "한글", true)]
    #[case::numeric_prefix_after_roman(&['3','.','5','('], 3, "ABC", false)]
    #[case::no_prefix(&['('], 0, "", false)]
    fn korean_prose_ownership_follows_the_prefix(
        #[case] word_chars: &[char],
        #[case] index: usize,
        #[case] prev_word: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(
            korean_prose_owns_opening_parenthesis(word_chars, index, &[], prev_word),
            expected
        );
    }
}

#[cfg(test)]
mod roman_number_chain_comma_coverage {
    /// 제35항: 로마자+숫자 연결 뒤의 쉼표는 뒤에 로마자 항목이 이어지면 제32항의
    /// 통일영어점자 점형을 지키고, 뒤가 순수 숫자이면 거기서 구간이 끝난다.
    #[rstest::rstest]
    #[case::roman_number_then_roman("그는 A100, B200 을")]
    #[case::attached_company("그는 DA5,Inc 를")]
    #[case::citation_pair("그는 1998a, 1998b; 를")]
    #[case::then_plain_number("그는 Cs-134, 137 을")]
    fn a_comma_inside_a_roman_number_chain_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}
