use std::marker::PhantomData;

use super::{CommandQueue, Commands, Query, QueryData, QueryFilter, Res, ResMut, Resource, World};

pub trait SystemParam: Sized {
    fn get(world: *const World, commands: *const CommandQueue) -> Self;
}

impl<Q, F> SystemParam for Query<Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    fn get(world: *const World, _commands: *const CommandQueue) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    fn get(world: *const World, _commands: *const CommandQueue) -> Self {
        let world = unsafe { &*world };
        let res = world.resource::<T>();

        unsafe { std::mem::transmute::<Res<'_, T>, Res<'a, T>>(res) }
    }
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    fn get(world: *const World, _commands: *const CommandQueue) -> Self {
        let world = unsafe { &*world };
        let res = world.resource_mut::<T>();

        unsafe { std::mem::transmute::<ResMut<'_, T>, ResMut<'a, T>>(res) }
    }
}

impl SystemParam for Commands {
    fn get(_world: *const World, commands: *const CommandQueue) -> Self {
        Self { queue: commands }
    }
}

pub trait System {
    fn run(&mut self, world: *const World, commands: *const CommandQueue);
}

pub trait IntoSystem<Params> {
    type System: System;

    fn into_system(self) -> Self::System;
}

pub struct FunctionSystem<F, Params> {
    f: F,
    _marker: PhantomData<fn() -> Params>,
}

impl<F> System for FunctionSystem<F, ()>
where
    F: FnMut(),
{
    fn run(&mut self, _world: *const World, _commands: *const CommandQueue) {
        (self.f)();
    }
}

impl<F> IntoSystem<()> for F
where
    F: FnMut(),
{
    type System = FunctionSystem<F, ()>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            _marker: PhantomData,
        }
    }
}

macro_rules! impl_system {
    ($($param:ident),+) => {
        impl<F, $($param),+> System for FunctionSystem<F, ($($param,)+)>
        where
            F: FnMut($($param),+),
            $($param: SystemParam),+
        {
            fn run(&mut self, world: *const World, commands: *const CommandQueue) {
                (self.f)(
                    $(<$param as SystemParam>::get(world, commands)),+
                );
            }
        }

        impl<F, $($param),+> IntoSystem<($($param,)+)> for F
        where
            F: FnMut($($param),+),
            $($param: SystemParam),+
        {
            type System = FunctionSystem<F, ($($param,)+)>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    _marker: PhantomData,
                }
            }
        }
    };
}

impl_system!(A);
impl_system!(A, B);
impl_system!(A, B, C);
