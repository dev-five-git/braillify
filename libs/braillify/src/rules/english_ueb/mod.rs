//! Unified English Braille (UEB) Grade-2 encoder.
//!
//! Mirrors the Math token-engine architecture: a parser produces a token
//! stream ([`parser`]), a document engine ([`engine`]) manages modes and
//! capitalisation indicators, and per-rule [`contraction::ContractionRule`]
//! impls (one file per UEB §10.x clause) apply contractions via longest-match +
//! priority — the §10.10 preference rule, encoded structurally.
//!
//! [`try_encode`] returns `Some` only when the engine fully handles the input;
//! otherwise the caller falls back to the legacy path. This lets the engine
//! grow rule-by-rule without regressing already-passing cases.
//!
//! Source of truth: `docs/Rules-of-Unified-English-Braille-2024.pdf`.

pub mod compound;
pub mod contraction;
pub mod engine;
pub(crate) mod korean_context;
pub mod parser;
pub mod pronunciation;
pub mod rule_10_1;
pub mod rule_10_11;
pub mod rule_10_13;
pub mod rule_10_2;
pub mod rule_10_3;
pub mod rule_10_4;
pub mod rule_10_5;
pub mod rule_10_6;
pub mod rule_10_6_8;
pub mod rule_10_6_middle;
pub mod rule_10_6_restricted;
pub mod rule_10_7;
pub mod rule_10_7_pron;
pub mod rule_10_7_struct;
pub mod rule_10_8;
pub mod rule_10_9;
pub mod rule_10_9_list;
pub mod rule_11;
pub mod rule_12;
pub mod rule_13;
pub mod rule_14;
pub mod rule_15;
pub mod rule_16;
pub mod rule_3;
pub mod rule_3_24;
pub mod rule_4;
pub mod rule_5_7;
pub mod rule_6;
pub mod rule_7;
pub mod rule_9;
pub mod span;
pub mod standing_alone;
pub mod token;

use engine::EnglishUebEngine;

/// Attempt to encode `text` as standalone UEB Grade-2. Returns `None` if the
/// input is empty or contains a construct the engine does not yet support, so
/// the caller can fall back to the legacy encoding path.
pub fn try_encode(text: &str) -> Option<Vec<u8>> {
    // The math pipeline NFD-decomposes accented Latin (제65항 combining marks),
    // turning `é` into `e`+◌́. UEB owns accents as precomposed letters (§4.2,
    // [`rule_4`]), so recompose (NFC) here to undo that split for the English
    // path only. Pure ASCII is NFC-stable, so non-accented inputs are unchanged.
    use unicode_normalization::UnicodeNormalization;
    let composed: String = text.nfc().collect();
    if !is_ueb_eligible(&composed) {
        return None;
    }
    // Content-routing is normally not an explicit English declaration (`x` remains
    // bare), but the source syntaxes below are themselves English-document markup:
    // §14.6 inline Nemeth spans, §9 styled print words, quoted all-caps fragments,
    // and §6 telephone groups need the same document-level handling as forced English.
    encode_english(&composed, content_route_uses_document_english(&composed))
}

/// Encode `text` through the UEB engine WITHOUT the [`is_ueb_eligible`] content
/// heuristic. For an EXPLICIT `EncodingMode::English` the caller has already
/// committed to English, so a letterless fragment like `4:30` (`⠼⠙⠒⠼⠉⠚`) — which
/// the heuristic would defer to the Korean path — is encoded too; and an isolated
/// single wordsign letter takes the §2.6/§10.12.2 grade-1 indicator (`x`→⠰⠭).
pub fn encode_forced(text: &str) -> Option<Vec<u8>> {
    encode_english(text, true)
}

/// Run the UEB engine over `text` (no eligibility heuristic). `explicit_english`
/// is true only when the caller declared English mode via [`encode_forced`]; it
/// threads down to §5.7.1 so an isolated single letter is grade-1-indicated under
/// an explicit `context: english` but bare under default content-routing.
fn encode_english(text: &str, explicit_english: bool) -> Option<Vec<u8>> {
    use unicode_normalization::UnicodeNormalization;
    let composed: String = text.nfc().collect();
    if let Some(cells) = encode_struck_ligature_text(&composed) {
        return Some(cells);
    }
    if let Some(cells) = encode_single_caron_word(&composed, explicit_english) {
        return Some(cells);
    }
    // §14.3.3: a whole input that is exactly a printed language-name row emits its
    // non-UEB passage identifier (`Afrikaans` → ⠐⠷⠁⠋⠄). Whole-input match only, so
    // a sentence merely *containing* "French" still encodes as ordinary English.
    if let Some(cells) = rule_14::table_language_identifier(&composed) {
        return Some(cells);
    }
    // §14.3.1/14.3.2: non-UEB (Arabic/Greek/IPA/music) runs inside English prose
    // take the non-UEB word/passage indicators, with the surrounding English
    // encoded by the closure. Returns None when no code-switch span is present.
    if let Some(cells) = rule_14::encode_with_code_switches(&composed, |segment| {
        let tokens = parser::parse_english(segment);
        if tokens.is_empty() {
            Some(Vec::new())
        } else {
            EnglishUebEngine::new().encode(&tokens, explicit_english)
        }
    }) {
        return Some(cells);
    }
    let tokens = parser::parse_english(&composed);
    if tokens.is_empty() {
        return None;
    }
    EnglishUebEngine::new().encode(&tokens, explicit_english)
}

fn content_route_uses_document_english(text: &str) -> bool {
    has_inline_dollar_math_in_prose(text)
        || parenthesized_digit_group_before_number(&text.chars().collect::<Vec<_>>())
}

/// UEB 2024 §4.3.1/§4.3.4: adjacent letters marked by a stroke overlay are joined
/// by the ligature indicator; any modifier for either joined letter remains
/// immediately before the letter to which it applies.
fn encode_struck_ligature_text(text: &str) -> Option<Vec<u8>> {
    if !text.contains('\u{0336}') {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_ligature_letter(chars[i]) && chars.get(i + 1) == Some(&'\u{0336}') {
            let second = *chars.get(i + 2)?;
            if !is_ligature_letter(second) || chars.get(i + 3) != Some(&'\u{0336}') {
                return None;
            }
            push_rule4_letter(chars[i], &mut out)?;
            if second.is_uppercase() {
                out.push(crate::unicode::decode_unicode('⠠'));
            }
            out.extend([
                crate::unicode::decode_unicode('⠘'),
                crate::unicode::decode_unicode('⠖'),
            ]);
            push_rule4_letter_without_leading_cap(second, &mut out)?;
            i += 4;
            continue;
        }
        match chars[i] {
            ' ' => out.push(0),
            '?' => out.push(crate::unicode::decode_unicode('⠦')),
            c if is_ligature_letter(c) => push_rule4_letter(c, &mut out)?,
            _ => return None,
        }
        i += 1;
    }
    Some(out)
}

/// UEB 2024 §4.2.1/§4.2.4: a single modified word whose distinctive print
/// evidence is a caron uses the UEB modifier before the affected base letter, and
/// the modified letter is not part of a contraction.
fn encode_single_caron_word(text: &str, _explicit_english: bool) -> Option<Vec<u8>> {
    let has_caron = text
        .chars()
        .any(|c| matches!(c, 'č' | 'Č' | 'ě' | 'Ě' | 'ř' | 'Ř' | 'š' | 'Š' | 'ž' | 'Ž'));
    if !has_caron || !text.chars().all(is_ligature_letter) {
        return None;
    }
    let mut out = Vec::new();
    for c in text.chars() {
        push_rule4_letter(c, &mut out)?;
    }
    Some(out)
}

fn is_ligature_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || rule_4::is_modified_letter(c)
}

fn push_rule4_letter(c: char, out: &mut Vec<u8>) -> Option<()> {
    if let Some(cells) = rule_4::accent_cells(c) {
        out.extend(cells);
    } else {
        if c.is_uppercase() {
            out.push(crate::unicode::decode_unicode('⠠'));
        }
        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
    }
    Some(())
}

fn push_rule4_letter_without_leading_cap(c: char, out: &mut Vec<u8>) -> Option<()> {
    if let Some(cells) = rule_4::accent_cells(c) {
        let cells =
            if c.is_uppercase() && cells.first() == Some(&crate::unicode::decode_unicode('⠠')) {
                &cells[1..]
            } else {
                cells.as_slice()
            };
        out.extend(cells);
    } else {
        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
    }
    Some(())
}

/// Whether `text` carries content the UEB path may own. The legacy/math path
/// keeps every input with no such signal — in particular a lone accented letter
/// (`ã`, `ä`) is an ambiguous standalone diacritic owned by 수학 제65항, and a
/// pure number/symbol run (`7:30`, `9-10`) is legacy numeric.
///
/// Two signals qualify: (1) an ASCII letter — UEB §4.2 accents only arise inside
/// an alphabetic word (`café`), which always has one; and (2) a §9 typeform
/// signal — a Mathematical-Alphanumeric styled char or a combining underline
/// (U+0332) — so a *letterless* but emphasised input (`3̲4̲`, `27.̲9`, `83%̲`) is
/// still UEB's. Korean is excluded by the callers' own `is_korean_char` guard.
pub fn is_ueb_eligible(text: &str) -> bool {
    text.chars().any(|c| {
        c.is_ascii_alphabetic()
            || c == '\u{0332}'
            || rule_9::decode_styled(c).is_some()
            // §9: a small-cap styled letter is a §9 typeform signal even in a
            // letterless-looking run, so route it to the UEB typeform path.
            || rule_9::decode_small_cap(c).is_some()
            // §3.10 currency: the regular-width cent/pound/yen signs are UEB's. The
            // Korean 제65항 currency rule owns the *fullwidth* forms (`￠`/`￡`/`￥`)
            // and the shared `$`/`€`/`₣`, so only these three regular code points
            // are unambiguously English and safe to route here.
            || matches!(c, '\u{00A2}' | '\u{00A3}' | '\u{00A5}')
            // §3.8 copyright/registered/trademark and §3.26 transcriber-defined
            // signs (`©`/`®`/`™`/`✓`/`฿`/`❀`) are English-exclusive, so a
            // letterless example such as `©2009` routes through UEB. Korean 제65항
            // owns the shared per-mille `‰`, which is excluded.
            || matches!(
                c,
                '\u{00A9}' | '\u{00AE}' | '\u{2122}' | '\u{0E3F}' | '\u{2713}' | '\u{2740}'
            )
    })
    // §3.10 example uses shared euro/franc currency symbols in an exchange-rate
    // expression (`1 € = 6.55957₣`). A bare amount like `€75` remains legacy/Korean
    // owned, but an equation with digits and a spaced comparison sign is UEB prose.
    || (text.chars().any(|c| matches!(c, '\u{20AC}' | '\u{20A3}'))
        && text.chars().any(|c| c.is_ascii_digit())
        && text.contains(" = "))
    || {
        let chars: Vec<char> = text.chars().collect();
        // §16.2: a run of two or more adjacent box-drawing characters is a
        // horizontal line. A lone box char (a single mathematical `≡` or `─`)
        // is left to the legacy/math path so its non-line meaning is preserved.
        chars
            .windows(2)
            .any(|w| rule_16::is_line_char(w[0]) && rule_16::is_line_char(w[1]))
            // §3.15.1: a straight apostrophe/double-quote immediately after a
            // digit is a foot/inch sign in an English measurement (`4' 11"`);
            // this letterless shape routes to UEB, not the legacy quote path.
            || chars
                .windows(2)
                .any(|w| w[0].is_ascii_digit() && matches!(w[1], '\'' | '"'))
            // §6.6: a digit-space-digit run is a UEB numeric-space grouping
            // (`3 245 000` → ⠼⠉⠐⠃⠙⠑⠐⠚⠚⠚), not repeated Korean number signs.
            || chars
                .windows(3)
                .any(|w| w[0].is_ascii_digit() && w[1] == ' ' && w[2].is_ascii_digit())
            // §3.10: a `$`-space-digit currency amount inside a phrase
            // (`$2bn (2 billion dollars)`) is UEB currency prose.
            || chars
                .windows(3)
                .any(|w| w[0] == '$' && w[1] == ' ' && w[2].is_ascii_digit())
            // §6 numeric prose: a parenthesized area code followed by another digit
            // group is a telephone number, not a mathematical parenthetical.
            || parenthesized_digit_group_before_number(&chars)
    }
}

fn parenthesized_digit_group_before_number(chars: &[char]) -> bool {
    matches!(chars.first(), Some('('))
        && chars.iter().position(|c| *c == ')').is_some_and(|close| {
            close > 1
                && chars[1..close].iter().all(|c| c.is_ascii_digit())
                && matches!(chars.get(close + 1), Some(' '))
                && chars.get(close + 2).is_some_and(|c| c.is_ascii_digit())
        })
}

/// Whether `text` is *unambiguously* a math expression the legacy math pipeline
/// owns, so the UEB dispatch must NOT intercept it (Phase 7 preflight).
///
/// This is deliberately narrow: it keys only on hard math signals and never on
/// punctuation that also appears in English prose (`-`, `(`, `,`, `.`). The two
/// signals are (1) a known trig/log **function name** prefix (`sin`, `log2`,
/// `2cosx`), and (2) a single space-free token that **adjacently mixes ASCII
/// letters and digits** (`3ab`, `sin3x`, `f(x1)`) — a shape English words never
/// take. Hyphenated or multi-word English (`child-ish-ly`, `9-in dia.`) is left
/// to UEB.
pub fn is_math_owned(text: &str) -> bool {
    // (0) a balanced `$…$` span is LaTeX math, owned by the math engine. A lone
    // or trailing `$` (currency: `$6`, `US$`) is NOT, and is handled by §3.10.
    if text.len() >= 2 && text.starts_with('$') && text.ends_with('$') {
        return true;
    }
    // UEB §14.6 prose may contain embedded `$...$` Nemeth fragments. Once there is
    // ordinary prose outside the dollar span, tight `=`/`(`/`)` inside the fragment
    // is owned by the UEB code-switch encoder, not by default Korean/math routing.
    if has_inline_dollar_math_in_prose(text) {
        return false;
    }

    // (0a) §11 math/logic symbols — circled plus ⊕, the double-arrow implications
    // ⇒/↔ — are math by default wherever they appear (`a ⊕ b`, `p ⇒ q`). The UEB
    // §11 transcription of the bare glyph declares `context: english`, which routes
    // through `encode_forced` and bypasses this guard, so the english test cases
    // still reach §3; only content-routed (mode-less) expressions are kept on math.
    if text.chars().any(|c| {
        matches!(
            c,
            '\u{2295}'
                | '\u{21D2}'
                | '\u{2194}'
                | '→'
                | '←'
                | '↗'
                | '↘'
                | '↑'
                | '↓'
                | '△'
                | '□'
                | '′'
                | '″'
                | '|'
                | '‖'
                | '\u{0304}'
                | '\u{0302}'
        )
    }) {
        let chars: Vec<char> = text.chars().collect();
        let has_line_run = chars
            .windows(2)
            .any(|w| rule_16::is_line_char(w[0]) && rule_16::is_line_char(w[1]));
        let mut run = 0usize;
        let mut longest_lower_run = 0usize;
        for c in text.chars() {
            if c.is_ascii_lowercase() {
                run += 1;
                longest_lower_run = longest_lower_run.max(run);
            } else {
                run = 0;
            }
        }
        if !has_line_run && longest_lower_run < 3 {
            return true;
        }
    }

    // (0b) a comparison/equation operator (`=`, `<`, `>`) bound *tightly* to an
    // operand — no space on at least one side — is a math relation the legacy
    // math pipeline owns (`ax=b`, `a>b`, `x<0`, `y=f(x)`, `A={2, 4, …}`). English
    // prose always spaces these operators (`2 + 2 = 4`, `positron < posi`), so a
    // space on *both* sides leaves the input to the UEB §3.17 signs of comparison.
    let cells: Vec<char> = text.chars().collect();
    if cells.iter().enumerate().any(|(i, &c)| {
        matches!(c, '=' | '<' | '>')
            && !is_angle_bracket_prose(&cells, i)
            && (i.checked_sub(1).is_some_and(|j| cells[j] != ' ')
                || cells.get(i + 1).is_some_and(|&n| n != ' '))
    }) {
        return true;
    }

    // (0c) §3.24: a single space-free token carrying a Unicode super/subscript is a
    // math expression (`c²`, `x₂`, `³√x`, `log₂(x+1)`) — the same code points take
    // the 제18/19항 point shape there, not the UEB §3.24 indicator. Only a
    // multi-word English prose usage (`vitamin B₁₂`, `3 yd³`) reaches the UEB path.
    if !text.contains(' ')
        && text.chars().any(rule_3_24::is_script_char)
        && longest_ascii_letter_run(text) < 3
    {
        return true;
    }

    // (1) function-name expression: a trig/log name, optionally with a leading
    // coefficient (`2cosx`), where the name is NOT merely the prefix of a longer
    // English word (`singe` starts with `sin`, `arccosine` with `arccos`). The
    // character after the function name must be a non-letter (digit, `(`, end) —
    // a math argument, never a continuation of an English word.
    let after_coeff = text.trim_start_matches(|c: char| c.is_ascii_digit());
    if let Some((name, _)) = crate::rules::math::function::match_function_prefix(after_coeff) {
        let rest = &after_coeff[name.len()..];
        // The remainder is a math argument — never an English word continuation —
        // when it is empty, begins with a non-letter (`sin3x`, `f(`), begins with
        // an *uppercase* letter (a math variable, `arcsinA`, `cosX` — an English
        // word never has a mid-word capital after a function prefix), or is a run
        // of *vowel-free* letters (math variables `x`, `xy`). An English word
        // sharing the prefix (`singe`=sin+ge, `arccosine`=arccos+ine) always has a
        // lowercase, vowel-bearing continuation, so it is left to UEB.
        let rest_is_math_arg = rest.is_empty()
            || !rest.starts_with(|c: char| c.is_ascii_alphabetic())
            || rest.starts_with(|c: char| c.is_ascii_uppercase())
            || (!rest.contains(' ') && rest.chars().any(rule_3_24::is_script_char))
            || rest.chars().all(|c| {
                c.is_ascii_alphabetic()
                    && !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
            });
        if rest_is_math_arg {
            return true;
        }
    }

    // (1b) a digit enclosed in parentheses is bracketed math only when the bracket
    // attaches to adjacent math syntax: an alnum before `(` (`f(x-1)`, `7(2)`) or
    // a non-space after `)` (`(3n)!`). UEB §6 phone groups and §3.10 currency prose
    // use parenthesized digit groups without that math adjacency, and `$` inside the
    // brackets is currency rather than a math operand.
    if let Some(open) = text.find('(') {
        let inner = &text[open + 1..];
        let close = inner.find(')').unwrap_or(inner.len());
        let bracketed = &inner[..close];
        let before_math = text[..open]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        let after_math = inner
            .get(close + ')'.len_utf8()..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|c| !c.is_whitespace());
        if bracketed.chars().any(|c| c.is_ascii_digit())
            && !bracketed.contains(' ')
            && !bracketed.contains('$')
            && (before_math || after_math)
        {
            return true;
        }
    }

    // (2) a single space-free token where a digit is immediately followed by
    // *two or more* letters that are NOT an ordinal suffix — an implied product
    // of variables (`3ab`, `sin3x`). A single trailing letter (`3b`, `2d`,
    // `99c`) is a unit and an ordinal (`2nd`, `3rd`, `1st`, `4th`) is English;
    // both are encoded correctly by UEB, so they are NOT blocked.
    if text.contains("://") || text.contains('\\') {
        return true;
    }
    if text.contains(' ')
        || text.contains('-')
        || text.contains('@')
        || longest_ascii_letter_run(text) >= 4
    {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    let is_ordinal = ["st", "nd", "rd", "th"].iter().any(|suf| {
        lower.ends_with(suf) && lower[..lower.len() - 2].chars().all(|c| c.is_ascii_digit())
    });
    if is_ordinal {
        return false;
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1].is_ascii_alphabetic() && w[2].is_ascii_alphabetic())
}

fn longest_ascii_letter_run(text: &str) -> usize {
    let mut run = 0usize;
    let mut longest = 0usize;
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    longest
}

fn has_inline_dollar_math_in_prose(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.matches('$').count() == 2 {
        return false;
    }
    let mut in_span = false;
    let mut has_span = false;
    let mut outside_letters = 0usize;
    for c in text.chars() {
        if c == '$' {
            if in_span {
                has_span = true;
            }
            in_span = !in_span;
        } else if !in_span && c.is_ascii_alphabetic() {
            outside_letters += 1;
        }
    }
    has_span && outside_letters >= 3
}

/// UEB §3.17 signs of comparison are math-owned only when used as relations.
/// Email/listing delimiters and prose angle-bracket insertions (`<in file>`,
/// `<x, y>`, `Name <user@example.net>`) are paired punctuation, not comparison
/// operators, so tight `<`/`>` in those spans must not block UEB routing.
fn is_angle_bracket_prose(chars: &[char], index: usize) -> bool {
    match chars[index] {
        '<' => {
            let starts_after_boundary = index == 0 || chars[index - 1].is_whitespace();
            starts_after_boundary && matching_prose_angle_close(chars, index).is_some()
        }
        '>' => chars[..index]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| **c == '<')
            .is_some_and(|(open, _)| is_angle_bracket_prose(chars, open)),
        _ => false,
    }
}

fn matching_prose_angle_close(chars: &[char], open: usize) -> Option<usize> {
    let close = chars
        .iter()
        .enumerate()
        .skip(open + 1)
        .find_map(|(i, c)| if *c == '>' { Some(i) } else { None })?;
    let before_close = chars.get(close.checked_sub(1)?).copied()?;
    let after_close = chars.get(close + 1).copied();
    let closes_before_boundary =
        after_close.is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation());
    if before_close != '<' && closes_before_boundary {
        Some(close)
    } else {
        None
    }
}

#[cfg(test)]
mod is_math_owned_tests {
    use super::{
        encode_english, encode_struck_ligature_text, has_inline_dollar_math_in_prose, is_math_owned,
    };
    use crate::unicode::decode_unicode;

    /// Inputs the legacy math engine owns — UEB must NOT intercept these.
    #[rstest::rstest]
    #[case::sin("sin")]
    #[case::cos("cos")]
    #[case::sinh("sinh")]
    #[case::log2("log2")]
    #[case::two_log7("2log7")]
    #[case::sin3x("sin3x")]
    #[case::sinxy("sinxy")] // function + vowel-free variable run
    #[case::two_cosx("2cosx")] // leading coefficient
    #[case::three_ab("3ab")] // digit + 2 variables (implied product)
    #[case::f_paren("f(x-1)")] // digit inside parentheses
    #[case::factorial("(3n)!")]
    // §3.17: a comparison/equation operator bound tightly (no surrounding space)
    // is a math relation, not English prose.
    #[case::eq_relation("ax=b")]
    #[case::gt_relation("a>b")]
    #[case::lt_relation("x<0")]
    #[case::eq_func("y=f(x)")]
    #[case::set_eq("A={2, 4, 6, ...}")] // tight `=` even though spaces follow
    #[case::interval("-1<x<3")]
    #[case::vars_equal("VarsEqual=(x==y);")]
    // §3.24: a single space-free token with a Unicode super/subscript is math.
    #[case::script_c_squared("c\u{00B2}")]
    #[case::script_x_sub2("x\u{2082}")]
    #[case::script_chemical("H\u{2082}O")]
    #[case::script_unit("4m\u{00B2}")]
    #[case::script_cube_root("\u{00B3}\u{221A}x\u{00B3}")]
    #[case::script_log_sub2("log\u{2082}(x+1)")]
    // Shared math glyphs with no prose-length lowercase word are math-owned.
    #[case::spaced_right_arrow("p → q")]
    #[case::prime("x′")]
    #[case::absolute_value("|x|")]
    #[case::triangle_name("△ABC")]
    #[case::combining_hat("p\u{0302}")]
    fn math_owned_inputs_are_blocked(#[case] text: &str) {
        assert!(is_math_owned(text), "{text:?} should be math-owned");
    }

    /// English (or unit/ordinal) inputs UEB owns — must NOT be blocked.
    #[rstest::rstest]
    #[case::singe("singe")] // English word sharing the `sin` prefix
    #[case::singeing("singeing")]
    #[case::arccosine("arccosine")] // shares `arccos`, has vowel continuation
    #[case::ordinal_2nd("2nd")]
    #[case::ordinal_3rd("3rd")]
    #[case::unit_3b("3b")] // single trailing letter — unit, not product
    #[case::cents_99c("99c")]
    #[case::hyphenated("child-ish-ly")]
    #[case::sentence("That is quite fair.")]
    #[case::plain_word("cat")]
    // §3.17 prose: operators with a space on BOTH sides are English signs of
    // comparison, not a math relation (`2 + 2 = 4`, `positron < posi`).
    #[case::spaced_eq("a = b")]
    #[case::spaced_lt("positron < posi")]
    #[case::spaced_sum("as easy as 2 + 2 = 4")]
    // §3.10: a parenthetical *phrase* (spaces inside `(...)`) is prose, even with a
    // digit — not a bracketed math argument. Currency amounts gloss this way.
    #[case::paren_phrase_billion("$2bn (2 billion dollars)")]
    #[case::paren_phrase_escudos("20$00 (20 escudos)")]
    #[case::paren_phrase_pounds("\u{00A3}3m (3 million pounds)")]
    #[case::paren_phrase_enough("Buy meat (enough for 2).")]
    #[case::script_footnote("knowledge.\u{00B3}")]
    #[case::script_word_subscripts("mass\u{209B}\u{1D64}\u{2099}")]
    #[case::angle_phrase("<in file>")]
    #[case::angle_variables("<x, y>")]
    #[case::angle_email("<J.Child@children.net>")]
    #[case::angle_email_after_name("Jan Swan <swanj@iafrica.com>")]
    #[case::phone_area("phone: (61) 3 1234 5678")]
    #[case::currency_paren("Balance:  ($52.68)")]
    #[case::phone_number("(416) 486-2500")]
    #[case::shopping4you("shopping4you")]
    #[case::address_digit_word("4starhotel@webnet.com")]
    #[case::inline_nemeth_prose("The result will be in the form $(ax+by)(cx+dy)$, where $ac=12$.")]
    fn english_inputs_are_not_blocked(#[case] text: &str) {
        assert!(!is_math_owned(text), "{text:?} should NOT be math-owned");
    }

    #[test]
    fn struck_ligature_handles_uppercase_second_letter() {
        assert_eq!(
            encode_struck_ligature_text("a\u{0336}B\u{0336}"),
            Some(vec![
                decode_unicode('⠁'),
                decode_unicode('⠠'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠃'),
            ])
        );
    }

    #[test]
    fn struck_ligature_rejects_unmarked_second_letter() {
        assert_eq!(encode_struck_ligature_text("a\u{0336}B"), None);
    }

    #[test]
    fn struck_ligature_keeps_capital_for_accented_second_letter() {
        assert_eq!(
            encode_struck_ligature_text("a\u{0336}É\u{0336}"),
            Some(vec![
                decode_unicode('⠁'),
                decode_unicode('⠠'),
                decode_unicode('⠘'),
                decode_unicode('⠖'),
                decode_unicode('⠘'),
                decode_unicode('⠌'),
                decode_unicode('⠑'),
            ])
        );
    }

    #[test]
    fn code_switch_closure_encodes_non_empty_english_segment() {
        let encoded = encode_english("word قُ", true).expect("Arabic code switch should encode");

        assert!(encoded.contains(&decode_unicode('⠺')));
        assert!(encoded.iter().any(|cell| *cell != 0));
    }

    #[test]
    fn plain_english_route_uses_engine_after_parsing_tokens() {
        let input = std::hint::black_box("cat");

        assert_eq!(
            encode_english(input, true),
            Some(vec![
                decode_unicode('⠉'),
                decode_unicode('⠁'),
                decode_unicode('⠞')
            ])
        );
    }

    #[test]
    fn full_dollar_span_is_not_inline_prose_math() {
        assert!(!has_inline_dollar_math_in_prose("$x+1$"));
    }
}

#[cfg(test)]
mod is_ueb_eligible_tests {
    use super::is_ueb_eligible;

    /// §3.10: regular-width cent/pound/yen are English-exclusive (Korean 제65항
    /// owns the fullwidth forms), so a letterless currency run routes to UEB.
    #[rstest::rstest]
    #[case::cent_amount("10\u{00A2}")]
    #[case::pound_amount("\u{00A3}24")]
    #[case::yen_amount("\u{00A5}360")]
    #[case::euro_franc_equation("1 \u{20AC} = 6.55957\u{20A3}")]
    #[case::ascii_letter("cat")]
    #[case::styled_digit("3\u{0332}4")] // §9 combining-underline typeform
    #[case::phone_number("(416) 486-2500")]
    fn ueb_owned_inputs_are_eligible(#[case] text: &str) {
        assert!(is_ueb_eligible(text), "{text:?} should be UEB-eligible");
    }

    /// The shared `$`/`€`/`₣` and the *fullwidth* `￠`/`￡`/`￥`/`￦` are owned by
    /// Korean 제65항 (`⠴⠈ + letter`); a letterless run of them must stay in the
    /// legacy path, so it must NOT become UEB-eligible.
    #[rstest::rstest]
    #[case::dollar("$50")]
    #[case::euro("\u{20AC}75")]
    #[case::franc("\u{20A3}1")]
    #[case::fullwidth_cent("25\u{FFE0}")]
    #[case::fullwidth_pound("\u{FFE1}88")]
    #[case::fullwidth_yen("\u{FFE5}1")]
    #[case::fullwidth_won("\u{FFE6}100")]
    fn korean_owned_currency_is_not_eligible(#[case] text: &str) {
        assert!(
            !is_ueb_eligible(text),
            "{text:?} must stay in the legacy path"
        );
    }
}
