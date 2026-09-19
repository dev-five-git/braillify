//! 한자의 한국어 음독 표.
//!
//! 한국 점자는 국어 문장 안의 한자를 그 한자의 한국어 독음으로 적는다. NIKL 병렬
//! 말뭉치의 참조도 같다 — `플로리다주(州)로` → `⠙⠮⠐⠥⠐⠕⠊⠨⠍⠦⠄⠨⠍⠠⠴⠐⠥`(주(주)로),
//! `중부내륙지선(支線)의` → 지선, `지천명(知天命)의` → 지천명.
//!
//! 표는 Unicode Unihan 데이터베이스의 `kHangul` 속성에서 뽑았다. 한 한자에 독음이
//! 여럿이면 출처 코드 우선순위(KS X 1001 > KS X 1002 > 교육용 기초 한자 > 인명용
//! 한자)로 하나를 고른다.
//!
//! 한계: `kHangul`은 낱자 단위 독음이라 문맥에 따라 독음이 갈리는 한자(車 = 차/거)나
//! 두음 법칙(李 = 리/이)은 반영하지 못한다.
//!
//! 자료 출처: Unicode® Unihan Database, © 1991–2026 Unicode, Inc.
//! Unicode License v3 (<https://www.unicode.org/license.txt>).

use std::collections::HashMap;
use std::sync::LazyLock;

/// `<한자>\t<독음>` 한 줄씩.
static READINGS: &str = include_str!("../resources/hanja-readings.txt");

static INDEX: LazyLock<HashMap<char, &'static str>> = LazyLock::new(|| {
    READINGS
        .lines()
        .filter_map(|line| {
            let (hanja, reading) = line.split_once('\t')?;
            let mut chars = hanja.chars();
            let hanja = chars.next()?;
            (chars.next().is_none() && !reading.is_empty()).then_some((hanja, reading))
        })
        .collect()
});

/// True iff `c` sits in a Unicode block that holds Han ideographs.
pub fn is_hanja(c: char) -> bool {
    matches!(c,
        '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{20000}'..='\u{2A6DF}'
        | '\u{2F800}'..='\u{2FA1F}')
}

/// 한자의 한국어 독음. 표에 없으면 `None`.
pub fn reading(c: char) -> Option<&'static str> {
    INDEX.get(&c).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::state('州', "주")]
    #[case::before('前', "전")]
    #[case::late('故', "고")]
    #[case::branch('支', "지")]
    #[case::line('線', "선")]
    #[case::fragrance('香', "향")]
    #[case::river('川', "천")]
    #[case::present('現', "현")]
    fn reads_corpus_hanja(#[case] hanja: char, #[case] expected: &str) {
        assert_eq!(reading(hanja), Some(expected));
    }

    #[test]
    fn hangul_and_latin_are_not_hanja() {
        assert!(!is_hanja('가'));
        assert!(!is_hanja('A'));
        assert!(is_hanja('州'));
    }

    /// 문장 안의 한자는 독음으로 적힌다 (`支線` → 지선).
    #[test]
    fn hanja_in_a_sentence_uses_its_reading() {
        assert_eq!(
            crate::encode_to_unicode("지선(支線)"),
            crate::encode_to_unicode("지선(지선)")
        );
    }
}
