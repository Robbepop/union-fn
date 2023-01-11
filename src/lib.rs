#![no_std]

pub use union_fn_macro::union_fn;

/// Implemented by union functions without context.
pub trait Call: UnionFn {
    /// Calls the union function.
    fn call(self) -> <Self as UnionFn>::Output;
}

/// Implemented by union functions with context.
pub trait CallWithContext: UnionFn {
    /// The shared execution context.
    type Context;

    /// Calls the union function with the given context.
    fn call(self, ctx: &mut Self::Context) -> <Self as UnionFn>::Output;
}

/// Implemented by the `#[union_fn]` enum type for the call optimized conversion.
pub trait IntoOpt: UnionFn {
    /// Converts the `#[union_fn]` enum to the call optimized type.
    fn into_opt(self) -> <Self as UnionFn>::Opt;
}

/// Trait implemented by a union function.
pub trait UnionFn {
    /// The common output type of all functions in the union function.
    type Output;
    /// The call optimized `#[union_fn]` type.
    type Opt;
    /// Type responsible to hold call optimized parameters.
    type Args;
    /// Type responsible to implement calls for the `#[union_fn]` type.
    type Impls;
    /// Type responsible to delegate optimized calls for the call optimized `#[union_fn]` type.
    type Delegator;
}
