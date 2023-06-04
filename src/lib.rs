#![allow(clippy::bool_comparison)]
#![allow(clippy::type_complexity)] // lol
#![allow(clippy::too_many_arguments)] // lmao

//! A generated entity component system 🦎
//!
//! The gecs crate provides a compile-time generated, zero-overhead ECS for simulations
//! on a budget. Unlike other ECS libraries, gecs takes a full ECS world structure
//! definition from code and precompiles all queries to achieve better performance with
//! no upfront cost or caching overhead. Queries in gecs can be inspected and checked at
//! compile-time in order to catch what would otherwise be bugs presenting only in tests
//! or execution. However, this comes at the cost of requiring all archetypes to be known
//! and declared at compile-time, so that adding or removing components from entities at
//! runtime isn't currently possible -- hybrid approaches could solve this in the future.
//!
//! Archetypes in gecs can be set to contain a fixed capacity of entities. If all of the
//! archetypes in your ECS world declaration are configured in this way, gecs will perform
//! zero allocations after startup. This guarantees that your ECS world will adhere to a
//! known and predictable memory overhead for constrained environments (e.g. servers on
//! cloud instances). Attempting to create a new entity in a full archetype will return
//! `None` (no panic). Support for dynamically-sized archetypes with `Vec`-like storage
//! behavior is planned for support at a later date but is not currently implemented.
//!
//! The goals for gecs are (in descending priority order):
//! - Fast iteration and find queries
//! - Fast entity creation and destruction
//! - Low, predictable memory overhead
//! - A user-friendly library interface
//! - Simplicity and focus in features
//!
//! All of the code that gecs generates in user crates is safe, and users of gecs can
//! use `#[deny(unsafe_code)]` in their own crates. Note that gecs does use unsafe code
//! internally to allow for compiler optimizations around known invariants. It is not a
//! goal of this library to be written entirely in safe Rust.
//!
//! # Getting Started
//!
//! See the [`ecs_world!`], [`ecs_find!`], and [`ecs_iter!`] macros to get started.

mod archetype;
mod util;

/// Handles for accessing entity representations as component data.
pub mod entity;

/// Error reporting for ECS operation failure.
pub mod error;

/// Traits for working with ECS types as generics.
pub mod traits;

/// Test

#[cfg(doc)]
mod macros {
    /// Macro for declaring a new ECS world struct with archetype storage.
    ///
    /// The `ecs_world!` macro is used for declaring an ECS world data structure to populate
    /// and perform queries on. All types used in the ECS world must be known at compile-time,
    /// and the full structure of each archetype must be declared with the world. Components
    /// may not be added or removed from entities at runtime.
    ///
    /// Note that irrespective of capacity configuration, a single ECS archetype can hold at
    /// most `16,777,216` entities due to the encoding structure of the `Entity` type. For
    /// similar reasons, an ECS world can have only `255` distinct archetypes.
    ///
    /// The `ecs_world!` macro has several inner pseudo-macros used for declaring archetypes
    /// or performing other tasks such as naming the ECS world's data type. These are not true
    /// macros and have no purpose or meaning outside of the body of an `ecs_world!` declaration.
    ///
    /// ## ecs_name!
    ///
    /// ```ignore
    /// ecs_name!(Name);
    /// ```
    /// The `ecs_name!` inner pseudo-macro is used for setting the name (in PascalCase) of the
    /// ECS world struct. Without this declaration, the world's name will default to `World`.
    ///
    /// ## ecs_archetype!
    ///
    /// ```ignore
    /// ecs_archetype!(Name, capacity, Component, ...);
    /// ```
    /// The `ecs_archetype!` inner pseudo-macro is used for declaring an archetype in an ECS
    /// world. It takes the following arguments:
    ///
    /// - `Name`: The name (in PascalCase) of the archetype Rust type.
    /// - `capacity`: The capacity of the archetype, specified in one of the following ways:
    ///     - A constant expression (e.g. `200`, or `config::ARCH_CAPACITY + 4`). This will
    ///       create a fixed-size archetype that can contain only that number of entities.
    ///       Attempting to add an entity to a full archetype will return `None` (no panic).
    ///     - The `dyn` keyword here is currently reserved for near-future work implementing
    ///       dynamically sized archetypes. It is not supported in this version of the library.
    /// - `Component, ...`: One or more component types to include in this archetype. Because
    ///   generated archetypes are `pub` with `pub` members, all components must be `pub` too.
    ///
    /// The `ecs_archetype!` declaration supports the following attributes:
    ///
    /// - `#[cfg]` attributes can be used both on the `ecs_archetype!` itself, and on
    ///   individual component parameters.
    /// - `#[archetype_id(N)]` can be used to override an archetype's `TYPE_ID` value to `N`
    ///   (which must be between `1` and `255`). By default, archetype IDs start at 1 and count
    ///   up sequentially from the last value, similar to enum discriminants. No two archetypes
    ///   may have the same archetype ID (this is compiler-enforced).
    ///
    /// # Example
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// // Components must be `pub`, as the ECS world will re-export them in its archetypes.
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// #[cfg(feature = "some_feature")] // CompC only exists if "some_feature" is enabled
    /// pub struct CompC(pub u32);
    ///
    /// const BAR_CAPACITY: usize = 30;
    ///
    /// ecs_world! {
    ///     ecs_name!(MyWorld); // Set the type name of this ECS structure to MyWorld
    ///
    ///     // Declare an archetype called ArchFoo with capacity 100 and two components
    ///     ecs_archetype!(
    ///         ArchFoo,
    ///         100,
    ///         CompA, // Note: Type paths are not currently supported for components
    ///         CompB,
    ///     );
    ///
    ///     // Declare ArchBar only if "some_feature" is enabled, otherwise it won't exist
    ///     #[cfg(feature = "some_feature")]
    ///     ecs_archetype!(
    ///         ArchBar,
    ///         BAR_CAPACITY, // Constants may also be used for archetype capacity
    ///         CompA,
    ///         CompC,
    ///     );
    ///    
    ///     #[archetype_id(6)]
    ///     ecs_archetype!(
    ///         ArchBaz,
    ///         400,
    ///         CompA,
    ///         CompB,
    ///         #[cfg(feature = "some_feature")]
    ///         CompC, // ArchBaz will only have CompC if "some_feature" is enabled
    ///     );
    /// }
    ///
    /// fn main() {
    ///     // Create a new empty world with allocated storage where appropriate
    ///     let mut world = MyWorld::default();
    ///
    ///     // Push an ArchFoo entity into the world and unwrap Option<Entity<ArchFoo>> result
    ///     let entity_a = world.push::<ArchFoo>((CompA(0), CompB(1))).unwrap();
    ///
    ///     // The length of the archetype should now be 1
    ///     assert_eq!(world.len::<ArchFoo>(), 1);
    ///
    ///     // Remove the entity (we don't need to turbofish because this is an Entity<ArchFoo>)
    ///     world.remove(entity_a);
    ///
    ///     assert_eq!(world.len::<ArchFoo>(), 0);
    ///     assert!(world.is_empty::<ArchFoo>());
    ///
    ///     // Use of #[cfg]-conditionals
    ///     #[cfg(feature = "some_feature")] world.push::<ArchBar>((CompA(2), CompB(3), CompC(4)));
    ///     world.push::<ArchBaz>((CompA(5), CompB(6), #[cfg(feature = "some_feature")] CompC(7)));
    ///
    ///     // Use of #[archetype_id(N)] assignment
    ///     assert_eq!(ArchFoo::TYPE_ID.get(), 1);
    ///     assert_eq!(ArchBaz::TYPE_ID.get(), 6);
    ///     #[cfg(feature = "some_feature")] assert_eq!(ArchBar::TYPE_ID.get(), 2);
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_world {
        {...} => {};
    }

    /// Finds a single entity in an ECS world and performs an operation on it, if found.
    ///
    /// ```ignore
    /// ecs_find!(world, entity, |comp_a: &CompA, comp_b: &mut CompB, ...| { ... });
    /// ```
    ///
    /// The `ecs_find!` macro finds a single entity in an ECS world and performs an operation
    /// on it, if that entity is found in archetype storage. It takes the following arguments:
    ///
    /// - `world`: The world (as an expression) that you want to query.
    /// - `entity`: The entity handle you want to look up. May be an `Entity<A>` or `EntityAny`.
    /// - `|comp_a: &CompA, comp_b: &mut CompB, ...| { ... }`: A closure containing the operation
    ///   to perform on the current entity's data. The parameters of the closure determine what
    ///   components for the entity that this query will access and how. Any component can be
    ///   accessed as `&Component` or `&mut Component`. The query will only check archetypes
    ///   that are known at compile-time to have all components requested in the query closure.
    ///     - Note that this closure is always treated as a `&mut FnMut`.
    ///
    /// The `ecs_find!` macro returns `true` if the entity was found, or false otherwise.
    ///
    /// # Special Query Closure Arguments
    ///
    /// Query closure arguments can have the following special types:
    ///
    /// - `&Entity<A>`/`&EntityAny`: Returns the current entity being accessed by the closure.
    ///   This is somewhat redundant for `ecs_find!` queries, but useful for `ecs_iter!` loops.
    ///   Note that this is always read-only -- the entity can never be accessed mutably.
    ///
    /// # Example
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// pub struct CompC(pub u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, 100, CompA, CompB);
    ///     ecs_archetype!(ArchBar, 100, CompA, CompC);
    /// }
    ///
    /// // If you need to use a non-mut reference, see the ecs_find_borrow! macro
    /// fn add_three(world: &mut World, entity: Entity<ArchFoo>) -> bool {
    ///     // The result will be true if the entity was found and operated on
    ///     ecs_find!(world, entity, |comp_a: &mut CompA| { comp_a.0 += 3; })
    /// }
    ///
    /// fn add_three_any(world: &mut World, entity: EntityAny) -> bool {
    ///     // The query syntax is the same for both Entity<A> and EntityAny
    ///     ecs_find!(world, entity, |comp_a: &mut CompA| { comp_a.0 += 3; })
    /// }
    ///
    /// fn main() {
    ///     let mut world = World::default();
    ///
    ///     // Note: Push returns an Option<Entity<A>>, so we must unwrap
    ///     let entity_a = world.push::<ArchFoo>((CompA(0), CompB(0))).unwrap();
    ///     let entity_b = world.push::<ArchBar>((CompA(0), CompC(0))).unwrap();
    ///
    ///     assert!(ecs_find!(world, entity_a, |c: &CompA| assert_eq!(c.0, 0)));
    ///     assert!(ecs_find!(world, entity_b, |c: &CompA| assert_eq!(c.0, 0)));
    ///
    ///     assert!(add_three(&mut world, entity_a));
    ///     assert!(add_three_any(&mut world, entity_b.into())); // Convert to an EntityAny
    ///
    ///     assert!(ecs_find!(world, entity_a, |c: &CompA| assert_eq!(c.0, 3)));
    ///     assert!(ecs_find!(world, entity_b, |c: &CompA| assert_eq!(c.0, 3)));
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_find {
        (...) => {};
    }

    /// Variant of `ecs_find!` that runtime-borrows data, for use with a non-mut world reference.
    ///
    /// See the [`ecs_find!`] macro for more information on find queries.
    ///
    /// This version borrows each archetype's data on a component-by-component basis at runtime
    /// rather than at compile-time, allowing for situations where compile-time borrow checking
    /// isn't sufficient. This is typically used for nested queries, where an `ecs_iter!` or an
    /// `ecs_find!` needs to happen in the body of another query. This operation is backed by
    /// [`std::cell::RefCell`] operations, and will panic if you attempt to mutably borrow an
    /// archetype's component row while any other borrow is currently active.
    ///
    /// # Example
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// pub struct Parent(pub Option<Entity<ArchFoo>>);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, 100, CompA, CompB, Parent);
    /// }
    ///
    /// fn main() {
    ///     let mut world = World::default();
    ///
    ///     let parent = world.push::<ArchFoo>((CompA(0), CompB(0), Parent(None))).unwrap();
    ///     let child = world.push::<ArchFoo>((CompA(1), CompB(0), Parent(Some(parent)))).unwrap();
    ///
    ///     // Assert that we found the parent, and that its CompB value is 0
    ///     assert!(ecs_find!(world, parent, |b: &CompB| assert_eq!(b.0, 0)));
    ///
    ///     ecs_iter_borrow!(world, |child_a: &CompA, parent: &Parent| {
    ///         if let Some(parent_entity) = parent.0 {
    ///             // Note: We can't mutably borrow the CompA or Parent component data here!
    ///             ecs_find_borrow!(world, parent_entity, |parent_b: &mut CompB| {
    ///                 // Copy the value from the child's CompA to the parent's CompB
    ///                 parent_b.0 = child_a.0;
    ///             });
    ///         }
    ///     });
    ///
    ///     // Assert that we found the parent, and that its CompB value is now 1
    ///     assert!(ecs_find!(world, parent, |b: &CompB| assert_eq!(b.0, 1)));
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_find_borrow {
        (...) => {};
    }

    /// Iterates over all entities across all archetypes that match the given component bounds.
    ///
    /// ```ignore
    /// ecs_iter!(world, |comp_a: &CompA, comp_b: &mut CompB, ...| { ... });
    /// ```
    ///
    /// The `ecs_iter!` macro iterates over all entities matching the conditions of its closure
    /// and executes that closure on those entities' data. It takes the following arguments:
    ///
    /// - `world`: The world (as an expression) that you want to query.
    /// - `|comp_a: &CompA, comp_b: &mut CompB, ...| { ... }`: A closure containing the operation
    ///   to perform on the current entity's data. The parameters of the closure determine what
    ///   components for the entity that this query will access and how. Any component can be
    ///   accessed as `&Component` or `&mut Component`. The query will only check archetypes
    ///   that are known at compile-time to have all components requested in the query closure.
    ///     - Note that this closure is always treated as a `&mut FnMut`.
    ///
    /// # Special Query Closure Arguments
    ///
    /// Query closure arguments can have the following special types:
    ///
    /// - `&Entity<A>`/`&EntityAny`: Returns the current entity being accessed by the closure.
    ///   This is somewhat redundant for `ecs_find!` queries, but useful for `ecs_iter!` loops.
    ///   Note that this is always read-only -- the entity can never be accessed mutably.
    ///
    /// # Example
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// pub struct CompC(pub u32);
    ///
    /// ecs_world! {
    ///     ecs_archetype!(ArchFoo, 100, CompA, CompB);
    ///     ecs_archetype!(ArchBar, 100, CompA, CompC);
    /// }
    ///
    /// fn main() {
    ///     let mut world = World::default();
    ///
    ///     let mut vec_a = Vec::<EntityAny>::new();
    ///     let mut vec_b = Vec::<EntityAny>::new();
    ///     let mut vec_c = Vec::<EntityAny>::new();
    ///
    ///     let entity_a = world.push::<ArchFoo>((CompA(0), CompB(0))).unwrap();
    ///     let entity_b = world.push::<ArchBar>((CompA(0), CompC(0))).unwrap();
    ///
    ///     // Iterates both ArchFoo and ArchBar since both have a CompA
    ///     ecs_iter!(world, |entity: &EntityAny, a: &mut CompA| {
    ///         vec_a.push(*entity);
    ///         a.0 += 3; // Add 3 to both entities
    ///     });
    ///     assert!(vec_a == vec![entity_a.into(), entity_b.into()]);
    ///
    ///     // Even though both ArchFoo and ArchBar have a CompA, only ArchFoo can
    ///     // provide Entity<ArchFoo> handles, so this will only iterate that one
    ///     ecs_iter!(world, |entity: &Entity<ArchFoo>, a: &mut CompA| {
    ///         vec_b.push((*entity).into());
    ///         a.0 += 3; // Add 3 to entity_a
    ///     });
    ///     assert!(vec_b == vec![entity_a.into()]);
    ///
    ///     // Iterates only ArchBar since ArchFoo does not have a CompC
    ///     ecs_iter!(world, |entity: &EntityAny, a: &mut CompA, _: &CompC| {
    ///         vec_c.push(*entity);
    ///         a.0 += 3; // Add 3 to entity_b
    ///     });
    ///     assert!(vec_c == vec![entity_b.into()]);
    ///
    ///     let mut sum = 0;
    ///     ecs_iter!(world, |a: &CompA| {
    ///         sum += a.0;
    ///     });
    ///     assert_eq!(sum, 12);
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_iter {
        (...) => {};
    }

    /// Variant of `ecs_iter!` that runtime-borrows data, for use with a non-mut world reference.
    ///
    /// See [`ecs_iter`] for more information on find queries.
    ///
    /// This version borrows each archetype's data on a component-by-component basis at runtime
    /// rather than at compile-time, allowing for situations where compile-time borrow checking
    /// isn't sufficient. This is typically used for nested queries, where an `ecs_iter!` or an
    /// `ecs_find!` needs to happen in the body of another query. This operation is backed by
    /// [`std::cell::RefCell`] operations, and will panic if you attempt to mutably borrow an
    /// archetype's component row while any other borrow is currently active.
    ///
    /// # Example
    ///
    /// See the example for ['ecs_find_borrow!`].
    #[macro_export]
    macro_rules! ecs_iter_borrow {
        (...) => {};
    }
}

#[cfg(not(doc))]
pub mod macros {
    pub use gecs_macros::ecs_world;
}

/// `use gecs::prelude::*;` to import common macros, traits, and types.
pub mod prelude {
    use super::*;

    #[doc(hidden)]
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
    pub use gecs_macros::{__ecs_find, __ecs_find_borrow};
    pub use gecs_macros::{__ecs_iter, __ecs_iter_borrow};

    pub use archetype::slices::*;
    pub use archetype::storage_dynamic::*;
    pub use archetype::storage_fixed::*;

    pub use traits::Archetype;
    pub use traits::{ArchetypeContainer, ComponentContainer};
    pub use traits::{HasArchetype, HasComponent};
}
