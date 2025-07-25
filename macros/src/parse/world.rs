use std::collections::HashSet;

use super::*;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Semi};
use syn::{parenthesized, Expr, Ident, Token};

mod kw {
    syn::custom_keyword!(cfg);

    syn::custom_keyword!(archetype_id);
    syn::custom_keyword!(component_id);

    syn::custom_keyword!(ecs_archetype);
    syn::custom_keyword!(ecs_name);
}

pub trait HasAttributeId {
    fn name_to_string(&self) -> String;
    fn span(&self) -> Span;
    fn id(&self) -> Option<u8>;
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
    pub components: Vec<ParseComponent>,
}

#[derive(Debug)]
pub struct ParseComponent {
    pub cfgs: Vec<ParseAttributeCfg>,
    pub id: Option<u8>,
    pub name: ParseComponentName,
    pub default: Option<Expr>,
}

impl HasCfgPredicates for ParseEcsWorld {
    fn collect_all_cfg_predicates(&self) -> Vec<TokenStream> {
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

impl Parse for ParseEcsWorld {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::<ParseItem, Semi>::parse_terminated(input)?
            .into_iter()
            .collect::<Vec<_>>();

        let mut name = format_ident!("EcsWorld");
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

        if archetypes.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "ecs world must have at least one archetype",
            ));
        }

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

        let components: Vec<ParseComponent> =
            Punctuated::<ParseComponent, Comma>::parse_terminated(&content)?
                .into_iter()
                .collect();

        // TODO: Check for duplicates?

        Ok(Self {
            cfgs,
            id: None,
            name,
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

        let name = input.parse::<ParseComponentName>()?;

        if let Some(ParseComponentGeneric::Placeholder(placeholder)) = name.generic {
            return Err(syn::Error::new(
                placeholder.span(),
                "placeholder not supported here",
            ));
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
            default: None, // TODO (default_field_values)
        })
    }
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
