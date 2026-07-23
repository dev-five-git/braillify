use phf::phf_map;

#[derive(Debug, PartialEq)]
pub enum KoreanChar {
    Choseong(char),
    Jongseong(char),
    Jungseong(char),
}
impl KoreanChar {
    pub fn get_char(&self) -> char {
        match self {
            KoreanChar::Choseong(c) => *c,
            KoreanChar::Jongseong(c) => *c,
            KoreanChar::Jungseong(c) => *c,
        }
    }
}

// ㄱ, ㄲ, ㄳ, ㄴ, ㄵ, ㄶ, ㄷ, ㄸ, ㄹ, ㄺ, ㄻ, ㄼ, ㄽ, ㄾ, ㄿ, ㅀ, ㅁ, ㅂ, ㅃ, ㅄ, ㅅ, ㅆ, ㅇ, ㅈ, ㅉ, ㅊ, ㅋ, ㅌ, ㅍ, ㅎ
pub static KOREAN_JAUEM_MAP: phf::Map<char, (char, Option<char>)> = phf_map! {
    'ㄱ' => ('ㄱ', None),
    'ㄲ' => ('ㄱ', Some('ㄱ')),
    'ㄳ' => ('ㄱ', Some('ㅅ')),
    'ㄴ' => ('ㄴ', None),
    'ㄵ' => ('ㄴ', Some('ㅈ')),
    'ㄶ' => ('ㄴ', Some('ㅎ')),
    'ㄷ' => ('ㄷ', None),
    'ㄸ' => ('ㄷ', Some('ㄷ')),
    'ㄹ' => ('ㄹ', None),
    'ㄺ' => ('ㄹ', Some('ㄱ')),
    'ㄻ' => ('ㄹ', Some('ㅁ')),
    'ㄼ' => ('ㄹ', Some('ㅂ')),
    'ㄽ' => ('ㄹ', Some('ㅅ')),
    'ㄾ' => ('ㄹ', Some('ㅌ')),
    'ㄿ' => ('ㄹ', Some('ㅍ')),
    'ㅀ' => ('ㄹ', Some('ㅎ')),
    'ㅁ' => ('ㅁ', None),
    'ㅂ' => ('ㅂ', None),
    'ㅃ' => ('ㅂ', Some('ㅂ')),
    'ㅄ' => ('ㅂ', Some('ㅅ')),
    'ㅅ' => ('ㅅ', None),
    'ㅆ' => ('ㅅ', Some('ㅅ')),
    'ㅇ' => ('ㅇ', None),
    'ㅈ' => ('ㅈ', None),
    'ㅉ' => ('ㅈ', Some('ㅈ')),
    'ㅊ' => ('ㅊ', None),
    'ㅋ' => ('ㅋ', None),
    'ㅌ' => ('ㅌ', None),
    'ㅍ' => ('ㅍ', None),
    'ㅎ' => ('ㅎ', None),
};

/// 자음을 분리합니다.
pub fn split_korean_jauem(text: char) -> Result<(char, Option<char>), String> {
    if let Some((cho, jong)) = KOREAN_JAUEM_MAP.get(&text) {
        return Ok((*cho, *jong));
    }
    Err("Invalid Korean character".to_string())
}
pub fn split_korean_char(text: char) -> Result<Vec<KoreanChar>, String> {
    // check korean char
    let code = text as u32;
    if (0x3131..=0x314E).contains(&code) {
        return Ok(vec![KoreanChar::Choseong(text)]);
    }
    if (0x314F..=0x3163).contains(&code) {
        return Ok(vec![KoreanChar::Jungseong(text)]);
    }
    if !(0xAC00..=0xD7A3).contains(&code) {
        return Err("Invalid Korean character".to_string());
    }

    const CHOSEONG: [char; 19] = [
        'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ',
        'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];
    const JUNGSEONG: [char; 21] = [
        'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ',
        'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
    ];
    const JONGSEONG: [char; 28] = [
        ' ', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ',
        'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];

    let code = text as u32;

    let uni = code - 0xAC00;
    let fn_idx = (uni / 588) as usize;
    let sn_idx = ((uni - (fn_idx as u32 * 588)) / 28) as usize;
    let tn_idx = (uni % 28) as usize;

    let mut result = Vec::new();
    result.push(KoreanChar::Choseong(CHOSEONG[fn_idx]));
    result.push(KoreanChar::Jungseong(JUNGSEONG[sn_idx]));
    if JONGSEONG[tn_idx] != ' ' {
        result.push(KoreanChar::Jongseong(JONGSEONG[tn_idx]));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 완전한 음절 (초성 + 중성 + 종성) 분해.
    #[rstest::rstest]
    #[case('강', 'ㄱ', 'ㅏ', Some('ㅇ'))]
    #[case('한', 'ㅎ', 'ㅏ', Some('ㄴ'))]
    #[case('글', 'ㄱ', 'ㅡ', Some('ㄹ'))]
    #[case('안', 'ㅇ', 'ㅏ', Some('ㄴ'))]
    #[case('녕', 'ㄴ', 'ㅕ', Some('ㅇ'))]
    #[case('나', 'ㄴ', 'ㅏ', None)]
    #[case('라', 'ㄹ', 'ㅏ', None)]
    fn split_korean_char_full_syllable(
        #[case] input: char,
        #[case] cho: char,
        #[case] jung: char,
        #[case] jong: Option<char>,
    ) {
        let mut expected = vec![KoreanChar::Choseong(cho), KoreanChar::Jungseong(jung)];
        if let Some(j) = jong {
            expected.push(KoreanChar::Jongseong(j));
        }
        assert_eq!(split_korean_char(input), Ok(expected));
    }

    /// 단일 초성(자음) 분해.
    #[rstest::rstest]
    #[case('ㄱ')]
    #[case('ㄴ')]
    #[case('ㄷ')]
    #[case('ㅁ')]
    #[case('ㅂ')]
    #[case('ㅅ')]
    #[case('ㅈ')]
    #[case('ㅊ')]
    #[case('ㅋ')]
    #[case('ㅌ')]
    #[case('ㅍ')]
    #[case('ㅎ')]
    fn split_korean_char_single_choseong(#[case] c: char) {
        assert_eq!(split_korean_char(c), Ok(vec![KoreanChar::Choseong(c)]));
    }

    /// 단일 중성(모음) 분해.
    #[rstest::rstest]
    #[case('ㅏ')]
    #[case('ㅐ')]
    #[case('ㅑ')]
    #[case('ㅒ')]
    #[case('ㅓ')]
    #[case('ㅔ')]
    #[case('ㅕ')]
    #[case('ㅖ')]
    #[case('ㅗ')]
    #[case('ㅘ')]
    #[case('ㅙ')]
    #[case('ㅚ')]
    #[case('ㅛ')]
    #[case('ㅜ')]
    #[case('ㅝ')]
    #[case('ㅞ')]
    #[case('ㅟ')]
    #[case('ㅠ')]
    #[case('ㅡ')]
    #[case('ㅢ')]
    #[case('ㅣ')]
    fn split_korean_char_single_jungseong(#[case] c: char) {
        assert_eq!(split_korean_char(c), Ok(vec![KoreanChar::Jungseong(c)]));
    }

    /// 한글이 아닌 문자는 에러.
    #[rstest::rstest]
    #[case('a')]
    #[case('1')]
    fn split_korean_char_non_korean_returns_err(#[case] c: char) {
        assert_eq!(
            split_korean_char(c),
            Err("Invalid Korean character".to_string())
        );
    }

    /// 각 `KoreanChar` variant의 `get_char` 분기.
    #[rstest::rstest]
    #[case(KoreanChar::Choseong('ㄱ'), 'ㄱ')]
    #[case(KoreanChar::Jungseong('ㅏ'), 'ㅏ')]
    #[case(KoreanChar::Jongseong('ㄴ'), 'ㄴ')]
    fn korean_char_get_char_all_variants(#[case] kc: KoreanChar, #[case] expected: char) {
        assert_eq!(kc.get_char(), expected);
    }

    /// split.rs line 58 - Err arm when char isn't in KOREAN_JAUEM_MAP.
    #[rstest::rstest]
    #[case('A')]
    #[case('1')]
    fn split_korean_jauem_returns_err_for_non_jamo(#[case] ch: char) {
        assert!(split_korean_jauem(ch).is_err());
    }
}
