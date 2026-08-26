use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

const PANIC_ERROR: &str = "internal panic in braillify";

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_last_error(error: impl Into<String>) {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(error.into()));
}

fn clear_last_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

unsafe fn input_from_ptr<'a>(text: *const c_char) -> Result<&'a str, String> {
    if text.is_null() {
        return Err("text must not be NULL".to_owned());
    }

    // SAFETY: The exported functions document that `text` must point to a
    // readable NUL-terminated byte string for the duration of the call.
    let text = unsafe { CStr::from_ptr(text) };
    text.to_str()
        .map_err(|error| format!("text must be valid UTF-8: {error}"))
}

fn string_into_raw(value: String) -> Result<*mut c_char, String> {
    CString::new(value)
        .map(CString::into_raw)
        .map_err(|_| "encoded output unexpectedly contained a NUL byte".to_owned())
}

unsafe fn encode_string(
    text: *const c_char,
    encode: impl FnOnce(&str) -> Result<String, String>,
) -> *mut c_char {
    clear_last_error();
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: The caller of this helper provides the same pointer guarantee
        // as the exported string-encoding functions.
        unsafe { input_from_ptr(text) }
            .and_then(encode)
            .and_then(string_into_raw)
    }));

    match result {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            set_last_error(error);
            ptr::null_mut()
        }
        Err(_) => {
            set_last_error(PANIC_ERROR);
            ptr::null_mut()
        }
    }
}

unsafe fn encode_bytes(
    text: *const c_char,
    out_len: *mut usize,
    encode: impl FnOnce(&str) -> Result<Vec<u8>, String>,
) -> *mut u8 {
    clear_last_error();
    if out_len.is_null() {
        set_last_error("out_len must not be NULL");
        return ptr::null_mut();
    }
    // SAFETY: `out_len` was checked above and is required by this function's
    // contract to point to writable memory.
    unsafe { *out_len = 0 };

    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: The caller of this helper provides the same pointer guarantee
        // as the exported byte-encoding function.
        unsafe { input_from_ptr(text) }.and_then(encode)
    }));
    match result {
        Ok(Ok(bytes)) => {
            let len = bytes.len();
            let allocation = bytes.into_boxed_slice();
            // SAFETY: `out_len` satisfies the documented caller contract.
            unsafe { *out_len = len };
            Box::into_raw(allocation).cast::<u8>()
        }
        Ok(Err(error)) => {
            set_last_error(error);
            ptr::null_mut()
        }
        Err(_) => {
            set_last_error(PANIC_ERROR);
            ptr::null_mut()
        }
    }
}

/// Returns a newly allocated copy of the last error for the current thread.
///
/// The caller owns the returned string and must release it with
/// [`braillify_string_free`]. Returns `NULL` when no error has occurred.
#[unsafe(no_mangle)]
pub extern "C" fn braillify_last_error() -> *mut c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|error| CString::new(error.as_str()).ok())
            .map_or(ptr::null_mut(), CString::into_raw)
    })
}

/// Encodes UTF-8 text into Braille cell bytes.
///
/// On success, returns an allocation owned by the caller and writes its size
/// to `out_len`. Release it with [`braillify_bytes_free`] using the same size.
/// On failure, returns `NULL`, writes zero to `out_len`, and records an error.
///
/// # Safety
///
/// `text` must point to a readable NUL-terminated byte string for the duration
/// of the call. `out_len` must point to writable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode(text: *const c_char, out_len: *mut usize) -> *mut u8 {
    // SAFETY: Both pointers satisfy this exported function's caller contract.
    unsafe { encode_bytes(text, out_len, braillify::encode) }
}

/// Encodes UTF-8 text as a newly allocated UTF-8 Unicode Braille string.
///
/// Release the returned string with [`braillify_string_free`].
///
/// # Safety
///
/// `text` must point to a readable NUL-terminated byte string for the duration
/// of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_unicode(text: *const c_char) -> *mut c_char {
    // SAFETY: `text` satisfies this exported function's caller contract.
    unsafe { encode_string(text, braillify::encode_to_unicode) }
}

/// Encodes UTF-8 text as a newly allocated NUL-terminated Braille-font string.
///
/// Release the returned string with [`braillify_string_free`].
///
/// # Safety
///
/// `text` must point to a readable NUL-terminated byte string for the duration
/// of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_encode_braille_font(text: *const c_char) -> *mut c_char {
    // SAFETY: `text` satisfies this exported function's caller contract.
    unsafe { encode_string(text, braillify::encode_to_braille_font) }
}

/// Releases a string returned by this library. Passing `NULL` is allowed.
///
/// # Safety
///
/// `value` must be `NULL` or a pointer returned by a string-returning function
/// in this library, and it must not have been released previously.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_string_free(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: The caller guarantees ownership of a compatible allocation.
        unsafe { drop(CString::from_raw(value)) };
    }
}

/// Releases bytes returned by [`braillify_encode`]. Passing `NULL` is allowed.
///
/// # Safety
///
/// `bytes` and `len` must be the exact pair returned by [`braillify_encode`],
/// and the allocation must not have been released previously.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn braillify_bytes_free(bytes: *mut u8, len: usize) {
    if !bytes.is_null() {
        let slice = ptr::slice_from_raw_parts_mut(bytes, len);
        // SAFETY: `slice` reconstructs the boxed slice returned by encode.
        unsafe { drop(Box::from_raw(slice)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(value: &str) -> CString {
        CString::new(value).expect("test input must not contain NUL")
    }

    fn take_error() -> Option<String> {
        let error = braillify_last_error();
        if error.is_null() {
            return None;
        }
        // SAFETY: The pointer came from `braillify_last_error` and remains
        // valid until it is released below.
        let message = unsafe { CStr::from_ptr(error) }
            .to_str()
            .expect("errors are valid UTF-8")
            .to_owned();
        // SAFETY: This function owns the returned error allocation.
        unsafe { braillify_string_free(error) };
        Some(message)
    }

    #[test]
    fn encode_returns_owned_bytes() {
        let input = input("안녕");
        let mut len = 0;
        // SAFETY: Both arguments are valid for this call.
        let bytes = unsafe { braillify_encode(input.as_ptr(), &mut len) };
        assert!(!bytes.is_null());
        assert!(len > 0);
        assert!(take_error().is_none());
        // SAFETY: This is the exact pointer/length pair returned above.
        unsafe { braillify_bytes_free(bytes, len) };
    }

    #[test]
    fn unicode_returns_owned_utf8_string() {
        let input = input("안녕");
        // SAFETY: `input` is a valid NUL-terminated string.
        let encoded = unsafe { braillify_encode_unicode(input.as_ptr()) };
        assert!(!encoded.is_null());
        // SAFETY: `encoded` is a valid string until released below.
        let result = unsafe { CStr::from_ptr(encoded) }.to_str().unwrap();
        assert!(!result.is_empty());
        assert!(
            result
                .chars()
                .all(|ch| ('\u{2800}'..='\u{28ff}').contains(&ch))
        );
        // SAFETY: This test owns the allocation.
        unsafe { braillify_string_free(encoded) };
    }

    #[test]
    fn braille_font_returns_owned_string() {
        let input = input("안녕");
        // SAFETY: `input` is a valid NUL-terminated string.
        let encoded = unsafe { braillify_encode_braille_font(input.as_ptr()) };
        assert!(!encoded.is_null());
        // SAFETY: This test owns the allocation.
        unsafe { braillify_string_free(encoded) };
    }

    #[test]
    fn null_text_records_an_error_and_zeroes_length() {
        let mut len = usize::MAX;
        // SAFETY: A null text pointer is an explicitly handled invalid input.
        let bytes = unsafe { braillify_encode(ptr::null(), &mut len) };
        assert!(bytes.is_null());
        assert_eq!(len, 0);
        assert_eq!(take_error().as_deref(), Some("text must not be NULL"));
    }

    #[test]
    fn null_length_records_an_error() {
        let input = input("안녕");
        // SAFETY: A null output pointer is an explicitly handled invalid input.
        let bytes = unsafe { braillify_encode(input.as_ptr(), ptr::null_mut()) };
        assert!(bytes.is_null());
        assert_eq!(take_error().as_deref(), Some("out_len must not be NULL"));
    }

    #[test]
    fn invalid_utf8_records_an_error() {
        let input = [0xff_u8, 0];
        // SAFETY: The buffer is readable and NUL-terminated; invalid UTF-8 is
        // handled by the binding.
        let encoded = unsafe { braillify_encode_unicode(input.as_ptr().cast()) };
        assert!(encoded.is_null());
        assert!(
            take_error()
                .unwrap()
                .starts_with("text must be valid UTF-8")
        );
    }

    #[test]
    fn engine_error_is_exposed_and_success_clears_it() {
        let unsupported = input("😀");
        // SAFETY: `unsupported` is a valid NUL-terminated string.
        let encoded = unsafe { braillify_encode_unicode(unsupported.as_ptr()) };
        assert!(encoded.is_null());
        assert!(take_error().is_some());

        let supported = input("안녕");
        // SAFETY: `supported` is a valid NUL-terminated string.
        let encoded = unsafe { braillify_encode_unicode(supported.as_ptr()) };
        assert!(!encoded.is_null());
        assert!(take_error().is_none());
        // SAFETY: This test owns the allocation.
        unsafe { braillify_string_free(encoded) };
    }

    #[test]
    fn free_functions_accept_null() {
        // SAFETY: Both free functions explicitly accept NULL.
        unsafe {
            braillify_string_free(ptr::null_mut());
            braillify_bytes_free(ptr::null_mut(), 0);
        }
    }

    #[test]
    fn byte_encoder_catches_panics_and_zeroes_length() {
        let input = input("안녕");
        let mut len = usize::MAX;
        // SAFETY: Both arguments are valid for this call. The injected encoder
        // deliberately panics to exercise the ABI boundary.
        let encoded = unsafe { encode_bytes(input.as_ptr(), &mut len, |_| panic!("test panic")) };
        assert!(encoded.is_null());
        assert_eq!(len, 0);
        assert_eq!(take_error().as_deref(), Some(PANIC_ERROR));
    }

    #[test]
    fn string_encoder_catches_panics() {
        let input = input("안녕");
        // SAFETY: `input` is valid. The injected encoder deliberately panics to
        // exercise the ABI boundary.
        let encoded = unsafe { encode_string(input.as_ptr(), |_| panic!("test panic")) };
        assert!(encoded.is_null());
        assert_eq!(take_error().as_deref(), Some(PANIC_ERROR));
    }
}
