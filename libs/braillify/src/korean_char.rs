use crate::{
    char_shortcut,
    char_struct::KoreanChar,
    jauem::{choseong::encode_choseong, jongseong::encode_jongseong},
    moeum::jungsong::encode_jungsong,
    split::split_korean_jauem,
    utils::build_char,
};

/// 합성 종성(예: ㄳ→ㄱ+ㅅ) 두 번째 자모가 있으면 인코딩 후 result에 추가한다.
fn extend_compound_jongseong(jong1: Option<char>, result: &mut Vec<u8>) -> Result<(), String> {
    if let Some(code) = jong1 {
        let bytes = encode_jongseong(code)?;
        result.extend(bytes);
    }
    Ok(())
}

pub fn encode_korean_char(korean: &KoreanChar) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let (cho0, cho1) = split_korean_jauem(korean.cho)?;
    if cho1.is_some() {
        // 쌍자음이라는 뜻, 초성은 반드시 쌍자음이다.
        result.push(32);
    }
    if let Some(jong) = korean.jong {
        let (jong0, jong1) = split_korean_jauem(jong)?;
        if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char('ㅇ', korean.jung, Some(jong0)))
        {
            // 초성 자체를 결합
            if cho0 != 'ㅇ' {
                result.push(encode_choseong(cho0)?);
            }
            result.extend(code);
            extend_compound_jongseong(jong1, &mut result)?;
        } else if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, Some(jong0)))
        {
            result.extend(code);
            extend_compound_jongseong(jong1, &mut result)?;
        } else if let Ok(code) =
            char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, None))
        {
            result.extend(code);
            // 종성 자체를 결합
            result.extend(encode_jongseong(jong)?);
        } else {
            // shortcut 이 없으므로 초성, 중성, 종성 모두 결합

            if cho0 != 'ㅇ' {
                result.push(encode_choseong(cho0)?);
            }
            result.extend(encode_jungsong(korean.jung)?);
            result.extend(encode_jongseong(jong)?);
        }
    } else if let Ok(code) =
        char_shortcut::encode_char_shortcut(build_char(cho0, korean.jung, None))
    {
        result.extend(code);
    } else {
        // shortcut 이 없으므로 초성 중성, 모두 결합
        if cho0 != 'ㅇ' {
            result.push(encode_choseong(cho0)?);
        }
        result.extend(encode_jungsong(korean.jung)?);
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
