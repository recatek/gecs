use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::HasCfgPredicates;

pub fn generate_cfg_checks_outer<S: HasCfgPredicates>(
    name: &str,
    source: &S,
    raw: TokenStream,
) -> TokenStream {
    let predicates = source.collect_all_cfg_predicates();
    let mut macros = Vec::<TokenStream>::with_capacity(predicates.len());

    let start = format_ident!("__cfg_ecs_{}_0", name);
    let finish = format_ident!("__impl_ecs_{}", name);

    if predicates.is_empty() {
        return quote!(::gecs::__internal::#finish!((), { #raw }););
    }

    for (idx, predicate) in predicates.iter().enumerate() {
        let this = format_ident!("__cfg_ecs_{}_{}", name, idx);
        let next = format_ident!("__cfg_ecs_{}_{}", name, idx + 1);

        let next = if (idx + 1) == predicates.len() {
            quote!(::gecs::__internal::#finish)
        } else {
            quote!(__ecs_cfg_macros::#next)
        };

        macros.push(quote!(
            #[cfg(#predicate)]
            #[doc(hidden)]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* true), $($args)*);
                }
            }

            #[cfg(not(#predicate))]
            #[doc(hidden)]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* false), $($args)*);
                }
            }

            pub(super) use #this;
        ));
    }

    quote!(
        mod __ecs_cfg_macros { #(#macros)* }
        __ecs_cfg_macros::#start!((), { #raw });
    )
}

pub fn generate_cfg_checks_inner<S: HasCfgPredicates>(
    name: &str,
    source: &S,
    raw: TokenStream,
) -> TokenStream {
    let predicates = source.collect_all_cfg_predicates();
    let mut macros = Vec::<TokenStream>::with_capacity(predicates.len());

    let start = format_ident!("__cfg_ecs_{}_0", name);
    let finish = format_ident!("__impl_ecs_{}", name);

    if predicates.is_empty() {
        return quote!(
            {
                ::gecs::__internal::#finish!((), { #raw })
            }
        );
    }

    for (idx, predicate) in predicates.iter().enumerate() {
        let this = format_ident!("__cfg_ecs_{}_{}", name, idx);
        let next = format_ident!("__cfg_ecs_{}_{}", name, idx + 1);

        let next = if (idx + 1) == predicates.len() {
            quote!(::gecs::__internal::#finish)
        } else {
            quote!(#next)
        };

        macros.push(quote!(
            #[cfg(#predicate)]
            #[doc(hidden)]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* true), $($args)*)
                }
            }

            #[cfg(not(#predicate))]
            #[doc(hidden)]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* false), $($args)*)
                }
            }
        ));
    }

    quote!(
        {
            // Generate the macros as part of the expression inline
            #(#macros)*

            #start!((), { #raw })
        }
    )
}
