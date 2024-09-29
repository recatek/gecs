use std::collections::HashMap;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{braced, parenthesized, LitBool};

pub trait HasCfgPredicates {
    fn macro_name() -> &'static str;
    fn collect_all_cfg_predicates(&self) -> Vec<TokenStream>;
}

#[derive(Debug)]
pub struct ParseCfgDecorated<T: Parse + HasCfgPredicates> {
    pub cfg_lookup: HashMap<String, bool>,
    pub inner: T,
}

impl<T: Parse + HasCfgPredicates> Parse for ParseCfgDecorated<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the deduced states for each cfg in the world definition
        let states;
        parenthesized!(states in input);
        let states: Vec<bool> = Punctuated::<LitBool, Comma>::parse_terminated(&states)?
            .into_iter()
            .map(|bool| bool.value)
            .collect();
        input.parse::<Comma>()?;

        // Re-parse the world itself (TODO: This is wasteful!)
        let inner;
        braced!(inner in input);
        let inner = inner.parse::<T>()?;

        // Create a map of each cfg to its deduced state
        let mut predicates = inner.collect_all_cfg_predicates();
        let mut cfg_lookup = HashMap::with_capacity(states.len());

        assert!(predicates.len() == states.len());
        for (predicate, state) in predicates.drain(..).zip(states) {
            cfg_lookup.insert(predicate.to_string(), state);
        }

        Ok(Self { cfg_lookup, inner })
    }
}
