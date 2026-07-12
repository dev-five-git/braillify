use crate::split;

pub fn build_char(choseong: char, jungseong: char, jongseong: Option<char>) -> char {
    let choseong_list = [
        'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ',
        'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];
    let jongseong_list = [
        '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ',
        'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];
    let choseong_index = choseong_list.iter().position(|&c| c == choseong).unwrap();
    let jungseong_index = jungseong as usize - 0x314F;
    let jongseong_index = if let Some(jongseong) = jongseong {
        jongseong_list.iter().position(|&c| c == jongseong).unwrap()
    } else {
        0
    };
    let hangul_code =
        0xAC00 + (choseong_index * 21 * 28) + (jungseong_index * 28) + jongseong_index;
    char::from_u32(hangul_code as u32).unwrap()
}

pub fn has_choseong_o(ch: char) -> bool {
    if let Ok(split) = split::split_korean_char(ch) {
        return split[0].get_char() == 'ㅇ';
    }
    false
}

pub fn is_korean_char(c: char) -> bool {
    (c as u32 >= 0x3131 && c as u32 <= 0x318E) || (0xAC00 <= c as u32 && c as u32 <= 0xD7A3)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_char() {
        assert_eq!(build_char('ㅇ', 'ㅏ', Some('ㄱ')), '악');
        assert_eq!(build_char('ㅇ', 'ㅏ', Some('ㄴ')), '안');
    }

    #[test]
    fn test_has_choseong_o() {
        assert!(has_choseong_o('ㅇ'));
        assert!(!has_choseong_o('ㄱ'));
        assert!(has_choseong_o('아'));
        assert!(!has_choseong_o('가'));
        assert!(has_choseong_o('앙'));
    }

    #[rstest::rstest]
    #[case::first_jamo('ㄱ', true)]
    #[case::last_jamo('ㆎ', true)]
    #[case::before_jamo('㄰', false)]
    #[case::after_jamo('㆏', false)]
    #[case::first_hangul_syllable('가', true)]
    #[case::last_hangul_syllable('힣', true)]
    #[case::before_hangul_syllable('꯿', false)]
    #[case::after_hangul_syllable('힤', false)]
    fn test_is_korean_char(#[case] input: char, #[case] expected: bool) {
        assert_eq!(is_korean_char(input), expected);
    }
}
