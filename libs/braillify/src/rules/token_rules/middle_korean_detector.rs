use crate::rules::context::EncodingMode;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

/// 제19항의 옛 글자표 규정을 적용할 인코딩 모드를 고르기 위해 옛 글자 문맥을 감지한다.
pub struct MiddleKoreanDetectorRule;

fn is_strong_middle_korean_char(c: char) -> bool {
    let code = c as u32;
    // Old Hangul compatibility jamo beyond modern range
    (0x3165..=0x318E).contains(&code)
        // Explicit compatibility forms commonly found in historical texts
        || matches!(
            c,
            'ㅿ'
                | 'ㆁ'
                | 'ㆆ'
                | 'ㅸ'
                | 'ㅹ'
                | 'ㆄ'
                | 'ㅱ'
                | 'ㆇ'
                | 'ㆈ'
                | 'ㆉ'
                | 'ㆊ'
                | 'ㆋ'
                | 'ㆌ'
                | 'ㆍ'
                | 'ㆎ'
                | 'ㅥ'
                | 'ㆀ'
                | 'ㆅ'
        )
        // Old Hangul Jamo
        || (0x1100..=0x115F).contains(&code)
        || (0x1160..=0x11FF).contains(&code)
        // Hangul Jamo Extended-A/B
        || (0xA960..=0xA97C).contains(&code)
        || (0xD7B0..=0xD7FB).contains(&code)
        // Precomposed old Hangul syllables in PUA
        || (0xE000..=0xF8FF).contains(&code)
}

fn has_middle_korean_tone_punctuation(word: &[char]) -> bool {
    word.iter().any(|c| matches!(*c, '\u{00B7}' | '\u{FF1A}'))
}

fn has_strong_middle_korean_context(word: &[char]) -> bool {
    word.iter().any(|c| is_strong_middle_korean_char(*c))
}

fn nearest_prev_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<&'a [char]> {
    let mut i = index;
    while i > 0 {
        i -= 1;
        if let Token::Word(word) = &tokens[i] {
            return Some(&word.chars);
        }
    }
    None
}

fn nearest_next_word<'a>(tokens: &'a [Token<'a>], index: usize) -> Option<&'a [char]> {
    let mut i = index + 1;
    while i < tokens.len() {
        if let Token::Word(word) = &tokens[i] {
            return Some(&word.chars);
        }
        i += 1;
    }
    None
}

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "19",
    subsection: None,
    name: "middle_korean_detector",
    standard_ref: "2024 Korean Braille Standard, 제19항 옛 글자",
    description: "중세국어 문맥 감지 후 인코딩 모드 전환",
};

impl TokenRule for MiddleKoreanDetectorRule {
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &META
    }

    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        5
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let has_strong_context = has_strong_middle_korean_context(&word.chars);
        let has_tone_punctuation = has_middle_korean_tone_punctuation(&word.chars);

        let prev_has_context =
            nearest_prev_word(tokens, index).is_some_and(has_strong_middle_korean_context);
        let next_has_context =
            nearest_next_word(tokens, index).is_some_and(has_strong_middle_korean_context);

        let has_middle_korean =
            has_strong_context || (has_tone_punctuation && (prev_has_context || next_has_context));
        let preserve_explicit_middle_korean_mode = has_tone_punctuation
            && word.chars.len() == 1
            && state.current_mode() == EncodingMode::MiddleKorean;

        if has_middle_korean {
            let should_enter_middle_korean = state.current_mode() != EncodingMode::MiddleKorean;
            if should_enter_middle_korean {
                state.push_mode(EncodingMode::MiddleKorean);
            }
        } else if state.current_mode() == EncodingMode::MiddleKorean
            && !preserve_explicit_middle_korean_mode
        {
            state.pop_mode();
        }

        Ok(TokenAction::Noop)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::rules::context::EncoderState;
    use crate::rules::token::{WordMeta, WordToken};

    fn word(text: &str) -> Token<'_> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Borrowed(text),
            meta: WordMeta::from_chars(&chars),
            chars,
        })
    }

    /// 국립국어원 answered on 2026-09-21 that the mode follows the 옛글자 — "옛글자가
    /// 들어가면 그 글자에 대하여 그렇게 표기합니다" — and that 한자 is not
    /// transcribable at all, so a Hanja is no evidence of a historical text. A
    /// modern sentence that quotes one in parentheses is ordinary Korean.
    #[rstest::rstest]
    #[case::gloss_in_parentheses("中國")]
    #[case::single_hanja("州")]
    #[case::reading_then_hanja("지천명")]
    fn a_hanja_alone_does_not_enter_middle_korean_mode(#[case] text: &str) {
        let tokens = [word(text)];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_ne!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn entering_middle_korean_mode_from_strong_context_pushes_mode() {
        let text = std::hint::black_box("ᄒ");
        let tokens = [word(text)];
        let mut state = EncoderState::new(false);

        assert_ne!(state.current_mode(), EncodingMode::MiddleKorean);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn tone_punctuation_with_next_context_pushes_mode() {
        let tokens = [word("·"), word("ᄒ")];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn middle_korean_mode_is_not_pushed_when_already_active() {
        let tokens = [word("ᄒ")];
        let mut state = EncoderState::new(false);
        state.push_mode(EncodingMode::MiddleKorean);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn extended_b_jamo_enters_middle_korean_mode() {
        let tokens = [word(std::hint::black_box("ힰ"))];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn old_hangul_jungseong_enters_middle_korean_mode() {
        let tokens = [word(std::hint::black_box("ᅠ"))];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn private_use_old_hangul_enters_middle_korean_mode() {
        let tokens = [word(std::hint::black_box("\u{E000}"))];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn tone_punctuation_with_neighboring_context_pushes_mode() {
        let tokens = [word("ᄒ"), word("·")];
        let mut state = EncoderState::new(false);

        MiddleKoreanDetectorRule
            .apply(&tokens, 1, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }

    #[test]
    fn explicit_middle_korean_tone_punctuation_preserves_mode() {
        let tokens = [word("·")];
        let mut state = EncoderState::new(false);
        state.push_mode(EncodingMode::MiddleKorean);

        MiddleKoreanDetectorRule
            .apply(&tokens, 0, &mut state)
            .expect("middle Korean detector should not fail");

        assert_eq!(state.current_mode(), EncodingMode::MiddleKorean);
    }
}
