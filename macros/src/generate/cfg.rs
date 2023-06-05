use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::ParseEcsWorld;

pub fn generate_cfg_checks(world: &ParseEcsWorld, raw: TokenStream) -> TokenStream {
    let predicates = world.collect_all_cfg_predicates();
    let mut macros = Vec::<TokenStream>::with_capacity(predicates.len());

    if predicates.is_empty() {
        return quote!(::gecs::__internal::__ecs_finalize!((), { #raw }););
    }

    for (idx, predicate) in predicates.iter().enumerate() {
        let this = format_ident!("__ecs_check_cfg_{}", idx);
        let next = format_ident!("__ecs_check_cfg_{}", idx + 1);

        let next = if (idx + 1) == predicates.len() {
            quote!(::gecs::__internal::__ecs_finalize)
        } else {
            quote!(__ecs_cfg_macros::#next)
        };

        macros.push(quote!(
            #[cfg(#predicate)]
            #[macro_export]
            macro_rules! #this {
                (($($bools:expr),*), $($args:tt)*) => {
                    #next!(($($bools,)* true), $($args)*);
                }
            }

            #[cfg(not(#predicate))]
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
        __ecs_cfg_macros::__ecs_check_cfg_0!((), { #raw });
    )
}
