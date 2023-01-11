use crate::{utils::make_tuple_type, UnionFn};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote_spanned};
use syn::spanned::Spanned as _;

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
        let output = self.output_type();
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
            let params = method.inputs(&self.state);
            let bindings = method.input_bindings(&self.state);
            let tuple_bindings = make_tuple_type(method_span, &bindings);
            quote_spanned!(method_span=>
                fn #method_ident( #ctx_param args: <#trait_ident as ::union_fn::UnionFn>::Args )
                    -> <#trait_ident as ::union_fn::UnionFn>::Output
                {
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
            let method_attrs = method.attrs();
            let params = method.ident_inputs(&self.state);
            let param_bindings = method.input_bindings(&self.state);
            quote_spanned!(method_span=>
                #( #method_attrs )*
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

    /// Expands the trait impl of either `union_fn::Call` or `union_fn::CallWithContext`.
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

    /// Expands the `#[union_fn]` union arguments type and impls.
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
            let tuple_params = make_tuple_type(method_span, params);
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
            let tuple_bindings = make_tuple_type(method_span, param_bindings);
            quote_spanned!(method_span=>
                pub fn #method_ident( #( #params ),* ) -> Self {
                    Self { #method_ident: #tuple_bindings }
                }
            )
        })
    }
}
