//! Compares the performance between the generated `enum` and `opt` types
//! of a `#[union_fn]` macro invocation in a simple interpreter usage scenario.
//!
//! The idea behind this comparison is that instruction dispatch based on the
//! generated `enum` type can be compared to the naive approach using loop-switch
//! based instruction dispatch whereas the `opt` type based dispatch is what this
//! macro brings to the table performance wise.

mod interpreter;

use criterion::{criterion_group, criterion_main, Criterion};
use interpreter::{execute, BranchOffset, Instr, TailInstr, TailContext};
use std::time::Duration;
use union_fn::IntoOpt;

criterion_group!(
    name = bench_interpret;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_interpret_enum,
        bench_interpret_opt,
        bench_interpret_tail,
);
criterion_main!(bench_interpret);

/// Instructions for counting until a given input from zero.
fn count_until() -> Vec<Instr> {
    vec![
        Instr::constant(0),  // local: counter
        Instr::local_get(1), // dup(counter)
        Instr::constant(1),
        Instr::add(),        // counter + 1
        Instr::local_tee(1), // counter = counter + 1
        Instr::local_get(0), // local: until
        Instr::ne(),
        Instr::br_eqz(BranchOffset::new(2)),
        Instr::br(BranchOffset::new(-7)),
        Instr::local_get(1),
        Instr::ret(),
    ]
}

fn bench_interpret_enum(c: &mut Criterion) {
    c.bench_function("interpret/enum", |b| {
        let instrs = count_until();
        b.iter(|| execute(&instrs, &[1_000_000]))
    });
}

fn bench_interpret_opt(c: &mut Criterion) {
    c.bench_function("interpret/opt", |b| {
        let instrs = count_until()
            .into_iter()
            .map(IntoOpt::into_opt)
            .collect::<Vec<_>>();
        b.iter(|| execute(&instrs, &[1_000_000]))
    });
}

/// Instructions for counting until a given input from zero.
fn tail_count_until() -> Vec<TailInstr> {
    vec![
        TailInstr::constant(0),  // local: counter
        TailInstr::local_get(1), // dup(counter)
        TailInstr::constant(1),
        TailInstr::add(),        // counter + 1
        TailInstr::local_tee(1), // counter = counter + 1
        TailInstr::local_get(0), // local: until
        TailInstr::ne(),
        TailInstr::br_eqz(BranchOffset::new(2)),
        TailInstr::br(BranchOffset::new(-7)),
        TailInstr::local_get(1),
        TailInstr::ret(),
    ]
}

fn bench_interpret_tail(c: &mut Criterion) {
    c.bench_function("interpret/tail", |b| {
        let instrs = tail_count_until()
            .into_iter()
            .map(IntoOpt::into_opt)
            .collect::<Vec<_>>();
        let mut ctx = TailContext::new(&instrs);
        b.iter(|| ctx.execute(&[1_000]))
    });
}
