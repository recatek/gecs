use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroU8};

use crate::error::EcsError;
use crate::traits::Archetype;

pub(crate) const MAX_ARCHETYPE_CAPACITY: usize = (1 << TYPE_SHIFT) as usize;

const TYPE_BITS: u32 = 8;
const TYPE_SHIFT: u32 = u32::BITS - TYPE_BITS;

pub struct Entity<A: Archetype> {
    inner: EntityAny,
    _type: PhantomData<A>,
}

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

    /// Creates a new typed entity from an EcsEntityAny. In match statements,
    /// this function is easier for the compiler to optimize than try_from().
    ///
    /// # Panics
    ///
    /// Panics if the given EcsEntityAny is not an entity of this type.
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

    /// Returns this entity's raw type_id value.
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

    /// Returns this entity's raw type_id value.
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
