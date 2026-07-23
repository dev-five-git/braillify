use super::context::EncoderState;
use super::token::Token;

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
