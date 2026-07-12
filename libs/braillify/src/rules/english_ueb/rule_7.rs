//! §7 Punctuation.
//!
//! Per RUEB 2024 §7, punctuation attaches to adjacent text without a space.
//! This file handles the position-independent marks; the double-quotation mark
//! (§7.6), whose open/close form depends on context, is toggled by the document
//! engine which has the cross-token state.

use crate::unicode::decode_unicode;

/// Encode a single position-independent punctuation char to braille cells,
/// or `None` if it is not handled here (`"` is handled by the engine).
pub fn encode_punctuation(c: char) -> Option<Vec<u8>> {
    Some(match c {
        '.' => vec![decode_unicode('⠲')],
        ',' => vec![decode_unicode('⠂')],
        '?' => vec![decode_unicode('⠦')],
        '!' => vec![decode_unicode('⠖')],
        // §13.5 and §13.6/§14 Spanish inverted punctuation signs.
        '¿' => vec![decode_unicode('⠢')],
        '¡' => vec![decode_unicode('⠖')],
        ';' => vec![decode_unicode('⠆')],
        ':' => vec![decode_unicode('⠒')],
        '-' => vec![decode_unicode('⠤')],
        // §7 en/em dash → ⠠⠤.
        '\u{2013}' | '\u{2014}' => vec![decode_unicode('⠠'), decode_unicode('⠤')],
        '\'' => vec![decode_unicode('⠄')],
        // §7.6 curly double quotation marks — directional, so (unlike the straight
        // `"` toggled by the engine) each maps to a fixed open/close cell.
        '\u{201C}' => vec![decode_unicode('⠦')],
        '\u{201D}' => vec![decode_unicode('⠴')],
        // §7.6 curly opening single quote → ⠠⠦ (a left single quote is never an
        // apostrophe). The right single quote (U+2019) is context-dependent
        // (apostrophe vs closing single quote) and is handled by the engine.
        '\u{2018}' => vec![decode_unicode('⠠'), decode_unicode('⠦')],
        '(' => vec![decode_unicode('⠐'), decode_unicode('⠣')],
        ')' => vec![decode_unicode('⠐'), decode_unicode('⠜')],
        '[' => vec![decode_unicode('⠨'), decode_unicode('⠣')],
        ']' => vec![decode_unicode('⠨'), decode_unicode('⠜')],
        '{' => vec![decode_unicode('⠸'), decode_unicode('⠣')],
        '}' => vec![decode_unicode('⠸'), decode_unicode('⠜')],
        '/' => vec![decode_unicode('⠸'), decode_unicode('⠌')], // §7 slash
        '\\' => vec![decode_unicode('⠸'), decode_unicode('⠡')], // §7 backslash
        '@' => vec![decode_unicode('⠈'), decode_unicode('⠁')], // §7 at sign
        // §7.6 angled (guillemet) quotation marks — fixed open/close cells.
        '\u{00AB}' => vec![decode_unicode('⠸'), decode_unicode('⠦')], // «
        '\u{00BB}' => vec![decode_unicode('⠸'), decode_unicode('⠴')], // »
        // §7 ellipsis: the single-character form is three full stops.
        '\u{2026}' => vec![
            decode_unicode('⠲'),
            decode_unicode('⠲'),
            decode_unicode('⠲'),
        ], // …
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::period('.', vec![decode_unicode('⠲')])]
    #[case::comma(',', vec![decode_unicode('⠂')])]
    #[case::question('?', vec![decode_unicode('⠦')])]
    #[case::inverted_question('¿', vec![decode_unicode('⠢')])]
    #[case::exclamation('!', vec![decode_unicode('⠖')])]
    #[case::inverted_exclamation('¡', vec![decode_unicode('⠖')])]
    #[case::semicolon(';', vec![decode_unicode('⠆')])]
    #[case::colon(':', vec![decode_unicode('⠒')])]
    #[case::hyphen('-', vec![decode_unicode('⠤')])]
    #[case::open_paren('(', vec![decode_unicode('⠐'), decode_unicode('⠣')])]
    #[case::open_bracket('[', vec![decode_unicode('⠨'), decode_unicode('⠣')])]
    #[case::open_brace('{', vec![decode_unicode('⠸'), decode_unicode('⠣')])]
    #[case::slash('/', vec![decode_unicode('⠸'), decode_unicode('⠌')])]
    #[case::backslash('\\', vec![decode_unicode('⠸'), decode_unicode('⠡')])]
    #[case::at_sign('@', vec![decode_unicode('⠈'), decode_unicode('⠁')])]
    #[case::en_dash('\u{2013}', vec![decode_unicode('⠠'), decode_unicode('⠤')])]
    #[case::em_dash('\u{2014}', vec![decode_unicode('⠠'), decode_unicode('⠤')])]
    #[case::curly_open_dquote('\u{201C}', vec![decode_unicode('⠦')])]
    #[case::curly_close_dquote('\u{201D}', vec![decode_unicode('⠴')])]
    #[case::curly_open_squote('\u{2018}', vec![decode_unicode('⠠'), decode_unicode('⠦')])]
    #[case::guillemet_open('\u{00AB}', vec![decode_unicode('⠸'), decode_unicode('⠦')])]
    #[case::guillemet_close('\u{00BB}', vec![decode_unicode('⠸'), decode_unicode('⠴')])]
    #[case::ellipsis('\u{2026}', vec![decode_unicode('⠲'), decode_unicode('⠲'), decode_unicode('⠲')])]
    fn encodes_known_punctuation(#[case] c: char, #[case] expected: Vec<u8>) {
        assert_eq!(encode_punctuation(c), Some(expected));
    }

    #[test]
    fn double_quote_not_handled_here() {
        assert_eq!(encode_punctuation('"'), None);
    }
}
