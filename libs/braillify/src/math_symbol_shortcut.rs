use phf::phf_map;

use crate::unicode::decode_unicode;

static SHORTCUT_MAP: phf::Map<char, &'static [u8]> = phf_map! {
    // PDF 한국 점자 규정 (수학) — 동그라미 숫자 ①②③④⑤⑥⑦⑧⑨⑩
    '\u{2460}' => &[decode_unicode('⠼'), decode_unicode('⠂')], // ①
    '\u{2461}' => &[decode_unicode('⠼'), decode_unicode('⠆')], // ②
    '\u{2462}' => &[decode_unicode('⠼'), decode_unicode('⠒')], // ③
    '\u{2463}' => &[decode_unicode('⠼'), decode_unicode('⠲')], // ④
    '\u{2464}' => &[decode_unicode('⠼'), decode_unicode('⠢')], // ⑤
    '\u{2465}' => &[decode_unicode('⠼'), decode_unicode('⠖')], // ⑥
    '\u{2466}' => &[decode_unicode('⠼'), decode_unicode('⠶')], // ⑦
    '\u{2467}' => &[decode_unicode('⠼'), decode_unicode('⠦')], // ⑧
    '\u{2468}' => &[decode_unicode('⠼'), decode_unicode('⠔')], // ⑨
    '\u{2469}' => &[decode_unicode('⠼'), decode_unicode('⠴')], // ⑩
    '+' => &[decode_unicode('⠢')], // 5 (덧셈표)
    '/' => &[decode_unicode('⠸'), decode_unicode('⠌')], // _/ (분수 기호)
    '\u{2212}' => &[decode_unicode('⠔')], // 9 (뺄셈표)
    '\u{00D7}' => &[decode_unicode('⠡')], // * (곱셈표)
    '\u{00F7}' => &[decode_unicode('⠌'), decode_unicode('⠌')], // // (나눗셈표)
    '=' => &[decode_unicode('⠒'), decode_unicode('⠒')], // 33 (등호)
    '>' => &[decode_unicode('⠢'), decode_unicode('⠢')], // 55 (보다크다)
    '<' => &[decode_unicode('⠔'), decode_unicode('⠔')], // 99 (보다작다)
    '\u{2260}' => &[decode_unicode('⠨'), decode_unicode('⠒'), decode_unicode('⠒')], // .33 (같지않다)
    '\u{2265}' => &[decode_unicode('⠲'), decode_unicode('⠲')], // 44 (크거나같다)
    '\u{2267}' => &[decode_unicode('⠲'), decode_unicode('⠲')], // 44 (크거나같다)
    '\u{2264}' => &[decode_unicode('⠖'), decode_unicode('⠖')], // 66 (작거나같다)
    '\u{2266}' => &[decode_unicode('⠖'), decode_unicode('⠖')], // 66 (작거나같다)
    '\u{2252}' => &[decode_unicode('⠐'), decode_unicode('⠒'), decode_unicode('⠒')], // "33 (근삿값)
    '\u{2236}' => &[decode_unicode('⠐'), decode_unicode('⠂')], // "1 (비)
    '\u{2192}' => &[decode_unicode('⠒'), decode_unicode('⠕')], // 3o (오른쪽 화살표)
    '\u{2190}' => &[decode_unicode('⠪'), decode_unicode('⠒')], // [3 (왼쪽 화살표)
    '\u{2194}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠕')], // [3o (양쪽 화살표)
    '\u{2191}' => &[decode_unicode('⠰'), decode_unicode('⠒'), decode_unicode('⠕')], // ;3o (위쪽 화살표)
    '\u{2193}' => &[decode_unicode('⠘'), decode_unicode('⠒'), decode_unicode('⠕')], // ^3o (아래쪽 화살표)
    '\u{21D2}' => &[decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')], // 33o (항진명제)
    '\u{21D4}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')], // [33o (필요충분)
    '\u{21C4}' => &[decode_unicode('⠪'), decode_unicode('⠶'), decode_unicode('⠕')], // [7o (동치명제)
    '\u{2032}' => &[decode_unicode('⠤')], // - (프라임)
    '\u{2033}' => &[decode_unicode('⠤'), decode_unicode('⠤')], // -- (더블 프라임, PDF 제17항)
    '\u{2034}' => &[decode_unicode('⠤'), decode_unicode('⠤'), decode_unicode('⠤')], // --- (트리플 프라임)
    '\u{00B2}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠃')], // ^#b (제곱)
    '\u{00B3}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠉')], // ^#c (세제곱)
    '\u{2074}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠙')], // ^#d (네제곱)
    '\u{2075}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠑')], // ^#e (오제곱)
    '\u{2077}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠛')], // ^#g (칠제곱)
    '\u{2079}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠊')], // ^#i (구제곱)
    '\u{00B9}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠁')], // ^#a (1제곱)
    '\u{2070}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠚')], // ^#j (0제곱)
    '\u{1D4F}' => &[decode_unicode('⠘'), decode_unicode('⠅')], // ^k (위첨자 k)
    '\u{1D50}' => &[decode_unicode('⠘'), decode_unicode('⠍')], // ^m (위첨자 m)
    '\u{02E3}' => &[decode_unicode('⠘'), decode_unicode('⠭')], // ^x (위첨자 x)
    '\u{207D}' => &[decode_unicode('⠘'), decode_unicode('⠦')], // ^8 (위첨자 ()
    '\u{207E}' => &[decode_unicode('⠴')], // 0 (위첨자 ))
    '\u{207F}' => &[decode_unicode('⠘'), decode_unicode('⠝')], // ^n (위첨자 n)
    '\u{207B}' => &[decode_unicode('⠘'), decode_unicode('⠔')], // ^9 (위첨자 마이너스)
    '\u{207A}' => &[decode_unicode('⠘'), decode_unicode('⠢')], // ^5 (위첨자 플러스)
    '\u{2080}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠚')], // ;#j (아래첨자 0)
    '\u{2081}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠁')], // ;#a (아래첨자 1)
    '\u{2082}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠃')], // ;#b (아래첨자 2)
    '\u{2083}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠉')], // ;#c (아래첨자 3)
    '\u{2084}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠙')], // ;#d (아래첨자 4)
    '\u{2085}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠑')], // ;#e (아래첨자 5)
    '\u{2086}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠋')], // ;#f (아래첨자 6)
    '\u{2087}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠛')], // ;#g (아래첨자 7)
    '\u{2088}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠓')], // ;#h (아래첨자 8)
    '\u{2089}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠊')], // ;#i (아래첨자 9)
    '\u{208D}' => &[decode_unicode('⠰'), decode_unicode('⠦')], // ;8 (아래첨자 ()
    '\u{208E}' => &[decode_unicode('⠴')], // 0 (아래첨자 ))
    '\u{2090}' => &[decode_unicode('⠰'), decode_unicode('⠁')], // ;a (아래첨자 a)
    '\u{2098}' => &[decode_unicode('⠰'), decode_unicode('⠍')], // ;m (아래첨자 m)
    '\u{2093}' => &[decode_unicode('⠰'), decode_unicode('⠭')], // ;x (아래첨자 x)
    '\u{2099}' => &[decode_unicode('⠰'), decode_unicode('⠝')], // ;n (아래첨자 n)
    '\u{208A}' => &[decode_unicode('⠰'), decode_unicode('⠢')], // ;5 (아래첨자 +)
    '\u{2044}' => &[decode_unicode('⠌')], // / (분수 슬래시)
    '\u{2500}' => &[decode_unicode('⠌')], // ─ (괘선 — PDF 제7항 분수선 기호 형태)
    '\u{2E29}' => &[decode_unicode('⠄')], // open-ended right delimiter (`\right.`)
    '_' => &[decode_unicode('⠠'), decode_unicode('⠤')], // 밑줄 marker (PDF 제23항 2)
    '\u{0332}' => &[decode_unicode('⠠'), decode_unicode('⠤')], // ̲ (combining low line — 밑줄 결합부호)
    '|' => &[decode_unicode('⠳')], // | (절댓값)
    '\u{00AC}' => &[decode_unicode('⠈'), decode_unicode('⠔')], // @9 (부정)
    '\u{00B0}' => &[decode_unicode('⠴'), decode_unicode('⠙')], // 0d (도)
    '\u{00B1}' => &[decode_unicode('⠢'), decode_unicode('⠔')], // ± (PDF 제2항 — plus-minus)
    '\u{00B7}' => &[decode_unicode('⠐')], // " (점 곱셈)
    '…' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')], // ,,, (줄임표)
    '⋯' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')], // ,,, (줄임표)
    '\u{221A}' => &[decode_unicode('⠜')], // > (근호)
    '\u{2224}' => &[decode_unicode('⠨'), decode_unicode('⠳')], // .\ (나누어떨어지지않는다)
    '\u{2220}' => &[decode_unicode('⠹')], // ? (각)
    '\u{22A5}' => &[decode_unicode('⠴'), decode_unicode('⠄')], // 0' (수직)
    '\u{2225}' => &[decode_unicode('⠰'), decode_unicode('⠆')], // ;2 (평행)
    '\u{2AFD}' => &[decode_unicode('⠰'), decode_unicode('⠆')], // ;2 (평행)
    '\u{223D}' => &[decode_unicode('⠠'), decode_unicode('⠄')], // ,' (닮음)
    '\u{2261}' => &[decode_unicode('⠶'), decode_unicode('⠶')], // 77 (합동)
    '\u{221E}' => &[decode_unicode('⠿')], // = (무한대)
    '\u{222B}' => &[decode_unicode('⠮')], // ! (부정적분)
    '\u{222E}' => &[decode_unicode('⠾')], // ) (선적분)
    '\u{222C}' => &[decode_unicode('⠮'), decode_unicode('⠮')], // !! (이중적분)
    '\u{2207}' => &[decode_unicode('⠸'), decode_unicode('⠩')], // _% (델연산자)
    '\u{2202}' => &[decode_unicode('⠫')], // $ (편도함수)
    '\u{2208}' => &[decode_unicode('⠖')], // 6 (원소 왼쪽)
    '\u{220B}' => &[decode_unicode('⠲')], // 4 (원소 오른쪽)
    '\u{2209}' => &[decode_unicode('⠨'), decode_unicode('⠖')], // .6 (원소 아닌)
    '\u{220C}' => &[decode_unicode('⠨'), decode_unicode('⠲')], // .4 (원소아닌 오른쪽)
    '\u{2282}' => &[decode_unicode('⠖'), decode_unicode('⠂')], // 61 (부분집합 왼쪽)
    '\u{2283}' => &[decode_unicode('⠐'), decode_unicode('⠲')], // "4 (부분집합 오른쪽)
    '\u{2284}' => &[decode_unicode('⠨'), decode_unicode('⠖'), decode_unicode('⠂')], // .61 (부분집합 아님)
    '\u{2285}' => &[decode_unicode('⠨'), decode_unicode('⠐'), decode_unicode('⠲')], // ."4 (부분집합 아님)
    '\u{2205}' => &[decode_unicode('⠨'), decode_unicode('⠋')], // .f (공집합)
    '\u{222A}' => &[decode_unicode('⠬')], // + (합집합)
    '\u{2229}' => &[decode_unicode('⠩')], // % (교집합)
    '\u{2200}' => &[decode_unicode('⠨'), decode_unicode('⠄')], // .' (모든)
    '\u{2203}' => &[decode_unicode('⠨'), decode_unicode('⠢')], // .5 (존재하는)
    '\u{2204}' => &[decode_unicode('⠨'), decode_unicode('⠨'), decode_unicode('⠢')], // ..5 (존재하지 않는)
    '\u{2227}' => &[decode_unicode('⠹')], // ? (논리곱)
    '\u{2228}' => &[decode_unicode('⠼')], // # (논리합)
    '\u{22BB}' => &[decode_unicode('⠼'), decode_unicode('⠤')], // #- (배타적 논리합)
    '\u{2234}' => &[decode_unicode('⠠'), decode_unicode('⠡')], // ,* (그러므로)
    '\u{2235}' => &[decode_unicode('⠈'), decode_unicode('⠌')], // @/ (왜냐하면)
    '\u{2248}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔')], // @9@9 (이중물결)
    '\u{224A}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒')], // @9@93 (이중물결 아래줄)
    '\u{2243}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒')], // @93 (물결 아래줄)
    '\u{2245}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒'), decode_unicode('⠒')], // @933 (물결아래등호)
    '\u{2241}' => &[decode_unicode('⠨'), decode_unicode('⠈'), decode_unicode('⠔')], // .@9 (not sim)
    '\u{226E}' => &[decode_unicode('⠨'), decode_unicode('⠔'), decode_unicode('⠔')], // .99 (보다작지않다)
    '\u{226F}' => &[decode_unicode('⠨'), decode_unicode('⠢'), decode_unicode('⠢')], // .55 (보다크지않다)
    '\u{2270}' => &[decode_unicode('⠨'), decode_unicode('⠖'), decode_unicode('⠖')], // .66 (작거나같지않다)
    '\u{2271}' => &[decode_unicode('⠨'), decode_unicode('⠲'), decode_unicode('⠲')], // .44 (크거나같지않다)
    '\u{25B7}' => &[decode_unicode('⠸'), decode_unicode('⠜')], // _> (오른쪽 세모꼴)
    '\u{25C1}' => &[decode_unicode('⠸'), decode_unicode('⠣')], // _< (왼쪽 세모꼴)
    '\u{25A1}' => &[decode_unicode('⠸'), decode_unicode('⠶')], // _7 (네모)
    '\u{25B3}' => &[decode_unicode('⠸'), decode_unicode('⠬')], // _+ (세모)
    '\u{25B1}' => &[decode_unicode('⠸'), decode_unicode('⠌'), decode_unicode('⠌')], // _// (평행사변형)
    '\u{23E2}' => &[decode_unicode('⠸'), decode_unicode('⠌'), decode_unicode('⠡')], // _/* (사다리꼴)
    '\u{2302}' => &[decode_unicode('⠸'), decode_unicode('⠪'), decode_unicode('⠅')], // _[k (집)
    '\u{2394}' => &[decode_unicode('⠸'), decode_unicode('⠪'), decode_unicode('⠕')], // _[o (기하 기호)
    '\u{29BE}' => &[decode_unicode('⠸'), decode_unicode('⠴'), decode_unicode('⠴')], // _00 (원안점)
    '\u{03A3}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠎')], // ,.s (총합)
    '\u{2295}' => &[decode_unicode('⠸'), decode_unicode('⠢')], // _5 (동그라미 덧셈표)
    '\u{2296}' => &[decode_unicode('⠸'), decode_unicode('⠔')], // _9 (동그라미 뺄셈표)
    '\u{2297}' => &[decode_unicode('⠸'), decode_unicode('⠡')], // _* (동그라미 곱셈표)
    '\u{2217}' => &[decode_unicode('⠸'), decode_unicode('⠣')], // _< (별표)
    '\u{2218}' => &[decode_unicode('⠸'), decode_unicode('⠴')], // _0 (동그라미)
    '\u{03B1}' => &[decode_unicode('⠨'), decode_unicode('⠁')], // .a (알파)
    '\u{03B2}' => &[decode_unicode('⠨'), decode_unicode('⠃')], // .b (베타)
    '\u{03B3}' => &[decode_unicode('⠨'), decode_unicode('⠛')], // .g (감마)
    '\u{03B4}' => &[decode_unicode('⠨'), decode_unicode('⠙')], // .d (델타)
    '\u{03B5}' => &[decode_unicode('⠨'), decode_unicode('⠑')], // .e (엡실론)
    '\u{03B6}' => &[decode_unicode('⠨'), decode_unicode('⠵')], // .z (제타)
    '\u{03B7}' => &[decode_unicode('⠨'), decode_unicode('⠱')], // .: (에타)
    '\u{03B8}' => &[decode_unicode('⠨'), decode_unicode('⠹')], // .? (세타)
    '\u{03B9}' => &[decode_unicode('⠨'), decode_unicode('⠊')], // .i (요타)
    '\u{03BA}' => &[decode_unicode('⠨'), decode_unicode('⠅')], // .k (카파)
    '\u{03BB}' => &[decode_unicode('⠨'), decode_unicode('⠇')], // .l (람다)
    '\u{03BC}' => &[decode_unicode('⠨'), decode_unicode('⠍')], // .m (뮤)
    '\u{03BD}' => &[decode_unicode('⠨'), decode_unicode('⠝')], // .n (뉴)
    '\u{03BE}' => &[decode_unicode('⠨'), decode_unicode('⠭')], // .x (크시)
    '\u{03BF}' => &[decode_unicode('⠨'), decode_unicode('⠕')], // .o (오미크론)
    '\u{03C0}' => &[decode_unicode('⠨'), decode_unicode('⠏')], // .p (파이)
    '\u{03C1}' => &[decode_unicode('⠨'), decode_unicode('⠗')], // .r (로)
    '\u{03C3}' => &[decode_unicode('⠨'), decode_unicode('⠎')], // .s (시그마)
    '\u{03C4}' => &[decode_unicode('⠨'), decode_unicode('⠞')], // .t (타우)
    '\u{03C5}' => &[decode_unicode('⠨'), decode_unicode('⠥')], // .u (입실론)
    '\u{03C6}' => &[decode_unicode('⠨'), decode_unicode('⠋')], // .f (피)
    '\u{03C7}' => &[decode_unicode('⠨'), decode_unicode('⠯')], // .& (키)
    '\u{03C8}' => &[decode_unicode('⠨'), decode_unicode('⠽')], // .y (프시)
    '\u{03C9}' => &[decode_unicode('⠨'), decode_unicode('⠺')], // .w (오메가)
    '\u{0391}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠁')], // ,.a (대문자 알파)
    '\u{0392}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠃')], // ,.b (대문자 베타)
    '\u{0393}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠛')], // ,.g (대문자 감마)
    '\u{0395}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠑')], // ,.e (대문자 엡실론)
    '\u{0396}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠵')], // ,.z (대문자 제타)
    '\u{0397}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠱')], // ,.: (대문자 에타)
    '\u{0398}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠹')], // ,.? (대문자 세타)
    '\u{0399}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠊')], // ,.i (대문자 요타)
    '\u{039A}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠅')], // ,.k (대문자 카파)
    '\u{039B}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠇')], // ,.l (대문자 람다)
    '\u{039C}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠍')], // ,.m (대문자 뮤)
    '\u{039D}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠝')], // ,.n (대문자 뉴)
    '\u{039E}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠭')], // ,.x (대문자 크시)
    '\u{039F}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠕')], // ,.o (대문자 오미크론)
    '\u{03A0}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠏')], // ,.p (대문자 파이)
    '\u{03A1}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠗')], // ,.r (대문자 로)
    '\u{03A4}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠞')], // ,.t (대문자 타우)
    '\u{03A5}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠥')], // ,.u (대문자 입실론)
    '\u{03A6}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠋')], // ,.f (대문자 피)
    '\u{03A7}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠯')], // ,.& (대문자 키)
    '\u{03A8}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠽')], // ,.y (대문자 프시)
    '\u{03A9}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠺')], // ,.w (대문자 오메가)
    '\u{0394}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠙')], // ,.d (대문자 델타)
    '\u{2196}' => &[decode_unicode('⠪'), decode_unicode('⠢')], // [5 (왼쪽 위 화살표)
    '\u{2197}' => &[decode_unicode('⠔'), decode_unicode('⠕')], // 9o (오른쪽 위 화살표)
    '\u{2198}' => &[decode_unicode('⠢'), decode_unicode('⠕')], // 5o (오른쪽 아래 화살표)
    '\u{2199}' => &[decode_unicode('⠪'), decode_unicode('⠔')], // [9 (왼쪽 아래 화살표)
    '\u{21CF}' => &[decode_unicode('⠨'), decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')], // .33o (함의 부정)
    '\u{2135}' => &[decode_unicode('⠗'), decode_unicode('⠋')], // rf (알레프)
    '\u{2206}' => &[decode_unicode('⠸'), decode_unicode('⠬')], // _+ (세모꼴)
    '\u{2219}' => &[decode_unicode('⠸'), decode_unicode('⠲')], // _4 (검정 동그라미)
    '\u{FF03}' => &[decode_unicode('⠸'), decode_unicode('⠹')], // _? (샤프 기호)
    '\u{1D9C}' => &[decode_unicode('⠘'), decode_unicode('⠉')], // ^c (여집합)
    '\u{0302}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠢')], // @@5 (결합 hat)
    '\u{0304}' => &[decode_unicode('⠈'), decode_unicode('⠉')], // @c (결합 가로바)
    '\u{0305}' => &[decode_unicode('⠈'), decode_unicode('⠉')], // @c (결합 윗줄)
    '\u{2016}' => &[decode_unicode('⠳'), decode_unicode('⠳')], // \\ (이중 세로선)
    '\u{2322}' => &[decode_unicode('⠈'), decode_unicode('⠪')], // @[ (호)
    // PDF 수학 제65항 5 — 문자 위 결합 부호 (틸데)
    '\u{0303}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')], // @@9 (결합 틸데)
    // 결합 윗 한 점 U+0307은 컨텍스트에 따라 의미가 다르다:
    //   - 숫자 뒤  : 순환소수 마크 (PDF 수학 제9항) → ⠈
    //   - 문자 뒤  : 문자 위 한 점 (PDF 수학 제65항 5) → ⠈⠲
    // 이 SHORTCUT_MAP의 값은 숫자 뒤 기본형이고, 문자 뒤 처리는 rule_65에서 별도 분기한다.
    '\u{0307}' => &[decode_unicode('⠈')], // @ (결합 윗점 - 기본/숫자 뒤)
    '\u{0308}' => &[decode_unicode('⠈'), decode_unicode('⠲'), decode_unicode('⠲')], // @44 (결합 윗 두 점)
    '\u{0309}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')], // @@9 (결합 고리/훅)
    '\u{030A}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')], // @@9 (결합 윗고리)
    '\u{211B}' => &[decode_unicode('⠠'), decode_unicode('⠗')], // ,R (ℛ = script R)
    '~' => &[decode_unicode('⠈'), decode_unicode('⠔')], // @9 (물결 = 닮음)
    '\u{0338}' => &[decode_unicode('⠨')], // . (부정 표지)
    '\u{203E}' => &[decode_unicode('⠈'), decode_unicode('⠉')], // @c (선분 기호 U+203E)
    '\u{20E1}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠕')], // [3O (직선 기호 U+20E1)
    '\u{20D7}' => &[decode_unicode('⠒'), decode_unicode('⠕')], // 3O (반직선 기호 U+20D7)
    // PDF 수학 제60항 6 — 추론 기호 ⊢/⊣/⊨/⫤
    '\u{22A2}' => &[decode_unicode('⠸'), decode_unicode('⠒')], // _3 (⊢ vdash)
    '\u{22A3}' => &[decode_unicode('⠈'), decode_unicode('⠸'), decode_unicode('⠒')], // @_3 (⊣ dashv)
    '\u{22A8}' => &[decode_unicode('⠘'), decode_unicode('⠸'), decode_unicode('⠒')], // ^_3 (⊨ models)
    '\u{2AE4}' => &[decode_unicode('⠨'), decode_unicode('⠸'), decode_unicode('⠒')], // ._3 (⫤ Dashv)
    // PDF 수학 제60항 7 — 앞선다 ≲ (보다같거나 작다 + 닮음)
    '\u{2272}' => &[decode_unicode('⠔'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔')], // 99@9 (≲ lesssim)
    // PDF 수학 제60항 8 — 앞서고같지않다 ≺ (보다작다)
    '\u{227A}' => &[decode_unicode('⠔'), decode_unicode('⠔')], // 99 (≺ prec — same as <)
    // PDF 수학 제61항 7 — 동치명제 ⇌
    '\u{21CC}' => &[decode_unicode('⠪'), decode_unicode('⠶'), decode_unicode('⠕')], // [7o (⇌ rightleftharpoons)
    // PDF 수학 제23항 1 — 켤레복소수/평균값 macron ¯
    '\u{00AF}' => &[decode_unicode('⠈'), decode_unicode('⠉')], // @c (¯ macron)
    // PDF 수학 제25항 — 총합 기호 ∑ (Greek capital Sigma과 동일 점형)
    '\u{2211}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠎')], // ,.s
    // PDF 수학 제26항 — 곱 기호 ∏
    '\u{220F}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠏')], // ,.p
};

pub fn encode_char_math_symbol_shortcut(text: char) -> Result<&'static [u8], String> {
    if let Some(code) = SHORTCUT_MAP.get(&text) {
        Ok(code)
    } else {
        Err("Invalid math symbol character".to_string())
    }
}

pub fn is_math_symbol_char(text: char) -> bool {
    SHORTCUT_MAP.contains_key(&text)
}

#[cfg(test)]
mod test {
    use super::*;

    /// `is_math_symbol_char` true 케이스 — 연산자/그리스/집합/미적분 기호 전체.
    #[rstest::rstest]
    // basic operators
    #[case('+')]
    #[case('−')]
    #[case('×')]
    #[case('÷')]
    #[case('=')]
    // greek
    #[case('α')]
    #[case('π')]
    #[case('ω')]
    // set / logic
    #[case('∈')]
    #[case('∅')]
    #[case('∪')]
    #[case('∩')]
    // calculus
    #[case('∫')]
    #[case('∞')]
    #[case('√')]
    fn is_math_symbol_char_recognizes_math_chars(#[case] ch: char) {
        assert!(is_math_symbol_char(ch));
    }

    /// `is_math_symbol_char` false 케이스 — ASCII 알파벳은 수학 기호 아님.
    #[test]
    fn is_math_symbol_char_rejects_ascii_letter() {
        assert!(!is_math_symbol_char('a'));
    }

    /// `encode_char_math_symbol_shortcut` 표 매핑.
    /// - `²` (제곱) → `⠘⠼⠃` (^#b, 3-cell)
    /// - `≥` (크거나 같음) → `⠲⠲` (2-cell)
    /// - `≤` (작거나 같음) → `⠖⠖` (2-cell)
    #[rstest::rstest]
    #[case('²', &['⠘', '⠼', '⠃'][..])]
    #[case('≥', &['⠲', '⠲'][..])]
    #[case('≤', &['⠖', '⠖'][..])]
    fn encode_char_math_symbol_shortcut_table(
        #[case] input: char,
        #[case] expected_chars: &[char],
    ) {
        let expected: Vec<u8> = expected_chars.iter().copied().map(decode_unicode).collect();
        assert_eq!(
            encode_char_math_symbol_shortcut(input).unwrap(),
            expected.as_slice()
        );
    }
}
