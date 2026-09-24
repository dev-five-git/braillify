# CORE LIBRARY (libs/braillify)

Korean + Math Braille encoding engine implementing 2024 Korean Braille Standard.

## STRUCTURE

```
src/
├── lib.rs              # Main encode() entry + encode_to_unicode() / encode_to_braille_font()
├── cli.rs              # CLI: REPL + one-shot mode (feature-gated)
├── main.rs             # Binary entry point
├── encoder.rs          # DocumentIR construction, token + char engine orchestration
├── char_struct.rs      # CharType enum (Korean/English/Number/Symbol/MathSymbol/Fraction)
├── korean_char.rs      # Full Korean syllable encoding
├── korean_part.rs      # Standalone jamo (consonant/vowel) encoding
├── jauem/              # Consonant handling
│   ├── choseong.rs     # Initial consonants
│   └── jongseong.rs    # Final consonants
├── moeum/              # Vowel handling
│   └── jungsong.rs     # Medial vowels
├── english.rs          # English letter encoding
├── english_logic.rs    # English context detection
├── hanja.rs            # 한자 → 한국어 음독 표 (Unihan kHangul, resources/hanja-readings.txt)
├── number.rs           # Number encoding
├── fraction.rs         # Fraction handling (Unicode + LaTeX)
├── math_symbol_shortcut.rs  # PHF math symbol lookup table
├── symbol_shortcut.rs       # PHF general symbol lookup table
├── word_shortcut.rs         # PHF word abbreviation lookup table
├── unicode.rs          # Internal braille code ↔ Unicode Braille conversion
├── split.rs            # Korean jamo decomposition
├── utils.rs            # Helper functions
└── rules/              # Rule engine (see below)
```

## ENCODING PIPELINE

```
Input text
  ↓ DocumentIR::parse()         (tokenize into Word/Space/Mode tokens)
  ↓ TokenRuleEngine::apply_all() (token-level rules by phase)
  │   ├── LatexMergeRule         (merge $...$ across spaces)
  │   ├── LatexFractionRule      (detect $\frac{}{})$)
  │   ├── LatexMathRule          (strip LaTeX → math notation)
  │   ├── InlineFractionRule     (detect N/N inline fractions)
  │   ├── MathExpressionTokenRule (detect & encode math expressions)
  │   └── ...other token rules
  ↓ emit()                      (character-level encoding)
      ├── Token::Word → RuleEngine (BrailleRule trait, char-by-char)
      ├── Token::Space → braille space byte
      ├── Token::Fraction → fraction encoding
      └── Token::PreEncoded → pass-through (from math encoder)
```

## RULE ARCHITECTURE

### Two parallel rule systems

| System | Trait | Engine | Operates On | Used By |
|--------|-------|--------|-------------|---------|
| Korean (char-level) | `BrailleRule` | `RuleEngine` | Individual characters (`CharType`) | Korean text encoding |
| Math (token-level) | `MathTokenRule` | `MathTokenEngine` | Token sequences (`MathToken`) | Math expression encoding |

### BrailleRule (Korean, character-level)

```rust
trait BrailleRule: Send + Sync {
    fn meta(&self) -> &'static RuleMeta;
    fn phase(&self) -> Phase;           // Preprocessing → CoreEncoding → InterCharacter
    fn matches(&self, ctx: &RuleContext) -> bool;
    fn apply(&self, ctx: &mut RuleContext) -> Result<RuleResult, String>;
}
```

Registered in `encoder.rs` → processes one character at a time via `RuleContext`.

### MathTokenRule (Math, token-level)

```rust
trait MathTokenRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u16;          // Lower = runs first (10=lookahead, 50=core, 100=symbol)
    fn matches(&self, tokens: &[MathToken], index: usize, state: &MathEncodeState) -> bool;
    fn apply(&self, tokens: &[MathToken], index: usize, result: &mut Vec<u8>,
             state: &mut MathEncodeState, engine: &MathTokenEngine) -> Result<MathTokenResult, String>;
}
```

Registered in `encoder.rs::build_math_engine()` → processes parsed MathToken sequences with lookahead.

### Math rule structs (in respective rule files)

| Priority | Struct | File | Handles |
|----------|--------|------|---------|
| 10 | `FractionReversalRule` | rule_7.rs | Denominator-first simple fractions |
| 10 | `ConditionalProbFractionRule` | rule_7.rs | =a/b with \| pattern |
| 10 | `CombinatoricsRule` | rule_12.rs | nPr, nCr |
| 50 | `NumberRule` | rule_1.rs | Number tokens |
| 50 | `VariableRule` | rule_12.rs | Lowercase variables |
| 50 | `UpperVariableRule` | rule_12.rs | Uppercase variables |
| 50 | `OperatorRule` | rule_2.rs | Arithmetic operators |
| 50 | `FunctionNameRule` | rule_47.rs | log, lim, sin, cos... |
| 50 | `BracketRule` | rule_6.rs | Open/close parentheses |
| 50 | `SuperscriptRule` | rule_18.rs | Superscript content |
| 50 | `SubscriptRule` | rule_19.rs | Subscript content |
| 50 | `DecimalPointRule` | rule_8.rs | Decimal points |
| 50 | `PrimeRule` | rule_53.rs | Prime marks |
| 100 | `MathSymbolRule` | encoder.rs | All math symbols (30+ dispatch chain) |

## KEY TYPES

| Type | Location | Purpose |
|------|----------|---------|
| `CharType` | `char_struct.rs` | Input character classification |
| `BrailleRule` | `rules/traits.rs` | Korean char-level rule trait |
| `MathTokenRule` | `rules/math/math_token_rule.rs` | Math token-level rule trait |
| `MathTokenEngine` | `rules/math/math_token_rule.rs` | Math rule dispatch engine |
| `MathToken` | `rules/math/parser.rs` | Parsed math expression token |
| `MathEncodeState` | `rules/math/math_token_rule.rs` | Shared math encoding state |
| `TokenRule` | `rules/token_rule.rs` | Token-level rule trait (pre-encoding) |
| `RuleEngine` | `rules/engine.rs` | Korean BrailleRule dispatch |
| `TokenRuleEngine` | `rules/token_engine.rs` | Token-level rule dispatch |

## ENTRY POINTS

| Function | Location | Usage |
|----------|----------|-------|
| `encode(text)` | `lib.rs` | Returns `Result<Vec<u8>, String>` |
| `encode_to_unicode(text)` | `lib.rs` | Returns Braille Unicode string |
| `encode_math_expression(text)` | `rules/math/encoder.rs` | Math-only encoding |
| `run_cli(args)` | `cli.rs` | CLI entry (feature: cli) |

## MATH RULES (src/rules/math/)

66 rule files (`rule_1.rs` through `rule_66.rs`) matching articles from the 2024 Korean Braille Standard math section (pages 51-84). Each file contains:

- `is_xxx()` detection functions (used in MathSymbolRule dispatch chain)
- `encode_xxx()` encoding functions (produce braille byte sequences)
- MathTokenRule struct implementations (where applicable)
- `#[cfg(test)] mod tests` with unit tests

Infrastructure:
- `encoder.rs` — `encode_math_expression()`, `build_math_engine()`, `MathSymbolRule`
- `parser.rs` — `parse_math_expression()` → `Vec<MathToken>`
- `function.rs` — Function name detection (sin, cos, log, etc.)
- `math_token_rule.rs` — `MathTokenRule` trait, `MathTokenEngine`, `MathEncodeState`

## CONVENTIONS

- PHF macros (`phf_map!`) for all static lookup tables
- Error handling via `Result<T, String>` — propagate, never suppress
- Feature flags: `cli` (default), `wasm`
- Tests inline with `#[cfg(test)]` in each module (see TEST ORGANIZATION)
- No `#[allow(dead_code)]` — all functions must be used or tested
- Math rules: one `.rs` file per standard article (제N항)

## TEST ORGANIZATION (NON-NEGOTIABLE)

### 1. 테스트 코드 배치

테스트 코드의 배치 우선순위는 **inline → tests/ 폴더** 순이다. `src/` 트리에는 테스트 전용 `.rs` 파일을 두지 않는다.

| 상황 | 배치 | 예시 |
|---|---|---|
| 단일 모듈에서만 쓰는 단위 테스트 | **inline `#[cfg(test)] mod tests { ... }`** (해당 `.rs` 파일 하단) | `rule_18.rs` 끝에 `mod tests` |
| 여러 모듈이 공유하는 테스트 utility (예: `test_helpers`) | **owning module 안에 inline `#[cfg(test)] mod name { ... }`** 블록 | `lib.rs` 안의 `mod test_helpers { ... }` |
| 공개 API 만 검증하는 시나리오 테스트 | **`tests/` 폴더 (integration test)** | `tests/cli_smoke.rs` |
| 위 어느 것도 불가능 | tests/ 폴더로 강제 | — |

**금지 패턴 (BLOCKING):**

- ❌ `src/test_helpers.rs`, `src/rule_18_test.rs` 같은 standalone test-only `.rs` 파일
- ❌ test fixture 만 담은 `mod something;` 선언 (별도 파일로 분리된 형태)
- ❌ production code 와 같은 파일에 `#[cfg(test)]` 없는 helper 를 두는 것

### 2. 파라미터화 테스트는 `rstest` 우선 (NON-NEGOTIABLE)

같은 호출 shape 으로 입력/기대값만 다른 케이스가 **3개 이상** 모이면 **반드시 `rstest::rstest`** 로 파라미터화한다. for-loop 위에 assertion 을 거는 패턴을 새로 추가하지 않는다.

**필수 변환 트리거 (둘 중 하나라도 해당 → rstest 사용):**

- 동일 fn 에 3+ assertion (입력/기대값만 다름)
- `for x in [a, b, c, ...] { assert!(...) }` 형태의 테이블 루프

**Cheat sheet:**

```rust
// ❌ Anti-pattern: hand-rolled table loop, 실패시 어느 케이스인지 불명확
#[test]
fn encodes_digits() {
    assert_eq!(encode_digit('1').unwrap(), decode_unicode('⠁'));
    assert_eq!(encode_digit('0').unwrap(), decode_unicode('⠚'));
    assert_eq!(encode_digit('9').unwrap(), decode_unicode('⠊'));
}

// ✅ rstest: 케이스별 라벨 → 실패 위치 즉시 파악
#[rstest::rstest]
#[case::one('1', '⠁')]
#[case::zero('0', '⠚')]
#[case::nine('9', '⠊')]
fn encodes_digits(#[case] ch: char, #[case] expected: char) {
    assert_eq!(encode_digit(ch).unwrap(), decode_unicode(expected));
}

// ✅ 단일 입력만 다른 경우 #[values(...)] 사용 가능
#[rstest::rstest]
fn unicode_superscripts_parse_ok(#[values('\u{2070}', '\u{00B9}', '\u{00B2}')] c: char) {
    let result = parse_math_expression(&format!("x{c}"));
    assert!(result.is_ok(), "parse failed for x{c:?}");
}
```

**원칙:**

- `#[case::label(...)]` 의 label 은 의미를 담는다 (`#[case::lower_a]`, `#[case::overflow_max]` 등)
- 실패 메시지에 input 을 포함시킬 필요가 줄어든다 — label 이 그 역할을 한다
- `#[case]` 와 `#[values]` 둘 다 가능한 경우 `#[case]` 우선 (의도 표현이 더 명확)
- smoke test (`let _ = enc(input);` 처럼 assertion 이 없는 경우) 는 변환 의무 없음 — for-loop 형태가 더 간결하면 유지

## ANTI-PATTERNS

- **Never use `unwrap()` on user input** — return `Err(String)`
- **Never hardcode Braille dots** — use constants or PHF tables
- **Never modify shortcut tables** without updating test cases
- **Never add `#[allow(dead_code)]`** — wire functions into encoder or tests instead
- **Never suppress type errors** — no `as any` equivalents

## TESTING

```bash
cargo test --release -p braillify    # All tests (4600+ unit + integration)
cargo test --release -p braillify --lib -- test_by_testcase --nocapture
cargo fmt && cargo clippy            # Format + lint
bun test test_cases/                 # JSON integrity checks (packages/node/pkg 빌드 필요)
```

규정 fixture 는 `test_cases/{korean,math,english,science}/*.json` 이고, 그와 별개로
`test_cases/{2021,2022,2023,2024,2025}_corpus/sentence_*.json` 에 국립국어원
한국어-한국점자 병렬 말뭉치 46만 7121문장이 들어 있다. 말뭉치는 `rule_map.json` 에서
`benchmark: true` 로 표시되어 **pass/fail 에 들어가지 않고 정확도만 보고**한다.

**Current status: 규정 fixture 5272/5272 (100%, `limitation` 없음), 말뭉치 456,089/467,121 (97.64%).**

⚠️ `roman_marker_bench` 는 **어절 수가 맞는 문장만** 센다. 띄어쓰기를 바꾸면
비교 모집단 자체가 움직이므로 네 수치를 그대로 빼서 비교하면 안 된다. 실제로
붙임표 결합을 고쳤을 때 모집단이 8,554 → 8,566 으로 늘면서 표지 합계가 1 늘었는데,
공통 모집단에서는 개선이었다. 띄어쓰기를 건드릴 때는 모집단 크기를 함께 재라.

`KNOWN_FAILURES` 상수는 더 이상 존재하지 않는다. raw `encode()` 가 모든 testcase 에서
PDF 정답과 byte-동일 결과를 낸다. 새로 추가되는 testcase 도 같은 기준을 만족해야 한다.

### 말뭉치를 따라가면 안 되는 자리 (국립국어원 회신 2026-09-17)

- `severity` · `reverse` 는 두 번째 `e` 에 강세가 있어 **철자**가 맞다. 말뭉치 15건이
  약자로 적은 것은 검수에서 놓친 오류다. 맞추려 하지 말 것.
- 한글 낱말에 붙은 **소괄호 종류는 규정에 없고 점역자의 선택**이다. 칸수가 적은 쪽이
  효율적이지만 규정이 정한 바가 없으므로 말뭉치 다수파(한글 소괄호)를 따른다.
- **제37항은 로마자표 바로 뒤의 단어에만** 적용된다. `Yes, I can.` 에서 로마자표는
  `Y` 앞에 오므로 `can` 은 약자를 쓴다. 제37항 표는 §10.1 알파벳 단어 약자와
  §10.5 하위 단어 약자 6개뿐이고 축어(§10.9)는 들어 있지 않다.
- 1급 점자는 두 갈래로만 판정한다. 글자열 전체가 축어와 같거나(`CD`=could),
  **§10.9.5 로 s 가 붙거나**(`SDS`=said+s, `ACS`=according+s — 축어 목록 전체에
  적용된다), **§10.9.3 의 열 축어**(braille, great, children, blind, first, friend,
  good, letter, little, quick)로 시작하고 뒤가 모음·y 가 아닐 때(`GDP`=good+p)다.
  그 밖에는 전치하지 않는다. 말뭉치는 이 자리에서 583번 ⠰ 를 빠뜨렸으므로
  정확도가 떨어지는 쪽이 규정에 맞는 쪽이다.
- ⠰ 를 붙이는 **경계** 는 이미 맞다. 전수로 세면 붙임표 뒤는 55:3 으로 붙이고,
  숫자 96:3 · 빗금 9:3 · 더하기 5:0 으로 붙이지 않는다 — `permits_grade1_boundary_after_run`
  이 그대로 한다. 말뭉치가 `CD/ATM`·`CD19` 에서 ⠰ 를 쓴 42건은 그쪽 비일관이지
  우리 버그가 아니다. 고치려 들지 말 것.

### 개선 후보를 고르는 법 (두 번 틀린 뒤 적어 둠)

**실패한 어절만 세면 안 된다. 반드시 전수로 "우리 × 말뭉치" 2×2 교차표를 만든다.**
1급 점자는 실패 표본에서 과잉 590 : 누락 42 라 유망해 보였으나 전수로는 `GDP` 가
699 : 118 로 현행이 다수파였고, 전부 끄자 -995 였다. 소괄호도 실패 표본에서
493 : 17 이었으나 바꾸자 2023 판본만 -577 이었다. 전수 교차표로 잰 `ever` 와
`be`/`con`/`dis` 는 결론이 뒤집히지 않았다.

## BENCHMARK

```bash
# 마이크로 벤치 (criterion) — Wave 0 인프라
cargo bench -p braillify --bench encode_native
cargo bench -p braillify --bench encode_math

# 메모리 프로파일 (dhat)
cargo bench -p braillify --bench memory_dhat --features dhat-heap

# 외부 점역기 비교 — 규정 fixture
bun run scripts/world-bench.ts            # 점자세상
bun run scripts/jeomsarang-bench.ts       # 점사랑 (BrailleTransLibrary)

# 외부 점역기 비교 — 말뭉치
bun run scripts/jeomjasesang-corpus-bench.ts
bun run scripts/jeomsarang-corpus-bench.ts

# braillify / 점자세상 / 점사랑 3종 동시 측정 + 교차표
cargo run --release -p braillify --example three_way_bench

# 로마자 구간 표지(제29·33·35항) 회귀 안전망 — 4유형 개별 추적
cargo run --release -p braillify --example roman_marker_bench
```

`roman_marker_bench` 는 로마자표/종료표의 과잉·누락을 따로 센다. 이 넷은 서로 반대
방향이라 총 정확도만 보면 한쪽을 고치며 다른 쪽을 망가뜨려도 드러나지 않는다. 구간
상태 기계를 건드릴 때에는 반드시 이 네 수치를 함께 본다.

벤치 결과: `bench/BASELINE.md`, `bench/FINAL_REPORT.md`,
`bench/WORLD_BENCH.md`, `bench/JEOMSARANG_BENCH.md`,
`bench/JEOMJASESANG_CORPUS_BENCH.md`, `bench/JEOMSARANG_CORPUS_BENCH.md`,
`bench/FINAL_BENCHMARK_COMPARISON.md` 참고.

## 로마자 구간 상태

로마자표 ⠴ · 종료표 ⠲ · 연속표 ⠰ 의 상태는 `EncoderState` 의 `is_english`,
`needs_english_continuation`, `roman_number_chain` 세 플래그가 들고 있다.
**프로덕션 코드에서 이 셋을 직접 대입하지 않는다** — 전부 `rules/roman_mode.rs` 의
동사를 거친다. 그 모듈이 `debug_assert_consistent` 로 "구간이 열린 채 연속표 예약"과
"구간이 열린 채 숫자 다리" 조합을 막는다. 이 불변식을 깨면 다음 로마자 어절에서
로마자표가 잘못 붙거나 빠진다.
