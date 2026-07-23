# Braillify

한국어 텍스트를 한국 점자로 변환하는 라이브러리.

## 프로젝트 구조

- `libs/braillify/` — Rust 핵심 변환 엔진
- `packages/node/` — Node.js WASM 바인딩
- `packages/python/` — Python 바인딩 (maturin)
- `apps/landing/` — Next.js 랜딩 페이지
- `test_cases/` — 점자 변환 테스트 케이스 (JSON)
- `docs/` — 2024 개정 한국 점자 규정 PDF
- `braillove-case-collector/` — 점자 내부표기 → 숫자/유니코드 변환기

## 빌드 & 테스트

```bash
bun install
cargo build --release -p braillify
bun test                    # 전체 테스트 (Rust + Bun + Python)
bun test test_cases/         # 테스트케이스 무결성 검증만
```

## 테스트 케이스 규칙

### 파일 구조

- `test_cases/korean/rule_{N}.json` — 한글 점자 제N항
- `test_cases/korean/rule_{N}_b1.json` — 제N항 붙임 1
- `test_cases/math/math_{N}.json` — 수학 점자 제N항
- 근거: `docs/2024 개정 한국 점자 규정.pdf`

### 엔트리 형식

```json
{
  "input": "입력 텍스트 (묵자 또는 LaTeX)",
  "note": "설명 (선택, 동일 input이 여럿이거나 맥락 필요 시에만)",
  "internal": "점자 내부표기",
  "expected": "브라유셀 인덱스 연결 문자열",
  "unicode": "점자 유니코드 문자열",
  "world": "경쟁사(World) 점역 결과 (참고용, 수정 금지)",
  "jeomsarang": "경쟁사(점사랑) 점역 결과 (참고용, 수정 금지)"
}
```

### ⚠️ `world` / `jeomsarang` 필드 — 경쟁사 benchmark (NEVER MODIFY, NEVER COMPARE)

- `world`, `jeomsarang`은 **타 업체 점역 프로그램의 결과**를 그대로 보존한 참고용 필드다.
- **braillify의 정답이 아니다.** braillify의 정답은 오직 `unicode` (= `expected`)이며, PDF 규정에 근거한다.
- **절대로 수정하지 않는다.** input/internal을 정정하더라도 `world`/`jeomsarang`은 원본 그대로 둔다.
- **testcase 검수의 기준으로 사용하지 않는다.** `world`/`jeomsarang`이 우리 `unicode`와 다르더라도 testcase 오류 근거가 아니며, 그것들이 틀린 것은 braillify와 무관하다.
- **인코더 정답 비교 대상이 아니다.** `cargo test test_by_testcase`는 인코더 결과를 `expected`/`unicode`와만 비교한다.
- 이 필드의 존재 의도는 외부 점역 결과와의 차이를 관찰하기 위한 **읽기 전용 비교 자료**이지, 별도 지표가 되어서는 안 된다.

### internal → expected/unicode 변환

`braillove-case-collector/converter.py`의 패턴을 따른다:

```
pattern: " a1b'k2l@cif/msp"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\0z7(_?w]#y)="
braille: ⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿
```

특수 매핑: `` ` ``→0, `{`→42, `}`→59, `~`→24, `|`→51

`expected`는 각 문자의 인덱스를 문자열로 이어붙인 것, `unicode`는 대응하는 점자 유니코드 문자를 이어붙인 것이다.

### 무결성 검증

`test_cases/testcase-integrity.test.ts`가 모든 엔트리의 internal → expected/unicode 일치를 검증한다. 대문자(수학 변수 A, B 등)를 포함한 internal은 기본 패턴 외이므로 skip된다.

### 테스트 케이스 작성 원칙

1. **PDF가 유일한 근거** — `docs/2024 개정 한국 점자 규정.pdf`에 없는 예제를 만들지 않는다.
2. **PDF 순서 준수** — 기호 정의 → 해당 예제 순서로, PDF에 나온 순서 그대로 배치한다.
3. **기호 단독 엔트리 필수** — 각 기호는 단독 엔트리로 먼저 등록하고, 그 뒤에 해당 기호를 사용하는 예제가 온다.
4. **note는 필요할 때만** — 동일 input이 다른 의미로 쓰일 때, 또는 맥락이 필요할 때만 추가한다. input을 반복하는 note는 쓰지 않는다.
5. **소속 정확히** — 각 엔트리는 해당 항 파일에만 존재한다. 다른 항의 예제를 섞지 않는다.

### LaTeX 입력

수학 수식은 LaTeX 형식의 input도 테스트한다. 기존 엔트리의 LaTeX 버전을 추가하는 방식이다:

- 형식: `$<LaTeX 수식>$` (앞에 `$`, 뒤에 `$`)
- 동일한 `internal`/`expected`/`unicode`를 공유
- `"note": "LaTeX"` 표기
- **기존 예제의 변환만** — 새로운 수식을 만들지 않는다

```json
{
  "input": "$\\frac{3}{4}$",
  "note": "LaTeX",
  "internal": "#d/#c",
  "expected": "6025129",
  "unicode": "⠼⠙⠌⠉"
}
```

#### ⚠️ 분수는 무조건 LaTeX `\frac{}{}` 표기 (NON-NEGOTIABLE)

수식의 **분수는 반드시 LaTeX `\frac{numerator}{denominator}` 형식만 사용**한다. `/`(슬래시)는 분수 표기와 별개의 표현이며 두 가지를 혼용하지 않는다.

- ✅ `$\frac{e^x-e^{-x}}{2}$` — 분수
- ❌ `(e^x-e^{-x})/2` — 슬래시(분수 아님)
- ❌ `$(e^x-e^{-x})/2$` — LaTeX 안의 슬래시도 분수 아님

`/`는 단순 슬래시 기호 점역으로 처리되며, 분수의 의미를 가지려면 `\frac{}{}`로 명시해야 한다. testcase 작성 시 분수 의미가 있는 모든 표현을 LaTeX로 통일한다. (수학 분수는 한국 점자 점역 시 분모를 분자보다 먼저 점역하므로 `\frac{a}{b}` → `b/a` 점역 결과가 나온다.)

주요 LaTeX 변환:

| 수식 | LaTeX |
|------|-------|
| 분수 | `$\frac{분자}{분모}$` |
| 근호 | `$\sqrt{x}$`, `$\sqrt[n]{x}$` |
| 위첨자 | `$x^{2}$` |
| 아래첨자 | `$x_{n}$` |
| 부등호 | `$\neq$`, `$\geq$`, `$\leq$` |
| 절댓값 | `$\|x\|$` |
| 무한대 | `$\infty$` |
| 적분 | `$\int f(x)dx$` |
| 집합 | `$\cup$`, `$\cap$`, `$\subset$`, `$\emptyset$` |
| 논리 | `$\land$`, `$\lor$`, `$\forall$`, `$\exists$` |

### 대문자 수학 변수

수학 점자에서 대문자 변수(A, B, N 등)를 사용하는 internal은 기본 64셀 패턴에 포함되지 않는다. 이런 엔트리는 `expected`/`unicode`가 빈 문자열이며, 무결성 테스트에서 자동으로 skip된다.

## Rust 테스트 작성 가이드

> 세부 규칙은 `libs/braillify/AGENTS.md#test-organization-non-negotiable` 참조. 아래는 모든 Rust 코드에 공통 적용되는 원칙이다.

### 테스트 파일 배치 (NON-NEGOTIABLE)

테스트 코드는 다음 우선순위로 배치한다:

1. **inline `#[cfg(test)] mod tests { ... }`** — 검증 대상 `.rs` 파일 하단에 직접 작성 (기본값)
2. **owning module 안의 inline `#[cfg(test)] mod <name> { ... }`** — 여러 모듈이 공유하는 테스트 utility 는 가장 가까운 공통 부모 모듈 (보통 `lib.rs`) 에 inline 블록으로 둔다
3. **`tests/` 폴더 (integration test)** — 위 두 가지 모두 불가능한 경우 (예: 공개 API 만 사용하는 시나리오 테스트)

**금지:**

- ❌ `src/` 트리 내 **test 전용 standalone `.rs` 파일** (`src/test_helpers.rs`, `src/foo_test.rs` 등)
- ❌ production code 와 같은 파일에 `#[cfg(test)]` 가드 없이 두는 테스트 fixture

### 파라미터화 테스트는 `rstest` 우선 (NON-NEGOTIABLE)

같은 호출 shape (동일 함수 호출, 입력/기대값만 다름) 으로 **3개 이상**의 케이스가 모이면 **반드시 `rstest::rstest`** 로 변환한다.

```rust
// ✅ 권장
#[rstest::rstest]
#[case::one('1', '⠁')]
#[case::zero('0', '⠚')]
#[case::nine('9', '⠊')]
fn encodes_digits(#[case] ch: char, #[case] expected: char) {
    assert_eq!(encode_digit(ch).unwrap(), decode_unicode(expected));
}

// ❌ for-loop 테이블 패턴은 새로 추가하지 않는다
#[test]
fn encodes_digits_BAD() {
    for (ch, expected) in [('1', '⠁'), ('0', '⠚'), ('9', '⠊')] {
        assert_eq!(encode_digit(ch).unwrap(), decode_unicode(expected));
    }
}
```

**원칙:**

- `#[case::label(...)]` 의 label 은 의미를 담는다 (`#[case::lower_a]`, `#[case::overflow_max]` 등) — 실패 메시지에서 즉시 케이스 식별 가능
- 단일 입력만 다른 경우 `#[values(...)]` 사용 가능 (`fn t(#[values('a', 'b')] c: char)`)
- assertion 없는 smoke test 는 변환 의무 없음 — for-loop 가 더 간결하면 유지
- 기존 테스트를 수정할 때 3+ assertion 패턴을 발견하면 **그 자리에서 rstest 로 마이그레이션** 한다

## 구현 원칙

### 일반화 필수, 꼼수 금지 (NON-NEGOTIABLE)

변환 로직은 **PDF 규정에 근거한 일반 알고리즘**으로 작성한다. 테스트 통과가 목적이 아니라, **모든 변형 입력을 규정대로 변환하는 것**이 목적이다.

- 테스트 케이스는 가능한 입력의 작은 부분집합일 뿐이다
- 알고리즘이 옳다면 테스트는 자연히 통과한다
- "테스트와 결과가 다르니 코드를 맞춘다"가 아니라, **테스트와 결과가 다르면 알고리즘 또는 테스트 둘 중 하나가 틀린 것이다**
- 테스트에 없는 새로운 입력이 들어와도 동일한 알고리즘으로 정확히 처리되어야 한다

### 금지된 꼼수 (BLOCKING — 발견 즉시 재작성)

#### 1. 입력 → 출력 직접 매핑

```rust
// 금지
if input == "안녕하세요" { return "⠁⠉⠊..."; }

match input {
    "안녕" => "...",
    "학교" => "...",
    _ => fallback(),
}
```

#### 2. 테스트 케이스 룩업 테이블

```rust
// 금지 — 테스트 입력/출력을 그대로 박아넣은 것
const KNOWN: &[(&str, &str)] = &[
    ("안녕", "⠁⠉..."),
    ("학교", "⠚⠁..."),
];
```

#### 3. expected 역산

테스트 JSON의 `expected`/`unicode` 값을 보고 **그 값이 나오도록 코드를 작성하는 것**. 알고리즘은 오직 `docs/2024 개정 한국 점자 규정.pdf`의 규정에서만 도출한다.

#### 4. 테스트 파일 의존

변환 로직이 `test_cases/` 경로의 파일을 읽거나 import하거나 참조하는 코드 일체. 테스트 데이터는 검증 단계에서만 쓰인다.

#### 5. 케이스별 분기 폭증

같은 종류의 처리를 입력 단위마다 if/else 또는 match로 늘어놓는 것. 같은 규정이 적용되는 입력은 **하나의 일반 함수**로 처리한다.

```rust
// 금지 — 음절별로 결과를 박아넣는 패턴
fn convert_syllable(s: &str) -> &str {
    match s {
        "각" => "⠊⠁⠁",
        "간" => "⠊⠁⠉",
        "갈" => "⠊⠁⠂",
        // ... 수천 줄
    }
}

// 올바름 — 초성/중성/종성 분해 후 규정대로 조합
fn convert_syllable(s: char) -> Vec<BrailleCell> {
    let (cho, jung, jong) = decompose_hangul(s);
    let mut out = vec![];
    out.extend(cho_to_braille(cho));
    out.extend(jung_to_braille(jung));
    if let Some(j) = jong { out.extend(jong_to_braille(j)); }
    out
}
```

> **예외:** 단일 자모/기호의 점형 정의(예: ㄱ → ⠈, 숫자 표시 → ⠼)는 PDF가 명시한 기본 매핑이므로 허용된다. **음절/단어/구절 단위의 매핑은 모두 금지.**

### 올바른 구현 방향

1. **PDF 규정 → 알고리즘** — 각 항(제N항)을 함수로 분리하고, 함수 doc에 근거 항 번호를 명시한다
2. **계층 분해** — 자모 → 음절 → 단어 → 문장 순으로 단계화하여 각 층은 자기 책임만 진다
3. **테스트는 검증 도구** — 알고리즘이 PDF 규정과 일치하는지 확인하는 용도이지, 코드가 맞춰야 할 정답표가 아니다
4. **테스트 추가 시 코드 수정 불필요** — 같은 규정에 속하는 새 예제는 이미 알고리즘이 처리하고 있어야 한다. 새 예제 추가만으로 코드 변경이 필요하다면 알고리즘이 일반화되지 않은 것이다

### 자가 검증 체크리스트

PR/커밋 전 다음을 모두 확인한다:

- [ ] 변환 로직 안에 `test_cases/` 경로 문자열이 없다
- [ ] 테스트의 `input` 문자열이 코드에 리터럴로 등장하지 않는다 (단일 자모/기호 제외)
- [ ] 테스트의 `expected`/`unicode` 문자열이 코드에 리터럴로 등장하지 않는다
- [ ] 같은 규정의 새 예제를 추가해도 코드 수정 없이 통과한다
- [ ] 음절/단어 단위 분기가 자모 단위 일반 함수로 통합되어 있다
- [ ] 모든 분기와 매핑은 PDF의 특정 항을 근거로 추적 가능하다
