use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        CompZ,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_query_cfg() {
    let mut world = EcsWorld::default();

    world.arch_foo.create((CompA(0), CompZ,));
    world.arch_foo.create((CompA(1), CompZ,));
    world.arch_foo.create((CompA(2), CompZ,));
    world.arch_foo.create((CompA(3), CompZ,));
    let entity = world.arch_foo.create((CompA(4), CompZ,));

    let mut sum = 0;
    ecs_iter!(world, |a: &CompA, #[cfg(any())] b: &CompZ| {
        sum += a.0;
    });

    ecs_find!(world, entity, |a: &CompA, #[cfg(any())] b: &CompZ| {
        sum += a.0;
    });

    assert_eq!(sum, 1 + 2 + 3 + 4 + 4);
    assert_eq!(world.arch_foo.len(), 5);
}
