use heck::AsUpperCamelCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote_spanned, ToTokens};
use syn::spanned::Spanned;

/// Extension methods for [`syn::Attribute`].
pub trait AttributeExt {
    /// Returns `true` if the [`Attribute`] is a Rust documentation attribute.
    ///
    /// [`Attribute`]: [`syn::Attribute`]
    fn is_docs_attribute(&self) -> bool;

    /// Returns `Some` if the [`Attribute`] is a Rust doc attribute.
    ///
    /// [`Attribute`]: [`syn::Attribute`]
    fn filter_docs(&self) -> Option<&syn::Attribute>;

    /// Returns the contents of the [`Attribute`] if it is a Rust doc attribute
    ///
    /// Otherwise returns `None`.
    fn get_docs(&self) -> Option<syn::LitStr>;
}

impl AttributeExt for syn::Attribute {
    fn is_docs_attribute(&self) -> bool {
        self.path.is_ident("doc")
    }

    fn filter_docs(&self) -> Option<&syn::Attribute> {
        if self.is_docs_attribute() {
            return Some(self);
        }
        None
    }

    fn get_docs(&self) -> Option<syn::LitStr> {
        self.filter_docs()
            .and_then(|attr| match attr.parse_meta().ok()? {
                syn::Meta::NameValue(syn::MetaNameValue {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => Some(lit_str),
                _ => None,
            })
    }
}

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
