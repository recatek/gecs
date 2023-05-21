gecs 🦎
-------
A generated entity component system.

[![Documentation](https://docs.rs/gecs/badge.svg)](https://docs.rs/gecs/)
[![Crates.io](https://img.shields.io/crates/v/gecs.svg)](https://crates.io/crates/gecs)

The gecs crate provides a compile-time generated, zero-overhead ECS for simulations on a budget. Compared to other ECS libraries, gecs precompiles queries using known archetype configurations to achieve better performance with no upfront cost or caching. Queries in gecs can also be checked at compile-time, in order to catch what would otherwise be bugs presenting only in tests or execution. This comes at the cost of requiring all archetypes to be declared at compile-time, so that adding or removing components at runtime to entities isn't currently possible (though hybrid approaches may be explored in the future).

License
-------

This library may be used under your choice of the Apache 2.0 or MIT license.
