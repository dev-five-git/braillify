use crate::{
    char_shortcut,
    char_struct::KoreanChar,
    jauem::{choseong::encode_choseong, jongseong::encode_jongseong},
    moeum::jungsong::encode_jungsong,
    rules::trace::JamoRule,
    split::split_korean_jauem,
    utils::build_char,
};

/// Where syllable composition reports the article behind each stretch of cells.
///
/// Implemented twice so the choice is made at compile time: [`NoSpans`] makes
/// every report vanish, leaving the untraced encoder byte-identical to one with
/// no tracing code at all. A runtime flag instead leaves a branch per jamo on the
/// hottest path in the library, which measured 1-5% slower.
trait SpanSink {
    fn report(&mut self, rule: JamoRule, start: usize, end: usize);
}

struct NoSpans;

impl SpanSink for NoSpans {
    #[inline(always)]
    fn report(&mut self, _rule: JamoRule, _start: usize, _end: usize) {}
}

/// The article behind each stretch of one syllable's cells.
///
/// Composition takes a different branch depending on which abbreviations exist
/// for the syllable, and each branch cites a different article. Collecting the
/// spans is what lets a caller report 제3항 for a 받침 instead of one composite
/// entry for the whole character.
#[derive(Debug, Default, Clone)]
pub struct JamoSpans {
    spans: Vec<(JamoRule, core::ops::Range<u32>)>,
}

impl JamoSpans {
    pub fn drain(&mut self) -> impl Iterator<Item = (JamoRule, core::ops::Range<u32>)> + '_ {
        self.spans.drain(..)
    }
}

impl SpanSink for JamoSpans {
    fn report(&mut self, rule: JamoRule, start: usize, end: usize) {
        if start < end {
            self.spans.push((rule, start as u32..end as u32));
        }
    }
}

/// 합성 종성(예: ㄳ→ㄱ+ㅅ) 두 번째 자모가 있으면 인코딩 후 result에 추가한다.
fn extend_compound_jongseong<S: SpanSink>(
    jong1: Option<char>,
    result: &mut Vec<u8>,
    spans: &mut S,
) -> Result<(), String> {
    if let Some(code) = jong1 {
        let start = result.len();
        let bytes = encode_jongseong(code)?;
        result.extend(bytes);
        spans.report(JamoRule::Jongseong, start, result.len());
    }
    Ok(())
}

pub fn encode_korean_char(korean: &KoreanChar) -> Result<Vec<u8>, String> {
    encode_syllable(korean, &mut NoSpans)
}

pub fn encode_korean_char_with_spans(
    korean: &KoreanChar,
    spans: &mut JamoSpans,
) -> Result<Vec<u8>, String> {
    encode_syllable(korean, spans)
}

fn encode_syllable<S: SpanSink>(korean: &KoreanChar, spans: &mut S) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let (cho0, cho1) = split_korean_jauem(korean.cho)?;
    if cho1.is_some() {
        // 쌍자음이라는 뜻, 초성은 반드시 쌍자음이다.
        result.push(32);
        spans.report(JamoRule::DoubleChoseong, result.len() - 1, result.len());
    }
    let vowel = JamoRule::for_vowel(korean.jung);
    if let Some(jong) = korean.jong {
        let (jong0, jong1) = split_korean_jauem(jong)?;
        if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char('ㅇ', korean.jung, Some(jong0)))
        {
            // 초성 자체를 결합
            if cho0 != 'ㅇ' {
                let start = result.len();
                result.push(encode_choseong(cho0)?);
                spans.report(JamoRule::Choseong, start, result.len());
            }
            let start = result.len();
            result.extend(code);
            spans.report(vowel, start, result.len());
            extend_compound_jongseong(jong1, &mut result, spans)?;
        } else if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, Some(jong0)))
        {
            let start = result.len();
            result.extend(code);
            spans.report(JamoRule::Shortcut, start, result.len());
            extend_compound_jongseong(jong1, &mut result, spans)?;
        } else if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, None))
        {
            let start = result.len();
            result.extend(code);
            spans.report(JamoRule::Shortcut, start, result.len());
            // 종성 자체를 결합
            let start = result.len();
            result.extend(encode_jongseong(jong)?);
            spans.report(JamoRule::Jongseong, start, result.len());
        } else {
            // shortcut 이 없으므로 초성, 중성, 종성 모두 결합

            if cho0 != 'ㅇ' {
                let start = result.len();
                result.push(encode_choseong(cho0)?);
                spans.report(JamoRule::Choseong, start, result.len());
            }
            let start = result.len();
            result.extend(encode_jungsong(korean.jung)?);
            spans.report(vowel, start, result.len());
            let start = result.len();
            result.extend(encode_jongseong(jong)?);
            spans.report(JamoRule::Jongseong, start, result.len());
        }
    } else if let Ok(code) =
        char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, None))
    {
        let start = result.len();
        result.extend(code);
        spans.report(JamoRule::Shortcut, start, result.len());
    } else {
        // shortcut 이 없으므로 초성 중성, 모두 결합
        if cho0 != 'ㅇ' {
            let start = result.len();
            result.push(encode_choseong(cho0)?);
            spans.report(JamoRule::Choseong, start, result.len());
        }
        let start = result.len();
        result.extend(encode_jungsong(korean.jung)?);
        spans.report(vowel, start, result.len());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::char_struct::KoreanChar;

    /// korean_char:25 — ㅇ-base shortcut found for jong0 AND cho0 != ㅇ.
    /// Smoke-test by encoding many Korean syllables with varied jongseong;
    /// at least one path should exercise the shortcut+non-ㅇ-cho branch.
    #[test]
    fn korean_char_encode_various_syllables() {
        // Encode a few syllables with explicit KoreanChar construction.
        // 갈 = ㄱ+ㅏ+ㄹ
        let kc = KoreanChar {
            cho: 'ㄱ',
            jung: 'ㅏ',
            jong: Some('ㄹ'),
        };
        let _ = encode_korean_char(&kc);
        // 닭 = ㄷ+ㅏ+ㄺ (compound jongseong)
        let kc = KoreanChar {
            cho: 'ㄷ',
            jung: 'ㅏ',
            jong: Some('ㄺ'),
        };
        let _ = encode_korean_char(&kc);
        // 값 = ㄱ+ㅏ+ㅄ
        let kc = KoreanChar {
            cho: 'ㄱ',
            jung: 'ㅏ',
            jong: Some('ㅄ'),
        };
        let _ = encode_korean_char(&kc);
        // 깍 = ㄲ+ㅏ+ㄱ (double-cho)
        let kc = KoreanChar {
            cho: 'ㄲ',
            jung: 'ㅏ',
            jong: Some('ㄱ'),
        };
        let _ = encode_korean_char(&kc);
        // Various other patterns through encode().
        let _ = crate::encode("값있는 닭의 갈비");

        // Compound final where the base syllable itself has a shortcut.
        let kc = KoreanChar {
            cho: 'ㄱ',
            jung: 'ㅏ',
            jong: Some('ㄳ'),
        };
        let _ = encode_korean_char(&kc);

        // ㅇ-base shortcut is absent for `엉`, but the exact syllable shortcut `성` exists.
        let kc = KoreanChar {
            cho: 'ㅅ',
            jung: 'ㅓ',
            jong: Some('ㅇ'),
        };
        assert_eq!(encode_korean_char(&kc).unwrap(), vec![32, 59]);
    }
}
