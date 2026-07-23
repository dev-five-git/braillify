# Braillify Performance Baseline — Wave 0

- Date: 2026-05-21
- Host CPU: AMD Ryzen 9 9950X 16-Core Processor
- OS: Microsoft Windows 11 Pro 10.0.26200
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)`
- cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Release profile: `opt-level = "s"`, `debug = 1`
- Criterion baseline: `phase1`

## Criterion results

Median values are from `target/criterion/**/phase1/estimates.json`.

| Benchmark | Median ns/op | Throughput |
|---|---:|---:|
| criterion/encode_math_concat/all | 139817 | 3.765 MiB/s |
| criterion/encode_math_latex_lines/00 | 8297 | 1.954 MiB/s |
| criterion/encode_math_latex_lines/01 | 6817 | 2.238 MiB/s |
| criterion/encode_math_latex_lines/02 | 20579 | 1.112 MiB/s |
| criterion/encode_math_latex_lines/03 | 10365 | 3.036 MiB/s |
| criterion/encode_math_latex_lines/04 | 3988 | 2.152 MiB/s |
| criterion/encode_math_latex_lines/05 | 3716 | 2.310 MiB/s |
| criterion/encode_math_latex_lines/06 | 5644 | 3.380 MiB/s |
| criterion/encode_math_latex_lines/07 | 3919 | 3.406 MiB/s |
| criterion/encode_math_latex_lines/08 | 4950 | 2.698 MiB/s |
| criterion/encode_math_latex_lines/09 | 5461 | 2.619 MiB/s |
| criterion/encode_math_latex_lines/10 | 5741 | 3.156 MiB/s |
| criterion/encode_math_latex_lines/11 | 5815 | 3.116 MiB/s |
| criterion/encode_math_latex_lines/12 | 5713 | 3.506 MiB/s |
| criterion/encode_math_latex_lines/13 | 5167 | 3.138 MiB/s |
| criterion/encode_math_latex_lines/14 | 5166 | 4.430 MiB/s |
| criterion/encode_math_latex_lines/15 | 3766 | 2.279 MiB/s |
| criterion/encode_math_latex_lines/16 | 5455 | 1.574 MiB/s |
| criterion/encode_math_latex_lines/17 | 5493 | 2.083 MiB/s |
| criterion/encode_math_latex_lines/18 | 4356 | 3.722 MiB/s |
| criterion/encode_math_latex_lines/19 | 6133 | 4.198 MiB/s |
| criterion/encode_math_latex_lines/20 | 4241 | 3.598 MiB/s |
| criterion/encode_math_latex_lines/21 | 4225 | 2.257 MiB/s |
| criterion/encode_math_latex_lines/22 | 3823 | 2.245 MiB/s |
| criterion/encode_math_latex_lines/23 | 6240 | 3.362 MiB/s |
| criterion/encode_math_latex_lines/24 | 5500 | 2.947 MiB/s |
| criterion/encode_math_latex_lines/25 | 8135 | 3.869 MiB/s |
| criterion/encode_math_latex_lines/26 | 4989 | 2.867 MiB/s |
| criterion/encode_math_latex_lines/27 | 5658 | 3.371 MiB/s |
| criterion/encode_math_latex_lines/28 | 3970 | 3.603 MiB/s |
| criterion/encode_math_latex_lines/29 | 12998 | 1.541 MiB/s |
| criterion/encode_prose/kim_sowol | 1011894 | 1.318 MiB/s |
| criterion/encode_prose/kim_yujeong | 1869861 | 0.674 MiB/s |
| criterion/encode_prose/synth_100k | 18048223150 | 0.012 MiB/s |
| criterion/encode_prose/synth_10k | 150291061 | 0.150 MiB/s |
| criterion/encode_prose/synth_1k | 2465147 | 0.924 MiB/s |
| criterion/encode_short/greet | 7372 | 1.940 MiB/s |
| criterion/encode_short/mixed | 12495 | 1.832 MiB/s |
| criterion/encode_short/name | 7707 | 2.227 MiB/s |
| criterion/encode_short/punct | 9749 | 3.326 MiB/s |
| criterion/encode_to_unicode/synth_1k | 3288900 | 0.692 MiB/s |

## DHAT heap profile

Source: `dhat-heap.json` (copied from Cargo bench package cwd `libs/braillify/dhat-heap.json`), produced by `cargo bench -p braillify --bench memory_dhat --features dhat-heap`.

| Metric | Value |
|---|---:|
| totalBytes | 260132822 |
| totalBlocks | 2502833 |
| atTGmaxBytes | 464533 |
| atTGmaxBlocks | 1798 |
| atTEndBytes | 800 |
| atTEndBlocks | 23 |

## Binary sizes

| Artifact | Size bytes |
|---|---:|
| `target/release/braillify.exe` | 2835456 |
| `target/wasm32-unknown-unknown/release/node.wasm` | 9511449 |
| `target/release/braillify_native.dll` | 2066432 |

## Correctness and quality baseline

| Gate | Result |
|---|---|
| `cargo build -p braillify` | pass |
| `cargo build --release -p braillify` | pass |
| `cargo test -p braillify --release test_by_testcase -- --nocapture` | 2419/2419 pass, 0 fail, 0 skip |
| `bun test test_cases/` | 14163 pass, 0 fail |
| `cargo clippy --release -p braillify --all-targets` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo bench -p braillify --bench encode_native -- --save-baseline phase1` | pass, produced `target/criterion/` |
| `cargo bench -p braillify --bench encode_math -- --save-baseline phase1` | pass |
| `cargo bench -p braillify --bench memory_dhat --features dhat-heap` | pass, produced `libs/braillify/dhat-heap.json` and root `dhat-heap.json` |
