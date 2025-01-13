use crate::entity::EntityAny;

#[cfg(doc)]
use crate::traits::{Archetype, World};

/// A struct reporting ordered entity creation and destruction events for archetypes and worlds.
/// See [`World::drain_events`] and [`Archetype::drain_events`] for more information.
#[cfg(feature = "events")]
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EcsEvent {
    Created(EntityAny),
    Destroyed(EntityAny),
}
