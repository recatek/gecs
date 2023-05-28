use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::data::{DataArchetype, DataWorld};
use crate::parse::{ParseParamType, ParseQueryFind, ParseQueryIter, ParseQueryParam};

// NOTE: We should avoid using panics to express errors in queries when generating.
// Doing so will attribute the error to the ecs_world! declaration (due to the redirect
// macro) rather than to the query macro itself. Always use an Err result where possible.

pub enum FetchMode {
    Mut,
    Borrow,
}

#[allow(non_snake_case)]
pub fn generate_query_find(mode: FetchMode, query: &ParseQueryFind) -> syn::Result<TokenStream> {
    let world_data = DataWorld::from_base64(&query.world_data);

    // Types and traits
    let EntityWorld = format_ident!("Entity{}", world_data.name);
    let Type = query.params.iter().map(to_type).collect::<Vec<_>>();

    // Variables and fields
    let world = &query.world;
    let entity = &query.entity;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();
    let slice = query.params.iter().map(to_slice).collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let maybe_into = query.params.iter().map(to_maybe_into).collect::<Vec<_>>();

    // Inner generation
    let slice_access = generate_slice_access(mode, &query.params);

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        if archetype_matches(&archetype, &query.params) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);

            queries.push(quote_spanned!(Span::mixed_site() =>
                #EntityWorld::#Archetype(entity) => {
                    // The closure needs to be made per-archetype because of AnyOf types
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;
                    let archetype = #world.get_archetype::<#Archetype>();
                    if let Some(idx) = archetype.resolve(entity) {
                        #slice_access // This depends on fetch mode
                        closure(#(&#maybe_mut #slice[idx] #maybe_into),*);
                        true
                    } else {
                        false
                    }
                }
            ));
        }
    }

    Ok(quote!(
        {
            match #entity.into() {
                #(#queries)*
                _ => false,
            }
        }
    ))
}

#[allow(non_snake_case)]
pub fn generate_query_iter(mode: FetchMode, query: &ParseQueryIter) -> syn::Result<TokenStream> {
    let world_data = DataWorld::from_base64(&query.world_data);

    // Types and traits
    let Type = query.params.iter().map(to_type).collect::<Vec<_>>();

    // Variables and fields
    let world = &query.world;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();
    let slice = query.params.iter().map(to_slice).collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let maybe_into = query.params.iter().map(to_maybe_into).collect::<Vec<_>>();

    // Inner generation
    let slice_access = generate_slice_access(mode, &query.params);

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        if archetype_matches(&archetype, &query.params) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);

            queries.push(quote_spanned!(Span::mixed_site() =>
                {
                    // The closure needs to be made per-archetype because of AnyOf types
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;
                    let archetype = #world.get_archetype::<#Archetype>();
                    let len = archetype.len();
                    #slice_access // This depends on fetch mode
                    for idx in 0..len {
                        closure(#(&#maybe_mut #slice[idx] #maybe_into),*);
                    }
                }
            ));
        }
    }
    Ok(quote!(#(#queries)*))
}

fn generate_slice_access(mode: FetchMode, params: &[ParseQueryParam]) -> TokenStream {
    let slice = params.iter().map(to_slice).collect::<Vec<_>>();
    let maybe_mut = params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let fn_borrow = params.iter().map(to_borrow).collect::<Vec<_>>();

    match mode {
        FetchMode::Mut => quote_spanned!(Span::mixed_site() =>
            let slices = archetype.get_mut_slices();
            #(let #maybe_mut #slice = slices.#slice;)*
        ),
        FetchMode::Borrow => quote_spanned!(Span::mixed_site() =>
            #(let #maybe_mut #slice = archetype.#fn_borrow;)*
        ),
    }
}

fn archetype_matches(archetype: &DataArchetype, params: &[ParseQueryParam]) -> bool {
    for param in params {
        let name = match &param.ty {
            ParseParamType::Component(ident) => ident.to_string(),
            ParseParamType::EntityAny => continue,
            ParseParamType::Entity(ident) => {
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

fn to_type(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseParamType::Component(ident) => quote!(#ident),
        ParseParamType::Entity(ident) => quote!(Entity<#ident>),
        ParseParamType::EntityAny => quote!(EntityAny),
    }
}

fn to_slice(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseParamType::Component(ident) => to_token_stream(&to_snake_ident(ident)),
        ParseParamType::Entity(_) => quote!(entities),
        ParseParamType::EntityAny => quote!(entities),
    }
}

fn to_borrow(param: &ParseQueryParam) -> TokenStream {
    match (param.is_mut, &param.ty) {
        (false, ParseParamType::Component(ident)) => quote!(borrow_slice::<#ident>()),
        (true, ParseParamType::Component(ident)) => quote!(borrow_mut_slice::<#ident>()),
        (_, ParseParamType::Entity(_)) => quote!(get_slice_entities()),
        (_, ParseParamType::EntityAny) => quote!(get_slice_entities()),
    }
}

fn to_maybe_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.is_mut {
        true => quote!(mut),
        false => quote!(),
    }
}

fn to_maybe_into(param: &ParseQueryParam) -> TokenStream {
    match &param.ty {
        ParseParamType::EntityAny => quote!(.into()),
        _ => quote!(),
    }
}

fn to_snake_ident(ident: &Ident) -> Ident {
    Ident::new(&to_snake_str(&ident.to_string()), ident.span())
}

fn to_snake_str(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
}

fn to_token_stream(ident: &Ident) -> TokenStream {
    quote!(#ident)
}
