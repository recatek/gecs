use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::parse::ParseEcsComponentId;

#[allow(non_snake_case)]
pub fn generate_ecs_component_id(util: ParseEcsComponentId) -> TokenStream {
    let get_id_Component = format_ident!("__get_id_{}", util.component);
    let archetype = match util.archetype {
        Some(archetype) => archetype.into_token_stream(),
        None => quote!(MatchedArchetype),
    };

    quote!(#archetype::#get_id_Component())
}
