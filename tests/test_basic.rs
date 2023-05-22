use gecs::prelude::*;

const TEST_CAPACITY: usize = 30;

pub struct ComponentA();
pub struct ComponentB();
pub struct ComponentC();

ecs_world! {
    archetype!(
        ArchFoo,
        TEST_CAPACITY,
        u16,
        u32,
        #[cfg(all())]
        u64,
        #[cfg(any())]
        DoesNotExist,
    );
}

pub fn test_basic() {
    let mut world = World::default();
    let arch = world.get_mut_archetype::<ArchFoo>();

    let entity = arch.push(0, 1, 2).unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    let entity1 = arch.push(00, 01, 02).unwrap();
    let entity2 = arch.push(10, 11, 12).unwrap();
    let entity3 = arch.push(20, 21, 22).unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    arch.remove(entity3);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    arch.remove(entity2);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }

    arch.remove(entity1);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.u_16[idx], slices.u_32[idx], slices.u_64[idx]
        );
    }
}
