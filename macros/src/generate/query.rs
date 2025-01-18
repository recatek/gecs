use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::data::{DataArchetype, DataWorld};
use crate::generate::util::to_snake;
use crate::parse::ParseEcsComponentId;

use crate::parse::{
    ParseCfgDecorated,
    ParseQueryFind, //.
    ParseQueryIter,
    ParseQueryIterDestroy,
    ParseQueryParam,
    ParseQueryParamType,
};

#[allow(non_snake_case)]
pub fn generate_ecs_component_id(util: ParseEcsComponentId) -> TokenStream {
    let Component = &util.component;
    let Archetype = match util.archetype {
        Some(archetype) => archetype.into_token_stream(),
        None => quote!(MatchedArchetype),
    };

    quote!(<#Archetype as ArchetypeHas<#Component>>::COMPONENT_ID)
}

// NOTE: We should avoid using panics to express errors in queries when generating.
// Doing so will attribute the error to the ecs_world! declaration (due to the redirect
// macro) rather than to the query macro itself. Always use an Err result where possible.

#[derive(Clone, Copy, Debug)]
pub enum FetchMode {
    Mut,
    Borrow,
}

#[allow(non_snake_case)]
pub fn generate_query_find(
    mode: FetchMode, //.
    query: ParseCfgDecorated<ParseQueryFind>,
) -> syn::Result<TokenStream> {
    let mut query_data = query.inner;
    let world_data = DataWorld::from_base64(&query_data.world_data);

    // Precompute the cfg-enabled status of any parameter in the predicate.
    for param in query_data.params.iter_mut() {
        param.is_cfg_enabled = is_cfg_enabled(param, &query.cfg_lookup);
    }

    let bound_params = bind_query_params(&world_data, &query_data.params)?;
    // NOTE: Beyond this point, query.params is only safe to use for information that
    // does not change depending on the type of the parameter (e.g. mutability). Anything
    // that might change after OneOf binding etc. must use the bound query params in
    // bound_params for the given archetype. Note that it's faster to use query.params
    // where available, since it avoids redundant computation for each archetype.

    // TODO PERF: We could avoid binding entirely if we know that the params have no OneOf.

    // Types
    let __WorldSelectTotal = format_ident!("__{}SelectTotal", world_data.name);

    // Variables and fields
    let world = &query_data.world;
    let entity = &query_data.entity;
    let body = &query_data.body;
    let arg = query_data.params.iter().map(to_name).collect::<Vec<_>>();
    let attrs = query_data
        .params
        .iter()
        .map(to_attributes)
        .collect::<Vec<_>>();

    // We want this to be hygenic because it's declared above the closure.
    let resolved_entity = quote_spanned!(Span::mixed_site() => entity);

    // Keywords
    let maybe_mut = query_data
        .params
        .iter()
        .map(to_maybe_mut)
        .collect::<Vec<_>>();

    // Explicit return value on the query
    let ret = match &query_data.ret {
        Some(ret) => quote!(-> #ret),
        None => quote!(),
    };

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let ArchetypeDirect = format_ident!("{}Direct", archetype.name);
            let Type = bound_params
                .iter()
                .map(|p| to_type(p, &archetype))
                .collect::<Vec<_>>(); // Bind-dependent!

            // Variables
            let archetype = format_ident!("{}", to_snake(&archetype.name));

            // Fetch the archetype directly to allow queries to be sneaky with
            // direct archetype access to get cross-archetype nested mutability
            let get_archetype = match mode {
                FetchMode::Borrow => quote!(&#world.#archetype),
                FetchMode::Mut => quote!(&mut #world.#archetype),
            };

            let fetch = match mode {
                FetchMode::Borrow => quote!(archetype.borrow(#resolved_entity)),
                FetchMode::Mut => quote!(archetype.view(#resolved_entity)),
            };

            #[rustfmt::skip]
            let bind = match mode {
                FetchMode::Borrow => bound_params.iter().map(find_bind_borrow).collect::<Vec<_>>(),
                FetchMode::Mut => bound_params.iter().map(find_bind_mut).collect::<Vec<_>>(),
            };

            queries.push(quote!(
                #__WorldSelectTotal::#Archetype(#resolved_entity) => {
                    // Alias the current archetype for use in the closure.
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types.
                    let mut closure = |#(#attrs #arg: &#maybe_mut #Type),*| #ret #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();

                    #fetch.map(|found| closure(#(#attrs #bind),*))
                }
                #__WorldSelectTotal::#ArchetypeDirect(#resolved_entity) => {
                    // Alias the current archetype for use in the closure.
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types.
                    let mut closure = |#(#attrs #arg: &#maybe_mut #Type),*| #ret #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();

                    #fetch.map(|found| closure(#(#attrs #bind),*))
                }
            ));
        }
    }

    if queries.is_empty() {
        Err(syn::Error::new_spanned(
            world,
            "query matched no archetypes in world",
        ))
    } else {
        Ok(quote!(
            {
                match #__WorldSelectTotal::try_from(#entity).expect("invalid entity type") {
                    #(#queries)*
                    _ => None,
                }
            }
        ))
    }
}

#[rustfmt::skip]
fn find_bind_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => { 
            let ident = to_snake_ident(ident); quote!(found.#ident)
        }
        ParseQueryParamType::Entity(_) => {
            quote!(found.entity)
        }
        ParseQueryParamType::EntityAny => {
            quote!(&(*found.entity).into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(found.entity)
        }
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version).into())
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
        ParseQueryParamType::With(_) => {
            todo!() // Not yet implemented
        }
        ParseQueryParamType::Without(_) => {
            todo!() // Not yet implemented
        }
    }
}

#[rustfmt::skip]
fn find_bind_borrow(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(ident) => {
            match param.is_mut { 
                true => quote!(&mut found.component_mut::<#ident>()),
                false => quote!(&found.component::<#ident>()),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(found.entity())
        }
        ParseQueryParamType::EntityAny => {
            quote!(&(*found.entity()).into())
        }
        ParseQueryParamType::EntityWild => {
            quote!(found.entity())
        }
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version).into())
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(found.index(), version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
                ParseQueryParamType::With(_) => {
            todo!() // Not yet implemented
        }
        ParseQueryParamType::Without(_) => {
            todo!() // Not yet implemented
        }
    }
}

#[allow(non_snake_case)]
pub fn generate_query_iter(
    mode: FetchMode, //.
    query: ParseCfgDecorated<ParseQueryIter>,
) -> syn::Result<TokenStream> {
    let mut query_data = query.inner;
    let world_data = DataWorld::from_base64(&query_data.world_data);

    // Precompute the cfg-enabled status of any parameter in the predicate.
    for param in query_data.params.iter_mut() {
        param.is_cfg_enabled = is_cfg_enabled(param, &query.cfg_lookup);
    }

    let bound_params = bind_query_params(&world_data, &query_data.params)?;
    // NOTE: Beyond this point, query.params is only safe to use for information that
    // does not change depending on the type of the parameter (e.g. mutability). Anything
    // that might change after OneOf binding etc. must use the bound query params in
    // bound_params for the given archetype. Note that it's faster to use query.params
    // where available, since it avoids redundant computation for each archetype.

    // TODO PERF: We could avoid binding entirely if we know that the params have no OneOf.

    // Variables and fields
    let world = &query_data.world;
    let body = &query_data.body;
    let arg = query_data.params.iter().map(to_name).collect::<Vec<_>>();
    let attrs = query_data
        .params
        .iter()
        .map(to_attributes)
        .collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query_data
        .params
        .iter()
        .map(to_maybe_mut)
        .collect::<Vec<_>>();

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let Type = bound_params
                .iter()
                .map(|p| to_type(p, &archetype))
                .collect::<Vec<_>>(); // Bind-dependent!

            // Variables
            let archetype = format_ident!("{}", to_snake(&archetype.name));

            // Fetch the archetype directly to allow queries to be sneaky with
            // direct archetype access to get cross-archetype nested mutability
            #[rustfmt::skip]
            let get_archetype = match mode {
                FetchMode::Borrow => quote!(&#world.#archetype),
                FetchMode::Mut => quote!(&mut #world.#archetype),
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
                    let mut closure = |#(#attrs #arg: &#maybe_mut #Type),*| #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();
                    let len = archetype.len();
                    let slices = #get_slices;

                    for idx in 0..len {
                        match closure(#(#attrs #bind),*).into() {
                            EcsStep::Continue => {
                                // Continue
                            },
                            EcsStep::Break => {
                                return;
                            },
                        }
                    }
                }
            ));
        }
    }

    if queries.is_empty() {
        Err(syn::Error::new_spanned(
            world,
            "query matched no archetypes in world",
        ))
    } else {
        Ok(quote!(
            // Use a closure so we can use return to cancel other archetype iterations
            (||{#(#queries)*})()
        ))
    }
}

#[allow(non_snake_case)]
pub fn generate_query_iter_destroy(
    mode: FetchMode,
    query: ParseCfgDecorated<ParseQueryIterDestroy>,
) -> syn::Result<TokenStream> {
    let mut query_data = query.inner;
    let world_data = DataWorld::from_base64(&query_data.world_data);

    // Precompute the cfg-enabled status of any parameter in the predicate.
    for param in query_data.params.iter_mut() {
        param.is_cfg_enabled = is_cfg_enabled(param, &query.cfg_lookup);
    }

    let bound_params = bind_query_params(&world_data, &query_data.params)?;
    // NOTE: Beyond this point, query.params is only safe to use for information that
    // does not change depending on the type of the parameter (e.g. mutability). Anything
    // that might change after OneOf binding etc. must use the bound query params in
    // bound_params for the given archetype. Note that it's faster to use query.params
    // where available, since it avoids redundant computation for each archetype.

    // TODO PERF: We could avoid binding entirely if we know that the params have no OneOf.

    // Variables and fields
    let world = &query_data.world;
    let body = &query_data.body;
    let arg = query_data.params.iter().map(to_name).collect::<Vec<_>>();
    let attrs = query_data
        .params
        .iter()
        .map(to_attributes)
        .collect::<Vec<_>>();

    // Special cases
    let maybe_mut = query_data
        .params
        .iter()
        .map(to_maybe_mut)
        .collect::<Vec<_>>();

    let mut queries = Vec::<TokenStream>::new();
    for archetype in world_data.archetypes {
        if let Some(bound_params) = bound_params.get(&archetype.name) {
            // Types and traits
            let Archetype = format_ident!("{}", archetype.name);
            let Type = bound_params
                .iter()
                .map(|p| to_type(p, &archetype))
                .collect::<Vec<_>>(); // Bind-dependent!

            // Variables
            let archetype = format_ident!("{}", to_snake(&archetype.name));

            // Fetch the archetype directly to allow queries to be sneaky with
            // direct archetype access to get cross-archetype nested mutability
            #[rustfmt::skip]
            let get_archetype = match mode {
                FetchMode::Borrow => quote!(&#world.#archetype),
                FetchMode::Mut => quote!(&mut #world.#archetype),
            };

            #[rustfmt::skip]
            let get_slices = match mode {
                FetchMode::Borrow => panic!("borrow unsupported for iter_remove"),
                FetchMode::Mut => quote!(archetype.get_all_slices_mut()),
            };

            #[rustfmt::skip]
            let bind = match mode {
                FetchMode::Borrow => panic!("borrow unsupported for iter_remove"),
                FetchMode::Mut => bound_params.iter().map(iter_bind_mut).collect::<Vec<_>>(),
            };

            queries.push(quote!(
                {
                    // Alias the current archetype for use in the closure
                    type MatchedArchetype = #Archetype;
                    // The closure needs to be made per-archetype because of OneOf types
                    let mut closure = |#(#attrs #arg: &#maybe_mut #Type),*| #body;

                    let archetype = #get_archetype;
                    let version = archetype.version();
                    let len = archetype.len();

                    // Iterate in reverse order to still visit each entity once.
                    // Note: This assumes that we remove entities by swapping.
                    for idx in (0..len).rev() {
                        let slices = #get_slices;
                        match closure(#(#attrs #bind),*).into() {
                            EcsStepDestroy::Continue => {
                                // Continue
                            },
                            EcsStepDestroy::Break => {
                                return;
                            },
                            EcsStepDestroy::ContinueDestroy => {
                                let entity = slices.entity[idx];
                                archetype.destroy(entity);
                            },
                            EcsStepDestroy::BreakDestroy => {
                                let entity = slices.entity[idx];
                                archetype.destroy(entity);
                                return;
                            },
                        }
                    }
                }
            ));
        }
    }

    if queries.is_empty() {
        Err(syn::Error::new_spanned(
            world,
            "query matched no archetypes in world",
        ))
    } else {
        Ok(quote!(
            // Use a closure so we can use return to cancel other archetype iterations
            (||{#(#queries)*})()
        ))
    }
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
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
                ParseQueryParamType::With(_) => {
            todo!() // Not yet implemented
        }
        ParseQueryParamType::Without(_) => {
            todo!() // Not yet implemented
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
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
                ParseQueryParamType::With(_) => {
            todo!() // Not yet implemented
        }
        ParseQueryParamType::Without(_) => {
            todo!() // Not yet implemented
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
        ParseQueryParamType::EntityDirect(ident) => quote!(EntityDirect<#ident>),
        ParseQueryParamType::EntityDirectAny => quote!(EntityDirectAny),
        ParseQueryParamType::EntityDirectWild => quote!(EntityDirect<#archetype_name>),
        ParseQueryParamType::OneOf(_) => panic!("must unpack OneOf first"),
        ParseQueryParamType::With(_) => todo!(),
        ParseQueryParamType::Without(_) => todo!(),
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

fn to_attributes(param: &ParseQueryParam) -> TokenStream {
    let mut attrs = TokenStream::new();
    for cfg in param.cfgs.iter() {
        let predicate = &cfg.predicate;
        attrs.extend(quote!(#[cfg(#predicate)]));
    }
    attrs
}

fn is_cfg_enabled(param: &ParseQueryParam, cfg_lookup: &HashMap<String, bool>) -> bool {
    for cfg in param.cfgs.iter() {
        if *cfg_lookup.get(&cfg.predicate.to_string()).unwrap() == false {
            return false;
        }
    }
    return true;
}

fn bind_query_params(
    world_data: &DataWorld,
    params: &[ParseQueryParam],
) -> syn::Result<HashMap<String, Vec<ParseQueryParam>>> {
    let mut result = HashMap::new();
    let mut bound = Vec::new();

    for archetype in world_data.archetypes.iter() {
        bound.clear();

        for param in params {
            match &param.param_type {
                ParseQueryParamType::EntityAny => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::EntityDirectAny => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::EntityWild => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::EntityDirectWild => {
                    bound.push(param.clone()); // Always matches
                }
                ParseQueryParamType::Component(name) => {
                    if param.is_cfg_enabled == false || archetype.contains_component(name) {
                        bound.push(param.clone());
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::Entity(name) => {
                    if param.is_cfg_enabled == false || archetype.name == name.to_string() {
                        bound.push(param.clone());
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::EntityDirect(name) => {
                    if param.is_cfg_enabled == false || archetype.name == name.to_string() {
                        bound.push(param.clone());
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::OneOf(args) => {
                    if param.cfgs.len() > 0 {
                        return Err(syn::Error::new(
                            param.name.span(),
                            "cfg attributes not currently supported on OneOf",
                        ));
                    }

                    if let Some(found) = bind_one_of(archetype, args)? {
                        // Convert this to a new Component type
                        bound.push(ParseQueryParam {
                            cfgs: param.cfgs.clone(),
                            name: param.name.clone(),
                            is_mut: param.is_mut,
                            param_type: found,
                            is_cfg_enabled: param.is_cfg_enabled,
                        });
                    } else {
                        continue; // No need to check more
                    }
                }
                ParseQueryParamType::With(_) => {
                    todo!() // Not yet implemented
                }
                ParseQueryParamType::Without(_) => {
                    todo!() // Not yet implemented
                }
            }
        }

        // Did we remap everything?
        if bound.len() == params.len() {
            result.insert(archetype.name.clone(), bound.clone());
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
