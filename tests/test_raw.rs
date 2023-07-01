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
        dyn,
        CompA,
        CompB,
    );

    ecs_archetype!(
        ArchBar,
        dyn,
        CompA,
        CompC,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_one_of_basic() {
    let mut world = World::default();

    let entity_a = world.create::<ArchFoo>((CompA(1), CompB(10)));
    let entity_b = world.create::<ArchBar>((CompA(1), CompC(10)));

    let mut entity_raw = None;
    let found = ecs_find!(world, entity_a, |raw: &EntityRaw<ArchFoo>| -> u32 {
        entity_raw = Some(*raw);
        6
    });

    assert!(found == Some(6));

    test_view_asm1(&mut world, entity_a, 3);
    test_view_asm2(&mut world, entity_a, 3);
    test_view_asm3(&mut world, 3);
    test_view_asm4(&mut world, 3);
}

#[inline(never)]
pub fn test_view_asm1(world: &mut World, entity: Entity<ArchFoo>, value: u32) {
    ecs_find!(world, entity, |a: &mut CompA| { a.0 = value });
}

#[inline(never)]
pub fn test_view_asm2(world: &mut World, entity: Entity<ArchFoo>, value: u32) {
    ecs_find_borrow!(world, entity, |a: &mut CompA| { a.0 = value });
}

#[inline(never)]
pub fn test_view_asm3(world: &mut World, value: u32) {
    ecs_iter!(world, |a: &mut CompA| { a.0 = value });
}

#[inline(never)]
pub fn test_view_asm4(world: &mut World, value: u32) {
    ecs_iter_borrow!(world, |a: &mut CompA| { a.0 = value });
}
