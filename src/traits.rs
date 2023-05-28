use std::cell::{Ref, RefMut};
use std::num::NonZeroU8;

use crate::entity::Entity;

pub trait Archetype: Sized {
    const TYPE_ID: NonZeroU8;

    fn get_slice_entities(&self) -> &[Entity<Self>];
}

pub trait HasArchetypes: Sized {
    fn get_archetype<A: Archetype>(&self) -> &A
    where
        Self: HasArchetype<A>,
    {
        <Self as HasArchetype<A>>::resolve_archetype(self)
    }

    fn get_mut_archetype<A: Archetype>(&mut self) -> &mut A
    where
        Self: HasArchetype<A>,
    {
        <Self as HasArchetype<A>>::resolve_mut_archetype(self)
    }
}

pub trait HasComponents: Archetype {
    fn get_slice<C>(&mut self) -> &[C]
    where
        Self: HasComponent<C>,
    {
        <Self as HasComponent<C>>::resolve_get_slice(self)
    }

    fn get_mut_slice<C>(&mut self) -> &mut [C]
    where
        Self: HasComponent<C>,
    {
        <Self as HasComponent<C>>::resolve_get_mut_slice(self)
    }

    fn borrow_slice<C>(&self) -> Ref<[C]>
    where
        Self: HasComponent<C>,
    {
        <Self as HasComponent<C>>::resolve_borrow_slice(self)
    }

    fn borrow_mut_slice<C>(&self) -> RefMut<[C]>
    where
        Self: HasComponent<C>,
    {
        <Self as HasComponent<C>>::resolve_borrow_mut_slice(self)
    }
}

pub trait HasArchetype<A: Archetype>: HasArchetypes {
    fn resolve_archetype(&self) -> &A;
    fn resolve_mut_archetype(&mut self) -> &mut A;
}

pub trait HasComponent<C>: HasComponents {
    fn resolve_get_slice(&mut self) -> &[C];
    fn resolve_get_mut_slice(&mut self) -> &mut [C];
    fn resolve_borrow_slice(&self) -> Ref<[C]>;
    fn resolve_borrow_mut_slice(&self) -> RefMut<[C]>;
}
