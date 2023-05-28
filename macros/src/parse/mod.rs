mod query;
mod world;

pub use query::*;
pub use world::*;

fn is_allowed_component_name(name: &str) -> bool {
    match name {
        "Entity" => false,
        "EntityAny" => false,
        "AnyOf" => false,
        "Option" => false,
        _ => true,
    }
}
