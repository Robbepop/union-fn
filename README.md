# `#[union_fn]`: Data Structure for Efficient Interpreters

This crate provides a procedural macro `#[union_fn]` that can be applied to Rust `trait` definitions.

The macro generates quite a few things which are of interest primarily (but not exclusively) to efficient
interpreters that have a high runtime overhead due to their instruction dispatch.

Interpreters usually use a `switch-loop` based instruction dispatch and a simple `enum` to represent
the different kinds of instructions. The problem is that every instruction dispatch incurs lots of overhead
simply by selecting and calling the handler of the instruction.

This crate solves one indirection by using a data structure which we call a `union-fn`, however
it could have also been called "inline closure" etc. It simply packs the function pointer next
to the `union`-packed data. This optimizes data locality and removes the need for a jump table.

## Example

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

    /// Divides the `value` by 2.
    fn div2(value: &mut Self::Context) {
        *value /= 2;
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

    Counter::div2().call(&mut value);
    assert_eq!(value, 21);

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
const _: () = {
    impl ::union_fn::UnionFn for CounterOpt {
        type Output = ();
        type Opt = Self;
        type Args = CounterArgs;
        type Impls = CounterImpls;
        type Delegator = CounterDelegate;
    }

    impl ::union_fn::UnionFn for Counter {
        type Output = ();
        type Opt = CounterOpt;
        type Args = CounterArgs;
        type Impls = CounterImpls;
        type Delegator = CounterDelegate;
    }

    ///Efficiently packed method arguments for the [`Counter`] type.
    #[derive(Copy, Clone)]
    pub union CounterArgs {
        /// Bumps the value `by` the amount.
        bump_by: i64,
        /// Selects the values in `choices` depending on `value`.
        select: [i64; 4],
        /// Divides the `value` by 2.
        div2: (),
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

        /// Divides the `value` by 2.
        pub fn div2() -> Self {
            Self { div2: () }
        }

        /// Resets the `value` to zero.
        pub fn reset() -> Self {
            Self { reset: () }
        }
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
            <Counter as ::union_fn::UnionFn>::Impls::bump_by(value, by)
        }

        /// Selects the values in `choices` depending on `value`.
        fn select(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let choices = unsafe { args.select };
            <Counter as ::union_fn::UnionFn>::Impls::select(value, choices)
        }

        /// Divides the `value` by 2.
        fn div2(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let () = unsafe { args.div2 };
            <Counter as ::union_fn::UnionFn>::Impls::div2(value)
        }
    
        /// Resets the `value` to zero.
        fn reset(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
            args: <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            let () = unsafe { args.reset };
            <Counter as ::union_fn::UnionFn>::Impls::reset(value)
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

        /// Divides the `value` by 2.
        fn div2(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            *value /= 2;
        }

        /// Resets the `value` to zero.
        fn reset(
            value: &mut <Counter as ::union_fn::CallWithContext>::Context,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            *value = 0;
        }
    }

    ///Call optimized structure of the [`Counter`] type.
    #[derive(Copy, Clone)]
    pub struct CounterOpt {
        handler: fn(
            ctx: &mut <Counter as ::union_fn::CallWithContext>::Context,
            <Counter as ::union_fn::UnionFn>::Args,
        ) -> <Counter as ::union_fn::UnionFn>::Output,
        args: <Counter as ::union_fn::UnionFn>::Args,
    }

    impl ::union_fn::CallWithContext for CounterOpt {
        type Context = i64;
        fn call(
            self,
            ctx: &mut Self::Context,
        ) -> <Counter as ::union_fn::UnionFn>::Output {
            (self.handler)(ctx, self.args)
        }
    }

    impl CounterOpt {
        /// Bumps the value `by` the amount.
        pub fn bump_by(by: i64) -> Self {
            Self {
                handler: <Counter as ::union_fn::UnionFn>::Delegator::bump_by,
                args: <Counter as ::union_fn::UnionFn>::Args::bump_by(by),
            }
        }

        /// Selects the values in `choices` depending on `value`.
        pub fn select(choices: [i64; 4]) -> Self {
            Self {
                handler: <Counter as ::union_fn::UnionFn>::Delegator::select,
                args: <Counter as ::union_fn::UnionFn>::Args::select(choices),
            }
        }

        /// Divides the `value` by 2.
        pub fn div2() -> Self {
            Self {
                handler: <Counter as ::union_fn::UnionFn>::Delegator::div2,
                args: <Counter as ::union_fn::UnionFn>::Args::div2(),
            }
        }

        /// Resets the `value` to zero.
        pub fn reset() -> Self {
            Self {
                handler: <Counter as ::union_fn::UnionFn>::Delegator::reset,
                args: <Counter as ::union_fn::UnionFn>::Args::reset(),
            }
        }
    }
};

#[derive(Copy, Clone)]
pub enum Counter {
    /// Bumps the value `by` the amount.
    BumpBy { by: i64 },
    /// Selects the values in `choices` depending on `value`.
    Select { choices: [i64; 4] },
    /// Divides the `value` by 2.
    Div2 {},
    /// Resets the `value` to zero.
    Reset {},
}

impl ::union_fn::IntoOpt for Counter {
    fn into_opt(self) -> <Counter as ::union_fn::UnionFn>::Opt {
        match self {
            Self::BumpBy { by } => <Counter as ::union_fn::UnionFn>::Opt::bump_by(by),
            Self::Select { choices } => {
                <Counter as ::union_fn::UnionFn>::Opt::select(choices)
            }
            Self::Div2 {} => <Counter as ::union_fn::UnionFn>::Opt::div2(),
            Self::Reset {} => <Counter as ::union_fn::UnionFn>::Opt::reset(),
        }
    }
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
    /// Divides the `value` by 2.
    pub fn div2() -> Self {
        Self::Div2 {}
    }
    /// Resets the `value` to zero.
    pub fn reset() -> Self {
        Self::Reset {}
    }
}

impl ::union_fn::CallWithContext for Counter {
    type Context = i64;
    fn call(self, ctx: &mut Self::Context) -> <Counter as ::union_fn::UnionFn>::Output {
        <<Counter as ::union_fn::UnionFn>::Opt as ::union_fn::CallWithContext>::call(
            <Counter as ::union_fn::IntoOpt>::into_opt(self),
            ctx,
        )
    }
}
```
