//! 수학 제17항 — 프라임 기호.
//!
//! ′(U+2032), ″(U+2033), ‴(U+2034)는 각각 `-`, `--`, `---`로 인코딩한다.
//! 내부 코드로는 단일 프라임이 36(⠤)이다.

pub fn is_prime_mark(c: char) -> bool {
    matches!(c, '\u{2032}' | '\u{2033}' | '\u{2034}')
}

pub fn encode_prime(c: char, result: &mut Vec<u8>) -> Result<(), String> {
    match c {
        '\u{2032}' => {
            result.push(36);
            Ok(())
        }
        '\u{2033}' => {
            result.extend_from_slice(&[36, 36]);
            Ok(())
        }
        '\u{2034}' => {
            result.extend_from_slice(&[36, 36, 36]);
            Ok(())
        }
        _ => Err(format!("unsupported prime mark: {c}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_prime_variants() -> Result<(), String> {
        let mut single = Vec::new();
        encode_prime('\u{2032}', &mut single)?;
        assert_eq!(single, vec![36]);

        let mut double = Vec::new();
        encode_prime('\u{2033}', &mut double)?;
        assert_eq!(double, vec![36, 36]);

        let mut triple = Vec::new();
        encode_prime('\u{2034}', &mut triple)?;
        assert_eq!(triple, vec![36, 36, 36]);

        Ok(())
    }

    #[test]
    fn rejects_unsupported_prime() {
        let mut result = Vec::new();
        assert!(encode_prime('a', &mut result).is_err());
        assert!(!is_prime_mark('a'));
    }
}
