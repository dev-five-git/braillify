use jni::objects::{JClass, JString};
use jni::sys::{jbyteArray, jstring};
use jni::JNIEnv;
use braillify as braillify_core;

fn throw_braillify_exception(env: &mut JNIEnv, msg: &str) {
    let _ = env.throw_new("com/devfive/BraillifyException", msg);
}

fn throw_runtime_exception(env: &mut JNIEnv, msg: &str) {
    let _ = env.throw_new("java/lang/RuntimeException", msg);
}

fn get_input_string(env: &mut JNIEnv, input: JString) -> Option<String> {
    match env.get_string(&input) {
        Ok(s) => Some(s.into()),
        Err(e) => {
            throw_runtime_exception(env, &format!("입력 문자열을 읽는 데 실패했습니다: {e}"));
            None
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_devfive_Braillify_encodeToUnicode(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let Some(input_str) = get_input_string(&mut env, input) else {
        return std::ptr::null_mut();
    };

    let mut out = String::with_capacity(input_str.len());

    for c in input_str.chars() {
        let char_str = c.to_string();
        match braillify_core::encode_to_unicode(&char_str) {
            Ok(encoded) => out.push_str(&encoded),
            Err(e) => {
                throw_braillify_exception(
                    &mut env,
                    &format!("점자 유니코드 변환 실패 (문자: '{c}'): {e}"),
                );
                return std::ptr::null_mut();
            }
        }
    }

    match env.new_string(out) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            throw_runtime_exception(&mut env, &format!("결과 문자열 생성 실패: {e}"));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_devfive_Braillify_encode(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jbyteArray {
    let Some(input_str) = get_input_string(&mut env, input) else {
        return std::ptr::null_mut();
    };

    let mut out: Vec<u8> = Vec::new();

    for c in input_str.chars() {
        let char_str = c.to_string();
        match braillify_core::encode(&char_str) {
            Ok(bytes) => out.extend_from_slice(&bytes),
            Err(e) => {
                throw_braillify_exception(
                    &mut env,
                    &format!("점자 바이트 변환 실패 (문자: '{c}'): {e}"),
                );
                return std::ptr::null_mut();
            }
        }
    }

    match env.byte_array_from_slice(&out) {
        Ok(arr) => arr.into_raw(),
        Err(e) => {
            throw_runtime_exception(&mut env, &format!("바이트 배열 생성 실패: {e}"));
            std::ptr::null_mut()
        }
    }
}
