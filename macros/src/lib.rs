mod data;
mod generate;
mod parse;

use data::DataWorld;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input};

use parse::*;

#[proc_macro]
pub fn ecs_world(args: TokenStream) -> TokenStream {
    let raw = args.clone().into(); // We'll need to parse twice
    let world_parse = parse_macro_input!(args as ParseWorld);
    generate::generate_cfg_checks(&world_parse, raw).into()
}

#[proc_macro]
pub fn __ecs_finalize(args: TokenStream) -> TokenStream {
    let world_data = DataWorld::new(parse_macro_input!(args as ParseFinalize));
    generate::generate_world(&world_data).into()
}

#[proc_macro]
pub fn __ecs_iter_mut(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryIter);

    match generate::generate_query_iter_mut(&query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn __ecs_iter_borrow(args: TokenStream) -> TokenStream {
    let query_parse = parse_macro_input!(args as ParseQueryIter);

    match generate::generate_query_iter_borrow(&query_parse) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
