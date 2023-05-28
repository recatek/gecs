use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::data::{DataArchetype, DataWorld};
use crate::parse::{ParseQueryIter, ParseQueryParam, ParseQueryParamType};

// NOTE: We should avoid using panics to express errors in queries when generating.
// Doing so will attribute the error to the ecs_world! declaration (due to the redirect
// macro) rather than to the query macro itself. Always use an Err result where possible.

pub fn generate_query_iter_mut(query: &ParseQueryIter) -> syn::Result<TokenStream> {
    let world = &query.world;
    let body = &query.body;

    let world_data = DataWorld::from_base64(&query.world_data);
    let mut queries = Vec::<TokenStream>::new();

    let param_ident = query.params.iter().map(to_name).collect::<Vec<_>>();
    let param_mut = query.params.iter().map(to_mut).collect::<Vec<_>>();
    let param_type = query.params.iter().map(to_type).collect::<Vec<_>>();
    let param_slice = query.params.iter().map(to_slice).collect::<Vec<_>>();
    let param_into = query.params.iter().map(to_into).collect::<Vec<_>>();

    for archetype in world_data.archetypes {
        if archetype_matches(&archetype, query) {
            let archetype_type = format_ident!("{}", archetype.name);

            queries.push(quote_spanned!(Span::mixed_site() =>
                {
                    // The closure needs to be made per-archetype because of AnyOf types
                    let mut closure = |#(#param_ident: &#param_mut #param_type),*| #body;
                    let archetype = #world.get_mut_archetype::<#archetype_type>();
                    let len = archetype.len();
                    #(let #param_mut #param_slice = slices.get_mut_slices().#param_slice;)*
                    for idx in 0..len {
                        closure(#(&#param_mut #param_slice[idx] #param_into),*);
                    }
                }
            ));
        }
    }
    Ok(quote!(#(#queries)*))
}

pub fn generate_query_iter_borrow(query: &ParseQueryIter) -> syn::Result<TokenStream> {
    let world = &query.world;
    let body = &query.body;

    let world_data = DataWorld::from_base64(&query.world_data);
    let mut queries = Vec::<TokenStream>::new();

    let param_ident = query.params.iter().map(to_name).collect::<Vec<_>>();
    let param_mut = query.params.iter().map(to_mut).collect::<Vec<_>>();
    let param_type = query.params.iter().map(to_type).collect::<Vec<_>>();
    let param_slice = query.params.iter().map(to_slice).collect::<Vec<_>>();
    let param_borrow = query.params.iter().map(to_borrow).collect::<Vec<_>>();
    let param_into = query.params.iter().map(to_into).collect::<Vec<_>>();

    for archetype in world_data.archetypes {
        if archetype_matches(&archetype, query) {
            let archetype_type = format_ident!("{}", archetype.name);

            queries.push(quote_spanned!(Span::mixed_site() =>
                {
                    // The closure needs to be made per-archetype because of AnyOf types
                    let mut closure = |#(#param_ident: &#param_mut #param_type),*| #body;
                    let archetype = #world.get_archetype::<#archetype_type>();
                    let len = archetype.len();
                    #(let #param_mut #param_slice = archetype.#param_borrow;)*
                    for idx in 0..len {
                        closure(#(&#param_mut #param_slice[idx] #param_into),*);
                    }
                }
            ));
        }
    }
    Ok(quote!(#(#queries)*))
}

fn archetype_matches(archetype: &DataArchetype, query: &ParseQueryIter) -> bool {
    for param in &query.params {
        let name = match &param.ty {
            ParseQueryParamType::Component(ident) => ident.to_string(),
            ParseQueryParamType::MutComponent(ident) => ident.to_string(),
            ParseQueryParamType::EntityAny => continue,
            ParseQueryParamType::Entity(ident) => {
                // If we want a specific archetype, then it must be this one
                if ident.to_string() != archetype.name {
                    return false;
                }
                continue;
            }
        };

        let mut contains = false;
        for component in &archetype.components {
            if component.name == name {
                contains = true;
                break;
            }
        }

        if contains == false {
            return false;
        }
    }

    true
}

fn to_name(param: &ParseQueryParam) -> TokenStream {
    let name = &param.name;
    quote!(#name)
}

fn to_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseQueryParamType::MutComponent(_) => quote!(mut),
        _ => quote!(),
    }
}

fn to_into(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseQueryParamType::EntityAny => quote!(.into()),
        _ => quote!(),
    }
}

fn to_type(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseQueryParamType::Component(ident) | ParseQueryParamType::MutComponent(ident) => {
            quote!(#ident)
        }
        ParseQueryParamType::Entity(ident) => quote!(Entity<#ident>),
        ParseQueryParamType::EntityAny => quote!(EntityAny),
    }
}

fn to_slice(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseQueryParamType::Component(ident) | ParseQueryParamType::MutComponent(ident) => {
            let ident = Ident::new(&to_snake(&ident.to_string()), ident.span());
            quote!(#ident)
        }
        ParseQueryParamType::Entity(_) => quote!(entities),
        ParseQueryParamType::EntityAny => quote!(entities),
    }
}

fn to_borrow(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseQueryParamType::Component(ident) => {
            quote!(borrow_slice::<#ident>())
        }
        ParseQueryParamType::MutComponent(ident) => {
            quote!(borrow_mut_slice::<#ident>())
        }
        ParseQueryParamType::Entity(_) | ParseQueryParamType::EntityAny => {
            quote!(get_slice_entities())
        }
    }
}

fn to_snake(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
}
