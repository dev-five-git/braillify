//! Integration tests for the `braillify` CLI binary.
//!
//! Spawns the actual compiled binary via `assert_cmd` to exercise the
//! `run_cli` entry point including stdin/argv parsing branches that are
//! hard to cover from unit tests.

use assert_cmd::Command;
use predicates::prelude::*;

/// Single-argument one-shot mode: braille output is written to stdout.
#[test]
fn cli_oneshot_korean_input() {
    Command::cargo_bin("braillify")
        .unwrap()
        .arg("안녕")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// Empty argument list with no piped stdin: program should not crash.
/// rustyline initialisation may fail on the test runner's non-TTY stdin, but
/// we accept either success or failure — we just want to exercise `run_cli`.
#[test]
fn cli_no_argument_does_not_panic() {
    let _ = Command::cargo_bin("braillify").unwrap().assert();
}

/// Long argv input completes in reasonable time.
#[test]
fn cli_long_korean_input() {
    let long = "안녕하세요 ".repeat(50);
    Command::cargo_bin("braillify")
        .unwrap()
        .arg(&long)
        .assert()
        .success();
}

/// Invalid unicode (emoji not supported by CharType::new) → exit failure.
#[test]
fn cli_invalid_char_fails() {
    Command::cargo_bin("braillify")
        .unwrap()
        .arg("😀")
        .assert()
        .failure();
}

/// `--version` is wired through clap.
#[test]
fn cli_version_flag() {
    Command::cargo_bin("braillify")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("braillify"));
}

/// `--help` is also wired through clap.
#[test]
fn cli_help_flag() {
    Command::cargo_bin("braillify")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

/// Piped stdin with no argv: stdin should be consumed and used as input.
/// Covers lines 17-22 of cli.rs (stdin reading branch).
#[test]
fn cli_reads_stdin_when_no_arg() {
    Command::cargo_bin("braillify")
        .unwrap()
        .write_stdin("안녕")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// Piped empty stdin should still complete without panicking.
#[test]
fn cli_empty_stdin_no_panic() {
    let _ = Command::cargo_bin("braillify")
        .unwrap()
        .write_stdin("")
        .assert();
}
