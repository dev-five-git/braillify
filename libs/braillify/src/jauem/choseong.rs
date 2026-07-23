use crate::unicode::decode_unicode;
// choseong map
use phf::phf_map;

pub static CHOSEONG_MAP: phf::Map<char, u8> = phf_map! {
    'ㄱ' => decode_unicode('⠈'),
    'ㄴ' => decode_unicode('⠉'),
    'ㄷ' => decode_unicode('⠊'),
    'ㄹ' => decode_unicode('⠐'),
    'ㅁ' => decode_unicode('⠑'),
    'ㅂ' => decode_unicode('⠘'),
    'ㅅ' => decode_unicode('⠠'),
    // 'ㅇ' => decode_unicode(''), // skip ㅇ of choseong
    'ㅈ' => decode_unicode('⠨'),
    'ㅊ' => decode_unicode('⠰'),
    'ㅋ' => decode_unicode('⠋'),
    'ㅌ' => decode_unicode('⠓'),
    'ㅍ' => decode_unicode('⠙'),
    'ㅎ' => decode_unicode('⠚'),
};

pub fn encode_choseong(text: char) -> Result<u8, String> {
    if let Some(code) = CHOSEONG_MAP.get(&text) {
        Ok(*code)
    } else {
        Err("Invalid Korean choseong character".to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::unicode::decode_unicode;

    use super::*;
    #[rstest::rstest]
    #[case::giyeok('ㄱ', '⠈')]
    #[case::nieun('ㄴ', '⠉')]
    #[case::digeut('ㄷ', '⠊')]
    #[case::rieul('ㄹ', '⠐')]
    #[case::mieum('ㅁ', '⠑')]
    #[case::bieup('ㅂ', '⠘')]
    #[case::siot('ㅅ', '⠠')]
    #[case::jieut('ㅈ', '⠨')]
    #[case::chieut('ㅊ', '⠰')]
    #[case::kieuk('ㅋ', '⠋')]
    #[case::tieut('ㅌ', '⠓')]
    #[case::pieup('ㅍ', '⠙')]
    #[case::hieut('ㅎ', '⠚')]
    pub fn test_encode_choseong(#[case] cho: char, #[case] expected: char) {
        assert_eq!(encode_choseong(cho).unwrap(), decode_unicode(expected));
    }
}
