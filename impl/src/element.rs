use crate::attr;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::iter::FromIterator;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse_quote, Abi, Attribute, BareFnArg, BoundLifetimes, GenericParam,
    Generics, Ident, Path, ReturnType, Token, Type, TypeBareFn, Visibility, WhereClause,
    Item, ItemFn, Lifetime,
};

pub struct Element {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr: TokenStream,
    orig_item: Option<TokenStream>,
    start_span: Span,
    end_span: Span,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let item = input.cursor();
        let vis: Visibility = input.parse()?;
        let static_token: Option<Token![static]> = input.parse()?;
        if static_token.is_some() {
            return Err(Error::new_spanned(
                static_token.unwrap(),
                "static is not supported by distributed_fn_slice, only fn items are supported",
            ));
            let mut_token: Option<Token![mut]> = input.parse()?;
            if let Some(mut_token) = mut_token {
                return Err(Error::new_spanned(
                    mut_token,
                    "static mut is not supported by distributed_fn_slice",
                ));
            }
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let start_span = input.span();
            let ty: Type = input.parse()?;
            let end_span = quote!(#ty).into_iter().last().unwrap().span();
            input.parse::<Token![=]>()?;
            let mut expr_semi = Vec::from_iter(input.parse::<TokenStream>()?);
            if let Some(tail) = expr_semi.pop() {
                syn::parse2::<Token![;]>(TokenStream::from(tail))?;
            }
            let expr = TokenStream::from_iter(expr_semi);
            Ok(Element {
                attrs,
                vis,
                ident,
                ty,
                expr,
                orig_item: None,
                start_span,
                end_span,
            })
        } else {
            let constness: Option<Token![const]> = input.parse()?;
            let asyncness: Option<Token![async]> = input.parse()?;
            let unsafety: Option<Token![unsafe]> = input.parse()?;
            let abi: Option<Abi> = input.parse()?;
            let fn_token: Token![fn] = input.parse().map_err(|_| {
                Error::new_spanned(
                    item.token_stream(),
                    "distributed element must be a function item",
                )
            })?;
            let ident: Ident = input.parse()?;
            let generics: Generics = input.parse()?;

            let content;
            let paren_token = parenthesized!(content in input);
            let mut inputs = Punctuated::new();
            while !content.is_empty() {
                content.parse::<Option<Token![mut]>>()?;
                let ident = if let Some(wild) = content.parse::<Option<Token![_]>>()? {
                    Ident::from(wild)
                } else {
                    content.parse()?
                };
                let colon_token: Token![:] = content.parse()?;
                let ty: Type = content.parse()?;
                inputs.push_value(BareFnArg {
                    attrs: Vec::new(),
                    name: Some((ident, colon_token)),
                    ty,
                });
                if !content.is_empty() {
                    let comma: Token![,] = content.parse()?;
                    inputs.push_punct(comma);
                }
            }

            let output: ReturnType = input.parse()?;
            let where_clause: Option<WhereClause> = input.parse()?;

            let content;
            braced!(content in input);
            content.parse::<TokenStream>()?;

            if let Some(constness) = constness {
                return Err(Error::new_spanned(
                    constness,
                    "const fn distributed slice element is not supported",
                ));
            }

            if let Some(asyncness) = asyncness {
                return Err(Error::new_spanned(
                    asyncness,
                    "async fn distributed slice element is not supported",
                ));
            }

            let lifetimes = if generics.params.is_empty() {
                None
            } else {
                let mut bound = BoundLifetimes {
                    for_token: Token![for](generics.lt_token.unwrap().span),
                    lt_token: generics.lt_token.unwrap(),
                    lifetimes: Punctuated::new(),
                    gt_token: generics.gt_token.unwrap(),
                };
                for param in generics.params.into_pairs() {
                    let (param, punct) = param.into_tuple();
                    if let GenericParam::Lifetime(_) = param {
                        bound.lifetimes.push_value(param);
                        if let Some(punct) = punct {
                            bound.lifetimes.push_punct(punct);
                        }
                    } else {
                        return Err(Error::new_spanned(
                            param,
                            "cannot have generic parameters on distributed slice element",
                        ));
                    }
                }
                Some(bound)
            };

            if let Some(where_clause) = where_clause {
                return Err(Error::new_spanned(
                    where_clause,
                    "where-clause is not allowed on distributed slice elements",
                ));
            }

            let start_span = item.span();
            let end_span = quote!(#output)
                .into_iter()
                .last()
                .as_ref()
                .map_or(paren_token.span.close(), TokenTree::span);
            let mut original_attrs = attrs;
            let linkme_path = attr::linkme_path(&mut original_attrs)?;

            let attrs = vec![
                parse_quote! {
                    #[allow(non_upper_case_globals)]
                },
                parse_quote! {
                    #[linkme(crate = #linkme_path)]
                },
            ];
            let vis = Visibility::Inherited;
            let expr = parse_quote!(#ident);
            let ty = Type::BareFn(TypeBareFn {
                lifetimes,
                unsafety,
                abi,
                fn_token,
                paren_token,
                inputs,
                variadic: None,
                output,
            });
            let ident = format_ident!("_LINKME_ELEMENT_{}", ident);
            let item = item.token_stream();
            let orig_item = Some(quote!(
                #(#original_attrs)*
                #item
            ));

            Ok(Element {
                attrs,
                vis,
                ident,
                ty,
                expr,
                orig_item,
                start_span,
                end_span,
            })
        }
    }
}

pub struct Element2 {
    item: ItemFn,
    ty: Type,
    attrs: Vec<Attribute>,
    start_span: Span,
    end_span: Span,
}

impl Parse for Element2 {
    fn parse(input: ParseStream) -> Result<Self> {
        let start_span = input.cursor().span();
        let item: ItemFn = syn::parse2(input.cursor().token_stream())?;

        let attrs = input.call(Attribute::parse_outer)?;
        let _vis: Visibility = input.parse()?;
        let constness: Option<Token![const]> = input.parse()?;
        let asyncness: Option<Token![async]> = input.parse()?;
        let unsafety: Option<Token![unsafe]> = input.parse()?;
        let _abi: Option<Abi> = input.parse()?;
        let fn_token: Token![fn] = input.parse().map_err(|_| {
            Error::new_spanned(
                item.to_token_stream(),
                "distributed element must be a function item",
            )
        })?;
        let _ident: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let mut inputs = Punctuated::new();
        while !content.is_empty() {
            content.parse::<Option<Token![mut]>>()?;
            let ident = if let Some(wild) = content.parse::<Option<Token![_]>>()? {
                Ident::from(wild)
            } else {
                content.parse()?
            };
            let colon_token: Token![:] = content.parse()?;
            let ty: Type = content.parse()?;
            inputs.push_value(BareFnArg {
                attrs: Vec::new(),
                name: Some((ident, colon_token)),
                ty,
            });
            if !content.is_empty() {
                let comma: Token![,] = content.parse()?;
                inputs.push_punct(comma);
            }
        }

        let output: ReturnType = input.parse()?;
        let _where_clause: Option<WhereClause> = input.parse()?;

        let content;
        braced!(content in input);
        content.parse::<TokenStream>()?;

        if let Some(constness) = constness {
            return Err(Error::new_spanned(
                constness,
                "const fn distributed slice element is not supported",
            ));
        }

        if let Some(asyncness) = asyncness {
            return Err(Error::new_spanned(
                asyncness,
                "async fn distributed slice element is not supported",
            ));
        }

        let lifetimes = if generics.params.is_empty() {
            None
        } else {
            let mut bound = BoundLifetimes {
                for_token: Token![for](generics.lt_token.unwrap().span),
                lt_token: generics.lt_token.unwrap(),
                lifetimes: Punctuated::new(),
                gt_token: generics.gt_token.unwrap(),
            };
            for param in generics.params.into_pairs() {
                let (param, punct) = param.into_tuple();
                if let GenericParam::Lifetime(_) = param {
                    bound.lifetimes.push_value(param);
                    if let Some(punct) = punct {
                        bound.lifetimes.push_punct(punct);
                    }
                }
            }
            Some(bound)
        };

        let end_span = quote!(#item)
            .into_iter()
            .last()
            .as_ref()
            .map_or(paren_token.span.close(), TokenTree::span);
        let mut original_attrs = attrs;
        let linkme_path = attr::linkme_path(&mut original_attrs)?;

        let attrs = vec![
            parse_quote! {
                #[allow(non_upper_case_globals)]
            },
            parse_quote! {
                #[linkme(crate = #linkme_path)]
            },
        ];

        let ty = Type::BareFn(TypeBareFn {
            lifetimes,
            unsafety,
            abi: Some(syn::parse2(quote! {extern "C"}).unwrap()),
            fn_token,
            paren_token,
            inputs,
            variadic: None,
            output,
        });

        Ok(Element2 {
            attrs,
            ty,
            item,
            start_span,
            end_span,
        })
    }
}

pub fn expand2(path: Path, pos: Option<usize>, input: Element2) -> TokenStream {
    let name = input.item.sig.ident.clone();
    let lifetime_params = input.item.sig.generics.params
        .iter().flat_map(|p| match p {
            GenericParam::Lifetime(lp) => Some(lp.lifetime.clone()),
            _ => None
        })
        .collect::<Vec<_>>();
    let type_and_const_params = input.item.sig.generics.params
        .iter().flat_map(|p| match p {
            GenericParam::Type(tp) => Some(tp.ident.clone()),
            GenericParam::Const(cp) => Some(cp.ident.clone()),
            _ => None
        })
        .collect::<Vec<_>>();
    let mut receiver = Vec::new();
    let arguments = input.item.sig.inputs
        .iter().flat_map(|arg| {
            match arg {
                syn::FnArg::Receiver(r) => {
                    receiver.push(r.self_token.clone());
                    None
                },
                syn::FnArg::Typed(pt) => {
                    Some((*pt.pat).clone())
                },
            }
        })
        .collect::<Vec<_>>();

    let ty = input.ty;
    let new = quote_spanned!(input.start_span=> __new);
    let uninit = quote_spanned!(input.end_span=> #new());
    let sort_key = pos.into_iter().map(|pos| format!("{:04}", pos));
    let linkme_path = match attr::linkme_path(&mut input.attrs.clone()) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let mut inner_impl = input.item.clone();
    let inner_impl_name = format_ident!("{}_inner_impl", name);
    inner_impl.sig.ident = inner_impl_name.clone();
    inner_impl.vis = Visibility::Inherited;
    let mut outer_impl = input.item.clone();
    let outer_impl_name = format_ident!("{}_outer_impl", name);
    outer_impl.sig.ident = outer_impl_name.clone();
    outer_impl.vis = Visibility::Inherited;
    outer_impl.sig.abi = Some(syn::parse2(quote! {extern "C"}).unwrap());
    outer_impl.block = Box::new(syn::parse2(quote! {{
        fn volatile<T>(x: T) { unsafe { let res = std::mem::read_volatile(&x); std::mem::forget(x); res } }
        volatile(
            #inner_impl_name::<#(#lifetime_params,)*#(#type_and_const_params,)*>(
                #(volatile(#receiver),)*#(volatile(#arguments),)*
            )
        )
    }}).unwrap());
    let mut rewritten_item = input.item.clone();
    rewritten_item.block = Box::new(syn::parse2(quote! {{
        #inner_impl #[inline(never)] #outer_impl
        unsafe fn __typecheck(_: #linkme_path::__private::Void) {
            let #new = #linkme_path::__private::value::<#ty>;
            #linkme_path::DistributedFnSlice::private_typecheck(#path, #uninit)
        }
        #outer_impl_name::<#(#lifetime_params,)*#(#type_and_const_params,)*>(
            #(#receiver,)*#(#arguments,)*
        )
    }}).unwrap());
    rewritten_item.sig.abi = Some(syn::parse2(quote! {extern "C"}).unwrap());
    quote! {
        #path ! {
            #(
                #![linkme_macro = #path]
                #![linkme_sort_key = #sort_key]
            )*
            #[inline(never)]
            #rewritten_item
        }
    }
}

pub fn expand(path: Path, pos: impl Into<Option<usize>>, input: Element) -> TokenStream {
    let pos = pos.into();
    do_expand(path, pos, input)
}

fn do_expand(path: Path, pos: Option<usize>, input: Element) -> TokenStream {
    let mut attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let ty = input.ty;
    let expr = input.expr;
    let orig_item = input.orig_item;

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let sort_key = pos.into_iter().map(|pos| format!("{:04}", pos));

    let new = quote_spanned!(input.start_span=> __new);
    let uninit = quote_spanned!(input.end_span=> #new());

    quote! {
        #path ! {
            #(
                #![linkme_macro = #path]
                #![linkme_sort_key = #sort_key]
            )*
            #(#attrs)*
            #vis static #ident : #ty = {
                unsafe fn __typecheck(_: #linkme_path::__private::Void) {
                    let #new = #linkme_path::__private::value::<#ty>;
                    #linkme_path::DistributedSlice::private_typecheck(&#path, #uninit)
                }

                #expr
            };
        }

        #orig_item
    }
}
