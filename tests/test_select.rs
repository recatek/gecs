use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompB(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        CompZ,
    );

    ecs_archetype!(
        ArchBar,
        CompB,
        CompZ,
    );
}

#[test]
fn test_select() {
    let mut world = EcsWorld::default();

    let entity: Entity<ArchFoo> = world.archetype_mut::<ArchFoo>().create((CompA(1), CompZ));

    match entity.try_into() {
        Ok(SelectEntity::ArchFoo(entity)) => assert!(world.contains(entity)),
        Ok(SelectEntity::ArchBar(_)) => panic!(),
        Err(_) => panic!(),
    };
}

#[test]
fn test_select_direct() {
    let mut world = EcsWorld::default();

    let entity = world.archetype_mut::<ArchFoo>().create((CompA(1), CompZ));
    let entity: EntityDirect<ArchFoo> = world.resolve_direct(entity).unwrap();

    match entity.try_into() {
        Ok(SelectEntityDirect::ArchFoo(entity)) => assert!(world.contains(entity)),
        Ok(SelectEntityDirect::ArchBar(_)) => panic!(),
        Err(_) => panic!(),
    };
}

#[test]
fn test_select_any() {
    let mut world = EcsWorld::default();

    let entity: EntityAny = world
        .archetype_mut::<ArchFoo>()
        .create((CompA(1), CompZ))
        .into_any();

    match entity.try_into() {
        Ok(SelectEntity::ArchFoo(entity)) => assert!(world.contains(entity)),
        Ok(SelectEntity::ArchBar(_)) => panic!(),
        Err(_) => panic!(),
    };
}

#[test]
fn test_select_any_direct() {
    let mut world = EcsWorld::default();

    let entity = world.archetype_mut::<ArchFoo>().create((CompA(1), CompZ));
    let entity: EntityDirectAny = world.resolve_direct(entity).unwrap().into_any();

    match entity.try_into() {
        Ok(SelectEntityDirect::ArchFoo(entity)) => assert!(world.contains(entity)),
        Ok(SelectEntityDirect::ArchBar(_)) => panic!(),
        Err(_) => panic!(),
    };
}
