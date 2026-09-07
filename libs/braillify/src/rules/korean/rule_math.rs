//! Math symbol encoding with Korean spacing rules.
//!
//! Math symbols (＋, −, ×, ÷, etc.) need spacing around them when
//! adjacent to Korean text, unless the Korean is a grammatical particle (josa).

use crate::char_struct::CharType;
use crate::math_symbol_shortcut;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};
use crate::utils;

pub static META: RuleMeta = RuleMeta {
    section: "math",
    subsection: None,
    name: "math_symbol_encoding",
    standard_ref: "2024 Korean Braille Standard (math symbols)",
    description: "Math symbols with Korean spacing rules",
};

/// Korean particles or copulas that do not form the right-hand operand of an
/// Article 46 expression by themselves.
const NON_OPERAND_KOREAN_SUFFIXES: &[&str] =
    &["과", "와", "의", "이다", "하고", "이랑", "랑", "아니다"];

pub struct RuleMath;

fn matching_opening_delimiter(ch: char) -> Option<char> {
    match ch {
        ')' => Some('('),
        ']' => Some('['),
        '}' => Some('{'),
        '〉' => Some('〈'),
        '》' => Some('《'),
        '」' => Some('「'),
        '』' => Some('『'),
        '】' => Some('【'),
        '〕' => Some('〔'),
        '〗' => Some('〖'),
        '〙' => Some('〘'),
        '〛' => Some('〚'),
        _ => None,
    }
}

fn matching_closing_delimiter(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '〈' => Some('〉'),
        '《' => Some('》'),
        '「' => Some('」'),
        '『' => Some('』'),
        '【' => Some('】'),
        '〔' => Some('〕'),
        '〖' => Some('〗'),
        '〘' => Some('〙'),
        '〚' => Some('〛'),
        _ => None,
    }
}

fn is_operand_separator(ch: char) -> bool {
    matches!(
        ch,
        '+' | '-'
            | '−'
            | '×'
            | '÷'
            | '='
            | '<'
            | '>'
            | '≤'
            | '≥'
            | '≠'
            | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '…'
            | '\''
            | '"'
            | '‘'
            | '’'
            | '“'
            | '”'
    )
}

/// Finds the syntactic operand immediately to the left of an Article 46 sign.
/// A balanced annotation remains part of its operand (`레트로(RETRO)`), while
/// an unmatched opening delimiter is a hard boundary (`기업(+5p)`).
fn left_operand(chars: &[char], operator_index: usize) -> &[char] {
    let mut start = operator_index;
    let mut openings = Vec::new();

    for index in (0..operator_index).rev() {
        let ch = chars[index];
        if let Some(opening) = matching_opening_delimiter(ch) {
            openings.push(opening);
            start = index;
            continue;
        }
        if matching_closing_delimiter(ch).is_some() {
            if openings.last() == Some(&ch) {
                openings.pop();
                start = index;
                continue;
            }
            break;
        }
        if openings.is_empty() && is_operand_separator(ch) {
            break;
        }
        start = index;
    }

    &chars[start..operator_index]
}

/// Finds the syntactic operand immediately to the right of an Article 46 sign.
/// Balanced annotations and numeric unit notation stay inside the operand, but
/// the next top-level sign or unmatched closing delimiter ends it.
fn right_operand(chars: &[char], operator_index: usize) -> &[char] {
    let mut end = operator_index + 1;
    let mut closings = Vec::new();

    for (index, ch) in chars.iter().copied().enumerate().skip(operator_index + 1) {
        if let Some(closing) = matching_closing_delimiter(ch) {
            closings.push(closing);
            end = index + 1;
            continue;
        }
        if matching_opening_delimiter(ch).is_some() {
            if closings.last() == Some(&ch) {
                closings.pop();
                end = index + 1;
                continue;
            }
            break;
        }
        if closings.is_empty() && is_operand_separator(ch) {
            break;
        }
        end = index + 1;
    }

    &chars[operator_index + 1..end]
}

fn first_korean_run(chars: &[char]) -> Option<String> {
    let start = chars.iter().position(|ch| utils::is_korean_char(*ch))?;
    let end = chars[start..]
        .iter()
        .position(|ch| !utils::is_korean_char(*ch))
        .map_or(chars.len(), |offset| start + offset);
    Some(chars[start..end].iter().collect())
}

fn rule_46_requires_padding(ctx: &RuleContext) -> bool {
    let left_is_korean_operand = left_operand(ctx.word_chars, ctx.index)
        .iter()
        .any(|ch| utils::is_korean_char(*ch));
    let right_is_non_suffix_korean_operand =
        first_korean_run(right_operand(ctx.word_chars, ctx.index))
            .is_some_and(|run| !NON_OPERAND_KOREAN_SUFFIXES.contains(&run.as_str()));

    left_is_korean_operand && right_is_non_suffix_korean_operand
}

/// U+002D is both HYPHEN-MINUS, so its braille meaning has to be inferred from
/// syntax.  Treat it as the Article 45 subtraction/minus sign only when the
/// surrounding token makes that role explicit.  In particular, a leading
/// signed number is a minus, while phone numbers, dates, ranges and identifiers
/// such as `02-799-1000` and `A-3` remain hyphenated.
fn is_semantic_ascii_minus(ctx: &RuleContext) -> bool {
    if ctx.current_char() != '-' {
        return false;
    }

    let next_starts_number = ctx.next_char().is_some_and(|next| {
        next.is_ascii_digit()
            || (next == '.'
                && ctx
                    .word_chars
                    .get(ctx.index + 2)
                    .is_some_and(char::is_ascii_digit))
    });
    let unary_boundary = ctx.prev_char().is_none_or(|prev| {
        matches!(
            prev,
            '(' | '['
                | '{'
                | '〈'
                | '《'
                | '「'
                | '『'
                | '【'
                | '〔'
                | '〖'
                | '〘'
                | '〚'
                | '‘'
                | '“'
                | '\''
                | '"'
                | ','
                | ':'
                | ';'
                | '='
                | '+'
                | '×'
                | '÷'
                | '<'
                | '>'
                | '≤'
                | '≥'
                | '≠'
        )
    });
    if next_starts_number && unary_boundary {
        return true;
    }

    // A sign cited by itself inside a matched delimiter is an operator, as in
    // the common polarity notation `양(+)극·음(-)극`. A hyphen joining text has
    // operands on the same side and therefore cannot have this shape.
    let isolated_operator = matches!(
        (ctx.prev_char(), ctx.next_char()),
        (Some('('), Some(')'))
            | (Some('['), Some(']'))
            | (Some('{'), Some('}'))
            | (Some('〈'), Some('〉'))
            | (Some('《'), Some('》'))
            | (Some('「'), Some('」'))
            | (Some('『'), Some('』'))
            | (Some('【'), Some('】'))
            | (Some('〔'), Some('〕'))
            | (Some('〖'), Some('〗'))
            | (Some('〘'), Some('〙'))
            | (Some('〚'), Some('〛'))
            | (Some('‘'), Some('’'))
            | (Some('“'), Some('”'))
            | (Some('\''), Some('\''))
            | (Some('"'), Some('"'))
    );
    if isolated_operator {
        return true;
    }

    // Article 46's printed example `5개-3개=2개` contains Hangul, so the
    // token-level mathematics parser deliberately leaves it to Korean rules.
    // A second explicit operator disambiguates the inner U+002D from a range.
    let has_other_math_operator = ctx.word_chars.iter().enumerate().any(|(index, ch)| {
        index != ctx.index
            && matches!(
                ch,
                '+' | '=' | '×' | '÷' | '<' | '>' | '≤' | '≥' | '≠' | '−'
            )
    });
    let prev_ends_operand = ctx.prev_char().is_some_and(|prev| {
        prev.is_alphanumeric()
            || utils::is_korean_char(prev)
            || matches!(prev, ')' | ']' | '}' | '〉' | '》' | '」' | '』' | '】')
    });

    prev_ends_operand && next_starts_number && has_other_math_operator
}

impl BrailleRule for RuleMath {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::MathSymbol(_))
            || (matches!(ctx.char_type, CharType::Symbol('-')) && is_semantic_ascii_minus(ctx))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let c = match ctx.char_type {
            CharType::MathSymbol(c) => *c,
            CharType::Symbol('-') if is_semantic_ascii_minus(ctx) => '\u{2212}',
            _ => return Ok(RuleResult::Skip),
        };

        // UEB §3.17 + Korean rules 29/35: a plus that belongs to a Roman
        // product/grade identifier stays inside that Roman section and uses
        // the UEB general-symbol cells ⠐⠖.  The token-level grammar has already
        // rejected completed sums and the ambiguous one-letter `A+` shape.
        if c == '+'
            && ctx.state.english_indicator
            && ctx.state.is_english
            && crate::rules::token_rules::math_expression::is_roman_plus_identifier(ctx.word_chars)
        {
            let encoded = crate::rules::english_ueb::rule_3::encode_symbol(c)
                .ok_or_else(|| "UEB plus sign must be defined".to_string())?;
            ctx.emit_slice(&encoded);
            return Ok(RuleResult::Consumed);
        }

        // PDF 제46항 — 사칙연산 기호(+, −, ×, ÷, =)가 한글 사이에
        // 나올 때에만 기호 앞뒤를 한 칸씩 띄어 쓴다.
        //
        // 판정:
        //   - 바로 인접한 피연산자 범위 안에 한글이 각각 있어야 한다.
        //   - 괄호 속 로마자·한글 주석은 그 피연산자에 포함한다.
        //     예: `레트로(RETRO)+뉴트로(NEWTRO)`.
        //   - 괄호 경계나 다른 연산 기호를 넘어 문법적으로 무관한 한글은
        //     찾지 않는다. 예: `기업(+5p)의`, `행사(1+1)이다`.
        //   - 우측 한글 묶음이 비어 있거나 조사·서술격 표현(과/와/의/이다 등)이면
        //     기호 양쪽을 띄어쓰지 않는다.
        //     예: `반지름×3.14이다` → `이다`는 JOSA → 띄어쓰지 않음.
        //     예: `5개−3개=2개` → `개`는 JOSA가 아님 → 띄어씀.
        let pad_spaces = rule_46_requires_padding(ctx);

        if pad_spaces {
            ctx.emit(0);
        }

        let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?;
        ctx.emit_slice(encoded);

        if pad_spaces {
            ctx.emit(0);
        }

        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::parenthesis('(', ')')]
    #[case::square_bracket('[', ']')]
    #[case::curly_brace('{', '}')]
    #[case::single_angle('〈', '〉')]
    #[case::double_angle('《', '》')]
    #[case::corner_bracket('「', '」')]
    #[case::white_corner_bracket('『', '』')]
    #[case::lenticular_bracket('【', '】')]
    #[case::tortoise_shell_bracket('〔', '〕')]
    #[case::white_lenticular_bracket('〖', '〗')]
    #[case::white_tortoise_shell_bracket('〘', '〙')]
    #[case::white_square_bracket('〚', '〛')]
    fn delimiter_pairs_are_bidirectional(#[case] opening: char, #[case] closing: char) {
        assert_eq!(matching_opening_delimiter(closing), Some(opening));
        assert_eq!(matching_closing_delimiter(opening), Some(closing));
    }

    #[test]
    fn apply_exercise() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        // Just exercise apply() for coverage; either Skip or Continue/Consumed is OK
        let _ = RuleMath.apply(&mut ctx);
    }

    #[test]
    fn matches_does_not_panic() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let ctx = owned.ctx_at(0);
        let _ = RuleMath.matches(&ctx);
    }

    #[test]
    fn apply_pads_math_symbol_between_korean_quantity_words() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("5개=2개3", false);
        let mut ctx = owned.ctx_at(2);

        let outcome = RuleMath.apply(&mut ctx).expect("math rule should apply");

        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(owned.result.starts_with(&[0]));
        assert!(owned.result.ends_with(&[0]));
    }

    #[rstest::rstest]
    #[case::plus('+')]
    #[case::times('×')]
    #[case::division('÷')]
    #[case::equals('=')]
    fn parenthesized_math_symbol_does_not_gain_inner_spaces(#[case] operator: char) {
        let input = format!("가({operator})나");
        let mut owned = crate::test_helpers::CtxOwned::for_text(&input, false);
        let mut ctx = owned.ctx_at(2);

        let outcome = RuleMath.apply(&mut ctx).expect("math rule should apply");

        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!owned.result.is_empty());
        assert_ne!(owned.result.first(), Some(&0));
        assert_ne!(owned.result.last(), Some(&0));
    }

    #[rstest::rstest]
    #[case::service("TV+")]
    #[case::alphanumeric_product("HDR10+")]
    #[case::mixed_case_service("U+tv")]
    fn roman_terminal_plus_uses_ueb_general_symbol(#[case] identifier: &str) {
        let output = crate::encode(&format!("가 {identifier} 나"))
            .expect("Roman product identifier must encode");
        let ueb_plus = crate::rules::english_ueb::rule_3::encode_symbol('+')
            .expect("UEB plus must be defined");

        assert!(
            output
                .windows(ueb_plus.len())
                .any(|cells| cells == ueb_plus),
            "identifier={identifier}"
        );
    }

    #[rstest::rstest]
    #[case::plus_math_symbol("양", "+", "극")]
    #[case::ascii_hyphen_minus_symbol("음", "-", "극")]
    fn full_encoder_preserves_tight_parenthesized_operator(
        #[case] left: &str,
        #[case] operator: &str,
        #[case] right: &str,
    ) {
        let input = format!("{left}({operator}){right}");
        let expected = [left, &format!("({operator})"), right]
            .into_iter()
            .map(|part| crate::encode_to_unicode(part).expect("component must encode"))
            .collect::<Vec<_>>()
            .concat();

        assert_eq!(
            crate::encode_to_unicode(&input).expect("full input must encode"),
            expected
        );
    }

    #[rstest::rstest]
    #[case::signed_integer("-3", 0, true)]
    #[case::parenthesized_signed_decimal("(-3.5)", 1, true)]
    #[case::quoted_negative_quantity("‘-2배’", 1, true)]
    #[case::pdf_phone_number("02-799-1000", 2, false)]
    #[case::identifier_suffix("A-3", 1, false)]
    #[case::calendar_date("2024-09-03", 4, false)]
    #[case::non_hyphen_character("A", 0, false)]
    fn ascii_hyphen_minus_is_disambiguated_by_syntax(
        #[case] input: &str,
        #[case] index: usize,
        #[case] expected: bool,
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(index);
        assert_eq!(is_semantic_ascii_minus(&ctx), expected, "input={input}");
    }

    #[rstest::rstest]
    #[case::korean_words("나루+배", 2, true)]
    #[case::korean_numeric_units("5개-3개", 2, true)]
    #[case::percentage_noun_operand("팬=51%지분", 1, true)]
    #[case::roman_annotation_on_left("레트로(RETRO)+뉴트로", 10, true)]
    #[case::korean_annotations_on_both_sides("AI(인공지능)+DX(디지털전환)", 8, true)]
    #[case::mixed_script_right_operand("밀레니얼+Z세대", 4, true)]
    #[case::signed_parenthetical("기업(+5p)의", 3, false)]
    #[case::numeric_sum("행사(1+1)이다", 4, false)]
    #[case::brand_particle("디즈니+와", 3, false)]
    #[case::roman_variable_left("T+3일", 1, false)]
    #[case::particle_after_annotated_roman("OPEC(석유수출국기구)+의", 13, false)]
    #[case::quoted_suffix("저소음+’", 3, false)]
    fn rule_46_padding_depends_on_actual_operands(
        #[case] input: &str,
        #[case] index: usize,
        #[case] expected: bool,
    ) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(input, false);
        let ctx = owned.ctx_at(index);
        assert_eq!(rule_46_requires_padding(&ctx), expected, "input={input}");
    }

    #[rstest::rstest]
    #[case::negative_percentage("-2.73%를", "−2.73%를")]
    #[case::negative_unit("체급(-67kg)은", "체급(−67kg)은")]
    #[case::parenthesized_polarity("음(-)극", "음(−)극")]
    #[case::article_46_equation("5개-3개=2개", "5개−3개=2개")]
    fn semantic_ascii_minus_matches_explicit_unicode_minus(
        #[case] ascii: &str,
        #[case] explicit: &str,
    ) {
        assert!(matches!(
            crate::char_struct::CharType::new('-').expect("hyphen-minus must classify"),
            crate::char_struct::CharType::Symbol('-')
        ));
        assert_eq!(
            crate::encode_to_unicode(ascii).expect("ASCII expression must encode"),
            crate::encode_to_unicode(explicit).expect("Unicode expression must encode"),
            "input={ascii}"
        );
    }

    #[rstest::rstest]
    #[case::pdf_phone_number("02-799-1000")]
    #[case::identifier_suffix("A-3")]
    #[case::calendar_date("2024-09-03")]
    fn non_operator_hyphens_do_not_become_minus(#[case] input: &str) {
        let explicit_minus = input.replacen('-', "−", 1);
        assert_ne!(
            crate::encode_to_unicode(input).expect("hyphenated input must encode"),
            crate::encode_to_unicode(&explicit_minus).expect("minus variant must encode"),
            "input={input}"
        );
    }
}
