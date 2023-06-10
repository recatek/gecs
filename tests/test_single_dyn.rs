use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        dyn,
        CompA,
        CompZ,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_push() {
    let mut world = World::default();

    world.arch_foo.push((CompA(0), CompZ,));
    world.arch_foo.push((CompA(1), CompZ,));
    world.arch_foo.push((CompA(2), CompZ,));
    world.arch_foo.push((CompA(3), CompZ,));
    world.arch_foo.push((CompA(4), CompZ,));

    assert_eq!(world.arch_foo.len(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_push_with_capacity_zero() {
    let mut world = World::with_capacity(0);

    world.arch_foo.push((CompA(0), CompZ,));
    world.arch_foo.push((CompA(1), CompZ,));
    world.arch_foo.push((CompA(2), CompZ,));
    world.arch_foo.push((CompA(3), CompZ,));
    world.arch_foo.push((CompA(4), CompZ,));

    assert_eq!(world.arch_foo.len(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_push_with_capacity_all() {
    let mut world = World::with_capacity(5);

    world.arch_foo.push((CompA(0), CompZ,));
    world.arch_foo.push((CompA(1), CompZ,));
    world.arch_foo.push((CompA(2), CompZ,));
    world.arch_foo.push((CompA(3), CompZ,));
    world.arch_foo.push((CompA(4), CompZ,));

    assert_eq!(world.arch_foo.len(), 5);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_entity() {
    let mut world = World::default();

    let entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    assert!(ecs_find!(world, entity_0, |v: &Entity<ArchFoo>| assert!(*v == entity_0)));
    assert!(ecs_find!(world, entity_1, |v: &Entity<ArchFoo>| assert!(*v == entity_1)));
    assert!(ecs_find!(world, entity_2, |v: &Entity<ArchFoo>| assert!(*v == entity_2)));
    assert!(ecs_find!(world, entity_3, |v: &Entity<ArchFoo>| assert!(*v == entity_3)));
    assert!(ecs_find!(world, entity_4, |v: &Entity<ArchFoo>| assert!(*v == entity_4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &Entity<ArchFoo>| assert!(*v == entity_0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &Entity<ArchFoo>| assert!(*v == entity_1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &Entity<ArchFoo>| assert!(*v == entity_2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &Entity<ArchFoo>| assert!(*v == entity_3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &Entity<ArchFoo>| assert!(*v == entity_4)));

    assert!(world.arch_foo.remove(entity_0).is_some());
    assert!(world.arch_foo.remove(entity_1).is_some());
    assert!(world.arch_foo.remove(entity_2).is_some());
    assert!(world.arch_foo.remove(entity_3).is_some());
    assert!(world.arch_foo.remove(entity_4).is_some());

    let entity_0b = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1b = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2b = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3b = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4b = world.arch_foo.push((CompA(4), CompZ,));

    assert!(entity_0 != entity_0b);
    assert!(entity_1 != entity_1b);
    assert!(entity_2 != entity_2b);
    assert!(entity_3 != entity_3b);
    assert!(entity_4 != entity_4b);

    assert!(ecs_find!(world, entity_0b, |v: &Entity<ArchFoo>| assert!(*v == entity_0b)));
    assert!(ecs_find!(world, entity_1b, |v: &Entity<ArchFoo>| assert!(*v == entity_1b)));
    assert!(ecs_find!(world, entity_2b, |v: &Entity<ArchFoo>| assert!(*v == entity_2b)));
    assert!(ecs_find!(world, entity_3b, |v: &Entity<ArchFoo>| assert!(*v == entity_3b)));
    assert!(ecs_find!(world, entity_4b, |v: &Entity<ArchFoo>| assert!(*v == entity_4b)));

    assert!(ecs_find_borrow!(world, entity_0b, |v: &Entity<ArchFoo>| assert!(*v == entity_0b)));
    assert!(ecs_find_borrow!(world, entity_1b, |v: &Entity<ArchFoo>| assert!(*v == entity_1b)));
    assert!(ecs_find_borrow!(world, entity_2b, |v: &Entity<ArchFoo>| assert!(*v == entity_2b)));
    assert!(ecs_find_borrow!(world, entity_3b, |v: &Entity<ArchFoo>| assert!(*v == entity_3b)));
    assert!(ecs_find_borrow!(world, entity_4b, |v: &Entity<ArchFoo>| assert!(*v == entity_4b)));
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_entity_with_capacity() {
    let mut world = World::with_capacity(5);

    let entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    assert!(ecs_find!(world, entity_0, |v: &Entity<ArchFoo>| assert!(*v == entity_0)));
    assert!(ecs_find!(world, entity_1, |v: &Entity<ArchFoo>| assert!(*v == entity_1)));
    assert!(ecs_find!(world, entity_2, |v: &Entity<ArchFoo>| assert!(*v == entity_2)));
    assert!(ecs_find!(world, entity_3, |v: &Entity<ArchFoo>| assert!(*v == entity_3)));
    assert!(ecs_find!(world, entity_4, |v: &Entity<ArchFoo>| assert!(*v == entity_4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &Entity<ArchFoo>| assert!(*v == entity_0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &Entity<ArchFoo>| assert!(*v == entity_1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &Entity<ArchFoo>| assert!(*v == entity_2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &Entity<ArchFoo>| assert!(*v == entity_3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &Entity<ArchFoo>| assert!(*v == entity_4)));

    assert!(world.arch_foo.remove(entity_0).is_some());
    assert!(world.arch_foo.remove(entity_1).is_some());
    assert!(world.arch_foo.remove(entity_2).is_some());
    assert!(world.arch_foo.remove(entity_3).is_some());
    assert!(world.arch_foo.remove(entity_4).is_some());

    let entity_0b = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1b = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2b = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3b = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4b = world.arch_foo.push((CompA(4), CompZ,));

    assert!(entity_0 != entity_0b);
    assert!(entity_1 != entity_1b);
    assert!(entity_2 != entity_2b);
    assert!(entity_3 != entity_3b);
    assert!(entity_4 != entity_4b);

    assert!(ecs_find!(world, entity_0b, |v: &Entity<ArchFoo>| assert!(*v == entity_0b)));
    assert!(ecs_find!(world, entity_1b, |v: &Entity<ArchFoo>| assert!(*v == entity_1b)));
    assert!(ecs_find!(world, entity_2b, |v: &Entity<ArchFoo>| assert!(*v == entity_2b)));
    assert!(ecs_find!(world, entity_3b, |v: &Entity<ArchFoo>| assert!(*v == entity_3b)));
    assert!(ecs_find!(world, entity_4b, |v: &Entity<ArchFoo>| assert!(*v == entity_4b)));

    assert!(ecs_find_borrow!(world, entity_0b, |v: &Entity<ArchFoo>| assert!(*v == entity_0b)));
    assert!(ecs_find_borrow!(world, entity_1b, |v: &Entity<ArchFoo>| assert!(*v == entity_1b)));
    assert!(ecs_find_borrow!(world, entity_2b, |v: &Entity<ArchFoo>| assert!(*v == entity_2b)));
    assert!(ecs_find_borrow!(world, entity_3b, |v: &Entity<ArchFoo>| assert!(*v == entity_3b)));
    assert!(ecs_find_borrow!(world, entity_4b, |v: &Entity<ArchFoo>| assert!(*v == entity_4b)));
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_find() {
    let mut world = World::default();

    let entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &CompA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)));
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_2, |v: &mut CompA| assert_eq!(v.0, 2)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)));

    world.arch_foo.remove(entity_2).unwrap();

    assert!(ecs_find!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)));

    assert!(ecs_find_borrow!(world, entity_0, |v: &mut CompA| assert_eq!(v.0, 0)));
    assert!(ecs_find_borrow!(world, entity_1, |v: &mut CompA| assert_eq!(v.0, 1)));
    assert!(ecs_find_borrow!(world, entity_3, |v: &mut CompA| assert_eq!(v.0, 3)));
    assert!(ecs_find_borrow!(world, entity_4, |v: &mut CompA| assert_eq!(v.0, 4)));

    assert_eq!(ecs_find!(world, entity_2, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_2, |_: &mut CompA| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &mut CompA| panic!()), false);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_iter() {
    let mut world = World::default();

    let _entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let _entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let _entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let _entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let _entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+2+3+4);

    world.arch_foo.remove(_entity_2).unwrap();

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 0+1+3+4);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_iter_write() {
    let mut world = World::default();

    let _entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let _entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let _entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let _entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let _entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    ecs_iter!(world, |v: &mut CompA| v.0 += 100);

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+102+103+104);

    world.arch_foo.remove(_entity_2).unwrap();

    let mut sum = 0;
    ecs_iter!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104);

    let mut sum = 0;
    ecs_iter!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104);

    let mut sum = 0;
    ecs_iter_borrow!(world, |v: &mut CompA| sum += v.0);
    assert_eq!(sum, 100+101+103+104);
}

#[test]
#[rustfmt::skip]
pub fn test_single_dyn_remove_replace() {
    let mut world = World::default();

    let entity_0 = world.arch_foo.push((CompA(0), CompZ,));
    let entity_1 = world.arch_foo.push((CompA(1), CompZ,));
    let entity_2 = world.arch_foo.push((CompA(2), CompZ,));
    let entity_3 = world.arch_foo.push((CompA(3), CompZ,));
    let entity_4 = world.arch_foo.push((CompA(4), CompZ,));

    assert_eq!(world.arch_foo.len(), 5);

    assert_eq!(world.arch_foo.remove(entity_4).unwrap().0.0, 4);
    assert_eq!(world.arch_foo.len(), 4);

    assert_eq!(world.arch_foo.remove(entity_1).unwrap().0.0, 1);
    assert_eq!(world.arch_foo.len(), 3);

    assert_eq!(world.arch_foo.remove(entity_2).unwrap().0.0, 2);
    assert_eq!(world.arch_foo.len(), 2);

    assert_eq!(world.arch_foo.remove(entity_3).unwrap().0.0, 3);
    assert_eq!(world.arch_foo.len(), 1);

    assert_eq!(world.arch_foo.remove(entity_0).unwrap().0.0, 0);
    assert_eq!(world.arch_foo.len(), 0);

    assert_eq!(ecs_find!(world, entity_0, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_1, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_2, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_3, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find!(world, entity_4, |_: &CompA| panic!()), false);

    assert_eq!(ecs_find_borrow!(world, entity_0, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_1, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_2, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_3, |_: &CompA| panic!()), false);
    assert_eq!(ecs_find_borrow!(world, entity_4, |_: &CompA| panic!()), false);

    assert!(world.arch_foo.remove(entity_0).is_none());
    assert!(world.arch_foo.remove(entity_1).is_none());
    assert!(world.arch_foo.remove(entity_2).is_none());
    assert!(world.arch_foo.remove(entity_3).is_none());
    assert!(world.arch_foo.remove(entity_4).is_none());

    let entity_0b = world.arch_foo.push((CompA(1000), CompZ,));
    let entity_1b = world.arch_foo.push((CompA(1001), CompZ,));
    let entity_2b = world.arch_foo.push((CompA(1002), CompZ,));
    let entity_3b = world.arch_foo.push((CompA(1003), CompZ,));
    let entity_4b = world.arch_foo.push((CompA(1004), CompZ,));

    assert!(ecs_find!(world, entity_0b, |v: &CompA| assert_eq!(v.0, 1000)));
    assert!(ecs_find!(world, entity_1b, |v: &CompA| assert_eq!(v.0, 1001)));
    assert!(ecs_find!(world, entity_2b, |v: &CompA| assert_eq!(v.0, 1002)));
    assert!(ecs_find!(world, entity_3b, |v: &CompA| assert_eq!(v.0, 1003)));
    assert!(ecs_find!(world, entity_4b, |v: &CompA| assert_eq!(v.0, 1004)));

    assert!(ecs_find_borrow!(world, entity_0b, |v: &CompA| assert_eq!(v.0, 1000)));
    assert!(ecs_find_borrow!(world, entity_1b, |v: &CompA| assert_eq!(v.0, 1001)));
    assert!(ecs_find_borrow!(world, entity_2b, |v: &CompA| assert_eq!(v.0, 1002)));
    assert!(ecs_find_borrow!(world, entity_3b, |v: &CompA| assert_eq!(v.0, 1003)));
    assert!(ecs_find_borrow!(world, entity_4b, |v: &CompA| assert_eq!(v.0, 1004)));
}
