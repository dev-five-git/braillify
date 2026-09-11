//! 로마자 구간 표지(제29·33·35항)의 오류를 유형별로 세는 회귀 안전망.
//!
//! 로마자표 ⠴ · 종료표 ⠲ 의 과잉/누락은 서로 반대 방향이라 총 정확도만 보면
//! 한쪽을 고치면서 다른 쪽을 망가뜨려도 드러나지 않는다. 구간 상태 기계를 고칠
//! 때에는 이 네 수치가 모두 나빠지지 않아야 한다.
//!
//! ```text
//! cargo run --release -p braillify --example roman_marker_bench
//! ```

use std::fs;
use std::path::Path;
use std::sync::Mutex;

const BLANK: char = '\u{2800}';
const ROMAN_START: char = '⠴';
const ROMAN_END: char = '⠲';

#[derive(Default, Clone, Copy)]
struct Counts {
    sentences: usize,
    exact: usize,
    start_excess: usize,
    start_missing: usize,
    end_excess: usize,
    end_missing: usize,
}

impl Counts {
    fn merge(&mut self, other: &Counts) {
        self.sentences += other.sentences;
        self.exact += other.exact;
        self.start_excess += other.start_excess;
        self.start_missing += other.start_missing;
        self.end_excess += other.end_excess;
        self.end_missing += other.end_missing;
    }
}

fn measure(path: &Path) -> Counts {
    let content = fs::read_to_string(path).unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    let mut counts = Counts::default();
    for record in &records {
        let input = record["input"].as_str().unwrap_or("");
        let expected = record["unicode"].as_str().unwrap_or("");
        if input.is_empty() || expected.is_empty() {
            continue;
        }
        counts.sentences += 1;
        let Ok(actual) = braillify::encode_to_unicode(input) else {
            continue;
        };
        if actual == expected {
            counts.exact += 1;
            continue;
        }
        let expected_words: Vec<&str> = expected.split(BLANK).collect();
        let actual_words: Vec<&str> = actual.split(BLANK).collect();
        if expected_words.len() != actual_words.len() {
            continue;
        }
        for (want, got) in expected_words.iter().zip(actual_words.iter()) {
            if want == got {
                continue;
            }
            match (want.starts_with(ROMAN_START), got.starts_with(ROMAN_START)) {
                (false, true) => counts.start_excess += 1,
                (true, false) => counts.start_missing += 1,
                _ => {}
            }
            match (want.ends_with(ROMAN_END), got.ends_with(ROMAN_END)) {
                (false, true) => counts.end_excess += 1,
                (true, false) => counts.end_missing += 1,
                _ => {}
            }
        }
    }
    counts
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_cases");
    let mut dirs: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("_corpus"))
        })
        .collect();
    dirs.sort();

    println!("연도\t문장\t정확\t로마자표과잉\t로마자표누락\t종료표과잉\t종료표누락");
    let mut grand = Counts::default();
    for dir in &dirs {
        let mut paths: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        let shared = Mutex::new(Counts::default());
        let shared_ref = &shared;
        std::thread::scope(|scope| {
            for path in &paths {
                scope.spawn(move || {
                    let counts = measure(path);
                    shared_ref.lock().unwrap().merge(&counts);
                });
            }
        });
        let counts = *shared.lock().unwrap();
        grand.merge(&counts);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            dir.file_name().unwrap().to_string_lossy(),
            counts.sentences,
            counts.exact,
            counts.start_excess,
            counts.start_missing,
            counts.end_excess,
            counts.end_missing
        );
    }
    println!(
        "합계\t{}\t{}\t{}\t{}\t{}\t{}",
        grand.sentences,
        grand.exact,
        grand.start_excess,
        grand.start_missing,
        grand.end_excess,
        grand.end_missing
    );
    println!(
        "\n정확도 {:.4}%  표지 오류 합계 {}",
        grand.exact as f64 / grand.sentences as f64 * 100.0,
        grand.start_excess + grand.start_missing + grand.end_excess + grand.end_missing
    );
}
