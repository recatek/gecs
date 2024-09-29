use std::cell::{Ref, RefMut};

use crate::entity::{ArchetypeId, Entity, EntityRaw};

/// The base trait for an ECS world in gecs.
///
/// This can be used in generic functions to access archetypes or create/destroy entities.
///
/// The `World` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
pub trait World: Sized {
    /// Creates a new entity with the given components to this archetype storage.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// # Panics
    ///
    /// Panics if the archetype can no longer expand to accommodate the new data.
    #[inline(always)]
    fn create<A: Archetype>(
        &mut self, //.
        components: A::Components,
    ) -> Entity<A>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_create(self, components)
    }

    /// Creates a new entity if there is sufficient spare capacity to store it.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// Unlike `create` this method will not reallocate when there is insufficient
    /// capacity. Instead, it will return an error along with given components.
    #[inline(always)]
    fn create_within_capacity<A: Archetype>(
        &mut self, //.
        components: A::Components,
    ) -> Result<Entity<A>, A::Components>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_create_within_capacity(self, components)
    }

    /// If the entity exists in the archetype, this destroys it.
    ///
    /// This can be called with either `EntityAny` or `Entity<A>` (for some archetype `A`).
    ///
    /// # Returns
    ///
    /// If called on an `EntityAny`, this will return a `bool` -- `true` if the entity
    /// found and was destroyed, or false otherwise.
    ///
    /// If called on an `Entity<A>`, this will return an `Option<(C0, C1, ... CN)>` of the
    /// destroyed entity's components if it was found and destroyed.
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
/// The `Archetype` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
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
    /// The borrow type when performing sequential borrows of an entity's components.
    type Borrow<'a>;
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

/// A trait promising that an ECS world has the given archetype.
///
/// Used for where bounds on functions that take an ECS world as a generic type.
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
///     // Declare archetype ArchFoo with one component: CompA
///     ecs_archetype!(ArchFoo, CompA);
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
    fn resolve_create(
        &mut self, //.
        data: A::Components,
    ) -> Entity<A>;

    #[doc(hidden)]
    fn resolve_create_within_capacity(
        &mut self, //.
        data: A::Components,
    ) -> Result<Entity<A>, A::Components>;

    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: Entity<A>) -> Option<A::Components>;
    #[doc(hidden)]
    fn resolve_archetype(&self) -> &A;
    #[doc(hidden)]
    fn resolve_archetype_mut(&mut self) -> &mut A;
}

/// A trait promising that an archetype has a given component.
///
/// Used for where bounds on functions that take an archetype as a generic type.
///
/// See [`Archetype`] for the methods that this enables on a type.
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
///     // Declare archetype ArchFoo with one component: CompA
///     ecs_archetype!(ArchFoo, CompA);
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
    #[doc(hidden)]
    fn resolve_borrow<'a>(borrow: &'a Self::Borrow<'a>) -> Ref<'a, C>;
    #[doc(hidden)]
    fn resolve_borrow_mut<'a>(borrow: &'a Self::Borrow<'a>) -> RefMut<'a, C>;
}

/// A `View` is a reference to a specific entity's components within an archetype.
///
/// This can be used in generic functions to access components from entity handles.
///
/// The `View` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
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

/// A trait promising that an entity view has a given component.
///
/// Used for where bounds on functions that take a view as a generic type.
///
/// See [`View`] for the methods that this enables on a type.
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
/// pub struct CompB(pub u32);
///
/// ecs_world! {
///     // Declare archetype ArchFoo with one component: CompA
///     ecs_archetype!(ArchFoo, CompA, CompB);
///     ecs_archetype!(ArchBar, CompA, CompB);
/// }
///
/// fn generic_access_comp_a<V>(view: &mut V) -> u32
/// where
///     V: ViewHas<CompA> + ViewHas<CompB>,
/// {
///     view.component::<CompA>().0 + view.component::<CompB>().0
/// }
///
/// fn generic_access(world: &mut EcsWorld, foo: Entity<ArchFoo>, bar: Entity<ArchBar>) {
///     let mut view_foo = world.archetype_mut::<ArchFoo>().view(foo).unwrap();
///     let val_foo = generic_access_comp_a(&mut view_foo);
///
///     let mut view_bar = world.archetype_mut::<ArchBar>().view(bar).unwrap();
///     let val_bar = generic_access_comp_a(&mut view_bar);
///
///     println!("{} {}", val_foo, val_bar);
/// }
///
/// # fn main() {} // Not actually running anything here
/// ```
pub trait ViewHas<C>: View {
    #[doc(hidden)]
    fn resolve_component(&self) -> &C;
    #[doc(hidden)]
    fn resolve_component_mut(&mut self) -> &mut C;
}

/// Trait promising that a given ECS world can resolve a type of entity key.
///
/// This is used for the destroy function, and implemented for `EntityAny` and `Entity<A>`.
pub trait WorldCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: K) -> K::DestroyOutput;
}

/// Trait promising that a given archetype can resolve a type of entity key.
///
/// This is implemented for `EntityAny`, `EntityRawAny`, `Entity<A>`, and `EntityRaw<A>`.
pub trait ArchetypeCanResolve<'a, View, K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;
    #[doc(hidden)]
    fn resolve_view(&'a mut self, entity: K) -> Option<View>;
}

#[doc(hidden)]
pub trait EntityKey {
    type DestroyOutput;
}

#[doc(hidden)]
pub trait StorageCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;
}
