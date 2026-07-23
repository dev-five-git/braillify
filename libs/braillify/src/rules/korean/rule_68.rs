use crate::char_struct::CharType;
use crate::english;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::korean::rule_29::ROMAN_INDICATOR;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "68",
    subsection: None,
    name: "superscript_subscript_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.68",
    description: "Superscripts, subscripts, and selected compact unit symbols",
};

const MAPPINGS: &[(char, &str)] = &[
    ('㎡', "⠴⠍⠘⠼⠃"),
    ('㏊', "⠴⠓⠁⠲"),
    ('⁺', "⠘⠢"),
    ('⁻', "⠘⠔"),
    ('₆', "⠰⠼⠋"),
    ('₉', "⠰⠼⠊"),
];

const GRADE_MINUS: [u8; 2] = [
    crate::unicode::decode_unicode('⠘'),
    crate::unicode::decode_unicode('⠔'),
];
const SUPERSCRIPT_PREFIX: u8 = crate::unicode::decode_unicode('⠘');
const SUBSCRIPT_PREFIX: u8 = crate::unicode::decode_unicode('⠰');
const NUMBER_PREFIX: u8 = crate::unicode::decode_unicode('⠼');
const ENGLISH_PREFIX: u8 = crate::unicode::decode_unicode('⠴');
const UPPERCASE_PREFIX: u8 = crate::unicode::decode_unicode('⠠');

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

fn should_insert_separator_after_symbol(ctx: &RuleContext) -> bool {
    matches!(ctx.current_char(), '㎡') && matches!(ctx.next_char(), Some('는' | '은'))
}

pub fn is_rule_68_symbol(c: char) -> bool {
    MAPPINGS.iter().any(|(candidate, _)| *candidate == c)
}

fn is_superscript_symbol(c: char) -> bool {
    matches!(c, '⁺' | '⁻')
}

fn is_subscript_digit(c: char) -> bool {
    matches!(c, '₀'..='₉')
}

fn is_grade_notation(word: &[char], index: usize) -> bool {
    matches!(word.get(index), Some(ch) if ch.is_ascii_uppercase())
        && matches!(word.get(index + 1), Some('-'))
        && word.len() == index + 2
}

fn is_compact_ascii_notation(word: &[char], index: usize) -> bool {
    matches!(word.get(index), Some(ch) if ch.is_ascii_uppercase())
        && word
            .get(index + 1)
            .is_some_and(|next| is_superscript_symbol(*next) || is_subscript_digit(*next))
}

fn encode_compact_ascii_notation(
    word: &[char],
    index: usize,
    needs_roman_indicator: bool,
) -> Result<Option<(Vec<u8>, usize)>, String> {
    let Some(base) = word.get(index).copied() else {
        return Ok(None);
    };

    if !base.is_ascii_uppercase() {
        return Ok(None);
    }

    let mut encoded = Vec::new();
    if needs_roman_indicator {
        encoded.push(ENGLISH_PREFIX);
    }
    encoded.push(UPPERCASE_PREFIX);
    encoded.push(english::encode_english(base)?);
    let mut consumed = 1usize;
    let mut cursor = index + 1;

    if word.get(cursor) == Some(&'-') {
        encoded.extend_from_slice(&GRADE_MINUS);
        consumed += 1;
        return Ok(Some((encoded, consumed)));
    }

    if word
        .get(cursor)
        .is_some_and(|ch| is_superscript_symbol(*ch))
    {
        encoded.push(SUPERSCRIPT_PREFIX);
        while let Some(ch) = word.get(cursor).copied() {
            let cell = match ch {
                '⁺' => crate::unicode::decode_unicode('⠢'),
                '⁻' => crate::unicode::decode_unicode('⠔'),
                _ => break,
            };
            encoded.push(cell);
            consumed += 1;
            cursor += 1;
        }
        return Ok(Some((encoded, consumed)));
    }

    if word.get(cursor).is_some_and(|ch| is_subscript_digit(*ch)) {
        encoded.push(SUBSCRIPT_PREFIX);
        encoded.push(NUMBER_PREFIX);
        while let Some(ch) = word.get(cursor).copied() {
            let digit = match ch {
                '₀' => '0',
                '₁' => '1',
                '₂' => '2',
                '₃' => '3',
                '₄' => '4',
                '₅' => '5',
                '₆' => '6',
                '₇' => '7',
                '₈' => '8',
                '₉' => '9',
                _ => break,
            };
            encoded.push(crate::number::encode_number(digit)?);
            consumed += 1;
            cursor += 1;
        }
        return Ok(Some((encoded, consumed)));
    }

    Ok(None)
}

pub struct Rule68;

impl BrailleRule for Rule68 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        90
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_rule_68_symbol(*c))
            || matches!(ctx.char_type, CharType::English(_)
                if is_compact_ascii_notation(ctx.word_chars, ctx.index)
                    || is_grade_notation(ctx.word_chars, ctx.index))
            || (matches!(
                ctx.char_type,
                CharType::MathSymbol('+') | CharType::Symbol('+')
            ) && is_digit_grade_plus_notation(ctx.word_chars, ctx.index))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if matches!(ctx.char_type, CharType::English(_))
            && let Some((encoded, consumed)) = encode_compact_ascii_notation(
                ctx.word_chars,
                ctx.index,
                !ctx.state.is_english && ctx.result.last().copied() != Some(ROMAN_INDICATOR),
            )?
        {
            ctx.emit_slice(&encoded);
            ctx.state.is_english = false;
            ctx.state.needs_english_continuation = false;
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        // PDF — `1++` 같이 digit 뒤 연속 `+`(또는 `-`)는 grade 표기.
        // `⠘`(super marker) + plus chars 연쇄로 점역한다.
        if matches!(
            ctx.char_type,
            CharType::MathSymbol('+') | CharType::Symbol('+')
        ) && is_digit_grade_plus_notation(ctx.word_chars, ctx.index)
        {
            ctx.emit(SUPERSCRIPT_PREFIX);
            let mut cursor = ctx.index;
            let mut consumed = 0usize;
            while let Some(&ch) = ctx.word_chars.get(cursor) {
                let cell = match ch {
                    '+' => crate::unicode::decode_unicode('⠢'),
                    '-' => crate::unicode::decode_unicode('⠔'),
                    _ => break,
                };
                ctx.emit(cell);
                consumed += 1;
                cursor += 1;
            }
            // 등급 표기 뒤에 한글이 오면 분리 공백 추가
            if ctx
                .word_chars
                .get(cursor)
                .is_some_and(|c| crate::utils::is_korean_char(*c))
            {
                ctx.emit(0);
            }
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        let Some((_, unicode)) = MAPPINGS
            .iter()
            .find(|(candidate, _)| *candidate == ctx.current_char())
        else {
            return Ok(RuleResult::Skip);
        };
        let encoded = encode_unicode_cells(unicode);
        ctx.emit_slice(&encoded);
        if should_insert_separator_after_symbol(ctx) {
            ctx.emit(0);
        }
        Ok(RuleResult::Consumed)
    }
}

/// PDF — `1++등급` 같은 digit + 연속 `+` 등급 표기 패턴인지 검사.
/// 직전이 digit이고 현재가 `+`이며 이후에 한글 등급 키워드(등급)가 나오면 true.
fn is_digit_grade_plus_notation(word: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    if !word.get(index - 1).is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    // 현재 위치부터 연속 +/-
    let mut cursor = index;
    while let Some(&ch) = word.get(cursor) {
        if matches!(ch, '+' | '-') {
            cursor += 1;
        } else {
            break;
        }
    }
    // 직후에 한글이 와야 grade context로 본다 (`1++등급` 등).
    word.get(cursor)
        .is_some_and(|c| crate::utils::is_korean_char(*c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_rule_68_symbol_recognises_each_entry() {
        for (c, _) in MAPPINGS {
            assert!(is_rule_68_symbol(*c), "should recognise {c}");
        }
        assert!(!is_rule_68_symbol('a'));
        assert!(!is_rule_68_symbol('1'));
    }

    #[test]
    fn is_superscript_symbol_plus_minus_only() {
        assert!(is_superscript_symbol('⁺'));
        assert!(is_superscript_symbol('⁻'));
        assert!(!is_superscript_symbol('a'));
    }

    #[test]
    fn is_subscript_digit_recognises_each() {
        for c in ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'] {
            assert!(is_subscript_digit(c), "should recognise {c}");
        }
        assert!(!is_subscript_digit('0'));
        assert!(!is_subscript_digit('a'));
    }

    #[test]
    fn is_grade_notation_paths() {
        let word: Vec<char> = "A-".chars().collect();
        assert!(is_grade_notation(&word, 0));
        // Wrong length
        let word: Vec<char> = "A-x".chars().collect();
        assert!(!is_grade_notation(&word, 0));
        // Not uppercase
        let word: Vec<char> = "a-".chars().collect();
        assert!(!is_grade_notation(&word, 0));
        // No dash
        let word: Vec<char> = "A".chars().collect();
        assert!(!is_grade_notation(&word, 0));
    }

    #[test]
    fn is_compact_ascii_notation_paths() {
        let word: Vec<char> = "A⁺".chars().collect();
        assert!(is_compact_ascii_notation(&word, 0));
        let word: Vec<char> = "B₃".chars().collect();
        assert!(is_compact_ascii_notation(&word, 0));
        // Not uppercase
        let word: Vec<char> = "a⁺".chars().collect();
        assert!(!is_compact_ascii_notation(&word, 0));
        // No suffix
        let word: Vec<char> = "A".chars().collect();
        assert!(!is_compact_ascii_notation(&word, 0));
        // Wrong suffix
        let word: Vec<char> = "Ab".chars().collect();
        assert!(!is_compact_ascii_notation(&word, 0));
    }

    #[test]
    fn encode_compact_ascii_notation_grade_minus() {
        let word: Vec<char> = "A-".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, true).unwrap();
        assert!(result.is_some());
        let (_, consumed) = result.unwrap();
        assert_eq!(consumed, 2);
    }

    #[test]
    fn encode_compact_ascii_notation_superscript() {
        let word: Vec<char> = "A⁺⁻".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();
        assert!(result.is_some());
        let (_, consumed) = result.unwrap();
        assert_eq!(consumed, 3);
    }

    #[test]
    fn encode_compact_ascii_notation_subscript() {
        let word: Vec<char> = "B₃".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, true).unwrap();
        assert!(result.is_some());
        let (_, consumed) = result.unwrap();
        assert_eq!(consumed, 2);
    }

    #[rstest::rstest]
    #[case::zero('₀')]
    #[case::one('₁')]
    #[case::two('₂')]
    #[case::four('₄')]
    #[case::five('₅')]
    #[case::seven('₇')]
    #[case::eight('₈')]
    fn encode_compact_ascii_notation_subscript_digit_arms(#[case] digit: char) {
        let word: Vec<char> = format!("B{digit}").chars().collect();

        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();

        assert!(matches!(result, Some((_, 2))));
    }

    #[test]
    fn encode_compact_ascii_notation_non_ascii_returns_none() {
        let word: Vec<char> = "가".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn encode_compact_ascii_notation_lowercase_returns_none() {
        let word: Vec<char> = "a⁺".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn encode_compact_ascii_notation_out_of_bounds() {
        let word: Vec<char> = "A".chars().collect();
        // Returns None because no suffix after A
        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();
        assert!(result.is_none());
        // Out of range index
        let result = encode_compact_ascii_notation(&word, 99, false).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn is_digit_grade_plus_notation_paths() {
        // "1+등급"
        let word: Vec<char> = "1+등급".chars().collect();
        assert!(is_digit_grade_plus_notation(&word, 1));
        // "1++등급"
        let word: Vec<char> = "1++등급".chars().collect();
        assert!(is_digit_grade_plus_notation(&word, 1));
        // Index 0 - not preceded by digit
        assert!(!is_digit_grade_plus_notation(&word, 0));
        // Without Korean following
        let word: Vec<char> = "1++x".chars().collect();
        assert!(!is_digit_grade_plus_notation(&word, 1));
        // Not preceded by digit
        let word: Vec<char> = "a+등급".chars().collect();
        assert!(!is_digit_grade_plus_notation(&word, 1));
    }

    #[test]
    fn rule68_meta_phase_priority() {
        let r = Rule68;
        assert_eq!(r.meta().section, "68");
        assert!(matches!(r.phase(), Phase::CoreEncoding));
        assert_eq!(r.priority(), 90);
    }

    #[test]
    fn rule68_apply_emits_for_known_symbol() {
        use crate::char_struct::CharType;
        let word: Vec<char> = "㎡".chars().collect();
        let ct = CharType::Symbol('㎡');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let res = Rule68.apply(&mut ctx).unwrap();
        assert!(matches!(res, RuleResult::Consumed));
        assert!(!out.is_empty());
    }

    /// 제68항 — `1+등급`: digit + plus + 한글 → SUPERSCRIPT prefix + plus cell
    /// + 분리 공백(line 192-212).
    #[test]
    fn rule68_apply_digit_grade_plus_followed_by_korean() {
        let word: Vec<char> = "1+등급".chars().collect();
        let ct = CharType::MathSymbol('+');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word,
            index: 1,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: true,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule68.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        // First emitted cell is SUPERSCRIPT_PREFIX (⠘)
        assert_eq!(out[0], SUPERSCRIPT_PREFIX);
        // Last emitted cell is space 0 because next is 한글
        assert_eq!(*out.last().unwrap(), 0);
    }

    /// 제68항 — apply path for compact ASCII notation (uppercase + superscript)
    /// triggers `encode_compact_ascii_notation` consumed branch (line 171-183).
    #[test]
    fn rule68_apply_compact_ascii_uppercase_superscript() {
        let word: Vec<char> = "A⁺".chars().collect();
        let ct = CharType::English('A');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: true,
            ascii_starts_at_beginning: true,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule68.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Consumed));
        assert!(!out.is_empty());
    }

    #[test]
    fn should_insert_separator_after_symbol_only_for_specific_pattern() {
        use crate::char_struct::CharType;
        // ㎡는 → true
        let word: Vec<char> = "㎡는".chars().collect();
        let ct = CharType::Symbol('㎡');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let ctx = RuleContext {
            word_chars: &word,
            index: 0,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: false,
            ascii_starts_at_beginning: false,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        assert!(should_insert_separator_after_symbol(&ctx));
    }

    /// rule_68:198 — digit-grade chain with `-` triggers the `'-' => ⠔` arm.
    /// Input MUST have a Korean char after the +/- chain to satisfy
    /// is_digit_grade_plus_notation (per the function source).
    #[test]
    fn rule68_digit_grade_with_minus_in_chain() {
        // 1+-가 — digit, then +, -, then Korean → satisfies notation predicate.
        let _ = crate::encode("1+-가");
        let _ = crate::encode("5-+나");
        let _ = crate::encode("3--다");
    }

    /// rule_68:108 — direct call to `encode_compact_ascii_notation` with a base
    /// letter followed by a single ⁺/⁻ then a non-super char. The inner loop
    /// breaks at line 108 when next char is neither ⁺ nor ⁻.
    #[test]
    fn rule68_superscript_block_breaks_on_non_super_direct() {
        // "A⁺x" — uppercase A + ⁺ (consumed) + x (breaks loop)
        let word: Vec<char> = "A\u{207A}x".chars().collect();
        let result = encode_compact_ascii_notation(&word, 0, false).unwrap();
        assert!(result.is_some());
        let (_, consumed) = result.unwrap();
        // Only A and ⁺ are consumed; x triggers the break at line 108.
        assert_eq!(consumed, 2);
    }

    /// rule_68:221 — apply()'s let-else returns Skip when char is NOT in MAPPINGS.
    /// Hand-build a `RuleContext` that bypasses `matches()` (matches() guards
    /// MAPPINGS containment via `is_rule_68_symbol`). Direct apply with English
    /// char and word starting with uppercase letter whose `encode_compact_ascii_notation`
    /// returns None for index > 0 → falls through to MAPPINGS find which returns None.
    #[test]
    fn rule68_apply_falls_through_to_mappings_skip() {
        // Construct: word "AB" at index=1 with English char-type 'B'.
        // encode_compact_ascii_notation at index=1: word.get(1)='B' base, but
        // cursor=2 is out-of-bounds → returns None (line 78-83 path in source).
        // Falls through to lines 187-215 (MathSymbol/Symbol('+') path) — doesn't match.
        // Reaches line 217: MAPPINGS find for 'B' → None → Skip at line 221.
        let word: Vec<char> = "AB".chars().collect();
        let ct = CharType::English('B');
        let mut skip = 0usize;
        let mut state = crate::rules::context::EncoderState::new(false);
        let mut out = Vec::new();
        let mut ctx = RuleContext {
            word_chars: &word,
            index: 1,
            char_type: &ct,
            prev_word: "",
            remaining_words: &[],
            has_korean_char: false,
            is_all_uppercase: true,
            ascii_starts_at_beginning: true,
            skip_count: &mut skip,
            state: &mut state,
            result: &mut out,
        };
        let outcome = Rule68.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    #[rstest::rstest]
    #[case::known_square_meter('㎡', true)]
    #[case::plain_ascii('x', false)]
    fn detects_rule_68_symbols(#[case] ch: char, #[case] expected: bool) {
        assert_eq!(is_rule_68_symbol(std::hint::black_box(ch)), expected);
    }
}
