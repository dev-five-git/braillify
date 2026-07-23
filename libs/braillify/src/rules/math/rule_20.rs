//! 수학 제20항 — 약동호.
//!
//! 약동호 ≒(U+2252)는 `rule_3::is_equality_symbol`에 포함되어 있어 equality
//! dispatch에서 처리된다. 별도 rule_20 dispatch는 dead code였다.
//! 본 파일은 PDF 규정 추적용으로만 보존된다.

#[cfg(test)]
mod tests {
    /// PDF 제20항 — `≒` (U+2252) 약동호 encode 파이프라인.
    /// rule_3::encode_equality_symbol 경로로 처리된다.
    #[test]
    fn approximation_symbol_through_pipeline() {
        let result = crate::encode("\u{2252}");
        assert!(result.is_ok());
        let result = crate::encode("\u{221A}3\u{2252}1.732");
        assert!(result.is_ok());
    }
}
