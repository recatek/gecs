#[cfg(not(feature = "sync"))]
pub use refcell::*;
#[cfg(not(feature = "sync"))]
mod refcell {
    pub type BorrowCell<T> = std::cell::RefCell<T>;
    pub type BorrowRef<'a, T> = std::cell::Ref<'a, T>;
    pub type BorrowRefMut<'a, T> = std::cell::RefMut<'a, T>;
}

#[cfg(feature = "sync")]
pub use atomic::*;
#[cfg(feature = "sync")]
mod atomic {
    pub type BorrowCell<T> = atomic_refcell::AtomicRefCell<T>;
    pub type BorrowRef<'a, T> = atomic_refcell::AtomicRef<'a, T>;
    pub type BorrowRefMut<'a, T> = atomic_refcell::AtomicRefMut<'a, T>;
}
