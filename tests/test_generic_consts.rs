use gecs::prelude::*;

pub struct CompA;
pub struct CompB;
pub struct CompC;
pub struct CompG<const N: u32>();

impl<const N: u32> CompG<N> {
    const fn get(&self) -> u32 { N }
}

ecs_world! {
    ecs_archetype!(
        ArchA1,
        CompA,
        CompG<1>,
    );

    ecs_archetype!(
        ArchA2,
        CompA,
        CompG<2>,
    );

    ecs_archetype!(
        ArchB3,
        CompB,
        CompG<3>,
    );

    ecs_archetype!(
        ArchB4,
        CompB,
        CompG<4>,
    );

    ecs_archetype!(
        ArchB56,
        CompC,
        CompG<5>,
        CompG<6>,
    );
}

#[test]
#[rustfmt::skip]
pub fn test_generic_components() {
    let mut world = EcsWorld::default();

    world.create::<ArchA1>((CompA, CompG::<1>()));
    world.create::<ArchA2>((CompA, CompG::<2>()));
    world.create::<ArchB3>((CompB, CompG::<3>()));
    world.create::<ArchB4>((CompB, CompG::<4>()));

    let mut sum = 0_usize;

    ecs_iter!(world, |_: &CompA, g: &CompG<_>| {
        sum += g.get() as usize
    });

    assert_eq!(sum, 1 + 2);

    let mut sum = 0_usize;

    ecs_iter!(world, |_: &CompB, g: &CompG<_>| {
        sum += g.get() as usize
    });

    assert_eq!(sum, 3 + 4);

    let mut sum = 0_usize;

    ecs_iter!(world, |_: &OneOf<CompA, CompB>, g: &CompG<_>| {
        sum += g.get() as usize
    });

    assert_eq!(sum, 1 + 2 + 3 + 4);
}
