use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Dyn, Semi};
use syn::{braced, bracketed, parenthesized, Expr, Ident, LitBool, LitInt, Token};

use super::*;

mod kw {
    syn::custom_keyword!(cfg);

    syn::custom_keyword!(archetype_id);
    syn::custom_keyword!(component_id);

    syn::custom_keyword!(ecs_archetype);
    syn::custom_keyword!(ecs_name);
}

pub trait HasAttributeId {
    fn name(&self) -> &Ident;
    fn id(&self) -> Option<u8>;
}

#[derive(Debug)]
pub struct ParseEcsFinalize {
    pub cfg_lookup: HashMap<String, bool>,
    pub world: ParseEcsWorld,
}

#[derive(Debug)]
pub struct ParseEcsWorld {
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
    pub cfgs: Vec<ParseAttributeCfg>,
    pub id: Option<u8>,
    pub name: Ident,
    pub capacity: ParseCapacity,
    pub components: Vec<ParseComponent>,
}

#[derive(Debug)]
pub struct ParseComponent {
    pub cfgs: Vec<ParseAttributeCfg>,
    pub id: Option<u8>,
    pub name: Ident,
}

#[derive(Debug)]
pub struct ParseAttributeCfg {
    pub predicate: TokenStream,
}

#[derive(Debug)]
pub struct ParseAttributeId {
    pub value: u8,
}

#[derive(Debug)]
pub struct ParseAttribute {
    pub span: Span,
    pub data: ParseAttributeData,
}

#[derive(Debug)]
pub enum ParseAttributeData {
    Cfg(ParseAttributeCfg),
    ArchetypeId(ParseAttributeId),
    ComponentId(ParseAttributeId),
}

#[derive(Debug)]
pub enum ParseCapacity {
    Dynamic,
    Expression(Expr),
}

impl ParseEcsWorld {
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

impl Parse for ParseEcsFinalize {
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
        let world = world.parse::<ParseEcsWorld>()?;

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

impl Parse for ParseEcsWorld {
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
        let attributes = input
            .call(parse_attributes)?
            .into_iter()
            .collect::<Vec<_>>();

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ecs_archetype) {
            parse_item_archetype(input, attributes)
        } else if lookahead.peek(kw::ecs_name) {
            parse_item_name(input, attributes)
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
            id: None,
            name,
            capacity,
            components,
        })
    }
}

impl Parse for ParseComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cfgs = Vec::new();

        let attributes = input
            .call(parse_attributes)?
            .into_iter()
            .collect::<Vec<_>>();

        let name = input.parse::<Ident>()?;

        // Don't allow special keyword names as component types
        if is_allowed_component_name(&name.to_string()) == false {
            return Err(syn::Error::new_spanned(name, "illegal component name"));
        }

        // See if we have a manually-assigned component ID
        let mut component_id = None;

        for attribute in attributes.into_iter() {
            match attribute.data {
                ParseAttributeData::Cfg(cfg) => cfgs.push(cfg),
                ParseAttributeData::ComponentId(id) => {
                    if component_id.is_some() {
                        return Err(syn::Error::new(
                            attribute.span,
                            "duplicate component id assignments",
                        ));
                    }
                    component_id = Some(id.value);
                }
                _ => {
                    return Err(syn::Error::new(
                        attribute.span,
                        "this attribute is not supported here",
                    ))
                }
            }
        }

        Ok(Self {
            cfgs,
            id: component_id,
            name,
        })
    }
}

impl Parse for ParseAttributeCfg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args;
        parenthesized!(args in input);

        // Don't care about parsing the predicate contents
        let predicate = args.parse::<TokenStream>()?;

        Ok(Self { predicate })
    }
}

impl Parse for ParseAttributeId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args;
        parenthesized!(args in input);

        // Grab the int literal and make sure it's the right type
        let value = args.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { value })
    }
}

impl Parse for ParseAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![#]>()?;
        let content;
        bracketed!(content in input);

        let span = content.span();
        let lookahead = content.lookahead1();
        let data = if lookahead.peek(kw::cfg) {
            content.parse::<kw::cfg>()?;
            ParseAttributeData::Cfg(content.parse()?)
        } else if lookahead.peek(kw::archetype_id) {
            content.parse::<kw::archetype_id>()?;
            ParseAttributeData::ArchetypeId(content.parse()?)
        } else if lookahead.peek(kw::component_id) {
            content.parse::<kw::component_id>()?;
            ParseAttributeData::ComponentId(content.parse()?)
        } else {
            return Err(lookahead.error());
        };

        Ok(Self { span, data })
    }
}

impl Parse for ParseCapacity {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Dyn) {
            input.parse::<Dyn>()?;
            Ok(ParseCapacity::Dynamic)
        } else {
            input.parse().map(ParseCapacity::Expression)
        }
    }
}

impl HasAttributeId for ParseArchetype {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn id(&self) -> Option<u8> {
        self.id
    }
}

impl HasAttributeId for ParseComponent {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn id(&self) -> Option<u8> {
        self.id
    }
}

fn parse_attributes(input: ParseStream) -> syn::Result<Vec<ParseAttribute>> {
    let mut attrs = Vec::new();
    while input.peek(Token![#]) {
        attrs.push(input.parse()?);
    }
    Ok(attrs)
}

fn parse_item_archetype(
    input: ParseStream,
    attributes: Vec<ParseAttribute>,
) -> syn::Result<ParseItem> {
    let mut archetype = input.parse::<ParseArchetype>()?;

    for attribute in attributes.into_iter() {
        match attribute.data {
            ParseAttributeData::Cfg(cfg) => {
                // We need to collect all cfgs in the world body
                archetype.cfgs.push(cfg);
            }
            ParseAttributeData::ArchetypeId(id) => {
                if archetype.id.is_some() {
                    return Err(syn::Error::new(
                        attribute.span,
                        "duplicate archetype id assignments",
                    ));
                }
                archetype.id = Some(id.value);
            }
            _ => {
                return Err(syn::Error::new(
                    attribute.span,
                    "this attribute is not supported here",
                ));
            }
        }
    }

    Ok(ParseItem::ParseArchetype(archetype))
}

fn parse_item_name(
    input: ParseStream, //.
    attributes: Vec<ParseAttribute>,
) -> syn::Result<ParseItem> {
    if attributes.is_empty() == false {
        return Err(syn::Error::new(
            attributes[0].span,
            "this attribute is not supported here",
        ));
    }

    let name = input.parse::<ParseName>()?;
    Ok(ParseItem::ParseName(name))
}
