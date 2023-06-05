mod query;
mod util;
mod world;

pub use query::*;
pub use util::*;
pub use world::*;

fn is_allowed_component_name(name: &str) -> bool {
    match name {
        "Entity" => false,
        "EntityAny" => false,
        "OneOf" => false,
        "Option" => false,
        _ => true,
    }
}
