# 점사랑 7.0 (BrailleLove) 정답률 벤치마크

- 측정일: 2026-05-22
- 비교 기준: PDF 규정 (2024 개정 한국 점자 규정)
  - PDF 정답 = test_cases JSON 의 `unicode` 필드
  - 점사랑 결과 = test_cases JSON 의 `jeomsarang` 필드 (fetch-jeomsarang.py 가 GUI 자동화로 수집)
- 비교 방식: 단순 유니코드 문자열 동치 (`jeomsarang === unicode`)
- Skip 정책: LaTeX 변형, 빈 input, jeomsarang 미수집, unicode 미정의 항목 제외

## 전체 요약

| 항목 | 값 |
|---|---:|
| 전체 testcase | 2419 |
| 측정 대상 | 2002 |
| 제외 (LaTeX) | 351 |
| 제외 (빈 input) | 0 |
| 제외 (jeomsarang 미수집) | 66 |
| 제외 (unicode 없음) | 0 |
| **점사랑 PDF 정답 일치** | **1362 (68.03%)** |
| **점사랑 PDF 정답 불일치** | **640 (31.97%)** |

> 참고 — braillify 의 PDF 정답 일치: **2419/2419 = 100.00%** (cargo test test_by_testcase).

## 카테고리별

| 카테고리 | 전체 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|---:|
| korean/ | 1527 | 1513 | 1229 | 284 | 81.23% |
| math/ | 892 | 489 | 133 | 356 | 27.20% |

## 파일별 (상위 30개, 일치율 낮은 순)

| 파일 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|
| korean/rule_27.json | 7 | 0 | 7 | 0.00% |
| korean/rule_28.json | 64 | 0 | 64 | 0.00% |
| korean/rule_30.json | 52 | 0 | 52 | 0.00% |
| korean/rule_39.json | 3 | 0 | 3 | 0.00% |
| korean/rule_53_b1.json | 1 | 0 | 1 | 0.00% |
| korean/rule_56.json | 5 | 0 | 5 | 0.00% |
| korean/rule_59.json | 1 | 0 | 1 | 0.00% |
| korean/rule_67.json | 2 | 0 | 2 | 0.00% |
| korean/rule_73.json | 2 | 0 | 2 | 0.00% |
| korean/rule_73_b1.json | 4 | 0 | 4 | 0.00% |
| math/math_11.json | 5 | 0 | 5 | 0.00% |
| math/math_13.json | 62 | 0 | 62 | 0.00% |
| math/math_16.json | 4 | 0 | 4 | 0.00% |
| math/math_17.json | 4 | 0 | 4 | 0.00% |
| math/math_19.json | 8 | 0 | 8 | 0.00% |
| math/math_21.json | 2 | 0 | 2 | 0.00% |
| math/math_22.json | 8 | 0 | 8 | 0.00% |
| math/math_23.json | 6 | 0 | 6 | 0.00% |
| math/math_25.json | 3 | 0 | 3 | 0.00% |
| math/math_29.json | 2 | 0 | 2 | 0.00% |
| math/math_30.json | 2 | 0 | 2 | 0.00% |
| math/math_31.json | 2 | 0 | 2 | 0.00% |
| math/math_32.json | 2 | 0 | 2 | 0.00% |
| math/math_35.json | 3 | 0 | 3 | 0.00% |
| math/math_36.json | 2 | 0 | 2 | 0.00% |
| math/math_37.json | 2 | 0 | 2 | 0.00% |
| math/math_38.json | 3 | 0 | 3 | 0.00% |
| math/math_42.json | 3 | 0 | 3 | 0.00% |
| math/math_45.json | 4 | 0 | 4 | 0.00% |
| math/math_46.json | 8 | 0 | 8 | 0.00% |

## 해석

이 측정은 점사랑 7.0 의 PDF 규정 준수도에 대한 객관적 지표이다.
일치하지 않는 testcase 는 점사랑 결과가 2024 개정 한국 점자 규정과 다르다는 의미이며,
braillify 의 정답성과는 무관하다 (braillify 알고리즘은 점사랑 결과를 참조하지 않는다 — AGENTS.md RED LINE).

상세 미스매치 목록은 [`JEOMSARANG_MISMATCHES.md`](./JEOMSARANG_MISMATCHES.md) 참고.
