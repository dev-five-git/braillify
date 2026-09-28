//! 과학 제18항 5 — 화살표 위아래에 적은 반응 조건. 위의 내용은 화살표 앞에,
//! 아래의 내용은 화살표 뒤에 ⠸⠷ ⠸⠾ 으로 묶어 붙여 적는다.
//!
//! 입력은 조건을 화살표와 같은 칸에 맞춰 식의 위아래 줄에 적은 글이다.

use crate::unicode::decode_unicode;

const ARROWS: [char; 4] = ['→', '←', '⇄', '⇌'];

/// 조건 줄의 글자가 화살표 칸에서 시작하면 그 조건.
fn condition_over(line: &str, arrow_column: usize) -> Option<&str> {
    let start = line.chars().take_while(|c| *c == ' ').count();
    let text = line.trim();
    (!text.is_empty() && start.abs_diff(arrow_column) <= 1).then_some(text)
}

fn wrap(condition: &[u8]) -> Vec<u8> {
    let mut out = vec![decode_unicode('⠸'), decode_unicode('⠷')];
    out.extend_from_slice(condition);
    out.extend([decode_unicode('⠸'), decode_unicode('⠾')]);
    out
}

/// 식 한 줄과 그 위·아래 조건 줄로 된 글을 적는다. 그런 모양이 아니면 `None`.
pub(crate) fn encode(
    text: &str,
    encode_formula: impl Fn(&str) -> Option<Vec<u8>>,
    encode_condition: impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let lines: Vec<&str> = text.split('\n').collect();
    let formula_at = lines
        .iter()
        .position(|line| line.chars().filter(|c| ARROWS.contains(c)).count() == 1)?;
    if lines.len() > 3 || formula_at > 1 {
        return None;
    }
    let formula = lines[formula_at];
    let arrow_column = formula.chars().position(|c| ARROWS.contains(&c))?;
    let above = formula_at
        .checked_sub(1)
        .map(|at| condition_over(lines[at], arrow_column));
    let below = lines
        .get(formula_at + 1)
        .map(|line| condition_over(line, arrow_column));
    if above.flatten().is_none() && below.flatten().is_none()
        || above.is_some_and(|c| c.is_none())
        || below.is_some_and(|c| c.is_none())
    {
        return None;
    }

    // 화살표의 점형은 화살표만 바꾼 식과 견주어 찾는다.
    let arrow = formula.chars().nth(arrow_column)?;
    let other = if arrow == '→' { '←' } else { '→' };
    let cells = encode_formula(formula)?;
    let swapped = encode_formula(&formula.replace(arrow, &other.to_string()))?;
    let start = cells
        .iter()
        .zip(&swapped)
        .take_while(|(a, b)| a == b)
        .count();
    let end = cells[start..]
        .iter()
        .position(|cell| *cell == 0)
        .map_or(cells.len(), |p| start + p);
    if start == cells.len() || start == end {
        return None;
    }

    let mut out = cells[..start].to_vec();
    if let Some(condition) = above.flatten() {
        out.extend(wrap(&encode_condition(condition)?));
    }
    out.extend_from_slice(&cells[start..end]);
    if let Some(condition) = below.flatten() {
        out.extend(wrap(&encode_condition(condition)?));
    }
    out.extend_from_slice(&cells[end..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formula(text: &str) -> Option<Vec<u8>> {
        Some(
            text.chars()
                .map(|c| match c {
                    ' ' => 0,
                    '→' => decode_unicode('⠒'),
                    '⇄' => decode_unicode('⠶'),
                    _ => decode_unicode('⠁'),
                })
                .collect(),
        )
    }

    fn condition(_: &str) -> Option<Vec<u8>> {
        Some(vec![decode_unicode('⠃')])
    }

    fn braille(cells: Vec<u8>) -> String {
        cells
            .into_iter()
            .map(crate::unicode::encode_unicode)
            .collect()
    }

    #[rstest::rstest]
    #[case::both(" x\na ⇄ a\n  y", "⠁⠀⠸⠷⠃⠸⠾⠶⠸⠷⠃⠸⠾⠀⠁")]
    #[case::above_only("  x\na → a", "⠁⠀⠸⠷⠃⠸⠾⠒⠀⠁")]
    fn places_conditions_around_the_arrow(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text, formula, condition).map(braille).as_deref(),
            Some(expected)
        );
    }

    #[rstest::rstest]
    #[case::one_line("a → a")]
    #[case::not_over_the_arrow("x\na     → a")]
    #[case::no_arrow("x\na a")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text, formula, condition), None);
    }
}
