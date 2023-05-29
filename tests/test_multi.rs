use gecs::prelude::*;

const TEST_CAPACITY: usize = 5;

#[derive(Debug, PartialEq)]
pub struct ComponentA(pub u32);
#[derive(Debug, PartialEq)]
pub struct ComponentB(pub u32);
#[derive(Debug, PartialEq)]
pub struct ComponentC(pub u32);

ecs_world! {
    archetype!(
        ArchFoo,
        TEST_CAPACITY,
        ComponentA,
        ComponentB,
    );

    archetype!(
        ArchBar,
        5, // Test both inputs
        ComponentA,
        ComponentC,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_multi_push_direct() {
    let mut world = World::default();

    world.archetype_mut::<ArchFoo>().push((ComponentA(0), ComponentB(10))).unwrap();
    world.archetype_mut::<ArchFoo>().push((ComponentA(1), ComponentB(11))).unwrap();
    world.archetype_mut::<ArchFoo>().push((ComponentA(2), ComponentB(12))).unwrap();
    world.archetype_mut::<ArchFoo>().push((ComponentA(3), ComponentB(13))).unwrap();
    world.archetype_mut::<ArchFoo>().push((ComponentA(4), ComponentB(14))).unwrap();

    world.archetype_mut::<ArchBar>().push((ComponentA(5), ComponentC(15))).unwrap();
    world.archetype_mut::<ArchBar>().push((ComponentA(6), ComponentC(16))).unwrap();
    world.archetype_mut::<ArchBar>().push((ComponentA(7), ComponentC(17))).unwrap();
    world.archetype_mut::<ArchBar>().push((ComponentA(8), ComponentC(18))).unwrap();
    world.archetype_mut::<ArchBar>().push((ComponentA(9), ComponentC(19))).unwrap();

    assert_eq!(world.len::<ArchFoo>(), 5);
    assert_eq!(world.len::<ArchBar>(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_push_indirect() {
    let mut world = World::default();

    world.push::<ArchFoo>((ComponentA(0), ComponentB(10))).unwrap();
    world.push::<ArchFoo>((ComponentA(1), ComponentB(11))).unwrap();
    world.push::<ArchFoo>((ComponentA(2), ComponentB(12))).unwrap();
    world.push::<ArchFoo>((ComponentA(3), ComponentB(13))).unwrap();
    world.push::<ArchFoo>((ComponentA(4), ComponentB(14))).unwrap();

    world.push::<ArchBar>((ComponentA(5), ComponentC(15))).unwrap();
    world.push::<ArchBar>((ComponentA(6), ComponentC(16))).unwrap();
    world.push::<ArchBar>((ComponentA(7), ComponentC(17))).unwrap();
    world.push::<ArchBar>((ComponentA(8), ComponentC(18))).unwrap();
    world.push::<ArchBar>((ComponentA(9), ComponentC(19))).unwrap();

    assert_eq!(world.len::<ArchFoo>(), 5);
    assert_eq!(world.len::<ArchBar>(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_find() {
    let mut world = World::default();

    let entity_0 = world.push::<ArchFoo>((ComponentA(0), ComponentB(10))).unwrap();
    let entity_1 = world.push::<ArchFoo>((ComponentA(1), ComponentB(11))).unwrap();
    let entity_2 = world.push::<ArchFoo>((ComponentA(2), ComponentB(12))).unwrap();
    let entity_3 = world.push::<ArchFoo>((ComponentA(3), ComponentB(13))).unwrap();
    let entity_4 = world.push::<ArchFoo>((ComponentA(4), ComponentB(14))).unwrap();

    let entity_5 = world.push::<ArchBar>((ComponentA(5), ComponentC(15))).unwrap();
    let entity_6 = world.push::<ArchBar>((ComponentA(6), ComponentC(16))).unwrap();
    let entity_7 = world.push::<ArchBar>((ComponentA(7), ComponentC(17))).unwrap();
    let entity_8 = world.push::<ArchBar>((ComponentA(8), ComponentC(18))).unwrap();
    let entity_9 = world.push::<ArchBar>((ComponentA(9), ComponentC(19))).unwrap();

    assert!(ecs_find!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_7, |v: &ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find!(world, entity_0, |v: &ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find!(world, entity_2, |v: &ComponentB| assert_eq!(v.0, 12)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find!(world, entity_7, |v: &ComponentC| assert_eq!(v.0, 17)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &ComponentB| assert_eq!(v.0, 12)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &ComponentC| assert_eq!(v.0, 17)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find!(world, entity_0, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find!(world, entity_2, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (2, 12))));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find!(world, entity_7, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (7, 17))));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find_borrow!(world, entity_2, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (2, 12))));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find_borrow!(world, entity_7, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (7, 17))));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_7, |v: &ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    // As above, but mutable component access:
    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &mut ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_7, |v: &mut ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find!(world, entity_2, |v: &mut ComponentB| assert_eq!(v.0, 12)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find!(world, entity_7, |v: &mut ComponentC| assert_eq!(v.0, 17)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut ComponentB| assert_eq!(v.0, 12)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut ComponentC| assert_eq!(v.0, 17)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find!(world, entity_2, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (2, 12))));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find!(world, entity_7, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (7, 17))));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (2, 12))));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (7, 17))));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &mut ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_7, |v: &mut ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut ComponentA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_7, |v: &mut ComponentA| assert_eq!(v.0, 7)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(world.remove(entity_2).is_some());
    assert!(world.remove(entity_7).is_some());

    assert!(ecs_find!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find!(world, entity_0, |v: &ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find!(world, entity_0, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &ComponentA| assert_eq!(v.0, 9)));

    // As above, but mutable component access:
    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentB| assert_eq!(v.0, 10)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentB| assert_eq!(v.0, 11)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentB| assert_eq!(v.0, 13)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentB| assert_eq!(v.0, 14)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentC| assert_eq!(v.0, 15)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentC| assert_eq!(v.0, 16)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentC| assert_eq!(v.0, 18)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentC| assert_eq!(v.0, 19)));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (0, 10))));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (1, 11))));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (3, 13))));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA, u: &mut ComponentB| assert_eq!((v.0, u.0), (4, 14))));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (5, 15))));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (6, 16))));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (8, 18))));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA, u: &mut ComponentC| assert_eq!((v.0, u.0), (9, 19))));

    assert!(ecs_find!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut ComponentA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut ComponentA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut ComponentA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut ComponentA| assert_eq!(v.0, 4)));
    assert!(ecs_find_borrow!(world, entity_5, |v: &mut ComponentA| assert_eq!(v.0, 5)));
    assert!(ecs_find_borrow!(world, entity_6, |v: &mut ComponentA| assert_eq!(v.0, 6)));
    assert!(ecs_find_borrow!(world, entity_8, |v: &mut ComponentA| assert_eq!(v.0, 8)));
    assert!(ecs_find_borrow!(world, entity_9, |v: &mut ComponentA| assert_eq!(v.0, 9)));

    assert_eq!(ecs_find!(world, entity_2, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_2, |_: &ComponentB| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &ComponentC| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &ComponentB| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &ComponentC| panic!()), false);

    assert_eq!(ecs_find!(world, entity_2, |_: &ComponentA, _: &ComponentB| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &ComponentA, _: &ComponentC| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &ComponentA, _: &ComponentB| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &ComponentA, _: &ComponentC| panic!()), false);

    assert_eq!(ecs_find!(world, entity_2, |_: &mut ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &mut ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_2, |_: &mut ComponentB| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &mut ComponentC| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &mut ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &mut ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &mut ComponentB| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &mut ComponentC| panic!()), false);

    assert_eq!(ecs_find!(world, entity_2, |_: &mut ComponentA, _: &mut ComponentB| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &mut ComponentA, _: &mut ComponentC| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &mut ComponentA, _: &mut ComponentB| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &mut ComponentA, _: &mut ComponentC| panic!()), false);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_iter() {
    let mut world = World::default();

    let _entity_0 = world.push::<ArchFoo>((ComponentA(0), ComponentB(10))).unwrap();
    let _entity_1 = world.push::<ArchFoo>((ComponentA(1), ComponentB(11))).unwrap();
    let _entity_2 = world.push::<ArchFoo>((ComponentA(2), ComponentB(12))).unwrap();
    let _entity_3 = world.push::<ArchFoo>((ComponentA(3), ComponentB(13))).unwrap();
    let _entity_4 = world.push::<ArchFoo>((ComponentA(4), ComponentB(14))).unwrap();

    let _entity_5 = world.push::<ArchBar>((ComponentA(5), ComponentC(15))).unwrap();
    let _entity_6 = world.push::<ArchBar>((ComponentA(6), ComponentC(16))).unwrap();
    let _entity_7 = world.push::<ArchBar>((ComponentA(7), ComponentC(17))).unwrap();
    let _entity_8 = world.push::<ArchBar>((ComponentA(8), ComponentC(18))).unwrap();
    let _entity_9 = world.push::<ArchBar>((ComponentA(9), ComponentC(19))).unwrap();

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4+5+6+7+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentB| sum += v.0);
    assert_eq!(sum, 10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentC| sum += v.0);
    assert_eq!(sum, 15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+2+3+4+10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+7+8+9+15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4+5+6+7+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentB| sum += v.0);
    assert_eq!(sum, 10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentC| sum += v.0);
    assert_eq!(sum, 15+16+17+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+2+3+4+10+11+12+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+7+8+9+15+16+17+18+19);

    assert!(world.remove(_entity_2).is_some());
    assert!(world.remove(_entity_7).is_some());

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA| sum += v.0);
    assert_eq!(sum, 0+1+3+4+5+6+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentB| sum += v.0);
    assert_eq!(sum, 10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentC| sum += v.0);
    assert_eq!(sum, 15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+3+4+10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+8+9+15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA| sum += v.0);
    assert_eq!(sum, 0+1+3+4+5+6+8+9);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentB| sum += v.0);
    assert_eq!(sum, 10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentC| sum += v.0);
    assert_eq!(sum, 15+16+18+19);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 0+1+3+4+10+11+13+14);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 5+6+8+9+15+16+18+19);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_iter_write() {
    let mut world = World::default();

    let _entity_0 = world.push::<ArchFoo>((ComponentA(0), ComponentB(10))).unwrap();
    let _entity_1 = world.push::<ArchFoo>((ComponentA(1), ComponentB(11))).unwrap();
    let _entity_2 = world.push::<ArchFoo>((ComponentA(2), ComponentB(12))).unwrap();
    let _entity_3 = world.push::<ArchFoo>((ComponentA(3), ComponentB(13))).unwrap();
    let _entity_4 = world.push::<ArchFoo>((ComponentA(4), ComponentB(14))).unwrap();

    let _entity_5 = world.push::<ArchBar>((ComponentA(5), ComponentC(15))).unwrap();
    let _entity_6 = world.push::<ArchBar>((ComponentA(6), ComponentC(16))).unwrap();
    let _entity_7 = world.push::<ArchBar>((ComponentA(7), ComponentC(17))).unwrap();
    let _entity_8 = world.push::<ArchBar>((ComponentA(8), ComponentC(18))).unwrap();
    let _entity_9 = world.push::<ArchBar>((ComponentA(9), ComponentC(19))).unwrap();

    ecs_iter!(world, |v: &mut ComponentA| v.0 += 100);
    ecs_iter!(world, |v: &mut ComponentB| v.0 += 100);
    ecs_iter!(world, |v: &mut ComponentC| v.0 += 100);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104+105+106+107+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentB| sum += v.0);
    assert_eq!(sum, 110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentC| sum += v.0);
    assert_eq!(sum, 115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+102+103+104+110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+107+108+109+115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104+105+106+107+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentB| sum += v.0);
    assert_eq!(sum, 110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentC| sum += v.0);
    assert_eq!(sum, 115+116+117+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+102+103+104+110+111+112+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+107+108+109+115+116+117+118+119);

    assert!(world.remove(_entity_2).is_some());
    assert!(world.remove(_entity_7).is_some());

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA| sum += v.0);
    assert_eq!(sum, 100+101+103+104+105+106+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentB| sum += v.0);
    assert_eq!(sum, 110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentC| sum += v.0);
    assert_eq!(sum, 115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+103+104+110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &ComponentA, u: &ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+108+109+115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA| sum += v.0);
    assert_eq!(sum, 100+101+103+104+105+106+108+109);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentB| sum += v.0);
    assert_eq!(sum, 110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentC| sum += v.0);
    assert_eq!(sum, 115+116+118+119);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentB| sum += v.0 + u.0);
    assert_eq!(sum, 100+101+103+104+110+111+113+114);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut ComponentA, u: &mut ComponentC| sum += v.0 + u.0);
    assert_eq!(sum, 105+106+108+109+115+116+118+119);
}

#[test]
#[rustfmt::skip]
pub fn test_multi_replace() {
    let mut world = World::default();

    let entity_0 = world.push::<ArchFoo>((ComponentA(0), ComponentB(10))).unwrap();
    let entity_1 = world.push::<ArchFoo>((ComponentA(1), ComponentB(11))).unwrap();
    let entity_2 = world.push::<ArchFoo>((ComponentA(2), ComponentB(12))).unwrap();
    let entity_3 = world.push::<ArchFoo>((ComponentA(3), ComponentB(13))).unwrap();
    let entity_4 = world.push::<ArchFoo>((ComponentA(4), ComponentB(14))).unwrap();

    let entity_5 = world.push::<ArchBar>((ComponentA(5), ComponentC(15))).unwrap();
    let entity_6 = world.push::<ArchBar>((ComponentA(6), ComponentC(16))).unwrap();
    let entity_7 = world.push::<ArchBar>((ComponentA(7), ComponentC(17))).unwrap();
    let entity_8 = world.push::<ArchBar>((ComponentA(8), ComponentC(18))).unwrap();
    let entity_9 = world.push::<ArchBar>((ComponentA(9), ComponentC(19))).unwrap();

    assert_eq!(world.len::<ArchFoo>(), 5);
    assert_eq!(world.len::<ArchBar>(), 5);

    assert_eq!(world.remove(entity_4).unwrap(), (ComponentA(4), ComponentB(14)));
    assert_eq!(world.len::<ArchFoo>(), 4);

    assert_eq!(world.remove(entity_1).unwrap(), (ComponentA(1), ComponentB(11)));
    assert_eq!(world.len::<ArchFoo>(), 3);

    assert_eq!(world.remove(entity_2).unwrap(), (ComponentA(2), ComponentB(12)));
    assert_eq!(world.len::<ArchFoo>(), 2);

    assert_eq!(world.remove(entity_3).unwrap(), (ComponentA(3), ComponentB(13)));
    assert_eq!(world.len::<ArchFoo>(), 1);

    assert_eq!(world.remove(entity_0).unwrap(), (ComponentA(0), ComponentB(10)));
    assert_eq!(world.len::<ArchFoo>(), 0);

    assert_eq!(world.remove(entity_9).unwrap(), (ComponentA(9), ComponentC(19)));
    assert_eq!(world.len::<ArchBar>(), 4);

    assert_eq!(world.remove(entity_6).unwrap(), (ComponentA(6), ComponentC(16)));
    assert_eq!(world.len::<ArchBar>(), 3);

    assert_eq!(world.remove(entity_7).unwrap(), (ComponentA(7), ComponentC(17)));
    assert_eq!(world.len::<ArchBar>(), 2);

    assert_eq!(world.remove(entity_8).unwrap(), (ComponentA(8), ComponentC(18)));
    assert_eq!(world.len::<ArchBar>(), 1);

    assert_eq!(world.remove(entity_5).unwrap(), (ComponentA(5), ComponentC(15)));
    assert_eq!(world.len::<ArchBar>(), 0);

    assert_eq!(ecs_find!(world, entity_0, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_1, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_2, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_3, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_4, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_5, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_6, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_7, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_8, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_9, |_: &ComponentA| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_0, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_1, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_3, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_4, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_5, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_6, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_7, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_8, |_: &ComponentA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_9, |_: &ComponentA| panic!()), false);

    assert!(world.remove(entity_0).is_none());
    assert!(world.remove(entity_1).is_none());
    assert!(world.remove(entity_2).is_none());
    assert!(world.remove(entity_3).is_none());
    assert!(world.remove(entity_4).is_none());
    assert!(world.remove(entity_5).is_none());
    assert!(world.remove(entity_6).is_none());
    assert!(world.remove(entity_7).is_none());
    assert!(world.remove(entity_8).is_none());
    assert!(world.remove(entity_9).is_none());

    let entity_0b = world.push::<ArchFoo>((ComponentA(1000), ComponentB(1010))).unwrap();
    let entity_1b = world.push::<ArchFoo>((ComponentA(1001), ComponentB(1011))).unwrap();
    let entity_2b = world.push::<ArchFoo>((ComponentA(1002), ComponentB(1012))).unwrap();
    let entity_3b = world.push::<ArchFoo>((ComponentA(1003), ComponentB(1013))).unwrap();
    let entity_4b = world.push::<ArchFoo>((ComponentA(1004), ComponentB(1014))).unwrap();
    let entity_5b = world.push::<ArchBar>((ComponentA(1005), ComponentC(1015))).unwrap();
    let entity_6b = world.push::<ArchBar>((ComponentA(1006), ComponentC(1016))).unwrap();
    let entity_7b = world.push::<ArchBar>((ComponentA(1007), ComponentC(1017))).unwrap();
    let entity_8b = world.push::<ArchBar>((ComponentA(1008), ComponentC(1018))).unwrap();
    let entity_9b = world.push::<ArchBar>((ComponentA(1009), ComponentC(1019))).unwrap();

    assert!(ecs_find!(world, entity_0b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1000, 1010))));
    assert!(ecs_find!(world, entity_1b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1001, 1011))));
    assert!(ecs_find!(world, entity_2b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1002, 1012))));
    assert!(ecs_find!(world, entity_3b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1003, 1013))));
    assert!(ecs_find!(world, entity_4b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1004, 1014))));
    assert!(ecs_find!(world, entity_5b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1005, 1015))));
    assert!(ecs_find!(world, entity_6b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1006, 1016))));
    assert!(ecs_find!(world, entity_7b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1007, 1017))));
    assert!(ecs_find!(world, entity_8b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1008, 1018))));
    assert!(ecs_find!(world, entity_9b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1009, 1019))));

    assert!(ecs_find_borrow!(world, entity_0b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1000, 1010))));
    assert!(ecs_find_borrow!(world, entity_1b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1001, 1011))));
    assert!(ecs_find_borrow!(world, entity_2b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1002, 1012))));
    assert!(ecs_find_borrow!(world, entity_3b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1003, 1013))));
    assert!(ecs_find_borrow!(world, entity_4b, |v: &ComponentA, u: &ComponentB| assert_eq!((v.0, u.0), (1004, 1014))));
    assert!(ecs_find_borrow!(world, entity_5b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1005, 1015))));
    assert!(ecs_find_borrow!(world, entity_6b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1006, 1016))));
    assert!(ecs_find_borrow!(world, entity_7b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1007, 1017))));
    assert!(ecs_find_borrow!(world, entity_8b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1008, 1018))));
    assert!(ecs_find_borrow!(world, entity_9b, |v: &ComponentA, u: &ComponentC| assert_eq!((v.0, u.0), (1009, 1019))));

}
