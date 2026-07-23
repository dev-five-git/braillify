use std::fs;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

const SEED: u64 = 0x4252_4149_4c4c_4946;
const SIZES: [(&str, usize); 3] = [
    ("synthetic_hangul_1k", 1_000),
    ("synthetic_hangul_10k", 10_000),
    ("synthetic_hangul_100k", 100_000),
];
const SYLLABLES: &[&str] = &[
    "가", "나", "다", "라", "마", "바", "사", "아", "자", "차", "카", "타", "파", "하", "안", "녕",
    "하", "세", "요", "습", "니", "다", "입", "것", "수", "있", "없", "한", "국", "어", "문", "장",
    "사", "람", "학", "교", "생", "각", "정", "보", "시", "간", "오", "늘", "내", "일", "마", "음",
    "길", "꽃", "산", "강", "바", "람", "햇", "살", "봄", "여", "름", "가", "을", "겨", "울", "책",
    "글", "말", "소", "리", "점", "자", "테", "스", "트", "성", "능", "개", "선",
];
const PUNCT: &[&str] = &[".", ",", "!", "?", "…"];
const DIGITS: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

static ENSURE_SYNTHETIC_FILES: OnceLock<()> = OnceLock::new();

pub fn ensure_files_exist() {
    ENSURE_SYNTHETIC_FILES.get_or_init(|| {
        write_files().expect("failed to materialize synthetic benchmark corpora");
    });
}

fn write_files() -> io::Result<()> {
    let dir = Path::new("benches/corpus");
    fs::create_dir_all(dir)?;

    for (name, chars) in SIZES {
        let path = dir.join(format!("{name}.txt"));
        if !path.exists() {
            fs::write(path, generate(chars))?;
        }
    }

    Ok(())
}

fn generate(target_chars: usize) -> String {
    let mut rng = Lcg::new(SEED ^ target_chars as u64);
    let mut out = String::with_capacity(target_chars * 3);

    while out.chars().count() < target_chars {
        let roll = rng.next_mod(100);
        if roll < 72 {
            out.push_str(SYLLABLES[rng.next_mod(SYLLABLES.len())]);
        } else if roll < 86 {
            out.push(' ');
        } else if roll < 94 {
            out.push_str(PUNCT[rng.next_mod(PUNCT.len())]);
            out.push(' ');
        } else {
            out.push_str(DIGITS[rng.next_mod(DIGITS.len())]);
        }
    }

    out.chars().take(target_chars).collect()
}

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn next_mod(&mut self, modulus: usize) -> usize {
        (self.next() as usize) % modulus
    }
}
