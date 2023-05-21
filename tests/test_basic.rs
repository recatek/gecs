use gecs::prelude::*;

const TEST_CAPACITY: usize = 30;

ecs_world! {
    archetype!(
        MyArchetype1,
        200,
        ComponentA,
        ComponentB,
        #[cfg(all())] ComponentC,
    );

    #[cfg(any())]
    archetype!(
        MyArchetype2,
        TEST_CAPACITY,
        ComponentA,
        ComponentB,
        ComponentC,
    );
}

use gecs::__internal::Slices3;
use gecs::__internal::StorageFixed3;

pub struct Foo {}
impl Archetype for Foo {
    #[allow(unconditional_panic)]
    const TYPE_ID: std::num::NonZeroU8 = match std::num::NonZeroU8::new(1) {
        Some(v) => v,
        None => [][0],
    };
}

struct FooSlices<'a> {
    entities: &'a [Entity<Foo>],
    s0: &'a mut [u16],
    s1: &'a mut [u32],
    s2: &'a mut [u64],
}

impl<'a> Slices3<'a, Foo, u16, u32, u64> for FooSlices<'a> {
    fn new(
        entities: &'a [Entity<Foo>],
        s0: &'a mut [u16],
        s1: &'a mut [u32],
        s2: &'a mut [u64],
    ) -> Self {
        FooSlices {
            entities,
            s0,
            s1,
            s2,
        }
    }
}

type TestArch = StorageFixed3<Foo, u16, u32, u64, 30>;

#[test]
pub fn test_basic() {
    let mut arch: TestArch = TestArch::new();

    let entity = arch.push(0, 1, 2).unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    let entity1 = arch.push(00, 01, 02).unwrap();
    let entity2 = arch.push(10, 11, 12).unwrap();
    let entity3 = arch.push(20, 21, 22).unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    arch.remove(entity3);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    arch.remove(entity2);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }

    arch.remove(entity1);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices::<FooSlices<'_>>();
    for idx in 0..slices.s0.len() {
        println!("{} {} {}", slices.s0[idx], slices.s1[idx], slices.s2[idx]);
    }
}
