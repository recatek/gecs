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

    // TODO PERF: We could avoid binding entirely if we know that the params have no OneOf.

    // Types
    let WorldDispatch = format_ident!("{}DispatchInternal", world_data.name);

    // Variables and fields
    let world = &query.world;
    let entity = &query.entity;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();

    // We want this to be hygenic because it's declared above the closure.
    let resolved_entity = quote_spanned!(Span::mixed_site() => entity);

    // Keywords
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        debug_assert!(archetype.build_data.is_none());

        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let ArchetypeRaw = format_ident!("{}Raw", archetype.name);
            let Type = bound_params
                .iter()
                .map(|p| to_type(p, &archetype))
                .collect::<Vec<_>>(); // Bind-dependent!

            #[rustfmt::skip]
            let get_archetype = match mode {
                FetchMode::Borrow => quote!(#world.archetype::<#Archetype>()),
                FetchMode::Mut => quote!(#world.archetype_mut::<#Archetype>()),
            };

            #[rustfmt::skip]
            let let_resolve = match mode {
                FetchMode::Borrow => quote!(let Some(idx) = archetype.resolve(#resolved_entity)),
                FetchMode::Mut => quote!(let Some((idx, entries)) = archetype.get_all_entries_mut(#resolved_entity)),
            };

            #[rustfmt::skip]
            let bind = match mode {
                FetchMode::Borrow => bound_params.iter().map(find_bind_borrow).collect::<Vec<_>>(),
                FetchMode::Mut => bound_params.iter().map(find_bind_mut).collect::<Vec<_>>(),
            };

            queries.push(quote!(
                #WorldDispatch::#Archetype(#resolved_entity) => {
                    // Alias the current archetype for use in the closure.
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types.
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();

                    if #let_resolve {
                        closure(#(#bind),*);
                        true
                    } else {
                        false
                    }
                }
                #WorldDispatch::#ArchetypeRaw(#resolved_entity) => {
                    // Alias the current archetype for use in the closure.
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types.
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();

                    if #let_resolve {
                        closure(#(#bind),*);
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
            match #WorldDispatch::from(#entity) {
                #(#queries)*
                _ => false,
            }
        }
    ))
}

#[rustfmt::skip]
fn find_bind_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => { 
            let ident = to_snake_ident(ident); quote!(entries.#ident)
        }
        ParseQueryParamType::Entity(_) => {
            quote!(entries.entity)
        }
        ParseQueryParamType::EntityAny => {
            quote!(entries.entity.into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(entries.entity)
        }
        ParseQueryParamType::EntityRaw(_) => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityRawAny => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityRawWild => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
    }
}

#[rustfmt::skip]
fn find_bind_borrow(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => {
            match param.is_mut { 
                true => quote!(&mut archetype.borrow_slice_mut::<#ident>()[idx]),
                false => quote!(&archetype.borrow_slice::<#ident>()[idx]),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(&archetype.get_slice_entities()[idx])
        }
        ParseQueryParamType::EntityAny => {
            quote!(&archetype.get_slice_entities()[idx].into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(&archetype.get_slice_entities()[idx])
        }
        ParseQueryParamType::EntityRaw(_) => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityRawAny => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityRawWild => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
    }
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

    // TODO PERF: We could avoid binding entirely if we know that the params have no OneOf.

    // Variables and fields
    let world = &query.world;
    let body = &query.body;
    let arg = query.params.iter().map(to_name).collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query.params.iter().map(to_maybe_mut).collect::<Vec<_>>();

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        debug_assert!(archetype.build_data.is_none());

        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let Type = bound_params
                .iter()
                .map(|p| to_type(p, &archetype))
                .collect::<Vec<_>>(); // Bind-dependent!

            #[rustfmt::skip]
            let get_archetype = match mode {
                FetchMode::Borrow => quote!(#world.archetype::<#Archetype>()),
                FetchMode::Mut => quote!(#world.archetype_mut::<#Archetype>()),
            };

            #[rustfmt::skip]
            let get_slices = match mode {
                FetchMode::Borrow => quote!(()),
                FetchMode::Mut => quote!(archetype.get_all_slices_mut()),
            };

            #[rustfmt::skip]
            let bind = match mode {
                FetchMode::Borrow => bound_params.iter().map(iter_bind_borrow).collect::<Vec<_>>(),
                FetchMode::Mut => bound_params.iter().map(iter_bind_mut).collect::<Vec<_>>(),
            };

            queries.push(quote!(
                {
                    // Alias the current archetype for use in the closure
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types
                    let mut closure = |#(#arg: &#maybe_mut #Type),*| #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();
                    let len = archetype.len();
                    let slices = #get_slices;

                    for idx in 0..len {
                        closure(#(#bind),*);
                    }
                }
            ));
        }
    }
    Ok(quote!(#(#queries)*))
}

#[rustfmt::skip]
fn iter_bind_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => { 
            let ident = to_snake_ident(ident); 
            match param.is_mut { 
                true => quote!(&mut slices.#ident[idx]),
                false => quote!(&slices.#ident[idx]),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(&slices.entity[idx])
        }
        ParseQueryParamType::EntityAny => {
            quote!(&slices.entity[idx].into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(&slices.entity[idx])
        }
        ParseQueryParamType::EntityRaw(_) => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityRawAny => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityRawWild => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
    }
}

#[rustfmt::skip]
fn iter_bind_borrow(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => {
            match param.is_mut { 
                true => quote!(&mut archetype.borrow_slice_mut::<#ident>()[idx]),
                false => quote!(&archetype.borrow_slice::<#ident>()[idx]),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(&archetype.get_slice_entities()[idx])
        }
        ParseQueryParamType::EntityAny => {
            quote!(&archetype.get_slice_entities()[idx].into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(&archetype.get_slice_entities()[idx])
        }
        ParseQueryParamType::EntityRaw(_) => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityRawAny => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityRawWild => {
            quote!(&::gecs::__internal::new_entity_raw::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
    }
}

fn to_name(param: &ParseQueryParam) -> TokenStream {
    let name = &param.name;
    quote!(#name)
}

#[rustfmt::skip]
fn to_type(param: &ParseQueryParam, archetype: &DataArchetype) -> TokenStream {
    let archetype_name = format_ident!("{}", archetype.name);
    match &param.param_type {
        ParseQueryParamType::Component(ident) => quote!(#ident),
        ParseQueryParamType::Entity(ident) => quote!(Entity<#ident>),
        ParseQueryParamType::EntityAny => quote!(EntityAny),
        ParseQueryParamType::EntityWild => quote!(Entity<#archetype_name>),
        ParseQueryParamType::EntityRaw(ident) => quote!(EntityRaw<#ident>),
        ParseQueryParamType::EntityRawAny => quote!(EntityRawAny),
        ParseQueryParamType::EntityRawWild => quote!(EntityRaw<#archetype_name>),
        ParseQueryParamType::OneOf(_) => panic!("must unpack OneOf first"),
    }
}

fn to_maybe_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.is_mut {
        true => quote!(mut),
        false => quote!(),
    }
}

fn to_snake_ident(ident: &Ident) -> Ident {
    Ident::new(&to_snake_str(&ident.to_string()), ident.span())
}

fn to_snake_str(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
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
                ParseQueryParamType::EntityRawAny => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::EntityWild => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::EntityRawWild => {
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
                ParseQueryParamType::EntityRaw(name) => {
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
