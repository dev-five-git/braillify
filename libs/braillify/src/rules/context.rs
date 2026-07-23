//! Shared context and state for rule execution.
//!
//! `RuleContext` provides the current encoding position and read access to input.
//! `EncoderState` tracks persistent state across characters/words (English mode, etc.).

use crate::char_struct::{CharType, KoreanChar};

/// Document-level predicates computed once before token rules run.
#[derive(Debug, Default, Clone, Copy)]
pub struct DocumentSummary {
    /// Result of `document_has_english_context_for_korean(tokens)`.
    pub has_english_context_for_korean: bool,
    /// Result of `document_is_english_majority(tokens)`.
    pub is_english_majority: bool,
    /// Result of `document_is_english_dominant(tokens)`.
    pub is_english_dominant: bool,
}

/// The encoding context determines how ambiguous characters are interpreted.
/// For example, `·` is a tone mark in MiddleKorean mode but a middle dot in Korean mode.
///
/// `Math` and `Number` are deliberately separate even though both wrap their content
/// with the Roman indicator `⠴`. The distinction matters for inputs whose textual
/// form is identical but whose semantic role differs:
///   - `Math`: ASCII letters are mathematical variables (제12항).
///     Single `i`, `v`, `x` ⇒ `⠴ + letter` (no terminator).
///   - `Number`: ASCII letters in {I,V,X,L,C,D,M} form Roman numerals (제36항).
///     Single `i` ⇒ `⠴ + letter + ⠲` (with terminator).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingMode {
    /// Default Korean braille encoding
    Korean,
    /// English/Roman letter section (between ⠴ and ⠲)
    English,
    /// Math expression encoding (제12항 — letters are variables)
    Math,
    /// Numeric / Roman numeral section (제36항 — letters are numerals)
    Number,
    /// Middle Korean (중세국어) — archaic characters with special rules
    MiddleKorean,
    /// Object symbol (사물부호) — 제49항: `○`, `×`, `△`, `□` 등이 사물부호로 쓰이는 경우.
    /// 글머리 기호(제72항)와 동일 문자지만 점자 마무리 `⠇`(7)이 붙는다는 차이가 있다.
    ObjectSymbol,
    /// IPA notation — 제38항: 발음 기호 표기.
    /// `[ ]`는 ⠐⠘⠷ … ⠘⠾, `/ /`는 ⠐⠘⠌ … ⠘⠌으로 묶는다.
    /// 음운 기호(ə, ː, θ, ŋ, æ 등)는 국제음성기호 점자 변환표에 따라 점역한다.
    Ipa,
}

impl std::str::FromStr for EncodingMode {
    type Err = ();

    /// Parse testcase JSON `context` field (e.g. "math", "number") into an `EncodingMode`.
    /// Unknown context strings (including "" and ad-hoc metadata like "strip_prefix:…")
    /// return `Err`, which the caller treats as "no explicit mode" → default encoding.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "korean" => Ok(Self::Korean),
            "english" => Ok(Self::English),
            "math" => Ok(Self::Math),
            "number" => Ok(Self::Number),
            "middle_korean" => Ok(Self::MiddleKorean),
            "object_symbol" => Ok(Self::ObjectSymbol),
            "ipa" => Ok(Self::Ipa),
            _ => Err(()),
        }
    }
}

/// Persistent state that survives across characters and words.
///
/// Tracks modal state like "are we currently in English mode?"
/// Rules can read and mutate this state.
#[derive(Debug, Clone)]
pub struct EncoderState {
    /// Stack of encoding modes. The top determines current context.
    pub mode_stack: Vec<EncodingMode>,
    /// Currently inside a Roman letter section (between ⠴ and ⠲)
    pub is_english: bool,
    /// Whether the input contains Korean (determines if Roman indicators are needed)
    pub english_indicator: bool,
    /// Currently in a triple-uppercase passage (⠠⠠⠠ ... ⠠⠄)
    pub triple_big_english: bool,
    /// Whether at least one word has been processed
    pub has_processed_word: bool,
    /// Need to emit English continuation marker (⠐) on next English char
    pub needs_english_continuation: bool,
    /// Rule 35 chain: English followed by digits may resume English without indicators
    pub roman_number_chain: bool,
    /// Stack tracking whether parentheses were opened in English context
    pub parenthesis_stack: Vec<bool>,
    /// Currently in a number sequence (수표 already emitted)
    pub is_number: bool,
    /// Currently in a consecutive uppercase run within a word
    pub is_big_english: bool,
    /// 제39항: 영-한 wrap이 활성화된 문서. 단독 단어 "in", "be" 등도 UEB 약자 적용.
    pub english_dominant_wrap_active: bool,
    /// 제39항: 영어 주도(영어 어절 ≫ 한글) 문서. 영자표시(⠴)·단일 대문자 표시
    /// (⠠)·종료표(⠲)를 모두 생략한다.
    pub english_dominant_no_indicator: bool,
    /// Document-level predicates cached for token rules.
    pub doc_summary: DocumentSummary,
    /// PDF 제12항 붙임 1 — document contains the `행렬` keyword.
    /// Enables matrix-name rendering of two-letter uppercase math identifiers.
    pub matrix_context_active: bool,
    /// Explicit math mode (`context = math` in fixtures/API options).
    /// Keeps parentheses in math form even when their contents include Hangul.
    pub math_mode_active: bool,
    /// 짝맞춤 작은따옴표(`‘…’`) 추적: `‘`를 만나면 +1, 닫음 `’`로 -1.
    /// 0보다 크면 현재 위치는 paired closing 위치이므로 `’`를 `⠴⠄`로 emit.
    /// 0이면 standalone apostrophe로 `⠄` 한 셀만 emit. (PDF 제61항)
    pub unmatched_open_single_quotes: i32,
}

impl EncoderState {
    pub fn new(english_indicator: bool) -> Self {
        Self {
            mode_stack: vec![EncodingMode::Korean],
            english_indicator,
            is_english: false,
            triple_big_english: false,
            has_processed_word: false,
            needs_english_continuation: false,
            roman_number_chain: false,
            parenthesis_stack: Vec::new(),
            is_number: false,
            is_big_english: false,
            english_dominant_wrap_active: false,
            english_dominant_no_indicator: false,
            doc_summary: DocumentSummary::default(),
            matrix_context_active: false,
            math_mode_active: false,
            unmatched_open_single_quotes: 0,
        }
    }

    /// Get the current encoding mode (top of stack, default Korean).
    pub fn current_mode(&self) -> EncodingMode {
        self.mode_stack
            .last()
            .copied()
            .unwrap_or(EncodingMode::Korean)
    }

    /// Push a new encoding mode onto the stack.
    pub fn push_mode(&mut self, mode: EncodingMode) {
        self.mode_stack.push(mode);
    }

    /// Pop the current encoding mode, returning to the previous one.
    pub fn pop_mode(&mut self) -> Option<EncodingMode> {
        if self.mode_stack.len() > 1 {
            self.mode_stack.pop()
        } else {
            None
        }
    }
}

/// Snapshot of the current encoding position within a word.
///
/// This is the "view" that each rule receives. Rules read this to decide
/// whether they match, then mutate `result` and `state` via `RuleContext`.
pub struct RuleContext<'a> {
    /// All characters in the current word
    pub word_chars: &'a [char],
    /// Current character index within the word
    pub index: usize,
    /// The classified type of the current character
    pub char_type: &'a CharType,
    /// Previous word (for cross-word context)
    pub prev_word: &'a str,
    /// Remaining words after this one
    pub remaining_words: &'a [&'a str],
    /// Whether this word contains any Korean syllable characters
    pub has_korean_char: bool,
    /// Whether the whole word is uppercase ASCII
    pub is_all_uppercase: bool,
    /// Whether ASCII letters start at index 0
    pub ascii_starts_at_beginning: bool,
    /// Skip count — rules can set this to skip subsequent characters
    pub skip_count: &'a mut usize,
    /// Shared mutable encoder state
    pub state: &'a mut EncoderState,
    /// Output buffer
    pub result: &'a mut Vec<u8>,
}

impl<'a> RuleContext<'a> {
    /// Current character.
    pub fn current_char(&self) -> char {
        self.word_chars[self.index]
    }

    /// Next character in the word, if any.
    pub fn next_char(&self) -> Option<char> {
        self.word_chars.get(self.index + 1).copied()
    }

    /// Previous character in the word, if any.
    pub fn prev_char(&self) -> Option<char> {
        if self.index > 0 {
            Some(self.word_chars[self.index - 1])
        } else {
            None
        }
    }

    /// Word length.
    pub fn word_len(&self) -> usize {
        self.word_chars.len()
    }

    /// Get the current KoreanChar if the char_type is Korean.
    pub fn as_korean(&self) -> Option<&KoreanChar> {
        if let CharType::Korean(k) = self.char_type {
            Some(k)
        } else {
            None
        }
    }

    /// Emit braille cell(s) to the output buffer.
    pub fn emit(&mut self, byte: u8) {
        self.result.push(byte);
    }

    /// Emit a slice of braille cells.
    pub fn emit_slice(&mut self, bytes: &[u8]) {
        self.result.extend_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn encoding_mode_from_str_all_variants() {
        assert_eq!(EncodingMode::from_str("korean"), Ok(EncodingMode::Korean));
        assert_eq!(EncodingMode::from_str("english"), Ok(EncodingMode::English));
        assert_eq!(EncodingMode::from_str("math"), Ok(EncodingMode::Math));
        assert_eq!(EncodingMode::from_str("number"), Ok(EncodingMode::Number));
        assert_eq!(
            EncodingMode::from_str("middle_korean"),
            Ok(EncodingMode::MiddleKorean)
        );
        assert_eq!(
            EncodingMode::from_str("object_symbol"),
            Ok(EncodingMode::ObjectSymbol)
        );
        assert_eq!(EncodingMode::from_str("ipa"), Ok(EncodingMode::Ipa));
    }

    #[test]
    fn encoding_mode_from_str_unknown_returns_err() {
        assert!(EncodingMode::from_str("unknown").is_err());
        assert!(EncodingMode::from_str("").is_err());
        assert!(EncodingMode::from_str("KOREAN").is_err());
    }

    #[test]
    fn encoder_state_mode_stack_pushes_and_pops_runtime_mode() {
        let mut state = EncoderState::new(false);
        let mode = std::hint::black_box(EncodingMode::English);

        assert_eq!(state.current_mode(), EncodingMode::Korean);
        state.push_mode(mode);
        assert_eq!(state.current_mode(), EncodingMode::English);
        assert_eq!(state.pop_mode(), Some(EncodingMode::English));
        assert_eq!(state.pop_mode(), None);
    }
}
