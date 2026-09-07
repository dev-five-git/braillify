use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

pub struct AsteriskSpacingRule;

/// Compatibility registration for the removed auxiliary-verb normalizer.
///
/// Korean rule 49 says that braille spacing follows print. Consequently the
/// encoder must not correct an attached `있다` by inserting a space that is not
/// present in the input. The registry type remains temporarily stable, but the
/// rule deliberately performs no transformation.
pub struct KoreanAuxiliaryVerbSpacingRule;

impl TokenRule for KoreanAuxiliaryVerbSpacingRule {
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
