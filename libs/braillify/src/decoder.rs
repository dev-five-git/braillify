/// Simple Korean braille decoder (점자 → 한글).
/// Handles basic Korean syllables, char shortcuts (가나다…), and abbreviated
/// syllable shortcuts (을 억 언 …). Numbers, English, and word shortcuts are
/// not yet handled.
pub fn decode(braille: &str) -> Result<String, String> {
    let bytes: Vec<u8> = braille
        .chars()
        .filter_map(|c| {
            let code = c as u32;
            if (0x2800..=0x28FF).contains(&code) {
                Some((code - 0x2800) as u8)
            } else if c == ' ' || c == '\n' {
                Some(0)
            } else {
                None
            }
        })
        .collect();

    decode_bytes(&bytes)
}

fn decode_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut result = String::new();
    let mut i = 0;

    enum Stage {
        Start,
        GotCho(u32),
        GotJung(u32, u32), // (cho_idx, jung_idx)
    }

    let mut stage = Stage::Start;

    while i < bytes.len() {
        let b = bytes[i];

        match stage {
            // ─── Start: no pending syllable ──────────────────────────────
            Stage::Start => {
                if b == 0 {
                    result.push(' ');
                    i += 1;
                } else if let Some((ch, len)) = try_two_byte_shortcut(bytes, i) {
                    result.push(ch);
                    i += len;
                } else if let Some((jung, jong)) = try_abbreviated_shortcut(b) {
                    // standalone 을/억/언/… → ㅇ초성 + jung + jong
                    result.push(build_syllable(11, jung, jong));
                    i += 1;
                } else if let Some(cho) = try_consonant_a_shortcut(b) {
                    // 가(43) or 사(7) — need to collect possible jongseong
                    stage = Stage::GotJung(cho, 0); // jung=0 = ㅏ
                    i += 1;
                } else if let Some((jung, consumed)) = try_jungseong(bytes, i) {
                    stage = Stage::GotJung(11, jung); // silent ㅇ
                    i += consumed;
                } else if let Some((cho, consumed)) = try_choseong_with_double(bytes, i) {
                    stage = Stage::GotCho(cho);
                    i += consumed;
                } else {
                    i += 1; // skip unknown byte
                }
            }

            // ─── GotCho: have initial consonant, waiting for vowel ────────
            Stage::GotCho(cho) => {
                if let Some((jung, consumed)) = try_jungseong(bytes, i) {
                    stage = Stage::GotJung(cho, jung);
                    i += consumed;
                } else if let Some((jung, jong)) = try_abbreviated_shortcut(b) {
                    // e.g. 글 = ㄱ(8) + 을(46) → cho + ㅡ + ㄹ
                    result.push(build_syllable(cho, jung, jong));
                    stage = Stage::Start;
                    i += 1;
                } else if b == 0 {
                    // consonant followed immediately by space → standalone jamo
                    result.push(cho_idx_to_jamo(cho));
                    result.push(' ');
                    stage = Stage::Start;
                    i += 1;
                } else {
                    // No vowel followed — this choseong byte doubles as a
                    // 나/다/마/바/자/카/타/파/하 shortcut (implicit ㅏ vowel).
                    // Don't consume the current byte; let GotJung handle it.
                    stage = Stage::GotJung(cho, 0); // jung=0 = ㅏ
                }
            }

            // ─── GotJung: have cho+jung, looking for jongseong ───────────
            Stage::GotJung(cho, jung) => {
                // Priority 1: two-byte shortcuts (성 정 청 것) — check before jongseong
                // because their first bytes (32,40,48,56) can also start jongseong sequences.
                if let Some((ch, len)) = try_two_byte_shortcut(bytes, i) {
                    result.push(build_syllable(cho, jung, 0));
                    result.push(ch);
                    stage = Stage::Start;
                    i += len;
                }
                // Priority 2: jongseong
                else if let (jong @ 1.., len) = try_jongseong(bytes, i) {
                    result.push(build_syllable(cho, jung, jong));
                    stage = Stage::Start;
                    i += len;
                }
                // Priority 3: space
                else if b == 0 {
                    result.push(build_syllable(cho, jung, 0));
                    result.push(' ');
                    stage = Stage::Start;
                    i += 1;
                }
                // Priority 4: abbreviated shortcut byte starts a new syllable
                // (e.g. "나을" — emit 나, then 을 as ㅇ+ㅡ+ㄹ)
                else if let Some((jung2, jong2)) = try_abbreviated_shortcut(b) {
                    result.push(build_syllable(cho, jung, 0));
                    result.push(build_syllable(11, jung2, jong2));
                    stage = Stage::Start;
                    i += 1;
                }
                // Priority 5: 가(43) or 사(7) shortcut starts a new syllable
                else if let Some(cho2) = try_consonant_a_shortcut(b) {
                    result.push(build_syllable(cho, jung, 0));
                    stage = Stage::GotJung(cho2, 0);
                    i += 1;
                }
                // Priority 6: choseong starts next syllable (incl. tense consonants)
                else if let Some((new_cho, consumed)) = try_choseong_with_double(bytes, i) {
                    result.push(build_syllable(cho, jung, 0));
                    stage = Stage::GotCho(new_cho);
                    i += consumed;
                }
                // Priority 7: consecutive vowel → emit current, silent ㅇ + new vowel
                else if let Some((new_jung, consumed)) = try_jungseong(bytes, i) {
                    result.push(build_syllable(cho, jung, 0));
                    stage = Stage::GotJung(11, new_jung);
                    i += consumed;
                }
                // Unknown byte
                else {
                    result.push(build_syllable(cho, jung, 0));
                    stage = Stage::Start;
                    i += 1;
                }
            }
        }
    }

    // Flush remaining state
    match stage {
        Stage::GotCho(cho) => result.push(build_syllable(cho, 0, 0)), // implicit ㅏ shortcut
        Stage::GotJung(cho, jung) => result.push(build_syllable(cho, jung, 0)),
        Stage::Start => {}
    }

    Ok(result.trim().to_string())
}

// ── Two-byte shortcuts ─────────────────────────────────────────────────────
// 성/정/청/것 are conventional pairings — NOT phonetically composed.
fn try_two_byte_shortcut(bytes: &[u8], i: usize) -> Option<(char, usize)> {
    let b0 = *bytes.get(i)?;
    let b1 = bytes.get(i + 1).copied()?;
    match (b0, b1) {
        (32, 59) => Some(('성', 2)), // ⠠⠻
        (40, 59) => Some(('정', 2)), // ⠨⠻
        (48, 59) => Some(('청', 2)), // ⠰⠻
        (56, 14) => Some(('것', 2)), // ⠸⠎
        _ => None,
    }
}

// ── Abbreviated syllable shortcuts ────────────────────────────────────────
// These bytes represent a (jung, jong) pair when they appear:
//   • after a choseong → GotCho uses them to complete the syllable
//   • in Start/GotJung → emit ㅇ(11)+jung+jong as standalone
//
// Unicode TIndex (jongseong position in syllable formula):
//   ㄱ=1 ㄴ=4 ㄷ=7 ㄹ=8 ㅁ=16 ㅂ=17 ㅅ=19 ㅆ=20 ㅇ=21 ㅈ=22 ㅊ=23 ㅋ=24 ㅌ=25 ㅍ=26 ㅎ=27
fn try_abbreviated_shortcut(b: u8) -> Option<(u32, u32)> {
    match b {
        57 => Some((4, 1)),  // 억: ㅓ + ㄱ
        62 => Some((4, 4)),  // 언: ㅓ + ㄴ
        30 => Some((4, 8)),  // 얼: ㅓ + ㄹ
        33 => Some((6, 4)),  // 연: ㅕ + ㄴ
        51 => Some((6, 8)),  // 열: ㅕ + ㄹ
        59 => Some((6, 21)), // 영: ㅕ + ㅇ
        45 => Some((8, 1)),  // 옥: ㅗ + ㄱ
        55 => Some((8, 4)),  // 온: ㅗ + ㄴ
        63 => Some((8, 21)), // 옹: ㅗ + ㅇ
        27 => Some((13, 4)), // 운: ㅜ + ㄴ
        47 => Some((13, 8)), // 울: ㅜ + ㄹ
        53 => Some((18, 4)), // 은: ㅡ + ㄴ
        46 => Some((18, 8)), // 을: ㅡ + ㄹ
        31 => Some((20, 4)), // 인: ㅣ + ㄴ
        _ => None,
    }
}

// ── Consonant-a shortcuts (가, 사) ────────────────────────────────────────
// These bytes encode a specific consonant + implicit ㅏ vowel,
// but unlike the shared-byte shortcuts (나=9=ㄴ초성) these bytes
// are NOT in the choseong map, so we need a separate handler.
fn try_consonant_a_shortcut(b: u8) -> Option<u32> {
    match b {
        43 => Some(0), // 가: ㄱ(0) + ㅏ
        7 => Some(9),  // 사: ㅅ(9) + ㅏ
        _ => None,
    }
}

// ── Choseong ─────────────────────────────────────────────────────────────
// Unicode choseong indices:
// ㄱ=0 ㄲ=1 ㄴ=2 ㄷ=3 ㄸ=4 ㄹ=5 ㅁ=6 ㅂ=7 ㅃ=8 ㅅ=9 ㅆ=10
// ㅇ=11 ㅈ=12 ㅉ=13 ㅊ=14 ㅋ=15 ㅌ=16 ㅍ=17 ㅎ=18
fn try_choseong(b: u8) -> Option<u32> {
    match b {
        8 => Some(0),   // ㄱ
        9 => Some(2),   // ㄴ  (also 나 shortcut when not followed by jungseong)
        10 => Some(3),  // ㄷ  (also 다 shortcut)
        11 => Some(15), // ㅋ  (also 카 shortcut)
        16 => Some(5),  // ㄹ
        17 => Some(6),  // ㅁ  (also 마 shortcut)
        19 => Some(16), // ㅌ  (also 타 shortcut)
        24 => Some(7),  // ㅂ  (also 바 shortcut)
        25 => Some(17), // ㅍ  (also 파 shortcut)
        26 => Some(18), // ㅎ  (also 하 shortcut)
        32 => Some(9),  // ㅅ  (also 사-related, but ⠠ 사=7)
        40 => Some(12), // ㅈ  (also 자 shortcut)
        48 => Some(14), // ㅊ
        _ => None,
    }
}

// ── Double (tense) choseong — 제2항 된소리 ───────────────────────────────
// 된소리표 ⠠ (byte 32) + base consonant byte → tense choseong.
//   32 + 8  → ㄲ (idx 1)
//   32 + 10 → ㄸ (idx 4)
//   32 + 24 → ㅃ (idx 8)
//   32 + 32 → ㅆ (idx 10)
//   32 + 40 → ㅉ (idx 13)
// Falls back to try_choseong (single byte, ㅅ) if not a double pair.
fn try_choseong_with_double(bytes: &[u8], i: usize) -> Option<(u32, usize)> {
    let b0 = *bytes.get(i)?;
    if b0 == 32
        && let Some(&b1) = bytes.get(i + 1)
    {
        // 된소리표(32) 뒤에 오는 바이트 패턴:
        //  ㅏ 모음: 된소리표 + 약자 바이트 (가=43, 사=7, 나/다/바/자는 공유)
        //  기타 모음: 된소리표 + 초성 바이트 (ㄱ=8, ㅅ=32) + 별도 중성
        // 예) 까=[32,43]  꺼=[32,8,14]  싸=[32,7]  쏘=[32,32,37]
        let double_idx = match b1 {
            8 | 43 => Some(1),  // ㄲ: 8=ㄱ초성(꺼·끼…), 43=가약자(까)
            10 => Some(4),      // ㄸ: 공유바이트 (따·뚜… 모두)
            24 => Some(8),      // ㅃ: 공유바이트 (빠·뿌… 모두)
            7 | 32 => Some(10), // ㅆ: 7=사약자(싸), 32=ㅅ초성(쏘·씩…)
            40 => Some(13),     // ㅉ: 공유바이트 (짜·쭈… 모두)
            _ => None,
        };
        if let Some(idx) = double_idx {
            return Some((idx, 2));
        }
    }
    try_choseong(b0).map(|idx| (idx, 1))
}

// ── Jungseong ────────────────────────────────────────────────────────────
// Unicode jungseong indices:
// ㅏ=0 ㅐ=1 ㅑ=2 ㅒ=3 ㅓ=4 ㅔ=5 ㅕ=6 ㅖ=7 ㅗ=8 ㅘ=9 ㅙ=10 ㅚ=11 ㅛ=12
// ㅜ=13 ㅝ=14 ㅞ=15 ㅟ=16 ㅠ=17 ㅡ=18 ㅢ=19 ㅣ=20
fn try_jungseong(bytes: &[u8], i: usize) -> Option<(u32, usize)> {
    let b = *bytes.get(i)?;
    let b2 = bytes.get(i + 1).copied();

    // Compound vowels (2 bytes) checked first
    match (b, b2) {
        (13, Some(23)) => return Some((16, 2)), // ㅟ  [⠍⠗]
        (28, Some(23)) => return Some((3, 2)),  // ㅒ  [⠜⠗]
        (39, Some(23)) => return Some((10, 2)), // ㅙ  [⠧⠗]
        (15, Some(23)) => return Some((15, 2)), // ㅞ  [⠏⠗]
        _ => {}
    }

    let idx = match b {
        35 => 0,  // ㅏ
        23 => 1,  // ㅐ
        28 => 2,  // ㅑ
        14 => 4,  // ㅓ
        29 => 5,  // ㅔ
        49 => 6,  // ㅕ
        12 => 7,  // ㅖ
        37 => 8,  // ㅗ
        39 => 9,  // ㅘ
        61 => 11, // ㅚ
        44 => 12, // ㅛ
        13 => 13, // ㅜ
        15 => 14, // ㅝ
        41 => 17, // ㅠ
        42 => 18, // ㅡ
        58 => 19, // ㅢ
        21 => 20, // ㅣ
        _ => return None,
    };
    Some((idx, 1))
}

// ── Jongseong ────────────────────────────────────────────────────────────
// Unicode jongseong TIndex (0 = none):
// ㄱ=1 ㄲ=2 ㄳ=3 ㄴ=4 ㄵ=5 ㄶ=6 ㄷ=7 ㄹ=8
// ㄺ=9 ㄻ=10 ㄼ=11 ㄽ=12 ㄾ=13 ㄿ=14 ㅀ=15 ㅁ=16 ㅂ=17 ㅄ=18
// ㅅ=19 ㅆ=20 ㅇ=21 ㅈ=22 ㅊ=23 ㅋ=24 ㅌ=25 ㅍ=26 ㅎ=27
fn try_jongseong(bytes: &[u8], i: usize) -> (u32, usize) {
    let b = match bytes.get(i) {
        Some(&v) => v,
        None => return (0, 0),
    };
    let b2 = bytes.get(i + 1).copied();

    // Compound jongseong (2 bytes) checked first
    match (b, b2) {
        (1, Some(1)) => return (2, 2),   // ㄲ
        (1, Some(4)) => return (3, 2),   // ㄳ
        (18, Some(5)) => return (5, 2),  // ㄵ
        (18, Some(52)) => return (6, 2), // ㄶ
        (2, Some(1)) => return (9, 2),   // ㄺ
        (2, Some(34)) => return (10, 2), // ㄻ
        (2, Some(3)) => return (11, 2),  // ㄼ
        (2, Some(4)) => return (12, 2),  // ㄽ
        (2, Some(38)) => return (13, 2), // ㄾ
        (2, Some(50)) => return (14, 2), // ㄿ
        (2, Some(52)) => return (15, 2), // ㅀ
        (3, Some(4)) => return (18, 2),  // ㅄ
        _ => {}
    }

    let idx = match b {
        1 => 1,   // ㄱ
        18 => 4,  // ㄴ
        20 => 7,  // ㄷ
        2 => 8,   // ㄹ
        34 => 16, // ㅁ
        3 => 17,  // ㅂ
        4 => 19,  // ㅅ
        12 => 20, // ㅆ
        54 => 21, // ㅇ
        5 => 22,  // ㅈ
        6 => 23,  // ㅊ
        22 => 24, // ㅋ
        38 => 25, // ㅌ
        50 => 26, // ㅍ
        52 => 27, // ㅎ
        _ => return (0, 0),
    };
    (idx, 1)
}

// ── Syllable builder ─────────────────────────────────────────────────────

fn build_syllable(cho: u32, jung: u32, jong: u32) -> char {
    let code = (cho * 21 + jung) * 28 + jong + 0xAC00;
    char::from_u32(code).unwrap_or('?')
}

fn cho_idx_to_jamo(idx: u32) -> char {
    const JAMO: &[char] = &[
        'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ',
        'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
    ];
    JAMO.get(idx as usize).copied().unwrap_or('?')
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    //! 공개 Unicode API, encoder roundtrip, 내부 셀 상태 전이를 서로 분리해 검증한다.
    //! `decode_cells`에 전달하는 숫자는 점자 Unicode의 U+2800 오프셋과 같은 6점 셀 값이다.

    use super::*;
    use rstest::rstest;

    /// 지원 범위의 한글을 공개 encoder와 decoder에 연속으로 통과시킨다.
    fn roundtrip(text: &str) -> String {
        let encoded = crate::encode_to_unicode(text).expect("encode failed");
        decode(&encoded).expect("decode failed")
    }

    /// Unicode 입력 정규화를 거치지 않고 decoder 상태 머신에 셀을 직접 전달한다.
    fn decode_cells(cells: &[u8]) -> String {
        decode_bytes(cells).expect("decode failed")
    }

    /// 공개 API가 점자 이외 문자를 무시하고 명시적인 공백 경계는 보존하는지 검증한다.
    #[rstest]
    #[case::geo_ri("⠈⠎⠐⠕", "거리")]
    #[case::space_after_choseong("⠈ ⠉", "ㄱ 나")]
    #[case::newline_gap_and_non_braille_ignored("A⠈⠎\n⠐⠕B", "거 리")]
    #[case::unsupported_braille_cell_skipped("⡀⠫", "가")]
    fn decodes_public_unicode_input(#[case] braille: &str, #[case] expected: &str) {
        assert_eq!(decode(braille).unwrap(), expected);
    }

    /// 일반 음절, 공백, 약자가 섞인 대표 단어가 공개 API에서 왕복 가능한지 검증한다.
    #[rstest]
    #[case::gang_a_ji("강아지")]
    #[case::han_geul("한 글")]
    #[case::na_ga_da("나가다")]
    #[case::seong_jeong_cheong("성정청")]
    #[case::geo_seong("거성")]
    fn roundtrips_supported_words(#[case] word: &str) {
        assert_eq!(roundtrip(word), word);
    }

    /// 한 셀로 표현되는 한국 점자 약자 음절을 encoder 결과에서 다시 복원한다.
    #[rstest]
    #[case::eul("을")]
    #[case::eok("억")]
    #[case::eon("언")]
    #[case::eol("얼")]
    #[case::yeon("연")]
    #[case::yeol("열")]
    #[case::yeong("영")]
    #[case::ok("옥")]
    #[case::on("온")]
    #[case::ong("옹")]
    #[case::un("운")]
    #[case::ul("울")]
    #[case::eun("은")]
    #[case::in_("인")]
    fn roundtrips_abbreviated_syllables(#[case] word: &str) {
        assert_eq!(roundtrip(word), word);
    }

    /// ㅏ 결합형과 분리형을 포함한 된소리가 단어 안에서도 왕복되는지 검증한다.
    #[rstest]
    #[case::compact_gga("까")]
    #[case::compact_tta("따")]
    #[case::compact_ppa("빠")]
    #[case::compact_ssa("싸")]
    #[case::compact_jja("짜")]
    #[case::ggeo("꺼")]
    #[case::ddu("뚜")]
    #[case::sso("쏘")]
    #[case::gga_da("까다")]
    #[case::jja_da("짜다")]
    #[case::ggeot_eo_yo("껐어요")]
    #[case::a_gga("아까")]
    #[case::gi_ppeu_da("기쁘다")]
    fn roundtrips_tense_consonant_words(#[case] word: &str) {
        assert_eq!(roundtrip(word), word);
    }

    /// 초성 셀 하나가 ㅏ를 내포하는 공유 약자형을 음절로 복원한다.
    #[rstest]
    #[case::implicit_ga(&[8], "가")]
    #[case::implicit_na(&[9], "나")]
    #[case::implicit_da(&[10], "다")]
    #[case::implicit_ka(&[11], "카")]
    #[case::implicit_ra(&[16], "라")]
    #[case::implicit_ma(&[17], "마")]
    #[case::implicit_ta(&[19], "타")]
    #[case::implicit_ba(&[24], "바")]
    #[case::implicit_pa(&[25], "파")]
    #[case::implicit_ha(&[26], "하")]
    #[case::implicit_sa(&[32], "사")]
    #[case::implicit_ja(&[40], "자")]
    #[case::implicit_cha(&[48], "차")]
    fn decodes_shared_choseong_a_shortcuts(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// ㄱ/ㅅ과 ㅏ가 결합된 전용 셀 및 그 뒤의 종성을 복원한다.
    #[rstest]
    #[case::ga(&[43], "가")]
    #[case::sa(&[7], "사")]
    #[case::gak(&[43, 1], "각")]
    #[case::sat(&[7, 4], "삿")]
    fn decodes_explicit_consonant_a_shortcuts(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// 초성 없이 시작하는 모든 단모음·겹모음 셀을 묵음 초성 ㅇ과 조합한다.
    #[rstest]
    #[case::a(&[35], "아")]
    #[case::ae(&[23], "애")]
    #[case::ya(&[28], "야")]
    #[case::yae(&[28, 23], "얘")]
    #[case::eo(&[14], "어")]
    #[case::e(&[29], "에")]
    #[case::yeo(&[49], "여")]
    #[case::ye(&[12], "예")]
    #[case::o(&[37], "오")]
    #[case::wa(&[39], "와")]
    #[case::wae(&[39, 23], "왜")]
    #[case::oe(&[61], "외")]
    #[case::yo(&[44], "요")]
    #[case::u(&[13], "우")]
    #[case::wo(&[15], "워")]
    #[case::we(&[15, 23], "웨")]
    #[case::wi(&[13, 23], "위")]
    #[case::yu(&[41], "유")]
    #[case::eu(&[42], "으")]
    #[case::ui(&[58], "의")]
    #[case::i(&[21], "이")]
    fn decodes_standalone_jungseong(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// 초성·중성 상태에서 약자, 다음 음절, 미지원 셀을 만나는 경계 전이를 검증한다.
    #[rstest]
    #[case::geul_from_initial_plus_eul(&[8, 46], "글")]
    #[case::geok_from_initial_plus_eok(&[8, 57], "걱")]
    #[case::gin_from_initial_plus_in(&[8, 31], "긴")]
    #[case::na_then_eul(&[9, 35, 46], "나을")]
    #[case::na_then_ga_shortcut(&[9, 35, 43], "나가")]
    #[case::na_then_sa_shortcut(&[9, 35, 7], "나사")]
    #[case::na_then_next_choseong(&[9, 35, 10, 35], "나다")]
    #[case::consecutive_vowels(&[35, 21], "아이")]
    #[case::fallback_s_after_non_double_marker(&[32, 35], "사")]
    #[case::unknown_cell_after_syllable(&[8, 35, 64], "가")]
    fn decodes_state_transition_edges(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// 두 셀을 함께 읽어야 하는 성/정/청/것 약자와 앞 음절 flush를 검증한다.
    #[rstest]
    #[case::seong(&[32, 59], "성")]
    #[case::jeong(&[40, 59], "정")]
    #[case::cheong(&[48, 59], "청")]
    #[case::geot(&[56, 14], "것")]
    #[case::after_geo_seong(&[8, 14, 32, 59], "거성")]
    #[case::after_geo_geot(&[8, 14, 56, 14], "거것")]
    fn decodes_two_byte_shortcuts(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// 기본 음절 `가`의 셀 `[8, 35]` 뒤에 단일 종성 셀을 붙여 각 받침을 검증한다.
    #[rstest]
    #[case::gak(&[1], "각")]
    #[case::gan(&[18], "간")]
    #[case::gad(&[20], "갇")]
    #[case::gal(&[2], "갈")]
    #[case::gam(&[34], "감")]
    #[case::gab(&[3], "갑")]
    #[case::gas(&[4], "갓")]
    #[case::gass(&[12], "갔")]
    #[case::gang(&[54], "강")]
    #[case::gaj(&[5], "갖")]
    #[case::gach(&[6], "갗")]
    #[case::gak_final(&[22], "갘")]
    #[case::gat(&[38], "같")]
    #[case::gap(&[50], "갚")]
    #[case::gah(&[52], "갛")]
    fn decodes_single_jongseong(#[case] jongseong: &[u8], #[case] expected: &str) {
        let mut cells = vec![8, 35];
        cells.extend_from_slice(jongseong);

        assert_eq!(decode_cells(&cells), expected);
    }

    /// 동일한 `가` 기본 셀 뒤에 두 종성 셀을 붙여 겹받침 조합을 검증한다.
    #[rstest]
    #[case::gakk(&[1, 1], "갂")]
    #[case::gaks(&[1, 4], "갃")]
    #[case::ganj(&[18, 5], "갅")]
    #[case::ganh(&[18, 52], "갆")]
    #[case::galg(&[2, 1], "갉")]
    #[case::galm(&[2, 34], "갊")]
    #[case::galb(&[2, 3], "갋")]
    #[case::gals(&[2, 4], "갌")]
    #[case::galt(&[2, 38], "갍")]
    #[case::galp(&[2, 50], "갎")]
    #[case::galh(&[2, 52], "갏")]
    #[case::gabs(&[3, 4], "값")]
    fn decodes_compound_jongseong(#[case] jongseong: &[u8], #[case] expected: &str) {
        let mut cells = vec![8, 35];
        cells.extend_from_slice(jongseong);

        assert_eq!(decode_cells(&cells), expected);
    }

    /// 된소리표 뒤의 축약 초성 또는 일반 초성·중성 조합을 직접 검증한다.
    #[rstest]
    #[case::compact_gga(&[32, 43], "까")]
    #[case::compact_tta(&[32, 10], "따")]
    #[case::compact_ppa(&[32, 24], "빠")]
    #[case::compact_ssa(&[32, 7], "싸")]
    #[case::compact_jja(&[32, 40], "짜")]
    #[case::ggeo(&[32, 8, 14], "꺼")]
    #[case::ddu(&[32, 10, 13], "뚜")]
    #[case::ppi(&[32, 24, 21], "삐")]
    #[case::sso(&[32, 32, 37], "쏘")]
    #[case::jjyu(&[32, 40, 41], "쮸")]
    fn decodes_tense_choseong_cells(#[case] cells: &[u8], #[case] expected: &str) {
        assert_eq!(decode_cells(cells), expected);
    }

    /// 입력이 끝난 경우 종성 탐색이 셀을 소비하지 않고 종료되는 경계를 보장한다.
    #[test]
    fn empty_jongseong_slice_returns_no_match() {
        assert_eq!(try_jongseong(&[], 0), (0, 0));
    }
}
