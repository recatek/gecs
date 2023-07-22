use gecs::prelude::*;

#[derive(Debug, PartialEq)]
pub struct CompA(pub u32);
#[derive(Debug, PartialEq)]
pub struct CompB(pub u32);
#[derive(Debug, PartialEq)]
pub struct CompC(pub u32);

ecs_world! {
    #[archetype_id(3)]
    ecs_archetype!(
        ArchFoo,
        5,
        CompA,
        CompB,
    );

    ecs_archetype!(
        ArchBar,
        5,
        CompA,
        CompC,
    );
}

#[test]
#[rustfmt::skip]
fn test_one_of_basic() {
    let mut world = EcsWorld::default();

    let entity_a = world.archetype_mut::<ArchFoo>().create((CompA(1), CompB(10)));
    let entity_b = world.archetype_mut::<ArchBar>().create((CompA(1), CompC(10)));

    let mut sum_a = 0;
    let mut sum_b = 0;

    ecs_find!(world, entity_a, |v: &mut OneOf<CompB, CompC>| {
        v.0 += 1;
    });

    ecs_find!(world, entity_b, |v: &mut OneOf<CompB, CompC>| {
        v.0 += 1;
    });

    ecs_iter!(world, |u: &CompA, v: &OneOf<CompB, CompC>| {
        sum_a += u.0;
        sum_b += v.0;
    });

    assert_eq!(sum_a, 2);
    assert_eq!(sum_b, 22);
}
