use std::num::NonZeroU8;

pub trait Archetype: Sized {
    const TYPE_ID: NonZeroU8;
}

pub trait HasArchetype<A: Archetype>: Sized {}
