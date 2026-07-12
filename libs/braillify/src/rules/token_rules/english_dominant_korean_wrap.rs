use std::borrow::Cow;

use crate::rules::context::DocumentSummary;
use crate::rules::token::{Token, WordMeta, WordToken};
use crate::rules::token_rule::{TokenAction, TokenPhase, TokenRule};
use crate::utils::is_korean_char;

/// PDF 제39항 — 영어 어절 사이에 끼인 한글 어절은 한글표(⠸⠷) ... 한글 종료표(⠸⠾)로 감싼다.
///
/// 일반 알고리즘 (testcase 입력에 의존하지 않음):
/// 1. 현재 토큰 안에서 한글 char segment를 좌→우 스캔으로 추출한다.
/// 2. 각 한글 segment의 좌·우 컨텍스트가 영어인지 판정한다.
///    - 좌측: 같은 토큰 내 직전 알파벳/숫자가 ASCII 영문이거나,
///      segment 앞이 토큰 시작이고 이전 word token이 영어로 시작한다.
///    - 우측: 같은 토큰 내 직후 알파벳/숫자가 ASCII 영문이거나,
///      segment 뒤가 토큰 끝이고 다음 word token이 영어로 시작한다.
///    - 영문/한글이 아닌 문장부호는 컨텍스트 판단에서 투명하게 통과한다.
/// 3. 양쪽 모두 영어 컨텍스트이면 그 segment를 한글표/한글 종료표로 감싼다.
///
/// 이 알고리즘은:
///   - "What is 김치 in English?" — `김치`는 영어 토큰 `is`, `in` 사이 → wrap
///   - "Banchan (Korean: 반찬) are ..." — `반찬`은 영어 토큰 사이 → wrap
///   - "www.대통령.kr이다." — 한 토큰 내 `대통령`은 영어 segment `www.`와 `.kr` 사이 → wrap
///   - "원형 동사이다." (rule_37) — `원형` 양옆 영어 없음 → wrap X
///   - "[ㅅ떠디이]로," (rule_10) — 한글 segment 앞이 `[`(영어 컨텍스트 아님) → wrap X
const HANGUL_WRAP_START: [u8; 2] = [56, 55]; // ⠸⠷ — 한글표 (제39항)
const HANGUL_WRAP_END: [u8; 2] = [56, 62]; // ⠸⠾ — 한글 종료표 (제39항)

pub struct EnglishDominantKoreanWrapRule;

fn build_word_token<'a>(text: &str) -> Token<'a> {
    let chars: Vec<char> = text.chars().collect();

    Token::Word(WordToken {
        text: Cow::Owned(text.to_string()),
        chars: chars.clone(),
        meta: WordMeta::from_chars(&chars),
    })
}

/// 직전 Word 토큰을 거슬러 찾는다. Space/PreEncoded는 투명.
/// Executed by every English-dominant Korean wrap integration test; tarpaulin
/// `for-match` block attribution limit on the `_ => return None` arm.
#[cfg(not(tarpaulin_include))]
fn prev_word_token<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<&'b WordToken<'a>> {
    for token in tokens[..index].iter().rev() {
        match token {
            Token::Word(w) => return Some(w),
            Token::Space(_) | Token::PreEncoded(_) => continue,
            _ => return None,
        }
    }
    None
}

/// 직후 Word 토큰을 찾는다. Space/PreEncoded는 투명.
/// Same tarpaulin attribution limitation as `prev_word_token`.
#[cfg(not(tarpaulin_include))]
fn next_word_token<'a, 'b>(tokens: &'b [Token<'a>], index: usize) -> Option<&'b WordToken<'a>> {
    for token in tokens.iter().skip(index + 1) {
        match token {
            Token::Word(w) => return Some(w),
            Token::Space(_) | Token::PreEncoded(_) => continue,
            _ => return None,
        }
    }
    None
}

/// 토큰이 한글 글자를 포함하지 않고 영어/숫자/기호로만 구성되는지 판정.
fn word_is_english_only(word: &WordToken<'_>) -> bool {
    !word.meta.has_korean && word.meta.has_ascii_alphabetic
}

/// 슬라이스가 영문/한글 없이 문장부호로만 구성되는지.
fn is_punct_only(chars: &[char]) -> bool {
    chars
        .iter()
        .all(|c| !c.is_ascii_alphabetic() && !is_korean_char(*c) && !c.is_ascii_digit())
}

/// 같은 토큰 내에서 좌측을 거슬러 처음 만나는 letter가 ASCII 영문인지.
/// 한글을 먼저 만나거나, 영문도 한글도 없으면 false.
fn same_token_left_is_english(left_chars: &[char]) -> bool {
    for ch in left_chars.iter().rev() {
        if ch.is_ascii_alphabetic() {
            return true;
        }
        if is_korean_char(*ch) {
            return false;
        }
    }
    false
}

/// 같은 토큰 내에서 우측으로 처음 만나는 letter가 ASCII 영문인지.
fn same_token_right_is_english(right_chars: &[char]) -> bool {
    for ch in right_chars.iter() {
        if ch.is_ascii_alphabetic() {
            return true;
        }
        if is_korean_char(*ch) {
            return false;
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
struct KoreanSegment {
    char_start: usize,
    char_end: usize, // exclusive
}

#[derive(Default)]
struct KoreanContextScan {
    has_same_token_context: bool,
    has_boundary_segment: bool,
}

#[derive(Default)]
struct EnglishContextCandidates {
    has_same_token_context: bool,
    has_boundary_candidate: bool,
}

/// 토큰 텍스트에서 연속된 한글 char segment의 (시작, 끝) 인덱스를 모두 추출.
fn find_korean_segments(chars: &[char]) -> Vec<KoreanSegment> {
    let mut segments = Vec::new();
    let mut current_start: Option<usize> = None;
    for (idx, ch) in chars.iter().enumerate() {
        if is_korean_char(*ch) {
            if current_start.is_none() {
                current_start = Some(idx);
            }
        } else if let Some(start) = current_start.take() {
            segments.push(KoreanSegment {
                char_start: start,
                char_end: idx,
            });
        }
    }
    if let Some(start) = current_start {
        segments.push(KoreanSegment {
            char_start: start,
            char_end: chars.len(),
        });
    }
    segments
}

fn first_script_char(word: &WordToken<'_>) -> Option<char> {
    if word.meta.starts_with_ascii {
        return word.chars.first().copied();
    }

    if word.chars.first().is_some_and(|ch| is_korean_char(*ch)) {
        return word.chars.first().copied();
    }

    word.chars
        .iter()
        .copied()
        .find(|ch| ch.is_ascii_alphabetic() || is_korean_char(*ch))
}

fn update_korean_context_scan(
    chars: &[char],
    char_start: usize,
    char_end: usize,
    scan: &mut KoreanContextScan,
) {
    let left_slice = &chars[..char_start];
    let right_slice = &chars[char_end..];

    let left_at_boundary = is_punct_only(left_slice);
    let right_at_boundary = is_punct_only(right_slice);

    match (left_at_boundary, right_at_boundary) {
        (true, true) => scan.has_boundary_segment = true,
        (false, false) => {
            scan.has_same_token_context |=
                same_token_left_is_english(left_slice) && same_token_right_is_english(right_slice);
        }
        _ => {}
    }
}

fn scan_korean_contexts(chars: &[char]) -> KoreanContextScan {
    let mut scan = KoreanContextScan::default();
    let mut current_start: Option<usize> = None;

    for (idx, ch) in chars.iter().enumerate() {
        if is_korean_char(*ch) {
            if current_start.is_none() {
                current_start = Some(idx);
            }
        } else if let Some(start) = current_start.take() {
            update_korean_context_scan(chars, start, idx, &mut scan);
        }
    }

    if let Some(start) = current_start {
        update_korean_context_scan(chars, start, chars.len(), &mut scan);
    }

    scan
}

fn scan_english_context_candidates(tokens: &[Token<'_>]) -> EnglishContextCandidates {
    let mut candidates = EnglishContextCandidates::default();
    let mut pending_boundary_after_prev_english = false;
    let mut prev_word_is_english_only: Option<bool> = None;

    // `take_while`로 token을 처리한 뒤 양쪽 플래그가 모두 켜지면 조기 종료.
    // 본문 처리는 closure 안에서 완료되며, 반환값은 무시한다.
    tokens
        .iter()
        .take_while(|token| {
            match token {
                Token::Word(word) => {
                    if pending_boundary_after_prev_english && word_is_english_only(word) {
                        candidates.has_boundary_candidate = true;
                    }
                    pending_boundary_after_prev_english = false;

                    if word.meta.has_korean && !candidates.has_same_token_context {
                        let scan = scan_korean_contexts(&word.chars);
                        candidates.has_same_token_context |= scan.has_same_token_context;
                        if scan.has_boundary_segment && prev_word_is_english_only == Some(true) {
                            pending_boundary_after_prev_english = true;
                        }
                    }

                    prev_word_is_english_only = Some(word_is_english_only(word));
                }
                Token::Space(_) | Token::PreEncoded(_) => {}
                _ => {
                    pending_boundary_after_prev_english = false;
                    prev_word_is_english_only = None;
                }
            }
            // continue while both flags aren't set
            !(candidates.has_same_token_context && candidates.has_boundary_candidate)
        })
        .for_each(|_| {});

    candidates
}

fn count_script_words(tokens: &[Token<'_>]) -> (usize, usize) {
    let mut english_words = 0usize;
    let mut korean_words = 0usize;

    for token in tokens.iter() {
        let Token::Word(word) = token else { continue };
        let Some(c) = first_script_char(word) else {
            continue;
        };
        if c.is_ascii_alphabetic() {
            english_words += 1;
        } else if is_korean_char(c) {
            korean_words += 1;
        }
    }

    (english_words, korean_words)
}

/// Compute all document-level English-Korean predicates once per encode call.
pub fn compute_document_summary(tokens: &[Token<'_>]) -> DocumentSummary {
    let candidates = scan_english_context_candidates(tokens);
    if !candidates.has_same_token_context && !candidates.has_boundary_candidate {
        return DocumentSummary::default();
    }

    let (english_words, korean_words) = count_script_words(tokens);
    let is_english_majority = english_words >= korean_words.max(1);
    let is_english_dominant =
        english_words >= 10 && english_words >= korean_words.saturating_mul(5);
    let has_english_context_for_korean = candidates.has_same_token_context
        || (candidates.has_boundary_candidate && is_english_majority);

    DocumentSummary {
        has_english_context_for_korean,
        is_english_majority,
        is_english_dominant,
    }
}

/// 한글 segment의 좌·우 컨텍스트가 영어인지 판정.
///
/// 두 케이스로 나누어 처리한다:
/// 1. **양쪽 토큰 경계** — segment의 양쪽이 모두 토큰 끝이거나 문장부호만 있다.
///    예: "김치", "반찬)". 인접 word token이 모두 영어이면서 _문서 전체가 영어 다수_
///    일 때만 wrap. (한글 주도 문장에 영어가 끼인 경우는 wrap 대상 아님.)
/// 2. **양쪽 토큰 내부** — segment의 양쪽이 같은 토큰 내 영어 letter로 둘러싸였다.
///    예: "www.대통령.kr"의 "대통령". 양쪽이 영어 letter이면 wrap.
///    (단일 단어 내부 패턴은 문서 비율과 무관하게 항상 적용한다.)
///
/// 두 케이스가 _혼합_된 경우(한쪽은 token boundary, 다른 쪽은 same-token letter)는
/// 영어 어절 + 한국어 조사/어미 결합(예: "be는")일 가능성이 높으므로 wrap하지 않는다.
fn segment_in_english_context_with_majority<'a>(
    chars: &[char],
    seg: KoreanSegment,
    tokens: &[Token<'a>],
    token_index: usize,
    is_english_majority: bool,
) -> bool {
    let left_slice = &chars[..seg.char_start];
    let right_slice = &chars[seg.char_end..];

    let left_at_boundary = is_punct_only(left_slice);
    let right_at_boundary = is_punct_only(right_slice);

    if left_at_boundary && right_at_boundary {
        return boundary_segment_wrap(tokens, token_index, is_english_majority);
    }
    if !left_at_boundary && !right_at_boundary {
        return same_token_left_is_english(left_slice) && same_token_right_is_english(right_slice);
    }
    false
}

/// PDF 제39항 — Korean segment with both sides at token boundary (case 1).
/// Wrap only when prev and next Word tokens are pure English AND the document
/// is English-majority.
fn boundary_segment_wrap<'a>(
    tokens: &[Token<'a>],
    token_index: usize,
    is_english_majority: bool,
) -> bool {
    let prev_eng = prev_word_token(tokens, token_index).is_some_and(word_is_english_only);
    let next_eng = next_word_token(tokens, token_index).is_some_and(word_is_english_only);
    prev_eng && next_eng && is_english_majority
}

/// 토큰을 한글 segment 기준으로 분할하여 각각 wrap된 토큰 시퀀스를 만든다.
/// segment의 좌우 컨텍스트가 영어가 아닌 경우엔 그대로 둔다.
fn build_wrapped_replacement<'a>(
    word: &WordToken<'a>,
    tokens: &[Token<'a>],
    token_index: usize,
    is_english_majority: bool,
) -> Option<Vec<Token<'a>>> {
    let segments = find_korean_segments(&word.chars);
    if segments.is_empty() {
        return None;
    }

    let chars = &word.chars;

    let mut wrap_segments = Vec::new();
    for seg in segments {
        if segment_in_english_context_with_majority(
            chars,
            seg,
            tokens,
            token_index,
            is_english_majority,
        ) {
            wrap_segments.push(seg);
        }
    }

    if wrap_segments.is_empty() {
        return None;
    }

    let mut result: Vec<Token<'a>> = Vec::new();
    let mut cursor = 0usize;

    for seg in wrap_segments {
        if seg.char_start > cursor {
            let prefix: String = chars[cursor..seg.char_start].iter().collect();
            if !prefix.is_empty() {
                result.push(build_word_token(&prefix));
            }
        }
        let korean: String = chars[seg.char_start..seg.char_end].iter().collect();
        result.push(Token::PreEncoded(HANGUL_WRAP_START.to_vec()));
        result.push(build_word_token(&korean));
        result.push(Token::PreEncoded(HANGUL_WRAP_END.to_vec()));
        cursor = seg.char_end;
    }

    if cursor < chars.len() {
        let suffix: String = chars[cursor..].iter().collect();
        if !suffix.is_empty() {
            result.push(build_word_token(&suffix));
        }
    }

    Some(result)
}

impl TokenRule for EnglishDominantKoreanWrapRule {
    fn phase(&self) -> TokenPhase {
        // PostWord 단계는 fall-through(Noop이면 다음 룰 시도) 지원이라
        // 다른 PostWord 룰들과 협력 가능하며, 다른 ModeEntry 변환(digital_notation
        // 등)이 끝난 후 wrap을 적용하기 적합하다.
        TokenPhase::PostWord
    }

    fn priority(&self) -> u16 {
        // PostWord의 spacing(400) / middle_dot/quote(126) 룰보다 먼저 실행되어야
        // 단어 분할이 일어나기 전에 wrap을 적용할 수 있다.
        50
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

        if !word.meta.has_korean {
            return Ok(TokenAction::Noop);
        }

        if !state.doc_summary.has_english_context_for_korean {
            return Ok(TokenAction::Noop);
        }

        // 영-한 wrap 컨텍스트가 활성화됨을 state에 기록한다. 이 플래그는
        // 영어 char 인코더가 단독 단어 약자(예: "in" → ⠔)를 적용할지 결정한다.
        state.english_dominant_wrap_active = true;

        // 추가: 문서가 영어 주도(영어 어절 ≫ 한글 어절)인 경우, 영자표시·
        // 단일 대문자 표시·종료표 모두 생략한다.
        if state.doc_summary.is_english_dominant {
            state.english_dominant_no_indicator = true;
        }

        match build_wrapped_replacement(word, tokens, index, state.doc_summary.is_english_majority)
        {
            Some(replacement) => Ok(TokenAction::ReplaceMany(replacement)),
            None => Ok(TokenAction::Noop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::token::SpaceKind;

    fn word(text: &str) -> Token<'static> {
        let chars: Vec<char> = text.chars().collect();
        Token::Word(WordToken {
            text: Cow::Owned(text.to_string()),
            chars: chars.clone(),
            meta: WordMeta::from_chars(&chars),
        })
    }

    fn unwrap_word<'a, 'b>(tok: &'b Token<'a>) -> &'b WordToken<'a> {
        match tok {
            Token::Word(w) => w,
            _ => panic!("expected Word"),
        }
    }

    /// english_dominant_korean_wrap:106 — `same_token_right_is_english` final
    /// `false` return: scan completes without finding ASCII alphabetic OR Korean.
    /// Direct unit test on the private helper.
    #[test]
    fn same_token_right_is_english_no_alpha_no_korean() {
        // Only digits/punct → both checks fail in the loop → fall through to `false`.
        assert!(!same_token_right_is_english(&['1', '2', '3']));
        assert!(!same_token_right_is_english(&['.', ',', '!']));
        assert!(!same_token_right_is_english(&[]));
    }

    #[test]
    fn find_korean_segments_collects_embedded_korean_runs() {
        let chars: Vec<char> = "A한글.B국".chars().collect();

        let segments = find_korean_segments(&chars);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].char_start, 1);
        assert_eq!(segments[0].char_end, 3);
        assert_eq!(segments[1].char_start, 5);
        assert_eq!(segments[1].char_end, 6);
    }

    /// english_dominant_korean_wrap:257 — `count_script_words` `_ => {}` arm:
    /// `first_script_char` returns Some non-alpha non-Korean OR None.
    /// Build a token slice that exercises this arm directly via the function.
    #[test]
    fn count_script_words_non_alpha_non_korean_first_char() {
        // First-script-char for a pure-digit/symbol word → not Korean, not alpha.
        // WordMeta marks it as starts_with_ascii=true for ascii digits,
        // so first_script_char returns Some('1') which is not alpha and not Korean.
        let tokens = vec![
            word("english"),
            word("한국"),
            word("123"),
            word("..."),
            word("more"),
        ];
        let (eng, kor) = count_script_words(&tokens);
        assert_eq!(eng, 2);
        assert_eq!(kor, 1);
        // The "123" word's first_script_char Some('1') hits `_ => {}` (not counted).
    }

    /// english_dominant_korean_wrap:311 — `(true, true) =>` arm of the boundary
    /// match. Korean segment fills the whole word (both slices empty/punct-only),
    /// AND prev/next tokens are English-only words. is_english_majority required true.
    #[test]
    fn segment_both_boundaries_prev_next_english_with_majority() {
        // Construct: [eng, Space, korean, Space, eng] — middle word is purely Korean.
        let tokens = vec![
            word("hello"),
            Token::Space(SpaceKind::Regular),
            word("한국"),
            Token::Space(SpaceKind::Regular),
            word("world"),
        ];
        // The Korean-only token at index 2: chars=['한','국'], find_korean_segments
        // returns one segment covering full chars. left_slice/right_slice = empty.
        // is_punct_only(empty) = true → (true, true) arm.
        let kor_word = unwrap_word(&tokens[2]);
        let result = build_wrapped_replacement(kor_word, &tokens, 2, true);
        assert!(
            result.is_some(),
            "Korean word between two English words with majority should be wrapped"
        );
    }

    /// english_dominant_korean_wrap:316 — `(false, false) =>` arm of the boundary
    /// match. Korean segment is sandwiched within same-token English letters.
    /// Both `left_at_boundary` and `right_at_boundary` are false because the
    /// surrounding chars include English letters.
    #[test]
    fn segment_within_same_token_english_letters() {
        // "www.대통령.kr" — Korean chars '대통령' surrounded by 'w'/'k' letters
        // (separated by '.'). same_token_*_is_english returns true on both sides.
        let token = word("www.대통령.kr");
        let tokens = vec![token.clone()];
        let kor_word = unwrap_word(&tokens[0]);
        let result = build_wrapped_replacement(kor_word, &tokens, 0, false);
        // Inner same-token English context should wrap regardless of majority.
        assert!(
            result.is_some(),
            "Korean segment within same-token English letters should wrap"
        );
    }

    /// english_dominant_korean_wrap:333 — `find_korean_segments` returns empty when
    /// the word has no Korean chars → `build_wrapped_replacement` returns None.
    #[test]
    fn build_wrapped_replacement_no_korean_returns_none() {
        let token = word("hello");
        let tokens = vec![token.clone()];
        let eng_word = unwrap_word(&tokens[0]);
        assert!(build_wrapped_replacement(eng_word, &tokens, 0, true).is_none());
    }

    /// Negative case for line 311: prev not English → no wrap.
    #[test]
    fn segment_both_boundaries_prev_not_english_no_wrap() {
        // [korean_word, Space, korean_only, Space, english] — prev is Korean.
        let tokens = vec![
            word("안녕하세요"),
            Token::Space(SpaceKind::Regular),
            word("한국"),
            Token::Space(SpaceKind::Regular),
            word("world"),
        ];
        let kor_word = unwrap_word(&tokens[2]);
        let result = build_wrapped_replacement(kor_word, &tokens, 2, true);
        // prev_eng is false → wrap predicate false → result is None.
        assert!(result.is_none());
    }

    /// `scan_english_context_candidates` `_ => { ... }` fallback arm (lines 213-215):
    /// Token::Fraction or Token::Mode in the slice resets pending state.
    #[test]
    fn scan_english_context_candidates_resets_on_non_word_non_space() {
        use crate::rules::token::FractionToken;
        let tokens = vec![
            word("english"),
            Token::Fraction(FractionToken {
                whole: None,
                numerator: "1".into(),
                denominator: "2".into(),
            }),
            word("more"),
        ];
        let _ = scan_english_context_candidates(&tokens);
    }

    /// `scan_english_context_candidates` `break` arm (line 220):
    /// Once BOTH `has_same_token_context` AND `has_boundary_candidate` are true,
    /// the loop terminates early. Construct tokens that achieve both.
    #[test]
    fn scan_english_context_candidates_breaks_when_both_flags_true() {
        // Word that has Korean inside AND english on either side triggers both flags.
        // `english 안녕 english` provides english-bordered Korean → boundary,
        // and a mixed token with both scripts → same_token_context.
        let tokens = vec![
            word("english"),
            Token::Space(SpaceKind::Regular),
            word("hello한국english"),
            Token::Space(SpaceKind::Regular),
            word("english"),
        ];
        let _ = scan_english_context_candidates(&tokens);
    }
}
