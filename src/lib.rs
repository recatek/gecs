#![allow(clippy::bool_comparison)]
#![allow(clippy::type_complexity)] // lol
#![allow(clippy::too_many_arguments)] // lmao

mod archetype;
mod util;

pub mod entity;
pub mod error;
pub mod traits;

pub mod prelude {
    use super::*;

    pub use gecs_macros::ecs_world;

    pub use entity::{Entity, EntityAny};
    pub use traits::Archetype;
    pub use traits::{ArchetypeContainer, ComponentContainer};
    pub use traits::{HasArchetype, HasComponent};
}

#[doc(hidden)]
pub mod __internal {
    use super::*;

    pub use gecs_macros::__ecs_finalize;
    pub use gecs_macros::{__ecs_find_borrow, __ecs_find_mut};
    pub use gecs_macros::{__ecs_iter_borrow, __ecs_iter_mut};

    pub use archetype::slices::*;
    pub use archetype::storage_dynamic::*;
    pub use archetype::storage_fixed::*;

    pub use traits::Archetype;
    pub use traits::{ArchetypeContainer, ComponentContainer};
    pub use traits::{HasArchetype, HasComponent};
}
