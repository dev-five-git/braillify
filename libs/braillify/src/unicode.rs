pub fn encode_unicode(text: u8) -> char {
    if text == 255 {
        return '\n';
    }
    char::from_u32(text as u32 + 0x2800).unwrap()
}

pub const fn decode_unicode(text: char) -> u8 {
    if (text as u32) < 0x2800 {
        panic!("Invalid unicode character");
    }
    (text as u32 - 0x2800) as u8
}

#[cfg(test)]
mod test {
    use super::*;
    /// `encode_unicode(b)`는 0..=63 범위 byte를 U+2800+b 점자 문자로 변환한다.
    /// 6-dot 64 패턴 전체 (⠀..⠿)를 한 번에 검증.
    #[test]
    fn encode_unicode_maps_byte_to_braille_offset() {
        for b in 0u8..64 {
            let expected = char::from_u32(u32::from(b) + 0x2800).unwrap();
            assert_eq!(
                encode_unicode(b),
                expected,
                "byte {b} should map to {expected:?}"
            );
        }
    }

    /// `encode_unicode(255)`는 special-case로 newline을 반환한다 (line break marker).
    #[test]
    fn encode_unicode_255_returns_newline() {
        assert_eq!(encode_unicode(255), '\n');
    }

    /// unicode.rs line 10 - decode_unicode panics for chars before U+2800.
    #[test]
    #[should_panic(expected = "Invalid unicode character")]
    fn decode_unicode_panics_for_non_braille_char() {
        let _ = decode_unicode('A');
    }
}
