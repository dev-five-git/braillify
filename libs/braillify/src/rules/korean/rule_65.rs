//! 제65항 — 화폐 기호.
//!
//! Currency symbols are rendered as letter marker + currency indicator +
//! mnemonic Latin letter.

use crate::char_struct::CharType;
use crate::english;
use crate::rules::RuleMeta;
use crate::rules::context::RuleContext;
use crate::rules::traits::{BrailleRule, Phase, RuleResult};

pub static META: RuleMeta = RuleMeta {
    section: "65",
    subsection: None,
    name: "currency_symbols",
    standard_ref: "2024 Korean Braille Standard, Ch.6 Art.65",
    description: "Encode currency symbols with currency marker sequence",
};

const LETTER_MARKER: u8 = 52; // ⠴
const CURRENCY_MARKER: u8 = 8; // ⠈

pub fn currency_letter(c: char) -> Option<char> {
    match c {
        '￦' => Some('w'),
        '$' => Some('s'),
        '￠' => Some('c'),
        '€' => Some('e'),
        '￡' => Some('l'),
        '₣' => Some('f'),
        '￥' => Some('y'),
        _ => None,
    }
}

pub fn is_currency_symbol(c: char) -> bool {
    currency_letter(c).is_some()
}

pub fn encode_currency_symbol(c: char) -> Result<Vec<u8>, String> {
    let letter = currency_letter(c).ok_or_else(|| "Invalid currency symbol".to_string())?;
    Ok(vec![
        LETTER_MARKER,
        CURRENCY_MARKER,
        english::encode_english(letter)?,
    ])
}

pub struct Rule65;

impl BrailleRule for Rule65 {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn phase(&self) -> Phase {
        Phase::CoreEncoding
    }

    fn priority(&self) -> u16 {
        360
    }

    fn matches(&self, ctx: &RuleContext) -> bool {
        matches!(ctx.char_type, CharType::Symbol(c) if is_currency_symbol(*c))
    }

    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String> {
        let CharType::Symbol(c) = ctx.char_type else {
            return Ok(RuleResult::Skip);
        };

        let encoded = encode_currency_symbol(*c)?;
        ctx.emit_slice(&encoded);

        // 화폐 기호 뒤에 한글이 바로 이어지면 띄어 쓴다.
        if ctx.next_char().is_some_and(crate::utils::is_korean_char) {
            ctx.emit(0);
        }

        Ok(RuleResult::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::decode_unicode;

    fn to_unicode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .copied()
            .map(crate::unicode::encode_unicode)
            .collect()
    }

    #[test]
    fn encodes_currency_symbols() {
        assert_eq!(to_unicode(&encode_currency_symbol('￦').unwrap()), "⠴⠈⠺");
        assert_eq!(to_unicode(&encode_currency_symbol('$').unwrap()), "⠴⠈⠎");
        assert_eq!(to_unicode(&encode_currency_symbol('€').unwrap()), "⠴⠈⠑");
    }

    #[test]
    fn detects_supported_currency_symbols() {
        assert!(is_currency_symbol('￦'));
        assert!(is_currency_symbol('$'));
        assert!(is_currency_symbol('￥'));
        assert!(!is_currency_symbol('£'));
    }

    #[test]
    fn marker_bytes_are_expected() {
        assert_eq!(decode_unicode('⠴'), LETTER_MARKER);
        assert_eq!(decode_unicode('⠈'), CURRENCY_MARKER);
    }

    #[test]
    fn unsupported_currency_symbol_rejects_at_boundary() {
        assert_eq!(currency_letter('£'), None);
        assert!(!is_currency_symbol('£'));
        assert!(encode_currency_symbol('£').is_err());
    }

    #[test]
    fn apply_currency_symbol_emits_space_before_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("$가", false);
        let mut ctx = owned.ctx_at(0);

        let outcome = Rule65.apply(&mut ctx).unwrap();

        assert!(matches!(outcome, RuleResult::Consumed));
        assert_eq!(
            owned.result,
            vec![LETTER_MARKER, CURRENCY_MARKER, decode_unicode('⠎'), 0]
        );
    }

    #[test]
    fn apply_skips_non_korean() {
        let mut owned = crate::test_helpers::CtxOwned::for_text("A", false);
        let mut ctx = owned.ctx_at(0);
        let outcome = Rule65.apply(&mut ctx).unwrap();
        assert!(matches!(outcome, RuleResult::Skip));
    }
}
