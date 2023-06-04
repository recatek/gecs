use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::data::{DataArchetype, DataWorld};
use crate::parse::{ParseQueryFind, ParseQueryIter, ParseQueryParam, ParseQueryParamType};

// NOTE: We should avoid using panics to express errors in queries when generating.
// Doing so will attribute the error to the ecs_world! declaration (due to the redirect
// macro) rather than to the query macro itself. Always use an Err result where possible.

#[derive(Clone, Copy, Debug)]
pub enum FetchMode {
    Mut,
    Borrow,
}

#[allow(non_snake_case)]
pub fn generate_query_find(mode: FetchMode, query: ParseQueryFind) -> syn::Result<TokenStream> {
    let world_data = DataWorld::from_base64(&query.world_data);
    let bound_params = bind_query_params(&world_data, &query.params)?;

    // NOTE: Beyond this point, query.params is only safe to use for information that
    // does not change depending on the type of the parameter (e.g. mutability). Anything
    // that might change after OneOf binding etc. must use the bound query params in
    // bound_params for the given archetype. Note that it's faster to use query.params
    // where available, since it avoids redundant computation for each archetype.

    // Types and traits
    let EntityWorld = format_ident!("Entity{}", world_data.name);

    // Variables and fields
    let world = &query.world;
    let entity = &query.entity;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();

    // Keywords
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let maybe_into = query.params.iter().map(to_maybe_into).collect::<Vec<_>>();

    // Mode-specific behavior
    let archetype_access = generate_archetype_access(mode);

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        debug_assert!(archetype.build_data.is_none());

        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let Type = bound_params.iter().map(to_type).collect::<Vec<_>>(); // Bind-dependent!

            // Variables and fields
            let slice = bound_params.iter().map(to_slice).collect::<Vec<_>>(); // Bind-dependent!

            // Mode-specific behavior
            let slice_access = generate_slice_access(mode, &bound_params); // Bind-dependent!

            queries.push(quote_spanned!(Span::mixed_site() =>
                #EntityWorld::#Archetype(entity) => {
                    // The closure needs to be made per-archetype because of OneOf types
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;
                    let archetype = #world.#archetype_access::<#Archetype>();
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
pub fn generate_query_iter(mode: FetchMode, query: ParseQueryIter) -> syn::Result<TokenStream> {
    let world_data = DataWorld::from_base64(&query.world_data);
    let bound_params = bind_query_params(&world_data, &query.params)?;

    // NOTE: Beyond this point, query.params is only safe to use for information that
    // does not change depending on the type of the parameter (e.g. mutability). Anything
    // that might change after OneOf binding etc. must use the bound query params in
    // bound_params for the given archetype. Note that it's faster to use query.params
    // where available, since it avoids redundant computation for each archetype.

    // Variables and fields
    let world = &query.world;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let maybe_into = query.params.iter().map(to_maybe_into).collect::<Vec<_>>();

    // Mode-specific behavior
    let archetype_access = generate_archetype_access(mode);

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        debug_assert!(archetype.build_data.is_none());

        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let Type = bound_params.iter().map(to_type).collect::<Vec<_>>(); // Bind-dependent!

            // Variables and fields
            let slice = bound_params.iter().map(to_slice).collect::<Vec<_>>(); // Bind-dependent!

            // Mode-specific behavior
            let slice_access = generate_slice_access(mode, &bound_params); // Bind-dependent!

            queries.push(quote_spanned!(Span::mixed_site() =>
                {
                    // The closure needs to be made per-archetype because of OneOf types
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;
                    let archetype = #world.#archetype_access::<#Archetype>();
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

fn generate_archetype_access(mode: FetchMode) -> TokenStream {
    match mode {
        FetchMode::Borrow => quote!(archetype),
        FetchMode::Mut => quote!(archetype_mut),
    }
}

fn generate_slice_access(mode: FetchMode, params: &[ParseQueryParam]) -> TokenStream {
    let slice = params.iter().map(to_slice).collect::<Vec<_>>();
    let maybe_mut = params.iter().map(to_maybe_mut).collect::<Vec<_>>();
    let fn_borrow = params.iter().map(to_borrow).collect::<Vec<_>>();

    match mode {
        FetchMode::Borrow => quote_spanned!(Span::mixed_site() =>
            #(let #maybe_mut #slice = archetype.#fn_borrow;)*
        ),
        FetchMode::Mut => quote_spanned!(Span::mixed_site() =>
            let slices = archetype.get_all_slices();
            #(let #maybe_mut #slice = slices.#slice;)*
        ),
    }
}

fn to_name(param: &ParseQueryParam) -> TokenStream {
    let name = &param.name;
    quote!(#name)
}

fn to_type(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => quote!(#ident),
        ParseQueryParamType::Entity(ident) => quote!(Entity<#ident>),
        ParseQueryParamType::EntityAny => quote!(EntityAny),
        ParseQueryParamType::OneOf(_) => panic!("must unpack OneOf first"),
    }
}

fn to_slice(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => to_token_stream(&to_snake_ident(ident)),
        ParseQueryParamType::Entity(_) => quote!(entities),
        ParseQueryParamType::EntityAny => quote!(entities),
        ParseQueryParamType::OneOf(_) => panic!("must unpack OneOf first"),
    }
}

fn to_borrow(param: &ParseQueryParam) -> TokenStream {
    match (param.is_mut, &param.param_type) {
        (false, ParseQueryParamType::Component(ident)) => quote!(borrow_slice::<#ident>()),
        (true, ParseQueryParamType::Component(ident)) => quote!(borrow_slice_mut::<#ident>()),
        (_, ParseQueryParamType::Entity(_)) => quote!(get_slice_entities()),
        (_, ParseQueryParamType::EntityAny) => quote!(get_slice_entities()),
        (_, ParseQueryParamType::OneOf(_)) => panic!("must unpack OneOf first"),
    }
}

fn to_maybe_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.is_mut {
        true => quote!(mut),
        false => quote!(),
    }
}

fn to_maybe_into(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::EntityAny => quote!(.into()),
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

fn bind_query_params(
    world_data: &DataWorld,
    params: &[ParseQueryParam],
) -> syn::Result<HashMap<String, Box<[ParseQueryParam]>>> {
    let mut result = HashMap::new();
    let mut bound = Vec::new();

    for archetype in world_data.archetypes.iter() {
        bound.clear();

        for param in params {
            match &param.param_type {
                ParseQueryParamType::EntityAny => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::Component(name) => {
                    if archetype.contains_component(name) {
                        bound.push(param.clone());
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::Entity(name) => {
                    if archetype.name == name.to_string() {
                        bound.push(param.clone());
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::OneOf(args) => {
                    if let Some(found) = bind_one_of(archetype, args)? {
                        // Convert this to a new Component type
                        bound.push(ParseQueryParam {
                            name: param.name.clone(),
                            is_mut: param.is_mut,
                            param_type: found,
                        });
                    } else {
                        continue; // No need to check more
                    }
                }
            }
        }

        // Did we remap everything?
        if bound.len() == params.len() {
            result.insert(archetype.name.clone(), bound.clone().into_boxed_slice());
        }
    }

    Ok(result)
}

fn bind_one_of(
    archetype: &DataArchetype, //.
    one_of_args: &[Ident],
) -> syn::Result<Option<ParseQueryParamType>> {
    let mut found: Option<Ident> = None;

    for arg in one_of_args.iter() {
        if archetype.contains_component(arg) {
            // An OneOf can only match one component in a given archetype
            if let Some(found) = found {
                return Err(syn::Error::new(
                    arg.span(),
                    format!(
                        "OneOf parameter is ambiguous for {}, matching both {} and {}",
                        archetype.name,
                        found.to_string(),
                        arg.to_string(),
                    ),
                ));
            }

            // We found at least one match for this archetype
            found = Some(arg.clone());
        }
    }

    // TODO: What about OneOf<Entity<A>, Entity<B>>?
    Ok(found.map(|ident| ParseQueryParamType::Component(ident)))
}
