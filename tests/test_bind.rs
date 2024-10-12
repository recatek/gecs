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

    ecs_iter!(world, |_: &EntityRaw<ArchFoo>| {});
    ecs_iter_borrow!(world, |_: &EntityRaw<ArchFoo>| {});
    ecs_find!(world, entity, |_: &EntityRaw<ArchFoo>| {});
    ecs_find_borrow!(world, entity, |_: &EntityRaw<ArchFoo>| {});
    ecs_find!(world, entity.into_any(), |_: &EntityRaw<ArchFoo>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityRaw<ArchFoo>| {});

    ecs_iter!(world, |_: &EntityRaw<_>| {});
    ecs_iter_borrow!(world, |_: &EntityRaw<_>| {});
    ecs_find!(world, entity, |_: &EntityRaw<_>| {});
    ecs_find_borrow!(world, entity, |_: &EntityRaw<_>| {});
    ecs_find!(world, entity.into_any(), |_: &EntityRaw<_>| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityRaw<_>| {});

    ecs_iter!(world, |_: &EntityAny| {});
    ecs_iter_borrow!(world, |_: &EntityAny| {});
    ecs_find!(world, entity, |_: &EntityAny| {});
    ecs_find_borrow!(world, entity, |_: &EntityAny| {});
    ecs_find!(world, entity.into_any(), |_: &EntityAny| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityAny| {});

    ecs_iter!(world, |_: &EntityRawAny| {});
    ecs_iter_borrow!(world, |_: &EntityRawAny| {});
    ecs_find!(world, entity, |_: &EntityRawAny| {});
    ecs_find_borrow!(world, entity, |_: &EntityRawAny| {});
    ecs_find!(world, entity.into_any(), |_: &EntityRawAny| {});
    ecs_find_borrow!(world, entity.into_any(), |_: &EntityRawAny| {});
}
