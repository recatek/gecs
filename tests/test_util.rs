use gecs::prelude::*;

pub struct CompA;
pub struct CompB;
pub struct CompC;

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA, // = 0
        CompC, // = 1
    );

    ecs_archetype!(
        ArchBar,
        #[component_id(6)]
        CompA, // = 6
        CompB, // = 7 (Implicit)
        CompC, // = 8 (Implicit)
    );

    ecs_archetype!(
        ArchBaz,
        CompA, // = 0 (Implicit)
        CompB, // = 1 (Implicit)
        #[component_id(200)]
        CompC, // = 200
    );
}

#[test]
#[rustfmt::skip]
fn test_component_id() {
    let mut world = EcsWorld::default();

    let entity_a = world.archetype_mut::<ArchFoo>().create((CompA, CompC));
    let entity_b = world.archetype_mut::<ArchBar>().create((CompA, CompB, CompC));
    let entity_c = world.archetype_mut::<ArchBaz>().create((CompA, CompB, CompC));

    ecs_find!(world, entity_a, |_: &CompC| {
        assert_eq!(ecs_component_id!(CompC), 1);
    });

    ecs_find!(world, entity_b, |_: &CompC| {
        assert_eq!(ecs_component_id!(CompC), 8);
    });

    ecs_find!(world, entity_c, |_: &CompC| {
        assert_eq!(ecs_component_id!(CompC), 200);
    });

    assert_eq!(ecs_component_id!(CompC, ArchFoo), 1);
    assert_eq!(ecs_component_id!(CompC, ArchBar), 8);
    assert_eq!(ecs_component_id!(CompC, ArchBaz), 200);
}
