use crate::{error::ExtError, UnionFn};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{spanned::Spanned, Result};

pub fn union_fn(args: TokenStream2, item: TokenStream2) -> TokenStream2 {
    UnionFn::new(args, item)
        .map(|func| func.expand())
        .unwrap_or_else(|error| error.to_compile_error())
}

/// State required for [`UnionFn`] analysis and expansion.
#[derive(Default)]
pub struct UnionFnState {
    /// The shared function context if any.
    context: Option<syn::TraitItemType>,
    /// The shared output type if any.
    output: Option<syn::TraitItemType>,
    /// Shared signature for all union functions.
    signature: Option<SharedSignature>,
}

/// The method signature shared by all functions in the [`UnionFn`].
struct SharedSignature {
    pub span: Span,
    pub constness: Option<syn::token::Const>,
    pub asyncness: Option<syn::token::Async>,
    pub unsafety: Option<syn::token::Unsafe>,
    pub abi: Option<syn::Abi>,
    pub output: syn::ReturnType,
}

impl SharedSignature {
    /// Returns the span of the shared `const` token or the signature itself.
    pub fn constness_span(&self) -> Span {
        self.constness.map(|c| c.span()).unwrap_or(self.span)
    }

    /// Returns the span of the shared `async` token or the signature itself.
    pub fn asyncness_span(&self) -> Span {
        self.asyncness.map(|c| c.span()).unwrap_or(self.span)
    }

    /// Returns the span of the shared `unsafe` token or the signature itself.
    pub fn unsafety_span(&self) -> Span {
        self.unsafety.map(|c| c.span()).unwrap_or(self.span)
    }

    /// Returns the span of the shared `abi` or the signature itself.
    pub fn abi_span(&self) -> Span {
        self.abi.as_ref().map(|c| c.span()).unwrap_or(self.span)
    }
}

impl UnionFnState {
    /// Registers a context type for the `#[union_fn]` trait.
    ///
    /// # Errors
    ///
    /// - If multiple or conflicting contexts are encountered.
    /// - If the context type is invalid or uses unsupported features.
    pub fn register_context(&mut self, item: &syn::TraitItemType) -> Result<()> {
        if let Some(context) = self.context.as_ref() {
            return format_err_spanned!(
                item,
                "encountered conflicting Context types in #[union_fn] trait"
            )
            .into_combine(format_err_spanned!(
                context,
                "previous Context definition here"
            ))
            .into_result();
        }
        if !item.generics.params.is_empty() {
            bail_spanned!(
                item.generics,
                "cannot have generics for Context type in #[union_fn] trait"
            )
        }
        if let Some(where_clause) = &item.generics.where_clause {
            bail_spanned!(
                where_clause,
                "cannot have where clause for Context type in #[union_fn] trait"
            )
        }
        if !item.bounds.is_empty() {
            bail_spanned!(
                item.bounds,
                "cannot have trait bounds for Context type in #[union_fn] trait"
            )
        }
        if item.default.is_none() {
            bail_spanned!(
                item,
                "must have a default for Context type in #[union_fn] trait"
            )
        }
        self.context = Some(item.clone());
        Ok(())
    }

    /// Returns a shared reference the to registered context type if any.
    pub fn get_context(&self) -> Option<&syn::Type> {
        if let Some(context) = self.context.as_ref() {
            return Some(&context.default.as_ref().unwrap().1);
        }
        None
    }

    /// Registers an output type for the `#[union_fn]` trait.
    ///
    /// # Errors
    ///
    /// - If multiple or conflicting output types are encountered.
    /// - If the output type is invalid or uses unsupported features.
    pub fn register_output(&mut self, item: &syn::TraitItemType) -> Result<()> {
        if let Some(output) = self.output.as_ref() {
            return format_err_spanned!(
                item,
                "encountered conflicting Output types in #[union_fn] trait"
            )
            .into_combine(format_err_spanned!(
                output,
                "previous Output definition here"
            ))
            .into_result();
        }
        if !item.generics.params.is_empty() {
            bail_spanned!(
                item.generics,
                "cannot have generics for Output type in #[union_fn] trait"
            )
        }
        if let Some(where_clause) = &item.generics.where_clause {
            bail_spanned!(
                where_clause,
                "cannot have where clause for Output type in #[union_fn] trait"
            )
        }
        if !item.bounds.is_empty() {
            bail_spanned!(
                item.bounds,
                "cannot have bounds for Output type in #[union_fn] trait"
            )
        }
        if item.default.is_none() {
            bail_spanned!(
                item,
                "must have a default for Output type in #[union_fn] trait"
            )
        }
        self.output = Some(item.clone());
        Ok(())
    }

    /// Returns a shared reference the to registered output type if any.
    pub fn get_output(&self) -> Option<&syn::Type> {
        if let Some(output) = self.output.as_ref() {
            return Some(&output.default.as_ref().unwrap().1);
        }
        None
    }

    /// Expand to the `#[union_fn]` `Output` type if any or `()`.
    pub fn get_output_type(&self, span: Span) -> syn::Type {
        let empty_tuple = || syn::parse_quote_spanned!(span=> ());
        match self.get_output() {
            Some(output) => output.clone(),
            None => self
                .signature
                .as_ref()
                .map(|sig| match &sig.output {
                    syn::ReturnType::Default => empty_tuple(),
                    syn::ReturnType::Type(_, ty) => (**ty).clone(),
                })
                .unwrap_or_else(empty_tuple),
        }
    }

    /// Registers an associated type of the `#[union_fn]` trait if valid.
    ///
    /// # Errors
    ///
    /// If an unsupported or invalid type structure is encountered.
    pub fn register_type(&mut self, item: &syn::TraitItemType) -> syn::Result<()> {
        if item.ident == "Context" {
            return self.register_context(item);
        }
        if item.ident == "Output" {
            return self.register_output(item);
        }
        bail_spanned!(
            item,
            "encountered unsupported associated type for #[union_fn] trait"
        )
    }

    /// Registers a method signature of the `#[union_fn]` trait.
    ///
    /// # Errors
    ///
    /// If there is a signature mismatch between methods.
    fn register_sigature(&mut self, sig: &syn::Signature) -> syn::Result<()> {
        if !sig.generics.params.is_empty() {
            bail_spanned!(sig.generics, "must not be generic")
        }
        if let Some(where_clause) = &sig.generics.where_clause {
            bail_spanned!(where_clause, "must not have a where clause")
        }
        if let Some(variadic) = &sig.variadic {
            bail_spanned!(variadic, "must not have variadic arguments")
        }
        match self.signature.as_ref() {
            None => {
                self.signature = Some(SharedSignature {
                    span: sig.span(),
                    constness: sig.constness,
                    asyncness: sig.asyncness,
                    unsafety: sig.unsafety,
                    abi: sig.abi.clone(),
                    output: sig.output.clone(),
                })
            }
            Some(signature) => {
                let span = sig.span();
                let make_err =
                    |err_span: Option<Span>, context: &str, mis_span: Span| -> syn::Result<()> {
                        format_err!(
                            err_span.map(|c| c.span()).unwrap_or(span),
                            "encountered mismatch in {context} for #[union_fn] method"
                        )
                        .into_combine(format_err!(mis_span, "mismatch with this method"))
                        .into_result()
                    };
                if sig.constness != signature.constness {
                    return make_err(
                        sig.constness.as_ref().map(Spanned::span),
                        "constness",
                        signature.constness_span(),
                    );
                }
                if sig.asyncness != signature.asyncness {
                    return make_err(
                        sig.asyncness.as_ref().map(Spanned::span),
                        "asyncness",
                        signature.asyncness_span(),
                    );
                }
                if sig.unsafety != signature.unsafety {
                    return make_err(
                        sig.unsafety.as_ref().map(Spanned::span),
                        "unsafety",
                        signature.unsafety_span(),
                    );
                }
                if sig.abi != signature.abi {
                    return make_err(
                        sig.abi.as_ref().map(Spanned::span),
                        "abi",
                        signature.abi_span(),
                    );
                }
            }
        }
        Ok(())
    }

    /// Registers an associated method of the `#[union_fn]` trait if valid.
    ///
    /// # Errors
    ///
    /// If an unsupported or invalid method structure is encountered.
    pub fn register_method(&mut self, item: &syn::TraitItemMethod) -> syn::Result<()> {
        self.register_sigature(&item.sig)?;
        if let Some(output) = self.get_output() {
            let make_err = |error: &dyn ToTokens| {
                format_err_spanned!(error, "must return Self::Output")
                    .into_combine(format_err_spanned!(output, "since Output is defined here"))
                    .into_result()
            };
            match &item.sig.output {
                syn::ReturnType::Default => return make_err(item),
                syn::ReturnType::Type(_, ty) => {
                    if **ty != syn::parse_quote!(Self::Output) {
                        return make_err(ty);
                    }
                }
            }
        }
        if item.default.is_none() {
            bail_spanned!(item, "must have default implementation")
        }
        for arg in item.sig.inputs.iter() {
            if let syn::FnArg::Receiver(receiver) = arg {
                bail_spanned!(receiver, "must not have self receiver argument")
            }
        }
        if let Some(context) = self.get_context() {
            let make_err = |error: &dyn ToTokens| {
                format_err_spanned!(
                    error,
                    "must have type of `&mut Self::Context` as first argument"
                )
                .into_combine(format_err_spanned!(
                    context,
                    "since Context is defined here"
                ))
                .into_result()
            };
            match item.sig.inputs.first() {
                Some(arg) => match arg {
                    syn::FnArg::Receiver(receiver) => bail_spanned!(
                        receiver,
                        "must not have a `self` receiver as first argument in #[union_fn] methods"
                    ),
                    syn::FnArg::Typed(pat_type) => {
                        if *pat_type.ty != syn::parse_quote!(&mut Self::Context) {
                            return make_err(pat_type);
                        }
                    }
                },
                None => return make_err(&item.sig),
            }
        }
        Ok(())
    }
}

impl UnionFn {
    /// Creates a new [`UnionFn`] from the given macro `args` and input trait `item`.
    ///
    /// # Errors
    ///
    /// If the `item` is invalid or unsupported.
    pub fn new(args: TokenStream2, item: TokenStream2) -> Result<Self> {
        if !args.is_empty() {
            bail_spanned!(args, "cannot have macro arguments for #[union_fn]")
        }
        let mut item = syn::parse2::<syn::ItemTrait>(item)?;
        Self::analyze_trait(&item)?;
        let mut state = UnionFnState::default();
        Self::sort_items(&mut item.items);
        Self::analyze_items(&mut state, &item.items)?;
        Ok(Self { item, state })
    }

    /// Analyzes the trait definition without its trait items.
    ///
    /// # Errors
    ///
    /// If the trait uses invalid or unsupported Rust features.
    fn analyze_trait(item: &syn::ItemTrait) -> syn::Result<()> {
        if let Some(token) = item.unsafety {
            bail_spanned!(
                token,
                "cannot have unsafe #[union_fn] trait; only associated methods can be unsafe"
            )
        }
        if let Some(token) = item.auto_token {
            bail_spanned!(token, "cannot have `auto` #[union_fn] trait")
        }
        if !item.generics.params.is_empty() {
            bail_spanned!(item.generics, "cannot have generic #[union_fn] trait")
        }
        if !item.supertraits.is_empty() {
            bail_spanned!(item.generics, "cannot have supertraits for union functions")
        }
        Ok(())
    }

    /// Sort items in the way the macro analysis expects them to be sorted.
    fn sort_items(items: &mut [syn::TraitItem]) {
        fn order_value(item: &syn::TraitItem) -> i32 {
            match item {
                syn::TraitItem::Const(_)
                | syn::TraitItem::Verbatim(_)
                | syn::TraitItem::Macro(_) => {
                    // Unsupported items are sorted to the start
                    // so that they are filtered out early on in
                    // the analysis process.
                    0
                }
                syn::TraitItem::Type(_) => {
                    // We need to process associated types before methods.
                    1
                }
                syn::TraitItem::Method(_) => {
                    // We need to process associated methods after types.
                    2
                }
                _ => 0, // same as with other unsupported items
            }
        }
        items.sort_by_key(order_value)
    }

    /// Analyzes the trait items and updates the `state` respectively.
    ///
    /// # Errors
    ///
    /// If unsupported or invalid items are encountered.
    fn analyze_items(state: &mut UnionFnState, items: &[syn::TraitItem]) -> Result<()> {
        items
            .iter()
            .try_for_each(|item| Self::analyze_item(state, item))?;
        Ok(())
    }

    /// Analyzes the given trait `item` and updates the `state` respectively.
    ///
    /// # Errors
    ///
    /// If the `item` is unsupported or invalid.
    fn analyze_item(state: &mut UnionFnState, item: &syn::TraitItem) -> Result<()> {
        macro_rules! unsupported {
            ($spanned:ident) => {{
                bail_spanned!(
                    $spanned,
                    "encountered unsupported trait item in #[union_fn] trait"
                )
            }};
        }
        match item {
            syn::TraitItem::Method(item) => state.register_method(item),
            syn::TraitItem::Type(item) => state.register_type(item),
            syn::TraitItem::Const(item) => unsupported!(item),
            syn::TraitItem::Macro(item) => unsupported!(item),
            syn::TraitItem::Verbatim(item) => unsupported!(item),
            unknown => bail_spanned!(unknown, "encountered unknown item in #[union_fn] trait"),
        }
    }
}
