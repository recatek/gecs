use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompG<T>(T);

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        #[component_id(10)]
        CompG<u32>,
    );

    ecs_archetype!(
        ArchBar,
        CompA,
        #[component_id(20)]
        CompG<u16>,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_generic_components() {
    let mut world = EcsWorld::default();

    world.create::<ArchFoo>((CompA(0), CompG(0_u32),));
    world.create::<ArchFoo>((CompA(1), CompG(0_u32),));
    world.create::<ArchFoo>((CompA(2), CompG(0_u32),));
    world.create::<ArchFoo>((CompA(3), CompG(0_u32),));
    world.create::<ArchFoo>((CompA(4), CompG(0_u32),));

    world.create::<ArchBar>((CompA(5), CompG(0_u16),));

    let mut vec = Vec::new();

    ecs_iter!(world, |a: &CompA, _: &CompG<u32>| {
        debug_assert!(ecs_component_id!(CompG<u32>) == 10);

        vec.push(a.0);
    });

    assert_eq!(vec.iter().sum::<u32>(), vec![0, 1, 2, 3, 4].iter().sum());

    vec.clear();

    ecs_iter!(world, |a: &CompA, _: &CompG<u16>| {
        debug_assert!(ecs_component_id!(CompG<u16>) == 20);

        vec.push(a.0);
    });

    assert_eq!(vec.iter().sum::<u32>(), vec![5].iter().sum());

    vec.clear();

    ecs_iter!(world, |a: &CompA, _: &OneOf<CompG<u32>, CompG<u16>>| {
        vec.push(a.0);
    });

    assert_eq!(vec.iter().sum::<u32>(), vec![0, 1, 2, 3, 4, 5].iter().sum());
}
