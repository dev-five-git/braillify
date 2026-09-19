# 점사랑 (BrailleTransLibrary) 정답률 벤치마크

- 측정일: 2026-09-07
- 비교 기준: PDF 규정 (2024 개정 한국 점자 규정)
  - PDF 정답 = test_cases JSON 의 `unicode` 필드
  - 점사랑 결과 = test_cases JSON 의 `jeomsarang` 필드 (BrailleTransLibrary DLL 로 직접 점역)
- 비교 방식: 단순 유니코드 문자열 동치 (`jeomsarang === unicode`)
- Skip 정책: LaTeX 변형, 빈 input, jeomsarang 미수집, unicode 미정의 항목 제외

## 전체 요약

| 항목 | 값 |
|---|---:|
| 전체 testcase | 5141 |
| 측정 대상 | 4781 |
| 제외 (LaTeX) | 351 |
| 제외 (빈 input) | 0 |
| 제외 (jeomsarang 미수집) | 9 |
| 제외 (unicode 없음) | 0 |
| **점사랑 PDF 정답 일치** | **1698 (35.52%)** |
| **점사랑 PDF 정답 불일치** | **3083 (64.48%)** |

> 참고 — braillify 의 PDF 정답 일치: **2419/2419 = 100.00%** (cargo test test_by_testcase).

## 카테고리별

| 카테고리 | 전체 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|---:|
| english/ | 2722 | 2716 | 294 | 2422 | 10.82% |
| korean/ | 1527 | 1515 | 1265 | 250 | 83.50% |
| math/ | 892 | 550 | 139 | 411 | 25.27% |

## 파일별 (상위 30개, 일치율 낮은 순)

| 파일 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|
| english/rule_10_10_2.json | 20 | 0 | 20 | 0.00% |
| english/rule_10_10_3.json | 17 | 0 | 17 | 0.00% |
| english/rule_10_10_4.json | 19 | 0 | 19 | 0.00% |
| english/rule_10_10_5.json | 9 | 0 | 9 | 0.00% |
| english/rule_10_10_6.json | 7 | 0 | 7 | 0.00% |
| english/rule_10_10_7.json | 19 | 0 | 19 | 0.00% |
| english/rule_10_10_8.json | 12 | 0 | 12 | 0.00% |
| english/rule_10_10_9.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_11_3.json | 10 | 0 | 10 | 0.00% |
| english/rule_10_11_6.json | 20 | 0 | 20 | 0.00% |
| english/rule_10_11_8.json | 8 | 0 | 8 | 0.00% |
| english/rule_10_11_9.json | 22 | 0 | 22 | 0.00% |
| english/rule_10_12_11.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_12_14.json | 18 | 0 | 18 | 0.00% |
| english/rule_10_12_15.json | 5 | 0 | 5 | 0.00% |
| english/rule_10_12_17.json | 13 | 0 | 13 | 0.00% |
| english/rule_10_12_3.json | 12 | 0 | 12 | 0.00% |
| english/rule_10_13_1.json | 7 | 0 | 7 | 0.00% |
| english/rule_10_13_10.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_11.json | 8 | 0 | 8 | 0.00% |
| english/rule_10_13_12.json | 18 | 0 | 18 | 0.00% |
| english/rule_10_13_3.json | 6 | 0 | 6 | 0.00% |
| english/rule_10_13_4.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_5.json | 6 | 0 | 6 | 0.00% |
| english/rule_10_13_6.json | 2 | 0 | 2 | 0.00% |
| english/rule_10_13_7.json | 2 | 0 | 2 | 0.00% |
| english/rule_10_13_8.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_9.json | 9 | 0 | 9 | 0.00% |
| english/rule_10_2_1.json | 19 | 0 | 19 | 0.00% |
| english/rule_10_2_2.json | 6 | 0 | 6 | 0.00% |

## 해석

이 측정은 점사랑 7.0 의 PDF 규정 준수도에 대한 객관적 지표이다.
일치하지 않는 testcase 는 점사랑 결과가 2024 개정 한국 점자 규정과 다르다는 의미이며,
braillify 의 정답성과는 무관하다 (braillify 알고리즘은 점사랑 결과를 참조하지 않는다 — AGENTS.md RED LINE).

상세 미스매치 목록은 [`JEOMSARANG_MISMATCHES.md`](./JEOMSARANG_MISMATCHES.md) 참고.
