use std::borrow::Cow;

use super::context::EncoderState;

pub struct DocumentIR<'a> {
    pub tokens: Vec<Token<'a>>,
    pub state: EncoderState,
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Word(WordToken<'a>),
    Space(SpaceKind),
    Fraction(FractionToken),
    Mode(ModeEvent),
    PreEncoded(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct WordToken<'a> {
    pub text: Cow<'a, str>,
    pub chars: Vec<char>,
    pub meta: WordMeta,
}

#[derive(Debug, Clone, Copy)]
pub struct WordMeta {
    pub has_korean: bool,
    pub is_all_uppercase: bool,
    pub starts_with_ascii: bool,
    pub has_ascii_alphabetic: bool,
}

impl WordMeta {
    pub fn from_chars(chars: &[char]) -> WordMeta {
        let has_korean = chars
            .iter()
            .any(|ch| (0xAC00..=0xD7A3).contains(&(*ch as u32)));
        let ascii_letter_count = chars.iter().filter(|ch| ch.is_ascii_alphabetic()).count();
        let uppercase_count = chars.iter().filter(|ch| ch.is_ascii_uppercase()).count();
        let has_ascii_alphabetic = ascii_letter_count > 0;

        let starts_with_ascii = chars.first().is_some_and(char::is_ascii_alphabetic);
        let is_all_uppercase = ascii_letter_count >= 2 && ascii_letter_count == uppercase_count;

        WordMeta {
            has_korean,
            is_all_uppercase,
            starts_with_ascii,
            has_ascii_alphabetic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceKind {
    Regular,
}

#[derive(Debug, Clone)]
pub struct FractionToken {
    pub whole: Option<String>,
    pub numerator: String,
    pub denominator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeEvent {
    EnterEnglish,
    EnterEnglishContinue,
    CapsWord,
    CapsPassageStart,
    CapsPassageEnd,
    /// Grade-1 indicator (⠰, byte 48). Required before CapsWord/CapsPassage when the
    /// uppercase ASCII letters spell a multi-letter shortform/contraction (UEB 5.7.2 + 10.9).
    /// Example: "CD" → ⠰⠠⠠⠉⠙ because ⠉⠙ alone = "could" shortform.
    Grade1Indicator,
}

impl<'a> DocumentIR<'a> {
    pub fn parse(text: &'a str, english_indicator: bool) -> Self {
        let raw_words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
        let mut tokens = Vec::new();
        let mut i = 0usize;

        while i < raw_words.len() {
            let mut owned_merged: Option<String> = None;
            let mut consumed = 1usize;

            // Merge $...$ expressions that may contain spaces into a single token.
            let first = raw_words[i];
            let mut dollar_count = first.chars().filter(|c| *c == '$').count();
            if dollar_count % 2 == 1 {
                let mut merged = first.to_string();
                let mut j = i + 1;
                while j < raw_words.len() {
                    merged.push(' ');
                    merged.push_str(raw_words[j]);
                    dollar_count += raw_words[j].chars().filter(|c| *c == '$').count();
                    j += 1;
                    if dollar_count % 2 == 0 {
                        break;
                    }
                }
                consumed = j.saturating_sub(i);
                owned_merged = Some(merged);
            } else if let Some((merged, merged_count)) = merge_math_span(&raw_words, i) {
                consumed = merged_count;
                owned_merged = Some(merged);
            }

            let text_cow = match owned_merged {
                Some(merged) => Cow::Owned(merged),
                None => Cow::Borrowed(raw_words[i]),
            };

            let chars: Vec<char> = text_cow.chars().collect();
            let meta = WordMeta::from_chars(&chars);
            tokens.push(Token::Word(WordToken {
                text: text_cow,
                chars,
                meta,
            }));

            i += consumed;
            if i < raw_words.len() {
                tokens.push(Token::Space(SpaceKind::Regular));
            }
        }

        DocumentIR {
            tokens,
            state: EncoderState::new(english_indicator),
        }
    }
}

fn is_korean_char(c: char) -> bool {
    let code = c as u32;
    (0xAC00..=0xD7A3).contains(&code) || (0x3131..=0x3163).contains(&code)
}

fn is_math_span_char(c: char) -> bool {
    is_korean_char(c)
        || c.is_ascii_alphanumeric()
        || matches!(
            c,
            '=' | '+'
                | '-'
                | '\u{2212}'
                | '×'
                | '÷'
                | '/'
                | '√'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | ','
                | '.'
                | '!'
                | '<'
                | '>'
                | ':'
                | ';'
                | '\''
                | '"'
        )
}

fn has_math_trigger(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '=' | '×' | '÷' | '/' | '√' | '{' | '}' | '[' | ']' | '(' | ')'
        )
    }) || text.contains("...")
}

fn merge_math_span(raw_words: &[&str], start: usize) -> Option<(String, usize)> {
    let mut merged = String::new();
    let mut end = start;
    let mut paren_balance = 0i32;
    let mut square_balance = 0i32;
    let mut curly_balance = 0i32;
    let mut saw_korean = false;
    let mut saw_trigger = false;
    let mut best: Option<(String, usize)> = None;

    while end < raw_words.len() {
        let word = raw_words[end];
        // Closure (not bare fn) is intentional: forces tarpaulin to attribute
        // the call to `is_math_span_char` body instead of fn declaration line.
        #[allow(clippy::redundant_closure)]
        let all_span = word.chars().all(|c| is_math_span_char(c));
        if !all_span {
            break;
        }

        if !merged.is_empty() {
            merged.push(' ');
        }
        merged.push_str(word);

        for ch in word.chars() {
            saw_korean |= is_korean_char(ch);
            saw_trigger |= matches!(
                ch,
                '=' | '×' | '÷' | '/' | '√' | '{' | '}' | '[' | ']' | '(' | ')'
            );
            match ch {
                '(' => paren_balance += 1,
                ')' => paren_balance -= 1,
                '[' => square_balance += 1,
                ']' => square_balance -= 1,
                '{' => curly_balance += 1,
                '}' => curly_balance -= 1,
                _ => {}
            }
        }

        let balanced = paren_balance == 0 && square_balance == 0 && curly_balance == 0;
        let multi_word = end > start;
        let looks_like_span = saw_trigger || has_math_trigger(&merged);
        let is_brace_math = merged.contains('=') && merged.contains('{') && merged.contains('}');
        // BMI 같은 영문자 + 한글 mixed (`BMI(체질량 지수) = ...`)는 일반 한국어 path가
        // 더 잘 처리. mixed_korean_math 분기는 순수 한글 명사구 + 수식 입력만 대상으로.
        let is_mixed_korean_math = saw_korean
            && merged.contains('=')
            && (merged.contains('×') || merged.contains('√'))
            && !merged.chars().any(|c| c.is_ascii_alphabetic());

        if multi_word && balanced && looks_like_span && (is_brace_math || is_mixed_korean_math) {
            best = Some((merged.clone(), end + 1 - start));
        }

        end += 1;
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_meta_korean_only() {
        let chars: Vec<char> = "안녕".chars().collect();
        let meta = WordMeta::from_chars(&chars);
        assert!(meta.has_korean);
        assert!(!meta.has_ascii_alphabetic);
        assert!(!meta.starts_with_ascii);
        assert!(!meta.is_all_uppercase);
    }

    #[test]
    fn word_meta_english_uppercase() {
        let chars: Vec<char> = "ATM".chars().collect();
        let meta = WordMeta::from_chars(&chars);
        assert!(!meta.has_korean);
        assert!(meta.has_ascii_alphabetic);
        assert!(meta.starts_with_ascii);
        assert!(meta.is_all_uppercase);
    }

    #[test]
    fn word_meta_mixed() {
        let chars: Vec<char> = "A한b".chars().collect();
        let meta = WordMeta::from_chars(&chars);
        assert!(meta.has_korean);
        assert!(meta.has_ascii_alphabetic);
        assert!(meta.starts_with_ascii);
        assert!(!meta.is_all_uppercase);
    }

    #[test]
    fn token_debug_clone_works() {
        let token = Token::Word(WordToken {
            text: Cow::Borrowed("hello"),
            chars: vec!['h', 'e', 'l', 'l', 'o'],
            meta: WordMeta::from_chars(&['h', 'e', 'l', 'l', 'o']),
        });
        let cloned = token.clone();
        assert!(format!("{cloned:?}").contains("Word"));
    }

    #[test]
    fn parse_simple_words() {
        let ir = DocumentIR::parse("hello world", false);
        assert_eq!(ir.tokens.len(), 3);

        match &ir.tokens[0] {
            Token::Word(w) => assert_eq!(w.text, "hello"),
            _ => panic!("expected first token to be word"),
        }
        assert!(matches!(ir.tokens[1], Token::Space(SpaceKind::Regular)));
        match &ir.tokens[2] {
            Token::Word(w) => assert_eq!(w.text, "world"),
            _ => panic!("expected third token to be word"),
        }
    }

    #[test]
    fn parse_empty() {
        let ir = DocumentIR::parse("", false);
        assert!(ir.tokens.is_empty());
    }

    #[test]
    fn parse_sets_meta() {
        let ir = DocumentIR::parse("ATM 한A", true);
        match &ir.tokens[0] {
            Token::Word(w) => {
                assert!(w.meta.is_all_uppercase);
                assert!(w.meta.starts_with_ascii);
            }
            _ => panic!("expected word"),
        }
        match &ir.tokens[2] {
            Token::Word(w) => {
                assert!(w.meta.has_korean);
                assert!(w.meta.has_ascii_alphabetic);
                assert!(!w.meta.is_all_uppercase);
            }
            _ => panic!("expected word"),
        }
    }
}
