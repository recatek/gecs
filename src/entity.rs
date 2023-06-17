use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::NonZeroU32;

use crate::error::EcsError;
use crate::index::{DataIndex, MAX_DATA_INDEX};
use crate::traits::Archetype;

// NOTE: While this is extremely unlikely to change, if it does, the proc
// macros need to be updated manually with the new type assumptions.
pub type ArchetypeId = u8;

// How many bits of a u32 entity index are reserved for the archetype ID.
pub(crate) const ARCHETYPE_ID_BITS: u32 = ArchetypeId::BITS;

/// A statically typed handle to an entity of a specific archetype.
///
/// On its own, this key does very little. Its primary purpose is to provide
/// indexed access to component data within an ECS world and its archetypes.
/// Entity handles are opaque and can't be accessed beyond type information.
///
/// As a data structure, an entity has two parts -- a slot index and a
/// generational version number. The slot index is used by the archetype data
/// structure to find the entity's component data, and the version number is
/// used to safely avoid attempts to access data for a stale `Entity` handle.
pub struct Entity<A: Archetype> {
    inner: EntityAny,
    _type: PhantomData<fn() -> A>,
}

/// A dynamically typed handle to an entity of some runtime archetype.
///
/// This behaves like an [`Entity`] key, but its type is only known at runtime.
/// To determine its type, use `archetype_id()`, or use the `resolve()` method
/// generated by the `ecs_world!` declaration to convert the `EntityAny` into
/// an enum with each possible archetype outcome.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct EntityAny {
    data: u32, // [ index (u24) | archetype_id (u8) ]
    version: NonZeroU32,
}

impl<A: Archetype> Entity<A> {
    #[inline(always)]
    pub(crate) fn new(index: DataIndex, version: NonZeroU32) -> Self {
        Self {
            inner: EntityAny::new(index, A::ARCHETYPE_ID, version),
            _type: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) fn index(&self) -> DataIndex {
        self.inner.index()
    }

    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.inner.version()
    }

    /// Creates a new typed entity from an `EntityAny`.
    ///
    /// In match statements, this tends to optimize better than `TryFrom`.
    ///
    /// # Panics
    ///
    /// Panics if the given `EntityAny` is not an entity of this type.
    #[inline(always)]
    pub fn from_any(entity: EntityAny) -> Self {
        if entity.archetype_id() != A::ARCHETYPE_ID {
            panic!("invalid entity conversion");
        }

        Self {
            inner: entity,
            _type: PhantomData,
        }
    }

    /// Returns this entity's raw `ARCHETYPE_ID` value.
    ///
    /// This is the same `ARCHETYPE_ID` as the archetype this entity belongs to.
    #[inline(always)]
    pub const fn archetype_id(self) -> ArchetypeId {
        A::ARCHETYPE_ID
    }
}

impl EntityAny {
    #[inline(always)]
    pub(crate) fn new(index: DataIndex, archetype_id: ArchetypeId, version: NonZeroU32) -> Self {
        let archetype_id: u32 = archetype_id.into();
        let data = (index.get() << ARCHETYPE_ID_BITS) | archetype_id;
        Self { data, version }
    }

    #[inline(always)]
    pub(crate) fn index(&self) -> DataIndex {
        unsafe {
            // SAFETY: We know the remaining data can fit in a DataIndex
            debug_assert!(self.data >> ARCHETYPE_ID_BITS <= MAX_DATA_INDEX);
            DataIndex::new_unchecked(self.data >> ARCHETYPE_ID_BITS)
        }
    }

    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.version
    }

    /// Returns this entity's raw `ARCHETYPE_ID` value.
    ///
    /// This is the same `ARCHETYPE_ID` as the archetype this entity belongs to.
    #[inline(always)]
    pub fn archetype_id(self) -> ArchetypeId {
        self.data as ArchetypeId // Trim off the bottom to get the ID
    }
}

impl<A: Archetype> From<Entity<A>> for EntityAny {
    #[inline(always)]
    fn from(entity: Entity<A>) -> Self {
        entity.inner
    }
}

impl<A: Archetype> TryFrom<EntityAny> for Entity<A> {
    type Error = EcsError;

    #[inline(always)]
    fn try_from(entity: EntityAny) -> Result<Self, Self::Error> {
        if entity.archetype_id() == A::ARCHETYPE_ID {
            Ok(Self {
                inner: entity,
                _type: PhantomData,
            })
        } else {
            Err(EcsError::InvalidEntityType)
        }
    }
}

// PhantomData boilerplate until https://github.com/rust-lang/rust/issues/26925 is resolved

impl<A: Archetype> Clone for Entity<A> {
    #[inline(always)]
    fn clone(&self) -> Entity<A> {
        Entity {
            inner: self.inner,
            _type: PhantomData,
        }
    }
}

impl<A: Archetype> Copy for Entity<A> {}

impl<A: Archetype> PartialEq for Entity<A> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<A: Archetype> Eq for Entity<A> {}

impl<A: Archetype> Hash for Entity<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
