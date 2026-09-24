use std::borrow::Cow;

use crate::rules::RuleMeta;
use crate::rules::token::{SpaceKind, Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

/// 제60항이 별표와 참고표의 앞뒤를 한 칸씩 띄우도록 하므로 별표 간격을 조정한다.
pub struct AsteriskSpacingRule;

/// 제60항 — 글의 첫머리에 선 별표는 주석을 이끄는 표이므로 본문대로 뒤를 한 칸
/// 띄운다(`*조용제 지사 이야기는`). [다만 2] 가 묵자를 따르게 하는 것은 본문 속에서
/// 주석을 가리키는 별표(`*가온 음자리표`)다.
pub struct LeadingAsteriskSpacingRule;

/// Compatibility registration for the removed auxiliary-verb normalizer.
///
/// Korean rule 49 says that braille spacing follows print. Consequently the
/// encoder must not correct an attached `있다` by inserting a space that is not
/// present in the input. The registry type remains temporarily stable, but the
/// rule deliberately performs no transformation.
pub struct KoreanAuxiliaryVerbSpacingRule;

static META_AUXILIARY_SPACING: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "korean_auxiliary_verb_spacing",
    standard_ref: "2024 Korean Braille Standard, 제49항",
    description: "Preserve Korean print spacing for auxiliary verbs",
};

impl TokenRule for KoreanAuxiliaryVerbSpacingRule {
    fn meta(&self) -> &'static RuleMeta {
        &META_AUXILIARY_SPACING
    }

    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        50 // Registry compatibility; no normalization is performed.
    }

    fn apply<'a>(
        &self,
        _tokens: &[Token<'a>],
        _index: usize,
        _state: &mut crate::rules::context::EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        Ok(TokenAction::Noop)
    }
}

fn is_last_word_index(tokens: &[Token], index: usize) -> bool {
    !tokens
        .iter()
        .skip(index + 1)
        .any(|t| matches!(t, Token::Word(_)))
}

static META: crate::rules::RuleMeta = crate::rules::RuleMeta {
    section: "60",
    subsection: None,
    name: "asterisk_spacing",
    standard_ref: "2024 Korean Braille Standard, 제60항 별표·참고표",
    description: "별표 앞뒤 띄어쓰기 조정",
};

impl TokenRule for AsteriskSpacingRule {
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &META
    }

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

fn owned_word<'a>(chars: &[char]) -> Token<'a> {
    Token::Word(WordToken {
        text: Cow::Owned(chars.iter().collect()),
        chars: chars.to_vec(),
        meta: WordMeta::from_chars(chars),
    })
}

impl TokenRule for LeadingAsteriskSpacingRule {
    fn meta(&self) -> &'static crate::rules::RuleMeta {
        &META
    }

    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        195
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
        let opens_the_text = !tokens[..index]
            .iter()
            .any(|token| matches!(token, Token::Word(_)));
        match current.chars.as_slice() {
            ['*', rest @ ..] if opens_the_text && !rest.is_empty() && !rest.contains(&'*') => {
                Ok(TokenAction::ReplaceMany(vec![
                    owned_word(&['*']),
                    Token::Space(SpaceKind::Regular),
                    owned_word(rest),
                ]))
            }
            _ => Ok(TokenAction::Noop),
        }
    }
}

#[cfg(test)]
mod leading_asterisk {
    /// 제60항: 글을 여는 별표 뒤는 한 칸 띄우고, 본문 속 별표와 겹별표는 묵자를 따른다.
    #[rstest::rstest]
    #[case::note_heading("*조용제 지사 이야기는", "⠐⠔⠀⠨⠥")]
    #[case::already_spaced("* 조용제 지사", "⠐⠔⠀⠨⠥")]
    #[case::pointer_in_the_text("가나 *가온 음자리표", "⠐⠔⠫⠷")]
    #[case::double_asterisk("**가나", "⠐⠔⠐⠔⠫")]
    #[case::asterisks_around_a_word("*안녕*", "⠐⠔⠣⠒")]
    #[case::reference_mark_alone("※한국리서치는", "⠐⠔⠀⠚⠒")]
    #[case::reference_mark_beside_an_asterisk("가 ※ 나 *", "⠸⠔")]
    fn a_note_heading_asterisk_takes_a_blank(#[case] input: &str, #[case] cells: &str) {
        let encoded = crate::encode_to_unicode(input).unwrap();
        assert!(encoded.contains(cells), "{encoded}");
    }
}

#[cfg(test)]
mod tests {
    /// 제49항은 묵자의 띄어쓰기를 따르며, 각 spaced 입력은 PDF에 그대로
    /// 실린 예제다. 대응 attached 입력에서는 없는 공백을 새로 만들지 않는다.
    #[rstest::rstest]
    #[case::rule18("그림을 그리고 있다.", "그림을 그리고있다.")]
    #[case::rule29(
        "그녀는 Los Angeles의 한인 타운에 살고 있다.",
        "그녀는 Los Angeles의 한인 타운에 살고있다."
    )]
    #[case::rule36(
        "가영이는 미적분학 II 과목을 수강하고 있다.",
        "가영이는 미적분학 II 과목을 수강하고있다."
    )]
    fn full_encoder_preserves_printed_auxiliary_spacing_only(
        #[case] spaced: &str,
        #[case] attached: &str,
    ) {
        let spaced_output = crate::encode_to_unicode(spaced).expect("PDF example must encode");
        let attached_output = crate::encode_to_unicode(attached).expect("control must encode");
        let spaced_blanks = spaced_output.chars().filter(|cell| *cell == '⠀').count();
        let attached_blanks = attached_output.chars().filter(|cell| *cell == '⠀').count();

        assert_eq!(spaced_blanks, attached_blanks + 1);
    }
}
