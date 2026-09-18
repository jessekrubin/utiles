use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use utiles_core::QuadkeyRef;

fn bench_quadkey_to_xyz(c: &mut Criterion) {
    let cases = [
        ("z0", ""),
        ("z4", "1203"),
        ("z12", "031310231012"),
        ("z20", "03131023101203131023"),
        ("z29", "03131023101203131023031310231"),
    ];

    let mut group = c.benchmark_group("quadkey_to_xyz");

    for (case, input) in cases {
        let quadkey = QuadkeyRef::new_unchecked(input);
        assert_eq!(quadkey.to_xyz_iter(), quadkey.to_xyz());

        group.bench_with_input(
            BenchmarkId::new("to_xyz", case),
            &quadkey,
            |b, quadkey| {
                b.iter(|| black_box(black_box(quadkey).to_xyz()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("to_xyz_iter", case),
            &quadkey,
            |b, quadkey| {
                b.iter(|| black_box(black_box(quadkey).to_xyz_iter()));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_quadkey_to_xyz);
criterion_main!(benches);
