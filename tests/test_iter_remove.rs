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

    world.archetype_mut::<ArchFoo>().create((CompA(1), CompB(10)));
    world.archetype_mut::<ArchFoo>().create((CompA(2), CompB(20)));
    world.archetype_mut::<ArchFoo>().create((CompA(3), CompB(30)));

    world.archetype_mut::<ArchBar>().create((CompA(4), CompC(10)));
    world.archetype_mut::<ArchBar>().create((CompA(5), CompC(10)));
    world.archetype_mut::<ArchBar>().create((CompA(6), CompC(10)));

    let mut vec_a = Vec::<u32>::new();
    let mut vec_b = Vec::<u32>::new();

    ecs_iter_remove!(world, |comp_a: &CompA| {
        if comp_a.0 & 1 == 0 {
            vec_a.push(comp_a.0);
            true
        } else {
            false
        }
    });

    ecs_iter!(world, |comp_a: &CompA| {
        vec_b.push(comp_a.0);
    });

    assert_eq!(vec_a.iter().copied().sum::<u32>(), 2 + 4 + 6);
    assert_eq!(vec_b.iter().copied().sum::<u32>(), 1 + 3 + 5);
}
