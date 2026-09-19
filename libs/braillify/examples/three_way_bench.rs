use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::thread;

#[derive(Default, Clone, Copy)]
struct Counts {
    total: usize,
    measured: usize,
    braillify: usize,
    world: usize,
    love: usize,
    braillify_only: usize,
    world_only: usize,
    love_only: usize,
    all_three: usize,
    none: usize,
}

impl Counts {
    fn merge(&mut self, other: &Counts) {
        self.total += other.total;
        self.measured += other.measured;
        self.braillify += other.braillify;
        self.world += other.world;
        self.love += other.love;
        self.braillify_only += other.braillify_only;
        self.world_only += other.world_only;
        self.love_only += other.love_only;
        self.all_three += other.all_three;
        self.none += other.none;
    }
}

/// 경쟁 점역기는 한글 모드만 제공하므로 영문으로 시작/끝나는 입력의 외곽
/// 로마자표/종료표는 세 결과와 정답 모두에서 동일하게 제거한 뒤 비교한다.
fn strip_outer(value: &str, input: &str) -> String {
    let mut out = value;
    if input.starts_with(|c: char| c.is_ascii_alphabetic()) {
        out = out.strip_prefix('⠴').unwrap_or(out);
    }
    if input.ends_with(|c: char| c.is_ascii_alphabetic()) {
        out = out.strip_suffix('⠲').unwrap_or(out);
    }
    out.to_string()
}

fn measure(path: &Path) -> Counts {
    let content = fs::read_to_string(path).unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    let mut c = Counts::default();
    for record in &records {
        let input = record["input"].as_str().unwrap_or("");
        let reference = record["unicode"].as_str().unwrap_or("");
        if input.is_empty() || reference.is_empty() {
            continue;
        }
        c.total += 1;
        let world = record["world"].as_str().unwrap_or("");
        let love = record["jeomsarang"].as_str().unwrap_or("");
        if world.is_empty() || love.is_empty() {
            continue;
        }
        c.measured += 1;

        let reference = strip_outer(reference, input);
        let actual = braillify::encode_to_unicode(input).unwrap_or_default();
        let b_ok = strip_outer(&actual, input) == reference;
        let w_ok = strip_outer(&world.replace(' ', "\u{2800}"), input) == reference;
        let l_ok = strip_outer(&love.replace(' ', "\u{2800}"), input) == reference;

        if b_ok {
            c.braillify += 1;
        }
        if w_ok {
            c.world += 1;
        }
        if l_ok {
            c.love += 1;
        }
        match (b_ok, w_ok, l_ok) {
            (true, true, true) => c.all_three += 1,
            (true, false, false) => c.braillify_only += 1,
            (false, true, false) => c.world_only += 1,
            (false, false, true) => c.love_only += 1,
            (false, false, false) => c.none += 1,
            _ => {}
        }
    }
    c
}

fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64 * 100.0
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_cases");
    let mut dirs: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("_corpus"))
        })
        .collect();
    dirs.sort();

    println!("year\ttotal\tmeasured\tbraillify\tworld\tlove\tb_only\tw_only\tl_only\tall3\tnone");
    let mut grand = Counts::default();
    for dir in &dirs {
        let mut paths: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        let shared = Mutex::new(Counts::default());
        thread::scope(|scope| {
            for path in &paths {
                scope.spawn(|| {
                    let c = measure(path);
                    shared.lock().unwrap().merge(&c);
                });
            }
        });
        let c = *shared.lock().unwrap();
        grand.merge(&c);
        let year = dir.file_name().unwrap().to_string_lossy();
        println!(
            "{year}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            c.total,
            c.measured,
            c.braillify,
            c.world,
            c.love,
            c.braillify_only,
            c.world_only,
            c.love_only,
            c.all_three,
            c.none
        );
    }
    println!(
        "ALL\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        grand.total,
        grand.measured,
        grand.braillify,
        grand.world,
        grand.love,
        grand.braillify_only,
        grand.world_only,
        grand.love_only,
        grand.all_three,
        grand.none
    );
    println!(
        "\nbraillify {:.2}%  world {:.2}%  love {:.2}%  (measured {})",
        percent(grand.braillify, grand.measured),
        percent(grand.world, grand.measured),
        percent(grand.love, grand.measured),
        grand.measured
    );
}
