//! LatexMergeRule: merges \$...\$ sequences across spaces (extracted from latex_math.rs).

use crate::rules::context::EncoderState;
use crate::rules::token::Token;
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};

use super::encode_latex_math_bytes_with_context;
use super::math_context_from_state;

pub struct LatexMergeRule;

impl TokenRule for LatexMergeRule {
    fn phase(&self) -> TokenPhase {
        TokenPhase::Normalization
    }

    fn priority(&self) -> u16 {
        10 // Very early — merge before anything else
    }

    fn apply<'a>(
        &self,
        tokens: &[Token<'a>],
        index: usize,
        state: &mut EncoderState,
    ) -> Result<TokenAction<'a>, String> {
        let Some(Token::Word(word)) = tokens.get(index) else {
            return Ok(TokenAction::Noop);
        };

        let text = word.text.as_ref();

        // PDF — `제$n$항까지의` 같이 Korean prefix + `$X$` + Korean suffix 패턴.
        // 단어 내부 `$X$` math 블록을 분리해 prefix/inner/suffix로 분해한다.
        if !text.starts_with('$') && text.contains('$') {
            let first_dollar = text.find('$').unwrap();
            let after_first = &text[first_dollar + 1..];
            if let Some(close_rel) = after_first.find('$') {
                let prefix = &text[..first_dollar];
                let inner = &text[first_dollar + 1..first_dollar + 1 + close_rel];
                let suffix = &text[first_dollar + 1 + close_rel + 1..];
                // prefix가 Korean으로 끝나고 inner가 단일 letter면 ⠴X⠲ quote 형태.
                let prefix_ends_korean = prefix
                    .chars()
                    .last()
                    .is_some_and(crate::utils::is_korean_char);
                let inner_single_letter =
                    inner.chars().count() == 1 && inner.chars().all(|c| c.is_ascii_alphabetic());
                if prefix_ends_korean && inner_single_letter {
                    let math_context = math_context_from_state(state);
                    if let Ok(prefix_bytes) = crate::encode(prefix)
                        && let Ok(inner_bytes) =
                            encode_latex_math_bytes_with_context(inner, math_context)
                        && let Ok(suffix_bytes) = crate::encode(suffix)
                    {
                        let mut bytes = Vec::with_capacity(
                            prefix_bytes.len() + inner_bytes.len() + suffix_bytes.len() + 2,
                        );
                        bytes.extend(prefix_bytes);
                        bytes.push(52); // ⠴
                        bytes.extend(inner_bytes);
                        bytes.push(50); // ⠲
                        bytes.extend(suffix_bytes);
                        return Ok(TokenAction::ReplaceMany(vec![Token::PreEncoded(bytes)]));
                    }
                }
            }
        }

        // Only trigger on words starting with $ but NOT ending with $
        // (single-token $...$ is already handled by downstream rules)
        if !text.starts_with('$') || text.ends_with('$') {
            return Ok(TokenAction::Noop);
        }
        // PDF — `$a$는` 같이 단어 안에 짝수 개의 `$`가 이미 있으면(math 블록이 word 내에서
        // 종료됨) Korean prose 컨텍스트로 본다. ⠴...⠲로 quoted된 letter + Korean particle을
        // 직접 emit한다 (Normalization 단계에서 처리해야 후속 MathExpressionTokenRule이
        // 우회되지 않는다).
        let dollar_count = text.chars().filter(|c| *c == '$').count();
        if dollar_count % 2 == 0 {
            // `$X$<suffix>` 패턴 처리: math 블록 + 비-math 접미사 (Korean/구두점 등).
            if dollar_count == 2
                && let Some(close_idx) = text[1..].find('$').map(|i| i + 1)
            {
                let inner = &text[1..close_idx];
                let suffix = &text[close_idx + 1..];
                let has_korean_suffix = suffix
                    .chars()
                    .next()
                    .is_some_and(crate::utils::is_korean_char);
                // 단일 letter: ASCII 알파벳 또는 `\<greek>` (예: \omega, \alpha)
                let inner_is_short_letter = (inner.chars().count() == 1
                    && inner.chars().all(|c| c.is_ascii_alphabetic()))
                    || (inner.starts_with('\\')
                        && inner.chars().count() > 1
                        && inner.chars().skip(1).all(|c| c.is_ascii_alphabetic())
                        && [
                            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
                            "iota", "kappa", "lambda", "mu", "nu", "xi", "pi", "rho", "sigma",
                            "tau", "upsilon", "phi", "chi", "psi", "omega",
                        ]
                        .contains(&&inner[1..]));
                // Case 1: 단일 letter + Korean → ⠴letter⠲ PreEncoded + Korean Word
                // suffix를 별도 Word로 유지해 다음 math expression이 Korean prose 컨텍스트로
                // 두 칸 간격(혹은 quote-wrap)을 판정할 수 있게 한다.
                let math_context = math_context_from_state(state);
                if has_korean_suffix
                    && inner_is_short_letter
                    && let Ok(inner_bytes) =
                        encode_latex_math_bytes_with_context(inner, math_context)
                {
                    let mut bytes = Vec::with_capacity(inner_bytes.len() + 2);
                    bytes.push(52); // ⠴
                    bytes.extend(inner_bytes);
                    bytes.push(50); // ⠲
                    let suffix_chars: Vec<char> = suffix.chars().collect();
                    let suffix_meta = crate::rules::token::WordMeta::from_chars(&suffix_chars);
                    let suffix_word = Token::Word(crate::rules::token::WordToken {
                        text: std::borrow::Cow::Owned(suffix.to_string()),
                        chars: suffix_chars,
                        meta: suffix_meta,
                    });
                    return Ok(TokenAction::ReplaceMany(vec![
                        Token::PreEncoded(bytes),
                        suffix_word,
                    ]));
                }
            }
            return Ok(TokenAction::Noop);
        }

        // PDF — `$...$` 토큰 분리 케이스: 이미 `DocumentIR::parse()`
        // (token.rs:106-119)가 dollar-count가 odd면 끝까지 merge하므로 이 함수가
        // 받는 시점에는 항상 단일 Word 토큰이다. 후속 다중 토큰 스캔 분기는
        // 도달 불가하므로 단순 Noop으로 종결한다.
        Ok(TokenAction::Noop)
    }
}
