use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompG<T>(T);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        #[component_id(10)] CompG<u32>,
        CompZ,
    );

    ecs_archetype!(
        ArchBar,
        CompA,
        #[component_id(20)] CompG<u16>,
        CompZ,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_generic_components() {
    let mut world = EcsWorld::default();

    world.create::<ArchFoo>((CompA(0), CompG(10_u32), CompZ,));
    world.create::<ArchFoo>((CompA(1), CompG(20_u32), CompZ,));
    world.create::<ArchFoo>((CompA(2), CompG(30_u32), CompZ,));
    world.create::<ArchFoo>((CompA(3), CompG(40_u32), CompZ,));
    world.create::<ArchFoo>((CompA(4), CompG(50_u32), CompZ,));

    world.create::<ArchBar>((CompA(5), CompG(60_u16), CompZ,));

    let mut sum = 0_usize;

    ecs_iter!(world, |a: &CompA, _: &CompG<u32>| {
        debug_assert!(ecs_component_id!(CompG<u32>) == 10);

        sum += a.0 as usize;
    });

    assert_eq!(sum, 0 + 1 + 2 + 3 + 4);

    sum = 0;

    ecs_iter!(world, |a: &CompA, _: &CompG<u16>| {
        debug_assert!(ecs_component_id!(CompG<u16>) == 20);

        sum += a.0 as usize;
    });

    assert_eq!(sum, 5);

    sum = 0;

    ecs_iter!(world, |a: &CompA, _: &OneOf<CompG<u32>, CompG<u16>>| {
        sum += a.0 as usize;
    });

    assert_eq!(sum, 0 + 1 + 2 + 3 + 4 + 5);

    sum = 0;

    ecs_iter!(world, |g: &CompG<_>| {
        sum += g.0 as usize;
    });

    assert_eq!(sum, 10 + 20 + 30 + 40 + 50 + 60);

    sum = 0;

    ecs_iter!(world, |g: &OneOf<CompG<_>>| {
        sum += g.0 as usize;
    });

    assert_eq!(sum, 10 + 20 + 30 + 40 + 50 + 60);
}
