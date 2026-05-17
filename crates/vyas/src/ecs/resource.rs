use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

pub trait Resource: 'static {}
impl<T: 'static> Resource for T {}

pub struct Res<'a, T: Resource> {
    value: Ref<'a, T>,
}

impl<'a, T: Resource> Res<'a, T> {
    pub(crate) fn new(value: Ref<'a, T>) -> Self {
        Self { value }
    }
}

impl<T: Resource> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct ResMut<'a, T: Resource> {
    value: RefMut<'a, T>,
}

impl<'a, T: Resource> ResMut<'a, T> {
    pub(crate) fn new(value: RefMut<'a, T>) -> Self {
        Self { value }
    }
}

impl<T: Resource> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Resource> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
