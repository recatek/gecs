use gecs::prelude::*;

pub struct CompA(pub u32);
pub struct CompB(pub u32);

ecs_world! {
    ecs_archetype!(
        ArchFoo,
        CompA,
        CompB,
    );
}

#[test]
pub fn test_generic_view() {
    let mut world = EcsWorld::new();

    let entity = world.create::<ArchFoo>((CompA(1), CompB(1)));

    let mut view = world.view(entity).unwrap();
    view_increment(&mut view);

    assert_eq!(view.component::<CompA>().0, 2);
    assert_eq!(view.component::<CompB>().0, 2);
}

#[test]
pub fn test_generic_view_get() {
    let mut world = EcsWorld::new();

    let entity = world.create::<ArchFoo>((CompA(1), CompB(1)));

    view_get_increment(&mut world, entity);

    let view = world.view(entity).unwrap();
    assert_eq!(view.component::<CompA>().0, 2);
    assert_eq!(view.component::<CompB>().0, 2);
}

fn view_increment<'a, V: View<'a>>(view: &mut V)
where
    V::Archetype: ArchetypeHas<CompA> + ArchetypeHas<CompB>,
{
    view.component_mut::<CompA>().0 += 1;
    view.component_mut::<CompB>().0 += 1;
}

fn view_get_increment<W, A>(world: &mut W, entity: Entity<A>)
where
    W: WorldHas<A>,
    A: ArchetypeHas<CompA> + ArchetypeHas<CompB>,
{
    let mut view = world.view(entity).unwrap();
    view.component_mut::<CompA>().0 += 1;
    view.component_mut::<CompB>().0 += 1;
}
