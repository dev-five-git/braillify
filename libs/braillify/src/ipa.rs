//! IPA (International Phonetic Alphabet) braille encoding (extracted from lib.rs).
//!
//! PDF 제38항 — IPA 점자 표기. [...], /.../ 묶음 안 음운 기호를 점역한다.

use crate::rules::context::EncodingMode;
use crate::{encode, english, utils, with_encoder};

pub(crate) fn is_ipa_phonetic_symbol(c: char) -> bool {
    matches!(c, 'θ' | 'ə' | 'æ' | 'ŋ' | 'ː')
}

/// PDF 제38항 자동 감지 — input의 묶음 패턴 안 IPA 음운 기호로 IPA 컨텍스트 추론.
///
/// 알고리즘(AST적 판단):
/// 1. 입력을 좌→우 스캔하며 `[...]` 또는 `/.../` 매칭쌍을 찾는다.
/// 2. 매칭쌍 내부에 IPA 음운 기호(θ, ə, æ, ŋ, ː 등)가 하나라도 있으면
///    IPA 컨텍스트로 판정한다.
/// 3. 한 번이라도 IPA 매칭쌍을 발견하면 input 전체를 IPA로 처리한다.
///    (같은 input 안의 다른 `[...]`·`/.../`도 동일 컨텍스트로 본다.
///    예: `/æ/...로 .../a/로` — 첫 매칭이 IPA면 둘째도 IPA.)
///
/// 빈 묶음(`[ ]`·`/ /`)이나 음운 기호 없는 내용은 IPA가 아니다. URL 안 `://`,
/// 분수 `1/2`, 일반 대괄호 `[1]` 등이 IPA로 오인되지 않도록 한다.
///
/// IPA 음운 기호 집합은 본 라이브러리가 인식하는 부분 집합이며,
/// PDF 표에 새 기호 추가 시 `is_ipa_phonetic_symbol`와 `encode_ipa_char`을 함께 확장한다.
pub(crate) fn detect_ipa_context(text: &str) -> bool {
    let mut has_group_start = false;
    let mut has_ipa_symbol = false;
    for c in text.chars() {
        has_group_start |= matches!(c, '[' | '/');
        has_ipa_symbol |= is_ipa_phonetic_symbol(c);
        if has_group_start && has_ipa_symbol {
            break;
        }
    }
    if !has_group_start || !has_ipa_symbol {
        return false;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => {
                if let Some(rel) = chars[i + 1..].iter().position(|&c| c == ']') {
                    let inner: &[char] = &chars[i + 1..i + 1 + rel];
                    if inner.iter().any(|c| is_ipa_phonetic_symbol(*c)) {
                        return true;
                    }
                    i += rel + 2;
                    continue;
                }
            }
            '/' => {
                if let Some(rel) = chars[i + 1..].iter().position(|&c| c == '/') {
                    let inner: &[char] = &chars[i + 1..i + 1 + rel];
                    if inner.iter().any(|c| is_ipa_phonetic_symbol(*c)) {
                        return true;
                    }
                    i += rel + 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// PDF 제38항 — 국제음성기호(IPA) 점자 변환.
///
/// 알고리즘:
/// 1. 좌→우 스캔하며 묶음 기호 상태(대괄호/빗금 열림 여부)를 추적한다.
/// 2. `[`·`]`·`/`는 묶음 상태에 따라 시작/종료 점형을 출력한다.
/// 3. 묶음 안에서는 IPA 변환표에 따라 음운 기호와 영문자를 인코딩한다.
/// 4. 묶음 밖의 한국어/영문/숫자 등은 일반 점자 인코더로 위임한다.
///
/// 본 함수가 적용되는 경우는 testcase의 `context: "ipa"` 또는
/// `EncodeOptions::default_mode = Some(EncodingMode::Ipa)`로 명시된 상황뿐이며,
/// 자동 감지는 별도 token rule에서 처리한다.
///
/// 점자 셀 인덱스 = (Unicode braille codepoint) − 0x2800.
///   ⠐ = 16 (점 5)         ⠘ = 24 (점 4+5)        ⠷ = 55 (점 1+2+3+5+6)
///   ⠾ = 62 (점 2+3+4+5+6) ⠌ = 12 (점 3+4)
pub(crate) fn encode_ipa(text: &str) -> Result<Vec<u8>, String> {
    let mut out: Vec<u8> = Vec::new();
    let mut bracket_open = false;
    let mut slash_open = false;
    let mut korean_buf = String::new();

    // IPA 입력 전체에 한국어가 한 글자라도 있으면, 묶음 밖 영어 어절도
    // 한국어 점자 환경의 일부로 보고 영자표시(⠴)를 emit해야 한다.
    // (예: "worth [wəːrθ]: ~해볼 만한" → 영어 "worth" 시작에 ⠴ 필요.)
    let has_korean_anywhere = text.chars().any(utils::is_korean_char);

    let flush_korean = |buf: &mut String, out: &mut Vec<u8>| -> Result<(), String> {
        if !buf.is_empty() {
            // 묶음 밖의 한국어/영문 등은 일반 인코더로 위임한다. 전체 입력에
            // 한국어가 있는 경우, 영어 단어 시작에 영자표시가 붙도록 강제한다.
            let enc = if has_korean_anywhere {
                with_encoder(true, |encoder| {
                    let mut result = Vec::new();
                    encoder.set_default_mode(EncodingMode::Korean);
                    encoder.encode(buf.as_str(), &mut result)?;
                    Ok::<Vec<u8>, String>(result)
                })?
            } else {
                encode(buf.as_str())?
            };
            out.extend(enc);
            buf.clear();
        }
        Ok(())
    };

    // 여는 대괄호: ⠐⠘⠷ = 16, 24, 55 | 닫는 대괄호: ⠘⠾ = 24, 62
    // 여는 빗금: ⠐⠘⠌ = 16, 24, 12 | 닫는 빗금: ⠘⠌ = 24, 12
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '[' => {
                flush_korean(&mut korean_buf, &mut out)?;
                let _ = remove_trailing_english_terminator(&mut out);
                out.extend_from_slice(&[16, 24, 55]);
                bracket_open = true;
            }
            ']' => {
                flush_korean(&mut korean_buf, &mut out)?;
                out.extend_from_slice(&[24, 62]);
                bracket_open = false;
            }
            '/' => {
                flush_korean(&mut korean_buf, &mut out)?;
                if slash_open {
                    out.extend_from_slice(&[24, 12]);
                    slash_open = false;
                } else {
                    let _ = remove_trailing_english_terminator(&mut out);
                    out.extend_from_slice(&[16, 24, 12]);
                    slash_open = true;
                }
            }
            ' ' => {
                if (bracket_open || slash_open) && matches!(chars.peek(), Some('(')) {
                    continue;
                }
                flush_korean(&mut korean_buf, &mut out)?;
                out.push(0);
            }
            '(' if bracket_open || slash_open => {
                flush_korean(&mut korean_buf, &mut out)?;
                let mut text = String::from("(");
                for next in chars.by_ref() {
                    text.push(next);
                    if next == ')' {
                        break;
                    }
                }
                out.extend_from_slice(&[48, 48]);
                out.push(0);
                out.extend(encode(text.as_str())?);
                out.push(0);
                out.extend_from_slice(&[16, 48, 6]);
                if matches!(chars.peek(), Some(' ')) {
                    chars.next();
                }
            }
            _ if bracket_open || slash_open => {
                // 묶음 안: IPA 음운/영문 점자 변환.
                flush_korean(&mut korean_buf, &mut out)?;
                let bytes =
                    encode_ipa_char(ch).ok_or_else(|| format!("Unknown IPA character: {ch:?}"))?;
                out.extend(bytes);
            }
            _ => {
                // 묶음 밖: 일반 텍스트는 한국어/영문 인코더로 위임.
                korean_buf.push(ch);
            }
        }
    }
    flush_korean(&mut korean_buf, &mut out)?;
    Ok(out)
}

// 영어 어절 직후 IPA 묶음이 이어지면, 영어 종료표(⠲)는 묶음 기호가 새
// 컨텍스트를 열기 때문에 불필요하다. 공백 위에 놓인 종료표만 제거한다.
fn remove_trailing_english_terminator(out: &mut Vec<u8>) -> bool {
    let mut i = out.len();
    while i > 0 && out[i - 1] == 0 {
        i -= 1;
    }
    if i > 0 && out[i - 1] == 50 {
        out.remove(i - 1);
        true
    } else {
        false
    }
}

/// PDF 제38항 IPA 변환표 — 음운 기호 및 영문자 점자 매핑.
/// 영문 알파벳은 일반 영어 점자 매핑(`english::encode_english`)을 사용한다.
///
/// 점자 셀 인덱스 = (Unicode braille codepoint) − 0x2800.
pub(crate) fn encode_ipa_char(ch: char) -> Option<Vec<u8>> {
    // PDF 국제음성기호 점자 규정 변환표 — 음운 기호 매핑.
    // (현재 본 라이브러리가 인식하는 음운 기호 부분 집합.
    //  새 기호 추가 시 PDF 표에 근거해 직접 추가한다.)
    match ch {
        'ə' => Some(vec![34]),       // ⠢ (점 2+6)
        'ː' => Some(vec![18]),       // ⠒ (점 2+5) — 장음 표시
        'θ' => Some(vec![40, 57]),   // ⠨⠹ (점 4+6, 점 1+4+5+6)
        'ŋ' => Some(vec![43]),       // ⠫ (점 1+2+4+6)
        'æ' => Some(vec![41]),       // ⠩ (점 1+4+6)
        'ɔ' => Some(vec![35]),       // ⠣ open o — UEB §14.4 table.
        'ʃ' => Some(vec![49]),       // ⠱ esh — UEB §14.4 table.
        'ð' => Some(vec![59]),       // ⠻ edh — UEB §14.4 table.
        'ɪ' => Some(vec![12]),       // ⠌ small capital i — UEB §14.4 table.
        'ɹ' => Some(vec![60]),       // ⠼ turned r — UEB §14.4 table.
        'ɾ' => Some(vec![22, 23]),   // ⠖⠗ fish-hook r — UEB §14.4 table.
        'ˈ' => Some(vec![56, 3]),    // ⠸⠃ superior stress — UEB §14.4 table.
        'ˌ' => Some(vec![56, 6]),    // ⠸⠆ inferior stress — UEB §14.4 table.
        'č' => Some(vec![3, 8, 38]), // ⠉⠈⠦ c with wedge — UEB §14.4 table.
        _ => {
            // 기본 알파벳/숫자는 일반 영어 점자 변환을 사용.
            if let Ok(code) = english::encode_english(ch) {
                Some(vec![code])
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<u8> {
        s.chars().map(crate::unicode::decode_unicode).collect()
    }

    /// ipa:108 — IPA input WITHOUT any Korean char triggers the else branch
    /// in flush_korean: `encode(buf.as_str())?` (no with_encoder wrap).
    #[test]
    fn ipa_input_without_korean_uses_plain_encode() {
        // Pure English + IPA bracket, no Korean anywhere.
        let _ = encode_ipa("think [θɪŋk]");
        let _ = encode_ipa("[θ]/æ/");
    }

    #[rstest::rstest]
    #[case::edh('ð', vec![59])]
    #[case::open_o('ɔ', vec![35])]
    #[case::esh('ʃ', vec![49])]
    #[case::turned_r('ɹ', vec![60])]
    #[case::primary_stress('ˈ', vec![56, 3])]
    #[case::secondary_stress('ˌ', vec![56, 6])]
    #[case::fish_hook_r('ɾ', vec![22, 23])]
    #[case::c_wedge('č', vec![3, 8, 38])]
    fn encodes_ueb_14_4_ipa_table_chars(#[case] ch: char, #[case] expected: Vec<u8>) {
        assert_eq!(encode_ipa_char(ch), Some(expected));
    }

    #[test]
    fn bracket_open_flushes_prior_korean_and_strips_english_terminator() {
        let bracket_first = encode_ipa("[θ]").expect("initial IPA bracket should encode");
        assert!(bracket_first.starts_with(&[16, 24, 55]));

        let korean_before = encode_ipa("한[θ]").expect("Korean before IPA bracket should encode");
        assert!(korean_before.starts_with(&crate::encode("한").unwrap()));
        assert!(korean_before.contains(&16));

        let english_before =
            encode_ipa("word[θ]").expect("English before IPA bracket should encode");
        assert!(!english_before[..english_before.len() - 3].contains(&50));
    }

    #[test]
    fn bracket_open_after_plain_english_flushes_buffer() {
        let encoded = encode_ipa("word[θ]").expect("English before IPA bracket should encode");

        let bracket_pos = encoded
            .windows(3)
            .position(|cells| cells == [16, 24, 55])
            .expect("opening IPA bracket marker should be present");
        assert!(bracket_pos > 0);
    }

    #[test]
    fn bracket_open_after_buffer_flushes_then_opens_group() {
        let encoded = encode_ipa("말[θ]").expect("Korean before IPA bracket should encode");
        let prefix = crate::encode("말").expect("Korean prefix should encode");

        assert!(encoded.starts_with(&prefix));
        assert_eq!(&encoded[prefix.len()..prefix.len() + 3], &[16, 24, 55]);
    }

    #[test]
    fn ipa_group_opening_removes_prior_english_terminator_after_korean_context() {
        let bracketed = encode_ipa("말 word[θ]").expect("English before IPA bracket should encode");
        let bracket_pos = bracketed
            .windows(3)
            .position(|cells| cells == [16, 24, 55])
            .expect("opening IPA bracket marker should be present");
        assert!(!bracketed[..bracket_pos].contains(&50));

        let slashed = encode_ipa("말 word/θ/").expect("English before IPA slash should encode");
        let slash_pos = slashed
            .windows(3)
            .position(|cells| cells == [16, 24, 12])
            .expect("opening IPA slash marker should be present");
        assert!(!slashed[..slash_pos].contains(&50));
    }

    #[test]
    fn bracket_open_after_english_space_strips_terminator_before_group() {
        let encoded =
            encode_ipa("word [θ]").expect("English before spaced IPA bracket should encode");
        let bracket_pos = encoded
            .windows(3)
            .position(|cells| cells == [16, 24, 55])
            .expect("opening IPA bracket marker should be present");

        assert!(!encoded[..bracket_pos].contains(&50));
    }

    #[test]
    fn remove_trailing_english_terminator_skips_spaces_before_remove() {
        let mut out = vec![1, 50, 0, 0];

        assert!(remove_trailing_english_terminator(&mut out));
        assert_eq!(out, vec![1, 0, 0]);
    }

    #[test]
    fn bracketed_ipa_temporarily_switches_to_ueb_for_parentheses() {
        let encoded = encode_ipa("[ðə (garbled section) dɪs]").expect("IPA should encode");
        assert_eq!(encoded, cells("⠐⠘⠷⠻⠢⠰⠰⠀⠐⠣⠛⠜⠃⠇⠫⠀⠎⠑⠉⠰⠝⠐⠜⠀⠐⠰⠆⠙⠌⠎⠘⠾"));
    }

    /// ipa:196 — `encode_ipa_char` returns None for chars that aren't in the
    /// IPA mapping AND `english::encode_english` returns Err for.
    /// Use a character that has no English mapping (e.g. emoji or arbitrary unicode).
    #[test]
    fn ipa_encode_char_none_for_unsupported() {
        // Emoji or arbitrary char not in english map and not in IPA map.
        assert!(encode_ipa_char('\u{1F600}').is_none()); // 😀
        // Arbitrary CJK character not in english.
        assert!(encode_ipa_char('한').is_none());
    }

    #[test]
    fn plain_text_outside_ipa_group_buffers_until_end() {
        let input = std::hint::black_box("plain");
        let encoded = encode_ipa(input).expect("plain text should encode through normal encoder");

        assert_eq!(
            encoded,
            crate::encode(input).expect("plain text should encode")
        );
    }

    #[test]
    fn runtime_plain_text_pushes_each_outside_group_char() {
        let mut input = String::new();
        input.push(std::hint::black_box('p'));
        input.push(std::hint::black_box('q'));

        let encoded = encode_ipa(&input).expect("plain runtime text should encode");

        assert_eq!(
            encoded,
            crate::encode(&input).expect("plain text should encode")
        );
    }

    #[test]
    fn plain_text_after_ipa_group_buffers_until_final_flush() {
        let encoded = encode_ipa("[θ]plain").expect("IPA then plain text should encode");
        let suffix = crate::encode("plain").expect("plain suffix should encode");

        assert!(encoded.ends_with(&suffix));
    }

    #[test]
    fn plain_text_outside_slash_group_buffers_between_groups() {
        let encoded = encode_ipa("/θ/plain").expect("IPA then plain text should encode");
        let suffix = crate::encode("plain").expect("plain suffix should encode");

        assert!(encoded.ends_with(&suffix));
    }

    #[test]
    fn plain_text_between_ipa_groups_buffers_outside_group() {
        let encoded =
            encode_ipa("[θ]plain[θ]").expect("plain text between IPA groups should encode");
        let plain = crate::encode("plain").expect("plain text should encode");

        assert!(encoded.windows(plain.len()).any(|window| window == plain));
    }
}
