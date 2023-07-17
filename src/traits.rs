use std::cell::{Ref, RefMut};

use crate::entity::{ArchetypeId, Entity, EntityRaw};

/// A set of helper functions for accessing archetypes from a world via turbofish.
pub trait World: Sized {
    /// Creates a new entity with the given components in the archetype, if there's room.
    ///
    /// Returns a handle for accessing the new entity.
    ///
    /// # Panics
    ///
    /// Panics if the archetype is full. For a panic-free version, use `try_create`.
    #[inline(always)]
    fn create<A: Archetype>(&mut self, components: A::Components) -> Entity<A>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_create(self, components)
    }

    /// Creates a new entity with the given components in the archetype, if there's room.
    ///
    /// Returns a handle for accessing the new entity, or `None` if the archetype is full.
    #[inline(always)]
    fn try_create<A: Archetype>(&mut self, components: A::Components) -> Option<Entity<A>>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_try_create(self, components)
    }

    /// If the entity exists in the archetype, this destroys it and returns its components.
    #[inline(always)]
    fn destroy<K: EntityKey>(&mut self, entity: K) -> K::DestroyOutput
    where
        Self: WorldCanResolve<K>,
    {
        <Self as WorldCanResolve<K>>::resolve_destroy(self, entity)
    }

    /// Gets a reference to the archetype of the given type from the world.
    #[inline(always)]
    fn archetype<A: Archetype>(&self) -> &A
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_archetype(self)
    }

    /// Gets a mutable reference to the archetype of the given type from the world.
    #[inline(always)]
    fn archetype_mut<A: Archetype>(&mut self) -> &mut A
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_archetype_mut(self)
    }
}

/// A trait describing each archetype in a given ECS world.
///
/// This can be used in generic functions to get type and component information.
///
/// The `Archetype` trait should be used only by the `ecs_world!` macro --
/// it is not intended to be manually implemented by any user data structures.
pub trait Archetype
where
    Self: Sized,
    for<'a> Self: ArchetypeCanResolve<'a, Self::View<'a>, Entity<Self>>,
    for<'a> Self: ArchetypeCanResolve<'a, Self::View<'a>, EntityRaw<Self>>,
{
    /// A unique type ID assigned to this archetype in generation.
    const ARCHETYPE_ID: ArchetypeId;

    /// A tuple of the components in this archetype.
    type Components;
    /// The slices type when accessing all of this archetype's slices simultaneously.
    type Slices<'a>;
    /// The view type when accessing a single entity's components simultaneously.
    type View<'a>: View
    where
        Self: 'a;

    #[doc(hidden)]
    fn get_slice_entities(&self) -> &[Entity<Self>];

    /// Returns a view containing mutable references to all of this entity's components.
    #[inline(always)]
    fn view<'a, K: EntityKey>(&'a mut self, entity: K) -> Option<Self::View<'a>>
    where
        Self: ArchetypeCanResolve<'a, Self::View<'a>, K>,
    {
        <Self as ArchetypeCanResolve<Self::View<'a>, K>>::resolve_view(self, entity)
    }

    /// Gets the given slice of components from the archetype's dense data.
    ///
    /// This requires mutable access to the archetype to bypass runtime borrow checks.
    #[inline(always)]
    fn get_slice<C>(&mut self) -> &[C]
    where
        Self: ArchetypeHas<C>,
    {
        <Self as ArchetypeHas<C>>::resolve_get_slice(self)
    }

    /// Gets the given mutable slice of components from the archetype's dense data.
    ///
    /// This requires mutable access to the archetype to bypass runtime borrow checks.
    #[inline(always)]
    fn get_slice_mut<C>(&mut self) -> &mut [C]
    where
        Self: ArchetypeHas<C>,
    {
        <Self as ArchetypeHas<C>>::resolve_get_slice_mut(self)
    }

    /// Borrows the given slice of components from the archetype's dense data.
    ///
    /// This performs a runtime borrow check.
    ///
    /// # Panics
    ///
    /// Panics if the runtime borrow fails, see [`std::cell::RefCell::borrow`].
    #[inline(always)]
    fn borrow_slice<C>(&self) -> Ref<[C]>
    where
        Self: ArchetypeHas<C>,
    {
        <Self as ArchetypeHas<C>>::resolve_borrow_slice(self)
    }

    /// Borrows the given mutable slice of components from the archetype's dense data.
    ///
    /// This performs a runtime borrow check.
    ///
    /// # Panics
    ///
    /// Panics if the runtime borrow fails, see [`std::cell::RefCell::borrow_mut`].
    #[inline(always)]
    fn borrow_slice_mut<C>(&self) -> RefMut<[C]>
    where
        Self: ArchetypeHas<C>,
    {
        <Self as ArchetypeHas<C>>::resolve_borrow_slice_mut(self)
    }
}

/// A trait promising that an archetype container (i.e. world) has an archetype.
///
/// Used for functions that take an ECS world as a generic type.
///
/// See [`World`] for the methods that this enables on a type.
///
/// Note that macros like `ecs_iter!` do not currently support these kinds of generics.
/// This is primarily an advanced feature as it requires manual ECS manipulation.
///
/// # Example
///
/// ```
/// use gecs::prelude::*;
///
/// pub struct CompA(pub u32);
///
/// ecs_world! {
///     // Declare archetype ArchFoo with capacity 100 and one component: CompA
///     ecs_archetype!(ArchFoo, 100, CompA);
/// }
///
/// fn get_arch_foo_len<W>(world: &mut W) -> usize
/// where
///     W: WorldHas<ArchFoo>,
/// {
///     world.archetype::<ArchFoo>().len()
/// }
///
/// # fn main() {} // Not actually running anything here
/// ```
pub trait WorldHas<A: Archetype>: World {
    #[doc(hidden)]
    fn resolve_create(&mut self, data: A::Components) -> Entity<A>;
    #[doc(hidden)]
    fn resolve_try_create(&mut self, data: A::Components) -> Option<Entity<A>>;
    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: Entity<A>) -> Option<A::Components>;

    #[doc(hidden)]
    fn resolve_archetype(&self) -> &A;
    #[doc(hidden)]
    fn resolve_archetype_mut(&mut self) -> &mut A;
}

/// A trait promising that an component container (i.e. archetype) has a component.
///
/// Used for functions that take an ECS world or archetype as a generic type.
///
/// See [`ComponentContainer`] for the methods that this enables on a type.
///
/// Note that macros like `ecs_iter!` do not currently support these kinds of generics.
/// This is primarily an advanced feature as it requires manual ECS manipulation.
///
/// # Example
///
/// ```
/// use gecs::prelude::*;
///
/// pub struct CompA(pub u32);
///
/// ecs_world! {
///     // Declare archetype ArchFoo with capacity 100 and one component: CompA
///     ecs_archetype!(ArchFoo, 100, CompA);
/// }
///
/// fn sum_comp_a<A>(archetype: &mut A) -> u32
/// where
///     A: ArchetypeHas<CompA>,
/// {
///     let mut sum = 0;
///
///     for component in archetype.get_slice::<CompA>().iter() {
///         sum += component.0;
///     }
///
///     sum
/// }
///
/// # fn main() {} // Not actually running anything here
/// ```
pub trait ArchetypeHas<C>: Archetype {
    #[doc(hidden)]
    fn resolve_get_slice(&mut self) -> &[C];
    #[doc(hidden)]
    fn resolve_get_slice_mut(&mut self) -> &mut [C];
    #[doc(hidden)]
    fn resolve_borrow_slice(&self) -> Ref<[C]>;
    #[doc(hidden)]
    fn resolve_borrow_slice_mut(&self) -> RefMut<[C]>;
}

pub trait View {
    #[inline(always)]
    fn component<C>(&self) -> &C
    where
        Self: ViewHas<C>,
    {
        <Self as ViewHas<C>>::resolve_component(self)
    }

    #[inline(always)]
    fn component_mut<C>(&mut self) -> &mut C
    where
        Self: ViewHas<C>,
    {
        <Self as ViewHas<C>>::resolve_component_mut(self)
    }
}

pub trait ViewHas<C> {
    #[doc(hidden)]
    fn resolve_component(&self) -> &C;
    #[doc(hidden)]
    fn resolve_component_mut(&mut self) -> &mut C;
}

pub trait CanBorrow<T> {
    #[doc(hidden)]
    fn resolve_borrow(&self) -> Ref<T>;
    #[doc(hidden)]
    fn resolve_borrow_mut(&self) -> RefMut<T>;
}

pub trait EntityKey {
    type DestroyOutput;
}

pub trait WorldCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: K) -> K::DestroyOutput;
}

pub trait ArchetypeCanResolve<'a, View, K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;
    #[doc(hidden)]
    fn resolve_view(&'a mut self, entity: K) -> Option<View>;
}

pub trait StorageCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;
}
