use crate::error::ExtError;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{spanned::Spanned, Result};

pub fn func_union(args: TokenStream2, item: TokenStream2) -> TokenStream2 {
    UnionFn::new(args, item)
        .map(|func| func.expand())
        .unwrap_or_else(|error| error.to_compile_error())
}

pub struct UnionFn {
    /// The underlying original Rust trait item.
    item: syn::ItemTrait,
    /// Extraneous state required for analysis and expansion.
    state: UnionFnState,
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
pub struct SharedSignature {
    pub span: Span,
    pub constness: Option<syn::token::Const>,
    pub asyncness: Option<syn::token::Async>,
    pub unsafety: Option<syn::token::Unsafe>,
    pub abi: Option<syn::Abi>,
    pub output: syn::ReturnType,
}

impl SharedSignature {
    pub fn constness_span(&self) -> Span {
        self.constness.map(|c| c.span()).unwrap_or(self.span)
    }

    pub fn asyncness_span(&self) -> Span {
        self.asyncness.map(|c| c.span()).unwrap_or(self.span)
    }

    pub fn unsafety_span(&self) -> Span {
        self.unsafety.map(|c| c.span()).unwrap_or(self.span)
    }

    pub fn abi_span(&self) -> Span {
        self.abi.as_ref().map(|c| c.span()).unwrap_or(self.span)
    }
}

impl UnionFnState {
    pub fn set_context(&mut self, item: &syn::TraitItemType) -> Result<()> {
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
                "cannot have generics for Output type in #[union_fn] trait"
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
        self.context = Some(item.clone());
        Ok(())
    }

    pub fn get_context(&self) -> Option<&syn::Type> {
        if let Some(context) = self.context.as_ref() {
            return Some(&context.default.as_ref().unwrap().1);
        }
        None
    }

    pub fn set_output(&mut self, item: &syn::TraitItemType) -> Result<()> {
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

    pub fn get_output(&self) -> Option<&syn::Type> {
        if let Some(output) = self.output.as_ref() {
            return Some(&output.default.as_ref().unwrap().1);
        }
        None
    }

    /// Registers an associated type of the `#[union_fn]` trait if valid.
    ///
    /// # Errors
    ///
    /// If an unsupported or invalid type structure is encountered.
    pub fn register_type(&mut self, item: &syn::TraitItemType) -> syn::Result<()> {
        if item.ident == "Context" {
            return self.set_context(item);
        }
        if item.ident == "Output" {
            return self.set_output(item);
        }
        bail_spanned!(
            item,
            "encountered unsupported associated type for #[union_fn] trait"
        )
    }

    /// Registers a method signature of the #[union_fn] trait.
    ///
    /// # Errors
    ///
    /// If there is a signature mismatch between methods.
    fn register_sigature(&mut self, sig: &syn::Signature) -> syn::Result<()> {
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
                if sig.constness != signature.constness {
                    return format_err!(
                        sig.constness.map(|c| c.span()).unwrap_or(span),
                        "encountered mismatch in constness for #[union_fn] method"
                    )
                    .into_combine(format_err!(
                        signature.constness_span(),
                        "mismatch with this method"
                    ))
                    .into_result();
                }
                if sig.asyncness != signature.asyncness {
                    return format_err!(
                        sig.asyncness.map(|c| c.span()).unwrap_or(span),
                        "encountered mismatch in asyncness for #[union_fn] method"
                    )
                    .into_combine(format_err!(
                        signature.asyncness_span(),
                        "mismatch with this method"
                    ))
                    .into_result();
                }
                if sig.unsafety != signature.unsafety {
                    return format_err!(
                        sig.unsafety.map(|c| c.span()).unwrap_or(span),
                        "encountered mismatch in unsafety for #[union_fn] method"
                    )
                    .into_combine(format_err!(
                        signature.unsafety_span(),
                        "mismatch with this method"
                    ))
                    .into_result();
                }
                if sig.abi != signature.abi {
                    return format_err!(
                        sig.abi.as_ref().map(|c| c.span()).unwrap_or(span),
                        "encountered mismatch in abi for #[union_fn] method"
                    )
                    .into_combine(format_err!(
                        signature.abi_span(),
                        "mismatch with this method"
                    ))
                    .into_result();
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
        match self.get_output() {
            Some(output) => {
                let make_err = |error: &dyn ToTokens| {
                    format_err_spanned!(error, "must return Self::Output or equivalent type")
                        .into_combine(format_err_spanned!(output, "since Context is defined here"))
                        .into_result()
                };
                match &item.sig.output {
                    syn::ReturnType::Default => return make_err(item),
                    syn::ReturnType::Type(_, ty) => {
                        if !(**ty == syn::parse_quote!(Self::Output) || &**ty == output) {
                            return make_err(ty);
                        }
                    }
                }
            }
            None => match item.sig.output {
                syn::ReturnType::Default => (), // ok
                syn::ReturnType::Type(_, _) => bail_spanned!(
                    item,
                    "cannot have a return type if Output is undefined in #[union_fn]"
                ),
            },
        }
        if let Some(context) = self.get_context() {
            let make_err = |error: &dyn ToTokens| {
                format_err_spanned!(
                    error,
                    "must have `ctx: &mut Self::Context` as first argument"
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
                        if !(arg == &syn::parse_quote!(ctx: &mut Self::Context)
                            || &*pat_type.ty == &syn::parse_quote!(&mut #context))
                        {
                            return make_err(pat_type);
                        }
                    }
                },
                None => return make_err(&item.sig.inputs),
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
            .map(|item| Self::analyze_item(state, item))
            .collect::<Result<()>>()?;
        Ok(())
    }

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

    /// Expands the parsed and analyzed [`UnionFn`] to proper Rust code.
    pub fn expand(&self) -> TokenStream2 {
        TokenStream2::new()
    }
}
