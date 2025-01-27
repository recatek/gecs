use std::cell::{Ref, RefMut};

use crate::entity::{ArchetypeId, Entity, EntityDirect};
use crate::version::ArchetypeVersion;

#[cfg(doc)]
use crate::entity::EntityDirectAny;

#[cfg(any(doc, feature = "events"))]
use crate::entity::EntityAny;

/// The base trait for an ECS world in gecs.
///
/// This can be used in generic functions to access archetypes or create/destroy entities.
///
/// The `World` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
pub trait World: Sized {
    const NUM_ARCHETYPES: usize;

    /// The capacity input builder struct type. Contains one usize for each archetype on init.
    type Capacities;

    /// Creates a new empty world.
    ///
    /// This will not immediately allocate. All archetypes will begin with 0 capacity.
    fn new() -> Self;

    /// Creates a new world with per-archetype capacities.
    ///
    /// This will allocate all archetypes to the given capacities (which may be zero).
    /// If a given archetype capacity is 0, that archetype will not allocate until later.
    ///
    /// # Panics
    ///
    /// This will panic if given a size that exceeds the maximum possible capacity
    /// value for an archetype (currently `16,777,216`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// // Components -- these must be pub because the world is exported as pub as well.
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// pub struct CompC(pub u32);
    ///
    /// ecs_world! {
    ///     // Declare two archetypes, ArchFoo and ArchBar.
    ///     ecs_archetype!(ArchFoo, CompA, CompB);
    ///     ecs_archetype!(ArchBar, CompA, CompC);
    ///     ecs_archetype!(ArchBaz, CompB, CompC);
    /// }
    ///
    /// fn main() {
    ///     let world = EcsWorld::with_capacity(EcsWorldCapacity {
    ///         arch_foo: 10,        // Initialize ArchFoo with capacity 10
    ///         ..Default::default() // Leave the rest (ArchBar and ArchBaz) at capacity 0
    ///     });
    /// }
    /// ```
    fn with_capacity(capacity: Self::Capacities) -> Self;

    /// Creates a new entity with the given components to this archetype storage.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// # Panics
    ///
    /// Panics if the archetype can no longer expand to accommodate the new data.
    #[inline(always)]
    fn create<A: Archetype>(
        &mut self, //.
        components: impl Into<A::Components>,
    ) -> Entity<A>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_create(self, components.into())
    }

    /// Creates a new entity if there is sufficient spare capacity to store it.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// Unlike `create` this method will not reallocate when there is insufficient
    /// capacity. Instead, it will return an error and the input components for reuse.
    ///
    /// Based off of [Vec::push_within_capacity].
    #[inline(always)]
    fn create_within_capacity<A: Archetype>(
        &mut self, //.
        components: impl Into<A::Components>,
    ) -> Result<Entity<A>, A::Components>
    where
        Self: WorldHas<A>,
    {
        <Self as WorldHas<A>>::resolve_create_within_capacity(self, components.into())
    }

    /// Returns true if this world contains the given entity key.
    #[inline(always)]
    fn contains<K: EntityKey>(&self, entity: K) -> bool
    where
        Self: WorldCanResolve<K>,
    {
        <Self as WorldCanResolve<K>>::resolve_contains(self, entity)
    }

    /// If the entity exists in the world, this returns a direct entity pointing to its data.
    /// See [`EntityDirect`] and [`EntityDirectAny`] for more information.
    #[inline(always)]
    fn to_direct<K: EntityKey>(&self, entity: K) -> Option<K::DirectOutput>
    where
        Self: WorldCanResolve<K>,
    {
        <Self as WorldCanResolve<K>>::resolve_direct(self, entity)
    }

    /// Gets a [`View`] for the given entity across archetypes in the full world.
    /// This is a convenience function for [`Archetype::view`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    ///     ecs_archetype!(ArchBar, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(1),));
    ///     // Note: Does not work with EntityAny/EntityDirectAny (archetype is unknown)
    ///     let mut view = world.view(entity_a).unwrap();
    ///
    ///     assert!(view.component::<CompA>().0 == 1);
    ///     view.component_mut::<CompA>().0 += 1;
    ///
    ///     let found = ecs_find!(world, entity_a, |comp_a: &CompA| { assert_eq!(comp_a.0, 2); });
    ///     assert!(found.is_some())
    /// }
    /// ```
    #[inline(always)]
    fn view<A: Archetype, K: EntityKeyTyped<A>>(&mut self, entity: K) -> Option<A::View<'_>>
    where
        Self: WorldHas<A>,
        A: ArchetypeCanResolve<K>,
    {
        <Self as WorldHas<A>>::resolve_archetype_mut(self).view(entity)
    }

    /// Gets a [`Borrow`] for the given entity across archetypes in the full world.
    /// This is a convenience function for [`Archetype::borrow`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(1),));
    ///     // Note: Does not work with EntityAny/EntityDirectAny (archetype is unknown)
    ///     let mut borrow = world.view(entity_a).unwrap();
    ///
    ///     assert!(borrow.component::<CompA>().0 == 1);
    ///     borrow.component_mut::<CompA>().0 += 1;
    ///
    ///     let found = ecs_find!(world, entity_a, |comp_a: &CompA| { assert_eq!(comp_a.0, 2); });
    ///     assert!(found.is_some())
    /// }
    /// ```
    #[inline(always)]
    fn borrow<A: Archetype, K: EntityKeyTyped<A>>(&self, entity: K) -> Option<A::Borrow<'_>>
    where
        Self: WorldHas<A>,
        A: ArchetypeCanResolve<K>,
    {
        <Self as WorldHas<A>>::resolve_archetype(self).borrow(entity)
    }

    /// If the entity exists in the world, this destroys it.
    ///
    /// This returns an `Option<(C0, C1, ..., Cn)>` where `(C0, C1, ..., Cn)` are the entity's
    /// former (now removed) components. A `Some` result means the entity was found and destroyed.
    /// A `None` result means the given entity handle was invalid.
    ///
    /// If called with [`EntityAny`] or [`EntityDirectAny`] this instead returns `Option<()>` as the
    /// return component type tuple can't be known at compile time. To get the components from the
    /// entity on destruction, convert the any-type entity into a typed entity before destroying it
    /// (see [`SelectEntity`](crate::SelectEntity) for example).
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

    /// Returns an iterator over all the entities created since the last time entity events were
    /// cleared on the world or on any specific archetypes. This list has no ordering guarantees.
    /// Note that entities appear in this list even if they have since been destroyed.
    ///
    /// These events accumulate until they are cleared by [`clear_events`](World::clear_events).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA;
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    ///     ecs_archetype!(ArchBar, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA,));
    ///     let entity_b = world.create::<ArchBar>((CompA,));
    ///     world.destroy(entity_a);
    ///     world.destroy(entity_b);
    ///
    ///     // Created events persist even after the entity is destroyed.
    ///     let created = world.iter_created().collect::<Vec<_>>();
    ///     assert_eq!(created.len(), 2);
    ///     assert!(created.contains(&&entity_a.into()));
    ///     assert!(created.contains(&&entity_b.into()));
    ///
    ///     let destroyed = world.iter_destroyed().collect::<Vec<_>>();
    ///     assert_eq!(created.len(), 2);
    ///     assert!(created.contains(&&entity_a.into()));
    ///     assert!(created.contains(&&entity_b.into()));
    ///
    ///     world.clear_events();
    ///     assert_eq!(world.iter_created().count(), 0);
    ///     assert_eq!(world.iter_destroyed().count(), 0);
    /// }
    /// ```
    #[cfg(feature = "events")]
    fn iter_created(&self) -> impl Iterator<Item = &EntityAny>;

    /// Returns an iterator over all the entities created since the last time entity events were
    /// cleared on the world or on any specific archetypes. This list has no ordering guarantees.
    ///
    /// These events accumulate until they are cleared by [`clear_events`](World::clear_events).
    ///
    /// # Examples
    ///
    /// See [`Archetype::iter_created`].
    #[cfg(feature = "events")]
    fn iter_destroyed(&self) -> impl Iterator<Item = &EntityAny>;

    /// Clears the currently stored entity creation/destruction events in this archetype.
    /// This must be done periodically to prevent events from accumulating indefinitely.
    ///
    /// This clears the events in all archetypes in the world. See the archetype-level
    /// [`Archetype::clear_events`] function to clear events only for a specific archetype.
    ///
    /// # Examples
    ///
    /// See [`World::iter_created`].
    #[cfg(feature = "events")]
    fn clear_events(&mut self);
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
    for<'a> Self: ArchetypeCanResolve<Entity<Self>>,
    for<'a> Self: ArchetypeCanResolve<EntityDirect<Self>>,
{
    /// A unique type ID assigned to this archetype in generation.
    const ARCHETYPE_ID: ArchetypeId;

    /// A struct with named storage to each component in this archetype.
    type Components: Components<Archetype = Self>;

    /// A struct yielded by iterators on this archetype, including the entity itself.
    type IterItem<'a>
    where
        Self: 'a;

    /// A struct yielded by mut iterators on this archetype, including the entity itself.
    type IterItemMut<'a>
    where
        Self: 'a;

    /// The slices type when accessing all of this archetype's slices simultaneously.
    type Slices<'a>
    where
        Self: 'a;

    /// The borrow type when performing sequential borrows of an entity's components.
    type Borrow<'a>: Borrow<'a, Archetype = Self>
    where
        Self: 'a;

    /// The view type when accessing a single entity's components simultaneously.
    type View<'a>: View<'a, Archetype = Self>
    where
        Self: 'a;

    /// Constructs a new, empty archetype.
    ///
    /// If the archetype uses dynamic storage, this archetype will not allocate until
    /// an entity is added to it. Otherwise, for static storage, the full capacity
    /// will be allocated on creation of the archetype.
    fn new() -> Self;

    /// Constructs a new archetype pre-allocated to the given storage capacity.
    ///
    /// If the given capacity would result in zero size, this will not allocate.
    fn with_capacity(capacity: usize) -> Self;

    /// Returns the number of entities in the archetype, also referred to as its length.
    fn len(&self) -> usize;

    /// Returns the total number of elements the archetype can hold without reallocating.
    /// If the archetype has fixed-sized storage, this is the absolute total capacity.
    ///
    /// Note that the archetype may not be able to me filled to its capacity if it has
    /// had to orphan/leak entity slots due to generational index overflow.
    fn capacity(&self) -> usize;

    /// Returns `true` if the archetype contains no elements.
    fn is_empty(&self) -> bool;

    /// Returns the generational version of the archetype. Intended for internal use.
    fn version(&self) -> ArchetypeVersion;

    /// Returns a read-only slice of all entities in this archetype.
    /// This slice is ordered arbitrarily and may change at later points.
    fn entities(&self) -> &[Entity<Self>];

    /// Creates a new entity with the given components to this archetype storage.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// # Panics
    ///
    /// Panics if the archetype can no longer expand to accommodate the new data.
    fn create(&mut self, components: impl Into<Self::Components>) -> Entity<Self>;

    /// Creates a new entity if there is sufficient spare capacity to store it.
    /// Returns a typed entity handle pointing to the new entity in the archetype.
    ///
    /// Unlike `create` this method will not reallocate when there is insufficient
    /// capacity. Instead, it will return an error along with given components.
    fn create_within_capacity(
        &mut self,
        components: impl Into<Self::Components>,
    ) -> Result<Entity<Self>, Self::Components>;

    /// Returns an iterator over all of the entities and their data.
    fn iter(&mut self) -> impl Iterator<Item = Self::IterItem<'_>>;

    /// Returns a mutable iterator over all of the entities and their data.
    fn iter_mut(&mut self) -> impl Iterator<Item = Self::IterItemMut<'_>>;

    /// Returns mutable slices to all data for all entities in the archetype. To get the
    /// data index for a specific entity using this function, use the `resolve` function.
    fn get_all_slices_mut(&mut self) -> Self::Slices<'_>;

    /// Returns true if this archetype contains the given entity key.
    #[inline(always)]
    fn contains<K: EntityKey>(&self, entity: K) -> bool
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_for(self, entity).is_some()
    }

    /// If the entity exists in the archetype, this returns its dense data slice index.
    /// The returned index is guaranteed to be within bounds of the dense data slices.
    #[inline(always)]
    fn resolve<K: EntityKey>(&self, entity: K) -> Option<usize>
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_for(self, entity)
    }

    /// If the entity exists in the archetype, this returns a direct entity pointing to its data.
    /// See [`EntityDirect`] and [`EntityDirectAny`] for more information.
    #[inline(always)]
    fn to_direct<K: EntityKey>(&self, entity: K) -> Option<K::DirectOutput>
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_direct(self, entity)
    }

    /// Returns a ['View'] with mutable references to all of this entity's components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(1),));
    ///     let mut view = world.arch_foo.view(entity_a).unwrap();
    ///
    ///     assert!(view.component::<CompA>().0 == 1);
    ///     view.component_mut::<CompA>().0 += 1;
    ///
    ///     let found = ecs_find!(world, entity_a, |comp_a: &CompA| { assert_eq!(comp_a.0, 2); });
    ///     assert!(found.is_some())
    /// }
    /// ```
    #[inline(always)]
    fn view<K: EntityKey>(&mut self, entity: K) -> Option<Self::View<'_>>
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_view(self, entity)
    }

    /// Returns a [`Borrow`] with borrowed references to all of this entity's components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(1),));
    ///     let mut borrow = world.arch_foo.borrow(entity_a).unwrap();
    ///
    ///     assert!(borrow.component::<CompA>().0 == 1);
    ///     borrow.component_mut::<CompA>().0 += 1;
    ///
    ///     let found = ecs_find!(world, entity_a, |comp_a: &CompA| { assert_eq!(comp_a.0, 2); });
    ///     assert!(found.is_some())
    /// }
    /// ```
    #[inline(always)]
    fn borrow<K: EntityKey>(&self, entity: K) -> Option<Self::Borrow<'_>>
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_borrow(self, entity)
    }

    /// If the entity exists in the archetype, this destroys it.
    ///
    /// This returns an `Option<(C0, C1, ..., Cn)>` where `(C0, C1, ..., Cn)` are the entity's
    /// former (now removed) components. A `Some` result means the entity was found and destroyed.
    /// A `None` result means the given entity handle was invalid.
    #[inline(always)]
    fn destroy<K: EntityKey>(&mut self, entity: K) -> Option<Self::Components>
    where
        Self: ArchetypeCanResolve<K>,
    {
        <Self as ArchetypeCanResolve<K>>::resolve_destroy(self, entity)
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

    /// Returns an iterator over all the entities created since the last time entity events were
    /// cleared on the world or on this specific archetype. This list has no ordering guarantees.
    /// Note that entities appear in this list even if they have since been destroyed.
    ///
    /// These events accumulate until they are cleared by [`clear_events`](Archetype::clear_events).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA;
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, CompA);
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.arch_foo.create((CompA,));
    ///     let entity_b = world.arch_foo.create((CompA,));
    ///     world.arch_foo.destroy(entity_a);
    ///     world.arch_foo.destroy(entity_b);
    ///
    ///     // Created events persist even after the entity is destroyed.
    ///     let created = world.arch_foo.iter_created().collect::<Vec<_>>();
    ///     assert_eq!(created.len(), 2);
    ///     assert!(created.contains(&&entity_a));
    ///     assert!(created.contains(&&entity_b));
    ///
    ///     let destroyed = world.arch_foo.iter_destroyed().collect::<Vec<_>>();
    ///     assert_eq!(created.len(), 2);
    ///     assert!(created.contains(&&entity_a));
    ///     assert!(created.contains(&&entity_b));
    ///
    ///     world.arch_foo.clear_events();
    ///     assert_eq!(world.arch_foo.iter_created().count(), 0);
    ///     assert_eq!(world.arch_foo.iter_destroyed().count(), 0);
    /// }
    /// ```
    #[cfg(feature = "events")]
    fn iter_created(&self) -> impl Iterator<Item = &Entity<Self>>;

    /// Returns an iterator over all the entities destroyed since the last time entity events were
    /// cleared on the world or on this specific archetype. This list has no ordering guarantees.
    ///
    /// These events accumulate until they are cleared by [`clear_events`](Archetype::clear_events).
    ///
    /// # Examples
    ///
    /// See [`Archetype::iter_created`].
    #[cfg(feature = "events")]
    fn iter_destroyed(&self) -> impl Iterator<Item = &Entity<Self>>;

    /// Clears the currently stored entity creation/destruction events in this archetype.
    /// This must be done periodically to prevent events from accumulating indefinitely.
    ///
    /// This clears only the events in this particular archetype. See the world-level
    /// [`World::clear_events`] function to clear events for all archetypes in a world.
    ///
    /// # Examples
    ///
    /// See [`Archetype::iter_created`].
    #[cfg(feature = "events")]
    fn clear_events(&mut self);
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
/// # Examples
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
/// # Examples
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
    const COMPONENT_ID: u8;

    #[doc(hidden)]
    fn resolve_get_slice(&mut self) -> &[C];
    #[doc(hidden)]
    fn resolve_get_slice_mut(&mut self) -> &mut [C];
    #[doc(hidden)]
    fn resolve_borrow_slice(&self) -> Ref<[C]>;
    #[doc(hidden)]
    fn resolve_borrow_slice_mut(&self) -> RefMut<[C]>;

    #[doc(hidden)]
    fn resolve_extract_components(components: &Self::Components) -> &C;
    #[doc(hidden)]
    fn resolve_extract_components_mut(components: &mut Self::Components) -> &mut C;
    #[doc(hidden)]
    fn resolve_extract_view<'a>(view: &'a Self::View<'_>) -> &'a C;
    #[doc(hidden)]
    fn resolve_extract_view_mut<'a>(view: &'a mut Self::View<'_>) -> &'a mut C;
    #[doc(hidden)]
    fn resolve_extract_borrow<'a>(borrow: &'a Self::Borrow<'_>) -> Ref<'a, C>;
    #[doc(hidden)]
    fn resolve_extract_borrow_mut<'a>(borrow: &'a Self::Borrow<'_>) -> RefMut<'a, C>;
}

pub trait Components {
    type Archetype: Archetype;

    type Tuple;

    fn get<C>(&self) -> &C
    where
        Self::Archetype: ArchetypeHas<C>;

    fn get_mut<C>(&mut self) -> &mut C
    where
        Self::Archetype: ArchetypeHas<C>;

    /// Converts this named components struct into a raw tuple of components, in the same order.
    /// This is a convenience function for when `into` won't work without explicit types.
    fn into_tuple(self) -> Self::Tuple;
}

/// A `View` is a mutable reference to a specific entity's components within an archetype. It
/// allows direct access to all of a specific entity's components, but exclusively borrows the
/// entire archetype in the process (for more flexibility here at a runtime cost, see [`Borrow`]).
///
/// This can be used in generic functions to access components from entity handles.
///
/// The `View` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
//
// DEV NOTE: There's no point in trying to make a View/ViewMut split because each column is in
// a RefCell anyway, so you can't make non-mut references to them without borrowing, and Borrow
// already exists for that. Borrow also does it better, since it doesn't fully borrow each slice.
pub trait View<'a> {
    type Archetype: Archetype<View<'a> = Self> + 'a;

    /// Returns the archetype dense index that this view refers to.
    fn index(&self) -> usize;

    /// Fetches the given component from this view.
    #[inline(always)]
    fn component<'b, C>(&'b self) -> &'b C
    where
        Self::Archetype: ArchetypeHas<C>,
    {
        <Self::Archetype as ArchetypeHas<C>>::resolve_extract_view(self)
    }

    /// Mutably fetches the given component from this view.
    #[inline(always)]
    fn component_mut<'b, C>(&'b mut self) -> &'b mut C
    where
        Self::Archetype: ArchetypeHas<C>,
    {
        <Self::Archetype as ArchetypeHas<C>>::resolve_extract_view_mut(self)
    }
}

/// A `Borrow` is a borrowed reference to a specific entity's components within an archetype.
///
/// This can be used in generic functions to access components from entity handles. Note that this
/// borrows an entire column of an archetype's components at a time, so with multiple entities
/// within the same archetype, only one set of component type/column for each can be exclusively
/// borrowed at a time.
///
/// The `Borrow` trait should be implemented only by the `ecs_world!` macro.
/// This is not intended for manual implementation by any user data structures.
pub trait Borrow<'a> {
    type Archetype: Archetype<Borrow<'a> = Self> + 'a;

    /// Returns the archetype dense index that this borrow refers to.
    fn index(&self) -> usize;

    /// Returns the entity handle that this borrow refers to.
    fn entity(&self) -> &Entity<Self::Archetype>;

    /// Gets the given component from this borrow. Performs a runtime check for safety.
    ///
    /// # Panics
    ///
    /// Panics if any other borrow has exclusive/mut access to any entry for this type of component
    /// within this same archetype, even if it is for a different entity.
    #[inline(always)]
    fn component<'b, C>(&'b self) -> Ref<'b, C>
    where
        Self::Archetype: ArchetypeHas<C>,
    {
        <Self::Archetype as ArchetypeHas<C>>::resolve_extract_borrow(self)
    }

    /// Gets the given component mutably from this borrow. Performs a runtime check for safety.
    ///
    /// # Panics
    ///
    /// Panics if any other borrow has any type of access to any entry for this type of component
    /// within this same archetype, even if it is for a different entity.
    #[inline(always)]
    fn component_mut<'b, C>(&'b self) -> RefMut<'b, C>
    where
        Self::Archetype: ArchetypeHas<C>,
    {
        <Self::Archetype as ArchetypeHas<C>>::resolve_extract_borrow_mut(self)
    }
}

/// Trait promising that a given ECS world can resolve a type of entity key.
///
/// This is implemented for [`Entity`], [`EntityDirect`]. [`EntityAny`], and [`EntityDirectAny`].
pub trait WorldCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_contains(&self, entity: K) -> bool;

    #[doc(hidden)]
    fn resolve_direct(&self, entity: K) -> Option<K::DirectOutput>;

    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: K) -> K::DestroyOutput;
}

/// Trait promising that a given archetype can resolve a type of entity key.
///
/// This is implemented for [`Entity`], [`EntityDirect`]. [`EntityAny`], and [`EntityDirectAny`].
pub trait ArchetypeCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;

    #[doc(hidden)]
    fn resolve_direct(&self, entity: K) -> Option<K::DirectOutput>;

    #[doc(hidden)]
    fn resolve_view(&mut self, entity: K) -> Option<Self::View<'_>>
    where
        Self: Archetype;

    #[doc(hidden)]
    fn resolve_borrow(&self, entity: K) -> Option<Self::Borrow<'_>>
    where
        Self: Archetype;

    // NOTE: Special case here! We don't use K::DestroyOutput, but the components directly, since
    // we will always know the component return types at the archetype (but not world) level.
    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: K) -> Option<Self::Components>
    where
        Self: Archetype;
}

#[doc(hidden)]
pub trait StorageCanResolve<K: EntityKey> {
    #[doc(hidden)]
    fn resolve_for(&self, entity: K) -> Option<usize>;
    #[doc(hidden)]
    fn resolve_direct(&self, entity: K) -> Option<K::DirectOutput>;
    #[doc(hidden)]
    fn resolve_destroy(&mut self, entity: K) -> K::DestroyOutput;
}

#[doc(hidden)]
pub trait EntityKey: Clone + Copy {
    #[doc(hidden)]
    type DestroyOutput;
    #[doc(hidden)]
    type DirectOutput;
}

#[doc(hidden)]
pub trait EntityKeyTyped<A: Archetype + ArchetypeCanResolve<Self>>: EntityKey {
    #[doc(hidden)]
    type Archetype: Archetype;
}
