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
pub fn test_single_iter() {
    let mut world = EcsWorld::default();

    world.arch_foo.create((CompA(0), CompZ,));
    world.arch_foo.create((CompA(1), CompZ,));
    world.arch_foo.create((CompA(2), CompZ,));
    world.arch_foo.create((CompA(3), CompZ,));
    world.arch_foo.create((CompA(4), CompZ,));

    let mut vec = Vec::new();

    for (_, a, _) in world.arch_foo.iter() {
        vec.push(a.0);
    }

    assert_eq!(vec, vec![0, 1, 2, 3, 4]);

    vec.clear();

    for (_, a, _) in world.arch_foo.iter_mut() {
        a.0 += 1;
    }

    for (_, a, _) in world.arch_foo.iter() {
        vec.push(a.0);
    }

    assert_eq!(vec, vec![1, 2, 3, 4, 5]);
}
