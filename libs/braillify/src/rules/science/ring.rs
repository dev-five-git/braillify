//! 과학 제12·14항 — 고리 화합물과 축합 다환식 화합물의 기호 표기 형식.
//!
//! 입력은 환핵 도형을 묵자 배치대로 여러 줄에 놓은 글이다. 세로 방향 육각 환핵은
//! U+2B21 `⬡`, 가로 방향은 U+2394 `⎔`, 오각 환핵은 U+2B20 `⬠` 로 적는다. 같은 줄에서
//! 세로 결합선을 나눈 이웃은 두 칸, 사선 결합선을 나눈 이웃은 한 칸·한 줄 떨어진다.

use crate::unicode::decode_unicode;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ring {
    Vertical,
    Horizontal,
    Pentagon,
}

fn ring(glyph: char) -> Option<Ring> {
    match glyph {
        '\u{2B21}' => Some(Ring::Vertical),
        '\u{2394}' => Some(Ring::Horizontal),
        '\u{2B20}' => Some(Ring::Pentagon),
        _ => None,
    }
}

/// 제12항 1·2.
fn ring_cells(ring: Ring) -> &'static str {
    match ring {
        Ring::Vertical => "⠯⠪⠕⠽",
        Ring::Horizontal => "⠯⠕⠪⠽",
        Ring::Pentagon => "⠯⠪⠅⠽",
    }
}

/// 제14항 2 — 뒤 핵이 앞 핵의 위쪽 사선 결합선에 붙으면 ⠬, 아래쪽이면 ⠩. 세로
/// 결합선으로 붙은 핵에는 적지 않는다. 붙지 않았으면 `None`.
fn fused(ring: Ring, from: (usize, usize), to: (usize, usize)) -> Option<Option<char>> {
    match (from.0.abs_diff(to.0), from.1.abs_diff(to.1)) {
        (1, 1) if ring != Ring::Pentagon => Some(Some(if to.1 < from.1 { '⠬' } else { '⠩' })),
        (2, 0) if ring == Ring::Vertical => Some(None),
        _ => None,
    }
}

/// 제14항 1·3 — 왼쪽에서 오른쪽으로, 위쪽에서 아래쪽으로 풀어 적는다. 바로 앞 핵이
/// 아니라 첫 번째 핵과 결합한 핵은 ⠤ 을 적은 뒤 적는다.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    let mut rings = Vec::new();
    for (row, line) in text.lines().enumerate() {
        for (column, glyph) in line.chars().enumerate() {
            match ring(glyph) {
                Some(kind) => rings.push((column, row, kind)),
                None if glyph == ' ' => {}
                None => return None,
            }
        }
    }
    let kind = rings.first()?.2;
    if rings.iter().any(|(_, _, other)| *other != kind) {
        return None;
    }
    rings.sort_by_key(|(column, row, _)| (*column, *row));
    let at = |index: usize| (rings[index].0, rings[index].1);
    let mut out = String::from(ring_cells(kind));
    for index in 1..rings.len() {
        let mark = match fused(kind, at(index - 1), at(index)) {
            Some(mark) => mark,
            None => {
                out.push('⠤');
                fused(kind, at(0), at(index))?
            }
        };
        out.extend(mark);
        out.push_str(ring_cells(kind));
    }
    Some(out.chars().map(decode_unicode).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(text: &str) -> Option<String> {
        encode(text).map(|cells| {
            cells
                .into_iter()
                .map(crate::unicode::encode_unicode)
                .collect()
        })
    }

    #[rstest::rstest]
    #[case::vertical("\u{2B21}", "⠯⠪⠕⠽")]
    #[case::horizontal("\u{2394}", "⠯⠕⠪⠽")]
    #[case::pentagon("\u{2B20}", "⠯⠪⠅⠽")]
    #[case::naphthalene("\u{2B21} \u{2B21}", "⠯⠪⠕⠽⠯⠪⠕⠽")]
    #[case::upper_then_lower(" \u{2B21}\n\u{2B21} \u{2B21}", "⠯⠪⠕⠽⠬⠯⠪⠕⠽⠩⠯⠪⠕⠽")]
    #[case::horizontal_zigzag(" \u{2394}\n\u{2394} \u{2394}", "⠯⠕⠪⠽⠬⠯⠕⠪⠽⠩⠯⠕⠪⠽")]
    #[case::bonded_to_the_first(" \u{2B21}\n\u{2B21}\n \u{2B21}", "⠯⠪⠕⠽⠬⠯⠪⠕⠽⠤⠩⠯⠪⠕⠽")]
    fn writes_rings_in_symbol_form(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(braille(text).as_deref(), Some(expected));
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::other_text("\u{2B21}A")]
    #[case::mixed_rings("\u{2B21} \u{2B20}")]
    #[case::apart("\u{2B21}   \u{2B21}")]
    #[case::horizontal_side_by_side("\u{2394} \u{2394}")]
    #[case::pentagons_do_not_fuse(" \u{2B20}\n\u{2B20}")]
    #[case::bonded_to_neither("\u{2B21} \u{2B21}\n\n     \u{2B21}")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(braille(text), None);
    }
}
