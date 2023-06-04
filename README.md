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
and declared at compile-time, so that adding or removing components to entities at
runtime isn't currently possible -- hybrid approaches could fix this in the future.

Archetypes in gecs can be set to contain a fixed capacity of entities. If all of the
archetypes in your ECS world declaration are configured in this way, gecs will perform
zero allocations after startup. This guarantees that your ECS world will adhere to a
known and predictable memory overhead for constrained environments (e.g. servers on
cloud instances). Attempting to create a new entity in a full archetype will return
`None` (no panics). Support for dynamically-sized archetypes with `Vec`-like storage
behavior is planned for support at a later date but is not currently implemented.

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

License
-------

This library may be used under your choice of the Apache 2.0 or MIT license.
