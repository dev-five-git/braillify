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

fn string_into_c_ptr(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(c_string) => c_string.into_raw(),
        Err(e) => {
            set_last_error(format!("CString conversion error: {}", e));
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn braillify_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        // Error messages never contain null bytes, so unwrap_or(null)
        // is defensive dead code.
        Some(msg) => CString::new(msg.clone())
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        None => ptr::null_mut(),
    })
}

/// Encodes a null-terminated UTF-8 C string into Braille bytes.
///
/// On success, the output length is written to `out_len`.
/// The returned buffer must be released using [`braillify_free_bytes`].
///
/// # Safety
///
/// - `text` must be null or point to a valid null-terminated C string.
/// - `out_len` must be null or point to valid writable memory for a `usize`.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_bytes`] with the length written to `out_len`.
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
            unsafe {
                *out_len = result.len();
            }

            let boxed = result.into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Encodes a null-terminated UTF-8 C string into Unicode Braille.
///
/// The returned string must be released using [`braillify_free_string`].
///
/// # Safety
///
/// - `text` must be null or point to a valid null-terminated C string.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_string`].
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
        Ok(result) => string_into_c_ptr(result),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Encodes a null-terminated UTF-8 C string for use with a Braille font.
///
/// The returned string must be released using [`braillify_free_string`].
///
/// # Safety
///
/// - `text` must be null or point to a valid null-terminated C string.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_string`].
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
        Ok(result) => string_into_c_ptr(result),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// # Safety
///
/// `value` must be null or point to a valid null-terminated C string.
unsafe fn read_str<'a>(value: *const c_char) -> Result<&'a str, String> {
    if value.is_null() {
        return Err("Null pointer argument".to_string());
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Encodes a null-terminated UTF-8 C string, read in a named context
/// (`science`, `math`, `korean`, ...), into Braille bytes. An unknown context
/// is an error.
///
/// # Safety
///
/// - `text` and `context` must be null or point to valid null-terminated C strings.
/// - `out_len` must be null or point to valid writable memory for a `usize`.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_bytes`] with the length written to `out_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_in_context(
    text: *const c_char,
    context: *const c_char,
    out_len: *mut usize,
) -> *mut u8 {
    clear_last_error();
    if out_len.is_null() {
        set_last_error("Null pointer argument".to_string());
        return ptr::null_mut();
    }
    let encoded = unsafe { read_str(text) }.and_then(|text| {
        let context = unsafe { read_str(context) }?;
        braillify::encode_in_context(text, context)
    });
    match encoded {
        Ok(result) => {
            unsafe {
                *out_len = result.len();
            }
            Box::into_raw(result.into_boxed_slice()) as *mut u8
        }
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// # Safety
///
/// `text` and `context` must be null or point to valid null-terminated C strings.
unsafe fn translate_in_context(
    text: *const c_char,
    context: *const c_char,
    translate: fn(&str, &str) -> Result<String, String>,
) -> *mut c_char {
    clear_last_error();
    let translated = unsafe { read_str(text) }.and_then(|text| {
        let context = unsafe { read_str(context) }?;
        translate(text, context)
    });
    match translated {
        Ok(result) => string_into_c_ptr(result),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Encodes a null-terminated UTF-8 C string, read in a named context, into
/// Unicode Braille.
///
/// # Safety
///
/// - `text` and `context` must be null or point to valid null-terminated C strings.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_string`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_to_unicode_in_context(
    text: *const c_char,
    context: *const c_char,
) -> *mut c_char {
    unsafe { translate_in_context(text, context, braillify::encode_to_unicode_in_context) }
}

/// Encodes a null-terminated UTF-8 C string, read in a named context, for use
/// with a Braille font.
///
/// # Safety
///
/// - `text` and `context` must be null or point to valid null-terminated C strings.
/// - A non-null returned pointer must be freed exactly once using
///   [`braillify_free_string`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_to_braille_font_in_context(
    text: *const c_char,
    context: *const c_char,
) -> *mut c_char {
    unsafe { translate_in_context(text, context, braillify::encode_to_braille_font_in_context) }
}

/// Frees a C string returned by this library.
///
/// # Safety
///
/// - `ptr` must be null or a pointer previously returned by a string-producing
///   function in this library.
/// - The pointer must not have already been freed.
/// - The pointer must not be used after this function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Frees a byte buffer returned by [`braillify_encode`].
///
/// # Safety
///
/// - `ptr` must be null or a pointer previously returned by
///   [`braillify_encode`].
/// - `len` must be the exact length written to `out_len` by
///   [`braillify_encode`].
/// - The buffer must not have already been freed.
/// - The pointer must not be used after this function returns.
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
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn string_into_c_ptr_converts_valid_string() {
        let result = string_into_c_ptr("braille".to_string());

        assert!(!result.is_null());
        let c_string = unsafe { CString::from_raw(result) };
        assert_eq!(c_string.to_str().unwrap(), "braille");
    }

    #[test]
    fn string_into_c_ptr_rejects_interior_null() {
        clear_last_error();

        let result = string_into_c_ptr("braille\0output".to_string());

        assert!(result.is_null());
        let error = braillify_get_last_error();
        assert!(!error.is_null());
        let error = unsafe { CString::from_raw(error) };
        assert!(error.to_str().unwrap().contains("CString conversion error"));
    }

    #[test]
    fn test_encode_to_unicode() {
        let input = CString::new("안녕하세요").unwrap();
        let result = unsafe { braillify_encode_to_unicode(input.as_ptr()) };

        assert!(!result.is_null());

        let c_str = unsafe { CString::from_raw(result) };
        assert_eq!(c_str.to_str().unwrap(), "⠣⠒⠉⠻⠚⠠⠝⠬");
    }

    #[test]
    fn test_encode_to_unicode_empty() {
        let input = CString::new("").unwrap();
        let result = unsafe { braillify_encode_to_unicode(input.as_ptr()) };

        assert!(!result.is_null());

        let c_str = unsafe { CString::from_raw(result) };
        assert_eq!(c_str.to_str().unwrap(), "");
    }

    #[test]
    fn test_encode_to_unicode_null() {
        let result = unsafe { braillify_encode_to_unicode(ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_encode_to_braille_font() {
        let input = CString::new("안녕하세요").unwrap();
        let result = unsafe { braillify_encode_to_braille_font(input.as_ptr()) };

        assert!(!result.is_null());

        let c_str = unsafe { CString::from_raw(result) };
        assert_eq!(c_str.to_str().unwrap(), "⠣⠒⠉⠻⠚⠠⠝⠬");
    }

    #[test]
    fn test_encode_to_braille_font_null() {
        let result = unsafe { braillify_encode_to_braille_font(ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_encode() {
        let input = CString::new("안녕").unwrap();
        let mut out_len: usize = 0;

        let result = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };

        assert!(!result.is_null());
        assert!(out_len > 0);

        unsafe {
            braillify_free_bytes(result, out_len);
        }
    }

    #[test]
    fn test_encode_null_text() {
        let mut out_len: usize = 0;
        let result = unsafe { braillify_encode(ptr::null(), &mut out_len) };

        assert!(result.is_null());
    }

    #[test]
    fn test_encode_null_out_len() {
        let input = CString::new("test").unwrap();
        let result = unsafe { braillify_encode(input.as_ptr(), ptr::null_mut()) };

        assert!(result.is_null());
    }

    #[test]
    fn test_get_last_error_after_null() {
        let _ = unsafe { braillify_encode_to_unicode(ptr::null()) };
        let err = braillify_get_last_error();

        assert!(!err.is_null());

        let err_str = unsafe { CString::from_raw(err) };
        assert!(err_str.to_str().unwrap().contains("Null pointer"));
    }

    #[test]
    fn test_free_string_null() {
        unsafe {
            braillify_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn test_free_bytes_null() {
        unsafe {
            braillify_free_bytes(ptr::null_mut(), 0);
        }
    }

    #[test]
    fn test_encode_invalid_utf8() {
        let input = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE]) };
        let mut out_len: usize = 0;

        let result = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };

        assert!(result.is_null());
    }

    #[test]
    fn test_encode_to_unicode_invalid_utf8() {
        let input = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE]) };
        let result = unsafe { braillify_encode_to_unicode(input.as_ptr()) };

        assert!(result.is_null());
    }

    #[test]
    fn test_encode_to_braille_font_invalid_utf8() {
        let input = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE]) };
        let result = unsafe { braillify_encode_to_braille_font(input.as_ptr()) };

        assert!(result.is_null());
    }

    #[test]
    fn test_free_string_non_null() {
        let string = CString::new("test").unwrap();
        let ptr = string.into_raw();

        unsafe {
            braillify_free_string(ptr);
        }
    }

    #[test]
    fn test_get_last_error_none() {
        let input = CString::new("a").unwrap();
        let result = unsafe { braillify_encode_to_unicode(input.as_ptr()) };

        assert!(!result.is_null());

        unsafe {
            braillify_free_string(result);
        }

        let err = braillify_get_last_error();
        assert!(err.is_null());
    }

    fn take_string(value: *mut c_char) -> String {
        assert!(!value.is_null());
        let owned = unsafe { CString::from_raw(value) };
        owned.to_str().unwrap().to_owned()
    }

    #[test]
    fn test_context_functions_read_the_named_context() {
        let text = CString::new("pOH").unwrap();
        let context = CString::new("science").unwrap();
        let unicode = take_string(unsafe {
            braillify_encode_to_unicode_in_context(text.as_ptr(), context.as_ptr())
        });
        let font = take_string(unsafe {
            braillify_encode_to_braille_font_in_context(text.as_ptr(), context.as_ptr())
        });
        assert_eq!(unicode, "⠴⠏⠠⠕⠠⠓");
        assert_eq!(font, unicode);

        let mut out_len: usize = 0;
        let bytes =
            unsafe { braillify_encode_in_context(text.as_ptr(), context.as_ptr(), &mut out_len) };
        assert!(!bytes.is_null());
        assert_eq!(out_len, 6);
        unsafe { braillify_free_bytes(bytes, out_len) };
    }

    #[test]
    fn test_context_functions_reject_bad_input() {
        let text = CString::new("pOH").unwrap();
        let unknown = CString::new("chemistry").unwrap();
        let invalid = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE]) };

        let result =
            unsafe { braillify_encode_to_unicode_in_context(text.as_ptr(), unknown.as_ptr()) };
        assert!(result.is_null());
        assert_eq!(
            take_string(braillify_get_last_error()),
            "unknown context: chemistry"
        );

        let result =
            unsafe { braillify_encode_to_braille_font_in_context(text.as_ptr(), ptr::null()) };
        assert!(result.is_null());
        assert_eq!(
            take_string(braillify_get_last_error()),
            "Null pointer argument"
        );

        let result =
            unsafe { braillify_encode_to_unicode_in_context(invalid.as_ptr(), unknown.as_ptr()) };
        assert!(result.is_null());
        assert!(take_string(braillify_get_last_error()).starts_with("Invalid UTF-8"));

        let result = unsafe {
            braillify_encode_in_context(text.as_ptr(), unknown.as_ptr(), ptr::null_mut())
        };
        assert!(result.is_null());
        let mut out_len: usize = 0;
        let result =
            unsafe { braillify_encode_in_context(text.as_ptr(), unknown.as_ptr(), &mut out_len) };
        assert!(result.is_null());
        assert_eq!(
            take_string(braillify_get_last_error()),
            "unknown context: chemistry"
        );
    }

    #[test]
    fn test_encode_invalid_char() {
        let input = CString::new("😀").unwrap();
        let mut out_len: usize = 0;

        let result = unsafe { braillify_encode(input.as_ptr(), &mut out_len) };

        assert!(result.is_null());
    }

    #[test]
    fn test_encode_to_unicode_invalid_char() {
        let input = CString::new("😀").unwrap();
        let result = unsafe { braillify_encode_to_unicode(input.as_ptr()) };

        assert!(result.is_null());
    }

    #[test]
    fn test_encode_to_braille_font_invalid_char() {
        let input = CString::new("😀").unwrap();
        let result = unsafe { braillify_encode_to_braille_font(input.as_ptr()) };

        assert!(result.is_null());
    }
}
