pub use union_fn_macro::union_fn;

/// Implemented by union functions without context.
pub trait Call: UnionFn {
    /// The common output type of all functions in the union function.
    type Output;

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

/// Trait implemented by a union function.
pub trait UnionFn {
    /// The common output type of all functions in the union function.
    type Output;
    /// The generated parameter union type shared by all functions in the union function.
    type Args;
    /// The underlying delegator type that implements all the functions.
    type Delegator;
}
