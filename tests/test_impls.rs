use gecs::prelude::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct CompA(pub u32);
#[derive(Clone, Default, Debug, PartialEq)]
pub struct CompB(pub u32);
#[derive(Clone, Default, Debug, PartialEq)]
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
pub fn test_default() {
    let mut world = EcsWorld::default();

    let entity_0 = world.create::<ArchFoo>(ArchFooComponents::default());
    let entity_1 = world.create::<ArchBar>(ArchBarComponents {
        comp_a: CompA(1),
        ..Default::default()
    });

    let components_0 = world.destroy(entity_0).unwrap();
    let components_1 = world.destroy(entity_1).unwrap();

    assert_eq!(components_0.comp_a.0, 0);
    assert_eq!(components_1.comp_a.0, 1);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_find_clone() {
    let mut old_world = EcsWorld::default();

    let entity_0 = old_world.create::<ArchFoo>((CompA(0), CompB(10)));
    let entity_1 = old_world.create::<ArchFoo>((CompA(1), CompB(11)));
    let entity_2 = old_world.create::<ArchFoo>((CompA(2), CompB(12)));
    let entity_3 = old_world.create::<ArchFoo>((CompA(3), CompB(13)));
    let entity_4 = old_world.create::<ArchFoo>((CompA(4), CompB(14)));

    let entity_5 = old_world.create::<ArchBar>((CompA(5), CompC(15)));
    let entity_6 = old_world.create::<ArchBar>((CompA(6), CompC(16)));
    let entity_7 = old_world.create::<ArchBar>((CompA(7), CompC(17)));
    let entity_8 = old_world.create::<ArchBar>((CompA(8), CompC(18)));
    let entity_9 = old_world.create::<ArchBar>((CompA(9), CompC(19)));

    let mut world = old_world.clone();

    old_world.destroy(entity_1);
    old_world.destroy(entity_3);
    old_world.destroy(entity_5);
    old_world.destroy(entity_7);
    old_world.destroy(entity_9);

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &CompB| assert_eq!(v.0, 12)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &CompC| assert_eq!(v.0, 17)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &CompB| assert_eq!(v.0, 12)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &CompC| assert_eq!(v.0, 17)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find!(world, entity_2, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (2, 12))).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find!(world, entity_7, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (7, 17))).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (2, 12))).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (7, 17))).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    // As above, but mutable component access:
    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &mut CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &mut CompB| assert_eq!(v.0, 12)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &mut CompC| assert_eq!(v.0, 17)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut CompB| assert_eq!(v.0, 12)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut CompC| assert_eq!(v.0, 17)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find!(world, entity_2, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (2, 12))).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find!(world, entity_7, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (7, 17))).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (2, 12))).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (7, 17))).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_7, |v: &mut CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut CompA| assert_eq!(v.0, 7)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(world.destroy(entity_2).is_some());
    assert!(world.destroy(entity_7).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &CompA| assert_eq!(v.0, 9)).is_some());

    // As above, but mutable component access:
    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompB| assert_eq!(v.0, 10)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompB| assert_eq!(v.0, 11)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompB| assert_eq!(v.0, 13)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompB| assert_eq!(v.0, 14)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompC| assert_eq!(v.0, 15)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompC| assert_eq!(v.0, 16)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompC| assert_eq!(v.0, 18)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompC| assert_eq!(v.0, 19)).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (0, 10))).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (1, 11))).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (3, 13))).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA, u: &mut CompB| assert_eq!((v.0, u.0), (4, 14))).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (5, 15))).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (6, 16))).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (8, 18))).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA, u: &mut CompC| assert_eq!((v.0, u.0), (9, 19))).is_some());

    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)).is_some());
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)).is_some());
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)).is_some());
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)).is_some());
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut CompA| assert_eq!(v.0, 5)).is_some());
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut CompA| assert_eq!(v.0, 6)).is_some());
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut CompA| assert_eq!(v.0, 8)).is_some());
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut CompA| assert_eq!(v.0, 9)).is_some());

    assert!(ecs_find!(world, entity_2, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_2, |_: &CompB| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &CompC| panic!()).is_none());

    assert!(ecs_find_borrow!(world, entity_2, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_2, |_: &CompB| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &CompC| panic!()).is_none());

    assert!(ecs_find!(world, entity_2, |_: &CompA, _: &CompB| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &CompA, _: &CompC| panic!()).is_none());

    assert!(ecs_find_borrow!(world, entity_2, |_: &CompA, _: &CompB| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &CompA, _: &CompC| panic!()).is_none());

    assert!(ecs_find!(world, entity_2, |_: &mut CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &mut CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_2, |_: &mut CompB| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &mut CompC| panic!()).is_none());

    assert!(ecs_find_borrow!(world, entity_2, |_: &mut CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &mut CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_2, |_: &mut CompB| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &mut CompC| panic!()).is_none());

    assert!(ecs_find!(world, entity_2, |_: &mut CompA, _: &mut CompB| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &mut CompA, _: &mut CompC| panic!()).is_none());

    assert!(ecs_find_borrow!(world, entity_2, |_: &mut CompA, _: &mut CompB| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &mut CompA, _: &mut CompC| panic!()).is_none());
}
