use proc_macro::TokenStream;

#[macro_use]
mod error;

mod generate;

#[proc_macro_attribute]
pub fn union_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate::func_union(attr.into(), item.into()).into()
}
