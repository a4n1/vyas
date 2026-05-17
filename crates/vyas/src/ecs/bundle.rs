use super::{Component, Entity, World};

pub trait Bundle: 'static {
    fn insert_into(self, entity: Entity, world: &mut World);
}

macro_rules! impl_bundle_tuple {
    ($($name:ident),+) => {
        impl<$($name: Component),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn insert_into(self, entity: Entity, world: &mut World) {
                let ($($name,)+) = self;
                $(world.insert(entity, $name);)+
            }
        }
    };
}

impl_bundle_tuple!(A);
impl_bundle_tuple!(A, B);
impl_bundle_tuple!(A, B, C);
impl_bundle_tuple!(A, B, C, D);
