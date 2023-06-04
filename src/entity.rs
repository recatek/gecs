use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroU8};

use crate::error::EcsError;
use crate::traits::Archetype;

pub(crate) const MAX_ARCHETYPE_CAPACITY: usize = (1 << TYPE_SHIFT) as usize;

const TYPE_BITS: u32 = 8;
const TYPE_SHIFT: u32 = u32::BITS - TYPE_BITS;

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
    _type: PhantomData<A>,
}

/// A dynamically typed handle to an entity of some runtime archetype.
///
/// This behaves like an [`Entity`] key, but its type is only known at runtime.
/// To determine its type, use the `type_id()` function, or use the `resolve()`
/// method generated by the `ecs_world!` declaration to convert the `EntityAny`
/// into an enum with each possible archetype outcome.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct EntityAny {
    data: u32, // [ index (u24) | type_id (NonZeroU8) ]
    version: NonZeroU32,
}

impl<A: Archetype> Entity<A> {
    #[inline(always)]
    pub(crate) fn new(index: u32, version: NonZeroU32) -> Self {
        Self {
            inner: EntityAny::new(index, A::TYPE_ID, version),
            _type: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) fn index(&self) -> u32 {
        self.inner.index()
    }

    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.inner.version()
    }

    /// Creates a new typed entity from an `EntityAny`.
    ///
    /// In match statements, this may optimize better than `TryFrom`.
    ///
    /// # Panics
    ///
    /// Panics if the given `EntityAny` is not an entity of this type.
    #[inline(always)]
    pub fn from_any(entity: EntityAny) -> Self {
        if entity.type_id() != A::TYPE_ID {
            panic!("invalid entity conversion");
        }

        Self {
            inner: entity,
            _type: PhantomData,
        }
    }

    /// Returns this entity's raw `TYPE_ID` value.
    ///
    /// This is the same `TYPE_ID` as the archetype this entity belongs to.
    #[inline(always)]
    pub const fn type_id(self) -> NonZeroU8 {
        A::TYPE_ID
    }
}

impl EntityAny {
    #[inline(always)]
    pub(crate) fn new(index: u32, type_id: NonZeroU8, version: NonZeroU32) -> Self {
        if index >= MAX_ARCHETYPE_CAPACITY.try_into().unwrap() {
            panic!("index too large for handle");
        }

        let type_id: u32 = type_id.get().into();
        let data = (index << TYPE_BITS) | type_id;

        Self { data, version }
    }

    #[inline(always)]
    pub(crate) fn index(&self) -> u32 {
        self.data >> TYPE_BITS
    }

    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.version
    }

    /// Returns this entity's raw `TYPE_ID` value.
    ///
    /// This is the same `TYPE_ID` as the archetype this entity belongs to.
    #[inline(always)]
    pub fn type_id(self) -> NonZeroU8 {
        debug_assert!(self.data as u8 != 0, "invalid type_id");
        // SAFETY: We can only be created with a NonZeroU8 type_id
        unsafe { NonZeroU8::new_unchecked(self.data as u8) }
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
        if entity.type_id() == A::TYPE_ID {
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
