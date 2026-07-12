#[cfg(feature = "cli")]
use std::env;

#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use braillify::cli::run_cli;

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    run_cli(env::args().collect())
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    use std::io::Write;
    use std::sync::OnceLock;

    use assert_cmd::assert::OutputAssertExt;
    use predicates::prelude::*;

    // 빌드를 한 번만 수행하고 재사용
    static BUILT_BINARY: OnceLock<escargot::CargoRun> = OnceLock::new();

    /// Generic retry-with-backoff: invokes `try_once` up to `max_attempts`
    /// times, sleeping `backoff_ms(attempt)` ms between failures. The final
    /// `Err` is returned. Extracted as a pure function so its retry/Err logic
    /// is directly unit-testable (rather than buried inside `get_built_binary`).
    fn retry_with_backoff<T, E, F, G>(
        max_attempts: u32,
        mut try_once: F,
        backoff_ms: G,
    ) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        G: Fn(u32) -> u64,
    {
        let mut last = None;
        for attempt in 1..=max_attempts {
            match try_once() {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last = Some(e);
                    if attempt < max_attempts {
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms(attempt)));
                    }
                }
            }
        }
        Err(last.expect("Err arm guarantees Some on at least one iteration"))
    }

    /// Panic helper for `get_built_binary` when all retries are exhausted.
    /// Extracted so it can be directly tested via `#[should_panic]` without
    /// relying on actual build failure.
    fn panic_build_failed(err: &dyn std::fmt::Debug) -> ! {
        panic!(
            "Failed to build braillify binary for testing after 3 attempts. Last error: {err:?}. This may happen on the first test run. Try running 'cargo build --bin braillify' manually first."
        )
    }

    fn build_backoff_ms(attempt: u32) -> u64 {
        200 * u64::from(attempt)
    }

    fn get_built_binary() -> &'static escargot::CargoRun {
        // 재시도 로직: 첫 번째 테스트에서 빌드가 실패할 수 있으므로 재시도
        BUILT_BINARY.get_or_init(|| {
            retry_with_backoff(
                3,
                || {
                    escargot::CargoBuild::new()
                        .bin("braillify")
                        .current_release()
                        .current_target()
                        .run()
                },
                build_backoff_ms,
            )
            .unwrap_or_else(|e| panic_build_failed(&e))
        })
    }

    /// Directly invokes `panic_build_failed` so the panic branch is attributed.
    #[test]
    #[should_panic(expected = "Failed to build braillify binary")]
    fn panic_build_failed_emits_message() {
        panic_build_failed(&"synthetic-error-for-coverage");
    }

    /// `retry_with_backoff` returns Ok immediately on first success.
    #[test]
    fn retry_succeeds_on_first_attempt() {
        let result: Result<i32, ()> = retry_with_backoff(3, || Ok(42), |_| 0);
        assert_eq!(result, Ok(42));
    }

    /// `retry_with_backoff` continues retrying through Err until success.
    /// Drives the `Err(e) => { last_error = Some(e); if attempt < max ... }`
    /// branch and the `Ok(v) => return Ok(v)` arm after multiple failures.
    #[test]
    fn retry_succeeds_after_two_failures() {
        let mut tries = 0;
        let result: Result<i32, &'static str> = retry_with_backoff(
            3,
            || {
                tries += 1;
                if tries < 3 { Err("not yet") } else { Ok(tries) }
            },
            |_| 0,
        );
        assert_eq!(result, Ok(3));
    }

    /// `retry_with_backoff` returns the final Err after exhausting attempts.
    /// Drives the `Err(last.expect(...))` final-return path.
    #[test]
    fn retry_returns_final_error_after_max_attempts() {
        let mut tries = 0;
        let result: Result<i32, &'static str> = retry_with_backoff(
            3,
            || {
                tries += 1;
                Err("always fails")
            },
            |_| 0,
        );
        assert_eq!(result, Err("always fails"));
        assert_eq!(tries, 3);
    }

    /// `retry_with_backoff` honours the backoff function (called for each
    /// retry except the last). Sleeps with 0ms here for test speed; we verify
    /// the function is invoked the right number of times.
    #[test]
    fn retry_backoff_invoked_for_intermediate_attempts() {
        use std::cell::RefCell;
        let backoffs: RefCell<Vec<u32>> = RefCell::new(Vec::new());
        let mut tries = 0;
        let _: Result<(), ()> = retry_with_backoff(
            3,
            || {
                tries += 1;
                Err(())
            },
            |attempt| {
                backoffs.borrow_mut().push(attempt);
                0
            },
        );
        // For max_attempts=3, backoff is called at attempt=1 and attempt=2,
        // not at attempt=3 (the last one).
        assert_eq!(*backoffs.borrow(), vec![1, 2]);
    }

    #[test]
    fn build_backoff_scales_by_attempt() {
        assert_eq!(build_backoff_ms(3), 600);
    }

    // assert_cmd를 사용한 통합 테스트들
    #[test]
    fn test_braillify_integration_single_word() {
        let mut cmd = get_built_binary().command();
        cmd.arg("안녕");
        let assert = cmd
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());

        // 점자 유니코드가 포함되어 있는지 확인
        let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
        assert!(
            stdout
                .chars()
                .any(|c| c as u32 >= 0x2800 && c as u32 <= 0x28FF)
        );
    }

    #[test]
    fn test_braillify_integration_english() {
        let mut cmd = get_built_binary().command();
        cmd.arg("hello");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_integration_mixed() {
        let mut cmd = get_built_binary().command();
        cmd.arg("안녕 hello");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_integration_numbers() {
        let mut cmd = get_built_binary().command();
        cmd.arg("123");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_pipe_input() {
        let mut cmd = get_built_binary().command();
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all("안녕\n".as_bytes()).unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
    }

    #[test]
    fn test_braillify_help() {
        let mut cmd = get_built_binary().command();
        cmd.arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("한국어 점자 변환 CLI"));
    }

    #[test]
    fn test_braillify_version() {
        let mut cmd = get_built_binary().command();
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("braillify"));
    }

    #[test]
    fn test_braillify_no_args() {
        let mut cmd = get_built_binary().command();
        // 인자 없이 실행하면 REPL 모드로 진입
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all("안녕\n".as_bytes()).unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("braillify REPL"));
    }

    #[test]
    fn test_braillify_empty_input() {
        let mut cmd = get_built_binary().command();
        cmd.arg("");
        cmd.assert().success().stdout(predicate::str::is_empty());
    }

    #[test]
    fn test_braillify_long_text() {
        let long_text = "안녕하세요 ".repeat(100);
        let mut cmd = get_built_binary().command();
        cmd.arg(&long_text);
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_special_characters() {
        let mut cmd = get_built_binary().command();
        cmd.arg("!@#$%^&*()");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_korean_sentences() {
        let mut cmd = get_built_binary().command();
        cmd.arg("안녕하세요. 오늘 날씨가 좋네요.");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_multiple_spaces() {
        let mut cmd = get_built_binary().command();
        cmd.arg("안녕    하세요");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn test_braillify_newlines() {
        let mut cmd = get_built_binary().command();
        cmd.arg("안녕\n하세요");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}
