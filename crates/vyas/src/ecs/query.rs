use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
};

use super::{Component, Entity, World};

pub struct Query<Q, F = ()> {
    pub(crate) world: *const World,
    pub(crate) _marker: PhantomData<(Q, F)>,
}

impl<Q, F> Query<Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    pub fn iter(&self) -> QueryIter<'_, Q, F> {
        QueryIter {
            query: self,
            index: 0,
        }
    }
}

impl<'q, Q, F> IntoIterator for &'q Query<Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    type Item = Q::Item<'q>;
    type IntoIter = QueryIter<'q, Q, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct QueryIter<'q, Q, F> {
    query: &'q Query<Q, F>,
    index: usize,
}

impl<'q, Q, F> Iterator for QueryIter<'q, Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    type Item = Q::Item<'q>;

    fn next(&mut self) -> Option<Self::Item> {
        let world: &'q World = unsafe { &*self.query.world };

        while self.index < world.alive.len() {
            let entity = world.alive[self.index];
            self.index += 1;

            if F::matches(world, entity) && Q::matches(world, entity) {
                return Some(Q::fetch(world, entity));
            }
        }

        None
    }
}

pub trait QueryData {
    type Item<'a>;

    fn matches(world: &World, entity: Entity) -> bool;
    fn fetch<'a>(world: &'a World, entity: Entity) -> Self::Item<'a>;
}

impl<T: Component> QueryData for &T {
    type Item<'a> = Ref<'a, T>;

    fn matches(world: &World, entity: Entity) -> bool {
        world.has::<T>(entity)
    }

    fn fetch<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        world.get::<T>(entity).unwrap()
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'a> = RefMut<'a, T>;

    fn matches(world: &World, entity: Entity) -> bool {
        world.has::<T>(entity)
    }

    fn fetch<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        world.get_mut::<T>(entity).unwrap()
    }
}

impl<A, B> QueryData for (&A, &B)
where
    A: Component,
    B: Component,
{
    type Item<'a> = (Ref<'a, A>, Ref<'a, B>);

    fn matches(world: &World, entity: Entity) -> bool {
        world.has::<A>(entity) && world.has::<B>(entity)
    }

    fn fetch<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        (
            world.get::<A>(entity).unwrap(),
            world.get::<B>(entity).unwrap(),
        )
    }
}

impl<A, B> QueryData for (&mut A, &B)
where
    A: Component,
    B: Component,
{
    type Item<'a> = (RefMut<'a, A>, Ref<'a, B>);

    fn matches(world: &World, entity: Entity) -> bool {
        world.has::<A>(entity) && world.has::<B>(entity)
    }

    fn fetch<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        (
            world.get_mut::<A>(entity).unwrap(),
            world.get::<B>(entity).unwrap(),
        )
    }
}

pub trait QueryFilter {
    fn matches(world: &World, entity: Entity) -> bool;
}

impl QueryFilter for () {
    fn matches(_world: &World, _entity: Entity) -> bool {
        true
    }
}

pub struct With<T>(PhantomData<T>);
pub struct Without<T>(PhantomData<T>);

impl<T: Component> QueryFilter for With<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        world.has::<T>(entity)
    }
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        !world.has::<T>(entity)
    }
}

impl<A, B> QueryFilter for (A, B)
where
    A: QueryFilter,
    B: QueryFilter,
{
    fn matches(world: &World, entity: Entity) -> bool {
        A::matches(world, entity) && B::matches(world, entity)
    }
}
