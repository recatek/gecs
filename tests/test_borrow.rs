use gecs::prelude::*;

#[derive(Debug, PartialEq)]
pub struct CompA(pub u32);
#[derive(Debug, PartialEq)]
pub struct CompB(pub u32);
#[derive(Debug, PartialEq)]
pub struct CompC(pub u32);

ecs_world! {
    ecs_archetype!(ArchFoo, CompA, CompB);
    ecs_archetype!(ArchBar, CompA, CompC);
}

#[test]
fn test_borrow() {
    let mut world = EcsWorld::default();

    let entity_a = world.create::<ArchFoo>((CompA(1), CompB(10)));
    let _entity_b = world.create::<ArchFoo>((CompA(2), CompB(20)));
    let entity_c = world.create::<ArchBar>((CompA(30), CompC(300)));

    let borrow_a = world.borrow(entity_a).unwrap();
    let borrow_c = world.borrow(entity_c).unwrap();

    borrow_a.component_mut::<CompA>().0 += 10;
    borrow_a.component_mut::<CompB>().0 += 10;
    borrow_c.component_mut::<CompA>().0 += 10;
    borrow_c.component_mut::<CompC>().0 += 10;

    assert_eq!(&*borrow_a.component::<CompA>(), &CompA(11));
    assert_eq!(&*borrow_a.component::<CompB>(), &CompB(20));
    assert_eq!(&*borrow_c.component::<CompA>(), &CompA(40));
    assert_eq!(&*borrow_c.component::<CompC>(), &CompC(310));
}
