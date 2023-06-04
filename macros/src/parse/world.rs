use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Dyn, Pound, Semi};
use syn::{braced, bracketed, parenthesized, Ident, LitBool, LitInt, Token};

use super::*;

mod kw {
    syn::custom_keyword!(ecs_archetype);
    syn::custom_keyword!(ecs_name);
    syn::custom_keyword!(cfg);
}

#[derive(Debug)]
pub struct ParseFinalize {
    pub cfg_lookup: HashMap<String, bool>,
    pub world: ParseWorld,
}

#[derive(Debug)]
pub struct ParseWorld {
    pub name: Ident,
    pub archetypes: Vec<ParseArchetype>,
}

#[derive(Debug)]
pub enum ParseItem {
    ParseName(ParseName),
    ParseArchetype(ParseArchetype),
}

#[derive(Debug)]
pub struct ParseName {
    pub name: Ident,
}

#[derive(Debug)]
pub struct ParseArchetype {
    pub cfgs: Vec<ParseCfg>,
    pub name: Ident,
    pub capacity: ParseCapacity,
    pub components: Vec<ParseComponent>,
}

#[derive(Debug)]
pub struct ParseComponent {
    pub cfgs: Vec<ParseCfg>,
    pub name: Ident,
}

#[derive(Debug)]
pub struct ParseCfg {
    pub predicate: TokenStream,
}

#[derive(Debug)]
pub enum ParseCapacity {
    Literal(LitInt),
    Constant(Ident),
    Dynamic,
}

impl ParseWorld {
    pub fn collect_all_cfg_predicates(&self) -> Vec<TokenStream> {
        let mut filter = HashSet::new();
        let mut result = Vec::new();

        // Filter duplicates while keeping order for determinism
        for archetype in self.archetypes.iter() {
            for cfg in archetype.cfgs.iter() {
                let predicate_tokens = cfg.predicate.clone();
                let predicate_string = predicate_tokens.to_string();

                if filter.insert(predicate_string) {
                    result.push(predicate_tokens);
                }
            }

            for component in archetype.components.iter() {
                for cfg in component.cfgs.iter() {
                    let predicate_tokens = cfg.predicate.clone();
                    let predicate_string = predicate_tokens.to_string();

                    if filter.insert(predicate_string) {
                        result.push(predicate_tokens);
                    }
                }
            }
        }

        result
    }
}

impl Parse for ParseFinalize {
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
        let world;
        braced!(world in input);
        let world = world.parse::<ParseWorld>()?;

        // Create a map of each cfg to its deduced state
        let mut predicates = world.collect_all_cfg_predicates();
        let mut cfg_lookup = HashMap::with_capacity(states.len());

        assert!(predicates.len() == states.len());
        for (predicate, state) in predicates.drain(..).zip(states) {
            cfg_lookup.insert(predicate.to_string(), state);
        }

        Ok(Self { cfg_lookup, world })
    }
}

impl Parse for ParseWorld {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::<ParseItem, Semi>::parse_terminated(input)?
            .into_iter()
            .collect::<Vec<_>>();

        let mut name = format_ident!("World");
        let mut archetypes = Vec::new();

        for item in items {
            match item {
                ParseItem::ParseArchetype(item) => {
                    // Collect all the archetypes
                    archetypes.push(item);
                }
                ParseItem::ParseName(item) => {
                    // TODO: Check for duplicates?
                    name = item.name;
                }
            }
        }

        // TODO: Check for duplicates?

        Ok(Self { name, archetypes })
    }
}

impl Parse for ParseItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cfgs = input.call(parse_cfg_outer)?.into_iter().collect::<Vec<_>>();

        let span = input.span();
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::ecs_archetype) {
            // archetype! pseudo-macro
            let mut archetype = input.parse::<ParseArchetype>()?;
            archetype.cfgs.extend(cfgs); // Push the parsed cfgs
            Ok(ParseItem::ParseArchetype(archetype))
        } else if lookahead.peek(kw::ecs_name) {
            // name! pseudo-macro
            if cfgs.is_empty() == false {
                Err(syn::Error::new(
                    span,
                    "#[cfg] unsupported on name declaration",
                ))
            } else {
                let name = input.parse::<ParseName>()?;
                Ok(ParseItem::ParseName(name))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for ParseName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::ecs_name>()?;
        input.parse::<Token![!]>()?;

        let content;
        parenthesized!(content in input);

        let name: Ident = content.parse()?;

        // TODO: Check for duplicates?

        Ok(Self { name })
    }
}

impl Parse for ParseArchetype {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cfgs = Vec::new(); // This will be filled at the item level

        input.parse::<kw::ecs_archetype>()?;
        input.parse::<Token![!]>()?;

        let content;
        parenthesized!(content in input);

        let name: Ident = content.parse()?;

        content.parse::<Comma>()?;
        let capacity: ParseCapacity = content.parse()?;
        content.parse::<Comma>()?;
        let components: Vec<ParseComponent> =
            Punctuated::<ParseComponent, Comma>::parse_terminated(&content)?
                .into_iter()
                .collect();

        // TODO: Check for duplicates?

        Ok(Self {
            cfgs,
            name,
            capacity,
            components,
        })
    }
}

impl Parse for ParseComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cfgs = input.call(parse_cfg_outer)?.into_iter().collect();
        let name = input.parse::<Ident>()?;

        // Don't allow special keyword names as component types
        if is_allowed_component_name(&name.to_string()) == false {
            return Err(syn::Error::new_spanned(name, "illegal component name"));
        }

        Ok(Self { cfgs, name })
    }
}

impl Parse for ParseCfg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Pound>()?;

        let content;
        bracketed!(content in input);

        content.parse::<kw::cfg>()?;

        let predicate;
        parenthesized!(predicate in content);

        // Don't care about parsing the predicate contents
        let predicate = predicate.parse::<TokenStream>()?;

        Ok(ParseCfg { predicate })
    }
}

impl Parse for ParseCapacity {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            input.parse().map(ParseCapacity::Literal)
        } else if lookahead.peek(Ident) {
            input.parse().map(ParseCapacity::Constant)
        } else if lookahead.peek(Dyn) {
            input.parse::<Dyn>()?;
            Ok(ParseCapacity::Dynamic)
        } else {
            Err(lookahead.error())
        }
    }
}

fn parse_cfg_outer(input: ParseStream) -> syn::Result<Vec<ParseCfg>> {
    let mut attrs = Vec::new();
    while input.peek(Token![#]) {
        attrs.push(input.parse::<ParseCfg>()?);
    }
    Ok(attrs)
}
