// Run with: cargo bench --bench memory_dhat --features dhat-heap
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[path = "synthetic.rs"]
mod synthetic;

fn main() {
    synthetic::ensure_files_exist();

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let kim_sowol = std::fs::read_to_string("libs/braillify/benches/corpus/kim_sowol.txt")
        .or_else(|_| std::fs::read_to_string("benches/corpus/kim_sowol.txt"))
        .unwrap();
    let synth_10k =
        std::fs::read_to_string("libs/braillify/benches/corpus/synthetic_hangul_10k.txt")
            .or_else(|_| std::fs::read_to_string("benches/corpus/synthetic_hangul_10k.txt"))
            .unwrap();
    let math = std::fs::read_to_string("libs/braillify/benches/corpus/math_latex.txt")
        .or_else(|_| std::fs::read_to_string("benches/corpus/math_latex.txt"))
        .unwrap();

    let _ = braillify::encode(&kim_sowol);
    let _ = braillify::encode(&synth_10k);
    let _ = braillify::encode(&math);
    let _ = braillify::encode_to_unicode(&kim_sowol);

    #[cfg(not(feature = "dhat-heap"))]
    {
        println!("memory_dhat: rebuild with --features dhat-heap to enable heap profiling");
    }
}
