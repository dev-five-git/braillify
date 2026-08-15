mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "encode")]
pub fn encode(text: &str) -> Result<Vec<u8>, String> {
    braillify::encode(text)
}

#[wasm_bindgen(js_name = "translateToUnicode")]
pub fn translate_to_unicode(text: &str) -> Result<String, String> {
    braillify::encode_to_unicode(text)
}

#[wasm_bindgen(js_name = "translateToBrailleFont")]
pub fn translate_to_braille_font(text: &str) -> Result<String, String> {
    braillify::encode_to_braille_font(text)
}

#[wasm_bindgen(js_name = "decodeFromUnicode")]
pub fn decode_from_unicode(braille: &str) -> Result<String, String> {
    braillify::decode(braille)
}

#[cfg(test)]
mod tests {
    //! Native-host tests for the wasm-bindgen shim. `wasm_bindgen` macros
    //! collapse to plain Rust functions on non-wasm targets, so the
    //! delegations to `braillify::*` are reachable by `cargo test`.
    use super::*;

    #[test]
    fn encode_delegates_to_core() {
        let result = encode("안녕").expect("encode must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_propagates_error() {
        // Emoji is rejected by core encoder → wasm shim propagates `Err`.
        assert!(encode("😀").is_err());
    }

    #[test]
    fn translate_to_unicode_delegates_to_core() {
        let result = translate_to_unicode("hi").expect("must succeed");
        for ch in result.chars() {
            let cp = ch as u32;
            assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
        }
    }

    #[test]
    fn translate_to_unicode_propagates_error() {
        assert!(translate_to_unicode("😀").is_err());
    }

    #[test]
    fn translate_to_braille_font_delegates_to_core() {
        let result = translate_to_braille_font("hi").expect("must succeed");
        assert!(!result.is_empty());
    }

    #[test]
    fn translate_to_braille_font_propagates_error() {
        assert!(translate_to_braille_font("😀").is_err());
    }

    /// JS에 노출되는 역점역 wrapper가 입력을 바꾸지 않고 core decoder에 위임한다.
    #[test]
    fn decode_from_unicode_delegates_to_core() {
        assert_eq!(
            decode_from_unicode("⠈⠎⠐⠕").expect("decode must succeed"),
            "거리"
        );
    }

    /// Node 공개 encoder가 만든 Unicode 점자를 같은 패키지의 decoder가 소비할 수 있어야 한다.
    #[test]
    fn decode_from_unicode_roundtrips_core_unicode() {
        let encoded = translate_to_unicode("강아지").expect("encode must succeed");

        assert_eq!(
            decode_from_unicode(&encoded).expect("decode must succeed"),
            "강아지"
        );
    }

    #[test]
    fn set_panic_hook_is_callable() {
        // Exercises the no-op path on default (no `console_error_panic_hook` feature).
        utils::set_panic_hook();
    }
}
