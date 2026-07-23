# Braille 점역기 종합 비교 벤치마크

> braillify / 점사랑 7.0 / 점자세상 의 **2024 개정 한국 점자 규정** 준수도 객관 비교

---

## Executive Summary

| 점역기 | PDF 정답 일치율 | 측정 / 일치 | korean/ | math/ |
|---|---:|---|---:|---:|
| **braillify** | **100.00%** | 2419 / 2419 | **100.00%** | **100.00%** |
| **점사랑 7.0** | **68.03%** | 2002 / 1362 | 81.23% | 27.20% |
| **점자세상** | **32.23%** | 1939 / 625 | 38.43% | 13.08% |

braillify 는 두 외부 점역기를 모든 영역에서 압도한다. 특히 수학 점자 영역의 격차가 크다 (점사랑 27%, 점자세상 13% vs braillify 100%).

---

## 측정 방법론

| 항목 | 값 |
|---|---|
| **기준** | 2024 개정 한국 점자 규정 (`docs/2024 개정 한국 점자 규정.pdf`) |
| **PDF 정답** | `test_cases/**/*.json` 의 `unicode` 필드 (유니코드 점자 문자열) |
| **비교 방식** | 단순 유니코드 문자열 동치 (`output === unicode`) |
| **전체 testcase** | 2419 (한글 1527 + 수학 892) |
| **공통 skip** | LaTeX 변형 351건 (동일 input 의 LaTeX 형식 — 의미적 중복) |

각 점역기별 수집 방식:

| 점역기 | 수집 방식 | 소요 시간 | 측정 가능 entry |
|---|---|---:|---:|
| braillify | 로컬 Rust `braillify::encode()` 직접 호출 | <1s | 2419 / 2419 (100%) |
| 점자세상 | HTTPS API (`braillekorea.org/braille/brailleProcAjax.do`) + 병렬 8 fetch | 64s | 1939 / 2068 (94%) |
| 점사랑 7.0 | Windows GUI 자동화 (pywinauto + win32 backend) | 162min | 2002 / 2068 (97%) |

> 점자세상 미수집 129건: API resultCode≠0 (점자세상이 처리 거부), 일부 특수기호만 있는 입력.
> 점사랑 미수집 66건: LaTeX 수식 입력 시 `{` 문자 키 입력 escape 한계 + 일부 입력 거부.

---

## 카테고리별 상세 비교

### korean/ — 한글 점자 (1527 testcase)

| 점역기 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|
| braillify | 1527 | 1527 | 0 | **100.00%** |
| 점사랑 7.0 | 1513 | 1229 | 284 | **81.23%** |
| 점자세상 | 1465 | 563 | 902 | **38.43%** |

### math/ — 수학 점자 (892 testcase)

| 점역기 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|
| braillify | 892 | 892 | 0 | **100.00%** |
| 점사랑 7.0 | 489 | 133 | 356 | **27.20%** |
| 점자세상 | 474 | 62 | 412 | **13.08%** |

수학 점자에서 두 외부 점역기 모두 4분의 1 ~ 8분의 1 수준으로 떨어진다. 한국 점자 규정의 수학 영역(제51-66항 등)이 가장 최근 개정된 부분이라 외부 점역기들이 미반영한 항목이 많은 것으로 해석된다.

---

## 외부 점역기가 0% 일치인 한글 testcase 파일 (점자세상 기준)

`bench/WORLD_BENCH.md` 의 파일별 일치율 상위 30 중 0% 인 파일들 (점자세상이 한 건도 PDF 정답과 일치하지 않은 한글 규정):

- rule_10, 12_b1, 14_b1, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 37, 38, 39, 42, 44_b1, 46, 47, 48, 51, 53_b1, 54

대다수가 한국 점자 규정의 한글 본문 규정 (제19-54항: 점역 약자, 옛한글, 영어 혼용 등) 및 일부 특수 처리 영역. braillify 는 이 영역 전부를 PDF 그대로 구현했다.

---

## 미스매치 샘플 (대표 예시)

상세 미스매치 목록: [`WORLD_MISMATCHES.md`](./WORLD_MISMATCHES.md), [`JEOMSARANG_MISMATCHES.md`](./JEOMSARANG_MISMATCHES.md)

### 예시 1: 한글 약어 처리

- input: `그래서`
- PDF 정답: `⠁⠎`
- braillify: `⠁⠎` ✓
- 외부 점역기: 종종 약어 미적용 → 전체 음절 풀어쓰기로 차이 발생

### 예시 2: 수학 분수

- input: `$\frac{3}{4}$` (LaTeX 표기, braillify 전용 입력)
- PDF 정답: `⠼⠙⠌⠉` (점자 분수 표기)
- braillify: `⠼⠙⠌⠉` ✓
- 외부 점역기: LaTeX 자체를 그대로 점역하거나 처리 거부 → 측정 대상에서 skip (LaTeX 동일 input 의 묵자 표기 버전으로만 비교)

---

## 측정 환경

- 측정일: 2026-05-22
- 호스트: AMD Ryzen 9 9950X, Microsoft Windows 11 Pro
- braillify: rustc 1.95.0 (release, opt-level=3, LTO thin)
- 점자세상: API 응답 (네트워크 의존, 일시 거부 3건은 preserve 정책으로 이전 값 유지)
- 점사랑: BrailleLove.exe 7.0 (`C:\Program Files (x86)\Jeomsarang7\`)

---

## 결론

1. **braillify 는 2024 개정 한국 점자 규정을 100% 충족**한다 (2419/2419, 0 known failures).
2. 두 외부 점역기 모두 PDF 규정 준수도가 낮으며, 특히 수학 점자 영역의 격차가 크다.
3. 점사랑 7.0 (68%) > 점자세상 (32%) — 점사랑이 점자세상보다 약 2.1배 정확.
4. 본 측정은 PDF 규정 준수도만 평가한다. 사용성, UI/UX, 인쇄 기능 등 다른 평가 축은 포함하지 않는다.
5. 외부 점역기와의 불일치는 외부 점역기의 PDF 미반영을 의미하며, braillify 의 정답성 검증과는 독립적이다 (braillify 알고리즘은 외부 점역기 출력을 참조하지 않는다 — AGENTS.md RED LINE).

---

## 재현 방법

```bash
# 1. 점자세상 (HTTP API, ~1분, CSRF 토큰 자동 갱신)
bun run scripts/fetch-world.ts        # test_cases JSON 의 world 필드 갱신
bun run scripts/world-bench.ts        # 정답률 분석 → bench/WORLD_BENCH.md

# 2. 점사랑 7.0 (GUI 자동화, ~2-3시간, PC 점유)
cd braillove-case-collector
uv run python ../scripts/fetch-jeomsarang.py    # jeomsarang 필드 갱신
bun run scripts/jeomsarang-bench.ts             # 정답률 분석 → bench/JEOMSARANG_BENCH.md

# 3. braillify 자체 검증
cd libs/braillify && cargo test --release test_by_testcase    # 2419/2419 ✓
bun test test_cases/                                          # 14163 ✓
```
