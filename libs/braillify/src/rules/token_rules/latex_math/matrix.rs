//! Matrix-related encoding for LaTeX expressions (extracted from latex_math.rs).
//!
//! Handles \\begin{matrix}, \\begin{array}, \\begin{pmatrix}, etc.

use crate::rules::math;
use crate::rules::math::math_token_rule::MathContext;
use crate::unicode::decode_unicode;

use super::strip_latex_to_math;

mod parser;
mod suffix;

#[cfg(test)]
pub(super) use suffix::subscript_digit_to_ascii;
use suffix::{encode_matrix_letter_with_numeric_subscripts, encode_matrix_suffix};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatrixDelimiter {
    Parentheses,
    VerticalBars,
    Cases,
    /// PDF 제10항 — `\begin{array}` 증감표. 상하 테두리(`⠖...⠲` / `⠓...⠚`)로 감싼다.
    Array,
}

impl MatrixDelimiter {
    fn open_bytes(self) -> Vec<u8> {
        match self {
            MatrixDelimiter::Parentheses => vec![decode_unicode('⠦')],
            MatrixDelimiter::VerticalBars => vec![decode_unicode('⠳')],
            // PDF 제6항 1 — 연립식(`\begin{cases}`)은 `⠶⠄`로 시작한다.
            MatrixDelimiter::Cases => vec![decode_unicode('⠶'), decode_unicode('⠄')],
            // PDF 제10항 — Array는 `encode_latex_array`로 분기되므로 호출자는
            // 이 함수에 Array variant를 절대 전달하지 않는다. 따라서 이 arm은
            // 호출 컨트랙트상 도달 불가능.
            MatrixDelimiter::Array => unreachable!(
                "MatrixDelimiter::Array is dispatched to encode_latex_array; \
                 open_bytes must never be called for the Array variant"
            ),
        }
    }

    fn close_bytes(self) -> Vec<u8> {
        match self {
            MatrixDelimiter::Parentheses => vec![decode_unicode('⠴')],
            MatrixDelimiter::VerticalBars => vec![decode_unicode('⠳')],
            // PDF 제6항 1 — 연립식 종결은 `⠠⠶`.
            MatrixDelimiter::Cases => vec![decode_unicode('⠠'), decode_unicode('⠶')],
            // See `open_bytes` — Array variant is dispatched to `encode_latex_array`
            // and never reaches this match.
            MatrixDelimiter::Array => unreachable!(
                "MatrixDelimiter::Array is dispatched to encode_latex_array; \
                 close_bytes must never be called for the Array variant"
            ),
        }
    }
}

pub(super) struct LatexMatrix<'a> {
    delimiter: MatrixDelimiter,
    prefix: &'a str,
    body: &'a str,
    suffix: &'a str,
}

pub(super) fn find_latex_matrix(latex_inner: &str) -> Option<LatexMatrix<'_>> {
    parser::find_latex_matrix(latex_inner)
}

fn split_matrix_body(body: &str) -> Vec<Vec<String>> {
    let mut rows = vec![Vec::new()];
    let mut current = String::new();
    let mut brace_depth = 0usize;
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                current.push(ch);
            }
            '&' if brace_depth == 0 => {
                if let Some(row) = rows.last_mut() {
                    row.push(current.trim().to_string());
                }
                current.clear();
            }
            '\\' if brace_depth == 0 && chars.peek() == Some(&'\\') => {
                chars.next();
                if let Some(row) = rows.last_mut() {
                    row.push(current.trim().to_string());
                }
                current.clear();
                rows.push(Vec::new());
            }
            _ => current.push(ch),
        }
    }

    if let Some(row) = rows.last_mut()
        && (!current.trim().is_empty() || !row.is_empty())
    {
        row.push(current.trim().to_string());
    }

    rows.into_iter().filter(|row| !row.is_empty()).collect()
}

fn promote_matrix_cell_variable(math_text: &str) -> String {
    // PDF 제26항: 행렬 원소는 소문자 변수를 그대로 사용한다 (대문자 변환 불필요)
    math_text.to_string()
}

fn encode_trimmed_math(text: &str, math_context: MathContext) -> Result<Vec<u8>, String> {
    let math_text = strip_latex_to_math(text.trim());
    if math_text.trim().is_empty() {
        return Ok(Vec::new());
    }
    math::encoder::encode_math_expression_with_context(&math_text, math_context)
}

fn encode_matrix_cell(cell: &str, math_context: MathContext) -> Result<Vec<u8>, String> {
    let math_text = strip_latex_to_math(cell.trim());
    let matrix_text = promote_matrix_cell_variable(&math_text);
    if let Some(bytes) = encode_matrix_letter_with_numeric_subscripts(&matrix_text, math_context)? {
        return Ok(bytes);
    }
    math::encoder::encode_math_expression_with_context(&matrix_text, math_context)
}

pub(super) fn encode_latex_matrix(
    matrix: &LatexMatrix<'_>,
    math_context: MathContext,
) -> Result<Vec<u8>, String> {
    // PDF 제10항 — `\begin{array}` 증감표: 위/아래 박스 테두리로 감싼 표.
    if matrix.delimiter == MatrixDelimiter::Array {
        return encode_latex_array(matrix, math_context);
    }

    let mut out = encode_trimmed_math(matrix.prefix, math_context)?;
    out.extend(matrix.delimiter.open_bytes());

    let rows = split_matrix_body(matrix.body);
    let is_cases = matrix.delimiter == MatrixDelimiter::Cases;
    for (row_index, row) in rows.iter().enumerate() {
        for (cell_index, cell) in row.iter().enumerate() {
            out.extend(encode_matrix_cell(cell, math_context)?);
            if cell_index + 1 < row.len() {
                out.push(0);
            }
        }
        if row_index + 1 < rows.len() {
            if is_cases {
                // PDF 제6항 1 — cases 환경의 행 구분자는 단일 공백.
                out.push(0);
            } else {
                out.push(0);
                out.push(decode_unicode('⠜'));
                out.push(0);
            }
        }
    }

    out.extend(matrix.delimiter.close_bytes());
    out.extend(encode_matrix_suffix(matrix.suffix, math_context)?);
    Ok(out)
}

/// PDF 제10항 — `\begin{array}` 증감표 인코더.
///
/// 출력 구조 (5라인, 각 라인 32 cells):
/// - 위 테두리: `⠖` + 30 × `⠒` + `⠲`
/// - 내용 라인: 2sp leading + cell1 + 2sp + cell2 + 2sp + cell3 + 2sp + cell4 + trailing pad to 32
/// - 아래 테두리: `⠓` + 30 × `⠒` + `⠚`
///
/// body에서 `\hline`을 제거하고 `\\`로 행 분리, `&`로 셀 분리한 뒤 각 셀을 math로 인코딩한다.
pub(super) fn encode_latex_array(
    matrix: &LatexMatrix<'_>,
    math_context: MathContext,
) -> Result<Vec<u8>, String> {
    let mut out = encode_trimmed_math(matrix.prefix, math_context)?;

    // `\hline`을 제거하고 본문을 정리.
    let body_no_hline = matrix.body.replace("\\hline", "");
    let rows = split_matrix_body(&body_no_hline);

    // 각 행의 내용을 인코딩 (셀 사이 2-칸 separator, 앞뒤 2-칸 padding).
    let mut encoded_rows: Vec<Vec<u8>> = Vec::new();
    for row in rows
        .iter()
        .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
    {
        let mut row_bytes = Vec::new();
        row_bytes.push(0); // 2 leading spaces
        row_bytes.push(0);
        let non_empty_cells = row.iter().enumerate().filter(|(_, c)| !c.trim().is_empty());
        for (display_index, (_, cell)) in non_empty_cells.enumerate() {
            if display_index > 0 {
                row_bytes.push(0); // 2 separator spaces
                row_bytes.push(0);
            }
            row_bytes.extend(encode_matrix_cell(cell, math_context)?);
        }
        encoded_rows.push(row_bytes);
    }

    // 테두리 너비 결정: PDF 제10항 — 4열 증감표는 30 dashes.
    // 일반적인 규칙: max(max_row_width, 30).
    let max_row_width = encoded_rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let inner_width = max_row_width.max(30);
    let total_width = inner_width + 2; // + 2 corners

    // 위 테두리 emit.
    out.push(decode_unicode('⠖'));
    for _ in 0..inner_width {
        out.push(decode_unicode('⠒'));
    }
    out.push(decode_unicode('⠲'));

    // 각 내용 행: trailing pad to total_width.
    for row_bytes in &encoded_rows {
        out.extend_from_slice(row_bytes);
        // trailing space padding to align row length.
        out.resize(out.len() + (total_width - row_bytes.len()), 0);
    }

    // 아래 테두리 emit.
    out.push(decode_unicode('⠓'));
    for _ in 0..inner_width {
        out.push(decode_unicode('⠒'));
    }
    out.push(decode_unicode('⠚'));

    out.extend(encode_matrix_suffix(matrix.suffix, math_context)?);
    Ok(out)
}

#[cfg(test)]
mod tests {
    /// `\begin{array}` with empty rows/cells drives lines 286, 294.
    /// Row entirely empty → `continue` at 286; specific empty cell → `continue` at 294.
    #[test]
    fn array_with_empty_rows_and_cells() {
        // Use real LaTeX input. The `\\` between cells creates rows; empty rows after \hline
        // and empty cells (between consecutive &) exercise both early-continues.
        let result = crate::encode_to_unicode(
            "$\\begin{array}{c|c|c|c}\\hline x & & y &\\\\\\hline\\end{array}$",
        );
        let unicode = result.unwrap();
        assert!(unicode.starts_with('⠖'));
        assert!(unicode.contains('⠓'));
    }
}
