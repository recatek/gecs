use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::data::{DataArchetype, DataWorld};
use crate::parse::ParseEcsComponentId;
use crate::util;

use crate::parse::{
    ParseCfgDecorated,
    ParseComponentName,
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
            let archetype = format_ident!("{}", util::to_snake(&archetype.name));

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
        ParseQueryParamType::Component(name) => { 
            let name = Ident::new(&name.as_snake_name(), Span::call_site()).to_token_stream();
            quote!(found.#name)
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
        ParseQueryParamType::Option(_) => {
            todo!() // Not yet implemented
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
        ParseQueryParamType::Component(name) => {
            match param.is_mut { 
                true => quote!(&mut found.component_mut::<#name>()),
                false => quote!(&found.component::<#name>()),
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
        ParseQueryParamType::Option(_) => {
            todo!() // Not yet implemented
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
            let archetype = format_ident!("{}", util::to_snake(&archetype.name));

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
            let archetype = format_ident!("{}", util::to_snake(&archetype.name));

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
        ParseQueryParamType::Component(name) => { 
            let ident = Ident::new(&name.as_snake_name(), Span::call_site());
            match param.is_mut { 
                true => quote!(&mut slices.#ident[idx]),
                false => quote!(&slices.#ident[idx]),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(&slices.entity[idx])
        }
        ParseQueryParamType::EntityWild => {
            quote!(&slices.entity[idx])
        }
        ParseQueryParamType::EntityAny => {
            quote!(&slices.entity[idx].into())
        }
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
        ParseQueryParamType::Option(_) => {
            todo!("Option not yet supported")
        }
        ParseQueryParamType::With(_) => {
            todo!("With not yet supported")
        }
        ParseQueryParamType::Without(_) => {
            todo!("Without not yet supported")
        }
    }
}

#[rustfmt::skip]
fn iter_bind_borrow(param: &ParseQueryParam) -> TokenStream {
    match &param.param_type {
        ParseQueryParamType::Component(name) => {
            match param.is_mut {
                true => quote!(&mut archetype.borrow_slice_mut::<#name>()[idx]),
                false => quote!(&archetype.borrow_slice::<#name>()[idx]),
            }
        }
        ParseQueryParamType::Entity(_) => {
            quote!(&archetype.entities()[idx])
        }
        ParseQueryParamType::EntityWild => {
            quote!(&archetype.entities()[idx])
        }
        ParseQueryParamType::EntityAny => {
            quote!(&archetype.entities()[idx].into())
        }
        ParseQueryParamType::EntityDirect(_) => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectWild => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version))
        }
        ParseQueryParamType::EntityDirectAny => {
            quote!(&::gecs::__internal::new_entity_direct::<MatchedArchetype>(idx, version).into())
        }
        ParseQueryParamType::OneOf(_) => {
            panic!("must unpack OneOf first")
        }
        ParseQueryParamType::Option(_) => {
            todo!("Option not yet supported")
        }
        ParseQueryParamType::With(_) => {
            todo!("With not yet supported")
        }
        ParseQueryParamType::Without(_) => {
            todo!("Without not yet supported")
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
        ParseQueryParamType::Component(name) => quote!(#name),
        ParseQueryParamType::Entity(ident) => quote!(Entity<#ident>),
        ParseQueryParamType::EntityWild => quote!(Entity<#archetype_name>),
        ParseQueryParamType::EntityAny => quote!(EntityAny),
        ParseQueryParamType::EntityDirect(ident) => quote!(EntityDirect<#ident>),
        ParseQueryParamType::EntityDirectWild => quote!(EntityDirect<#archetype_name>),
        ParseQueryParamType::EntityDirectAny => quote!(EntityDirectAny),
        ParseQueryParamType::OneOf(_) => panic!("must unpack OneOf first"),
        ParseQueryParamType::Option(_) => todo!("Option not yet supported"),
        ParseQueryParamType::With(_) => todo!("With not yet supported"),
        ParseQueryParamType::Without(_) => todo!("Without not yet supported"),
    }
}

fn to_maybe_mut(param: &ParseQueryParam) -> TokenStream {
    match &param.is_mut {
        true => quote!(mut),
        false => quote!(),
    }
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

    true
}

fn bind_query_params(
    world_data: &DataWorld,
    params: &[ParseQueryParam],
) -> syn::Result<HashMap<String, Vec<ParseQueryParam>>> {
    let mut result = HashMap::new();
    let mut binding = Vec::new();

    for archetype in world_data.archetypes.iter() {
        let mut matches = true;
        binding.clear();

        for param in params {
            match &param.param_type {
                ParseQueryParamType::Component(name) => {
                    if param.is_cfg_enabled == false {
                        continue; // Skip this entirely
                    }

                    // We have to take the archetype's version because we might have to bind a
                    // placeholder generic argument in the process (including nested in OneOf).
                    if let Some(bound) = archetype.try_bind_component(name)? {
                        binding.push(ParseQueryParam {
                            cfgs: param.cfgs.clone(),
                            name: param.name.clone(),
                            is_mut: param.is_mut,
                            param_type: ParseQueryParamType::Component(bound.clone()),
                            is_cfg_enabled: param.is_cfg_enabled,
                        });
                    } else {
                        matches = false;
                        break; // No need to check more
                    }
                }

                ParseQueryParamType::Entity(name) => {
                    if param.is_cfg_enabled == false {
                        continue; // Skip this entirely
                    }

                    if archetype.name == name.to_string() {
                        binding.push(param.clone());
                    } else {
                        matches = false;
                        break; // No need to check more
                    }
                }

                ParseQueryParamType::EntityWild => {
                    binding.push(param.clone()); // Always matches
                }

                ParseQueryParamType::EntityAny => {
                    binding.push(param.clone()); // Always matches
                }

                ParseQueryParamType::EntityDirect(name) => {
                    if param.is_cfg_enabled == false {
                        continue; // Skip this entirely
                    }

                    if archetype.name == name.to_string() {
                        binding.push(param.clone());
                    } else {
                        matches = false;
                        break; // No need to check more
                    }
                }

                ParseQueryParamType::EntityDirectWild => {
                    binding.push(param.clone()); // Always matches
                }

                ParseQueryParamType::EntityDirectAny => {
                    binding.push(param.clone()); // Always matches
                }

                ParseQueryParamType::OneOf(args) => {
                    if param.is_cfg_enabled == false {
                        continue; // Skip this entirely
                    }

                    if let Some(found) = bind_one_of(archetype, args)? {
                        // Convert this to a new Component type
                        binding.push(ParseQueryParam {
                            cfgs: param.cfgs.clone(),
                            name: param.name.clone(),
                            is_mut: param.is_mut,
                            param_type: found,
                            is_cfg_enabled: param.is_cfg_enabled,
                        });
                    } else {
                        matches = false;
                        break; // No need to check more
                    }
                }

                ParseQueryParamType::Option(_) => {
                    todo!("Option not yet supported")
                }

                ParseQueryParamType::With(_) => {
                    todo!("With not yet supported")
                }

                ParseQueryParamType::Without(_) => {
                    todo!("Without not yet supported")
                }
            }
        }

        // Did we remap everything?
        if matches {
            result.insert(archetype.name.clone(), binding.clone());
        }
    }

    Ok(result)
}

fn bind_one_of(
    archetype: &DataArchetype, //.
    one_of_args: &[ParseComponentName],
) -> syn::Result<Option<ParseQueryParamType>> {
    let mut found: Option<ParseComponentName> = None;

    for name in one_of_args.iter() {
        if let Some(bound) = archetype.try_bind_component(name)? {
            // An OneOf can only match one component in a given archetype
            if let Some(found) = found {
                return Err(syn::Error::new(
                    name.span(),
                    format!(
                        "OneOf parameter is ambiguous for {}, matching both {} and {}",
                        archetype.name, name, found,
                    ),
                ));
            }

            // We found at least one match for this archetype
            found = Some(bound.clone());
        }
    }

    // TODO: What about OneOf<Entity<A>, Entity<B>>?
    Ok(found.map(|name| ParseQueryParamType::Component(name)))
}
