use std::borrow::Cow;

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::{EncodingMode, RuleContext};
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "72",
    subsection: None,
    name: "placeholder_markers",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.72",
    description: "Single list and placeholder markers without grouping suffix",
};

const MAPPINGS: &[(char, &str)] = &[
    ('○', "⠸⠴"),
    ('□', "⠸⠶"),
    ('△', "⠸⠬"),
    ('▲', "⠸⠬"),
    ('▴', "⠸⠬"),
    ('•', "⠸⠲"),
    ('◎', "⠸⠴⠴"),
    ('▣', "⠸⠶⠶"),
];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

pub fn is_rule_72_symbol(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c)
}

fn is_vertex_decoration(c: char) -> bool {
    matches!(c, '\'' | '′' | '″' | '\u{2034}') || ('\u{2070}'..='\u{209f}').contains(&c)
}

fn consume_triangle_name(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start) != Some(&'△') {
        return None;
    }

    let mut index = start + 1;
    for _ in 0..3 {
        if !chars.get(index).is_some_and(char::is_ascii_uppercase) {
            return None;
        }
        index += 1;
        while chars.get(index).is_some_and(|c| is_vertex_decoration(*c)) {
            index += 1;
        }
    }
    Some(index)
}

/// 수학 점자 제40·42·43항의 삼각형 이름은 `△` 뒤에 꼭짓점 대문자
/// 세 개를 붙여 쓴다 (`△ABC`, `△A′B′C′`). 합동·닮음 관계로 같은
/// 형태가 이어지는 경우도 제72항 글머리 기호로 재해석하지 않는다.
fn is_triangle_geometry_expression(chars: &[char]) -> bool {
    let Some(mut index) = consume_triangle_name(chars, 0) else {
        return false;
    };

    loop {
        if index == chars.len() {
            return true;
        }
        if chars[index..]
            .iter()
            .all(|c| matches!(*c, ',' | '.' | ';' | '?' | '!'))
        {
            return true;
        }
        if !matches!(chars[index], '=' | '≡' | '≅' | '∼' | '∽' | '≈') {
            return false;
        }
        index += 1;
        let Some(next) = consume_triangle_name(chars, index) else {
            return false;
        };
        index = next;
    }
}

fn another_marker_item_exists(tokens: &[Token<'_>], index: usize) -> bool {
    tokens.iter().enumerate().any(|(i, token)| {
        i != index
            && matches!(
                token,
                Token::Word(word) if matches!(word.chars.first(), Some('△' | '▲' | '▴'))
            )
    })
}

fn owned_word(text: String) -> Token<'static> {
    let chars = text.chars().collect::<Vec<_>>();
    let meta = WordMeta::from_chars(&chars);
    Token::Word(WordToken {
        text: Cow::Owned(text),
        chars,
        meta,
    })
}

/// 제72항 글머리 기호가 항목 내용에 붙은 일반 텍스트를, 수식 판정보다 먼저
/// `기호`와 `내용`으로 가른다. 제72항의 예제는 모두 묵자에 한 칸이 있는 꼴
/// (`□ 2021 세계한국어한마당`)이고 규정은 칸을 새로 넣으라고 하지 않으므로,
/// 묵자에 칸이 없으면 붙여 적는다. 수학 제40·42·43항 문법은 아래 구조 판정으로
/// 제외한다.
pub struct Rule72AttachedMarkerTokenRule;

impl TokenRule for Rule72AttachedMarkerTokenRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        90
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };
        let Some(marker) = word.chars.first().copied() else {
            return Ok(TokenAction::Noop);
        };
        if !matches!(marker, '△' | '▲' | '▴') || word.chars.len() == 1 || word.meta.has_korean
        {
            return Ok(TokenAction::Noop);
        }
        // 제57항의 반복 가림표는 하나의 묶음이다. 첫 `△`를 제72항의
        // 글머리 기호로 떼어 내면 문자 규칙이 반복 개수를 볼 수 없으므로,
        // 같은 표지가 연속될 때에는 원래 토큰을 그대로 둔다.
        if word.chars.get(1) == Some(&marker) {
            return Ok(TokenAction::Noop);
        }
        // 수학 제42·43항의 관계식은 한 토큰 안에서 이어지므로, 같은 문장에
        // 다른 `△` 항목이 따로 있으면 제72항의 글머리 기호 나열이다.
        if marker == '△'
            && is_triangle_geometry_expression(&word.chars)
            && !another_marker_item_exists(tokens, index)
        {
            return Ok(TokenAction::Noop);
        }

        let rest = word.chars[1..].iter().collect::<String>();
        Ok(TokenAction::ReplaceMany(vec![
            owned_word(marker.to_string()),
            owned_word(rest),
        ]))
    }
}

pub struct Rule72;

impl BrailleRule for Rule72 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        80
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_rule_72_symbol(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let current = ctx.current_char();
        let repeated = ctx.prev_char() == Some(current) || ctx.next_char() == Some(current);
        if repeated && matches!(current, '○' | '△' | '□') {
            return Ok(RuleResult::Skip);
        }

        // 명시적인 사물부호 문맥(제49항)과 수학 제40항의 `△ABC`는
        // 제72항의 동형 글머리 기호보다 우선한다.
        if matches!(ctx.state.current_mode(), EncodingMode::ObjectSymbol)
            || is_triangle_geometry_expression(&ctx.word_chars[ctx.index..])
        {
            return Ok(RuleResult::Skip);
        }

        // 일반 텍스트 추출 과정에서 `△항목`, `△R&D`, `△2025`처럼 글머리
        // 기호와 항목 내용의 경계가 사라질 수 있다. 제72항 공식 예제처럼
        // 둘 사이 한 칸을 복원하되, 문자 종류를 열거하지 않고 비공백 내용이
        // 실제로 이어지는지만 판정한다.
        let tight_before_content = matches!(current, '△' | '▲' | '▴')
            && ctx.next_char().is_some_and(|c| !c.is_whitespace());
        let contextual_marker = ctx.word_len() == 1
            || ctx
                .next_char()
                .is_some_and(|c| c.is_whitespace() || matches!(c, '(' | '\'' | '"'))
            || tight_before_content
            || matches!(current, '◎' | '▣');
        if !contextual_marker {
            return Ok(RuleResult::Skip);
        }

        let Some((_, unicode)) = MAPPINGS.iter().find(|(candidate, _)| *candidate == current)
        else {
            return Ok(RuleResult::Skip);
        };
        let encoded = encode_unicode_cells(unicode);
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_to_char(input: char) -> (RuleResult, Vec<u8>) {
        let mut owned = crate::test_helpers::CtxOwned::for_text(&input.to_string(), false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule72.apply(&mut ctx).unwrap();
        (outcome, ctx.result.clone())
    }

    #[rstest::rstest]
    #[case::circle('○', true)]
    #[case::square('□', true)]
    #[case::latin('A', false)]
    fn detects_rule_72_symbols(#[case] input: char, #[case] expected: bool) {
        assert_eq!(is_rule_72_symbol(input), expected);
    }

    #[rstest::rstest]
    #[case::circle('○', "⠸⠴")]
    #[case::square('□', "⠸⠶")]
    #[case::triangle('△', "⠸⠬")]
    #[case::bullet('•', "⠸⠲")]
    #[case::filled_triangle('▲', "⠸⠬")]
    #[case::small_filled_triangle('▴', "⠸⠬")]
    #[case::double_circle('◎', "⠸⠴⠴")]
    #[case::filled_square('▣', "⠸⠶⠶")]
    fn apply_encodes_placeholder_markers(#[case] input: char, #[case] expected: &str) {
        let (outcome, output) = apply_to_char(input);

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(output, encode_unicode_cells(expected));
    }

    fn cell_after_triangle_marker(input: &str) -> Option<char> {
        let encoded = crate::encode_to_unicode(input).expect("encodes");
        let at = encoded.find("⠸⠬").expect("triangle marker is emitted");
        encoded[at + "⠸⠬".len()..].chars().next()
    }

    #[rstest::rstest]
    #[case::outline("△문화")]
    #[case::filled("▲문화")]
    #[case::small_filled("▴문화")]
    #[case::roman_item("목록은 △R&D이다")]
    #[case::numeric_item("목록은 △2025년이다")]
    #[case::quoted_item("목록은 △‘첫째’이다")]
    #[case::roman_token("목록은 △AI")]
    #[case::numeric_token("목록은 △2025")]
    fn attached_triangle_list_markers_stay_attached(#[case] input: &str) {
        assert_ne!(cell_after_triangle_marker(input), Some('\u{2800}'));
    }

    #[rstest::rstest]
    #[case::outline("△ 문화")]
    #[case::roman_item("목록은 △ R&D이다")]
    #[case::numeric_token("목록은 △ 2025")]
    fn spaced_triangle_list_markers_keep_their_blank(#[case] input: &str) {
        assert_eq!(cell_after_triangle_marker(input), Some('\u{2800}'));
    }

    #[test]
    fn repeated_triangle_stays_grouped_for_rule_57() {
        assert_eq!(crate::encode_to_unicode("△△").unwrap(), "⠸⠬⠬⠇");
    }

    #[test]
    fn list_marker_between_attached_items_emits_the_marker_alone() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("첫째△둘째", false);
        let mut ctx = owned.ctx_at(2);

        let outcome = Rule72.apply(&mut ctx).unwrap();

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(&*ctx.result, &encode_unicode_cells("⠸⠬"));
    }

    #[test]
    fn math_triangle_name_is_not_reinterpreted_as_a_list_marker() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("△ABC", false);
        let mut ctx = owned.ctx_at(0);

        let outcome = Rule72.apply(&mut ctx).unwrap();

        assert!(matches!(outcome, RuleResult::Skip));
        assert!(ctx.result.is_empty());
    }

    #[rstest::rstest]
    #[case::plain("△ABC")]
    #[case::primed("△A′B′C′")]
    #[case::congruent("△ABC≡△DEF")]
    #[case::similar_primed("△ABC∽△A′B′C′")]
    #[case::trailing_punctuation("△ABC.")]
    fn recognizes_math_triangle_grammar(#[case] input: &str) {
        let chars = input.chars().collect::<Vec<_>>();
        assert!(is_triangle_geometry_expression(&chars));
    }

    #[rstest::rstest]
    #[case::acronym_with_gloss("△UAM(도심항공교통)")]
    #[case::brand_with_digits("△G3930P")]
    #[case::numeric_item("△2025")]
    #[case::incomplete_second_triangle("△ABC=△AB")]
    fn attached_list_items_do_not_match_triangle_geometry(#[case] input: &str) {
        let chars = input.chars().collect::<Vec<_>>();
        assert!(!is_triangle_geometry_expression(&chars));
    }

    #[test]
    fn attached_marker_rule_ignores_an_empty_word_token() {
        let tokens = vec![Token::Word(WordToken {
            text: Cow::Borrowed(""),
            chars: Vec::new(),
            meta: WordMeta::from_chars(&[]),
        })];
        let mut state = crate::rules::context::EncoderState::new(false);

        assert!(matches!(
            Rule72AttachedMarkerTokenRule
                .apply(&tokens, 0, &mut state)
                .expect("empty input is a no-op"),
            TokenAction::Noop
        ));
    }

    #[test]
    fn detects_double_circle_placeholder_symbol() {
        assert!(is_rule_72_symbol('◎'));
    }

    #[test]
    fn metadata_reports_rule_72_identity() {
        assert_eq!(Rule72.meta().section, "72");
        assert_eq!(Rule72.phase(), Phase::CoreEncoding);
        assert_eq!(Rule72.priority(), 80);
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule72.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}

#[cfg(test)]
mod bullet_tail_coverage {
    /// 제72항: a bullet owns the item that follows it, whether that item runs
    /// to the end of the line or stops at punctuation.
    #[rstest::rstest]
    #[case::bullet_then_item("\u{25CB} 정원은 넓다")]
    #[case::bullet_then_punctuation("\u{25CB} 정원.")]
    #[case::bullet_alone("\u{25CB}")]
    #[case::bullet_then_bullet("\u{25CB} \u{25A1} 정원")]
    #[case::item_runs_to_the_end("\u{25CB} 정원")]
    #[case::item_then_comma("\u{25CB} 정원, 마당")]
    fn a_bullet_item_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod bullet_item_coverage {
    /// 제72항: 글머리 기호는 뒤 항목을 거느린다. 항목이 줄 끝까지 가든 문장 부호에서
    /// 끊기든 마찬가지다.
    #[rstest::rstest]
    #[case::bullet_then_item("\u{25CB} 정원은 넓다")]
    #[case::bullet_then_punctuation("\u{25CB} 정원.")]
    #[case::bullet_alone("\u{25CB}")]
    #[case::bullet_then_bullet("\u{25CB} \u{25A1} 정원")]
    #[case::item_runs_to_the_end("\u{25CB} 정원")]
    #[case::item_then_comma("\u{25CB} 정원, 마당")]
    #[case::triangle_bullet("\u{25B3} 정원")]
    fn a_bullet_item_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod repeated_marker_coverage {
    /// 제72항: 같은 글머리 기호가 여러 항목을 이끌면 그 기호는 이름이 아니라
    /// 글머리 기호다.
    #[rstest::rstest]
    #[case::two_triangles("\u{25B3} 정원 \u{25B3} 마당")]
    #[case::one_triangle("\u{25B3} 정원")]
    #[case::filled_triangle("\u{25B2} 정원 \u{25B2} 마당")]
    fn a_repeated_triangle_marker_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}

#[cfg(test)]
mod triangle_geometry_coverage {
    /// 제72항: `△ABC` 꼴은 도형 이름이다. 같은 삼각형 기호가 다른 항목도 이끌고
    /// 있으면 그때는 글머리 기호로 본다.
    #[rstest::rstest]
    #[case::two_geometry_names("\u{25B3}ABC 와 \u{25B3}DEF 는 합동이다")]
    #[case::one_geometry_name("\u{25B3}ABC 는 정삼각형이다")]
    #[case::filled_geometry("\u{25B2}ABC 와 \u{25B2}DEF")]
    fn a_triangle_geometry_name_encodes(#[case] input: &str) {
        assert!(crate::encode_to_unicode(input).is_ok());
    }
}
