use crate::{attr, linker};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    bracketed, Attribute, Error, GenericArgument, Ident, Lifetime, PathArguments, Token, Type,
    Visibility, TypeSlice,
};

struct Declaration {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let mut_token: Option<Token![mut]> = input.parse()?;
        if let Some(mut_token) = mut_token {
            return Err(Error::new_spanned(
                mut_token,
                "static mut is not supported by distributed_fn_slice",
            ));
        }
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let mut ty: Type = input.parse()?;
        let inner_ty: &mut Type = match &mut ty {
            Type::Slice(TypeSlice { elem, .. }) => &mut *elem,
            _ => return Err(Error::new_spanned(
                ty.to_token_stream(),
                "distributed_fn_slice must be a slice",
            )),
        };
        let mut fn_ty = match inner_ty {
            Type::BareFn(fn_ty) => fn_ty,
            _ => return Err(Error::new_spanned(
                inner_ty.to_token_stream(),
                "distributed_fn_slice can only contain bare function pointers",
            )),
        };
        match &fn_ty.abi {
            None => {
                fn_ty.abi = Some(syn::parse2(quote! {extern "C"}).unwrap());
            }
            Some(abi) => {
                let is_c = abi.name.is_none() || abi.name.as_ref().unwrap().to_token_stream().to_string().trim() == "\"C\"";
                if !is_c {
                    return Err(Error::new_spanned(
                        abi.to_token_stream(),
                        "distributed_fn_slice can only contain function pointers that use C ABI",
                    ))
                }
            }
        }

        input.parse::<Token![=]>()?;

        let content;
        bracketed!(content in input);
        content.parse::<Token![..]>()?;

        input.parse::<Token![;]>()?;

        Ok(Declaration {
            attrs,
            vis,
            ident,
            ty,
        })
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let msg = "distributed_fn_slice is not implemented for this platform";
    let error = Error::new_spanned(&input, msg);
    let unsupported_platform = error.to_compile_error();

    let decl: Declaration = match syn::parse2(input) {
        Ok(decl) => decl,
        Err(err) => return err.to_compile_error(),
    };

    let mut attrs = decl.attrs;
    let vis = decl.vis;
    let ident = decl.ident;
    let mut ty = decl.ty;
    let name = ident.to_string();

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    populate_static_lifetimes(&mut ty);

    let used = if cfg!(feature = "used_linker") {
        quote!(#[used(linker)])
    } else {
        quote!(#[used])
    };

    let linux_section = linker::linux::section(&ident);
    let linux_section_start = linker::linux::section_start(&ident);
    let linux_section_stop = linker::linux::section_stop(&ident);
    let linux_dupcheck = linux_section.replacen("generic_linkme", "generic_linkm2", 1);
    let linux_dupcheck_start = linux_section_start.replacen("generic_linkme", "generic_linkm2", 1);
    let linux_dupcheck_stop = linux_section_stop.replacen("generic_linkme", "generic_linkm2", 1);

    let macho_section = linker::macho::section(&ident);
    let macho_section_start = linker::macho::section_start(&ident);
    let macho_section_stop = linker::macho::section_stop(&ident);
    let macho_dupcheck = macho_section.replacen("generic_linkme", "generic_linkm2", 1);
    let macho_dupcheck_start = macho_section_start.replacen("generic_linkme", "generic_linkm2", 1);
    let macho_dupcheck_stop = macho_section_stop.replacen("generic_linkme", "generic_linkm2", 1);

    let windows_section = linker::windows::section(&ident);
    let windows_section_start = linker::windows::section_start(&ident);
    let windows_section_stop = linker::windows::section_stop(&ident);
    let windows_dupcheck = windows_section.replacen("generic_linkme", "generic_linkm2", 1);
    let windows_dupcheck_start = windows_section_start.replacen("generic_linkme", "generic_linkm2", 1);
    let windows_dupcheck_stop = windows_section_stop.replacen("generic_linkme", "generic_linkm2", 1);

    let illumos_section = linker::illumos::section(&ident);
    let illumos_section_start = linker::illumos::section_start(&ident);
    let illumos_section_stop = linker::illumos::section_stop(&ident);
    let illumos_dupcheck = illumos_section.replacen("generic_linkme", "generic_linkm2", 1);
    let illumos_dupcheck_start = illumos_section_start.replacen("generic_linkme", "generic_linkm2", 1);
    let illumos_dupcheck_stop = illumos_section_stop.replacen("generic_linkme", "generic_linkm2", 1);

    let freebsd_section = linker::freebsd::section(&ident);
    let freebsd_section_start = linker::freebsd::section_start(&ident);
    let freebsd_section_stop = linker::freebsd::section_stop(&ident);
    let freebsd_dupcheck = freebsd_section.replacen("generic_linkme", "generic_linkm2", 1);
    let freebsd_dupcheck_start = freebsd_section_start.replacen("generic_linkme", "generic_linkm2", 1);
    let freebsd_dupcheck_stop = freebsd_section_stop.replacen("generic_linkme", "generic_linkm2", 1);

    let call_site = Span::call_site();
    let link_section_macro_str = format!("_generic_linkme_macro_{}", ident);
    let link_section_macro = Ident::new(&link_section_macro_str, call_site);

    quote! {
        #(#attrs)*
        #vis static #ident: #linkme_path::DistributedFnSlice<#ty> = {
            #[cfg(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "illumos",
                target_os = "freebsd",
            ))]
            extern "Rust" {
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_start)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_start)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_section_start)]
                static LINKME_START: #linkme_path::__private::u8;

                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_stop)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_stop)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_section_stop)]
                static LINKME_STOP: #linkme_path::__private::u8;

                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_dupcheck_start)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_dupcheck_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_dupcheck_start)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_dupcheck_start)]
                static DUPCHECK_START: #linkme_path::__private::usize;

                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_dupcheck_stop)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_dupcheck_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_dupcheck_stop)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_dupcheck_stop)]
                static DUPCHECK_STOP: #linkme_path::__private::usize;
            }

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_start]
            static LINKME_START: [#linkme_path::__private::u8; 0] = [];

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_stop]
            static LINKME_STOP: [#linkme_path::__private::u8; 0] = [];

            #[cfg(target_os = "windows")]
            #[link_section = #windows_dupcheck_start]
            static DUPCHECK_START: () = ();

            #[cfg(target_os = "windows")]
            #[link_section = #windows_dupcheck_stop]
            static DUPCHECK_STOP: () = ();

            #used
            #[cfg(any(target_os = "none", target_os = "linux", target_os = "illumos", target_os = "freebsd"))]
            #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_section)]
            #[cfg_attr(target_os = "illumos", link_section = #illumos_section)]
            #[cfg_attr(target_os = "freebsd", link_section = #freebsd_section)]
            static mut LINKME_PLEASE: [#linkme_path::__private::u8; 0] = [];

            #used
            #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_dupcheck)]
            #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = #macho_dupcheck)]
            #[cfg_attr(target_os = "windows", link_section = #windows_dupcheck)]
            #[cfg_attr(target_os = "illumos", link_section = #illumos_dupcheck)]
            #[cfg_attr(target_os = "freebsd", link_section = #freebsd_dupcheck)]
            static DUPCHECK: #linkme_path::__private::usize = 1;

            #[cfg(not(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "windows",
                target_os = "illumos",
                target_os = "freebsd",
            )))]
            #unsupported_platform

            #linkme_path::__private::assert!(
                #linkme_path::__private::mem::size_of::<<#ty as #linkme_path::__private::Slice>::Element>() > 0,
            );

            unsafe {
                #linkme_path::DistributedFnSlice::private_new(
                    #name,
                    &LINKME_START,
                    &LINKME_STOP,
                    &DUPCHECK_START,
                    &DUPCHECK_STOP,
                )
            }
        };

        #[doc(hidden)]
        #[macro_export]
        macro_rules! #link_section_macro {
            (
                #![linkme_macro = $macro:path]
                #![linkme_sort_key = $key:tt]
                $item:item
            ) => {
                $macro ! {
                    #![linkme_linux_section = concat!(#linux_section, $key)]
                    #![linkme_macho_section = concat!(#macho_section, $key)]
                    #![linkme_windows_section = concat!(#windows_section, $key)]
                    #![linkme_illumos_section = concat!(#illumos_section, $key)]
                    #![linkme_freebsd_section = concat!(#freebsd_section, $key)]
                    $item
                }
            };
            (
                #![linkme_linux_section = $linux_section:expr]
                #![linkme_macho_section = $macho_section:expr]
                #![linkme_windows_section = $windows_section:expr]
                #![linkme_illumos_section = $illumos_section:expr]
                #![linkme_freebsd_section = $freebsd_section:expr]
                $item:item
            ) => {
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = $linux_section)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = $macho_section)]
                #[cfg_attr(target_os = "windows", link_section = $windows_section)]
                #[cfg_attr(target_os = "illumos", link_section = $illumos_section)]
                #[cfg_attr(target_os = "freebsd", link_section = $freebsd_section)]
                $item
            };
            ($item:item) => {
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_section)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = #macho_section)]
                #[cfg_attr(target_os = "windows", link_section = #windows_section)]
                #[cfg_attr(target_os = "illumos", link_section = #illumos_section)]
                #[cfg_attr(target_os = "freebsd", link_section = #freebsd_section)]
                $item
            };
        }

        #[doc(hidden)]
        #vis use #link_section_macro as #ident;
    }
}

fn populate_static_lifetimes(ty: &mut Type) {
    match ty {
        Type::Array(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Group(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Paren(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Path(ty) => {
            if let Some(qself) = &mut ty.qself {
                populate_static_lifetimes(&mut qself.ty);
            }
            for segment in &mut ty.path.segments {
                if let PathArguments::AngleBracketed(segment) = &mut segment.arguments {
                    for arg in &mut segment.args {
                        if let GenericArgument::Type(arg) = arg {
                            populate_static_lifetimes(arg);
                        }
                    }
                }
            }
        }
        Type::Ptr(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Reference(ty) => {
            if ty.lifetime.is_none() {
                ty.lifetime = Some(Lifetime::new("'static", ty.and_token.span));
            }
            populate_static_lifetimes(&mut ty.elem);
        }
        Type::Slice(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Tuple(ty) => ty.elems.iter_mut().for_each(populate_static_lifetimes),
        Type::ImplTrait(_)
        | Type::Infer(_)
        | Type::Macro(_)
        | Type::Never(_)
        | Type::TraitObject(_)
        | Type::BareFn(_)
        | Type::Verbatim(_) => {}
        #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
        _ => unimplemented!("unknown Type"),
    }
}
