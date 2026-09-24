//! 과학 제31항 — 수 뒤에 단위가 붙은 물리량 식(`6.670×10⁻⁸cm³g⁻¹sec⁻²`).
//!
//! 식은 수학 점자로 적고, 수 뒤의 단위는 한글 제69항대로 로마자표 ⠴ 를 앞세운다.
//! 식 안의 단위에는 로마자 종료표를 적지 않는다. 단위 사이의 곱 `∙` 는 ⠐, 나눗셈
//! `/` 는 ⠸⠌ 로 적는다.

use crate::english::encode_english;
use crate::rules::korean::rule_69::SI_PREFIXES;
use crate::unicode::decode_unicode;
use phf::{Set, phf_set};

/// 접두어 없이 쓰는 단위 기호. `cm` 는 접두어 `c` 와 `m` 로 읽는다.
static BASE_UNITS: Set<&'static str> = phf_set! {
    "m", "g", "s", "sec", "min", "h", "A", "K", "mol", "cd", "N", "J", "W", "Pa", "Hz", "V",
    "C", "dyn", "erg", "cal", "L", "l",
};

fn is_unit(spelling: &str) -> bool {
    BASE_UNITS.contains(spelling)
        || SI_PREFIXES.iter().any(|prefix| {
            spelling
                .strip_prefix(prefix)
                .is_some_and(|base| BASE_UNITS.contains(base))
        })
}

/// `at` 에서 시작하는 가장 긴 단위 기호의 길이.
fn unit_len(chars: &[char], at: usize) -> Option<usize> {
    let run = chars[at..]
        .iter()
        .take_while(|c| c.is_ascii_alphabetic())
        .count();
    (1..=run)
        .rev()
        .find(|len| is_unit(&chars[at..at + len].iter().collect::<String>()))
}

fn superscript(ch: char) -> Option<char> {
    match ch {
        '⁰' => Some('0'),
        '¹' => Some('1'),
        '²' => Some('2'),
        '³' => Some('3'),
        '⁴'..='⁹' => char::from_digit(ch as u32 - '⁴' as u32 + 4, 10),
        '⁻' => Some('-'),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
enum Piece {
    Unit(String),
    Exponent(String),
    Times,
    Per,
}

/// `at` 에서 시작하는 단위 묶음(`cm³g⁻¹sec⁻²`, `dyn∙cm²/g²`)과 그 끝.
fn unit_run(chars: &[char], mut at: usize) -> Option<(Vec<Piece>, usize)> {
    let mut pieces = Vec::new();
    while let Some(&ch) = chars.get(at) {
        if let Some(len) = unit_len(chars, at) {
            pieces.push(Piece::Unit(chars[at..at + len].iter().collect()));
            at += len;
        } else if superscript(ch).is_some() {
            let exponent: String = chars[at..].iter().map_while(|c| superscript(*c)).collect();
            at += exponent.chars().count();
            pieces.push(Piece::Exponent(exponent));
        } else if matches!(ch, '∙' | '·') {
            pieces.push(Piece::Times);
            at += 1;
        } else if ch == '/' {
            pieces.push(Piece::Per);
            at += 1;
        } else {
            break;
        }
    }
    let opens = matches!(pieces.first(), Some(Piece::Unit(_)));
    let closes = !matches!(pieces.last(), Some(Piece::Times | Piece::Per));
    (opens && closes).then_some((pieces, at))
}

/// 단위 묶음을 적는다. 숫자 첨자 뒤의 a~j 앞에는 수학 제12항 [다만]의 ⠐ 을,
/// 음의 지수 뒤에 오는 단위 앞에는 로마자표를 다시 적는다(과학 제31항의 예).
fn encode_run(pieces: &[Piece]) -> Result<Vec<u8>, String> {
    let mut out = vec![decode_unicode('⠴')];
    let mut after_exponent: Option<bool> = None;
    for piece in pieces {
        match piece {
            Piece::Unit(unit) => {
                match after_exponent {
                    Some(true) => out.push(decode_unicode('⠴')),
                    Some(false) if unit.starts_with(|c: char| ('a'..='j').contains(&c)) => {
                        out.push(decode_unicode('⠐'));
                    }
                    _ => {}
                }
                for letter in unit.chars() {
                    if letter.is_ascii_uppercase() {
                        out.push(decode_unicode('⠠'));
                    }
                    out.push(encode_english(letter.to_ascii_lowercase())?);
                }
                after_exponent = None;
                continue;
            }
            Piece::Exponent(exponent) => {
                out.push(decode_unicode('⠘'));
                let mut number = false;
                for ch in exponent.chars() {
                    match ch {
                        '-' => out.push(decode_unicode('⠔')),
                        digit => {
                            if !number {
                                out.push(decode_unicode('⠼'));
                                number = true;
                            }
                            out.push(crate::number::encode_number(digit)?);
                        }
                    }
                }
                after_exponent = Some(exponent.contains('-'));
                continue;
            }
            Piece::Times => out.push(decode_unicode('⠐')),
            Piece::Per => out.extend([decode_unicode('⠸'), decode_unicode('⠌')]),
        }
        after_exponent = None;
    }
    Ok(out)
}

fn ends_number(ch: char) -> bool {
    ch.is_ascii_digit() || superscript(ch).is_some_and(|c| c.is_ascii_digit())
}

fn is_operator(ch: char) -> bool {
    matches!(
        ch,
        '=' | '≒' | '≈' | '×' | '÷' | '±' | '+' | '−' | '<' | '>' | '≤' | '≥'
    )
}

/// 수 뒤의 글자가 단위라는 증거. 한 글자는 변수와 같으므로(`3m+2n`) 두 글자 이상의
/// 단위이거나 지수·`∙`·`/` 로 이은 묶음이어야 한다.
fn is_evident_unit(pieces: &[Piece]) -> bool {
    pieces.len() >= 2 || matches!(pieces, [Piece::Unit(unit)] if unit.chars().count() >= 2)
}

/// 수 뒤에 단위가 붙은 식을 적는다. 한글이나 빈칸이 없는 묵자 식이어야 하고, 식에
/// 연산 기호나 등호가 있어야 한다 — 수와 단위만 적은 것(`180cm`)은 한글 제69항의
/// 몫이다. 단위를 뺀 나머지는 `encode_math` 로 적는다.
pub(crate) fn encode(
    text: &str,
    encode_math: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Option<Vec<u8>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.iter().any(|c| {
        c.is_whitespace()
            || crate::utils::is_korean_char(*c)
            || matches!(c, '$' | '\\' | '{' | '}' | '^' | '_')
    }) {
        return None;
    }
    let mut runs = Vec::new();
    let mut at = 1;
    while at < chars.len() {
        let run = ends_number(chars[at - 1])
            .then(|| unit_run(&chars, at))
            .flatten()
            .filter(|(pieces, end)| {
                is_evident_unit(pieces) && chars.get(*end).is_none_or(|c| is_operator(*c))
            });
        match run {
            Some((pieces, end)) => {
                runs.push((at, end, pieces));
                at = end + 1;
            }
            None => at += 1,
        }
    }
    let mut math = Vec::new();
    let mut from = 0;
    for (start, end, _) in &runs {
        math.push(from..*start);
        from = *end;
    }
    math.push(from..chars.len());
    let has_operator = math
        .iter()
        .any(|range| chars[range.clone()].iter().any(|c| is_operator(*c)));
    if runs.is_empty() || !has_operator {
        return None;
    }
    let mut out = Vec::new();
    for (at, range) in math.iter().enumerate() {
        let segment: String = chars[range.clone()].iter().collect();
        out.extend(encode_math(&segment).ok()?);
        if let Some((_, _, pieces)) = runs.get(at) {
            out.extend(encode_run(pieces).ok()?);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(cells: &[u8]) -> String {
        cells
            .iter()
            .map(|cell| crate::unicode::encode_unicode(*cell))
            .collect()
    }

    fn math(segment: &str) -> Result<Vec<u8>, String> {
        crate::encode(segment)
    }

    #[rstest::rstest]
    #[case::gravitational_constant(
        "G=(6.670±0.005)×10⁻⁸cm³g⁻¹sec⁻²≒6.670×10⁻⁸dyn∙cm²/g²",
        "⠠⠛⠒⠒⠦⠼⠋⠲⠋⠛⠚⠢⠔⠼⠚⠲⠚⠚⠑⠴⠡⠼⠁⠚⠘⠔⠼⠓⠴⠉⠍⠘⠼⠉⠐⠛⠘⠔⠼⠁⠴⠎⠑⠉⠘⠔⠼⠃⠐⠒⠒⠼⠋⠲⠋⠛⠚⠡⠼⠁⠚⠘⠔⠼⠓⠴⠙⠽⠝⠐⠉⠍⠘⠼⠃⠸⠌⠛⠘⠼⠃"
    )]
    #[case::speed_of_light("c=3×10⁸m/s", "⠉⠒⠒⠼⠉⠡⠼⠁⠚⠘⠼⠓⠴⠍⠸⠌⠎")]
    #[case::capital_unit("p=5Pa", "⠏⠒⠒⠼⠑⠴⠠⠏⠁")]
    #[case::positive_exponent_before_a_letter_past_j("a=2m²s⁻¹", "⠁⠒⠒⠼⠃⠴⠍⠘⠼⠃⠎⠘⠔⠼⠁")]
    fn writes_units_after_numbers(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text, math).map(|cells| braille(&cells)).as_deref(),
            Some(expected)
        );
    }

    #[rstest::rstest]
    #[case::number_and_unit_only("180cm")]
    #[case::korean_sentence("속도는 3×10⁸m/s이다")]
    #[case::no_unit("x=3×10⁸")]
    #[case::variable_after_number("y=3x")]
    #[case::run_before_a_letter("x=3mz")]
    #[case::dangling_operator("x=3m/")]
    #[case::single_letter_is_a_variable("x=3m+2n")]
    #[case::latex("$a^{3m+2n}$")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text, math), None);
    }

    #[rstest::rstest]
    #[case::centimetre("cm", true)]
    #[case::kilogram("kg", true)]
    #[case::second("sec", true)]
    #[case::dyne("dyn", true)]
    #[case::not_a_unit("xyz", false)]
    fn knows_units(#[case] spelling: &str, #[case] expected: bool) {
        assert_eq!(is_unit(spelling), expected);
    }

    #[test]
    fn stops_where_the_run_cannot_go_on() {
        let chars: Vec<char> = "m∙".chars().collect();
        assert_eq!(unit_run(&chars, 0), None);
        let chars: Vec<char> = "cm²x".chars().collect();
        assert_eq!(
            unit_run(&chars, 0),
            Some((
                vec![Piece::Unit("cm".into()), Piece::Exponent("2".into())],
                3
            ))
        );
    }
}
