use gecs::prelude::*;

#[derive(Debug, PartialEq)]
pub struct CompA(pub u32);

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
    );
}

pub trait RequiresSend: Send {}
pub trait RequiresSync: Sync {}

#[cfg(feature = "sync")]
impl RequiresSend for EcsWorld {}
#[cfg(feature = "sync")]
impl RequiresSync for EcsWorld {}

#[cfg(feature = "sync")]
#[test]
fn test_requires_send() {}
#[cfg(feature = "sync")]
#[test]
fn test_requires_sync() {}

#[cfg(feature = "sync")]
#[test]
fn test_sync() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;
    use std::sync::OnceLock;

    const VALUE: u32 = 5;
    const THREADS: u32 = 10;

    static SUM: AtomicU32 = AtomicU32::new(0);
    static WORLD: OnceLock<EcsWorld> = OnceLock::new();

    let mut world = EcsWorld::default();
    let entity = world.create::<ArchFoo>((CompA(VALUE),));
    let _ = WORLD.set(world);

    let handles = (0..THREADS)
        .map(|_| {
            thread::spawn(move || {
                let borrow = WORLD.get().unwrap().borrow(entity).unwrap();
                SUM.fetch_add(borrow.component::<CompA>().0, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(SUM.load(Ordering::SeqCst), VALUE * THREADS);
}
