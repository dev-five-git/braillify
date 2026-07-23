use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;

fn math_corpus() -> String {
    fs::read_to_string("benches/corpus/math_latex.txt").expect("math corpus file missing")
}

fn bench_math_lines(c: &mut Criterion) {
    let corpus = math_corpus();
    let expressions: Vec<&str> = corpus
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    let mut group = c.benchmark_group("encode/math/latex_lines");
    for (index, expression) in expressions.iter().enumerate() {
        let label = format!("{index:02}");
        group.throughput(Throughput::Bytes(expression.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            expression,
            |b, expression| {
                b.iter(|| braillify::encode(black_box(expression)));
            },
        );
    }
    group.finish();
}

fn bench_math_concat(c: &mut Criterion) {
    let corpus = math_corpus();
    let concat = corpus
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let mut group = c.benchmark_group("encode/math/concat");
    group.throughput(Throughput::Bytes(concat.len() as u64));
    group.bench_function("all", |b| {
        b.iter(|| braillify::encode(black_box(&concat)));
    });
    group.finish();
}

criterion_group!(benches, bench_math_lines, bench_math_concat);
criterion_main!(benches);
