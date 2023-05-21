mod data;
mod generate;
mod parse;

use data::World;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input};

use parse::*;

#[proc_macro]
pub fn ecs_world(args: TokenStream) -> TokenStream {
    let raw = args.clone().into(); // We'll need to parse twice
    let world = parse_macro_input!(args as ParseWorld);
    generate::generate_cfg_checks(world, raw).into()
}

#[proc_macro]
pub fn __ecs_finalize(args: TokenStream) -> TokenStream {
    let world = World::new(parse_macro_input!(args as ParseFinalize));
    let raw = format!("{:#?}", world);
    quote!(pub const OUTPUT: &str = #raw;).into()
}
