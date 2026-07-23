use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;

mod synthetic;

fn corpus(name: &str) -> String {
    synthetic::ensure_files_exist();
    fs::read_to_string(format!("benches/corpus/{name}.txt")).expect("corpus file missing")
}

fn bench_short_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/short");
    let cases = [
        ("greet", "안녕하세요"),
        ("name", "오정민입니다"),
        ("mixed", "BMI는 22.5kg/m²이다."),
        ("punct", "그래서, 그러나, 그리고…"),
    ];

    for (name, input) in cases {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, &s| {
            b.iter(|| braillify::encode(black_box(s)));
        });
    }

    group.finish();
}

fn bench_prose(c: &mut Criterion) {
    let kim_sowol = corpus("kim_sowol");
    let kim_yujeong = corpus("kim_yujeong");
    let synth1k = corpus("synthetic_hangul_1k");
    let synth10k = corpus("synthetic_hangul_10k");
    let synth100k = corpus("synthetic_hangul_100k");

    let mut group = c.benchmark_group("encode/prose");
    group.sample_size(10);
    for (label, text) in [
        ("kim_sowol", &kim_sowol),
        ("kim_yujeong", &kim_yujeong),
        ("synth_1k", &synth1k),
        ("synth_10k", &synth10k),
        ("synth_100k", &synth100k),
    ] {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), text.as_str(), |b, s| {
            b.iter(|| braillify::encode(black_box(s)));
        });
    }

    group.finish();
}

fn bench_to_unicode(c: &mut Criterion) {
    let synth1k = corpus("synthetic_hangul_1k");
    let mut group = c.benchmark_group("encode_to_unicode");
    group.throughput(Throughput::Bytes(synth1k.len() as u64));
    group.bench_function("synth_1k", |b| {
        b.iter(|| braillify::encode_to_unicode(black_box(&synth1k)));
    });
    group.finish();
}

criterion_group!(benches, bench_short_strings, bench_prose, bench_to_unicode);
criterion_main!(benches);
