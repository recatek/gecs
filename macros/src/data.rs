use std::collections::HashMap;

use crate::parse::{ParseCapacity, ParseCfg, ParseFinalize};

#[derive(Debug)]
pub struct DataWorld {
    pub name: String,
    pub archetypes: Vec<DataArchetype>,
}

#[derive(Debug)]
pub struct DataArchetype {
    pub name: String,
    pub capacity: DataCapacity,
    pub components: Vec<DataComponent>,
}

#[derive(Debug)]
pub struct DataComponent {
    pub name: String,
}

#[derive(Debug)]
pub enum DataCapacity {
    Literal(usize),
    Constant(String),
    Dynamic,
}

impl DataWorld {
    pub fn new(mut parse: ParseFinalize) -> Self {
        let cfg_data = parse.cfg_lookup;
        let mut archetypes = Vec::new();

        for mut archetype in parse.world.archetypes.drain(..) {
            if evaluate_cfgs(&cfg_data, &archetype.cfgs) == false {
                continue;
            }

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
                name: archetype.name.to_string(),
                capacity: convert_capacity(archetype.capacity),
                components,
            })
        }

        DataWorld {
            name: "World".to_string(), // TODO: Allow this to be configured
            archetypes,
        }
    }
}

fn convert_capacity(capacity: ParseCapacity) -> DataCapacity {
    match capacity {
        ParseCapacity::Literal(lit) => DataCapacity::Literal(lit.base10_parse::<usize>().unwrap()),
        ParseCapacity::Constant(ident) => DataCapacity::Constant(ident.to_string()),
        ParseCapacity::Dynamic => DataCapacity::Dynamic,
    }
}

fn evaluate_cfgs(cfg_data: &HashMap<String, bool>, cfgs: &[ParseCfg]) -> bool {
    for cfg in cfgs {
        let predicate = cfg.predicate.to_string();
        if *cfg_data.get(&predicate).unwrap() == false {
            return false;
        }
    }
    true
}
