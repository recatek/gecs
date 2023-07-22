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
pub fn test_raw_basic() {
    let mut world = EcsWorld::default();

    let entity_a = world.create::<ArchFoo>((CompA(1), CompB(10)));
    let entity_b = world.create::<ArchBar>((CompA(2), CompC(20)));

    let entity_raw_a = ecs_find!(world, entity_a, |raw: &EntityRaw<ArchFoo>| {
        *raw
    }).unwrap();

    let entity_raw_b = ecs_find!(world, entity_b, |raw: &EntityRaw<ArchBar>| {
        *raw
    }).unwrap();

    assert!(ecs_find!(world, entity_raw_a, |a: &CompA, b: &CompB| {
        assert_eq!(a.0, 1);
        assert_eq!(b.0, 10);
    }).is_some());

    assert!(ecs_find!(world, entity_raw_b, |a: &CompA, c: &CompC| {
        assert_eq!(a.0, 2);
        assert_eq!(c.0, 20);
    }).is_some());

    // Adding a new entity doesn't invalidate dense indices
    let entity_c = world.create::<ArchFoo>((CompA(3), CompB(30)));
    let entity_d = world.create::<ArchBar>((CompA(4), CompC(40)));

    assert!(ecs_find!(world, entity_raw_a, |a: &CompA, b: &CompB| {
        assert_eq!(a.0, 1);
        assert_eq!(b.0, 10);
    }).is_some());

    assert!(ecs_find!(world, entity_raw_b, |a: &CompA, c: &CompC| {
        assert_eq!(a.0, 2);
        assert_eq!(c.0, 20);
    }).is_some());

    // Destroying an entity invalidates dense indices
    world.destroy(entity_c);
    world.destroy(entity_d);

    assert!(ecs_find!(world, entity_raw_a, |_: &CompA| {}).is_none());
    assert!(ecs_find!(world, entity_raw_b, |_: &CompA| {}).is_none());
}
