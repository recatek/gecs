use std::collections::HashMap;

use crate::parse::{ParseCfg, ParseFinalize};

#[derive(Debug)]
pub struct World {
    pub archetypes: Vec<Archetype>,
}

#[derive(Debug)]
pub struct Archetype {
    pub name: String,
    pub components: Vec<Component>,
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
}

impl World {
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

                components.push(Component {
                    name: component.name.to_string(),
                });
            }

            archetypes.push(Archetype {
                name: archetype.name.to_string(),
                components,
            })
        }

        World { archetypes }
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
