use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_slices_n {
    ($slices:ident, $slices_mut:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub trait $slices<'a, A: Archetype, #(T~I,)*> {
                fn new(entities: &'a [Entity<A>], #(s~I: &'a [T~I],)*) -> Self;
            }

            pub trait $slices_mut<'a, A: Archetype, #(T~I,)*> {
                fn new(entities: &'a [Entity<A>], #(s~I: &'a mut [T~I],)*) -> Self;
            }
        });
    };
}

// Declare slices for up to 16 components.
seq!(N in 1..=16 {
    declare_slices_n!(Slices~N, SlicesMut~N, N);
});

// Declare additional slices for up to 32 components.
#[cfg(feature = "32_components")]
seq!(N in 17..=32 {
    declare_slices_n!(Slices~N, SlicesMut~N, N);
});
