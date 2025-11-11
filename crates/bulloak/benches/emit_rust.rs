#![allow(missing_docs)]
use bulloak_rust::scaffold;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn emit_big_tree_rust(c: &mut Criterion) {
    let tree =
        std::fs::read_to_string("benches/bench_data/cancel.tree").unwrap();
    let ast = bulloak_syntax::parse_one(&tree).unwrap();

    let cfg = Default::default();
    let mut group = c.benchmark_group("sample-size-10");
    group.bench_function("emit-big-tree-rust", |b| {
        b.iter(|| scaffold::scaffold(black_box(&ast), &cfg))
    });
    group.finish();
}

criterion_group!(benches, emit_big_tree_rust);
criterion_main!(benches);
