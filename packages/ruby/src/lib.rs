use magnus::{Error, RString, Ruby, function};

/// 텍스트를 점자 바이트(ASCII-8BIT String)로 인코딩합니다.
fn encode(ruby: &Ruby, text: String) -> Result<RString, Error> {
    braillify::encode(&text)
        .map(|bytes| ruby.str_from_slice(&bytes))
        .map_err(|e| Error::new(ruby.exception_arg_error(), e))
}

/// 텍스트를 점자 유니코드 문자열로 인코딩합니다.
fn translate_to_unicode(ruby: &Ruby, text: String) -> Result<String, Error> {
    braillify::encode_to_unicode(&text).map_err(|e| Error::new(ruby.exception_arg_error(), e))
}

/// 텍스트를 점자 폰트 문자열로 인코딩합니다.
fn translate_to_braille_font(ruby: &Ruby, text: String) -> Result<String, Error> {
    braillify::encode_to_braille_font(&text).map_err(|e| Error::new(ruby.exception_arg_error(), e))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Braillify")?;
    module.define_module_function("encode", function!(encode, 1))?;
    module.define_module_function("translate_to_unicode", function!(translate_to_unicode, 1))?;
    module.define_module_function(
        "translate_to_braille_font",
        function!(translate_to_braille_font, 1),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //! magnus 바인딩 테스트.
    //!
    //! `#[ruby_test]`(rb-sys-test-helpers)가 각 테스트 전에 임베디드 Ruby VM을
    //! 부트하므로 외부 Ruby 프로세스 없이 `Ruby::get()`이 사용 가능하다.
    //! python 바인딩(packages/python/src/lib.rs)의 테스트 구성을 미러링한다.
    use super::*;
    use magnus::prelude::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn encode_happy_path_returns_bytes() {
        let ruby = Ruby::get().unwrap();
        let result = encode(&ruby, "안녕".to_string()).expect("encode must succeed");
        assert!(!result.is_empty());
    }

    #[ruby_test]
    fn encode_engine_failure_maps_to_arg_error() {
        let ruby = Ruby::get().unwrap();
        // 😀는 지원하지 않는 CharType → core encode가 Err 반환 → ArgumentError 매핑.
        assert!(encode(&ruby, "😀".to_string()).is_err());
    }

    #[ruby_test]
    fn translate_to_unicode_happy_path() {
        let ruby = Ruby::get().unwrap();
        let result = translate_to_unicode(&ruby, "hi".to_string()).expect("must succeed");
        assert!(!result.is_empty());
        // 출력은 점자 유니코드(U+2800..=U+28FF) 범위여야 한다.
        for ch in result.chars() {
            let cp = ch as u32;
            assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
        }
    }

    #[ruby_test]
    fn translate_to_unicode_failure_maps_to_arg_error() {
        let ruby = Ruby::get().unwrap();
        assert!(translate_to_unicode(&ruby, "😀".to_string()).is_err());
    }

    #[ruby_test]
    fn translate_to_braille_font_happy_path() {
        let ruby = Ruby::get().unwrap();
        let result = translate_to_braille_font(&ruby, "hi".to_string()).expect("must succeed");
        assert!(!result.is_empty());
    }

    #[ruby_test]
    fn translate_to_braille_font_failure_maps_to_arg_error() {
        let ruby = Ruby::get().unwrap();
        assert!(translate_to_braille_font(&ruby, "😀".to_string()).is_err());
    }

    /// `#[magnus::init]` 등록 본문을 실행하고, 등록된 모듈 함수가 Ruby에서
    /// 실제로 호출 가능한지 funcall로 검증한다.
    #[ruby_test]
    fn init_registers_all_module_functions() {
        let ruby = Ruby::get().unwrap();
        init(&ruby).expect("module init");
        let module = ruby.define_module("Braillify").unwrap();
        let unicode: String = module.funcall("translate_to_unicode", ("안녕",)).unwrap();
        assert!(!unicode.is_empty());
        let font: String = module
            .funcall("translate_to_braille_font", ("안녕",))
            .unwrap();
        assert!(!font.is_empty());
        let bytes: RString = module.funcall("encode", ("안녕",)).unwrap();
        assert!(!bytes.is_empty());
    }
}
