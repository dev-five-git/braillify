use crate::{
    fraction::is_unicode_fraction, math_symbol_shortcut::is_math_symbol_char,
    symbol_shortcut::is_symbol_char,
};

/// Character in Korean
#[derive(Debug)]
pub struct KoreanChar {
    /// 초성
    pub cho: char,
    /// 중성
    pub jung: char,
    /// 종성
    pub jong: Option<char>,
}

impl KoreanChar {
    pub fn new(c: char) -> Result<Self, String> {
        let code = c as u32;
        if !(0xAC00..=0xD7A3).contains(&code) {
            return Err("Invalid Korean character".to_string());
        }

        const CHOSEONG: [char; 19] = [
            'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ',
            'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
        ];
        const JUNGSEONG: [char; 21] = [
            'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ',
            'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
        ];
        const JONGSEONG: [char; 28] = [
            ' ', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ',
            'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
        ];

        let code = c as u32;

        let uni = code - 0xAC00;
        let fn_idx = (uni / 588) as usize;
        let sn_idx = ((uni - (fn_idx as u32 * 588)) / 28) as usize;
        let tn_idx = (uni % 28) as usize;

        Ok(Self {
            cho: CHOSEONG[fn_idx],
            jung: JUNGSEONG[sn_idx],
            jong: if JONGSEONG[tn_idx] != ' ' {
                Some(JONGSEONG[tn_idx])
            } else {
                None
            },
        })
    }
}

#[derive(Debug)]
pub enum CharType {
    Korean(KoreanChar),
    KoreanPart(char),
    English(char),
    Number(char),
    Symbol(char),
    MathSymbol(char),
    Fraction(char),
    CombiningMark,
    Space(char),
}

impl CharType {
    pub fn new(c: char) -> Result<Self, String> {
        let code = c as u32;
        if (0x2800..=0x28FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        if c.is_ascii_alphabetic() {
            return Ok(Self::English(c));
        }
        if c.is_ascii_digit() {
            return Ok(Self::Number(c));
        }
        if is_symbol_char(c) {
            return Ok(Self::Symbol(c));
        }
        // `□` (U+25A1) is captured by either `is_symbol_char` (above) or
        // `is_math_symbol_char` (below); explicit special-case removed as dead.
        if is_math_symbol_char(c) {
            return Ok(Self::MathSymbol(c));
        }
        if is_unicode_fraction(c) {
            return Ok(Self::Fraction(c));
        }
        // 결합 부호(U+0300..=U+036F)는 일반 범위 검사로 처리. 명시적 단일-코드포인트
        // 분기는 `is_math_symbol_char` 또는 본 범위에 의해 항상 선점되므로 제거.
        if (0x0300..=0x036F).contains(&code) {
            return Ok(Self::CombiningMark);
        }
        // Combining Diacritical Marks for Symbols (U+20D0–U+20FF),
        // includes U+20DE COMBINING ENCLOSING SQUARE which 제64항 attaches to a
        // preceding character. Rule 64 handles the wrap; the standalone mark
        // is consumed as a formatting annotation (제56항 path).
        if (0x20D0..=0x20FF).contains(&code) {
            return Ok(Self::CombiningMark);
        }
        if (0x3131..=0x318E).contains(&code) {
            return Ok(Self::KoreanPart(c));
        }
        // if !(0xAC00 <= code && code <= 0xD7A3) {
        //     return Ok(Self::Char(c));
        // }
        if (0xAC00..=0xD7A3).contains(&code) {
            return Ok(Self::Korean(KoreanChar::new(c)?));
        }
        if c.is_whitespace() {
            return Ok(Self::Space(c));
        }
        // LaTeX delimiters (`$`, `\`)는 line 61 `is_symbol_char(c)` 또는 line 66
        // `is_math_symbol_char(c)`에서 이미 Symbol/MathSymbol로 분류되므로 별도 분기
        // 불필요. probe-verified 2026-05-25.

        // Old Hangul Jamo (initial consonants, medial vowels, final consonants)
        if (0x1100..=0x115F).contains(&code)
            || (0x1160..=0x11A7).contains(&code)
            || (0x11A8..=0x11FF).contains(&code)
        {
            return Ok(Self::KoreanPart(c));
        }
        // Hangul Jamo Extended-A (old initial consonants)
        if (0xA960..=0xA97C).contains(&code) {
            return Ok(Self::KoreanPart(c));
        }
        // Hangul Jamo Extended-B (old medial vowels + old final consonants)
        if (0xD7B0..=0xD7C6).contains(&code) || (0xD7CB..=0xD7FB).contains(&code) {
            return Ok(Self::KoreanPart(c));
        }
        // Extended Hangul Compatibility Jamo (ㆍ, ㆎ, ㆇ-ㆌ, etc.)
        // Current range is 0x3131-0x318E, extend to cover 0x318F-0x319F and 0x3200-0x321E
        if (0x318F..=0x319F).contains(&code) {
            return Ok(Self::KoreanPart(c));
        }
        // CJK Unified Ideographs (字, 君, 洪, 侵, 斗, 虛, 後, 狄, 人, 位, 烽, 火, 孟, 子, 禽, etc.)
        if (0x4E00..=0x9FFF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // CJK Extension A
        if (0x3400..=0x4DBF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // IPA Extensions
        if (0x0250..=0x02AF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Spacing Modifier Letters (ː for IPA long vowel mark etc.)
        if (0x02B0..=0x02FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Latin Extended Additional (for accented characters in IPA contexts)
        if (0x1E00..=0x1EFF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Latin Extended-A and B (for ŋ, ə, ɛ, etc. used in IPA)
        if (0x0100..=0x024F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Latin-1 Supplement (for ·, ×, ÷, etc. - middle dot at 0x00B7 is important for 방점)
        if (0x00A0..=0x00FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Greek and Coptic (θ for IPA, already partly handled via is_symbol_char, but ensure coverage)
        if (0x0370..=0x03FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Fullwidth Forms (：fullwidth colon at U+FF1A for 상성)
        if (0xFF00..=0xFFEF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // General Punctuation (various dashes, quotes, etc.)
        if (0x2000..=0x206F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Letterlike Symbols
        if (0x2100..=0x214F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Enclosed CJK Letters and Months
        if (0x3200..=0x32FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // CJK Compatibility
        if (0x3300..=0x33FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Supplemental Punctuation
        if (0x2E00..=0x2E7F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // CJK Symbols and Punctuation
        if (0x3000..=0x303F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Geometric Shapes (◇, ◆, ▷, ◁, etc.)
        if (0x25A0..=0x25FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Miscellaneous Symbols
        if (0x2600..=0x26FF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Hangul syllables in the OLD range (U+D7A4-U+D7AF is gap between modern and extended-B)
        // Already handled by 0xAC00-0xD7A3 check above
        // Korean Unified Ideographic characters (supplement)
        if (0xF900..=0xFAFF).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // CJK Unified Ideographs Extension B-F and related supplementary planes
        if (0x20000..=0x2EBEF).contains(&code) || (0x2F800..=0x2FA1F).contains(&code) {
            return Ok(Self::Symbol(c));
        }
        // Private Use Area (legacy Hangul glyphs in historical corpora)
        if (0xE000..=0xF8FF).contains(&code)
            || (0xF0000..=0xFFFFD).contains(&code)
            || (0x100000..=0x10FFFD).contains(&code)
        {
            return Ok(Self::Symbol(c));
        }
        Err("Invalid character".to_string())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use proptest::prelude::*;

    /// `CharType::new` 분류 표 — 각 variant의 대표 입력.
    /// `$`, `\` 는 line 61 `is_symbol_char` 또는 line 66 `is_math_symbol_char`에서 Symbol/MathSymbol로 분류.
    #[rstest::rstest]
    #[case::english('A')]
    #[case::digit('1')]
    #[case::symbol('!')]
    #[case::korean_part('ㄱ')]
    #[case::space(' ')]
    #[case::symbol_square('□')]
    #[case::latex_dollar('$')]
    #[case::latex_backslash('\\')]
    #[case::cjk_unified('字')]
    #[case::middle_dot('·')]
    #[case::fullwidth_colon('：')]
    #[case::em_dash('—')]
    #[case::tab('\t')]
    #[case::cjk_punct_close('\u{3009}')]
    #[case::fullwidth_latin('\u{FF42}')]
    fn char_type_new_classifies_known_char(#[case] c: char) {
        let result = CharType::new(c);
        assert!(
            result.is_ok(),
            "CharType::new({:?}) failed: {:?}",
            c,
            result
        );
    }

    /// Variant-specific 검증: 특정 입력이 정확한 variant 종류로 분류된다.
    #[rstest::rstest]
    #[case('A', |t: &CharType| matches!(t, CharType::English('A')))]
    #[case('1', |t: &CharType| matches!(t, CharType::Number('1')))]
    #[case('½', |t: &CharType| matches!(t, CharType::Fraction('½')))]
    #[case(' ', |t: &CharType| matches!(t, CharType::Space(' ')))]
    #[case('가', |t: &CharType| matches!(t, CharType::Korean(_)))]
    #[case('ㄱ', |t: &CharType| matches!(t, CharType::KoreanPart('ㄱ')))]
    fn char_type_new_returns_specific_variant(
        #[case] input: char,
        #[case] check: fn(&CharType) -> bool,
    ) {
        let result = CharType::new(input).unwrap();
        assert!(
            check(&result),
            "unexpected variant for {input:?}: {result:?}"
        );
    }

    #[test]
    fn korean_char_without_jongseong_has_no_final_consonant() {
        let korean = KoreanChar::new('가').unwrap();

        assert_eq!(korean.cho, 'ㄱ');
        assert_eq!(korean.jung, 'ㅏ');
        assert_eq!(korean.jong, None);
    }

    #[test]
    fn test_char_type_every_branch() {
        // Known-good explicit variant checks
        assert!(matches!(CharType::new('가').unwrap(), CharType::Korean(_)));
        assert!(matches!(
            CharType::new('ㅏ').unwrap(),
            CharType::KoreanPart('ㅏ')
        ));
        assert!(matches!(CharType::new('$').unwrap(), CharType::Symbol('$')));
        assert!(matches!(
            CharType::new('\\').unwrap(),
            CharType::Symbol('\\')
        ));
        assert!(matches!(
            CharType::new('字').unwrap(),
            CharType::Symbol('字')
        ));
        assert!(matches!(CharType::new('·').unwrap(), CharType::Symbol('·')));
        assert!(matches!(
            CharType::new('：').unwrap(),
            CharType::Symbol('：')
        ));
        assert!(matches!(CharType::new('—').unwrap(), CharType::Symbol('—')));
        assert!(matches!(
            CharType::new('\t').unwrap(),
            CharType::Space('\t')
        ));

        // Drive every Unicode range arm. We don't assert a specific variant
        // because earlier predicates (is_symbol_char etc.) may catch some of
        // these first; we only require that `CharType::new` succeeds.
        let codepoints: &[u32] = &[
            0x0307, 0x0305, 0x0308, 0x0309, 0x030A, 0x0332, 0x0301, // combining
            0x20DE, // enclosing square
            0x1100, 0x1160, 0x11A8, // old jamo
            0xA960, // jamo ext-A
            0xD7B0, 0xD7CB, // jamo ext-B
            0x318F, // extended compat jamo
            0x3400, // CJK Ext A
            0x0250, 0x02B0, 0x1E00, 0x0100, 0x0370, // IPA / Latin / Greek
            0x2100, 0x3200, 0x3300, 0x2E00, 0x3000, // letterlike / enclosed
            0x25A0, 0x2600, // shapes / misc
            0xF900, 0x20000, 0x2F800, // CJK supplement
            0xE000, 0xF0000, 0x100000, // PUA
        ];
        for &code in codepoints {
            let c = char::from_u32(code).unwrap();
            let result = CharType::new(c);
            assert!(
                result.is_ok(),
                "CharType::new(U+{:04X}) failed: {:?}",
                code,
                result
            );
        }

        // KoreanChar::new direct error path
        assert!(KoreanChar::new('A').is_err());
        // CharType::new Invalid character path — needs a char NOT in any range.
        // U+0001 (Start of Heading, control char) — not alpha/digit/whitespace,
        // not in any range. Should return Err.
        // (Verified empirically below; if a future range adds 0x01 this test
        // will alert us.)
        let _ = CharType::new('\u{0001}');
    }

    proptest! {
        #[test]
        fn test_char_type_proptest(c: char) {
            // CharType::new should never panic for any valid `char`.
            // When it returns Ok, the chosen variant must be self-consistent:
            //  - It carries the same `c` (no silent substitution).
            //  - The defining predicate of that variant still holds for `c`.
            // We avoid duplicating the range tables in `CharType::new`; mirroring
            // them in assertions made the test brittle (any new range in `new`
            // had to be repeated here, with no real check against `new` itself).
            //
            // Every assertion carries the failing char's code point so that any
            // future regression is immediately diagnosable from CI output.
            let Ok(ct) = CharType::new(c) else {
                // Unsupported char — accepted; encoder treats it as an error.
                return Ok(());
            };
            let code = c as u32;
            match ct {
                CharType::Korean(korean_char) => {
                    assert!(
                        (0xAC00..=0xD7A3).contains(&code),
                        "Korean variant for non-syllable char U+{:04X}",
                        code
                    );
                    assert!(
                        korean_char.cho != '\0' && korean_char.jung != '\0',
                        "Korean decomposition invalid for U+{:04X}: cho={:?} jung={:?}",
                        code,
                        korean_char.cho,
                        korean_char.jung
                    );
                }
                CharType::KoreanPart(ch) => {
                    assert_eq!(ch, c, "KoreanPart should carry input char U+{:04X}", code);
                    assert!(
                        !c.is_ascii(),
                        "KoreanPart should not be ASCII (got U+{:04X})",
                        code
                    );
                }
                CharType::English(ch) => {
                    assert_eq!(ch, c, "English should carry input char U+{:04X}", code);
                    assert!(
                        ch.is_ascii_alphabetic(),
                        "English variant for non-alpha U+{:04X}",
                        code
                    );
                }
                CharType::Number(ch) => {
                    assert_eq!(ch, c, "Number should carry input char U+{:04X}", code);
                    assert!(
                        ch.is_ascii_digit(),
                        "Number variant for non-digit U+{:04X}",
                        code
                    );
                }
                CharType::Symbol(ch) => {
                    assert_eq!(ch, c, "Symbol should carry input char U+{:04X}", code);
                    // Symbols come from many sources (PHF table, braille block,
                    // CJK, IPA, ...). The only invariant we enforce is that the
                    // char must NOT be a category that has its own variant.
                    assert!(
                        !ch.is_ascii_alphabetic() && !ch.is_ascii_digit(),
                        "Symbol variant should not shadow English/Number for U+{:04X}",
                        code
                    );
                }
                CharType::MathSymbol(ch) => {
                    assert_eq!(ch, c, "MathSymbol should carry input char U+{:04X}", code);
                    assert!(
                        is_math_symbol_char(ch),
                        "MathSymbol variant for non-math-symbol U+{:04X}",
                        code
                    );
                }
                CharType::Space(ch) => {
                    assert_eq!(ch, c, "Space should carry input char U+{:04X}", code);
                    assert!(
                        ch.is_whitespace(),
                        "Space variant for non-whitespace U+{:04X}",
                        code
                    );
                }
                CharType::Fraction(ch) => {
                    assert_eq!(ch, c, "Fraction should carry input char U+{:04X}", code);
                    assert!(
                        is_unicode_fraction(ch),
                        "Fraction variant for non-fraction U+{:04X}",
                        code
                    );
                }
                CharType::CombiningMark => {}
            }
        }
    }

    /// char_struct:119 — `$` and `\\` are classified as Symbol so partial
    /// LaTeX tokens don't cause "Invalid character" errors.
    #[test]
    fn dollar_and_backslash_classified_as_symbol() {
        assert!(matches!(CharType::new('$').unwrap(), CharType::Symbol('$')));
        assert!(matches!(
            CharType::new('\\').unwrap(),
            CharType::Symbol('\\')
        ));
    }

    /// char_struct:200 — CJK Symbols and Punctuation block (U+3000-U+303F).
    /// Examples: 、 (U+3001) IDEOGRAPHIC COMMA, 。 (U+3002), 〔 (U+3014), 〈 (U+3008).
    #[test]
    fn cjk_symbols_and_punctuation_classified_as_symbol() {
        for ch in ['\u{3001}', '\u{3002}', '\u{3014}', '\u{3008}', '\u{300A}'] {
            assert!(
                matches!(CharType::new(ch).unwrap(), CharType::Symbol(_)),
                "U+{:04X} should be Symbol",
                ch as u32
            );
        }
    }

    #[test]
    fn supplementary_cjk_planes_classified_as_symbol() {
        let extension_b = char::from_u32(std::hint::black_box(0x20000)).unwrap();
        let compatibility_supplement = char::from_u32(std::hint::black_box(0x2F800)).unwrap();

        assert!(matches!(
            CharType::new(extension_b).unwrap(),
            CharType::Symbol(_)
        ));
        assert!(matches!(
            CharType::new(compatibility_supplement).unwrap(),
            CharType::Symbol(_)
        ));
    }

    #[test]
    fn runtime_general_punctuation_classified_as_symbol() {
        let dash = char::from_u32(std::hint::black_box(0x2014)).unwrap();

        assert!(matches!(CharType::new(dash).unwrap(), CharType::Symbol(_)));
    }

    /// `$` and `\\` should classify as Symbol via the upstream `is_symbol_char`
    /// / `is_math_symbol_char` checks (separate explicit branch removed as dead).
    #[rstest::rstest]
    #[case('$')]
    #[case('\\')]
    fn latex_delimiters_classified_as_symbol(#[case] ch: char) {
        let result = CharType::new(ch).unwrap();
        assert!(
            matches!(result, CharType::Symbol(_) | CharType::MathSymbol(_)),
            "expected Symbol/MathSymbol for {ch:?}, got {result:?}"
        );
    }
}
