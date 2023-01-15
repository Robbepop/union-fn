| Continuous Integration |  Documentation   |      Crates.io       |
|:----------------------:|:----------------:|:--------------------:|
| [![ci][1]][2]          | [![docs][3]][4] | [![crates][5]][6]  |

[1]: https://github.com/Robbepop/union-fn/actions/workflows/rust.yml/badge.svg
[2]: https://github.com/Robbepop/union-fn/actions/workflows/rust.yml
[3]: https://docs.rs/union-fn/badge.svg
[4]: https://docs.rs/union-fn
[5]: https://img.shields.io/crates/v/union-fn.svg
[6]: https://crates.io/crates/union-fn

# `#[union_fn]`: Data Structure for Efficient Interpreters

This crate provides a procedural macro `#[union_fn]` that can be applied to Rust `trait` definitions.

A `#[union_fn]` can be thought of as a set of polymorphic, parameterized functions
that are optimized for data locality and polymorphic calls.

## Motivation & Idea

Interpreters usually use a `switch-loop` based instruction dispatch where a simple `enum` represents
all kinds of different instructions such as `Add` and `Branch`.
Instruction dispatch occurs in between instruction execution and has a lot of overhead when using
this form of dispatch via branch table which often is not optimized ideally.

The `#[union_fn]` macro decreases the dispatch costs down to the minimal by embedding the function
pointer to the instruction handling instruction directly into the type next to its function parameters.
This way there is no need for a branch table and a call dispatch is equal to an indirect function call.

Due to alignment of Rust `enum` discriminants there is a lot of wasted space for the `enum`
representation which is properly utilized by the optimized representation by storing a function pointer
instead of the `enum` discriminant. Therefore both types usually have equal `size_of`. The function
pointed to then knows how to decode the function parameters encoded via `union` with zero overhead.

## Codegen

The `#[union_fn]` macro primarily generates 2 different types:

- An enum representation of all trait methods referred to by the trait's identifier.
    - Useful to inspect, debug and create the different calls.
    - Accessed via the trait's identifier, e.g. `Foo`.
    - Each method generates a constructor with the same name and arguments.
- A type optimized for data locality and polymorphic calls.
    - Primarily used for actual calling during the compute phase.
    - Accessed via `<Foo as union_fn::UnionFn>::Opt>` where `Foo` is the trait's identifier.
    - Each method generates a constructor with the same name and arguments OR;
      it is possible to convert from the `enum` representation via the `union_fn::IntoOpt` trait.

## Example

### Interpreters

A full fledged calculator example that acts as inspiration for interpreters can be found [here](./tests/ui/pass/calculator.rs).

### Codegen

Given the following Rust code:

```rust
use union_fn::union_fn;

#[union_fn]
trait Counter {
    type Context = i64;

    /// Bumps the value `by` the amount.
    fn bump_by(value: &mut Self::Context, by: i64) {
        *value += by;
    }

    /// Selects the values in `choices` depending on `value`.
    fn select(value: &mut Self::Context, choices: [i64; 4]) {
        *value = choices.get(*value as usize).copied().unwrap_or(0)
    }

    /// Resets the `value` to zero.
    fn reset(value: &mut Self::Context) {
        *value = 0;
    }
}

fn main() {
    let mut value = 0;

    Counter::bump_by(1).call(&mut value);
    assert_eq!(value, 1);

    Counter::bump_by(41).call(&mut value);
    assert_eq!(value, 42);

    Counter::reset().call(&mut value);
    assert_eq!(value, 0);

    let choices = [11, 22, 33, 44];
    let opt = Counter::select(choices).into_opt();
    for i in 0..5 {
        let mut value = i;
        opt.call(&mut value);
        assert_eq!(value, choices.get(i as usize).copied().unwrap_or(0));
    }
}
```

This proc. macro will expand to roughly the following code:
(Note, for demonstration purposes whitespace and derive macro expansions have been changed.)

```rust
#[derive(::core::marker::Copy, ::core::clone::Clone)]
pub enum Counter {
    /// Bumps the value `by` the amount.
    BumpBy { by: i64 },
    /// Selects the values in `choices` depending on `value`.
    Select { choices: [i64; 4] },
    /// Resets the `value` to zero.
    Reset {},
}

impl Counter {
    /// Bumps the value `by` the amount.
    pub fn bump_by(by: i64) -> Self {
        Self::BumpBy { by }
    }

    /// Selects the values in `choices` depending on `value`.
    pub fn select(choices: [i64; 4]) -> Self {
        Self::Select { choices }
    }

    /// Resets the `value` to zero.
    pub fn reset() -> Self {
        Self::Reset {}
    }
}

impl ::union_fn::CallWithContext for Counter {
    type Context = i64;
    fn call(self, ctx: &mut Self::Context) -> <Counter as ::union_fn::UnionFn>::Output {
        <<Counter as ::union_fn::IntoOpt>::Opt as ::union_fn::CallWithContext>::call(
            <Counter as ::union_fn::IntoOpt>::into_opt(self),
            ctx,
        )
    }
}

const _: () = {
    ///Call optimized structure of the [`Counter`] type.
    #[derive(::core::marker::Copy, ::core::clone::Clone)]
    pub struct CounterOpt {
        handler: fn(
            ctx: &mut <Counter as ::union_fn::CallWithContext>::Context,
            <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output,
        args: <Counter as ::union_fn::UnionFn>::Args,
    }

    impl ::union_fn::IntoOpt for Counter {
        type Opt = CounterOpt;
        type Delegator = CounterDelegate;
        type Impls = CounterImpls;

        fn into_opt(self) -> Self::Opt {
            match self {
                Self::BumpBy { by } => <Counter as ::union_fn::IntoOpt>::Opt::bump_by(by),
                Self::Select { choices } => {
                    <Counter as ::union_fn::IntoOpt>::Opt::select(choices)
                }
                Self::Reset {} => <Counter as ::union_fn::IntoOpt>::Opt::reset(),
            }
        }
    }

    impl ::union_fn::CallWithContext for Counter {
        type Context = i64;

        fn call(
            self,
            ctx: &mut Self::Context,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            match self {
                Self::BumpBy { by } => {
                    <Counter as ::union_fn::IntoOpt>::Impls::bump_by(ctx, by)
                }
                Self::Select { choices } => {
                    <Counter as ::union_fn::IntoOpt>::Impls::select(ctx, choices)
                }
                Self::Reset { } => {
                    <Counter as ::union_fn::IntoOpt>::Impls::reset(ctx,)
                }
            }
        }
    }

    impl CounterOpt {
        /// Bumps the value `by` the amount.
        pub fn bump_by(by: i64) -> Self {
            Self {
                handler: <Counter as ::union_fn::IntoOpt>::Delegator::bump_by,
                args: <Counter as ::union_fn::UnionFn>::Args::bump_by(by),
            }
        }

        /// Selects the values in `choices` depending on `value`.
        pub fn select(choices: [i64; 4]) -> Self {
            Self {
                handler: <Counter as ::union_fn::IntoOpt>::Delegator::select,
                args: <Counter as ::union_fn::UnionFn>::Args::select(choices),
            }
        }

        /// Resets the `value` to zero.
        pub fn reset() -> Self {
            Self {
                handler: <Counter as ::union_fn::IntoOpt>::Delegator::reset,
                args: <Counter as ::union_fn::UnionFn>::Args::reset(),
            }
        }
    }

    ///Efficiently packed method arguments for the [`Counter`] type.
    #[derive(::core::marker::Copy, ::core::clone::Clone)]
    pub union CounterArgs {
        /// Bumps the value `by` the amount.
        bump_by: i64,
        /// Selects the values in `choices` depending on `value`.
        select: [i64; 4],
        /// Resets the `value` to zero.
        reset: (),
    }

    impl CounterArgs {
        /// Bumps the value `by` the amount.
        pub fn bump_by(by: i64) -> Self {
            Self { bump_by: by }
        }

        /// Selects the values in `choices` depending on `value`.
        pub fn select(choices: [i64; 4]) -> Self {
            Self { select: choices }
        }

        /// Resets the `value` to zero.
        pub fn reset() -> Self {
            Self { reset: () }
        }
    }

    impl ::union_fn::UnionFn for CounterOpt {
        type Output = ();
        type Args = CounterArgs;
    }

    impl ::union_fn::UnionFn for Counter {
        type Output = ();
        type Args = CounterArgs;
    }

    ///Decodes and delegates packed arguments to the implementation of [`Counter`] methods.
    pub enum CounterDelegate {}

    impl CounterDelegate {
        /// Bumps the value `by` the amount.
        fn bump_by(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let by = unsafe { args.bump_by };
            <Counter as ::union_fn::IntoOpt>::Impls::bump_by(value, by)
        }

        /// Selects the values in `choices` depending on `value`.
        fn select(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let choices = unsafe { args.select };
            <Counter as ::union_fn::IntoOpt>::Impls::select(value, choices)
        }

        /// Resets the `value` to zero.
        fn reset(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let () = unsafe { args.reset };
            <Counter as ::union_fn::IntoOpt>::Impls::reset(value)
        }
    }

    ///Implements all methods of the [`Counter`] type.
    pub enum CounterImpls {}

    impl CounterImpls {
        /// Bumps the value `by` the amount.
        fn bump_by(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            by: i64,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            *value += by;
        }

        /// Selects the values in `choices` depending on `value`.
        fn select(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            choices: [i64; 4],
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            *value = choices.get(*value as usize).copied().unwrap_or(0);
        }

        /// Resets the `value` to zero.
        fn reset(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            *value = 0;
        }
    }
};
```
