use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_view_n {
    ($view:ident, $view_mut:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub trait $view<'a, A: Archetype, #(T~I,)*> {
                fn new(entity: &'a Entity<A>, #(e~I: &'a T~I,)*) -> Self;
            }

            pub trait $view_mut<'a, A: Archetype, #(T~I,)*> {
                fn new(entity: &'a Entity<A>, #(e~I: &'a mut T~I,)*) -> Self;
            }
        });
    };
}

seq!(N in 1..=16 { declare_view_n!(View~N, ViewMut~N, N); });
#[cfg(feature = "32_components")]
seq!(N in 17..=32 { declare_view_n!(View~N, ViewMut~N, N); });
