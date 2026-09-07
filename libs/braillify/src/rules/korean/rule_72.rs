use std::borrow::Cow;

use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::{EncodingMode, RuleContext};
use crate::rules::token::{SpaceKind, Token, WordMeta, WordToken};
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

fn owned_word(text: String) -> Token<'static> {
    let chars = text.chars().collect::<Vec<_>>();
    let meta = WordMeta::from_chars(&chars);
    Token::Word(WordToken {
        text: Cow::Owned(text),
        chars,
        meta,
    })
}

/// 제72항 글머리 기호가 항목 내용에 붙은 일반 텍스트를, 수식 판정보다
/// 먼저 `기호 + 한 칸 + 내용`으로 복원한다. 수학 제40·42·43항 문법은
/// 위의 구조 판정으로 제외한다.
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
        if marker == '△' && is_triangle_geometry_expression(&word.chars) {
            return Ok(TokenAction::Noop);
        }

        let rest = word.chars[1..].iter().collect::<String>();
        Ok(TokenAction::ReplaceMany(vec![
            owned_word(marker.to_string()),
            Token::Space(SpaceKind::Regular),
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
        if tight_before_content {
            ctx.emit(0);
        }
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

    #[rstest::rstest]
    #[case::outline("△문화", "△ 문화")]
    #[case::filled("▲문화", "△ 문화")]
    #[case::small_filled("▴문화", "△ 문화")]
    #[case::roman_item("목록은 △R&D이다", "목록은 △ R&D이다")]
    #[case::numeric_item("목록은 △2025년이다", "목록은 △ 2025년이다")]
    #[case::quoted_item("목록은 △‘첫째’이다", "목록은 △ ‘첫째’이다")]
    #[case::roman_token("목록은 △AI", "목록은 △ AI")]
    #[case::numeric_token("목록은 △2025", "목록은 △ 2025")]
    fn attached_triangle_list_markers_supply_the_rule_72_boundary(
        #[case] input: &str,
        #[case] standard_print: &str,
    ) {
        assert_eq!(
            crate::encode_to_unicode(input),
            crate::encode_to_unicode(standard_print)
        );
    }

    #[test]
    fn repeated_triangle_stays_grouped_for_rule_57() {
        assert_eq!(crate::encode_to_unicode("△△").unwrap(), "⠸⠬⠬⠇");
    }

    #[test]
    fn list_marker_after_attached_previous_item_still_supplies_right_boundary() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("첫째△둘째", false);
        let mut ctx = owned.ctx_at(2);

        let outcome = Rule72.apply(&mut ctx).unwrap();

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(&*ctx.result, &encode_unicode_cells("⠸⠬⠀"));
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
