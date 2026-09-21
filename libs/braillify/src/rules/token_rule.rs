use super::RuleMeta;
use super::context::EncoderState;
use super::token::Token;

/// Placeholder for a token rule that has not declared its source article yet.
/// Rules keeping this default are reported as unattributed rather than being
/// credited to an article nobody checked against the standard.
pub static UNDECLARED_TOKEN_RULE: RuleMeta = RuleMeta {
    section: "?",
    subsection: None,
    name: "undeclared_token_rule",
    standard_ref: "",
    description: "",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenPhase {
    Normalization = 0,
    FractionDetection = 1,
    WordShortcut = 2,
    ModeEntry = 3,
    UppercasePassage = 4,
    PostWord = 5,
}

pub enum TokenAction<'a> {
    Noop,
    Replace(Token<'a>),
    #[cfg(test)]
    InsertBefore(Vec<Token<'a>>),
    ReplaceMany(Vec<Token<'a>>),
    /// 현재 토큰(i)부터 N개의 토큰을 모두 제거하고 주어진 토큰들로 교체한다.
    /// 다중 토큰 패턴(예: Word+Space+Word)을 단일 결과로 합칠 때 사용.
    ReplaceRange(usize, Vec<Token<'a>>),
    #[cfg(test)]
    Remove,
}

pub trait TokenRule: Send + Sync {
    /// The standard article this rule implements. Defaults to
    /// [`UNDECLARED_TOKEN_RULE`] so a rule is reported as unattributed until
    /// someone checks its article against the PDF.
    fn meta(&self) -> &'static RuleMeta {
        &UNDECLARED_TOKEN_RULE
    }

    fn phase(&self) -> TokenPhase;
    fn priority(&self) -> u16 {
        100
    }
    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String>;
}
