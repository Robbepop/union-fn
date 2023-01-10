use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote_spanned};
use syn::spanned::Spanned;

/// Turns `args` into a Rust tuple type.
///
/// # Note
///
/// - Returns `()` if `args` is empty.
/// - Returns `T` if `args` represents a single `T` element.
/// - Returns `(T1, T2, ..)` otherwise.
///
/// Uses `span` as the base span for expansion.
pub fn make_tuple_type<I, T>(span: Span, args: I) -> TokenStream2
where
    I: IntoIterator<Item = T>,
    T: ToTokens,
{
    let args = args.into_iter().collect::<Vec<_>>();
    match args.len() {
        0 => quote_spanned!(span=> () ),
        1 => {
            let fst = &args[0];
            quote_spanned!(fst.span()=> #fst)
        }
        _ => {
            quote_spanned!(span=>
                ( #( #args ),* )
            )
        }
    }
}
