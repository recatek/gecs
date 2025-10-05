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
fn test_select_view() {
    let mut world = EcsWorld::default();

    let entity_a = world.create::<ArchFoo>((CompA(1), CompB(10)));
    let entity_b = world.create::<ArchBar>((CompA(2), CompC(20)));

    let entity_any_a = entity_a.into_any();
    let entity_any_b = entity_b.into_any();

    match world.view(entity_any_a).unwrap() {
        SelectView::ArchFoo(view) => {
            assert_eq!(view.component::<CompA>(), &CompA(1));
            assert_eq!(view.component::<CompB>(), &CompB(10));
        }
        SelectView::ArchBar(_) => panic!("Wrong view type"),
    }

    match world.view(entity_any_b).unwrap() {
        SelectView::ArchFoo(_) => panic!("Wrong view type"),
        SelectView::ArchBar(view) => {
            assert_eq!(view.component::<CompA>(), &CompA(2));
            assert_eq!(view.component::<CompC>(), &CompC(20));
        }
    }

    let select_view = world.view(entity_any_a).unwrap();
    assert_eq!(select_view.component::<CompA>().unwrap(), &CompA(1));

    let mut select_view = world.view_mut(entity_any_a).unwrap();
    select_view.component_mut::<CompA>().unwrap().0 += 10;
    assert_eq!(select_view.component::<CompA>().unwrap(), &CompA(11));

    let select_borrow = world.borrow(entity_any_b).unwrap();
    assert_eq!(&*select_borrow.component::<CompA>().unwrap(), &CompA(2));
    select_borrow.component_mut::<CompA>().unwrap().0 += 10;
    assert_eq!(&*select_borrow.component::<CompA>().unwrap(), &CompA(12));
}
