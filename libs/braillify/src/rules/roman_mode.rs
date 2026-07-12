//! 제29항 로마자(영어) 모드 전환 오케스트레이션.
//!
//! 한글 문맥에서 영어 구간의 진입/이탈을 관리한다 — 로마자표 ⠴(52)·연속표 ⠰(48)
//! emit과 [`EncoderState`]의 영어 모드 플래그 전환. 영어 *점형*은 UEB 모듈
//! ([`crate::rules::english_ueb`])이 생산하고, 이 모듈은 그 점형을 한글 문장 안에
//! 끼워 넣는 "예외" 표지(로마자표/연속표)와 모드 전환만 담당한다. 종료표 ⠲(50)는
//! 호출자([`crate::rules::emit`])가 문맥(다음 어절/문장부호)을 보고 직접 emit한다.
//!
//! Reference: 2024 Korean Braille Standard, Ch.4 Sec.10 Art.28-39

use crate::rules::context::EncoderState;
use crate::rules::korean::rule_29::{ENGLISH_CONTINUATION, ROMAN_INDICATOR};

/// 영어 모드를 종료한다 (종료표 ⠲는 호출자가 emit; 여기선 상태만 전환).
pub(crate) fn exit_english(state: &mut EncoderState, needs_continuation: bool) {
    state.is_english = false;
    state.needs_english_continuation = needs_continuation;
    state.roman_number_chain = false;
}

/// 영어 모드로 진입하며 로마자표 ⠴ (또는 직전 종료 후 연속표 ⠰)를 emit한다.
pub(crate) fn enter_english(state: &mut EncoderState, result: &mut Vec<u8>) {
    if state.needs_english_continuation {
        result.push(ENGLISH_CONTINUATION);
    } else {
        result.push(ROMAN_INDICATOR);
    }
    state.is_english = true;
    state.needs_english_continuation = false;
    state.roman_number_chain = false;
}

/// 제35항 — 로마자+숫자 연결(`D-100` 등)을 위해 영어 모드를 잠시 내려놓는다.
pub(crate) fn exit_english_for_roman_number_chain(state: &mut EncoderState) {
    exit_english(state, false);
    state.roman_number_chain = true;
}

/// 숫자 뒤 로마자가 다시 이어질 때 영어 모드를 (표지 없이) 재개한다.
pub(crate) fn resume_english_from_roman_number_chain(state: &mut EncoderState) {
    state.is_english = true;
    state.needs_english_continuation = false;
    state.roman_number_chain = false;
}

/// 어절 시작에서 영어 letter로 진입할 때 로마자표/연속표를 emit하고 영어 모드를
/// 켠다 (제28/35/39항). 진입 조건이 아니면 아무 것도 하지 않는다.
pub(crate) fn enter_english_if_starting(
    state: &mut EncoderState,
    word_chars: &[char],
    has_ascii_alphabetic: bool,
    result: &mut Vec<u8>,
) {
    let starts_english = state.english_indicator
        && !state.is_english
        && has_ascii_alphabetic
        && word_chars.first().is_some_and(|c| c.is_ascii_alphabetic());
    if !starts_english {
        return;
    }
    if state.roman_number_chain {
        resume_english_from_roman_number_chain(state);
    } else if state.english_dominant_no_indicator {
        // 영어 주도 문서(제39항): 영자표시 ⠴ 생략, 상태만 영어 모드로 전환.
        state.is_english = true;
        state.needs_english_continuation = false;
        state.roman_number_chain = false;
    } else {
        enter_english(state, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_english_if_starting_emits_roman_indicator() {
        let mut state = EncoderState::new(true);
        let mut result = Vec::new();

        enter_english_if_starting(&mut state, &['a'], true, &mut result);

        assert_eq!(result, vec![ROMAN_INDICATOR]);
        assert!(state.is_english);
        assert!(!state.needs_english_continuation);
        assert!(!state.roman_number_chain);
    }
}
