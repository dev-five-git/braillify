use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_last_error(err: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(err);
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

/// Convert a Rust `String` into a heap-allocated C string, recording any
/// embedded-NUL conversion failure on the thread-local error slot.
///
/// In practice `braillify::encode_to_unicode` and `encode_to_braille_font`
/// return only Braille Unicode codepoints (U+2800..=U+28FF) and printable
/// ASCII, never containing `'\0'`, so the `Err` arm is defensive. The helper
/// exists primarily so the failure path is directly unit-testable.
fn into_cstring_ptr_or_null(result: String) -> *mut c_char {
    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(e) => {
            set_last_error(format!("CString conversion error: {}", e));
            ptr::null_mut()
        }
    }
}

/// 마지막 에러 메시지를 반환합니다. 호출자가 braillify_free_string으로 해제해야 합니다.
/// Returns the last error message. Caller must free with braillify_free_string.
#[unsafe(no_mangle)]
pub extern "C" fn braillify_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        Some(msg) => CString::new(msg.clone())
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        None => ptr::null_mut(),
    })
}

/// 텍스트를 점자 바이트 배열로 인코딩합니다.
/// Encodes text to braille byte array.
/// 성공 시 바이트 배열 포인터 반환, 실패 시 null 반환.
/// Returns byte array pointer on success, null on failure.
///
/// # Safety
/// `text` and `out_len` must be valid non-null pointers for the duration of the call.
/// `text` must point to a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode(text: *const c_char, out_len: *mut usize) -> *mut u8 {
    clear_last_error();

    if text.is_null() || out_len.is_null() {
        set_last_error("Null pointer argument".to_string());
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(text) };
    let text_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8: {}", e));
            return ptr::null_mut();
        }
    };

    match braillify::encode(text_str) {
        Ok(result) => {
            unsafe { *out_len = result.len() };
            let boxed = result.into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// 텍스트를 점자 유니코드 문자열로 인코딩합니다.
/// Encodes text to braille unicode string.
///
/// # Safety
/// `text` must be a valid non-null pointer to a NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_to_unicode(text: *const c_char) -> *mut c_char {
    clear_last_error();

    if text.is_null() {
        set_last_error("Null pointer argument".to_string());
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(text) };
    let text_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8: {}", e));
            return ptr::null_mut();
        }
    };

    match braillify::encode_to_unicode(text_str) {
        Ok(result) => into_cstring_ptr_or_null(result),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// 텍스트를 점자 폰트 문자열로 인코딩합니다.
/// Encodes text to braille font string.
///
/// # Safety
/// `text` must be a valid non-null pointer to a NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_to_braille_font(text: *const c_char) -> *mut c_char {
    clear_last_error();

    if text.is_null() {
        set_last_error("Null pointer argument".to_string());
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(text) };
    let text_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8: {}", e));
            return ptr::null_mut();
        }
    };

    match braillify::encode_to_braille_font(text_str) {
        Ok(result) => into_cstring_ptr_or_null(result),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Rust에서 할당한 문자열을 해제합니다.
/// Frees a string allocated by Rust.
///
/// # Safety
/// `ptr` must be a pointer previously returned by this library from a string-returning FFI call,
/// and it must not be freed more than once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Rust에서 할당한 바이트 배열을 해제합니다.
/// Frees a byte array allocated by Rust.
///
/// # Safety
/// `ptr` and `len` must come from `braillify_encode`, and `ptr` must not be freed more than once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_free_bytes(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
    }
}

#[cfg(test)]
mod tests {
    //! FFI surface tests.
    //!
    //! Each exported function gets:
    //!   - happy path (valid UTF-8 input → non-null return)
    //!   - null-pointer guard (null input → null return + last_error set)
    //!   - invalid-UTF-8 guard (the CStr conversion error path)
    //!   - encode-error guard (e.g. emoji input rejected by `braillify::encode`)
    //!
    //! All allocations returned by the FFI are explicitly freed via the
    //! matching `braillify_free_*` calls so the leak-free path is exercised.
    use super::*;

    fn cstring(s: &str) -> CString {
        CString::new(s).expect("input must not contain NUL")
    }

    #[test]
    fn encode_happy_path_and_free() {
        let input = cstring("안녕");
        let mut out_len: usize = 0;
        let ptr = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };
        assert!(!ptr.is_null());
        assert!(out_len > 0);
        unsafe { braillify_free_bytes(ptr, out_len) };
    }

    #[test]
    fn encode_null_pointer_sets_last_error() {
        let mut out_len: usize = 0;
        let ptr = unsafe { braillify_encode(std::ptr::null(), &mut out_len) };
        assert!(ptr.is_null());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        unsafe { braillify_free_string(err_ptr) };
    }

    #[test]
    fn encode_null_out_len_sets_last_error() {
        let input = cstring("a");
        let ptr = unsafe { braillify_encode(input.as_ptr(), std::ptr::null_mut()) };
        assert!(ptr.is_null());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        unsafe { braillify_free_string(err_ptr) };
    }

    #[test]
    fn encode_invalid_utf8_sets_last_error() {
        // Build a CString-shaped buffer with raw invalid-UTF-8 bytes.
        let bytes: [u8; 3] = [0xFF, 0xFE, 0x00];
        let ptr = bytes.as_ptr() as *const c_char;
        let mut out_len: usize = 0;
        let result = unsafe { braillify_encode(ptr, &mut out_len) };
        assert!(result.is_null());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        unsafe { braillify_free_string(err_ptr) };
    }

    #[test]
    fn encode_engine_failure_sets_last_error() {
        // 😀 (emoji) is not a supported CharType → braillify::encode returns Err.
        let input = cstring("😀");
        let mut out_len: usize = 0;
        let ptr = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };
        assert!(ptr.is_null());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        unsafe { braillify_free_string(err_ptr) };
    }

    #[test]
    fn encode_to_unicode_happy_path() {
        let input = cstring("안녕");
        let ptr = unsafe { braillify_encode_to_unicode(input.as_ptr()) };
        assert!(!ptr.is_null());
        let cstr = unsafe { CStr::from_ptr(ptr) };
        assert!(!cstr.to_str().unwrap().is_empty());
        unsafe { braillify_free_string(ptr) };
    }

    #[test]
    fn encode_to_unicode_null_pointer() {
        let ptr = unsafe { braillify_encode_to_unicode(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn encode_to_unicode_invalid_utf8() {
        let bytes: [u8; 3] = [0xFF, 0xFE, 0x00];
        let ptr = unsafe { braillify_encode_to_unicode(bytes.as_ptr() as *const c_char) };
        assert!(ptr.is_null());
    }

    #[test]
    fn encode_to_unicode_engine_failure() {
        let input = cstring("😀");
        let ptr = unsafe { braillify_encode_to_unicode(input.as_ptr()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn encode_to_braille_font_happy_path() {
        let input = cstring("hi");
        let ptr = unsafe { braillify_encode_to_braille_font(input.as_ptr()) };
        assert!(!ptr.is_null());
        unsafe { braillify_free_string(ptr) };
    }

    #[test]
    fn encode_to_braille_font_null_pointer() {
        let ptr = unsafe { braillify_encode_to_braille_font(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn encode_to_braille_font_invalid_utf8() {
        let bytes: [u8; 3] = [0xFF, 0xFE, 0x00];
        let ptr = unsafe { braillify_encode_to_braille_font(bytes.as_ptr() as *const c_char) };
        assert!(ptr.is_null());
    }

    #[test]
    fn encode_to_braille_font_engine_failure() {
        let input = cstring("😀");
        let ptr = unsafe { braillify_encode_to_braille_font(input.as_ptr()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn get_last_error_returns_null_when_no_error() {
        // Clear by running a happy-path call first.
        let input = cstring("a");
        let mut out_len: usize = 0;
        let ptr = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };
        unsafe { braillify_free_bytes(ptr, out_len) };
        // Now the thread-local error must be empty.
        let err_ptr = braillify_get_last_error();
        assert!(err_ptr.is_null());
    }

    #[test]
    fn free_string_handles_null() {
        // Must not panic on a null pointer.
        unsafe { braillify_free_string(std::ptr::null_mut()) };
    }

    #[test]
    fn free_bytes_handles_null() {
        unsafe { braillify_free_bytes(std::ptr::null_mut(), 0) };
    }

    #[test]
    fn set_last_error_helper_round_trip() {
        set_last_error("test message".to_string());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        let msg = unsafe { CStr::from_ptr(err_ptr) }.to_str().unwrap();
        assert_eq!(msg, "test message");
        unsafe { braillify_free_string(err_ptr) };
        clear_last_error();
    }

    #[test]
    fn clear_last_error_resets() {
        set_last_error("x".to_string());
        clear_last_error();
        let err_ptr = braillify_get_last_error();
        assert!(err_ptr.is_null());
    }

    /// `into_cstring_ptr_or_null` happy path: pure ASCII string is converted
    /// into a non-null pointer.
    #[test]
    fn into_cstring_ptr_happy_path() {
        clear_last_error();
        let ptr = into_cstring_ptr_or_null("hello".to_string());
        assert!(!ptr.is_null());
        let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(s, "hello");
        unsafe { braillify_free_string(ptr) };
    }

    /// `into_cstring_ptr_or_null` failure path: a string containing an interior
    /// `\0` byte fails `CString::new` → null pointer + recorded last_error.
    /// This branch is defensive (braille output never contains NUL), but the
    /// helper exists to make it directly testable.
    #[test]
    fn into_cstring_ptr_with_interior_nul_sets_last_error() {
        clear_last_error();
        let with_nul = "abc\u{0}xyz".to_string();
        let ptr = into_cstring_ptr_or_null(with_nul);
        assert!(ptr.is_null());
        let err_ptr = braillify_get_last_error();
        assert!(!err_ptr.is_null());
        let msg = unsafe { CStr::from_ptr(err_ptr) }.to_str().unwrap();
        assert!(msg.contains("CString conversion error"));
        unsafe { braillify_free_string(err_ptr) };
    }
}
