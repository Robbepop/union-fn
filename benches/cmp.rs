//! Compares the performance between the generated `enum` and `opt` types
//! of a `#[union_fn]` macro invocation in a simple interpreter usage scenario.
//!
//! The idea behind this comparison is that instruction dispatch based on the
//! generated `enum` type can be compared to the naive approach using loop-switch
//! based instruction dispatch whereas the `opt` type based dispatch is what this
//! macro brings to the table performance wise.

mod interpreter;

use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::time::Duration;

criterion_group!(
    name = bench_interpret;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_interpret_enum,
        bench_interpret_opt,
);
criterion_main!(bench_interpret);

fn bench_interpret_enum(c: &mut Criterion) {
    c.bench_function("interpret/enum", |b| {
        // TODO: setup
        b.iter(|| {
            // TODO: benchmark routine
        })
    });
}

fn bench_interpret_opt(c: &mut Criterion) {
    c.bench_function("interpret/enum", |b| {
        // TODO: setup
        b.iter(|| {
            // TODO: benchmark routine
        })
    });
}
