//! 제29항 — 국어 문장 안에 로마자가 나올 때에는 그 앞에 로마자표 ⠴(52)을 적고
//! 그 뒤에 로마자 종료표 ⠲(50)을 적는다.
//!
//! 제31항 — 국어 문장 안에 그리스 문자가 나올 때에도 로마자표와 종료표를 적는다.
//! 제33항 — 문장 부호의 점형이 다른 경우 종료표를 생략하는 규칙.
//! 제35항 — 로마자와 숫자가 이어 나올 때에는 종료표를 적지 않는다.
//!
//! 로마자표/종료표/연속표 emit 오케스트레이션은 문자 단위 엔진
//! ([`crate::rules::emit`])이 직접 수행한다 — 영어 진입/이탈 상태 기계, 괄호 스택,
//! 숫자 연결(제35항), 제39항 한글 wrap이 모두 거기서 얽혀 있기 때문이다. 이 모듈은
//! 그 점형 바이트 상수만 노출한다.
//!
//! (과거 `Rule29`는 `Phase::ModeManagement` `BrailleRule`이었으나, 프로덕션 emit
//! 경로는 `CoreEncoding`/`InterCharacter` 단계만 적용하므로 한 번도 실행되지 않는
//! 죽은 코드였다 — 로마자표는 `emit`/제28항이 emit한다 — 그래서 제거했다.)
//!
//! Reference: 2024 Korean Braille Standard, Chapter 4, Section 10, Articles 29, 31, 33, 35

/// Roman letter indicator (로마자표) ⠴.
pub const ROMAN_INDICATOR: u8 = 52;

/// English continuation indicator (연속표) ⠰.
pub const ENGLISH_CONTINUATION: u8 = 48;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_values() {
        assert_eq!(ROMAN_INDICATOR, 52);
        assert_eq!(ENGLISH_CONTINUATION, 48);
    }

    /// 제29항 — 국어 문장 안 로마자 앞에 로마자표 ⠴가 emit되는지 (emit 경로 통합).
    #[test]
    fn roman_indicator_emitted_in_korean() {
        let result = crate::encode_to_unicode("그는 Canada로").unwrap();
        assert!(result.contains('⠴'), "should contain roman indicator ⠴");
    }

    /// 제29항/제35항 — 숫자 어절(`1,234`) 뒤 로마자 단위(`km`)에서도 로마자표가
    /// emit되는지. `crate::encode` 통합으로 검증해 helper 내부에 의존하지 않는다.
    #[test]
    fn numeric_prev_word_drives_roman_indicator() {
        let out = crate::encode("1,234 km는").expect("must encode");
        assert!(out.contains(&ROMAN_INDICATOR));
    }
}
