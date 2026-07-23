use phf::phf_map;

use crate::{moeum::jungsong::JUNGSEONG_MAP, unicode::decode_unicode};

pub static KOREAN_PART_MAP: phf::Map<char, &'static [u8]> = phf_map! {
    'ㄱ' => &[decode_unicode('⠁')],
    'ㄲ' => &[decode_unicode('⠁'), decode_unicode('⠁')],
    'ㄳ' => &[decode_unicode('⠁'), decode_unicode('⠄')],
    'ㄴ' => &[decode_unicode('⠒')],
    'ㄵ' => &[decode_unicode('⠒'), decode_unicode('⠅')],
    'ㄶ' => &[decode_unicode('⠒'), decode_unicode('⠴')],
    'ㄷ' => &[decode_unicode('⠔')],
    'ㄸ' => &[decode_unicode('⠔'), decode_unicode('⠔')],
    'ㄹ' => &[decode_unicode('⠂')],
    'ㄺ' => &[decode_unicode('⠂'), decode_unicode('⠁')],
    'ㄻ' => &[decode_unicode('⠂'), decode_unicode('⠢')],
    'ㄼ' => &[decode_unicode('⠂'), decode_unicode('⠃')],
    'ㄽ' => &[decode_unicode('⠂'), decode_unicode('⠄')],
    'ㄾ' => &[decode_unicode('⠂'), decode_unicode('⠦')],
    'ㄿ' => &[decode_unicode('⠂'), decode_unicode('⠲')],
    'ㅀ' => &[decode_unicode('⠂'), decode_unicode('⠴')],
    'ㅁ' => &[decode_unicode('⠢')],
    'ㅂ' => &[decode_unicode('⠃')],
    'ㅃ' => &[decode_unicode('⠃'), decode_unicode('⠃')],
    'ㅄ' => &[decode_unicode('⠃'), decode_unicode('⠄')],
    'ㅅ' => &[decode_unicode('⠄')],
    'ㅆ' => &[decode_unicode('⠄'), decode_unicode('⠄')],
    'ㅇ' => &[decode_unicode('⠶')],
    'ㅈ' => &[decode_unicode('⠅')],
    'ㅉ' => &[decode_unicode('⠅'), decode_unicode('⠅')],
    'ㅊ' => &[decode_unicode('⠆')],
    'ㅋ' => &[decode_unicode('⠖')],
    'ㅌ' => &[decode_unicode('⠦')],
    'ㅍ' => &[decode_unicode('⠲')],
    'ㅎ' => &[decode_unicode('⠴')],
};

/// 제8항 자음자나 모음자가 단독으로 쓰일 때에는 해당 글자 앞에 온표 =을 적어 나타내며, 자음자는 받침으로 적는다
pub fn encode_korean_part(text: char) -> Result<&'static [u8], String> {
    if let Some(code) = KOREAN_PART_MAP.get(&text) {
        return Ok(code);
    }
    if let Some(code) = JUNGSEONG_MAP.get(&text) {
        return Ok(code);
    }
    Err("Invalid Korean part character".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_korean_part_input() {
        assert!(encode_korean_part('a').is_err());
        assert!(encode_korean_part('1').is_err());
        assert!(encode_korean_part('!').is_err());
    }

    #[test]
    fn accepts_jongseong_jamo() {
        assert!(encode_korean_part('ㄱ').is_ok());
        assert!(encode_korean_part('ㅎ').is_ok());
    }

    #[test]
    fn accepts_jungseong_jamo() {
        assert!(encode_korean_part('ㅏ').is_ok());
        assert!(encode_korean_part('ㅣ').is_ok());
    }
}
