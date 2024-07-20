mod data;
mod generate;
mod parse;

use data::DataWorld;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input};

use generate::FetchMode;

use parse::*;

/// See `ecs_world!` in the `gecs` docs for more info.
#[proc_macro]
pub fn ecs_world(args: TokenStream) -> TokenStream {
    let raw = args.clone().into(); // We'll need to parse twice
    let world_parse = parse_macro_input!(args as ParseEcsWorld);
    generate::generate_cfg_checks(&world_parse, raw).into()
}

/// See `ecs_component_id!` in the `gecs` docs for more info.
#[proc_macro]
pub fn ecs_component_id(args: TokenStream) -> TokenStream {
    let util = parse_macro_input!(args as ParseEcsComponentId);
    generate::generate_ecs_component_id(util).into()
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_finalize(args: TokenStream) -> TokenStream {
    let raw_input = args.to_string();

    match DataWorld::new(parse_macro_input!(args as ParseEcsFinalize)) {
        Ok(world_data) => generate::generate_world(&world_data, &raw_input).into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_find(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryFind);

    match generate::generate_query_find(FetchMode::Mut, query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_find_borrow(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryFind);

    match generate::generate_query_find(FetchMode::Borrow, query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_iter(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryIter);

    match generate::generate_query_iter(FetchMode::Mut, query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_iter_borrow(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryIter);

    match generate::generate_query_iter(FetchMode::Borrow, query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
#[doc(hidden)]
pub fn __ecs_iter_destroy(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryIterDestroy);

    match generate::generate_query_iter_destroy(FetchMode::Mut, query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
