use gecs::prelude::*;

#[derive(Debug, PartialEq)]
pub struct CompA(pub u32);

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
    );
}

#[test]
fn test_raw() {
    let mut world = EcsWorld::default();

    // Filler to check for bad region access
    world.create::<ArchFoo>((CompA(0xFEEEEEED),));

    let entity = world.create::<ArchFoo>((CompA(0xDEADBEEF),));

    // Filler to check for bad region access
    world.create::<ArchFoo>((CompA(0xBEEEEEEF),));

    assert_eq!(ecs_find!(world, entity, |a: &CompA| a.0), Some(0xDEADBEEF));

    let entity = EntityAny::from_raw(entity.into_any().raw()).unwrap();
    assert_eq!(ecs_find!(world, entity, |a: &CompA| a.0), Some(0xDEADBEEF));
}
