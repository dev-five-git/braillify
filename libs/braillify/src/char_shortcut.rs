use phf::phf_map;

use crate::unicode::decode_unicode;

pub static SHORTCUT_MAP: phf::Map<char, &'static [u8]> = phf_map! {
    '가' => &[decode_unicode('⠫')],
    '나' => &[decode_unicode('⠉')],
    '다' => &[decode_unicode('⠊')],
    '마' => &[decode_unicode('⠑')],
    '바' => &[decode_unicode('⠘')],
    '사' => &[decode_unicode('⠇')],
    '자' => &[decode_unicode('⠨')],
    '카' => &[decode_unicode('⠋')],
    '타' => &[decode_unicode('⠓')],
    '파' => &[decode_unicode('⠙')],
    '하' => &[decode_unicode('⠚')],
    '것' => &[decode_unicode('⠸'), decode_unicode('⠎')],
    '억' => &[decode_unicode('⠹')],
    '언' => &[decode_unicode('⠾')],
    '얼' => &[decode_unicode('⠞')],
    '연' => &[decode_unicode('⠡')],
    '열' => &[decode_unicode('⠳')],
    '영' => &[decode_unicode('⠻')],
    '옥' => &[decode_unicode('⠭')],
    '온' => &[decode_unicode('⠷')],
    '옹' => &[decode_unicode('⠿')],
    '운' => &[decode_unicode('⠛')],
    '울' => &[decode_unicode('⠯')],
    '은' => &[decode_unicode('⠵')],
    '을' => &[decode_unicode('⠮')],
    '인' => &[decode_unicode('⠟')],
    '성' => &[decode_unicode('⠠'), decode_unicode('⠻')],
    '정' => &[decode_unicode('⠨'), decode_unicode('⠻')],
    '청' => &[decode_unicode('⠰'), decode_unicode('⠻')],
};

pub fn encode_char_shortcut(text: char) -> Result<&'static [u8], String> {
    if let Some(code) = SHORTCUT_MAP.get(&text) {
        Ok(code)
    } else {
        Err("Invalid Korean char shortcut".to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// 11개 기본 한글 약자 (가/나/다/마/바/사/자/카/타/파/하) 단일 셀 점역.
    #[rstest::rstest]
    #[case('가', '⠫')]
    #[case('나', '⠉')]
    #[case('다', '⠊')]
    #[case('마', '⠑')]
    #[case('바', '⠘')]
    #[case('사', '⠇')]
    #[case('자', '⠨')]
    #[case('카', '⠋')]
    #[case('타', '⠓')]
    #[case('파', '⠙')]
    #[case('하', '⠚')]
    fn encode_char_shortcut_single_cell(#[case] input: char, #[case] expected: char) {
        assert_eq!(
            encode_char_shortcut(input).unwrap(),
            &[decode_unicode(expected)]
        );
    }

    /// 2-cell 약자 — `것` → `⠸⠎`.
    #[test]
    fn encode_char_shortcut_multi_cell() {
        assert_eq!(
            encode_char_shortcut('것').unwrap(),
            &[decode_unicode('⠸'), decode_unicode('⠎')]
        );
    }
}
