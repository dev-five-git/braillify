//! 과학 제13항 3·4 — 결합선만으로 그린 환핵 그림의 공간 표기 형식.
//!
//! 묵자 그림을 한 칸씩 옮긴다. 오른쪽 위로 가는 사선은 ⠜, 왼쪽 위로 가는 사선은
//! ⠣, 세로선은 ⠸ 이고 빈칸은 그대로 둔다.

use crate::unicode::decode_unicode;

const LINE_BREAK: u8 = 255;

fn line_cell(c: char) -> Option<u8> {
    match c {
        '/' => Some(decode_unicode('⠜')),
        '\\' => Some(decode_unicode('⠣')),
        '|' => Some(decode_unicode('⠸')),
        ' ' => Some(0),
        _ => None,
    }
}

/// 결합선만으로 된 여러 줄 그림이면 그 점자. 아니면 `None`.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    if !text.contains('\n') || text.trim().is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for (at, line) in text.split('\n').enumerate() {
        if at > 0 {
            out.push(LINE_BREAK);
        }
        for c in line.trim_end().chars() {
            out.push(line_cell(c)?);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(text: &str) -> Option<String> {
        encode(text).map(|cells| {
            cells
                .into_iter()
                .map(|c| {
                    if c == LINE_BREAK {
                        '\n'
                    } else {
                        crate::unicode::encode_unicode(c)
                    }
                })
                .collect()
        })
    }

    #[rstest::rstest]
    #[case::slopes(" /\\\n/  \\", " ⠜⠣\n⠜⠀⠀⠣")]
    #[case::double_vertical("|  ||\n|  ||", "⠸⠀⠀⠸⠸\n⠸⠀⠀⠸⠸")]
    fn copies_the_drawing_cell_by_cell(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            braille(text).as_deref(),
            Some(expected.replace(' ', "⠀").as_str())
        );
    }

    #[rstest::rstest]
    #[case::one_line("/\\")]
    #[case::with_atoms(" H\n/ \\")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(braille(text), None);
    }
}
