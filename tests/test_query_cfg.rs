use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompB(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        #[cfg(any())] CompZ,
    );

    ecs_archetype!(
        ArchBar,
        CompB,
        #[cfg(any())] CompZ,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_query_cfg() {
    let mut world = EcsWorld::default();

    world.arch_foo.create((CompA(0), #[cfg(any())] CompZ,));
    world.arch_foo.create((CompA(1), #[cfg(any())] CompZ,));
    world.arch_foo.create((CompA(2), #[cfg(any())] CompZ,));
    world.arch_foo.create((CompA(3), #[cfg(any())] CompZ,));

    world.arch_bar.create((CompB(0), #[cfg(any())] CompZ,));

    let entity = world.arch_foo.create((CompA(4), #[cfg(any())] CompZ,));

    let mut sum = 0;
    ecs_iter!(world, |a: &CompA, #[cfg(any())] b: &CompZ| {
        sum += a.0;
    });

    let foo = ecs_find!(world, entity, |#[cfg(any())] e: &EntityAny, a: &CompA, #[cfg(any())] b: &CompZ| {
        sum += a.0;
        1234
    });

    let mut matches = 0;
    ecs_iter!(world, |#[cfg(any())] e: &Entity<ArchFoo>| { // Should hit all entities
        matches += 1;
    });

    assert_eq!(foo, Some(1234));
    assert_eq!(sum, 1 + 2 + 3 + 4 + 4);
    assert_eq!(matches, 6);
    assert_eq!(world.arch_foo.len(), 5);
}
