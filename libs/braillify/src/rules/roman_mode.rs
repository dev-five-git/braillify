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

/// 세 플래그가 만들 수 있는 조합 중 뜻이 있는 것만 허용한다.
///
/// 연속표 예약(`needs_english_continuation`)은 "구간을 닫았으니 다음 로마자는 ⠰로
/// 잇는다"는 뜻이고, 제35항 숫자 다리(`roman_number_chain`)는 "구간은 이어지지만
/// 지금은 숫자를 적는 중"이라는 뜻이다. 둘 다 구간이 닫힌 동안에만 성립하므로,
/// 구간이 열린 채로 어느 하나가 서 있으면 상태가 어긋난 것이다. 이 조합이 깨진 채
/// 진행하면 다음 로마자 어절에서 로마자표가 잘못 붙거나 빠진다.
fn debug_assert_consistent(state: &EncoderState) {
    debug_assert!(
        !(state.is_english && state.needs_english_continuation),
        "구간이 열린 채로 연속표를 예약할 수 없다"
    );
    debug_assert!(
        !(state.is_english && state.roman_number_chain),
        "구간이 열린 채로 숫자 다리가 설 수 없다"
    );
}

/// 영어 모드를 종료한다 (종료표 ⠲는 호출자가 emit; 여기선 상태만 전환).
pub(crate) fn exit_english(state: &mut EncoderState, needs_continuation: bool) {
    state.is_english = false;
    state.needs_english_continuation = needs_continuation;
    state.roman_number_chain = false;
    if !needs_continuation {
        state.roman_section_is_english_context = false;
    }
    debug_assert_consistent(state);
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
    debug_assert_consistent(state);
}

/// 제35항 — 로마자+숫자 연결(`D-100` 등)을 위해 영어 모드를 잠시 내려놓는다.
pub(crate) fn exit_english_for_roman_number_chain(state: &mut EncoderState) {
    state.is_english = false;
    state.needs_english_continuation = false;
    state.roman_number_chain = true;
    debug_assert_consistent(state);
}

/// 숫자 뒤 로마자가 다시 이어질 때 영어 모드를 (표지 없이) 재개한다.
pub(crate) fn resume_english_from_roman_number_chain(state: &mut EncoderState) {
    state.is_english = true;
    state.needs_english_continuation = false;
    state.roman_number_chain = false;
    debug_assert_consistent(state);
}

/// 표지를 적지 않고 로마자 구간을 연 상태로 만든다. 로마자표 ⠴ 를 호출자가 이미
/// 적었거나(제28항 문자 단계) 제39항이 생략하는 자리에서 쓴다.
pub(crate) fn mark_section_open(state: &mut EncoderState) {
    state.is_english = true;
    state.needs_english_continuation = false;
    state.roman_number_chain = false;
    debug_assert_consistent(state);
}

/// 제35항 숫자 다리 상태는 구간을 닫을 때만 남긴다. 구간을 열면 다리는 이미 로마자로
/// 이어진 것이므로 해소한다.
pub(crate) fn set_section_open_keeping_number_chain(state: &mut EncoderState, open: bool) {
    state.is_english = open;
    state.needs_english_continuation = false;
    if open {
        state.roman_number_chain = false;
    }
    debug_assert_consistent(state);
}

/// 다음 진입에서 연속표 ⠰ 대신 로마자표 ⠴ 를 적도록 예약을 지운다.
pub(crate) fn clear_pending_continuation(state: &mut EncoderState) {
    state.needs_english_continuation = false;
    debug_assert_consistent(state);
}

/// 제35항 숫자 다리를 해소한다.
pub(crate) fn clear_number_chain(state: &mut EncoderState) {
    state.roman_number_chain = false;
    debug_assert_consistent(state);
}

/// 로마자 구간을 닫되 제35항 숫자 다리 상태는 그대로 둔다. 단위 표기 뒤에 숫자가
/// 이어질 수 있는 자리에서 쓴다.
pub(crate) fn close_section_keeping_number_chain(state: &mut EncoderState) {
    state.is_english = false;
    state.needs_english_continuation = false;
    debug_assert_consistent(state);
}

/// 제69항 단위 표기 뒤 — 로마자가 이어지면 구간을 열어 두면서 제35항 숫자 다리를
/// 해소하고, 이어지지 않으면 구간만 닫고 숫자 다리는 남겨 둔다.
pub(crate) fn resolve_section_after_unit(state: &mut EncoderState, continues: bool) {
    state.is_english = continues;
    state.needs_english_continuation = false;
    if continues {
        state.roman_number_chain = false;
    }
    debug_assert_consistent(state);
}

/// 뒤에 로마자가 이어지는지에 따라 구간을 열어 두거나 닫는다 (제35항 단위 표기
/// 뒤처럼 한 자리에서 두 결과가 갈릴 때).
pub(crate) fn set_section_open(state: &mut EncoderState, open: bool) {
    state.is_english = open;
    state.needs_english_continuation = false;
    state.roman_number_chain = false;
    debug_assert_consistent(state);
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
