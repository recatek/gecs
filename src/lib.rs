#![allow(clippy::type_complexity)] // lol
#![allow(clippy::too_many_arguments)] // lmao
#![allow(clippy::needless_doctest_main)] // this has false positives with gecs's macros
#![allow(clippy::bool_comparison)] // using "== false" is easier to read in some cases
#![allow(clippy::len_zero)] // using "len() > 0" is easier to read in some cases

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
//! Archetypes in gecs can be set to contain a fixed or dynamic capacity of entities. If
//! all of the archetypes in your ECS world declaration are set to a fixed capacity, gecs
//! will perform zero allocations after startup. This guarantees that your ECS world will
//! adhere to a known and predictable memory overhead for constrained environments (e.g.
//! servers on cloud instances). Attempting to add an entity to a full archetype can
//! either report failure or panic depending on the method you call to do so.
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
//! See the [`ecs_world!`], [`ecs_find!`], and [`ecs_iter!`] macros for more information.
//! The following example creates a world with three components and two archetypes:
//!
//! ```rust
//! use gecs::prelude::*;
//!
//! // Components -- these must be pub because the world is exported as pub as well.
//! pub struct CompA(pub u32);
//! pub struct CompB(pub u32);
//! pub struct CompC(pub u32);
//!
//! ecs_world! {
//!     // Declare two archetypes, ArchFoo and ArchBar.
//!     ecs_archetype!(ArchFoo, 100, CompA, CompB); // Fixed capacity of 100 entities.
//!     ecs_archetype!(ArchBar, dyn, CompA, CompC); // Dynamic (dyn) entity capacity.
//! }
//!
//! fn main() {
//!     let mut world = EcsWorld::default(); // Initialize an empty new ECS world.
//!
//!     // Add entities to the world by populating their components and receive their handles.
//!     let entity_a = world.create::<ArchFoo>((CompA(1), CompB(20)));
//!     let entity_b = world.create::<ArchBar>((CompA(3), CompC(40)));
//!
//!     // Each archetype now has one entity.
//!     assert_eq!(world.archetype::<ArchFoo>().len(), 1);
//!     assert_eq!(world.archetype::<ArchBar>().len(), 1);
//!
//!     // Look up each entity and check its CompB or CompC value.
//!     // We use the is_some() check here to make sure the entity was indeed found.
//!     assert!(ecs_find!(world, entity_a, |c: &CompB| assert_eq!(c.0, 20)).is_some());
//!     assert!(ecs_find!(world, entity_b, |c: &CompC| assert_eq!(c.0, 40)).is_some());
//!
//!     // Add to entity_a's CompA value.
//!     ecs_find!(world, entity_a, |c: &mut CompA| { c.0 += 1; });
//!
//!     // Sum both entities' CompA values with one iter despite being different archetypes.
//!     let mut sum = 0;
//!     ecs_iter!(world, |c: &CompA| { sum += c.0 });
//!     assert_eq!(sum, 5); // Adding 2 + 3 -- recall that we added 1 to entity_a's CompA.
//!
//!     // Collect both entities that have a CompA component.
//!     let mut found = Vec::new();
//!     ecs_iter!(world, |entity: &EntityAny, _: &CompA| { found.push(*entity); });
//!     assert!(found == vec![entity_a.into(), entity_b.into()]);
//!
//!     // Destroy both entities -- this will return an Option containing their components.
//!     assert!(world.destroy(entity_a).is_some());
//!     assert!(world.destroy(entity_b).is_some());
//!
//!     // Try to look up a stale entity handle -- this will return None.
//!     assert!(ecs_find!(world, entity_a, |_: &Entity<ArchFoo>| { panic!() }).is_none());
//! }
//! ```

mod archetype;
mod index;
mod util;

/// Handles for accessing entity representations as component data.
pub mod entity;

/// Error reporting for ECS operation failure.
pub mod error;

/// Traits for working with ECS types as generics.
pub mod traits;

/// A checked generational version.
pub mod version;

/// Enums for controlling iteration stepping.
pub mod iter;

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
    /// similar reasons, an ECS world can have only `256` distinct archetypes. Archetypes can
    /// store up to `16` distinct components by default. Use the `32_components` crate feature
    /// to raise this limit to `32` components -- note that this may impact compilation speed.
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
    /// ECS world struct. Without this declaration, the world's name will default to `EcsWorld`.
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
    ///     - A constant expression (e.g. `200` or `config::ARCH_CAPACITY + 4`). This will
    ///       create a fixed-size archetype that can contain at most that number of entities.
    ///     - The `dyn` keyword can be used to create a dynamically-sized archetype. This can
    ///       grow to accommodate up to `16,777,216` entities. To initialize an ECS world's
    ///       dynamic archetype with a pre-allocated capacity, use the `with_capacity()`
    ///       function at world creation. This function will be automatically generated
    ///       with a named capacity argument for each dynamic archetype in that world.
    /// - `Component, ...`: One or more component types to include in this archetype. Because
    ///   generated archetypes are `pub` with `pub` members, all components must be `pub` too.
    ///
    /// The `ecs_archetype!` declaration supports the following attributes:
    ///
    /// - `#[cfg]` attributes can be used both on the `ecs_archetype!` itself, and on
    ///   individual component parameters.
    /// - `#[archetype_id(N)]` can be used to override this archetype's `ARCHETYPE_ID` to `N`
    ///   (which must be between `0` and `255`). By default, archetype IDs start at `0` and
    ///   count up sequentially from the last value, similar to enum discriminants. No two
    ///   archetypes may have the same archetype ID (this is compiler-enforced).
    ///
    /// # Example
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// // Components must be `pub`, as the ECS world will re-export them in its archetypes.
    /// pub struct CompA(pub u32);
    /// pub struct CompB(pub u32);
    /// #[cfg(feature = "some_feature")] // CompC only exists if "some_feature" is enabled.
    /// pub struct CompC(pub u32);
    ///
    /// const BAR_CAPACITY: usize = 30;
    ///
    /// ecs_world! {
    ///     ecs_name!(MyWorld); // Set the type name of this ECS structure to MyWorld.
    ///
    ///     // Declare an archetype called ArchFoo with capacity 100 and two components.
    ///     ecs_archetype!(
    ///         ArchFoo,
    ///         100,
    ///         CompA, // Note: Type paths are not currently supported for components.
    ///         CompB,
    ///     );
    ///
    ///     // Declare ArchBar only if "some_feature" is enabled, otherwise it won't exist.
    ///     #[cfg(feature = "some_feature")]
    ///     ecs_archetype!(
    ///         ArchBar,
    ///         BAR_CAPACITY, // Constants may also be used for archetype capacity.
    ///         CompA,
    ///         CompC,
    ///     );
    ///    
    ///     #[archetype_id(6)]
    ///     ecs_archetype!(
    ///         ArchBaz,
    ///         dyn, // Use the dyn keyword for a dynamically-sized archetype.
    ///         CompA,
    ///         CompB,
    ///         #[cfg(feature = "some_feature")]
    ///         CompC, // ArchBaz will only have CompC if "some_feature" is enabled.
    ///     );
    /// }
    ///
    /// fn main() {
    ///     // Create a new world. Because ArchBaz is the only dynamic archetype, we only need to
    ///     // set one capacity in world creation (the parameter is named capacity_arch_baz). The
    ///     // other fixed-size archetypes will always be created sized to their full capacity.
    ///     let mut world = MyWorld::with_capacity(30);
    ///
    ///     // Create an ArchFoo entity in the world and unwrap the Option<Entity<ArchFoo>>.
    ///     // Alternatively, we could use .create(), which will panic if the archetype is full.
    ///     let entity_a = world.try_create::<ArchFoo>((CompA(0), CompB(1))).unwrap();
    ///
    ///     // The length of the archetype should now be 1.
    ///     assert_eq!(world.archetype::<ArchFoo>().len(), 1);
    ///
    ///     // Destroy the entity (we don't need to turbofish because this is an Entity<ArchFoo>).
    ///     world.destroy(entity_a);
    ///
    ///     assert_eq!(world.archetype::<ArchFoo>().len(), 0);
    ///     assert!(world.archetype::<ArchFoo>().is_empty());
    ///
    ///     // Use of #[cfg]-conditionals.
    ///     #[cfg(feature = "some_feature")] world.create::<ArchBar>((CompA(2), CompB(3), CompC(4)));
    ///     world.create::<ArchBaz>((CompA(5), CompB(6), #[cfg(feature = "some_feature")] CompC(7)));
    ///
    ///     // Use of #[archetype_id(N)] assignment.
    ///     assert_eq!(ArchFoo::ARCHETYPE_ID, 0);
    ///     assert_eq!(ArchBaz::ARCHETYPE_ID, 6);
    ///     #[cfg(feature = "some_feature")] assert_eq!(ArchBar::ARCHETYPE_ID, 1);
    /// }
    /// ```
    ///
    /// # Using an ECS World Across Modules
    ///
    /// The `ecs_world!` macro locally generates a number of archetypes and macros, including its
    /// own `ecs_find!` and `ecs_iter!` macros and their borrow equivalents. These are all added
    /// to the module scope where the `ecs_world!` invocation exists, and are all marked `pub`.
    /// If you want to use a generated ECS world in another module or crate, you must import not
    /// only the world struct, but its archetypes and macros. The recommended way to do this is to
    /// wrap your `ecs_world!` declaration in its own prelude-like module and then glob import it:
    ///
    /// ```
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA;
    /// pub struct CompB;
    ///
    /// pub mod my_world {
    ///     pub mod prelude {
    ///         // Pull in all the components we want to use as local identifiers.
    ///         use super::super::*;
    ///
    ///         ecs_world! {
    ///             ecs_archetype!(ArchFoo, 10, CompA, CompB);
    ///         }
    ///     }
    /// }
    ///
    /// // Pull the world from another module/crate into scope with its archetypes and macros.
    /// use my_world::prelude::*;
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///     let entity = world.create::<ArchFoo>((CompA, CompB));
    ///     assert!(ecs_find!(world, entity, || {}).is_some());
    /// }
    /// ```
    ///
    /// Note that `ecs_find!`, `ecs_iter!`, and their borrow equivalents are generated specific
    /// to each world, and are scoped to the location of the `ecs_world!` that generated them.
    /// If you need to have multiple distinct ECS worlds in the same scope, you will need to
    /// disambiguate between their query macros manually.
    #[macro_export]
    macro_rules! ecs_world {
        {...} => {};
    }

    /// Returns the compile-time ID of a given component in its archetype.
    ///
    /// ```ignore
    /// ecs_component_id!(Component);            // Can be used in a query body
    /// ecs_component_id!(Component, Archetype); // If used outside of a query
    /// ```
    ///
    /// The `ecs_component_id!` returns the compile-time ID (as a `u8`) of a given component in
    /// an archetype. If used in a query, the archetype parameter defaults to `MatchedArchetype`,
    /// which is the type alias automatically set for each query referencing the current matched
    /// archetype for the current execution of the query body.
    ///
    /// This is a const operation, and can be used to parameterize const generics.
    ///
    /// By default, component IDs are assigned sequentially starting at `0`, with a maximum of
    /// `255`. Component IDs can also be manually set using the `#[component_id(N)]` attribute
    /// on elements of the component list in an `ecs_archetype!` declaration within `ecs_world!`.
    /// Like enum discriminants, components without this attribute will count up from the last
    /// manually set ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gecs::prelude::*;
    ///
    /// pub struct CompA;
    /// pub struct CompB;
    /// pub struct CompC;
    ///
    /// ecs_world! {
    ///     ecs_archetype!(
    ///         ArchFoo,
    ///         5,
    ///         CompA, // = 0
    ///         CompC, // = 1
    ///     );
    ///
    ///     ecs_archetype!(
    ///         ArchBar,
    ///         5,
    ///         #[component_id(6)]
    ///         CompA, // = 6
    ///         CompB, // = 7 (Implicit)
    ///         CompC, // = 8 (Implicit)
    ///     );
    ///
    ///     ecs_archetype!(
    ///         ArchBaz,
    ///         5,
    ///         CompA, // = 0 (Implicit)
    ///         CompB, // = 1 (Implicit)
    ///         #[component_id(200)]
    ///         CompC, // = 200
    ///     );
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.archetype_mut::<ArchFoo>().create((CompA, CompC));
    ///     let entity_b = world.archetype_mut::<ArchBar>().create((CompA, CompB, CompC));
    ///     let entity_c = world.archetype_mut::<ArchBaz>().create((CompA, CompB, CompC));
    ///
    ///     ecs_find!(world, entity_a, |_: &CompC| {
    ///         assert_eq!(ecs_component_id!(CompC), 1);
    ///     });
    ///
    ///     ecs_find!(world, entity_b, |_: &CompC| {
    ///         assert_eq!(ecs_component_id!(CompC), 8);
    ///     });
    ///
    ///     ecs_find!(world, entity_c, |_: &CompC| {
    ///         assert_eq!(ecs_component_id!(CompC), 200);
    ///     });
    ///
    ///     assert_eq!(ecs_component_id!(CompC, ArchFoo), 1);
    ///     assert_eq!(ecs_component_id!(CompC, ArchBar), 8);
    ///     assert_eq!(ecs_component_id!(CompC, ArchBaz), 200);
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_component_id {
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
    /// - `entity`: The entity handle you want to look up. May be an `Entity<A>`, `EntityRaw<A>`,
    ///   `EntityAny`, or `EntityRawAny` handle.
    /// - `|comp_a: &CompA, comp_b: &mut CompB, ...| { ... }`: A closure containing the operation
    ///   to perform on the current entity's data. The parameters of the closure determine what
    ///   components for the entity that this query will access and how. Any component can be
    ///   accessed as `&Component` or `&mut Component`. The query will only check archetypes
    ///   that are known at compile-time to have all components requested in the query closure.
    ///     - Note that this closure is always treated as a `&mut FnMut`.
    ///
    /// The `ecs_find!` macro returns an `Option` type of the return value of the closure (which
    /// may be `Option<()>` if the closure has no return). The value will be `Some` if the entity
    /// was found, or `None` otherwise.
    ///
    /// # Special Arguments
    ///
    /// Query closure arguments can have the following special types:
    ///
    /// - `&Entity<A>`/`&EntityAny`: Returns the current entity being accessed by the closure.
    ///   This is somewhat redundant for `ecs_find!` queries, but useful for `ecs_iter!` loops.
    ///   Note that this is always read-only -- the entity can never be accessed mutably.
    /// - `&EntityRaw<A>`/`EntityRawAny`: As above, but using raw handles to the direct position
    ///   of the entity in its archetype. This can accelerate lookup, but may be invalidated
    ///   if the archetype changes. See [`EntityRawAny`] for more information.
    /// - `&Entity<_>`/`&EntityRaw<_>`: When used with the special `_` wildcard, each execution
    ///   of this query will return a typed (raw) entity handle for the exact archetype matched
    ///   for this specific execution. This can be used to optimize switched behavior by type.
    /// - `&OneOf<A, B, ...>` or `&mut OneOf<A, B, ...>`: See [`OneOf`](crate::OneOf).
    ///
    /// In query closures, a special `MatchedArchetype` type alias is set to the currently
    /// matched archetype being accessed during this execution of the closure. This can be used
    /// for generic operations.
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
    /// // If you need to use a non-mut reference, see the ecs_find_borrow! macro.
    /// fn add_three(world: &mut EcsWorld, entity: Entity<ArchFoo>) -> bool {
    ///     // The result will be true if the entity was found and operated on.
    ///     ecs_find!(world, entity, |comp_a: &mut CompA| { comp_a.0 += 3; }).is_some()
    /// }
    ///
    /// fn add_three_any(world: &mut EcsWorld, entity: EntityAny) -> bool {
    ///     // The query syntax is the same for both Entity<A> and EntityAny.
    ///     ecs_find!(world, entity, |comp_a: &mut CompA| { comp_a.0 += 3; }).is_some()
    /// }
    ///
    /// fn main() {
    ///     let mut world = EcsWorld::default();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(0), CompB(0)));
    ///     let entity_b = world.create::<ArchBar>((CompA(0), CompC(0)));
    ///
    ///     assert!(ecs_find!(world, entity_a, |c: &CompA| assert_eq!(c.0, 0)).is_some());
    ///     assert!(ecs_find!(world, entity_b, |c: &CompA| assert_eq!(c.0, 0)).is_some());
    ///
    ///     assert!(add_three(&mut world, entity_a));
    ///     assert!(add_three_any(&mut world, entity_b.into())); // Convert to an EntityAny
    ///
    ///     assert!(ecs_find!(world, entity_a, |c: &CompA| assert_eq!(c.0, 3)).is_some());
    ///     assert!(ecs_find!(world, entity_b, |c: &CompA| assert_eq!(c.0, 3)).is_some());
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
    ///     let mut world = EcsWorld::default();
    ///
    ///     let parent = world.create::<ArchFoo>((CompA(0), CompB(0), Parent(None)));
    ///     let child = world.create::<ArchFoo>((CompA(1), CompB(0), Parent(Some(parent))));
    ///
    ///     // Assert that we found the parent, and that its CompB value is 0.
    ///     assert!(ecs_find!(world, parent, |b: &CompB| assert_eq!(b.0, 0)).is_some());
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
    ///     // Assert that we found the parent, and that its CompB value is now 1.
    ///     assert!(ecs_find!(world, parent, |b: &CompB| assert_eq!(b.0, 1)).is_some());
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
    /// # Special Arguments
    ///
    /// Query closure arguments can have the following special types:
    ///
    /// - `&Entity<A>`/`&EntityAny`: Returns the current entity being accessed by the closure.
    ///   This is somewhat redundant for `ecs_find!` queries, but useful for `ecs_iter!` loops.
    ///   Note that this is always read-only -- the entity can never be accessed mutably.
    /// - `&EntityRaw<A>`/`EntityRawAny`: As above, but using raw handles to the direct position
    ///   of the entity in its archetype. This can accelerate lookup, but may be invalidated
    ///   if the archetype changes. See [`EntityRawAny`] for more information.
    /// - `&Entity<_>`/`&EntityRaw<_>`: When used with the special `_` wildcard, each execution
    ///   of this query will return a typed (raw) entity handle for the exact archetype matched
    ///   for this specific execution. This can be used to optimize switched behavior by type.
    /// - `&OneOf<A, B, ...>` or `&mut OneOf<A, B, ...>`: See [`OneOf`](crate::OneOf).
    ///
    /// In query closures, a special `MatchedArchetype` type alias is set to the currently
    /// matched archetype being accessed during this execution of the closure. This can be used
    /// for generic operations.
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
    ///     let mut world = EcsWorld::default();
    ///
    ///     let mut vec_a = Vec::<EntityAny>::new();
    ///     let mut vec_b = Vec::<EntityAny>::new();
    ///     let mut vec_c = Vec::<EntityAny>::new();
    ///
    ///     let entity_a = world.create::<ArchFoo>((CompA(0), CompB(0)));
    ///     let entity_b = world.create::<ArchBar>((CompA(0), CompC(0)));
    ///
    ///     // This iterates both ArchFoo and ArchBar since both have a CompA.
    ///     ecs_iter!(world, |entity: &EntityAny, a: &mut CompA| {
    ///         vec_a.push(*entity);
    ///         a.0 += 3; // Add 3 to both entities
    ///     });
    ///     assert!(vec_a == vec![entity_a.into(), entity_b.into()]);
    ///
    ///     // Even though both ArchFoo and ArchBar have a CompA, only ArchFoo can
    ///     // provide Entity<ArchFoo> handles, so this will only iterate that one.
    ///     ecs_iter!(world, |entity: &Entity<ArchFoo>, a: &mut CompA| {
    ///         vec_b.push((*entity).into());
    ///         a.0 += 3; // Add 3 to entity_a
    ///     });
    ///     assert!(vec_b == vec![entity_a.into()]);
    ///
    ///     // This iterates only ArchBar since ArchFoo does not have a CompC.
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
    /// See [`ecs_iter`] for more information on iter queries.
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
    /// See the example for [`ecs_find_borrow!`].
    #[macro_export]
    macro_rules! ecs_iter_borrow {
        (...) => {};
    }

    /// Variant of `ecs_iter!` that allows for destroying the current entity while iterating.
    ///
    /// See [`ecs_iter`] for more information on iter queries.
    ///
    /// This version works similarly to [`ecs_iter`], but with the `EcsStepDestroy` enum that
    /// supports two additional values: `BreakDestroy` and `ContinueDestroy`. These will break
    /// or continue and also immediately remove that entity after that iteration step. The
    /// entity and its handle are not preserved after this process. Note that this iterates the
    /// world in a different order from the normal `ecs_iter!` (which should not be relied upon
    /// for deterministic iteration anyway). This is also a bit slower than normal `ecs_iter!`.
    ///
    /// Note that performing an early-out break will end the iteration for all archetypes. This
    /// can have unpredictable order, and is recommended only for searching for single entities.
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
    ///     let mut world = EcsWorld::default();
    ///
    ///     world.archetype_mut::<ArchFoo>().create((CompA(1), CompB(10)));
    ///     world.archetype_mut::<ArchFoo>().create((CompA(2), CompB(20)));
    ///     world.archetype_mut::<ArchFoo>().create((CompA(3), CompB(30)));
    ///
    ///     world.archetype_mut::<ArchBar>().create((CompA(4), CompC(10)));
    ///     world.archetype_mut::<ArchBar>().create((CompA(5), CompC(10)));
    ///     world.archetype_mut::<ArchBar>().create((CompA(6), CompC(10)));
    ///
    ///     let mut vec_a = Vec::<u32>::new();
    ///     let mut vec_b = Vec::<u32>::new();
    ///
    ///     ecs_iter_destroy!(world, |comp_a: &CompA| {
    ///         if comp_a.0 & 1 == 0 {
    ///             vec_a.push(comp_a.0);
    ///             EcsStepDestroy::ContinueDestroy
    ///         } else {
    ///             EcsStepDestroy::Continue
    ///         }
    ///     });
    ///
    ///     ecs_iter!(world, |comp_a: &CompA| {
    ///         vec_b.push(comp_a.0);
    ///     });
    ///
    ///     assert_eq!(vec_a.iter().copied().sum::<u32>(), 2 + 4 + 6);
    ///     assert_eq!(vec_b.iter().copied().sum::<u32>(), 1 + 3 + 5);
    /// }
    /// ```
    #[macro_export]
    macro_rules! ecs_iter_destroy {
        (...) => {};
    }
}

/// A special parameter type for ECS query closures to match one of multiple components.
///
/// The `OneOf<A, B, C, ...>` pseudo-type argument to an ECS closure allows a query to match
/// exactly one of the given component types. The component can be accessed as a `&T` (or
/// `&mut T` for a `&mut OneOf<...>`), and is effectively duck-typed within the body of the
/// closure -- no traits or `where` clauses are needed to access elements of the component,
/// so long as the same element or method is available on all potential results of the `OneOf`
/// binding. If an archetype has more than one of the requested components in a `OneOf`, this
/// will result in a compilation error. This query will only match archetypes that have one of
/// the requested components.
///
/// ---
///
/// This is not a real struct and does not exist in any live code, it is a pseudo-type that
/// only has meaning within an ECS query closure when parsed by the operation macro. It is
/// presented here as a standalone struct for documentation purposes only.
///
/// # Example
///
/// ```rust
/// use gecs::prelude::*;
///
/// pub struct CompA(pub u32);
/// pub struct CompB(pub u32);
/// pub struct CompC(pub u32);
///
/// ecs_world! {
///     ecs_archetype!(ArchFoo, 5, CompA, CompB);
///     ecs_archetype!(ArchBar, 5, CompA, CompC);
/// }
///
/// fn main() {
///     let mut world = EcsWorld::default();
///
///     let entity_a = world.archetype_mut::<ArchFoo>().create((CompA(1), CompB(10)));
///     let entity_b = world.archetype_mut::<ArchBar>().create((CompA(1), CompC(10)));
///
///     let mut sum_a = 0;
///     let mut sum_b = 0;
///
///     // All three of these queries match both ArchFoo and ArchBar:
///     ecs_find!(world, entity_a, |v: &mut OneOf<CompB, CompC>| {
///         v.0 += 1;
///     });
///
///     ecs_find!(world, entity_b, |v: &mut OneOf<CompB, CompC>| {
///         v.0 += 1;
///     });
///
///     ecs_iter!(world, |u: &CompA, v: &OneOf<CompB, CompC>| {
///         sum_a += u.0;
///         sum_b += v.0;
///     });
///
///     assert_eq!(sum_a, 2);
///     assert_eq!(sum_b, 22);
/// }
/// ```
#[cfg(doc)]
pub struct OneOf {
    hidden: (),
}

#[cfg(not(doc))]
pub use gecs_macros::{ecs_component_id, ecs_world};

/// `use gecs::prelude::*;` to import common macros, traits, and types.
#[rustfmt::skip]
pub mod prelude {
    use super::*;

    pub use gecs_macros::{ecs_component_id, ecs_world};

    pub use entity::{ArchetypeId, Entity, EntityAny, EntityRaw, EntityRawAny};

    pub use iter::{EcsStepDestroy, EcsStep};

    pub use traits::EntityKey;
    pub use traits::{WorldCanResolve, ArchetypeCanResolve, StorageCanResolve};

    pub use traits::{World, WorldHas};
    pub use traits::{Archetype, ArchetypeHas};
    pub use traits::{View, ViewHas};
}

#[doc(hidden)]
#[rustfmt::skip]
pub mod __internal {
    use super::*;

    pub use gecs_macros::__ecs_finalize;
    pub use gecs_macros::{__ecs_find, __ecs_find_borrow};
    pub use gecs_macros::{__ecs_iter, __ecs_iter_borrow, __ecs_iter_destroy};

    pub use error::EcsError;

    pub use entity::__internal::*;

    pub use version::{VersionArchetype, VersionSlot};

    pub use archetype::slices::*;
    pub use archetype::storage_dynamic::*;
    pub use archetype::storage_fixed::*;
    pub use archetype::view::*;

    pub use iter::{EcsStepDestroy, EcsStep};

    pub use traits::EntityKey;
    pub use traits::{WorldCanResolve, ArchetypeCanResolve, StorageCanResolve};

    pub use traits::{World, WorldHas};
    pub use traits::{Archetype, ArchetypeHas};
    pub use traits::{View, ViewHas};
}
