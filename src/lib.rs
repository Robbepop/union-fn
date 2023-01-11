#![no_std]

pub use union_fn_macro::union_fn;

/// Allows `#[union_fn]` types with context to be called as functions.
/// 
/// # Note
/// 
/// This trait automatically implemented by `#[union_fn]` expansions.
pub trait Call: UnionFn {
    /// Calls the union function.
    fn call(self) -> <Self as UnionFn>::Output;
}

/// Allows `#[union_fn]` types with context to be called as functions.
/// 
/// # Note
/// 
/// This trait automatically implemented by `#[union_fn]` expansions.
pub trait CallWithContext: UnionFn {
    /// The shared execution context.
    type Context;

    /// Calls the union function with the given context.
    fn call(self, ctx: &mut Self::Context) -> <Self as UnionFn>::Output;
}

/// Allows `#[union_fn]` types to convert to their optimized instance.
/// 
/// # Note
/// 
/// This trait automatically implemented by `#[union_fn]` expansions.
pub trait IntoOpt: UnionFn {
    /// Converts the `#[union_fn]` enum to the call optimized type.
    fn into_opt(self) -> <Self as UnionFn>::Opt;
}

/// Stores information about a `#[union_fn]` macro expansion.
/// 
/// This helps to link different generated types together and
/// allow them to work interconnectedly.
///
/// # Note
/// 
/// This trait automatically implemented by `#[union_fn]` expansions.
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
