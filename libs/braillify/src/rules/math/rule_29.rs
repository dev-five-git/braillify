//! 수학 제29항 — 근사 등호 (≈).
//!
//! 근사 등호 `≈`(U+2248)는 `rule_3::is_equality_symbol`에 포함되어 있어
//! equality dispatch에서 처리된다. 별도 rule_29 dispatch는 dead code였다.
//! 본 파일은 PDF 규정 추적용으로만 보존된다.

#[cfg(test)]
mod tests {
    /// PDF 제29항 — `≈` (U+2248) 근사 등호 encode 파이프라인.
    /// rule_3::encode_equality_symbol 경로로 처리된다.
    #[test]
    fn approximate_equal_through_pipeline() {
        let result = crate::encode("\u{2248}");
        assert!(result.is_ok());
        let result = crate::encode("X\u{2248}F/N");
        assert!(result.is_ok());
    }
}
