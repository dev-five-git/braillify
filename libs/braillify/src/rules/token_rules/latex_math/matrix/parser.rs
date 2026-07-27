use super::{LatexMatrix, MatrixDelimiter};

pub(super) fn find_latex_matrix(latex_inner: &str) -> Option<LatexMatrix<'_>> {
    let begin_pos = latex_inner.find("\\begin{")?;
    let env_start = begin_pos + "\\begin{".len();
    let env_end = latex_inner[env_start..].find('}')? + env_start;
    let env = &latex_inner[env_start..env_end];

    // `\begin{array}{|c|c|c|}` 형태에서 column spec(`{...}`)을 건너뛴다.
    let mut body_start = env_end + 1;
    let mut array_column_spec = None;
    if env == "array" && latex_inner.as_bytes().get(body_start) == Some(&b'{') {
        let column_spec_start = body_start + 1;
        body_start = find_braced_group_end(latex_inner, body_start)?;
        array_column_spec = Some(&latex_inner[column_spec_start..body_start - 1]);
    }

    let end_marker = format!("\\end{{{env}}}");
    let relative_end = latex_inner[body_start..].find(&end_marker)?;
    let body_end = body_start + relative_end;
    let suffix_start = body_end + end_marker.len();
    let body = &latex_inner[body_start..body_end];

    let delimiter = match env {
        "matrix" | "pmatrix" | "bmatrix" | "Bmatrix" => MatrixDelimiter::Parentheses,
        "vmatrix" | "Vmatrix" => MatrixDelimiter::VerticalBars,
        "cases" => MatrixDelimiter::Cases,
        "array" if is_variation_table_array(array_column_spec, body) => MatrixDelimiter::Array,
        "array" => MatrixDelimiter::Parentheses,
        _ => return None,
    };

    Some(LatexMatrix {
        delimiter,
        prefix: &latex_inner[..begin_pos],
        body,
        suffix: &latex_inner[suffix_start..],
    })
}

fn find_braced_group_end(input: &str, open_brace: usize) -> Option<usize> {
    if input.as_bytes().get(open_brace) != Some(&b'{') {
        return None;
    }

    let mut depth = 1usize;
    let mut idx = open_brace + 1;
    while idx < input.len() {
        match input.as_bytes()[idx] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx + 1);
                }
            }
            _ => {}
        }
        idx += 1;
    }
    None
}

fn is_variation_table_array(column_spec: Option<&str>, body: &str) -> bool {
    body.contains("\\hline") && column_spec.is_some_and(|spec| spec.contains('|'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::bare_matrix(
        "\\begin{matrix} 1 & 2 \\\\ 3 & 4 \\end{matrix}",
        MatrixDelimiter::Parentheses
    )]
    #[case::parenthesized(
        "\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}",
        MatrixDelimiter::Parentheses
    )]
    #[case::bracketed(
        "\\begin{bmatrix} 1 & 0 \\\\ 0 & 1 \\end{bmatrix}",
        MatrixDelimiter::Parentheses
    )]
    #[case::braced("\\begin{Bmatrix} x & y \\end{Bmatrix}", MatrixDelimiter::Parentheses)]
    #[case::determinant(
        "\\begin{vmatrix} a & b \\\\ c & d \\end{vmatrix}",
        MatrixDelimiter::VerticalBars
    )]
    #[case::norm(
        "\\begin{Vmatrix} 1 & 0 \\\\ 0 & 1 \\end{Vmatrix}",
        MatrixDelimiter::VerticalBars
    )]
    #[case::cases(
        "\\begin{cases} a & x>0 \\\\ b & x<0 \\end{cases}",
        MatrixDelimiter::Cases
    )]
    #[case::generic_array(
        "\\begin{array}{cc} x & y \\\\ z & w \\end{array}",
        MatrixDelimiter::Parentheses
    )]
    #[case::variation_table(
        "\\begin{array}{p{2cm}|c|c|c|}\\hline x & y & z & w \\\\ \\hline\\end{array}",
        MatrixDelimiter::Array
    )]
    fn finds_supported_matrix_like_environments(
        #[case] latex: &str,
        #[case] expected: MatrixDelimiter,
    ) {
        let matrix = find_latex_matrix(latex).unwrap();

        assert_eq!(matrix.delimiter, expected);
        assert!(matrix.body.contains('&'));
    }

    #[rstest::rstest]
    #[case::boxed_table(Some("|c|c|"), "\\hline x & y \\\\ \\hline", true)]
    #[case::hline_without_column_rule(Some("cc"), "\\hline x & y \\\\ \\hline", false)]
    #[case::ruled_columns_without_hline(Some("|c|c|"), "x & y \\\\ z & w", false)]
    #[case::missing_column_spec(None, "\\hline x & y", false)]
    fn detects_variation_table_arrays(
        #[case] column_spec: Option<&str>,
        #[case] body: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(is_variation_table_array(column_spec, body), expected);
    }

    #[test]
    fn braced_group_rejects_non_opening_or_unclosed_brace() {
        assert!(find_braced_group_end("abc", 0).is_none());
        assert!(find_braced_group_end("{abc", 0).is_none());
    }

    /// 제10항 — `\begin{array}{|c|c|c|}` column spec with nested `{}` braces.
    #[test]
    fn array_column_spec_nested_braces() {
        // The {2cm} inside the column spec exercises the depth tracking.
        let result = crate::encode_to_unicode(
            "$\\begin{array}{p{2cm}|c|c|c|}\\hline x & y & z & w \\\\ \\hline\\end{array}$",
        );
        let unicode = result.unwrap();
        assert!(unicode.starts_with('⠖'));
        assert!(unicode.contains('⠓'));
    }

    /// Only matrix-like environments are recognised; an unknown `\begin{...}`
    /// environment is not a matrix and returns `None`.
    #[test]
    fn unknown_environment_is_not_a_matrix() {
        assert!(find_latex_matrix("\\begin{foobar} 1 & 2 \\end{foobar}").is_none());
    }
}
