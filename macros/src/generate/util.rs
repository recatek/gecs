use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::parse::ParseEcsComponentId;

#[allow(non_snake_case)]
pub fn generate_ecs_component_id(util: ParseEcsComponentId) -> TokenStream {
    let get_id_component = format_ident!("get_id_{}", to_snake(&util.component.to_string()));
    let archetype = match util.archetype {
        Some(archetype) => archetype.into_token_stream(),
        None => quote!(MatchedArchetype),
    };

    quote!(#archetype::#get_id_component())
}

pub fn to_snake(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
}
