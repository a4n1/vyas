use crate::{
    color::Color,
    ecs::{Bundle, Entity, World},
    position::Direction,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightsGpu {
    pub ambient_light: [f32; 4],
    pub directional_light: [f32; 4],
    pub direction: [f32; 4],
}

impl LightsGpu {
    pub(crate) fn new(ambient_light: &AmbientLight, directional_light: &DirectionalLight) -> Self {
        let [ambient_r, ambient_g, ambient_b] = ambient_light.color.into();
        let [directional_r, directional_g, directional_b] = directional_light.color.into();
        let direction = directional_light.direction;
        let direction = glam::Vec3::new(direction.x, direction.y, direction.z).normalize_or_zero();

        Self {
            ambient_light: [ambient_r, ambient_g, ambient_b, ambient_light.brightness],
            directional_light: [
                directional_r,
                directional_g,
                directional_b,
                directional_light.illuminance,
            ],
            direction: [direction.x, direction.y, direction.z, 0.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AmbientLight {
    pub color: Color,
    pub brightness: f32,
}

impl Bundle for AmbientLight {
    fn insert_into(self, _entity: Entity, world: &mut World) {
        world.insert_resource::<Self>(self);
    }
}

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub color: Color,
    pub illuminance: f32,
    pub direction: Direction,
}

impl Bundle for DirectionalLight {
    fn insert_into(self, _entity: Entity, world: &mut World) {
        world.insert_resource::<Self>(self);
    }
}
