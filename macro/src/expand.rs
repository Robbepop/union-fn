use crate::utils::IdentExt as _;
use crate::{utils::make_tuple_type, UnionFn};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned as _;

impl UnionFn {
    /// Expands the parsed and analyzed [`UnionFn`] to proper Rust code.
    pub fn expand(&self) -> TokenStream2 {
        let span = self.item.span();
        let reflect = self.expand_reflection();
        let args_type = self.expand_union_fn_args();
        let delegate_type = self.expand_union_fn_delegate();
        let impls_type = self.expand_union_fn_impls();
        let opt_type = self.expand_union_fn_opt();
        let enum_type = self.expand_union_fn_enum();
        quote_spanned!(span=>
            #enum_type
            const _: () = {
                #opt_type
                #args_type
                #reflect
                #delegate_type
                #impls_type
            };
        )
    }

    /// Exapnds the code to implement the base `UnionFn` trait.
    fn expand_reflection(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let ident_opt = self.ident_opt();
        let ident_args = self.ident_args();
        let output = self.output_type();
        quote_spanned!(trait_span=>
            impl ::union_fn::UnionFn for #ident_opt {
                type Output = #output;
                type Args = #ident_args;
            }

            impl ::union_fn::UnionFn for #trait_ident {
                type Output = #output;
                type Args = #ident_args;
            }
        )
    }

    /// Expand hidden delegators from `UnionFnArgs` to actual function parameters and implementations.
    fn expand_union_fn_impls(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let impls_docs = format!("Implements all methods of the [`{trait_ident}`] type.");
        let ident_impls = self.ident_impls();
        let impls = self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_attrs = method.attrs();
            let impl_block = method.impl_block();
            let ctx_param = method
                .context(&self.state)
                .map(|ctx| {
                    quote_spanned!(
                        method_span=> #ctx: &mut <#trait_ident as ::union_fn::CallWithContext>::Context,
                    )
                });
            let params = method.inputs(&self.state);
            quote_spanned!(method_span=>
                #( #method_attrs )*
                fn #method_ident( #ctx_param #( #params ),* ) -> <#trait_ident as ::union_fn::UnionFn>::Output #impl_block
            )
        });
        quote_spanned!(trait_span=>
            #[doc = #impls_docs]
            pub enum #ident_impls {}

            impl #ident_impls {
                #( #impls )*
            }
        )
    }

    /// Expand hidden delegators from `UnionFnArgs` to actual function parameters and implementations.
    fn expand_union_fn_delegate(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let delegate_docs = format!("Decodes and delegates packed arguments to the implementation of [`{trait_ident}`] methods.");
        let ident_delegate = self.ident_delegate();
        let delegates = self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_attrs = method.attrs();
            let ctx_ident = method
                .context(&self.state)
                .map(|ctx| quote_spanned!(method_span=> #ctx,));
            let ctx_param = method
                .context(&self.state)
                .map(|ctx| {
                    quote_spanned!(
                        method_span=> #ctx: &mut <#trait_ident as ::union_fn::CallWithContext>::Context,
                    )
                });
            let bindings = method.input_bindings(&self.state);
            let tuple_bindings = make_tuple_type(method_span, &bindings);
            quote_spanned!(method_span=>
                #( #method_attrs )*
                fn #method_ident( #ctx_param args: &<#trait_ident as ::union_fn::UnionFn>::Args )
                    -> <#trait_ident as ::union_fn::UnionFn>::Output
                {
                    let #tuple_bindings = unsafe { args.#method_ident };
                    <#trait_ident as ::union_fn::IntoOpt>::Impls::#method_ident( #ctx_ident #( #bindings ),* )
                }
            )
        });
        quote_spanned!(trait_span=>
            #[doc = #delegate_docs]
            pub enum #ident_delegate {}

            impl #ident_delegate {
                #( #delegates )*
            }
        )
    }

    /// Expand the `#[union_fn]` type.
    fn expand_union_fn_opt(&self) -> TokenStream2 {
        let span = self.span();
        let trait_ident = self.ident();
        let ident_opt = self.ident_opt();
        let ident_impls = self.ident_impls();
        let ident_delegate = self.ident_delegate();
        let opt_docs = format!("Call optimized structure of the [`{trait_ident}`] type.");
        let call_impl = self.expand_call_impl();
        let constructors = self.expand_constructors();
        let conversions = self.expand_union_fn_opt_into_opt_arms();
        let ctx = self.state.get_context().map(|_| {
            quote_spanned!(span=>
                ctx: &mut <#trait_ident as ::union_fn::CallWithContext>::Context,
            )
        });
        quote_spanned!(span=>
            #[doc = #opt_docs]
            #[derive(::core::marker::Copy, ::core::clone::Clone)]
            pub struct #ident_opt {
                handler: fn(#ctx &<#trait_ident as ::union_fn::UnionFn>::Args) -> <#trait_ident as ::union_fn::UnionFn>::Output,
                args: <#trait_ident as ::union_fn::UnionFn>::Args,
            }

            impl ::union_fn::IntoOpt for #trait_ident {
                type Opt = #ident_opt;
                type Delegator = #ident_delegate;
                type Impls = #ident_impls;

                fn into_opt(self) -> Self::Opt {
                    match self {
                        #( #conversions )*
                    }
                }
            }

            #call_impl
            #constructors
        )
    }

    /// Expands the arms of the conversion to the call optimized type of the user facing `#[union_fn]` enum type.
    fn expand_union_fn_opt_into_opt_arms(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(move |method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let variant_ident = method_ident.to_upper_camel_case();
            let fields = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                Self::#variant_ident {
                    #( #fields ),*
                } => {
                    <Self as ::union_fn::IntoOpt>::Opt::#method_ident( #( #fields ),* )
                }
            )
        })
    }

    /// Expand the user facing `#[union_fn]` enum type.
    fn expand_union_fn_enum(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let attrs = self.attrs();
        let variants = self.expand_union_fn_enum_variants();
        let constructors = self.expand_union_fn_enum_constructors();
        let call_impl = self.expand_union_fn_enum_call_impl();
        quote_spanned!(trait_span=>
            #( #attrs )*
            #[derive(::core::marker::Copy, ::core::clone::Clone)]
            pub enum #trait_ident {
                #( #variants ),*
            }

            impl #trait_ident {
                #( #constructors )*
            }

            #call_impl
        )
    }

    /// Expands the enum variants of the user facing `#[union_fn]` enum type.
    fn expand_union_fn_enum_variants(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_docs = method.doc_attrs();
            let variant_ident = method_ident.to_upper_camel_case();
            let variant_fields = method.ident_inputs(&self.state);
            quote_spanned!(method_span=>
                #( #method_docs )*
                #variant_ident {
                    #( #variant_fields ),*
                }
            )
        })
    }

    /// Expands the enum constructors of the user facing `#[union_fn]` enum type.
    fn expand_union_fn_enum_constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_attrs = method.attrs();
            let variant_ident = method_ident.to_upper_camel_case();
            let params = method.ident_inputs(&self.state);
            let fields = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                #( #method_attrs )*
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self::#variant_ident {
                        #( #fields ),*
                    }
                }
            )
        })
    }

    /// Expands the trait impl of either `union_fn::Call` or `union_fn::CallWithContext`.
    fn expand_union_fn_enum_call_impl(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let match_arms = self.expand_union_fn_enum_call_impl_arms();
        match self.state.get_context() {
            Some(context) => {
                quote_spanned!(trait_span=>
                    impl ::union_fn::CallWithContext for #trait_ident {
                        type Context = #context;

                        fn call(self, ctx: &mut Self::Context) -> <#trait_ident as ::union_fn::UnionFn>::Output {
                            match self {
                                #( #match_arms )*
                            }
                        }
                    }
                )
            }
            None => {
                quote_spanned!(trait_span=>
                    impl ::union_fn::Call for #trait_ident {
                        fn call(self) -> <#trait_ident as ::union_fn::UnionFn>::Output {
                            match self {
                                #( #match_arms )*
                            }
                        }
                    }
                )
            }
        }
    }

    /// Expands the match arms of either the `union_fn::Call` or `union_fn::CallWithContext` impl.
    fn expand_union_fn_enum_call_impl_arms(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let ctx_param = self
            .state
            .get_context()
            .map(|ctx| quote_spanned!(ctx.span()=> ctx,));
        self.methods().map(move |method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let variant_ident = method_ident.to_upper_camel_case();
            let bindings = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                Self::#variant_ident { #( #bindings ),* } => {
                    <Self as ::union_fn::IntoOpt>::Impls::#method_ident(
                        #ctx_param #( #bindings ),*
                    )
                }
            )
        })
    }

    /// Expand the `#[union_fn]` constructors.
    fn expand_constructors(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let ident_opt = self.ident_opt();
        let constructors = self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_attrs = method.attrs();
            let params = method.ident_inputs(&self.state);
            let param_bindings = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                #( #method_attrs )*
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self {
                        handler: <#trait_ident as ::union_fn::IntoOpt>::Delegator::#method_ident,
                        args: <#trait_ident as ::union_fn::UnionFn>::Args::#method_ident( #( #param_bindings ),* ),
                    }
                }
            )
        });
        quote_spanned!(trait_span=>
            impl #ident_opt {
                #( #constructors )*
            }
        )
    }

    /// Expands the trait impl of either `union_fn::Call` or `union_fn::CallWithContext`.
    fn expand_call_impl(&self) -> TokenStream2 {
        let span = self.span();
        let ident = self.ident();
        let ident_opt = self.ident_opt();
        match self.state.get_context() {
            Some(context) => {
                quote_spanned!(span=>
                    impl ::union_fn::CallWithContext for #ident_opt {
                        type Context = #context;

                        #[inline]
                        fn call(self, ctx: &mut Self::Context) -> <#ident as ::union_fn::UnionFn>::Output {
                            (self.handler)(ctx, &self.args)
                        }
                    }
                )
            }
            None => {
                quote_spanned!(span=>
                    impl ::union_fn::Call for #ident_opt {
                        #[inline]
                        fn call(self) -> <#ident as ::union_fn::UnionFn>::Output {
                            (self.handler)(&self.args)
                        }
                    }
                )
            }
        }
    }

    /// Expands the `#[union_fn]` union arguments type and impls.
    fn expand_union_fn_args(&self) -> TokenStream2 {
        let trait_span = self.span();
        let trait_ident = self.ident();
        let args_docs =
            format!("Efficiently packed method arguments for the [`{trait_ident}`] type.");
        let ident_args = self.ident_args();
        let variants = self.expand_union_args_variants();
        let constructors = self.expand_union_args_constructors();
        quote_spanned!(trait_span =>
            #[doc = #args_docs]
            #[derive(core::marker::Copy, core::clone::Clone)]
            pub union #ident_args {
                #( #variants ),*
            }

            impl #ident_args {
                #( #constructors )*
            }
        )
    }

    /// Expands the `#[union_fn]` union variants.
    fn expand_union_args_variants(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_docs = method.doc_attrs();
            let params = method.input_types(&self.state);
            let tuple_params = make_tuple_type(method_span, params);
            quote_spanned!(method_span =>
                #( #method_docs )*
                #method_ident: #tuple_params
            )
        })
    }

    /// Expands the `#[union_fn]` union variant constructors.
    fn expand_union_args_constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.methods().map(|method| {
            let method_span = method.span();
            let method_ident = method.ident();
            let method_attrs = method.attrs();
            let params = method.ident_inputs(&self.state);
            let param_bindings = method.input_bindings(&self.state);
            let tuple_bindings = make_tuple_type(method_span, param_bindings);
            quote_spanned!(method_span=>
                #( #method_attrs )*
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self { #method_ident: #tuple_bindings }
                }
            )
        })
    }
}
