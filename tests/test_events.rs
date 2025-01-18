use gecs::prelude::*;

pub struct CompA;

ecs_world! {
    ecs_archetype!(ArchFoo, CompA);
    ecs_archetype!(ArchBar, CompA);
    ecs_archetype!(ArchBaz, CompA);
    ecs_archetype!(ArchQux, CompA);
}

#[test]
#[cfg(feature = "events")]
fn test_size_hint() {
    let mut world = EcsWorld::new();

    for _ in 0..4 {
        world.create::<ArchFoo>((CompA,));
    }

    for _ in 0..11 {
        world.create::<ArchBar>((CompA,));
    }

    for _ in 0..23 {
        world.create::<ArchBaz>((CompA,));
    }

    for _ in 0..6 {
        world.create::<ArchQux>((CompA,));
    }

    assert_eq!(world.iter_created().count(), 4 + 11 + 23 + 6);

    let (min, max) = world.iter_created().size_hint();
    assert!(max.is_some_and(|max| max >= min));
    assert_eq!(min, 4 + 11 + 23 + 6);
    assert_eq!(max, Some(4 + 11 + 23 + 6));


    let mut iter = world.iter_created();

    for _ in 0..10 {
        iter.next();
    }

    let (min, max) = iter.size_hint();
    assert!(max.is_some_and(|max| max >= min));
    assert_eq!(min, 4 + 11 + 23 + 6 - 10);
    assert_eq!(max, Some(4 + 11 + 23 + 6 - 10));
}
