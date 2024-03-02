use std::fs;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rs_1brc::*;

pub fn criterion_benchmark(c: &mut Criterion) {

    let f_name = r"C:\a\m10m.txt";
    let raw = fs::read(f_name).unwrap();

    c.bench_function("par_iter"  , |b| b.iter(|| impl07(&raw, black_box(8))));
    c.bench_function("std thread", |b| b.iter(|| impl08(&raw, black_box(8))));
    c.bench_function("fxHash"    , |b| b.iter(|| impl09(&raw, black_box(8))));
    c.bench_function("i32parse"  , |b| b.iter(|| impl10(&raw, black_box(8))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);