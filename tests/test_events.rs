use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(ArchFoo, CompA, CompZ);
}

#[test]
#[cfg(feature = "events")]
fn test_events_repeat() {
    let mut world = EcsWorld::default();

    let entity = world.create::<ArchFoo>((CompA(0), CompZ));
    world.destroy(entity);

    let events = world.drain_events().collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![
            EcsEvent::Created(entity.into()),
            EcsEvent::Destroyed(entity.into())
        ]
    );
    assert!(world.drain_events().collect::<Vec<_>>().is_empty());

    // Try it a second time to make sure we make new events
    let entity = world.create::<ArchFoo>((CompA(0), CompZ));
    world.destroy(entity);

    let events = world.drain_events().collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![
            EcsEvent::Created(entity.into()),
            EcsEvent::Destroyed(entity.into())
        ]
    );
    assert!(world.drain_events().collect::<Vec<_>>().is_empty());
}
