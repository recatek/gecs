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

pub fn test_bind() {
    let mut world = EcsWorld::default();

    let entity = world.archetype_mut::<ArchFoo>().create((CompA(1), CompZ));

    ecs_iter!(world, |_: &Entity<ArchFoo>| {});
    ecs_iter_borrow!(world, |_: &Entity<ArchFoo>| {});
    ecs_find!(world, entity, |_: &Entity<ArchFoo>| {});
    ecs_find_borrow!(world, entity, |_: &Entity<ArchFoo>| {});
    ecs_find!(world, entity.into_any(), |_: &Entity<ArchFoo>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &Entity<ArchFoo>| {});

    ecs_iter!(world, |_: &Entity<_>| {});
    ecs_iter_borrow!(world, |_: &Entity<_>| {});
    ecs_find!(world, entity, |_: &Entity<_>| {});
    ecs_find_borrow!(world, entity, |_: &Entity<_>| {});
    ecs_find!(world, entity.into_any(), |_: &Entity<_>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &Entity<_>| {});

    ecs_iter!(world, |_: &EntityDirect<ArchFoo>| {});
    ecs_iter_borrow!(world, |_: &EntityDirect<ArchFoo>| {});
    ecs_find!(world, entity, |_: &EntityDirect<ArchFoo>| {});
    ecs_find_borrow!(world, entity, |_: &EntityDirect<ArchFoo>| {});
    ecs_find!(world, entity.into_any(), |_: &EntityDirect<ArchFoo>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityDirect<ArchFoo>| {});

    ecs_iter!(world, |_: &EntityDirect<_>| {});
    ecs_iter_borrow!(world, |_: &EntityDirect<_>| {});
    ecs_find!(world, entity, |_: &EntityDirect<_>| {});
    ecs_find_borrow!(world, entity, |_: &EntityDirect<_>| {});
    ecs_find!(world, entity.into_any(), |_: &EntityDirect<_>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityDirect<_>| {});

    ecs_iter!(world, |_: &EntityAny| {});
    ecs_iter_borrow!(world, |_: &EntityAny| {});
    ecs_find!(world, entity, |_: &EntityAny| {});
    ecs_find_borrow!(world, entity, |_: &EntityAny| {});
    ecs_find!(world, entity.into_any(), |_: &EntityAny| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityAny| {});

    ecs_iter!(world, |_: &EntityDirectAny| {});
    ecs_iter_borrow!(world, |_: &EntityDirectAny| {});
    ecs_find!(world, entity, |_: &EntityDirectAny| {});
    ecs_find_borrow!(world, entity, |_: &EntityDirectAny| {});
    ecs_find!(world, entity.into_any(), |_: &EntityDirectAny| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityDirectAny| {});
}
