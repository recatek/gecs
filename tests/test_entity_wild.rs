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
        CompA,
        CompB,
    );

    ecs_archetype!(
        ArchBar,
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

    ecs_iter!(world, |entity: &Entity<_>| {
        match entity.into() {
            ArchetypeSelectEntity::ArchFoo(entity) => check_entity_type_a(entity),
            ArchetypeSelectEntity::ArchBar(entity) => check_entity_type_b(entity),
        }
    });

    ecs_find!(world, entity_a, |entity: &Entity<_>, _: &CompB| {
        check_entity_type_a(*entity);
    });

    ecs_find!(world, entity_b, |entity: &Entity<_>, _: &CompC| {
        check_entity_type_b(*entity);
    });

    ecs_iter!(world, |entity: &Entity<_>, _: &CompB| {
        check_entity_type_a(*entity);
    });

    ecs_iter!(world, |entity: &Entity<_>, _: &CompC| {
        check_entity_type_b(*entity);
    });
}

fn check_entity_type_a(_: Entity<ArchFoo>) {}
fn check_entity_type_b(_: Entity<ArchBar>) {}
