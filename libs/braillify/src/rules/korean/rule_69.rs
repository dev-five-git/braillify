use crate::char_struct::CharType;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::korean::rule_29::{ENGLISH_CONTINUATION, ROMAN_INDICATOR};
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "69",
    subsection: None,
    name: "measurement_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.69",
    description: "Measurement and scientific unit symbols",
};

const SINGLE_MAPPINGS: &[(char, &str)] = &[
    ('㎎', "⠴⠍⠛"),
    ('㎗', "⠙⠇⠲"),
    ('㎠', "⠉⠍⠘⠼⠃"),
    ('㎞', "⠴⠅⠍⠲"),
    ('㎒', "⠴⠠⠍⠠⠓⠵⠲"),
    ('Ω', "⠴⠠⠨⠺⠲"),
    ('%', "⠴⠏"),
    ('‰', "⠴⠏⠍"),
    ('°', "⠴⠙"),
    ('℃', "⠴⠙⠠⠉"),
    ('℉', "⠴⠙⠠⠋"),
    ('′', "⠴⠤"),
    ('″', "⠴⠤⠤"),
    ('Å', "⠴⠡"),
];

const ASCII_UNIT_MAPPINGS: &[(&str, &str)] = &[
    ("cm", "⠴⠉⠍⠲"),
    ("kg", "⠴⠅⠛⠲"),
    ("in", "⠴⠊⠝⠲"),
    ("mm", "⠴⠍⠍⠲"),
    ("min", "⠍⠔⠲"),
    ("cal", "⠴⠉⠁⠇"),
    ("GB", "⠴⠠⠠⠛⠃⠲"),
    ("m", "⠴⠍⠲"),
    ("h", "⠴⠓⠲"),
];

const PERCENT_ABBREVIATION_MAPPINGS: &[(&str, &str)] = &[("%ile", "⠴⠏⠞"), ("%p", "⠴⠏⠏")];

const SEPARATED_SYMBOLS: &[char] = &['%', '‰', '°', '℃', '℉'];

fn encode_unicode_cells(unicode: &str) -> Vec<u8> {
    unicode
        .chars()
        .map(crate::unicode::decode_unicode)
        .collect()
}

pub fn is_rule_69_symbol(c: char) -> bool {
    SINGLE_MAPPINGS.iter().any(|(candidate, _)| *candidate == c) || c == 'μ'
}

fn is_numeric_or_unit_context(ctx: &RuleContext) -> bool {
    ctx.prev_char()
        .is_some_and(|prev| prev.is_ascii_digit() || matches!(prev, '/' | 'μ'))
        || ctx.prev_word.chars().next().is_some()
            && ctx
                .prev_word
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'))
        || ctx.prev_char() == Some('/')
}

/// 단어 자체가 단위 연쇄(cal/㎠/min 등)로 구성된 경우 첫 음절이 한국어 뒤에 와도
/// 단위로 해석한다. 단위 연쇄의 특징: 단어 내에 `/`가 있거나 제69항 단위 기호(㎠, ㎏ 등)가
/// 섞여 있다.
fn word_looks_like_unit_chain(word: &[char]) -> bool {
    let mut has_separator = false;
    let mut has_unit_symbol = false;
    for c in word {
        if *c == '/' {
            has_separator = true;
        } else if is_rule_69_symbol(*c) || *c == 'μ' {
            has_unit_symbol = true;
        }
    }
    let has_ascii_letter = word.iter().any(char::is_ascii_alphabetic);
    has_separator && (has_unit_symbol || has_ascii_letter)
}

fn is_symbol_measurement_context(ctx: &RuleContext, symbol: char) -> bool {
    match symbol {
        'μ' => {
            ctx.next_char().is_some_and(|ch| ch.is_ascii_alphabetic())
                || is_numeric_or_unit_context(ctx)
        }
        'Ω' => {
            ctx.next_char().is_some_and(crate::utils::is_korean_char)
                || is_numeric_or_unit_context(ctx)
        }
        _ => true,
    }
}

/// Check whether `tail` starts with the ASCII-only string `s` (char-by-char).
/// All entries in `ASCII_UNIT_MAPPINGS` are ASCII, so byte length and char count
/// coincide; we avoid materializing `tail` into a `String` on the hot path.
fn chars_start_with_ascii(tail: &[char], s: &str) -> bool {
    if tail.len() < s.len() {
        return false;
    }
    s.bytes().zip(tail.iter()).all(|(b, c)| (b as char) == *c)
}

pub(crate) fn encode_ascii_unit(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let tail = &word[index..];
    for (unit, unicode) in ASCII_UNIT_MAPPINGS {
        if !chars_start_with_ascii(tail, unit) {
            continue;
        }
        return Some((encode_unicode_cells(unicode), unit.len()));
    }
    None
}

fn encode_percent_abbreviation(word: &[char], index: usize) -> Option<(Vec<u8>, usize)> {
    let tail = &word[index..];
    for (abbr, unicode) in PERCENT_ABBREVIATION_MAPPINGS {
        if !chars_start_with_ascii(tail, abbr) {
            continue;
        }
        if *abbr == "%p"
            && tail
                .get(abbr.len())
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        {
            continue;
        }
        return Some((encode_unicode_cells(unicode), abbr.len()));
    }
    None
}

pub(crate) fn parse_numeric_ascii_unit_prefix(word: &[char]) -> Option<(String, Vec<u8>, usize)> {
    let numeric_len = word
        .iter()
        .take_while(|c| c.is_ascii_digit() || matches!(**c, ',' | '.'))
        .count();
    if numeric_len == 0 || numeric_len >= word.len() {
        return None;
    }

    let numeric = word[..numeric_len].iter().collect::<String>();
    let (unit, consumed) = encode_ascii_unit(word, numeric_len)?;
    Some((numeric, unit, numeric_len + consumed))
}

fn trim_recent_english_indicator(result: &mut Vec<u8>) {
    if result
        .last()
        .is_some_and(|cell| matches!(*cell, ENGLISH_CONTINUATION | ROMAN_INDICATOR))
    {
        result.pop();
    }
}

fn should_insert_separator_after_symbol(symbol: char, next: Option<char>) -> bool {
    SEPARATED_SYMBOLS.contains(&symbol) && next.is_some_and(crate::utils::is_korean_char)
}

pub struct Rule69;

impl BrailleRule for Rule69 {
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
        matches!(ctx.char_type, CharType::Symbol(c) if is_rule_69_symbol(*c) && is_symbol_measurement_context(ctx, *c))
            || matches!(ctx.char_type, CharType::Number(_)
                if ctx.index == 0 && parse_numeric_ascii_unit_prefix(ctx.word_chars).is_some())
            || matches!(ctx.char_type, CharType::English(_)
                if (is_numeric_or_unit_context(ctx)
                    || (ctx.index == 0 && word_looks_like_unit_chain(ctx.word_chars)))
                    && encode_ascii_unit(ctx.word_chars, ctx.index).is_some())
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        if matches!(ctx.char_type, CharType::Number(_))
            && ctx.index == 0
            && let Some((numeric, unit, consumed)) = parse_numeric_ascii_unit_prefix(ctx.word_chars)
        {
            let mut encoded = crate::encode(&numeric)?;
            encoded.extend(unit);
            ctx.emit_slice(&encoded);
            ctx.state.is_english = false;
            ctx.state.needs_english_continuation = false;
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        if matches!(ctx.char_type, CharType::English(_))
            && (is_numeric_or_unit_context(ctx)
                || (ctx.index == 0 && word_looks_like_unit_chain(ctx.word_chars)))
            && let Some((encoded, consumed)) = encode_ascii_unit(ctx.word_chars, ctx.index)
        {
            trim_recent_english_indicator(ctx.result);
            ctx.emit_slice(&encoded);
            ctx.state.is_english = false;
            ctx.state.needs_english_continuation = false;
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == '%'
            && let Some((encoded, consumed)) =
                encode_percent_abbreviation(ctx.word_chars, ctx.index)
        {
            ctx.emit_slice(&encoded);
            *ctx.skip_count = consumed.saturating_sub(1);
            if ctx
                .word_chars
                .get(ctx.index + consumed)
                .is_some_and(|ch| crate::utils::is_korean_char(*ch))
            {
                ctx.emit(0);
            }
            return Ok(RuleResult::Consumed);
        }

        if ctx.current_char() == 'μ' {
            trim_recent_english_indicator(ctx.result);
            let mut encoded = encode_unicode_cells("⠴⠨⠍");
            let mut consumed = 1usize;

            if let Some((unit_encoded, unit_len)) = encode_ascii_unit(ctx.word_chars, ctx.index + 1)
            {
                let mut unit_without_prefix = unit_encoded;
                if unit_without_prefix.first() == Some(&crate::unicode::decode_unicode('⠴')) {
                    unit_without_prefix.remove(0);
                }
                encoded.extend(unit_without_prefix);
                consumed += unit_len;
            } else {
                encoded.extend(encode_unicode_cells("⠍"));
            }

            ctx.emit_slice(&encoded);
            ctx.state.is_english = false;
            ctx.state.needs_english_continuation = false;
            *ctx.skip_count = consumed.saturating_sub(1);
            return Ok(RuleResult::Consumed);
        }

        // `matches()` guard `is_rule_69_symbol(c)` is a `SINGLE_MAPPINGS` lookup,
        // so reaching here without the prior μ/ASCII-unit/`%`-shortcut paths
        // means the char is guaranteed to be in `SINGLE_MAPPINGS`.
        let (_, unicode) = SINGLE_MAPPINGS
            .iter()
            .find(|(candidate, _)| *candidate == ctx.current_char())
            .expect("matches() guarantees the char is in SINGLE_MAPPINGS");
        let encoded = encode_unicode_cells(unicode);
        ctx.emit_slice(&encoded);
        if should_insert_separator_after_symbol(ctx.current_char(), ctx.next_char()) {
            ctx.emit(0);
        }
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Rule69, encode_ascii_unit, encode_percent_abbreviation, parse_numeric_ascii_unit_prefix,
        word_looks_like_unit_chain,
    };

    #[rstest::rstest]
    #[case::slash_with_ascii_unit("cal/min", true)]
    #[case::slash_with_unit_symbol("kg/㎠", true)]
    #[case::slash_without_unit_component("//", false)]
    #[case::unit_symbol_without_slash("㎠", false)]
    fn detects_unit_chain_words(#[case] input: &str, #[case] expected: bool) {
        let chars: Vec<char> = input.chars().collect();

        assert_eq!(word_looks_like_unit_chain(&chars), expected);
    }

    #[test]
    fn parses_compact_number_unit_word() {
        let chars: Vec<char> = "180cm".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse 180cm");
        assert_eq!(parsed.0, "180");
        assert_eq!(parsed.2, chars.len());
    }

    #[test]
    fn parses_decimal_number_unit_word() {
        let chars: Vec<char> = "1,234.5kg".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse decimal kg");

        assert_eq!(parsed.0, "1,234.5");
        assert_eq!(parsed.2, chars.len());
    }

    #[test]
    fn parses_leading_decimal_numeric_unit_word() {
        let chars: Vec<char> = ".5kg".chars().collect();
        let parsed = parse_numeric_ascii_unit_prefix(&chars).expect("should parse .5kg");

        assert_eq!(parsed.0, ".5");
        assert_eq!(parsed.2, chars.len());
    }

    /// 제69항 — percent-derived measurement abbreviations are data-driven, and
    /// `%p` only contracts at an abbreviation boundary.
    #[rstest::rstest]
    #[case::percentile("%ile", 4)]
    #[case::percentage_point("%p는", 2)]
    fn encodes_percent_abbreviation(#[case] input: &str, #[case] consumed: usize) {
        let chars: Vec<char> = input.chars().collect();
        let (encoded, actual_consumed) = encode_percent_abbreviation(&chars, 0).expect("abbr");
        assert!(!encoded.is_empty());
        assert_eq!(actual_consumed, consumed);
    }

    #[test]
    fn percent_p_does_not_match_inside_ascii_word() {
        let chars: Vec<char> = "%point".chars().collect();
        assert!(encode_percent_abbreviation(&chars, 0).is_none());
    }

    #[test]
    fn ascii_unit_scan_continues_past_non_matching_candidates() {
        let chars: Vec<char> = "zzz".chars().collect();

        assert!(encode_ascii_unit(&chars, 0).is_none());
    }

    #[test]
    fn rule69_metadata_is_stable() {
        use crate::rules::traits::BrailleRule;

        assert_eq!(Rule69.meta().name, "measurement_symbols");
        assert_eq!(Rule69.phase(), crate::rules::traits::Phase::CoreEncoding);
        assert_eq!(Rule69.priority(), 90);
    }

    /// rule_69:255 — `μ` (mu) alone or followed by non-unit chars triggers the
    /// else branch where `encode_unicode_cells("⠍")` is appended.
    #[test]
    fn rule69_mu_alone_without_unit() {
        // μ followed by Korean (no ASCII unit) → encode_ascii_unit returns None →
        // else branch at line 255 fires.
        let result = crate::encode_to_unicode("3μ가");
        assert!(result.is_ok());
        // μ at end with no following text.
        let result = crate::encode_to_unicode("3μ");
        assert!(result.is_ok());
    }
}
