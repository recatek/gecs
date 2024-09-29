use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::HasCfgPredicates;

pub fn generate_cfg_checks<S: HasCfgPredicates>(source: &S, raw: TokenStream) -> TokenStream {
    let prefix = S::macro_name();
    let predicates = source.collect_all_cfg_predicates();
    let mut macros = Vec::<TokenStream>::with_capacity(predicates.len());

    let start = format_ident!("__impl_ecs_{}_cfg_0", prefix);
    let finish = format_ident!("__impl_ecs_{}", prefix);

    if predicates.is_empty() {
        return quote!(::gecs::__internal::#finish!((), { #raw }););
    }

    for (idx, predicate) in predicates.iter().enumerate() {
        let this = format_ident!("__impl_ecs_{}_cfg_{}", prefix, idx);
        let next = format_ident!("__impl_ecs_{}_cfg_{}", prefix, idx + 1);

        let next = if (idx + 1) == predicates.len() {
            quote!(::gecs::__internal::#finish)
        } else {
            quote!(__ecs_cfg_macros::#next)
        };

        macros.push(quote!(
            #[cfg(#predicate)]
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* true), $($args)*);
                }
            }

            #[cfg(not(#predicate))]
            #[doc(hidden)]
            #[macro_export]
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
