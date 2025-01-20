use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_slices_n {
    ($slices:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub trait $slices<'a, A: Archetype, #(T~I,)*> {
                fn new(entities: &'a [Entity<A>], #(s~I: &'a mut [T~I],)*) -> Self;
            }
        });
    };
}

seq!(N in 1..=16 { declare_slices_n!(Slices~N, N); });
#[cfg(feature = "32_components")]
seq!(N in 17..=32 { declare_slices_n!(Slices~N, N); });
