use gecs::prelude::*;

const TEST_CAPACITY: usize = 30;

pub struct ComponentA(pub u16);
pub struct ComponentB(pub u32);
pub struct ComponentC(pub u64);

ecs_world! {
    archetype!(
        ArchFoo,
        TEST_CAPACITY,
        ComponentA,
        ComponentB,
        #[cfg(all())]
        ComponentC,
        #[cfg(any())]
        DoesNotExist,
    );

    archetype!(
        ArchBar,
        30,
        ComponentA,
        ComponentB,
        ComponentC,
    );

    archetype!(
        ArchBaz,
        30,
        ComponentA,
        ComponentB,
        ComponentC,
    );
}

pub fn test(world: &mut World) {
    let mut sum: u64 = 0;

    ecs_iter_borrow!(world, |_: &EntityAny, _: &mut ComponentA| {
        sum += 1;
    });
}

pub fn test_basic() {
    let mut world = World::default();
    let arch = world.get_mut_archetype::<ArchFoo>();

    let entity = arch
        .push(ComponentA(0), ComponentB(1), ComponentC(2))
        .unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    arch.remove(entity);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    let entity1 = arch
        .push(ComponentA(00), ComponentB(01), ComponentC(02))
        .unwrap();
    let entity2 = arch
        .push(ComponentA(10), ComponentB(11), ComponentC(12))
        .unwrap();
    let entity3 = arch
        .push(ComponentA(20), ComponentB(21), ComponentC(22))
        .unwrap();

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    arch.remove(entity3);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    arch.remove(entity2);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }

    arch.remove(entity1);

    println!("{}", arch.len());
    let slices = arch.get_mut_slices();
    for idx in 0..slices.entities.len() {
        println!(
            "{} {} {}",
            slices.component_a[idx].0, slices.component_b[idx].0, slices.component_c[idx].0
        );
    }
}
