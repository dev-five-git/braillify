import { spawnSync } from "node:child_process";

// Every platform measures coverage (cargo-tarpaulin) and emits Cobertura XML,
// which each CI job uploads to codecov. codecov MERGES the per-OS reports for a
// commit, and a line counts as covered if ANY report covered it — so the
// authoritative project number is the union across platforms.
//
// Why the union matters: cargo-tarpaulin's LLVM engine is deterministic on
// Linux/macOS (a stable 100%), but on Windows the LLVM instrumentation runtime
// miscounts a shifting ~15-line set on every run (upstream LLVM/rustc bug, see
// https://github.com/rust-lang/rust/issues/77553 and LLVM #74086). It is
// toolchain-independent (windows-gnu flickers identically) and NOT a real gap:
// the same tests run and the same lines execute on every OS. The merge cancels
// that Windows flicker, so codecov reports a genuine 100%.
//
// We therefore enforce `--fail-under 100` only on Linux, where the number is
// deterministic — that fails CI fast on an actual coverage regression. Windows
// and macOS still run tarpaulin and upload (feeding the merge) but must not fail
// CI on the recorder's flicker, so they run without the threshold.
const enforceThreshold = process.platform === "linux";

const cargoArgs = [
  "tarpaulin",
  "--engine",
  "llvm",
  "--out",
  "xml",
  "--out",
  "stdout",
  "--timeout",
  "300",
  ...(enforceThreshold ? ["--fail-under", "100"] : []),
  "--",
  "--skip",
  "test_by_testcase",
];

const result = spawnSync("cargo", cargoArgs, { stdio: "inherit" });
if (result.error) {
  throw result.error;
}
process.exit(result.status ?? 1);
