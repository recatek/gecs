#![allow(clippy::bool_comparison)]
#![allow(clippy::type_complexity)] // lol
#![allow(clippy::too_many_arguments)] // lmao

mod archetype;
mod entity;
mod error;
mod traits;
mod util;

pub mod prelude {
    use super::*;

    pub use entity::{Entity, EntityAny};
    pub use gecs_macros::ecs_world;
    pub use traits::Archetype;
}

pub mod __internal {
    use super::*;

    pub use archetype::slices::*;
    pub use archetype::storage_fixed::*;
    pub use gecs_macros::__ecs_finalize;
}
