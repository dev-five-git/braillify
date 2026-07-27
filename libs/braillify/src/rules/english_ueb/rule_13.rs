//! UEB §13 Foreign Language helpers.
//!
//! §13.1 uses typography and print evidence to decide whether material is
//! foreign. The engine treats an italic/bold typeform run as foreign when the
//! run itself contains PDF-defined foreign evidence: non-English punctuation
//! (`¿`, `¡`, `«`, `»`), a §13.3 foreign accent, or an unrecorded word in the
//! same styled phrase. §13.2 then suppresses UEB contractions in that material.
//! §13.5 keeps UEB accent modifiers for occasional foreign material in English
//! context, while §13.6 uses foreign-code accent cells for instructional or
//! bilingual sentences carrying foreign-code signals.

use crate::english::encode_english;
use crate::unicode::decode_unicode;

use super::{rule_4, rule_12};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentCode {
    Ueb,
    Foreign,
}

pub const fn compose_combining(base: char, mark: char) -> Option<char> {
    match (base, mark) {
        ('a', '\u{0301}') => Some('á'),
        ('e', '\u{0301}') => Some('é'),
        ('i', '\u{0301}') => Some('í'),
        ('o', '\u{0301}') => Some('ó'),
        ('u', '\u{0301}') => Some('ú'),
        ('A', '\u{0301}') => Some('Á'),
        ('E', '\u{0301}') => Some('É'),
        ('I', '\u{0301}') => Some('Í'),
        ('O', '\u{0301}') => Some('Ó'),
        ('U', '\u{0301}') => Some('Ú'),
        ('a', '\u{0300}') => Some('à'),
        ('e', '\u{0300}') => Some('è'),
        ('i', '\u{0300}') => Some('ì'),
        ('o', '\u{0300}') => Some('ò'),
        ('u', '\u{0300}') => Some('ù'),
        ('A', '\u{0300}') => Some('À'),
        ('E', '\u{0300}') => Some('È'),
        ('I', '\u{0300}') => Some('Ì'),
        ('O', '\u{0300}') => Some('Ò'),
        ('U', '\u{0300}') => Some('Ù'),
        ('a', '\u{0302}') => Some('â'),
        ('e', '\u{0302}') => Some('ê'),
        ('i', '\u{0302}') => Some('î'),
        ('o', '\u{0302}') => Some('ô'),
        ('u', '\u{0302}') => Some('û'),
        ('A', '\u{0302}') => Some('Â'),
        ('E', '\u{0302}') => Some('Ê'),
        ('I', '\u{0302}') => Some('Î'),
        ('O', '\u{0302}') => Some('Ô'),
        ('U', '\u{0302}') => Some('Û'),
        ('a', '\u{0308}') => Some('ä'),
        ('e', '\u{0308}') => Some('ë'),
        ('u', '\u{0308}') => Some('ü'),
        ('n', '\u{0303}') => Some('ñ'),
        ('N', '\u{0303}') => Some('Ñ'),
        ('a', '\u{0303}') => Some('ã'),
        ('A', '\u{0303}') => Some('Ã'),
        ('o', '\u{0303}') => Some('õ'),
        ('O', '\u{0303}') => Some('Õ'),
        ('a', '\u{0304}') => Some('ā'),
        ('A', '\u{0304}') => Some('Ā'),
        ('e', '\u{0304}') => Some('ē'),
        ('E', '\u{0304}') => Some('Ē'),
        ('i', '\u{0304}') => Some('ī'),
        ('I', '\u{0304}') => Some('Ī'),
        ('o', '\u{0304}') => Some('ō'),
        ('O', '\u{0304}') => Some('Ō'),
        ('u', '\u{0304}') => Some('ū'),
        ('U', '\u{0304}') => Some('Ū'),
        ('y', '\u{0304}') => Some('ȳ'),
        ('Y', '\u{0304}') => Some('Ȳ'),
        ('a', '\u{0306}') => Some('ă'),
        ('A', '\u{0306}') => Some('Ă'),
        ('e', '\u{0306}') => Some('ĕ'),
        ('E', '\u{0306}') => Some('Ĕ'),
        ('i', '\u{0306}') => Some('ĭ'),
        ('I', '\u{0306}') => Some('Ĭ'),
        ('o', '\u{0306}') => Some('ŏ'),
        ('O', '\u{0306}') => Some('Ŏ'),
        ('u', '\u{0306}') => Some('ŭ'),
        ('U', '\u{0306}') => Some('Ŭ'),
        ('a', '\u{030A}') => Some('å'),
        ('A', '\u{030A}') => Some('Å'),
        ('c', '\u{030C}') => Some('č'),
        ('C', '\u{030C}') => Some('Č'),
        ('e', '\u{030C}') => Some('ě'),
        ('E', '\u{030C}') => Some('Ě'),
        ('r', '\u{030C}') => Some('ř'),
        ('R', '\u{030C}') => Some('Ř'),
        ('s', '\u{030C}') => Some('š'),
        ('S', '\u{030C}') => Some('Š'),
        ('z', '\u{030C}') => Some('ž'),
        ('Z', '\u{030C}') => Some('Ž'),
        ('c', '\u{0327}') => Some('ç'),
        ('C', '\u{0327}') => Some('Ç'),
        ('t', '\u{0326}') => Some('ț'),
        ('T', '\u{0326}') => Some('Ț'),
        ('s', '\u{0326}') => Some('ș'),
        ('S', '\u{0326}') => Some('Ș'),
        _ => None,
    }
}

pub const fn is_foreign_letter(c: char) -> bool {
    matches!(
        c,
        'Á' | 'á'
            | 'À'
            | 'à'
            | 'Ç'
            | 'ç'
            | 'È'
            | 'è'
            | 'É'
            | 'é'
            | 'Ê'
            | 'ê'
            | 'Ë'
            | 'ë'
            | 'Í'
            | 'í'
            | 'Î'
            | 'î'
            | 'Ñ'
            | 'ñ'
            | 'Ó'
            | 'ó'
            | 'Ô'
            | 'ô'
            | 'Ú'
            | 'ú'
            | 'Ọ'
            | 'ọ'
            | 'Ụ'
            | 'ụ'
            | 'Ä'
            | 'ä'
    )
}

pub fn foreign_accent_cells(c: char, spanish_context: bool) -> Option<Vec<u8>> {
    let cap = c.is_uppercase();
    let cell = match c.to_lowercase().next()? {
        'ç' => '⠯',
        'é' if spanish_context => '⠮',
        'é' => '⠿',
        'à' | 'á' => '⠷',
        'è' => '⠮',
        'ê' => '⠣',
        'ë' => '⠫',
        'î' => '⠩',
        'ô' => '⠹',
        'ú' => '⠾',
        'í' => '⠌',
        'ó' => '⠬',
        'ñ' => '⠻',
        'ọ' => '⠪',
        'ụ' => '⠳',
        'ä' => '⠜',
        _ => return None,
    };
    Some(if cap {
        vec![decode_unicode('⠠'), decode_unicode(cell)]
    } else {
        vec![decode_unicode(cell)]
    })
}

pub fn has_foreign_code_signal(chars: &[char]) -> bool {
    chars
        .iter()
        .any(|c| matches!(c, '¿' | '¡' | '«' | '»' | 'ñ' | 'Ñ' | 'ọ' | 'Ọ' | 'ụ' | 'Ụ'))
        || chars.iter().filter(|c| is_foreign_letter(**c)).count() >= 2
}

pub fn spanish_context(chars: &[char]) -> bool {
    chars.iter().any(|c| {
        matches!(
            c,
            '¿' | '¡' | 'ñ' | 'Ñ' | 'á' | 'Á' | 'í' | 'Í' | 'ó' | 'Ó' | 'ú' | 'Ú'
        )
    })
}

pub fn foreign_word_signal(chars: &[char]) -> bool {
    chars.iter().any(|c| is_foreign_letter(*c))
}

pub fn likely_foreign_passage(words: &[Vec<char>], doc_letters: &[char]) -> bool {
    if words.is_empty() {
        return false;
    }
    let has_any_foreign_letter = doc_letters.iter().any(|c| is_foreign_letter(*c));
    let has_foreign_punct = doc_letters
        .iter()
        .any(|c| matches!(c, '¿' | '¡' | '«' | '»'));
    if !has_foreign_code_signal(doc_letters) && !has_any_foreign_letter && !has_foreign_punct {
        return false;
    }
    if words.len() >= 80 && has_foreign_code_signal(doc_letters) {
        // §13.8.1 mixed-language literature: when English and another language are
        // freely interspersed with foreign-code accent evidence, the example uses
        // uncontracted braille throughout to avoid ambiguity.
        return true;
    }
    let foreign_marked = words.iter().filter(|w| foreign_word_signal(w)).count();
    if foreign_marked >= 2 && has_foreign_code_signal(doc_letters) && words.len() >= 3 {
        // §15.2.1 English poetry marks stress on vowels (`hót`, `bréath`,
        // `ánkles`) — the deaccented base is a real English word, so the
        // passage is NOT foreign even though several "foreign letters" appear.
        // Require the deaccented base be *unknown* to CMU for the passage to
        // count as foreign; otherwise the accented letters are UEB §4.2 stress
        // marks and get routed through `accent_cells`.
        //
        // The `words.len() >= 3` gate rules out 2-word all-accented §4.2 phrases
        // (`crème brûlée`, `Prométhée enchaîné`, `Voyage À Nice`) which have
        // enough foreign letters to look like a §13.6 sentence but no foreign
        // sentence structure. Those phrases stay on the UEB-accent path.
        let unrecorded_deaccented = words
            .iter()
            .filter(|w| foreign_word_signal(w))
            .filter(|w| {
                let base: String = w
                    .iter()
                    .map(|c| deaccent(*c))
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                base.len() > 1 && !super::pronunciation::cmudict::is_recorded_word(&base)
            })
            .count();
        if unrecorded_deaccented * 2 >= foreign_marked {
            return true;
        }
        // Do not decide the passage is English solely because accented words
        // deaccent to common English spellings.  UEB §13.6.4/§13.7.3 examples such
        // as French instructional/passages contain accented function words
        // (`dès`) whose base spelling collides with English-looking strings, but
        // the surrounding unaccented vocabulary is still dominantly foreign.  Fall
        // through to the whole-sentence non-CMU inventory check below.
    }
    let unrecorded = words
        .iter()
        .filter(|w| {
            let word: String = w.iter().flat_map(|c| c.to_lowercase()).collect();
            word.len() > 1 && !super::pronunciation::cmudict::is_recorded_word(&word)
        })
        .count();
    if has_foreign_code_signal(doc_letters) {
        // 3+ words gate keeps a 2-word all-accented §4.2 phrase (`crème brûlée`,
        // `Prométhée enchaîné`) on the UEB-accent path — those look like a §13.6
        // sentence by accent count alone but have no §13.6 sentence structure.
        return words.len() >= 3 && foreign_marked >= 1 && unrecorded * 2 >= words.len();
    }
    // §13.6 weak-signal path: a single foreign accent letter (é, è, ú, etc.) or
    // foreign punctuation (¿, ¡, «, ») combined with a sentence whose long-word
    // (>1 char) inventory is ≥ one-third non-CMU-recorded AND contains at least
    // two *non-accented* non-CMU words (French/Spanish function words such as
    // `combien`, `vas`, `dans`, `ans`) is a §13.6 foreign-code context.
    //
    // The non-accented gate is what tells a §13.6 pure-French sentence apart
    // from a §4.2 UEB-accent phrase:
    //   • `Il y a combien de temps que tu vas dans ce collège? (deux ans)` —
    //     4+ non-accented non-CMU words → §13.6.
    //   • `crème brûlée` — 0 non-accented non-CMU words → §4.2 UEB accents.
    //   • `Prométhée enchaîné` — 0 non-accented non-CMU words → §4.2.
    //   • `Voyage À Nice` — 0 non-accented non-CMU words → §4.2.
    // The one-third overall ratio keeps §13.5.1 English narratives with one
    // foreign word (`für`/`Sietske`) — 5/27 ≈ 18% — on the UEB-accent path.
    let unrecorded_non_accented = words
        .iter()
        .filter(|w| {
            let word: String = w.iter().flat_map(|c| c.to_lowercase()).collect();
            word.chars().count() > 1
                && !w.iter().any(|c| is_foreign_letter(*c))
                && !super::pronunciation::cmudict::is_recorded_word(&word)
        })
        .count();
    words.len() >= 5
        && (foreign_marked >= 1 || has_foreign_punct)
        && unrecorded * 3 >= words.len()
        && unrecorded_non_accented >= 2
}

/// Strip a single UEB §4.2 accent to its base ASCII letter — used only to
/// probe whether an accented word's deaccented form is a known English word.
fn deaccent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' | 'Å' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 'O',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ç' => 'c',
        'Ç' => 'C',
        _ => c,
    }
}

pub fn encode_uncontracted_word(
    chars: &[char],
    accent_code: AccentCode,
    spanish: bool,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for &c in chars {
        if c.is_uppercase() && !is_foreign_letter(c) {
            out.push(decode_unicode('⠠'));
        }
        let lower = c.to_lowercase().next()?;
        match accent_code {
            AccentCode::Foreign => {
                if let Some(cells) = foreign_accent_cells(c, spanish) {
                    out.extend(cells);
                } else if let Some(cells) = rule_12::early_letter(c) {
                    out.extend(cells);
                } else {
                    out.push(encode_english(lower).ok()?);
                }
            }
            AccentCode::Ueb => {
                if let Some(cells) = rule_4::accent_cells(c) {
                    out.extend(cells);
                } else if let Some(cells) = rule_12::early_letter(c) {
                    out.extend(cells);
                } else {
                    out.push(encode_english(lower).ok()?);
                }
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<u8> {
        s.chars().map(decode_unicode).collect()
    }

    #[rstest::rstest]
    #[case::spanish_acute_o('ó', true, "⠬")]
    #[case::spanish_ntilde('ñ', true, "⠻")]
    #[case::french_eacute('é', false, "⠿")]
    #[case::french_egrave('è', false, "⠮")]
    #[case::french_ccedilla('ç', false, "⠯")]
    #[case::spanish_eacute('é', true, "⠮")]
    #[case::grave_or_acute_a('á', false, "⠷")]
    #[case::ecircumflex('ê', false, "⠣")]
    #[case::ediaeresis('ë', false, "⠫")]
    #[case::icircumflex('î', false, "⠩")]
    #[case::ocircumflex('ô', false, "⠹")]
    #[case::uacute('ú', false, "⠾")]
    #[case::iacute('í', false, "⠌")]
    #[case::igbo_o_dot('Ọ', false, "⠠⠪")]
    #[case::igbo_u_dot('Ụ', false, "⠠⠳")]
    #[case::adiaeresis('ä', false, "⠜")]
    fn maps_foreign_code_accents_from_13_6_and_14_3(
        #[case] c: char,
        #[case] spanish: bool,
        #[case] expected: &str,
    ) {
        assert_eq!(foreign_accent_cells(c, spanish), Some(cells(expected)));
    }

    #[rstest::rstest]
    #[case::acute_lower_a('a', '\u{0301}', 'á')]
    #[case::acute_upper_a('A', '\u{0301}', 'Á')]
    #[case::acute_upper_i('I', '\u{0301}', 'Í')]
    #[case::acute_upper_o('O', '\u{0301}', 'Ó')]
    #[case::acute_upper_u('U', '\u{0301}', 'Ú')]
    #[case::grave_lower_a('a', '\u{0300}', 'à')]
    #[case::grave_lower_i('i', '\u{0300}', 'ì')]
    #[case::grave_lower_u('u', '\u{0300}', 'ù')]
    #[case::grave_upper_a('A', '\u{0300}', 'À')]
    #[case::grave_upper_e('E', '\u{0300}', 'È')]
    #[case::grave_lower_o('o', '\u{0300}', 'ò')]
    #[case::grave_upper_i('I', '\u{0300}', 'Ì')]
    #[case::grave_upper_o('O', '\u{0300}', 'Ò')]
    #[case::grave_upper_u('U', '\u{0300}', 'Ù')]
    #[case::circumflex_lower_a('a', '\u{0302}', 'â')]
    #[case::circumflex_lower_e('e', '\u{0302}', 'ê')]
    #[case::circumflex_lower_o('o', '\u{0302}', 'ô')]
    #[case::circumflex_lower_u('u', '\u{0302}', 'û')]
    #[case::circumflex_upper_a('A', '\u{0302}', 'Â')]
    #[case::circumflex_upper_e('E', '\u{0302}', 'Ê')]
    #[case::circumflex_upper_i('I', '\u{0302}', 'Î')]
    #[case::circumflex_upper_o('O', '\u{0302}', 'Ô')]
    #[case::circumflex_upper_u('U', '\u{0302}', 'Û')]
    #[case::diaeresis_lower_a('a', '\u{0308}', 'ä')]
    #[case::diaeresis_lower_e('e', '\u{0308}', 'ë')]
    #[case::diaeresis_lower_u('u', '\u{0308}', 'ü')]
    #[case::tilde_lower_n('n', '\u{0303}', 'ñ')]
    #[case::tilde_upper_n('N', '\u{0303}', 'Ñ')]
    #[case::tilde_lower_a('a', '\u{0303}', 'ã')]
    #[case::tilde_upper_a('A', '\u{0303}', 'Ã')]
    #[case::tilde_lower_o('o', '\u{0303}', 'õ')]
    #[case::tilde_upper_o('O', '\u{0303}', 'Õ')]
    #[case::macron_lower_a('a', '\u{0304}', 'ā')]
    #[case::macron_upper_a('A', '\u{0304}', 'Ā')]
    #[case::macron_lower_e('e', '\u{0304}', 'ē')]
    #[case::macron_upper_e('E', '\u{0304}', 'Ē')]
    #[case::macron_lower_i('i', '\u{0304}', 'ī')]
    #[case::macron_upper_i('I', '\u{0304}', 'Ī')]
    #[case::macron_lower_o('o', '\u{0304}', 'ō')]
    #[case::macron_upper_o('O', '\u{0304}', 'Ō')]
    #[case::macron_lower_u('u', '\u{0304}', 'ū')]
    #[case::macron_lower_y('y', '\u{0304}', 'ȳ')]
    #[case::macron_upper_u('U', '\u{0304}', 'Ū')]
    #[case::macron_upper_y('Y', '\u{0304}', 'Ȳ')]
    #[case::breve_lower_a('a', '\u{0306}', 'ă')]
    #[case::breve_upper_a('A', '\u{0306}', 'Ă')]
    #[case::breve_lower_e('e', '\u{0306}', 'ĕ')]
    #[case::breve_upper_e('E', '\u{0306}', 'Ĕ')]
    #[case::breve_lower_i('i', '\u{0306}', 'ĭ')]
    #[case::breve_upper_i('I', '\u{0306}', 'Ĭ')]
    #[case::breve_lower_o('o', '\u{0306}', 'ŏ')]
    #[case::breve_upper_o('O', '\u{0306}', 'Ŏ')]
    #[case::breve_lower_u('u', '\u{0306}', 'ŭ')]
    #[case::breve_upper_u('U', '\u{0306}', 'Ŭ')]
    #[case::ring_lower_a('a', '\u{030A}', 'å')]
    #[case::ring_upper_a('A', '\u{030A}', 'Å')]
    #[case::caron_lower_c('c', '\u{030C}', 'č')]
    #[case::caron_upper_c('C', '\u{030C}', 'Č')]
    #[case::caron_lower_e('e', '\u{030C}', 'ě')]
    #[case::caron_upper_e('E', '\u{030C}', 'Ě')]
    #[case::caron_lower_r('r', '\u{030C}', 'ř')]
    #[case::caron_upper_r('R', '\u{030C}', 'Ř')]
    #[case::caron_lower_s('s', '\u{030C}', 'š')]
    #[case::caron_upper_s('S', '\u{030C}', 'Š')]
    #[case::caron_lower_z('z', '\u{030C}', 'ž')]
    #[case::caron_upper_z('Z', '\u{030C}', 'Ž')]
    #[case::cedilla_lower_c('c', '\u{0327}', 'ç')]
    #[case::cedilla_upper_c('C', '\u{0327}', 'Ç')]
    #[case::comma_below_lower_t('t', '\u{0326}', 'ț')]
    #[case::comma_below_upper_t('T', '\u{0326}', 'Ț')]
    #[case::comma_below_lower_s('s', '\u{0326}', 'ș')]
    #[case::comma_below_upper_s('S', '\u{0326}', 'Ș')]
    fn composes_foreign_combining_letters(
        #[case] base: char,
        #[case] mark: char,
        #[case] expected: char,
    ) {
        assert_eq!(compose_combining(base, mark), Some(expected));
    }

    #[test]
    fn compose_combining_rejects_unknown_pair() {
        assert_eq!(compose_combining('x', '\u{0301}'), None);
    }

    #[rstest::rstest]
    #[case::lower_vowels('á', 'a')]
    #[case::upper_a('Å', 'A')]
    #[case::lower_e('ê', 'e')]
    #[case::upper_e('Ë', 'E')]
    #[case::lower_i('ï', 'i')]
    #[case::upper_i('Î', 'I')]
    #[case::lower_u('ü', 'u')]
    #[case::upper_u('Û', 'U')]
    #[case::upper_vowels('Ö', 'O')]
    #[case::lower_ntilde('ñ', 'n')]
    #[case::upper_ntilde('Ñ', 'N')]
    #[case::lower_cedilla('ç', 'c')]
    #[case::upper_cedilla('Ç', 'C')]
    #[case::plain_letter('q', 'q')]
    fn deaccents_foreign_letters_for_dictionary_probe(#[case] input: char, #[case] expected: char) {
        assert_eq!(deaccent(input), expected);
    }

    #[test]
    fn encodes_foreign_words_uncontracted_under_13_2() {
        assert_eq!(
            encode_uncontracted_word(
                &['s', 'h', 'a', 'm', 'i', 's', 'e', 'n'],
                AccentCode::Ueb,
                false
            ),
            Some(cells("⠎⠓⠁⠍⠊⠎⠑⠝"))
        );
    }

    #[test]
    fn foreign_code_words_can_include_early_english_letters() {
        assert_eq!(
            encode_uncontracted_word(&['þ'], AccentCode::Foreign, false),
            Some(cells("⠼⠮"))
        );
    }

    #[test]
    fn foreign_accent_cells_rejects_plain_letters() {
        assert_eq!(foreign_accent_cells('x', false), None);
    }

    #[test]
    fn foreign_accent_cells_allocates_for_plain_accent() {
        let accent = std::hint::black_box('ä');
        assert_eq!(foreign_accent_cells(accent, false), Some(cells("⠜")));
    }

    #[test]
    fn foreign_accent_cells_allocates_for_spanish_accent() {
        assert_eq!(foreign_accent_cells('ú', true), Some(cells("⠾")));
    }

    #[rstest::rstest]
    #[case::lowercase_enye('ñ', false, "⠻")]
    #[case::french_e_acute('é', false, "⠿")]
    #[case::spanish_e_acute('é', true, "⠮")]
    fn foreign_accent_cells_emits_lowercase_foreign_letter(
        #[case] input: char,
        #[case] spanish_context: bool,
        #[case] expected: &str,
    ) {
        assert_eq!(
            foreign_accent_cells(input, spanish_context),
            Some(cells(expected))
        );
    }

    #[test]
    fn foreign_accent_cells_capitalizes_uppercase_foreign_letter() {
        assert_eq!(foreign_accent_cells('Ñ', false), Some(cells("⠠⠻")));
        assert_eq!(foreign_accent_cells('É', true), Some(cells("⠠⠮")));
        assert_eq!(foreign_accent_cells('@', false), None);
    }

    #[test]
    fn foreign_accent_cells_handles_lowercase_mapping_after_cap_check() {
        assert_eq!(foreign_accent_cells('Ọ', false), Some(cells("⠠⠪")));
    }

    #[test]
    fn foreign_accent_runtime_allocates_and_uncontracted_extends() {
        let letter = std::hint::black_box('ä');
        assert_eq!(foreign_accent_cells(letter, false), Some(cells("⠜")));
        assert_eq!(
            encode_uncontracted_word(&[letter], AccentCode::Foreign, false),
            Some(cells("⠜"))
        );
    }

    #[rstest::rstest]
    #[case::foreign_ch_groupsign_suppressed(&['c', 'h', 'i', 'c'], "⠉⠓⠊⠉")]
    #[case::foreign_en_wordsign_suppressed(&['e', 'n'], "⠑⠝")]
    #[case::foreign_title_the_suppressed(&['T', 'h', 'e'], "⠠⠞⠓⠑")]
    fn encodes_foreign_typeform_words_uncontracted_under_13_2(
        #[case] input: &[char],
        #[case] expected: &str,
    ) {
        assert_eq!(
            encode_uncontracted_word(input, AccentCode::Ueb, false),
            Some(cells(expected))
        );
    }

    /// §12.2: under the UEB accent code an Old-English early letter (thorn `þ`)
    /// is emitted via its §12 early-letter sign rather than spelled as ASCII —
    /// exercising the `early_letter` fallback arm of `encode_uncontracted_word`.
    #[test]
    fn encode_uncontracted_word_emits_early_letter_under_ueb() {
        assert_eq!(
            encode_uncontracted_word(&['þ'], AccentCode::Ueb, false),
            Some(cells("⠼⠮"))
        );
    }

    /// §13.6 weak-signal path: pure-French `Il y a combien de temps que tu vas
    /// dans ce collège? (deux ans)` has only one foreign letter (`è`) so
    /// `has_foreign_code_signal` is false, but the sentence is dominantly
    /// non-CMU French — so `likely_foreign_passage` must still be true so the
    /// whole sentence is encoded uncontracted with foreign-code accents.
    #[test]
    fn detects_pure_french_sentence_as_foreign_passage_from_13_6() {
        let sentence = "Il y a combien de temps que tu vas dans ce collège? (deux ans)";
        let words: Vec<Vec<char>> = sentence
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .map(|w| w.chars().collect())
            .collect();
        let doc_letters: Vec<char> = sentence.chars().collect();
        assert!(likely_foreign_passage(&words, &doc_letters));
    }

    /// §13.5.1 narrative counter-case: `Sietske took … "Ein Geschenk für uns" …
    /// he laughed. Then he opened the tin box.` has one foreign letter (`ü`)
    /// but a dominantly-English sentence (~18% non-CMU), so it must STAY on
    /// the UEB-accent path (§13.5) — the pure-French L15 relaxation must not
    /// leak into this case.
    #[test]
    fn keeps_english_narrative_off_foreign_passage_from_13_5() {
        let sentence = concat!(
            "Sietske took out the parcel and handed it to the soldier. ",
            "Ein Geschenk für uns [A gift for us], he laughed. ",
            "Then he opened the tin box."
        );
        let words: Vec<Vec<char>> = sentence
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .map(|w| w.chars().collect())
            .collect();
        let doc_letters: Vec<char> = sentence.chars().collect();
        assert!(!likely_foreign_passage(&words, &doc_letters));
    }
}
