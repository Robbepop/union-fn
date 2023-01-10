use crate::{error::ExtError, utils::make_tuple_type};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote_spanned, ToTokens};
use std::collections::VecDeque;
use syn::{spanned::Spanned, Result};

pub fn func_union(args: TokenStream2, item: TokenStream2) -> TokenStream2 {
    UnionFn::new(args, item)
        .map(|func| func.expand())
        .unwrap_or_else(|error| error.to_compile_error())
}

struct UnionFn {
    /// The underlying original Rust trait item.
    item: syn::ItemTrait,
    /// Extraneous state required for analysis and expansion.
    state: UnionFnState,
}

/// State required for [`UnionFn`] analysis and expansion.
#[derive(Default)]
struct UnionFnState {
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

    /// Registers a method signature of the #[union_fn] trait.
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
                        if &*pat_type.ty != &syn::parse_quote!(&mut Self::Context) {
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
        let span = self.item.span();
        let reflect = self.expand_reflection();
        let args = self.expand_args();
        let delegator = self.expand_delegator();
        let handler = self.expand_union_fn();
        quote_spanned!(span=>
            const _: () = {
                #reflect
                #args
                #delegator
            };
            #handler
        )
    }

    fn expand_reflection(&self) -> TokenStream2 {
        let span = self.item.span();
        let ident = &self.item.ident;
        let output = self.expand_output_type();
        quote_spanned!(span=>
            impl ::union_fn::UnionFn for #ident {
                type Output = #output;
                type Args = UnionFnArgs;
                type Delegator = UnionFnDelegator;
            }
        )
    }

    fn expand_delegator(&self) -> TokenStream2 {
        let span = self.item.span();
        let ident = &self.item.ident;
        let delegators = self.item.items.iter().filter_map(|item| match item {
            syn::TraitItem::Method(item) => Some(item),
            _ => None,
        }).map(|method| {
            let span = method.span();
            let method_ident = &method.sig.ident;
            let impl_ident = quote::format_ident!("_{}_impl", method_ident);
            let ctx_ident = self.state.get_context().map(|_| &method.sig.inputs[0]).and_then(|fn_arg| match fn_arg {
                syn::FnArg::Typed(pat_type) => Some(pat_type),
                syn::FnArg::Receiver(_) => None,
            }).map(|pat_type| {
                let span = pat_type.span();
                let pat = &pat_type.pat;
                quote_spanned!(span=>
                    #pat: &mut <#ident as ::union_fn::CallWithContext>::Context,
                )
            });
            let ctx_param = self.state.get_context().map(|_| {
                quote_spanned!(span=>
                    ctx: &mut <#ident as ::union_fn::CallWithContext>::Context,
                )
            });
            let ctx_arg = self.state.get_context().map(|_| {
                quote_spanned!(span=>
                    ctx,
                )
            });
            let mut args = method.sig.inputs.iter().filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => Some(pat_type),
                syn::FnArg::Receiver(_) => None
            }).collect::<VecDeque<_>>();
            if self.state.get_context().is_some() {
                // Throw away context argument before processing.
                args.pop_front();
            }
            let bindings = (0..args.len()).map(|index| quote::format_ident!("_{}", index)).collect::<Vec<_>>();
            let args = args.iter().collect::<Vec<_>>();
            let block = method.default.as_ref().unwrap();
            let tuple_bindings = make_tuple_type(span, &bindings);

            quote_spanned!(span=>
                fn #method_ident( #ctx_param args: <#ident as ::union_fn::UnionFn>::Args ) -> <#ident as ::union_fn::UnionFn>::Output {
                    let #tuple_bindings = unsafe { args.#method_ident };
                    Self::#impl_ident( #ctx_arg #( #bindings ),* )
                }

                fn #impl_ident( #ctx_ident #( #args ),* ) -> <#ident as ::union_fn::UnionFn>::Output #block
            )
        });
        quote_spanned!(span=>
            pub enum UnionFnDelegator {}

            impl UnionFnDelegator {
                #( #delegators )*
            }
        )
    }

    fn expand_union_fn(&self) -> TokenStream2 {
        let span = self.item.span();
        let ident = &self.item.ident;
        let call_impl = self.expand_call_impl();
        let constructors = self.expand_constructors();
        let ctx = self.state.get_context().map(|_| {
            quote_spanned!(span=>
                ctx: &mut <#ident as ::union_fn::CallWithContext>::Context,
            )
        });
        quote_spanned!(span=>
            #[derive(::core::marker::Copy, ::core::clone::Clone)]
            pub struct #ident {
                handler: fn(#ctx <#ident as ::union_fn::UnionFn>::Args) -> <#ident as ::union_fn::UnionFn>::Output,
                args: <#ident as ::union_fn::UnionFn>::Args,
            }

            #call_impl
            #constructors
        )
    }

    fn expand_constructors(&self) -> TokenStream2 {
        let span = self.item.span();
        let ident = &self.item.ident;
        let handlers = self.item.items.iter().filter_map(|item| match item {
            syn::TraitItem::Method(item) => Some(item),
            _ => None,
        }).map(|method| {
            let span = method.span();
            let method_ident = &method.sig.ident;
            let mut args = method.sig.inputs.iter().filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => Some(pat_type),
                syn::FnArg::Receiver(_) => None
            }).collect::<VecDeque<_>>();
            if self.state.get_context().is_some() {
                // Throw away context argument before processing.
                args.pop_front();
            }
            let bindings = (0..args.len()).map(|index| quote::format_ident!("_{}", index)).collect::<Vec<_>>();
            let args = args.iter().collect::<Vec<_>>();
            let constructor_args = args.iter().enumerate().map(|(n, arg)| {
                let span = arg.span();
                let binding = quote::format_ident!("_{}", n);
                let ty = &arg.ty;
                quote_spanned!(span=>
                    #binding: #ty
                )
            });

            quote_spanned!(span=>
                pub fn #method_ident( #( #constructor_args ),* ) -> Self {
                    Self {
                        handler: <#ident as ::union_fn::UnionFn>::Delegator::#method_ident,
                        args: <#ident as ::union_fn::UnionFn>::Args::#method_ident( #( #bindings ),* ),
                    }
                }
            )
        });
        quote_spanned!(span=>
            impl #ident {
                #( #handlers )*
            }
        )
    }

    fn expand_output_type(&self) -> TokenStream2 {
        let span = self.item.span();
        match self.state.get_output() {
            Some(output) => {
                quote_spanned!(span=> #output)
            }
            None => self
                .state
                .signature
                .as_ref()
                .map(|sig| match &sig.output {
                    syn::ReturnType::Default => quote_spanned!(span=> ()),
                    syn::ReturnType::Type(_, ty) => quote_spanned!(span=> #ty),
                })
                .unwrap_or_else(|| quote_spanned!(span=> ())),
        }
    }

    fn expand_call_impl(&self) -> TokenStream2 {
        let span = self.item.span();
        let ident = &self.item.ident;
        match self.state.get_context() {
            Some(context) => {
                quote_spanned!(span=>
                    impl ::union_fn::CallWithContext for #ident {
                        type Context = #context;

                        fn call(self, ctx: &mut Self::Context) -> <#ident as ::union_fn::UnionFn>::Output {
                            (self.handler)(ctx, self.args)
                        }
                    }
                )
            }
            None => {
                quote_spanned!(span=>
                    impl ::union_fn::Call for #ident {
                        fn call(self) -> <#ident as ::union_fn::UnionFn>::Output {
                            (self.handler)(self.args)
                        }
                    }
                )
            }
        }
    }

    fn expand_args(&self) -> TokenStream2 {
        let variants = self
            .item
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Method(item) => Some(item),
                _ => None,
            })
            .map(|method| {
                let ident = &method.sig.ident;
                let mut arg_types = method.sig.inputs.iter().filter_map(|arg| match arg {
                    syn::FnArg::Typed(pat_type) => Some(&pat_type.ty),
                    syn::FnArg::Receiver(_) => None,
                });
                if self.state.get_context().is_some() {
                    // Throw away context argument before processing.
                    let _ = arg_types.next();
                }
                let inputs = make_tuple_type(method.span(), arg_types);
                quote_spanned!(method.span() =>
                    #ident: #inputs
                )
            });
        let constructors = self
            .item
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Method(item) => Some(item),
                _ => None,
            })
            .map(|method| {
                let span = method.span();
                let method_ident = &method.sig.ident;
                let mut args = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => Some(pat_type),
                        syn::FnArg::Receiver(_) => None,
                    })
                    .collect::<VecDeque<_>>();
                if self.state.get_context().is_some() {
                    // Throw away context argument before processing.
                    args.pop_front();
                }
                let constructor_args = args.iter().enumerate().map(|(n, arg)| {
                    let span = arg.span();
                    let binding = quote::format_ident!("_{}", n);
                    let ty = &arg.ty;
                    quote_spanned!(span=>
                        #binding: #ty
                    )
                });
                let bindings = (0..args.len())
                    .map(|index| quote::format_ident!("_{}", index))
                    .collect::<Vec<_>>();
                let constructor_params = make_tuple_type(span, &bindings);
                quote_spanned!(span=>
                    pub fn #method_ident( #( #constructor_args ),* ) -> Self {
                        Self { #method_ident: #constructor_params }
                    }
                )
            });
        quote_spanned!(self.item.span() =>
            #[derive(core::marker::Copy, core::clone::Clone)]
            pub union UnionFnArgs {
                #( #variants ),*
            }

            impl UnionFnArgs {
                #( #constructors )*
            }
        )
    }
}
