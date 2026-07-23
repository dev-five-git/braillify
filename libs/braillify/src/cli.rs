use std::io::{self, IsTerminal, Read, Write};

use anyhow::{Result, bail};
use clap::Parser;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::encode_to_unicode;

#[derive(Parser, Debug)]
#[command(name = "braillify", about = "한국어 점자 변환 CLI", version)]
struct Cli {
    /// 입력 문자열. 없으면 REPL 모드로 진입합니다
    input: Option<String>,
}

pub fn run_cli(mut args: Vec<String>) -> Result<()> {
    if args.len() == 1 && !std::io::stdin().is_terminal() {
        let mut buffer = vec![];
        io::stdin().read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            args.push(String::from_utf8(buffer)?);
        }
    }
    match Cli::parse_from(args).input {
        Some(text) => run_one_shot(&text),
        None => run_repl(),
    }
}

fn run_one_shot(text: &str) -> Result<()> {
    let out = encode_to_unicode(text).map_err(|e| anyhow::anyhow!("점자 변환 실패: {}", e))?;
    let mut stdout = io::stdout();
    stdout.write_all(out.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Format a single REPL input line for printing.
///
/// Pure function so unit tests can exercise the encode-success vs encode-error
/// branches without spinning up rustyline. The outer `run_repl` loop is just
/// `read line → format → write` glue; all logic lives here.
pub(crate) fn process_repl_line(line: &str) -> String {
    match encode_to_unicode(line) {
        Ok(out) => out,
        Err(e) => format!("오류: {}", e),
    }
}

// Interactive rustyline loop is unreachable in cargo-tarpaulin runs (stdin is
// never a terminal in CI/test harness). The pure encoding logic was extracted
// to `process_repl_line` which is unit-tested directly above.
#[cfg(not(tarpaulin_include))]
fn run_repl() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "braillify REPL - 입력을 점자로 변환합니다. 종료: Ctrl+C or Ctrl+D"
    )?;
    stdout.flush()?;

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(&line).ok();
                writeln!(stdout, "{}", process_repl_line(&line))?;
                stdout.flush()?;
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                writeln!(stdout, "종료합니다.")?;
                stdout.flush()?;
                break;
            }
            Err(err) => bail!("입력 오류: {}", err),
        }
    }
    Ok(())
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    use super::*;
    #[test]
    fn test_cli_parsing_with_input() {
        let args = vec!["braillify", "안녕하세요"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.input, Some("안녕하세요".to_string()));
    }

    #[test]
    fn test_cli_parsing_without_input() {
        let args = vec!["braillify"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.input, None);
    }

    // 유닛 테스트들
    #[test]
    fn test_run_one_shot_success() {
        let result = run_one_shot("안녕");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_one_shot_invalid_input() {
        // 빈 문자열이나 특수한 경우 테스트
        let result = run_one_shot("");
        assert!(result.is_ok()); // 빈 문자열도 유효한 입력
    }

    // 성능 테스트
    #[test]
    fn test_braillify_performance_long_text() {
        let long_text = "안녕하세요 ".repeat(100);
        let start = std::time::Instant::now();

        let result = run_one_shot(&long_text);
        let duration = start.elapsed();

        assert!(result.is_ok());
        // 1초 이내에 완료되어야 함
        assert!(duration.as_secs() < 1);
    }

    // 에러 처리 테스트
    #[test]
    fn test_braillify_error_handling() {
        // 특수 문자나 매우 긴 입력에 대한 에러 처리 테스트
        let very_long_text = "a".repeat(10000);
        let result = run_one_shot(&very_long_text);
        // 에러가 발생하더라도 프로그램이 크래시되지 않아야 함
        // (실제로는 성공할 수도 있지만, 에러 처리가 제대로 되는지 확인)
        let _ = result;
    }

    #[test]
    fn test_braillify_invalid_input() {
        // Use an emoji which is not supported by CharType::new()
        let result = run_one_shot("😀");
        assert!(result.is_err());
    }

    /// `run_cli` with explicit text input dispatches to `run_one_shot`.
    /// Covers lines 16-26 (arg parsing + dispatch).
    #[test]
    fn run_cli_with_argument_runs_one_shot() {
        let args = vec!["braillify".to_string(), "안녕".to_string()];
        let result = run_cli(args);
        assert!(result.is_ok());
    }

    /// `run_cli` with invalid input still returns Err from one_shot path.
    #[test]
    fn run_cli_with_invalid_input_propagates_error() {
        let args = vec!["braillify".to_string(), "😀".to_string()];
        let result = run_cli(args);
        assert!(result.is_err());
    }

    /// `process_repl_line` returns the encoded braille string on success.
    #[test]
    fn process_repl_line_encodes_valid_input() {
        let out = process_repl_line("안녕");
        assert!(!out.is_empty());
        assert!(!out.starts_with("오류"));
        // Must be Braille Unicode codepoints
        for ch in out.chars() {
            let cp = ch as u32;
            assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
        }
    }

    /// `process_repl_line` returns a human-readable error string on encoder failure.
    #[test]
    fn process_repl_line_reports_encoder_error() {
        let out = process_repl_line("😀");
        assert!(out.starts_with("오류"));
    }

    /// Empty input is a valid encode → empty result string.
    #[test]
    fn process_repl_line_empty_input() {
        let out = process_repl_line("");
        assert_eq!(out, "");
    }
}
