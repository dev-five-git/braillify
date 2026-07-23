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
cargo test                           # All tests (390+ unit + 14 integration)
cargo test test_by_testcase          # Full testcase suite (2419 cases)
cargo fmt && cargo clippy            # Format + lint
bun test test_cases/                 # JSON integrity checks (14163 assertions)
```

Test cases in `test_cases/korean/*.json` and `test_cases/math/*.json`.

**Current status: 2419/2419 passing (100% PDF 규정 준수, 0 known failures).**

`KNOWN_FAILURES` 상수는 더 이상 존재하지 않는다. raw `encode()` 가 모든 testcase 에서
PDF 정답과 byte-동일 결과를 낸다. 새로 추가되는 testcase 도 같은 기준을 만족해야 한다.

## BENCHMARK

```bash
# 마이크로 벤치 (criterion) — Wave 0 인프라
cargo bench -p braillify --bench encode_native
cargo bench -p braillify --bench encode_math

# 메모리 프로파일 (dhat)
cargo bench -p braillify --bench memory_dhat --features dhat-heap

# 외부 점역기 비교 (점자세상 / 점사랑 7.0)
bun run scripts/world-bench.ts        # PDF 정답 일치율 측정
bun run scripts/jeomsarang-bench.ts
```

벤치 결과: `bench/BASELINE.md`, `bench/FINAL_REPORT.md`,
`bench/WORLD_BENCH.md`, `bench/JEOMSARANG_BENCH.md`,
`bench/FINAL_BENCHMARK_COMPARISON.md` 참고.
