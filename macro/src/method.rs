use crate::generate::UnionFnState;
use proc_macro2::Span;
use quote::format_ident;
use syn::spanned::Spanned as _;

/// A method of the `#[union_fn]` trait.
///
/// Allows to provide an API based on the fact that the methods
/// have previously been checked to align to syntactical preconditions.
pub struct UnionFnMethod<'a> {
    item: &'a syn::TraitItemMethod,
}

impl<'a> From<&'a syn::TraitItemMethod> for UnionFnMethod<'a> {
    fn from(item: &'a syn::TraitItemMethod) -> Self {
        Self { item }
    }
}

impl<'a> UnionFnMethod<'a> {
    /// Returns the span of the method.
    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns the identifier of the method.
    pub fn ident(&self) -> &syn::Ident {
        &self.item.sig.ident
    }

    /// Returns the attributes of the method.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Returns the inputs of the method without the context parameter.
    ///
    /// This returns the inputs exactly as they are found in the proc macro invocation.
    pub fn inputs(&self, state: &UnionFnState) -> Vec<&syn::PatType> {
        let mut iter = self.item.sig.inputs.iter().filter_map(|item| match item {
            syn::FnArg::Receiver(receiver) => {
                panic!("encountered invalid self receiver: {receiver:?}")
            }
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        });
        if state.get_context().is_some() {
            // If the trait has a context we need to pop the context argument.
            let _ = iter.next();
        }
        iter.collect()
    }

    /// Returns the input types of the method without the context parameter.
    pub fn input_types(&self, state: &UnionFnState) -> Vec<&syn::Type> {
        self.inputs(state)
            .iter()
            .map(|pat_type| &*pat_type.ty)
            .collect()
    }

    /// Returns the input bindings of the method inputs without the context parameter.
    ///
    /// # Note
    ///
    /// This replaces the patterns of the inputs if they are not identifiers
    /// with artificial numbered identifiers in the form `_N`. This is required
    /// for some proc. macro expansions.
    pub fn input_bindings(&self, state: &UnionFnState) -> Vec<syn::Ident> {
        self.inputs(state)
            .into_iter()
            .enumerate()
            .map(|(n, pat_type)| Self::ident_or_numbered(&pat_type.pat, n))
            .collect()
    }

    /// Returns input bindings without the context parameter of the method.
    ///
    /// # Note
    ///
    /// This replaces the patterns of the inputs if they are not identifiers
    /// with artificial numbered identifiers in the form `_N`. This is required
    /// for some proc. macro expansions.
    pub fn ident_inputs(&self, state: &UnionFnState) -> Vec<syn::PatType> {
        self.inputs(state)
            .into_iter()
            .enumerate()
            .map(|(n, pat_type)| {
                let ident = Self::ident_or_numbered(&pat_type.pat, n);
                let pat = syn::parse_quote!(#ident);
                let attrs = pat_type.attrs.clone();
                let colon_token = pat_type.colon_token;
                let ty = pat_type.ty.clone();
                syn::PatType {
                    attrs,
                    pat,
                    colon_token,
                    ty,
                }
            })
            .collect()
    }

    /// Returns the identifier of the given pattern or generated a numbered dummy.
    ///
    /// Returns an identifier if the pattern is equivalent to an identifier
    /// and otherwise returns an artificial numbered identifier in the form
    /// `_N`.
    fn ident_or_numbered(pat: &syn::Pat, n: usize) -> syn::Ident {
        let make_numbered = || format_ident!("_{}", n);
        if let syn::Pat::Path(pat_path) = pat {
            if pat_path.qself.is_some() {
                return make_numbered();
            }
            if let Some(ident) = pat_path.path.get_ident() {
                return ident.clone();
            }
        }
        make_numbered()
    }

    /// Returns the context parameter pattern of the method if any.
    ///
    /// # Note
    ///
    /// We are not interested in the type since we previously asserted that
    /// it is always just a reference to the associated trait `&mut Self::Context`.
    pub fn context(&self, state: &UnionFnState) -> Option<&syn::Pat> {
        state
            .get_context()
            .map(|_| &self.item.sig.inputs[0])
            .and_then(|arg| match arg {
                syn::FnArg::Receiver(receiver) => {
                    panic!("encountered invalid self receiver: {receiver:?}")
                }
                syn::FnArg::Typed(pat_type) => Some(&*pat_type.pat),
            })
    }

    /// Returns the default implementation block of the method.
    pub fn impl_block(&self) -> &syn::Block {
        self.item
            .default
            .as_ref()
            .expect("all `#[union_fn]` methods have a default implementation")
    }
}
