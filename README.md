gecs 🦎
-------
A generated entity component system.

[![Documentation](https://docs.rs/gecs/badge.svg)](https://docs.rs/gecs/)
[![Crates.io](https://img.shields.io/crates/v/gecs.svg)](https://crates.io/crates/gecs)

The gecs crate provides a compile-time generated, zero-overhead ECS for simulations
on a budget. Unlike other ECS libraries, gecs takes a full ECS world structure
definition from code and precompiles all queries to achieve better performance with
no upfront cost or caching overhead. Queries in gecs can be inspected and checked at
compile-time in order to catch what would otherwise be bugs presenting only in tests
or execution. However, this comes at the cost of requiring all archetypes to be known
and declared at compile-time, so that adding or removing components from entities at
runtime isn't currently possible -- hybrid approaches could solve this in the future.

Archetypes in gecs can be set to contain a fixed or dynamic capacity of entities. If
all of the archetypes in your ECS world declaration are set to a fixed capacity, gecs
will perform zero allocations after startup. This guarantees that your ECS world will
adhere to a known and predictable memory overhead for constrained environments (e.g.
servers on cloud instances). Attempting to add an entity to a full archetype can
either report failure or panic depending on the method you call to do so.

The goals for gecs are (in descending priority order):
- Fast iteration and find queries
- Fast entity creation and destruction
- Low, predictable memory overhead
- A user-friendly library interface
- Simplicity and focus in features

All of the code that gecs generates in user crates is safe, and users of gecs can
use `#[deny(unsafe_code)]` in their own crates. Note that gecs does use unsafe code
internally to allow for compiler optimizations around known invariants. It is not a
goal of this library to be written entirely in safe Rust.

# Getting Started

See the `ecs_world!`, `ecs_find!`, and `ecs_iter!` macros for more information.
The following example creates a world with three components and two archetypes:

```rust
use gecs::prelude::*;

// Components -- these must be pub because the world is exported as pub as well.
pub struct CompA(pub u32);
pub struct CompB(pub u32);
pub struct CompC(pub u32);

ecs_world! {
    // Declare two archetypes, ArchFoo and ArchBar.
    ecs_archetype!(ArchFoo, 100, CompA, CompB); // Fixed capacity of 100 entities.
    ecs_archetype!(ArchBar, dyn, CompA, CompC); // Dynamic (dyn) entity capacity.
}

fn main() {
    let mut world = EcsWorld::default(); // Initialize an empty new ECS world.

    // Add entities to the world by populating their components and receive their handles.
    let entity_a = world.create::<ArchFoo>((CompA(1), CompB(20)));
    let entity_b = world.create::<ArchBar>((CompA(3), CompC(40)));

    // Each archetype now has one entity.
    assert_eq!(world.archetype::<ArchFoo>().len(), 1);
    assert_eq!(world.archetype::<ArchBar>().len(), 1);

    // Look up each entity and check its CompB or CompC value.
    // We use the is_some() check here to make sure the entity was indeed found.
    assert!(ecs_find!(world, entity_a, |c: &CompB| assert_eq!(c.0, 20)).is_some());
    assert!(ecs_find!(world, entity_b, |c: &CompC| assert_eq!(c.0, 40)).is_some());

    // Add to entity_a's CompA value.
    ecs_find!(world, entity_a, |c: &mut CompA| { c.0 += 1; });

    // Sum both entities' CompA values with one iter despite being different archetypes.
    let mut sum = 0;
    ecs_iter!(world, |c: &CompA| { sum += c.0 });
    assert_eq!(sum, 5); // Adding 2 + 3 -- recall that we added 1 to entity_a's CompA.

    // Collect both entities that have a CompA component.
    let mut found = Vec::new();
    ecs_iter!(world, |entity: &EntityAny, _: &CompA| { found.push(*entity); });
    assert!(found == vec![entity_a.into(), entity_b.into()]);

    // Destroy both entities -- this will return an Option containing their components.
    assert!(world.destroy(entity_a).is_some());
    assert!(world.destroy(entity_b).is_some());

    // Try to look up a stale entity handle -- this will return None.
    assert!(ecs_find!(world, entity_a, |_: &Entity<ArchFoo>| { panic!() }).is_none());
}
```
License
-------

This library may be used under your choice of the Apache 2.0 or MIT license.
