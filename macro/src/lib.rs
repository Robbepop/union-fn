use generate::UnionFnState;
use method::UnionFnMethod;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::spanned::Spanned;

#[macro_use]
mod error;
mod expand;
mod generate;
mod method;
mod utils;

#[proc_macro_attribute]
pub fn union_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate::union_fn(attr.into(), item.into()).into()
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
