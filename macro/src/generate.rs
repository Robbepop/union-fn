use crate::{error::ExtError, utils::make_tuple_type};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Result};

pub fn union_fn(args: TokenStream2, item: TokenStream2) -> TokenStream2 {
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

impl UnionFn {
    /// Returns the span of the `#[union_fn]` trait.
    fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns the identifier of the `#[union_fn]` trait.
    fn ident(&self) -> &syn::Ident {
        &self.item.ident
    }

    /// Returns an iterator over the `#[union_fn]` methods.
    fn methods(&self) -> impl Iterator<Item = UnionFnMethod> {
        self.item
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Method(item) => Some(item),
                _ => None,
            })
            .map(UnionFnMethod::from)
    }
}

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

impl UnionFn {
    /// Expands the parsed and analyzed [`UnionFn`] to proper Rust code.
    pub fn expand(&self) -> TokenStream2 {
        let span = self.item.span();
        let reflect = self.expand_reflection();
        let args = self.expand_union_args();
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

    /// Exapnds the code to implement the base `UnionFn` trait.
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

    /// Expand hidden delegators from `UnionFnArgs` to actual function parameters and implementations.
    fn expand_delegator(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let delegates = self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let impl_ident = format_ident!("_{}_impl", method_ident);
            let impl_block = method.impl_block();
            let ctx_ident = method.context(&self.state).map(|ctx| quote_spanned!(method_span=> #ctx,));
            let ctx_param = method.context(&self.state).map(|ctx| quote_spanned!(method_span=> #ctx: &mut <#trait_ident as ::union_fn::CallWithContext>::Context,));
            let params = method.inputs(&self.state);
            let bindings = method.input_bindings(&self.state);
            let tuple_bindings = make_tuple_type(method_span, &bindings);
            quote_spanned!(method_span=>
                fn #method_ident( #ctx_param args: <#trait_ident as ::union_fn::UnionFn>::Args ) -> <#trait_ident as ::union_fn::UnionFn>::Output {
                    let #tuple_bindings = unsafe { args.#method_ident };
                    Self::#impl_ident( #ctx_ident #( #bindings ),* )
                }

                fn #impl_ident( #ctx_param #( #params ),* ) -> <#trait_ident as ::union_fn::UnionFn>::Output #impl_block
            )
        });
        quote_spanned!(trait_span=>
            pub enum UnionFnDelegator {}

            impl UnionFnDelegator {
                #( #delegates )*
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

    /// Expand the `#[union_fn]` constructors.
    fn expand_constructors(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let constructors = self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let params = method.ident_inputs(&self.state);
            let param_bindings = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self {
                        handler: <#trait_ident as ::union_fn::UnionFn>::Delegator::#method_ident,
                        args: <#trait_ident as ::union_fn::UnionFn>::Args::#method_ident( #( #param_bindings ),* ),
                    }
                }
            )
        });
        quote_spanned!(trait_span=>
            impl #trait_ident {
                #( #constructors )*
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

    fn expand_union_args(&self) -> TokenStream2 {
        let trait_span = self.span();
        let variants = self.expand_union_args_variants();
        let constructors = self.expand_union_args_constructors();
        quote_spanned!(trait_span =>
            #[derive(core::marker::Copy, core::clone::Clone)]
            pub union UnionFnArgs {
                #( #variants ),*
            }

            impl UnionFnArgs {
                #( #constructors )*
            }
        )
    }

    /// Expands the `#[union_fn]` union variants.
    fn expand_union_args_variants(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let params = method.input_types(&self.state);
            let tuple_params = make_tuple_type(method_span, &params);
            quote_spanned!(method_span =>
                #method_ident: #tuple_params
            )
        })
    }

    /// Expands the `#[union_fn]` union variant constructors.
    fn expand_union_args_constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let params = method.ident_inputs(&self.state);
            let param_bindings = method.input_bindings(&self.state);
            let tuple_bindings = make_tuple_type(method_span, &param_bindings);
            quote_spanned!(method_span=>
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self { #method_ident: #tuple_bindings }
                }
            )
        })
    }
}
