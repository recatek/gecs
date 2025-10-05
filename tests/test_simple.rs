use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompB(pub u32);
pub struct CompZ; // ZST

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        CompZ,
    );
}
