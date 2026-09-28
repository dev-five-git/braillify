//! 과학 점자 규정 — 화학식·화학 반응식과 그 원소 기호.

pub mod bond_lines;
pub mod circuit;
pub mod conditions;
pub mod diagram;
pub mod elements;
pub mod formula;
pub mod genotype;
pub mod quantity;
pub mod ring;
pub mod weather;

/// 묵자가 그림뿐인 기호는 입력에 `[그림: 명칭]` 으로 적는다. 그 명칭.
pub(crate) fn picture_name(text: &str) -> Option<&str> {
    text.strip_prefix("[그림:")?
        .strip_suffix(']')
        .map(str::trim)
}
