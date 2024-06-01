use criterion::{black_box, criterion_group, criterion_main, Criterion};
use utiles::dev::sqlite_u64::{
    i64_to_u64_ne_bytes, i64_to_u64_ptr, i64_to_u64_unsafe_transmute,
    u64_to_i64_ne_bytes, u64_to_i64_ptr, u64_to_i64_unsafe_transmute,
};

fn benchmark_conversion(
    c: &mut Criterion,
    name: &str,
    u64_to_i64: fn(u64) -> i64,
    i64_to_u64: fn(i64) -> u64,
) {
    let values_u64: Vec<u64> = vec![
        black_box(0),
        black_box(0xFFFF_FFFF_FFFF_FFFF),
        black_box(9_223_372_036_854_775_807 + 1234),
        black_box(1),
    ];

    let values_i64: Vec<i64> = vec![
        black_box(0),
        black_box(-1),
        black_box(-9_223_372_036_854_774_575),
        black_box(1),
    ];

    c.bench_function(&format!("{} u64_to_i64", name), |b| {
        b.iter(|| {
            for &value in values_u64.iter() {
                let _ = u64_to_i64(value);
            }
        })
    });

    c.bench_function(&format!("{} i64_to_u64", name), |b| {
        b.iter(|| {
            for &value in values_i64.iter() {
                let _ = i64_to_u64(value);
            }
        })
    });
}

fn benchmarks(c: &mut Criterion) {
    benchmark_conversion(
        c,
        "transmute",
        u64_to_i64_unsafe_transmute,
        i64_to_u64_unsafe_transmute,
    );
    benchmark_conversion(c, "ptr", u64_to_i64_ptr, i64_to_u64_ptr);
    benchmark_conversion(c, "ne_bytes", u64_to_i64_ne_bytes, i64_to_u64_ne_bytes);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
