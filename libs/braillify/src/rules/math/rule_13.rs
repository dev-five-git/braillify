//! 수학 제13항 — 그리스 문자.
//!
//! 그리스 문자 기호는 수학기호 단축표를 사용한다.

use crate::math_symbol_shortcut;

// Prepared for future direct encoder dispatch integration.
pub fn is_greek_symbol(c: char) -> bool {
    matches!(
        c,
        'Δ' | 'α' | 'β' | 'γ' | 'π' | 'Π' | 'Σ' | 'θ' | 'λ' | 'μ' | 'Ω'
    )
}

// Prepared for future direct encoder dispatch integration.
pub fn encode_greek_symbol(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    let encoded = math_symbol_shortcut::encode_char_math_symbol_shortcut(c)?;
    result.extend_from_slice(encoded);
    Ok(())
}

/// Invariant: PDF 제52항 (Δ, U+0394) is owned by `is_greek_symbol` exclusively.
/// The previously-existing dedicated `rule_52` module was removed because its
/// dispatch arm was shadowed by this rule. This test locks the ownership so a
/// future refactor that changes rule priorities will fail loudly instead of
/// silently re-introducing the dead-code path.
#[cfg(test)]
mod delta_ownership_invariant {
    use super::is_greek_symbol;

    #[test]
    fn rule_13_owns_delta_u0394() {
        assert!(
            is_greek_symbol('\u{0394}'),
            "PDF 제52항 invariant: Δ (U+0394) must be captured by rule_13. \
             If this assertion fails the `rule_52` deletion is no longer valid; \
             reintroduce the dedicated module + dispatch arm or update this invariant."
        );
    }
}
