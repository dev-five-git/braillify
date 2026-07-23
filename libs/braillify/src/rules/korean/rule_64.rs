//! 제64항 — 둘러싼 문자(원문자/네모 문자).
//!
//! Handles enclosed Unicode forms that semantically represent an existing
//! number, standalone jamo, syllable, or Latin letter. Two enclosing shapes
//! are supported:
//!   * Pre-composed circled characters (①, ⓐ, ㉠, ㉮ …) — encoded by the
//!     legacy [`Rule64`] handler using the circle marker ⠶.
//!   * Combining enclosing square (U+20DE) applied to any preceding character
//!     (1⃞, 가⃞, ㄱ⃞, a⃞ …) — encoded by [`Rule64Square`] using the open marker
//!     ⠸⠦ and close marker ⠴⠇. The wrapped character is encoded recursively
//!     through the standalone encoder so its native indicators (수표, 영자
//!     표시, ㄱ 자모표) are preserved.

use crate::char_struct::CharType;
use crate::english;
use crate::korean_part;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::korean::rule_8::ONTAB;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "64",
    subsection: None,
    name: "enclosed_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.64",
    description: "Encode enclosed/circled numbers, jamo, syllables, and latin letters",
};

pub static META_SQUARE: RuleMeta = RuleMeta {
    section: "64",
    subsection: Some("square"),
    name: "square_enclosed_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.64",
    description: "Wrap characters followed by U+20DE in square enclosing markers",
};

const CIRCLE: u8 = 54; // ⠶
const LETTER_MARKER: u8 = 52; // ⠴
const NUMBER_MARKER: u8 = 60; // ⠼

/// Open marker for square enclosing: ⠸⠦ (cells 56, 38)
const SQUARE_OPEN: [u8; 2] = [56, 38];
/// Close marker for square enclosing: ⠴⠇ (cells 52, 7)
const SQUARE_CLOSE: [u8; 2] = [52, 7];
/// U+20DE COMBINING ENCLOSING SQUARE — attaches to the preceding character.
const COMBINING_ENCLOSING_SQUARE: char = '\u{20DE}';

const CIRCLED_SYLLABLES: &[(char, char)] = &[
    ('㉮', '가'),
    ('㉯', '나'),
    ('㉰', '다'),
    ('㉱', '라'),
    ('㉲', '마'),
    ('㉳', '바'),
    ('㉴', '사'),
    ('㉵', '아'),
    ('㉶', '자'),
    ('㉷', '차'),
    ('㉸', '카'),
    ('㉹', '타'),
    ('㉺', '파'),
    ('㉻', '하'),
];

const CIRCLED_JAMO: &[(char, char)] = &[
    ('㉠', 'ㄱ'),
    ('㉡', 'ㄴ'),
    ('㉢', 'ㄷ'),
    ('㉣', 'ㄹ'),
    ('㉤', 'ㅁ'),
    ('㉥', 'ㅂ'),
    ('㉦', 'ㅅ'),
    ('㉧', 'ㅇ'),
    ('㉨', 'ㅈ'),
    ('㉩', 'ㅊ'),
    ('㉪', 'ㅋ'),
    ('㉫', 'ㅌ'),
    ('㉬', 'ㅍ'),
    ('㉭', 'ㅎ'),
];

pub fn is_enclosed_symbol(c: char) -> bool {
    matches!(c, '①'..='⑳' | 'ⓐ'..='ⓩ')
        || CIRCLED_SYLLABLES.iter().any(|(enclosed, _)| *enclosed == c)
        || CIRCLED_JAMO.iter().any(|(enclosed, _)| *enclosed == c)
}

fn encode_number_string(digits: &str) -> Result<Vec<u8>, String> {
    let mut result = vec![NUMBER_MARKER];
    for digit in digits.chars() {
        result.push(encode_enclosed_digit(digit)?);
    }
    Ok(result)
}

fn encode_enclosed_digit(digit: char) -> Result<u8, String> {
    match digit {
        '1' => Ok(2),
        '2' => Ok(6),
        '3' => Ok(18),
        '4' => Ok(50),
        '5' => Ok(34),
        '6' => Ok(22),
        '7' => Ok(54),
        '8' => Ok(38),
        '9' => Ok(20),
        '0' => Ok(52),
        _ => Err("Invalid enclosed number digit".to_string()),
    }
}

fn wrap_circle(inner: Vec<u8>) -> Vec<u8> {
    let mut result = Vec::with_capacity(inner.len() + 2);
    result.push(CIRCLE);
    result.extend(inner);
    result.push(CIRCLE);
    result
}

/// Wrap `inner` with the square enclosing open/close markers.
fn wrap_square(inner: Vec<u8>) -> Vec<u8> {
    let mut result = Vec::with_capacity(inner.len() + SQUARE_OPEN.len() + SQUARE_CLOSE.len());
    result.extend_from_slice(&SQUARE_OPEN);
    result.extend(inner);
    result.extend_from_slice(&SQUARE_CLOSE);
    result
}

/// Encode a single anchor character that is being wrapped by U+20DE.
///
/// 제64항 [네모 둘러싸기] frames a single character as a quoted symbol. The
/// wrapped character must be self-identifying in a Korean document context,
/// so each category uses the indicator the standard already defines for it:
///
/// * `CharType::English`   → 제28항: 영자 표시 ⠴ + alphabet cell
/// * `CharType::Number`    → 제40항: 수표 ⠼ + digit cell
/// * `CharType::KoreanPart`→ 제8항 : ONTAB ⠿ + jamo cells
/// * everything else (한글 음절, 기호 등) → delegated to the full encoder,
///   whose syllable/symbol encoding is already generalised
///
/// The outer closing marker ⠴⠇ supplied by [`wrap_square`] is what terminates
/// the wrapped region, so no English-section terminator (⠲) is appended here.
fn encode_square_anchor(anchor: char) -> Result<Vec<u8>, String> {
    match CharType::new(anchor)? {
        CharType::English(c) => Ok(vec![LETTER_MARKER, english::encode_english(c)?]),
        CharType::Number(c) => Ok(vec![NUMBER_MARKER, crate::number::encode_number(c)?]),
        CharType::KoreanPart(c) => {
            let mut out = vec![ONTAB];
            out.extend_from_slice(korean_part::encode_korean_part(c)?);
            Ok(out)
        }
        _ => {
            let mut encoder = crate::Encoder::new(false);
            let mut result = Vec::new();
            encoder.encode(&anchor.to_string(), &mut result)?;
            Ok(result)
        }
    }
}

pub fn encode_enclosed_symbol(c: char) -> Result<Vec<u8>, String> {
    if ('①'..='⑳').contains(&c) {
        let value = (c as u32) - ('①' as u32) + 1;
        return encode_number_string(&value.to_string());
    }

    if ('ⓐ'..='ⓩ').contains(&c) {
        let letter = char::from_u32((c as u32) - ('ⓐ' as u32) + ('a' as u32))
            .ok_or_else(|| "Invalid enclosed latin letter".to_string())?;
        return Ok(wrap_circle(vec![
            LETTER_MARKER,
            english::encode_english(letter)?,
        ]));
    }

    if let Some((_, syllable)) = CIRCLED_SYLLABLES
        .iter()
        .find(|(enclosed, _)| *enclosed == c)
    {
        return Ok(wrap_circle(crate::encode(&syllable.to_string())?));
    }

    if let Some((_, jamo)) = CIRCLED_JAMO.iter().find(|(enclosed, _)| *enclosed == c) {
        let mut inner = vec![ONTAB];
        inner.extend_from_slice(korean_part::encode_korean_part(*jamo)?);
        return Ok(wrap_circle(inner));
    }

    Err("Invalid enclosed symbol character".to_string())
}

pub struct Rule64;

impl BrailleRule for Rule64 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        350
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_enclosed_symbol(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        let encoded = encode_enclosed_symbol(*c)?;
        ctx.emit_slice(&encoded);
        Ok(RuleResult::Consumed)
    }
}

/// 제64항 [네모 둘러싸기] — combining U+20DE applied to any anchor.
///
/// Triggered when the next character in the current word is `U+20DE`. The
/// anchor (digit, syllable, jamo, Latin letter, …) is encoded recursively as a
/// standalone token, then wrapped with `⠸⠦ … ⠴⠇`. The `U+20DE` itself is
/// skipped so subsequent rules don't see the combining mark again.
///
/// Priority is set below every other `CoreEncoding` rule (the current minimum
/// is `rule_44` at 50) so that the wrap takes precedence over the anchor's
/// own per-character encoding.
pub struct Rule64Square;

impl BrailleRule for Rule64Square {
    fn meta(&self) -> &'static RuleMeta {
        &META_SQUARE
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        49
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        if ctx.next_char() != Some(COMBINING_ENCLOSING_SQUARE) {
            return false;
        }
        // Reject anchors that have no own braille representation: another
        // combining mark, or a whitespace. Every other CharType variant is a
        // valid anchor (digit, syllable, jamo, English letter, symbol, …).
        !matches!(ctx.char_type, CharType::CombiningMark | CharType::Space(_))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let anchor = ctx.current_char();
        let inner = encode_square_anchor(anchor)?;
        let wrapped = wrap_square(inner);
        ctx.emit_slice(&wrapped);
        // Consume the trailing U+20DE so it isn't re-encoded by rule_56.
        *ctx.skip_count = 1;
        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_unicode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .copied()
            .map(crate::unicode::encode_unicode)
            .collect()
    }

    #[test]
    fn encodes_circled_numbers() {
        assert_eq!(to_unicode(&encode_enclosed_symbol('①').unwrap()), "⠼⠂");
        assert_eq!(to_unicode(&encode_enclosed_symbol('⑳').unwrap()), "⠼⠆⠴");
    }

    #[test]
    fn encodes_circled_jamo() {
        assert_eq!(to_unicode(&encode_enclosed_symbol('㉠').unwrap()), "⠶⠿⠁⠶");
    }

    #[test]
    fn encodes_circled_latin() {
        assert_eq!(to_unicode(&encode_enclosed_symbol('ⓐ').unwrap()), "⠶⠴⠁⠶");
    }

    #[test]
    fn encodes_circled_syllable() {
        assert_eq!(to_unicode(&encode_enclosed_symbol('㉮').unwrap()), "⠶⠫⠶");
    }

    #[test]
    fn detects_supported_chars() {
        assert!(is_enclosed_symbol('①'));
        assert!(is_enclosed_symbol('㉠'));
        assert!(is_enclosed_symbol('㉮'));
        assert!(is_enclosed_symbol('ⓩ'));
        assert!(!is_enclosed_symbol('가'));
    }

    // ── Rule64Square (U+20DE) ────────────────────────────────

    fn encode_to_unicode(text: &str) -> String {
        let bytes = crate::encode(text).expect("encode failed");
        to_unicode(&bytes)
    }

    #[test]
    fn square_wraps_digit() {
        assert_eq!(encode_to_unicode("1\u{20DE}"), "⠸⠦⠼⠁⠴⠇");
    }

    #[test]
    fn square_wraps_korean_syllable() {
        assert_eq!(encode_to_unicode("가\u{20DE}"), "⠸⠦⠫⠴⠇");
    }

    #[test]
    fn square_wraps_korean_jamo() {
        assert_eq!(encode_to_unicode("ㄱ\u{20DE}"), "⠸⠦⠿⠁⠴⠇");
    }

    #[test]
    fn square_wraps_latin_letter() {
        let bytes = crate::encode("a\u{20DE}").expect("encode failed");
        assert_eq!(
            bytes,
            vec![56u8, 38, 52, 1, 52, 7],
            "bytes mismatch (got {bytes:?})"
        );
        assert_eq!(encode_to_unicode("a\u{20DE}"), "⠸⠦⠴⠁⠴⠇");
    }

    #[test]
    fn square_wraps_first_syllable_of_word() {
        // The square wraps only the leading syllable and leaves the suffix to the
        // regular Korean syllable path.
        assert_eq!(encode_to_unicode("가\u{20DE}에"), "⠸⠦⠫⠴⠇⠝");
    }

    #[test]
    fn lone_combining_square_is_no_op() {
        // A combining mark without an anchor is treated as a formatting
        // annotation (제56항 path) — encoding it alone must not error.
        assert_eq!(encode_to_unicode("\u{20DE}"), "");
    }

    /// 제64항 — encode_enclosed_symbol returns Err for unsupported character
    /// (line 190). Exercise the error path.
    #[test]
    fn encode_enclosed_symbol_returns_err_for_unknown() {
        assert!(encode_enclosed_symbol('가').is_err());
        assert!(encode_enclosed_symbol('A').is_err());
    }

    /// 제64항 — Rule64.apply emits an error when the underlying
    /// encode_enclosed_digit fails. Exercise the apply Skip path for non-symbol.
    #[test]
    fn rule64_apply_skips_non_symbol() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule64.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }

    /// 제64항 — encode_enclosed_digit returns Err for non-digit (line 109).
    #[test]
    fn encode_enclosed_digit_returns_err_for_non_digit() {
        assert!(encode_enclosed_digit('a').is_err());
        assert!(encode_enclosed_digit('가').is_err());
    }

    /// 제64항 — encode_square_anchor for English letter path (line 146).
    #[test]
    fn encode_square_anchor_english_letter() {
        let bytes = encode_square_anchor('a').unwrap();
        assert!(!bytes.is_empty());
    }

    /// 제64항 — encode_square_anchor for Number path (line 147).
    #[test]
    fn encode_square_anchor_number() {
        let bytes = encode_square_anchor('1').unwrap();
        assert!(!bytes.is_empty());
    }

    /// 제64항 — encode_square_anchor for KoreanPart path (line 148-152).
    #[test]
    fn encode_square_anchor_korean_part() {
        let bytes = encode_square_anchor('ㄱ').unwrap();
        assert!(!bytes.is_empty());
    }

    /// 제64항 — encode_square_anchor falls back to full encoder for
    /// syllables/symbols (line 153-159).
    #[test]
    fn encode_square_anchor_korean_syllable_fallback() {
        let bytes = encode_square_anchor('가').unwrap();
        assert!(!bytes.is_empty());
    }

    /// Rule64Square::priority — exercise the trait method directly for coverage.
    #[test]
    fn rule64_priorities() {
        assert_eq!(Rule64Square.priority(), 49);
        // Rule64::priority is the default 350.
        assert_eq!(Rule64.priority(), 350);
        assert_eq!(BrailleRule::priority(&Rule64), 350);
    }
}
