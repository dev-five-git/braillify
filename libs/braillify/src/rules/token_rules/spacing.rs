use std::borrow::Cow;

use crate::rules::token::{SpaceKind, Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct AsteriskSpacingRule;

/// 한국어 보조용언 `있다`(있-) 분리.
///
/// PDF 한국 점자 규정 / 한글 띄어쓰기 일반 원칙에 따라 보조용언 `있다`(있다·있었다 등)는
/// 본용언과 띄어 쓴다. 묵자 입력은 띄어쓰기가 생략되어 한 단어로 들어오는 경우가 있어
/// (예: "덮여있다"), 토큰 단계에서 명시적으로 분리하여 점자 출력의 빈칸을 보장한다.
///
/// 보수적 매칭:
/// - 단어 끝이 `있다` 또는 그 변형(`있다.`, `있다?`, `있다!`, `있어`, `있었다` 등)일 때만
///   접두 부분과 분리한다.
/// - 접두 부분이 비어 있으면 분리하지 않는다(독립된 "있다" 토큰은 그대로 둔다).
/// - 접두 부분에 한글 음절이 1개라도 있어야 한다.
pub struct KoreanAuxiliaryVerbSpacingRule;

const AUX_VERB_SUFFIXES: &[&str] = &[
    // 보조용언 본형 "있다"만 우선 분리. 변형 형태(있어요, 있습니다, 있었다 등)는
    // testcase 회귀 분석을 거치며 보수적으로 확장한다.
    "있다.", "있다",
];

fn split_aux_verb(text: &str) -> Option<(&str, &str)> {
    for suffix in AUX_VERB_SUFFIXES {
        if let Some(prefix) = text.strip_suffix(suffix)
            && !prefix.is_empty()
            && prefix.chars().any(crate::utils::is_korean_char)
        {
            return Some((prefix, *suffix));
        }
    }
    None
}

impl TokenRule for KoreanAuxiliaryVerbSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        50 // Word_shortcut(100)·LaTeX(110+)보다 먼저 분리
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        if !word.meta.has_korean {
            return Ok(TokenAction::Noop);
        }

        let text = word.text.as_ref();
        let Some((prefix, suffix)) = split_aux_verb(text) else {
            return Ok(TokenAction::Noop);
        };

        let prefix_owned = prefix.to_string();
        let suffix_owned = suffix.to_string();
        let prefix_chars: Vec<char> = prefix_owned.chars().collect();
        let suffix_chars: Vec<char> = suffix_owned.chars().collect();

        Ok(TokenAction::ReplaceMany(vec![
            Token::Word(WordToken {
                text: Cow::Owned(prefix_owned),
                chars: prefix_chars.clone(),
                meta: WordMeta::from_chars(&prefix_chars),
            }),
            Token::Space(SpaceKind::Regular),
            Token::Word(WordToken {
                text: Cow::Owned(suffix_owned),
                chars: suffix_chars.clone(),
                meta: WordMeta::from_chars(&suffix_chars),
            }),
        ]))
    }
}

fn is_last_word_index(tokens: &[Token], index: usize) -> bool {
    !tokens
        .iter()
        .skip(index + 1)
        .any(|t| matches!(t, Token::Word(_)))
}

impl TokenRule for AsteriskSpacingRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        400
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(current)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        if !is_last_word_index(tokens, index) {
            return Ok(TokenAction::Noop);
        }

        let mut trailing_spaces = 0usize;

        if current.text == "*" || current.text.ends_with('*') {
            trailing_spaces += 1;
        }

        if trailing_spaces == 0 {
            return Ok(TokenAction::Noop);
        }

        let replacement = vec![
            Token::Word(current.clone()),
            Token::PreEncoded(vec![0; trailing_spaces]),
        ];
        Ok(TokenAction::ReplaceMany(replacement))
    }
}
