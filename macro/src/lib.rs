use analyse::UnionFnState;
use method::UnionFnMethod;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::format_ident;
use syn::spanned::Spanned;

#[macro_use]
mod error;
mod analyse;
mod expand;
mod method;
mod utils;

/// Proc. macro applied on Rust trait definitions to generate so-called "union functions".
/// 
/// A `#[union_fn]` can be thought of as a set of polymorphic, parameterized functions
/// that are optimized for data locality and calling.
/// 
/// A `#[union_fn]` invocation will primarily generate an `enum` type of the same name
/// that has one variant per trait method, mimicking the method parameters.
/// The generated `enum` can be turned into an optimized version for better data locality
/// and call performance via the [`IntoOpt::into_opt`] trait method.
/// 
/// Calling instances of the `enum` or its optimized version are done via the
/// [`Call::call`] or [`CallWithContext::call`] trait method depending on if the
/// trait defines an [`type Output`] associated type.
/// 
/// [`IntoOpt::into_opt`]: trait.IntoOpt.html
/// [`Call::call`]: trait.Call.html
/// [`CallWithContext::call`]: trait.CallWithContext.html
/// [`type Output`]: trait.UnionFn.html#associatedtype.Output
/// 
/// ## Example
/// 
/// Given the following Rust code and `#[union_fn]` macro invocation:
/// 
/// ```
/// # use union_fn::union_fn;
/// #
/// #[union_fn]
/// trait Counter {
///     type Context = i64;
/// 
///     /// Bumps the value `by` the amount.
///     fn bump_by(value: &mut Self::Context, by: i64) {
///         *value += by;
///     }
/// 
///     /// Selects the values in `choices` depending on `value`.
///     fn select(value: &mut Self::Context, choices: [i64; 4]) {
///         *value = choices.get(*value as usize).copied().unwrap_or(0)
///     }
/// 
///     /// Divides the `value` by 2.
///     fn div2(value: &mut Self::Context) {
///         *value /= 2;
///     }
/// 
///     /// Resets the `value` to zero.
///     fn reset(value: &mut Self::Context) {
///         *value = 0;
///     }
/// }
/// 
/// fn main() {
///     let mut value = 0;
/// 
///     Counter::bump_by(1).call(&mut value);
///     assert_eq!(value, 1);
/// 
///     Counter::bump_by(41).call(&mut value);
///     assert_eq!(value, 42);
/// 
///     Counter::div2().call(&mut value);
///     assert_eq!(value, 21);
/// 
///     Counter::reset().call(&mut value);
///     assert_eq!(value, 0);
/// 
///     let choices = [11, 22, 33, 44];
///     let opt = Counter::select(choices).into_opt();
///     for i in 0..5 {
///         let mut value = i;
///         opt.call(&mut value);
///         assert_eq!(value, choices.get(i as usize).copied().unwrap_or(0));
///     }
/// }
/// ```
/// The proc macro will generate roughly the following expansion:
/// 
/// ```
/// #[derive(Copy, Clone)]
/// pub enum Counter {
///     /// Bumps the value `by` the amount.
///     BumpBy { by: i64 },
///     /// Selects the values in `choices` depending on `value`.
///     Select { choices: [i64; 4] },
///     /// Resets the `value` to zero.
///     Reset {},
/// }
/// 
/// impl ::union_fn::IntoOpt for Counter {
///     fn into_opt(self) -> <Counter as ::union_fn::UnionFn>::Opt {
///         match self {
///             Self::BumpBy { by } => <Counter as ::union_fn::UnionFn>::Opt::bump_by(by),
///             Self::Select { choices } => {
///                 <Counter as ::union_fn::UnionFn>::Opt::select(choices)
///             }
///             Self::Reset {} => <Counter as ::union_fn::UnionFn>::Opt::reset(),
///         }
///     }
/// }
/// 
/// impl Counter {
///     /// Bumps the value `by` the amount.
///     pub fn bump_by(by: i64) -> Self {
///         Self::BumpBy { by }
///     }
///     /// Selects the values in `choices` depending on `value`.
///     pub fn select(choices: [i64; 4]) -> Self {
///         Self::Select { choices }
///     }
///     /// Resets the `value` to zero.
///     pub fn reset() -> Self {
///         Self::Reset {}
///     }
/// }
/// 
/// impl ::union_fn::CallWithContext for Counter {
///     type Context = i64;
///     fn call(self, ctx: &mut Self::Context) -> <Counter as ::union_fn::UnionFn>::Output {
///         <<Counter as ::union_fn::UnionFn>::Opt as ::union_fn::CallWithContext>::call(
///             <Counter as ::union_fn::IntoOpt>::into_opt(self),
///             ctx,
///         )
///     }
/// }
/// 
/// const _: () = {
///     impl ::union_fn::UnionFn for CounterOpt {
///         type Output = ();
///         type Opt = Self;
///         type Args = CounterArgs;
///         type Impls = CounterImpls;
///         type Delegator = CounterDelegate;
///     }
/// 
///     impl ::union_fn::UnionFn for Counter {
///         type Output = ();
///         type Opt = CounterOpt;
///         type Args = CounterArgs;
///         type Impls = CounterImpls;
///         type Delegator = CounterDelegate;
///     }
/// 
///     ///Efficiently packed method arguments for the [`Counter`] type.
///     #[derive(Copy, Clone)]
///     pub union CounterArgs {
///         /// Bumps the value `by` the amount.
///         bump_by: i64,
///         /// Selects the values in `choices` depending on `value`.
///         select: [i64; 4],
///         /// Resets the `value` to zero.
///         reset: (),
///     }
/// 
///     impl CounterArgs {
///         /// Bumps the value `by` the amount.
///         pub fn bump_by(by: i64) -> Self {
///             Self { bump_by: by }
///         }
/// 
///         /// Selects the values in `choices` depending on `value`.
///         pub fn select(choices: [i64; 4]) -> Self {
///             Self { select: choices }
///         }
/// 
///         /// Resets the `value` to zero.
///         pub fn reset() -> Self {
///             Self { reset: () }
///         }
///     }
/// 
///     ///Decodes and delegates packed arguments to the implementation of [`Counter`] methods.
///     pub enum CounterDelegate {}
/// 
///     impl CounterDelegate {
///         /// Bumps the value `by` the amount.
///         fn bump_by(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             args: <Counter as ::union_fn::UnionFn>::Args,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             let by = unsafe { args.bump_by };
///             <Counter as ::union_fn::UnionFn>::Impls::bump_by(value, by)
///         }
/// 
///         /// Selects the values in `choices` depending on `value`.
///         fn select(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             args: <Counter as ::union_fn::UnionFn>::Args,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             let choices = unsafe { args.select };
///             <Counter as ::union_fn::UnionFn>::Impls::select(value, choices)
///         }
/// 
///         /// Resets the `value` to zero.
///         fn reset(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             args: <Counter as ::union_fn::UnionFn>::Args,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             let () = unsafe { args.reset };
///             <Counter as ::union_fn::UnionFn>::Impls::reset(value)
///         }
///     }
/// 
///     ///Implements all methods of the [`Counter`] type.
///     pub enum CounterImpls {}
/// 
///     impl CounterImpls {
///         /// Bumps the value `by` the amount.
///         fn bump_by(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             by: i64,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             *value += by;
///         }
/// 
///         /// Selects the values in `choices` depending on `value`.
///         fn select(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             choices: [i64; 4],
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             *value = choices.get(*value as usize).copied().unwrap_or(0);
///         }
/// 
///         /// Resets the `value` to zero.
///         fn reset(
///             value: &mut <Counter as ::union_fn::CallWithContext>::Context,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             *value = 0;
///         }
///     }
/// 
///     ///Call optimized structure of the [`Counter`] type.
///     #[derive(Copy, Clone)]
///     pub struct CounterOpt {
///         handler: fn(
///             ctx: &mut <Counter as ::union_fn::CallWithContext>::Context,
///             <Counter as ::union_fn::UnionFn>::Args,
///         ) -> <Counter as ::union_fn::UnionFn>::Output,
///         args: <Counter as ::union_fn::UnionFn>::Args,
///     }
/// 
///     impl ::union_fn::CallWithContext for CounterOpt {
///         type Context = i64;
///         fn call(
///             self,
///             ctx: &mut Self::Context,
///         ) -> <Counter as ::union_fn::UnionFn>::Output {
///             (self.handler)(ctx, self.args)
///         }
///     }
/// 
///     impl CounterOpt {
///         /// Bumps the value `by` the amount.
///         pub fn bump_by(by: i64) -> Self {
///             Self {
///                 handler: <Counter as ::union_fn::UnionFn>::Delegator::bump_by,
///                 args: <Counter as ::union_fn::UnionFn>::Args::bump_by(by),
///             }
///         }
/// 
///         /// Selects the values in `choices` depending on `value`.
///         pub fn select(choices: [i64; 4]) -> Self {
///             Self {
///                 handler: <Counter as ::union_fn::UnionFn>::Delegator::select,
///                 args: <Counter as ::union_fn::UnionFn>::Args::select(choices),
///             }
///         }
/// 
///         /// Resets the `value` to zero.
///         pub fn reset() -> Self {
///             Self {
///                 handler: <Counter as ::union_fn::UnionFn>::Delegator::reset,
///                 args: <Counter as ::union_fn::UnionFn>::Args::reset(),
///             }
///         }
///     }
/// };
/// ```
#[proc_macro_attribute]
pub fn union_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    analyse::union_fn(attr.into(), item.into()).into()
}

struct UnionFn {
    /// The underlying original Rust trait item.
    item: syn::ItemTrait,
    /// Extraneous state required for analysis and expansion.
    state: UnionFnState,
}

impl UnionFn {
    /// Returns the span of the `#[union_fn]` trait.
    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns the identifier of the `#[union_fn]` trait.
    pub fn ident(&self) -> &syn::Ident {
        &self.item.ident
    }

    /// Returns the identifier for the call optimized `#[union_fn]` type.
    pub fn ident_opt(&self) -> syn::Ident {
        format_ident!("{}Opt", self.ident())
    }

    /// Returns the identifier for the args union `#[union_fn]` type.
    pub fn ident_args(&self) -> syn::Ident {
        format_ident!("{}Args", self.ident())
    }

    /// Returns the identifier for the impls `#[union_fn]` type.
    pub fn ident_impls(&self) -> syn::Ident {
        format_ident!("{}Impls", self.ident())
    }

    /// Returns the identifier for the delegate `#[union_fn]` type.
    pub fn ident_delegate(&self) -> syn::Ident {
        format_ident!("{}Delegate", self.ident())
    }

    /// Returns an iterator over the `#[union_fn]` methods.
    pub fn methods(&self) -> impl Iterator<Item = UnionFnMethod> {
        self.item
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Method(item) => Some(item),
                _ => None,
            })
            .map(UnionFnMethod::from)
    }

    /// Expand to the `#[union_fn]` `Output` type if any or `()`.
    pub fn output_type(&self) -> syn::Type {
        self.state.get_output_type(self.span())
    }
}
