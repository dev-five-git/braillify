// JNI entry points require a live JVM, so tarpaulin excludes this crate; the
// separate `jvm-test` job verifies the real JNI path.
#![cfg(not(tarpaulin_include))]

//! JNI bridge for the desktop/server JVM binding.
//!
//! The bridge intentionally calls the Rust core directly. It owns no encoder
//! state and never lets a Rust panic unwind across the JNI ABI boundary.

use std::ffi::c_void;
use std::ptr;
use std::slice;

use jni::errors::ErrorPolicy;
use jni::objects::{JByteArray, JClass, JString};
use jni::strings::JNIString;
use jni::sys::{JNI_VERSION_1_8, jbyteArray, jint, jstring};
use jni::{Env, EnvUnowned};

const PUBLIC_EXCEPTION: &str = "io/github/kdyann/braillify/BraillifyException";
const INTERNAL_EXCEPTION: &str = "io/github/kdyann/braillify/BraillifyInternalException";

#[derive(Debug)]
enum BridgeError {
    Core(String),
    InvalidUtf16,
    Jni(jni::errors::Error),
}

impl From<jni::errors::Error> for BridgeError {
    fn from(error: jni::errors::Error) -> Self {
        Self::Jni(error)
    }
}

fn decode_utf16_units(units: &[u16]) -> Result<String, BridgeError> {
    String::from_utf16(units).map_err(|_| BridgeError::InvalidUtf16)
}

/// Reads Java UTF-16 code units through `GetStringChars` and decodes them
/// strictly. `JNIEnv::get_string` uses modified UTF-8 semantics; reading code
/// units directly is what lets this binding reject isolated surrogates instead
/// of silently replacing them.
fn get_string_strict(env: &mut Env<'_>, input: &JString<'_>) -> Result<String, BridgeError> {
    if input.is_null() {
        return Err(BridgeError::Jni(jni::errors::Error::NullPtr(
            "Braillify input",
        )));
    }

    let raw_env = env.get_raw();
    let raw_string = input.as_raw();
    // SAFETY: `raw_env` and `raw_string` were supplied by the JVM for this
    // native call. JNI guarantees the function table for the duration of it.
    let length = unsafe { ((**raw_env).v1_1.GetStringLength)(raw_env, raw_string) };
    // SAFETY: same JNI-owned values as above. A null result means a pending JVM
    // exception (normally OutOfMemoryError), which the caller preserves.
    let chars = unsafe { ((**raw_env).v1_1.GetStringChars)(raw_env, raw_string, ptr::null_mut()) };
    if chars.is_null() {
        return Err(BridgeError::Jni(jni::errors::Error::NullPtr(
            "GetStringChars result",
        )));
    }

    // Copy before releasing the JVM buffer, then strictly decode the copy.
    // SAFETY: JNI returned `chars` with exactly `length` UTF-16 code units.
    let units = unsafe { slice::from_raw_parts(chars.cast::<u16>(), length as usize) }.to_vec();
    // SAFETY: `chars` was obtained above from `GetStringChars` for `raw_string`
    // and has not previously been released.
    unsafe { ((**raw_env).v1_1.ReleaseStringChars)(raw_env, raw_string, chars) };
    decode_utf16_units(&units)
}

fn throw_if_possible(env: &mut Env<'_>, class: &str, message: &str) {
    // Pending VM exceptions (notably OutOfMemoryError) must be preserved.
    if env.exception_check() {
        return;
    }
    let _ = env.throw_new(JNIString::new(class), JNIString::new(message));
}

fn report_error(env: &mut Env<'_>, error: BridgeError) {
    match error {
        BridgeError::Core(message) => throw_if_possible(env, PUBLIC_EXCEPTION, &message),
        BridgeError::InvalidUtf16 => throw_if_possible(
            env,
            PUBLIC_EXCEPTION,
            "Input contains malformed UTF-16 (an unpaired surrogate)",
        ),
        BridgeError::Jni(jni::errors::Error::NullPtr("Braillify input")) => throw_if_possible(
            env,
            "java/lang/NullPointerException",
            "text must not be null",
        ),
        BridgeError::Jni(_) => throw_if_possible(
            env,
            INTERNAL_EXCEPTION,
            "The JVM native bridge could not complete the conversion",
        ),
    }
}

fn report_panic(env: &mut Env<'_>) {
    throw_if_possible(
        env,
        INTERNAL_EXCEPTION,
        "The native braillify engine failed unexpectedly",
    );
}

fn encode_impl(env: &mut Env<'_>, input: &JString<'_>) -> Result<jbyteArray, BridgeError> {
    let text = get_string_strict(env, input)?;
    let encoded = braillify::encode(&text).map_err(BridgeError::Core)?;
    let array: JByteArray<'_> = env.byte_array_from_slice(&encoded)?;
    Ok(array.into_raw())
}

fn translate_impl(
    env: &mut Env<'_>,
    input: &JString<'_>,
    translate: fn(&str) -> Result<String, String>,
) -> Result<jstring, BridgeError> {
    let text = get_string_strict(env, input)?;
    let translated = translate(&text).map_err(BridgeError::Core)?;
    Ok(env.new_string(translated)?.into_raw())
}

struct BridgePolicy;

impl<T: Default> ErrorPolicy<T, BridgeError> for BridgePolicy {
    type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

    fn on_error<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut Self::Captures<'unowned_env_local, 'native_method>,
        error: BridgeError,
    ) -> jni::errors::Result<T> {
        report_error(env, error);
        Ok(T::default())
    }

    fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut Self::Captures<'unowned_env_local, 'native_method>,
        _payload: Box<dyn std::any::Any + Send + 'static>,
    ) -> jni::errors::Result<T> {
        report_panic(env);
        Ok(T::default())
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kdyann_braillify_Braillify_encodeNative<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> jbyteArray {
    unowned_env
        .with_env(|env| encode_impl(env, &input))
        .resolve::<BridgePolicy>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kdyann_braillify_Braillify_translateToUnicodeNative<
    'local,
>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> jstring {
    unowned_env
        .with_env(|env| translate_impl(env, &input, braillify::encode_to_unicode))
        .resolve::<BridgePolicy>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kdyann_braillify_Braillify_translateToBrailleFontNative<
    'local,
>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> jstring {
    unowned_env
        .with_env(|env| translate_impl(env, &input, braillify::encode_to_braille_font))
        .resolve::<BridgePolicy>()
}

/// Test-only JNI entry point used to prove that a panic is mapped to the
/// internal exception class. It is excluded from release artifacts.
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kdyann_braillify_Braillify_panicForTestingNative<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    unowned_env
        .with_env(|_env| -> Result<(), BridgeError> { panic!("intentional JNI test panic") })
        .resolve::<BridgePolicy>();
}

#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(_vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint {
    JNI_VERSION_1_8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::isolated_high(&[0xD800])]
    #[case::isolated_low(&[0xDC00])]
    #[case::reversed_pair(&[0xDC00, 0xD800])]
    fn malformed_utf16_is_rejected(#[case] units: &[u16]) {
        assert!(matches!(
            decode_utf16_units(units),
            Err(BridgeError::InvalidUtf16)
        ));
    }

    #[test]
    fn supplementary_pair_is_preserved() {
        assert_eq!(decode_utf16_units(&[0xD83D, 0xDE00]).unwrap(), "😀");
    }

    #[test]
    fn embedded_nul_is_preserved_by_utf16_decoder() {
        assert_eq!(
            decode_utf16_units(&[b'a' as u16, 0, b'b' as u16]).unwrap(),
            "a\0b"
        );
    }

    #[rstest::rstest]
    #[case::bytes("안녕")]
    #[case::unicode("hello")]
    #[case::braille_font("점자")]
    fn core_delegations_accept_supported_text(#[case] input: &str) {
        assert!(braillify::encode(input).is_ok());
        assert!(braillify::encode_to_unicode(input).is_ok());
        assert!(braillify::encode_to_braille_font(input).is_ok());
    }

    #[test]
    fn core_error_remains_an_error_at_bridge_boundary() {
        assert!(matches!(
            braillify::encode("😀").map_err(BridgeError::Core),
            Err(BridgeError::Core(_))
        ));
    }
}
