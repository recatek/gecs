use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;
use crate::archetype::view::*;

macro_rules! declare_iter_n {
    ($iter:ident, $iter_mut:ident, $view:ident, $view_mut:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub struct $iter<'a, A: Archetype, V: $view<'a, A, #(T~I,)*>, #(T~I,)*> {
                pub(crate) remaining: usize,
                pub(crate) ptr_entity: *const Entity<A>,
                #(pub(crate) ptr_d~I: *const T~I,)*
                pub(crate) phantom: std::marker::PhantomData<fn() -> (&'a A, &'a V)>,
            }

            pub struct $iter_mut<'a, A: Archetype, V: $view_mut<'a, A, #(T~I,)*>, #(T~I,)*> {
                pub(crate) remaining: usize,
                pub(crate) ptr_entity: *const Entity<A>,
                #(pub(crate) ptr_d~I: *mut T~I,)*
                pub(crate) phantom: std::marker::PhantomData<fn() -> (&'a A, &'a V)>,
            }

            impl<
                'a,
                A: Archetype + 'a,
                V: $view<'a, A, #(T~I,)*> + 'a,
                #(T~I: 'a,)*
            > Iterator for $iter<'a, A, V, #(T~I,)*> {
                type Item = V;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }

                    unsafe {
                        let entity = &*self.ptr_entity;
                        #(let arg~I = &*self.ptr_d~I;)*

                        self.ptr_entity = self.ptr_entity.offset(1);
                        #(self.ptr_d~I = self.ptr_d~I.offset(1);)*

                        self.remaining -= 1;
                        Some(V::new(entity, #(arg~I,)*))
                    }
                }
            }

            impl<
                'a,
                A: Archetype + 'a,
                V: $view_mut<'a, A, #(T~I,)*> + 'a,
                #(T~I: 'a,)*
            > Iterator for $iter_mut<'a, A, V, #(T~I,)*> {
                type Item = V;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }

                    unsafe {
                        let entity = &*self.ptr_entity;
                        #(let arg~I = &mut *self.ptr_d~I;)*

                        self.ptr_entity = self.ptr_entity.offset(1);
                        #(self.ptr_d~I = self.ptr_d~I.offset(1);)*

                        self.remaining -= 1;
                        Some(V::new(entity, #(arg~I,)*))
                    }
                }
            }
        });
    };
}

seq!(N in 1..=16 { declare_iter_n!(Iter~N, IterMut~N, View~N, ViewMut~N, N); });
#[cfg(feature = "32_components")]
seq!(N in 17..=32 { declare_iter_n!(Iter~N, IterMut~N, View~N, ViewMut~N,  N); });
