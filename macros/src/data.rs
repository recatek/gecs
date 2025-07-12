use std::collections::HashMap;
use std::fmt::Display;

use base64::Engine as _;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use speedy::{Readable, Writable};
use syn::{self, Ident, LitInt};

use crate::util;

use crate::parse::{
    HasAttributeId, //.
    ParseAttributeCfg,
    ParseCfgDecorated,
    ParseComponentGeneric,
    ParseComponentName,
    ParseEcsWorld,
};

#[derive(Debug, Readable, Writable)]
pub struct DataWorld {
    pub name: String,
    pub archetypes: Vec<DataArchetype>,
}

#[derive(Debug, Readable, Writable)]
pub struct DataArchetype {
    pub id: u8,
    pub name: String,
    pub components: Vec<DataComponent>,
}

#[derive(Debug, Readable, Writable)]
pub struct DataComponent {
    pub id: u8,
    pub name: DataComponentName,
}

#[derive(Debug, Readable, Writable)]
pub struct DataComponentName {
    pub name: String,
    pub generic: Option<DataComponentGeneric>,
}

#[derive(Debug, Readable, Writable)]
pub enum DataComponentGeneric {
    Placeholder,
    Ident(String),
    LitInt(String),
}

impl DataWorld {
    pub fn new(mut parse: ParseCfgDecorated<ParseEcsWorld>) -> syn::Result<Self> {
        let cfg_lookup = parse.cfg_lookup;

        let mut archetypes = Vec::new();
        let mut archetype_ids = HashMap::new();
        let mut last_archetype_id = None;

        for mut archetype in parse.inner.archetypes.drain(..) {
            if evaluate_cfgs(&cfg_lookup, &archetype.cfgs) == false {
                continue;
            }

            // Advance the archetype ID, either implicitly or from the ID attribute
            last_archetype_id = advance_attribute_id(
                &archetype, //.
                &mut archetype_ids,
                last_archetype_id,
            )?;

            let mut component_ids = HashMap::new();
            let mut last_component_id = None;

            let mut components = Vec::new();
            for component in archetype.components.drain(..) {
                if evaluate_cfgs(&cfg_lookup, &component.cfgs) == false {
                    continue;
                }

                // Advance the component ID, either implicitly or from the ID attribute
                last_component_id = advance_attribute_id(
                    &component, //.
                    &mut component_ids,
                    last_component_id,
                )?;

                components.push(DataComponent {
                    id: last_component_id.unwrap(), // TODO
                    name: DataComponentName::new(&component.name),
                });
            }

            archetypes.push(DataArchetype {
                id: last_archetype_id.unwrap(),
                name: archetype.name.to_string(),
                components,
            })
        }

        Ok(DataWorld {
            name: parse.inner.name.to_string(),
            archetypes,
        })
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD_NO_PAD
            .encode(self.write_to_vec().expect("failed to serialize world"))
    }

    pub fn from_base64(base64: &str) -> Self {
        Self::read_from_buffer(
            &base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(base64)
                .expect("failed to deserialize world"),
        )
        .expect("failed to deserialize world")
    }
}

impl DataArchetype {
    pub fn try_bind_component(
        &self,
        name: &ParseComponentName,
    ) -> syn::Result<Option<ParseComponentName>> {
        let mut found = None;

        for component in self.components.iter() {
            if component.name.matches_with_placeholder(name) {
                if let Some(found) = found {
                    return Err(syn::Error::new(
                        name.span(),
                        format!(
                            "Component parameter is ambiguous for {}, matching both {} and {}",
                            self.name, found, component.name,
                        ),
                    ));
                }

                found = Some(component.name.as_parse());
            }
        }

        Ok(found)
    }
}

impl DataComponentName {
    pub fn new(name: &ParseComponentName) -> Self {
        use DataComponentGeneric as D;
        use ParseComponentGeneric as P;

        DataComponentName {
            name: name.name.to_string(),
            generic: name.generic.as_ref().map(|generic| match generic {
                P::Placeholder(_) => D::Placeholder,
                P::Ident(ident) => D::Ident(ident.to_string()),
                P::LitInt(lit) => D::LitInt(lit.token().to_string()),
            }),
        }
    }

    pub fn as_parse(&self) -> ParseComponentName {
        use DataComponentGeneric as D;
        use ParseComponentGeneric as P;

        let name = Ident::new(&self.name, Span::call_site());
        let generic = match &self.generic {
            None => None,
            Some(D::Placeholder) => panic!("placeholder type not allowed in this conversion"),
            Some(D::Ident(ident)) => Some(P::Ident(Ident::new(ident, Span::call_site()))),
            Some(D::LitInt(lit)) => Some(P::LitInt(LitInt::new(lit, Span::call_site()))),
        };

        ParseComponentName { name, generic }
    }

    pub fn as_snake_name(&self) -> String {
        use DataComponentGeneric as D;

        match &self.generic {
            None => format!("{}", util::to_snake(&self.name)),
            Some(D::Ident(ident)) => {
                format!(
                    "{}_{}", //.
                    util::to_snake(&self.name),
                    util::to_snake(&ident)
                )
            }
            Some(D::LitInt(lit)) => {
                format!(
                    "{}_{}", //.
                    util::to_snake(&self.name),
                    util::to_snake(&lit)
                )
            }
            Some(D::Placeholder) => panic!("placeholder type invalid as name"),
        }
    }

    pub fn matches_with_placeholder(&self, name: &ParseComponentName) -> bool {
        use DataComponentGeneric as D;
        use ParseComponentGeneric as P;

        if name.name.to_string() != self.name {
            return false;
        }

        match (name.generic.as_ref(), self.generic.as_ref()) {
            (None, None) => true,                       // Neither is generic
            (Some(P::Placeholder(_)), Some(_)) => true, // Placeholder matches any generic
            (Some(P::Ident(a)), Some(D::Ident(b))) => a.to_string() == b.as_str(),
            (Some(P::LitInt(a)), Some(D::LitInt(b))) => a.token().to_string() == b.as_str(),
            _ => false,
        }
    }
}

impl ToTokens for DataComponentName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_parse().to_tokens(tokens)
    }
}

impl Display for DataComponentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DataComponentGeneric as D;

        match &self.generic {
            None => write!(f, "{}", self.name),
            Some(D::Placeholder) => panic!("placeholder type not allowed in string conversion"),
            Some(D::Ident(ident)) => write!(f, "{}<{}>", self.name, ident),
            Some(D::LitInt(lit)) => write!(f, "{}<{}>", self.name, lit),
        }
    }
}

fn evaluate_cfgs(cfg_lookup: &HashMap<String, bool>, cfgs: &[ParseAttributeCfg]) -> bool {
    for cfg in cfgs {
        let predicate = cfg.predicate.to_string();
        if *cfg_lookup.get(&predicate).unwrap() == false {
            return false;
        }
    }
    true
}

fn advance_attribute_id(
    item: &impl HasAttributeId,
    ids: &mut HashMap<u8, String>,
    last: Option<u8>,
) -> syn::Result<Option<u8>> {
    let next = {
        if let Some(archetype_id) = item.id() {
            Ok(archetype_id)
        } else if let Some(last) = last {
            if let Some(next) = last.checked_add(1) {
                Ok(next)
            } else {
                let span = item.span();
                Err(syn::Error::new(span, "attribute id may not exceed 255"))
            }
        } else {
            Ok(0) // Start counting from 0
        }
    }?;

    // We can't have an archetype ID of 0
    if let Some(name) = ids.insert(next, item.name_to_string()) {
        Err(syn::Error::new(
            item.span(),
            format!("attribute id {} is already assigned to {}", next, name,),
        ))
    } else {
        // We have a valid, unused archetype ID
        Ok(Some(next))
    }
}
