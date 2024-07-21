use std::collections::HashMap;

use base64::Engine as _;
use speedy::{Readable, Writable};
use syn::Ident;

use crate::parse::{HasAttributeId, ParseAttributeCfg, ParseEcsFinalize};

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

    // This data is only used for building the world and is not needed in queries
    #[speedy(skip)]
    pub build_data: Option<DataArchetypeBuildOnly>,
}

#[derive(Debug, Readable, Writable)]
pub struct DataComponent {
    pub id: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct DataArchetypeBuildOnly;

impl DataWorld {
    pub fn new(mut parse: ParseEcsFinalize) -> syn::Result<Self> {
        let cfg_data = parse.cfg_lookup;

        let mut archetypes = Vec::new();
        let mut archetype_ids = HashMap::new();
        let mut last_archetype_id = None;

        for mut archetype in parse.world.archetypes.drain(..) {
            if evaluate_cfgs(&cfg_data, &archetype.cfgs) == false {
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
                if evaluate_cfgs(&cfg_data, &component.cfgs) == false {
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
                    name: component.name.to_string(),
                });
            }

            let build_data = DataArchetypeBuildOnly;

            archetypes.push(DataArchetype {
                id: last_archetype_id.unwrap(),
                name: archetype.name.to_string(),
                components,
                build_data: Some(build_data),
            })
        }

        Ok(DataWorld {
            name: parse.world.name.to_string(),
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
    pub fn contains_component(&self, name: &Ident) -> bool {
        for component in self.components.iter() {
            if component.name == name.to_string() {
                return true;
            }
        }
        false
    }
}

fn evaluate_cfgs(cfg_data: &HashMap<String, bool>, cfgs: &[ParseAttributeCfg]) -> bool {
    for cfg in cfgs {
        let predicate = cfg.predicate.to_string();
        if *cfg_data.get(&predicate).unwrap() == false {
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
                let span = item.name().span();
                Err(syn::Error::new(span, "attribute id may not exceed 255"))
            }
        } else {
            Ok(0) // Start counting from 0
        }
    }?;

    // We can't have an archetype ID of 0
    if let Some(name) = ids.insert(next, item.name().to_string()) {
        Err(syn::Error::new(
            item.name().span(),
            format!("attribute id {} is already assigned to {}", next, name,),
        ))
    } else {
        // We have a valid, unused archetype ID
        Ok(Some(next))
    }
}
