use std::collections::HashMap;
use std::num::NonZeroU8;

use base64::Engine as _;
use speedy::{Readable, Writable};

use crate::parse::{ParseArchetype, ParseAttributeCfg, ParseCapacity, ParseFinalize};

#[derive(Debug, Readable, Writable)]
pub struct DataWorld {
    pub name: String,
    pub archetypes: Vec<DataArchetype>,
}

#[derive(Debug, Readable, Writable)]
pub struct DataArchetype {
    pub id: NonZeroU8,
    pub name: String,
    pub capacity: DataCapacity,
    pub components: Vec<DataComponent>,
}

#[derive(Debug, Readable, Writable)]
pub struct DataComponent {
    pub name: String,
}

#[derive(Debug, Readable, Writable)]
#[speedy(tag_type = u8)]
pub enum DataCapacity {
    Literal(usize),
    Constant(String),
    Dynamic,
}

impl DataWorld {
    pub fn new(mut parse: ParseFinalize) -> syn::Result<Self> {
        let cfg_data = parse.cfg_lookup;

        let mut archetypes = Vec::new();
        let mut archetype_ids = HashMap::new();
        let mut last_archetype_id = None;

        for mut archetype in parse.world.archetypes.drain(..) {
            if evaluate_cfgs(&cfg_data, &archetype.cfgs) == false {
                continue;
            }

            // Advance the archetype ID, either implicitly or from the ID attribute
            last_archetype_id = advance_archetype_id(
                &archetype, //.
                &mut archetype_ids,
                last_archetype_id,
            )?;

            let mut components = Vec::new();
            for component in archetype.components.drain(..) {
                if evaluate_cfgs(&cfg_data, &component.cfgs) == false {
                    continue;
                }

                components.push(DataComponent {
                    name: component.name.to_string(),
                });
            }

            archetypes.push(DataArchetype {
                id: last_archetype_id.unwrap(),
                name: archetype.name.to_string(),
                capacity: convert_capacity(archetype.capacity),
                components,
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

fn convert_capacity(capacity: ParseCapacity) -> DataCapacity {
    match capacity {
        ParseCapacity::Literal(lit) => DataCapacity::Literal(lit.base10_parse::<usize>().unwrap()),
        ParseCapacity::Constant(ident) => DataCapacity::Constant(ident.to_string()),
        ParseCapacity::Dynamic => DataCapacity::Dynamic,
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

fn advance_archetype_id(
    archetype: &ParseArchetype,
    ids: &mut HashMap<NonZeroU8, String>,
    last: Option<NonZeroU8>,
) -> syn::Result<Option<NonZeroU8>> {
    let next = if let Some(archetype_id) = archetype.id {
        Some(archetype_id)
    } else {
        match last {
            Some(last) => last.checked_add(1),
            None => NonZeroU8::new(1), // Start from 1
        }
    };

    // We can't have an archetype ID of 0
    if let Some(next) = next {
        if let Some(name) = ids.insert(next, archetype.name.to_string()) {
            Err(syn::Error::new(
                archetype.name.span(),
                format!(
                    "archetype id {} is already assigned to {}{}",
                    next,
                    name,
                    match archetype.id {
                        Some(_) => "",
                        None => " (note: ids are automatically assigned in sequence)",
                    },
                ),
            ))
        } else {
            // We have a valid, unused archetype ID
            Ok(Some(next))
        }
    } else {
        Err(syn::Error::new(
            archetype.name.span(),
            format!("archetype id must be between 1 and 255"),
        ))
    }
}
