use seq_macro::seq;

macro_rules! declare_components_n {
    (
        $name:ident,
        $n:literal
    ) => {
        seq!(I in 0..$n {
            pub trait $name<#(T~I,)*> {
                #[doc(hidden)]
                fn raw_new(#(c~I: T~I,)*) -> Self;
                #[doc(hidden)]
                fn raw_get(self) -> (#(T~I,)*);
            }
        });
    }
}

seq!(N in 1..=16 { declare_components_n!(Components~N, N); });
#[cfg(feature = "32_components")]
seq!(N in 17..=32 { declare_components_n!(Components~N, N); });
