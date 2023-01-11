use heck::AsUpperCamelCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote_spanned, ToTokens};
use syn::spanned::Spanned;

/// Extension methods for [`syn::Ident`].
pub trait IdentExt {
    /// Converts the identifier to an upper camel case identifier.
    fn to_upper_camel_case(&self) -> syn::Ident;
}

impl IdentExt for syn::Ident {
    fn to_upper_camel_case(&self) -> syn::Ident {
        format_ident!("{}", AsUpperCamelCase(self.to_string()).to_string())
    }
}

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
