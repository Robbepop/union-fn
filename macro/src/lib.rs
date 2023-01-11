use proc_macro::TokenStream;

#[macro_use]
mod error;
mod generate;
mod utils;

#[proc_macro_attribute]
pub fn union_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate::union_fn(attr.into(), item.into()).into()
}
