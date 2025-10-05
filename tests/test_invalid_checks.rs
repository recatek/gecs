use gecs::prelude::*;

pub struct CompA;

mod world1 {
    use super::*;

    ecs_world! {
        #[archetype_id(20)]
        ecs_archetype!(ArchFoo, CompA);
        #[archetype_id(30)]
        ecs_archetype!(ArchBar, CompA);
    }
}

mod world2 {
    use super::*;

    ecs_world! {
        #[archetype_id(25)]
        ecs_archetype!(ArchBaz, CompA);
    }
}

#[test]
fn test_zero_version() {
    let raw = (0u32, 0u32);
    let res = EntityAny::from_raw(raw);
    assert!(matches!(res, Err(EcsError::InvalidRawEntity)));
}

#[test]
#[should_panic]
fn test_invalid_archetype_id_1() {
    // Multiple worlds are useful for creating bad entity keys
    let mut world1 = world1::EcsWorld::default();
    let mut world2 = world2::EcsWorld::default();

    let entity = world2.create::<world2::ArchBaz>((CompA,)).into_any();

    // This should panic due to invalid archetype ID
    world1.view(entity);
}

#[test]
fn test_invalid_archetype_id_2() {
    // Multiple worlds are useful for creating bad entity keys
    let mut world2 = world2::EcsWorld::default();

    let entity = world2.create::<world2::ArchBaz>((CompA,)).into_any();

    let select: Result<world1::SelectEntity, _> = entity.try_into();
    assert!(select.is_err());
}
