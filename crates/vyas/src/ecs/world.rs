use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
};

use super::{Bundle, Query, QueryData, QueryFilter, Res, ResMut, Resource};

pub trait Component: 'static {}
impl<T: 'static> Component for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) u64);

pub struct World {
    next_entity: u64,
    pub(crate) alive: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn AnyStorage>>,
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            alive: Vec::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.alive.push(entity);
        bundle.insert_into(entity, self);
        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.alive.retain(|&e| e != entity);
        for storage in self.components.values_mut() {
            storage.remove_entity(entity);
        }
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        self.storage_mut::<T>().insert(entity, component);
    }

    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.storage::<T>()
            .map(|storage| storage.borrow().contains_key(&entity))
            .unwrap_or(false)
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<Ref<'_, T>> {
        let storage = self.storage::<T>()?;
        if !storage.borrow().contains_key(&entity) {
            return None;
        }

        Some(Ref::map(storage.borrow(), move |map| {
            map.get(&entity).expect("component existed during borrow")
        }))
    }

    pub fn get_mut<T: Component>(&self, entity: Entity) -> Option<RefMut<'_, T>> {
        let storage = self.storage::<T>()?;
        if !storage.borrow().contains_key(&entity) {
            return None;
        }

        Some(RefMut::map(storage.borrow_mut(), move |map| {
            map.get_mut(&entity)
                .expect("component existed during borrow")
        }))
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    pub fn resource<T: Resource>(&self) -> Res<'_, T> {
        let cell = self
            .resources
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("missing resource: {}", std::any::type_name::<T>()));

        let value = Ref::map(cell.borrow(), |boxed| {
            boxed
                .downcast_ref::<T>()
                .expect("resource had incorrect concrete type")
        });

        Res::new(value)
    }

    pub fn resource_mut<T: Resource>(&self) -> ResMut<'_, T> {
        let cell = self
            .resources
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("missing resource: {}", std::any::type_name::<T>()));

        let value = RefMut::map(cell.borrow_mut(), |boxed| {
            boxed
                .downcast_mut::<T>()
                .expect("resource had incorrect concrete type")
        });

        ResMut::new(value)
    }

    pub fn query<Q, F>(&self) -> Query<Q, F>
    where
        Q: QueryData,
        F: QueryFilter,
    {
        Query {
            world: self as *const World,
            _marker: PhantomData,
        }
    }

    fn storage<T: Component>(&self) -> Option<&RefCell<HashMap<Entity, T>>> {
        self.components
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<RefCell<HashMap<Entity, T>>>()
    }

    fn storage_mut<T: Component>(&mut self) -> &mut HashMap<Entity, T> {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(RefCell::new(HashMap::<Entity, T>::new())));

        self.components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_any_mut()
            .downcast_mut::<RefCell<HashMap<Entity, T>>>()
            .unwrap()
            .get_mut()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

trait AnyStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity(&mut self, entity: Entity);
}

impl<T: Component> AnyStorage for RefCell<HashMap<Entity, T>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.get_mut().remove(&entity);
    }
}
