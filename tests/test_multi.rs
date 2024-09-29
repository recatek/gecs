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
pub fn test_archetype_id() {
    assert_eq!(ArchFoo::ARCHETYPE_ID, 3);
    assert_eq!(ArchBar::ARCHETYPE_ID, 4); // Implicit
}

#[test]
#[rustfmt::skip]
pub fn test_multi_create_direct() {
    let mut world = EcsWorld::with_capacity(EcsWorldCapacity {
        arch_foo: 5,
        arch_bar: 3,
    });

    world.archetype_mut::<ArchFoo>().create((CompA(0), CompB(10)));
    world.archetype_mut::<ArchFoo>().create((CompA(1), CompB(11)));
    world.archetype_mut::<ArchFoo>().create((CompA(2), CompB(12)));
    world.archetype_mut::<ArchFoo>().create((CompA(3), CompB(13)));
    world.archetype_mut::<ArchFoo>().create((CompA(4), CompB(14)));

    world.archetype_mut::<ArchBar>().create((CompA(5), CompC(15)));
    world.archetype_mut::<ArchBar>().create((CompA(6), CompC(16)));
    world.archetype_mut::<ArchBar>().create((CompA(7), CompC(17)));
    world.archetype_mut::<ArchBar>().create((CompA(8), CompC(18)));
    world.archetype_mut::<ArchBar>().create((CompA(9), CompC(19)));

    assert_eq!(world.archetype::<ArchFoo>().len(), 5);
    assert_eq!(world.archetype::<ArchBar>().len(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_create_indirect() {
    let mut world = EcsWorld::default();

    world.create::<ArchFoo>((CompA(0), CompB(10)));
    world.create::<ArchFoo>((CompA(1), CompB(11)));
    world.create::<ArchFoo>((CompA(2), CompB(12)));
    world.create::<ArchFoo>((CompA(3), CompB(13)));
    world.create::<ArchFoo>((CompA(4), CompB(14)));

    world.create::<ArchBar>((CompA(5), CompC(15)));
    world.create::<ArchBar>((CompA(6), CompC(16)));
    world.create::<ArchBar>((CompA(7), CompC(17)));
    world.create::<ArchBar>((CompA(8), CompC(18)));
    world.create::<ArchBar>((CompA(9), CompC(19)));

    assert_eq!(world.archetype::<ArchFoo>().len(), 5);
    assert_eq!(world.archetype::<ArchBar>().len(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_find() {
    let mut world = EcsWorld::default();

    let entity_0 = world.create::<ArchFoo>((CompA(0), CompB(10)));
    let entity_1 = world.create::<ArchFoo>((CompA(1), CompB(11)));
    let entity_2 = world.create::<ArchFoo>((CompA(2), CompB(12)));
    let entity_3 = world.create::<ArchFoo>((CompA(3), CompB(13)));
    let entity_4 = world.create::<ArchFoo>((CompA(4), CompB(14)));

    let entity_5 = world.create::<ArchBar>((CompA(5), CompC(15)));
    let entity_6 = world.create::<ArchBar>((CompA(6), CompC(16)));
    let entity_7 = world.create::<ArchBar>((CompA(7), CompC(17)));
    let entity_8 = world.create::<ArchBar>((CompA(8), CompC(18)));
    let entity_9 = world.create::<ArchBar>((CompA(9), CompC(19)));

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

#[test]
#[rustfmt::skip]
pub fn test_multi_iter() {
    let mut world = EcsWorld::default();

    let _entity_0 = world.create::<ArchFoo>((CompA(0), CompB(10)));
    let _entity_1 = world.create::<ArchFoo>((CompA(1), CompB(11)));
    let _entity_2 = world.create::<ArchFoo>((CompA(2), CompB(12)));
    let _entity_3 = world.create::<ArchFoo>((CompA(3), CompB(13)));
    let _entity_4 = world.create::<ArchFoo>((CompA(4), CompB(14)));

    let _entity_5 = world.create::<ArchBar>((CompA(5), CompC(15)));
    let _entity_6 = world.create::<ArchBar>((CompA(6), CompC(16)));
    let _entity_7 = world.create::<ArchBar>((CompA(7), CompC(17)));
    let _entity_8 = world.create::<ArchBar>((CompA(8), CompC(18)));
    let _entity_9 = world.create::<ArchBar>((CompA(9), CompC(19)));

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4+5+6+7+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompB| sum += v.0);
    assert_eq!(sum, 10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompC| sum += v.0);
    assert_eq!(sum, 15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+2+3+4+10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+7+8+9+15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4+5+6+7+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompB| sum += v.0);
    assert_eq!(sum, 10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompC| sum += v.0);
    assert_eq!(sum, 15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+2+3+4+10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+7+8+9+15+16+17+18+19);

    assert!(world.destroy(_entity_2).is_some());
    assert!(world.destroy(_entity_7).is_some());

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4+5+6+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompB| sum += v.0);
    assert_eq!(sum, 10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompC| sum += v.0);
    assert_eq!(sum, 15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+3+4+10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+8+9+15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4+5+6+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompB| sum += v.0);
    assert_eq!(sum, 10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompC| sum += v.0);
    assert_eq!(sum, 15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+3+4+10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+8+9+15+16+18+19);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_iter_write() {
    let mut world = EcsWorld::default();

    let _entity_0 = world.create::<ArchFoo>((CompA(0), CompB(10)));
    let _entity_1 = world.create::<ArchFoo>((CompA(1), CompB(11)));
    let _entity_2 = world.create::<ArchFoo>((CompA(2), CompB(12)));
    let _entity_3 = world.create::<ArchFoo>((CompA(3), CompB(13)));
    let _entity_4 = world.create::<ArchFoo>((CompA(4), CompB(14)));

    let _entity_5 = world.create::<ArchBar>((CompA(5), CompC(15)));
    let _entity_6 = world.create::<ArchBar>((CompA(6), CompC(16)));
    let _entity_7 = world.create::<ArchBar>((CompA(7), CompC(17)));
    let _entity_8 = world.create::<ArchBar>((CompA(8), CompC(18)));
    let _entity_9 = world.create::<ArchBar>((CompA(9), CompC(19)));

    ecs_iter!(world, |v: &mut CompA| v.0 += 100);
    ecs_iter!(world, |v: &mut CompB| v.0 += 100);
    ecs_iter!(world, |v: &mut CompC| v.0 += 100);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104+105+106+107+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompB| sum += v.0);
    assert_eq!(sum, 110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompC| sum += v.0);
    assert_eq!(sum, 115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+102+103+104+110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+107+108+109+115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104+105+106+107+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompB| sum += v.0);
    assert_eq!(sum, 110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompC| sum += v.0);
    assert_eq!(sum, 115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+102+103+104+110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+107+108+109+115+116+117+118+119);

    assert!(world.destroy(_entity_2).is_some());
    assert!(world.destroy(_entity_7).is_some());

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104+105+106+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompB| sum += v.0);
    assert_eq!(sum, 110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompC| sum += v.0);
    assert_eq!(sum, 115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+103+104+110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA, u: &CompC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+108+109+115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104+105+106+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompB| sum += v.0);
    assert_eq!(sum, 110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompC| sum += v.0);
    assert_eq!(sum, 115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+103+104+110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA, u: &mut CompC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+108+109+115+116+118+119);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_replace() {
    let mut world = EcsWorld::default();

    let entity_0 = world.create::<ArchFoo>((CompA(0), CompB(10)));
    let entity_1 = world.create::<ArchFoo>((CompA(1), CompB(11)));
    let entity_2 = world.create::<ArchFoo>((CompA(2), CompB(12)));
    let entity_3 = world.create::<ArchFoo>((CompA(3), CompB(13)));
    let entity_4 = world.create::<ArchFoo>((CompA(4), CompB(14)));

    let entity_5 = world.create::<ArchBar>((CompA(5), CompC(15)));
    let entity_6 = world.create::<ArchBar>((CompA(6), CompC(16)));
    let entity_7 = world.create::<ArchBar>((CompA(7), CompC(17)));
    let entity_8 = world.create::<ArchBar>((CompA(8), CompC(18)));
    let entity_9 = world.create::<ArchBar>((CompA(9), CompC(19)));

    assert_eq!(world.archetype::<ArchFoo>().len(), 5);
    assert_eq!(world.archetype::<ArchBar>().len(), 5);

    assert_eq!(world.destroy(entity_4).unwrap(), (CompA(4), CompB(14)));
    assert_eq!(world.archetype::<ArchFoo>().len(), 4);

    assert_eq!(world.destroy(entity_1).unwrap(), (CompA(1), CompB(11)));
    assert_eq!(world.archetype::<ArchFoo>().len(), 3);

    assert_eq!(world.destroy(entity_2).unwrap(), (CompA(2), CompB(12)));
    assert_eq!(world.archetype::<ArchFoo>().len(), 2);

    assert_eq!(world.destroy(entity_3).unwrap(), (CompA(3), CompB(13)));
    assert_eq!(world.archetype::<ArchFoo>().len(), 1);

    assert_eq!(world.destroy(entity_0).unwrap(), (CompA(0), CompB(10)));
    assert_eq!(world.archetype::<ArchFoo>().len(), 0);

    assert_eq!(world.destroy(entity_9).unwrap(), (CompA(9), CompC(19)));
    assert_eq!(world.archetype::<ArchBar>().len(), 4);

    assert_eq!(world.destroy(entity_6).unwrap(), (CompA(6), CompC(16)));
    assert_eq!(world.archetype::<ArchBar>().len(), 3);

    assert_eq!(world.destroy(entity_7).unwrap(), (CompA(7), CompC(17)));
    assert_eq!(world.archetype::<ArchBar>().len(), 2);

    assert_eq!(world.destroy(entity_8).unwrap(), (CompA(8), CompC(18)));
    assert_eq!(world.archetype::<ArchBar>().len(), 1);

    assert_eq!(world.destroy(entity_5).unwrap(), (CompA(5), CompC(15)));
    assert_eq!(world.archetype::<ArchBar>().len(), 0);

    assert!(ecs_find!(world, entity_0, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_1, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_2, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_3, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_4, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_5, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_6, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_7, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_8, |_: &CompA| panic!()).is_none());
    assert!(ecs_find!(world, entity_9, |_: &CompA| panic!()).is_none());
    
    assert!(ecs_find_borrow!(world, entity_0, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_1, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_2, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_3, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_4, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_5, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_6, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_7, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_8, |_: &CompA| panic!()).is_none());
    assert!(ecs_find_borrow!(world, entity_9, |_: &CompA| panic!()).is_none());

    assert!(world.destroy(entity_0).is_none());
    assert!(world.destroy(entity_1).is_none());
    assert!(world.destroy(entity_2).is_none());
    assert!(world.destroy(entity_3).is_none());
    assert!(world.destroy(entity_4).is_none());
    assert!(world.destroy(entity_5).is_none());
    assert!(world.destroy(entity_6).is_none());
    assert!(world.destroy(entity_7).is_none());
    assert!(world.destroy(entity_8).is_none());
    assert!(world.destroy(entity_9).is_none());

    let entity_0b = world.create::<ArchFoo>((CompA(1000), CompB(1010)));
    let entity_1b = world.create::<ArchFoo>((CompA(1001), CompB(1011)));
    let entity_2b = world.create::<ArchFoo>((CompA(1002), CompB(1012)));
    let entity_3b = world.create::<ArchFoo>((CompA(1003), CompB(1013)));
    let entity_4b = world.create::<ArchFoo>((CompA(1004), CompB(1014)));
    let entity_5b = world.create::<ArchBar>((CompA(1005), CompC(1015)));
    let entity_6b = world.create::<ArchBar>((CompA(1006), CompC(1016)));
    let entity_7b = world.create::<ArchBar>((CompA(1007), CompC(1017)));
    let entity_8b = world.create::<ArchBar>((CompA(1008), CompC(1018)));
    let entity_9b = world.create::<ArchBar>((CompA(1009), CompC(1019)));

    assert!(ecs_find!(world, entity_0b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1000, 1010))).is_some());
    assert!(ecs_find!(world, entity_1b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1001, 1011))).is_some());
    assert!(ecs_find!(world, entity_2b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1002, 1012))).is_some());
    assert!(ecs_find!(world, entity_3b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1003, 1013))).is_some());
    assert!(ecs_find!(world, entity_4b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1004, 1014))).is_some());
    assert!(ecs_find!(world, entity_5b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1005, 1015))).is_some());
    assert!(ecs_find!(world, entity_6b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1006, 1016))).is_some());
    assert!(ecs_find!(world, entity_7b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1007, 1017))).is_some());
    assert!(ecs_find!(world, entity_8b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1008, 1018))).is_some());
    assert!(ecs_find!(world, entity_9b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1009, 1019))).is_some());

    assert!(ecs_find_borrow!(world, entity_0b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1000, 1010))).is_some());
    assert!(ecs_find_borrow!(world, entity_1b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1001, 1011))).is_some());
    assert!(ecs_find_borrow!(world, entity_2b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1002, 1012))).is_some());
    assert!(ecs_find_borrow!(world, entity_3b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1003, 1013))).is_some());
    assert!(ecs_find_borrow!(world, entity_4b, |v: &CompA, u: &CompB| assert_eq!((v.0, u.0), (1004, 1014))).is_some());
    assert!(ecs_find_borrow!(world, entity_5b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1005, 1015))).is_some());
    assert!(ecs_find_borrow!(world, entity_6b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1006, 1016))).is_some());
    assert!(ecs_find_borrow!(world, entity_7b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1007, 1017))).is_some());
    assert!(ecs_find_borrow!(world, entity_8b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1008, 1018))).is_some());
    assert!(ecs_find_borrow!(world, entity_9b, |v: &CompA, u: &CompC| assert_eq!((v.0, u.0), (1009, 1019))).is_some());

}
