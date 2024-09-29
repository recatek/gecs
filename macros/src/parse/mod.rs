mod attribute;
mod cfg;
mod query;
mod util;
mod world;

pub use attribute::*;
pub use cfg::*;
pub use query::*;
pub use util::*;
pub use world::*;

fn is_allowed_component_name(name: &str) -> bool {
    match name {
        "Entity" => false,
        "EntityAny" => false,
        "OneOf" => false,
        "AnyOf" => false,  // Reserved
        "Option" => false, // Reserved
        _ => true,
    }
}
