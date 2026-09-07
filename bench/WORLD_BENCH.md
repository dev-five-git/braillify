# 점자세상 (braillekorea.org) 정답률 벤치마크

- 측정일: 2026-08-31
- 비교 기준: PDF 규정 (2024 개정 한국 점자 규정)
  - PDF 정답 = test_cases JSON 의 `unicode` 필드
  - 점자세상 결과 = test_cases JSON 의 `world` 필드 (fetch-world.ts 가 braillekorea.org API 에서 수집)
- 비교 방식: 유니코드 문자열 동치. 입력 경계가 영문자인 경우에만 API의 외곽 영문 시작(⠴)/끝(⠲) 표지를 비교에서 제외한다.
- Skip 정책: LaTeX 변형, 빈 input, world 미수집, unicode 미정의 항목 제외

## 전체 요약

| 항목 | 값 |
|---|---:|
| 전체 testcase | 5141 |
| 측정 대상 | 4619 |
| 제외 (LaTeX) | 351 |
| 제외 (빈 input) | 0 |
| 제외 (world 미수집) | 171 |
| 제외 (unicode 없음) | 0 |
| **점자세상 PDF 정답 일치** | **2050 (44.38%)** |
| **점자세상 PDF 정답 불일치** | **2569 (55.62%)** |

> 참고 — braillify 의 PDF 정답 일치: **2419/2419 = 100.00%** (cargo test test_by_testcase).
> 단, braillify 측정에는 `KNOWN_FAILURES` 라우팅이 포함되어 있어 raw encode 정답률은 별도 측정 필요.

## 카테고리별

| 카테고리 | 전체 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|---:|
| english/ | 2722 | 2680 | 1379 | 1301 | 51.46% |
| korean/ | 1527 | 1465 | 602 | 863 | 41.09% |
| math/ | 892 | 474 | 69 | 405 | 14.56% |

## 파일별 (상위 30개, 일치율 낮은 순)

| 파일 | 측정 | 일치 | 불일치 | 일치율 |
|---|---:|---:|---:|---:|
| english/rule_10_12_15.json | 5 | 0 | 5 | 0.00% |
| english/rule_10_12_5.json | 18 | 0 | 18 | 0.00% |
| english/rule_10_13_1.json | 7 | 0 | 7 | 0.00% |
| english/rule_10_13_10.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_11.json | 8 | 0 | 8 | 0.00% |
| english/rule_10_13_12.json | 18 | 0 | 18 | 0.00% |
| english/rule_10_13_2.json | 28 | 0 | 28 | 0.00% |
| english/rule_10_13_3.json | 6 | 0 | 6 | 0.00% |
| english/rule_10_13_4.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_5.json | 6 | 0 | 6 | 0.00% |
| english/rule_10_13_6.json | 2 | 0 | 2 | 0.00% |
| english/rule_10_13_7.json | 2 | 0 | 2 | 0.00% |
| english/rule_10_13_8.json | 4 | 0 | 4 | 0.00% |
| english/rule_10_13_9.json | 9 | 0 | 9 | 0.00% |
| english/rule_10_5_4.json | 5 | 0 | 5 | 0.00% |
| english/rule_10_8_2.json | 5 | 0 | 5 | 0.00% |
| english/rule_10_9_7.json | 6 | 0 | 6 | 0.00% |
| english/rule_10_9_9.json | 3 | 0 | 3 | 0.00% |
| english/rule_11_2_2.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_3_1.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_3_2.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_3_3.json | 1 | 0 | 1 | 0.00% |
| english/rule_11_3_4.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_4_2.json | 4 | 0 | 4 | 0.00% |
| english/rule_11_4_3.json | 3 | 0 | 3 | 0.00% |
| english/rule_11_5_1.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_5_2.json | 1 | 0 | 1 | 0.00% |
| english/rule_11_6_1.json | 2 | 0 | 2 | 0.00% |
| english/rule_11_6_3.json | 1 | 0 | 1 | 0.00% |
| english/rule_11_7_1.json | 3 | 0 | 3 | 0.00% |

## 해석

이 측정은 점자세상의 PDF 규정 준수도에 대한 객관적 지표이다.
일치하지 않는 testcase는 점자세상 결과가 2024 개정 한국 점자 규정과 다르다는 의미이며,
braillify 의 정답성과는 무관하다 (braillify 알고리즘은 점자세상 결과를 참조하지 않는다 — AGENTS.md RED LINE).

상세 미스매치 목록은 [`WORLD_MISMATCHES.md`](./WORLD_MISMATCHES.md) 참고.
