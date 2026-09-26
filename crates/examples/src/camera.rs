use std::time::Instant;

use vyas::prelude::*;

pub(crate) struct NoClipCamera {
    yaw: f32,
    pitch: f32,
    speed: f32,
    last_update: Instant,
}

impl NoClipCamera {
    pub(crate) fn new(position: &WorldPosition, looking_at: &WorldPosition) -> Self {
        let x = looking_at.x - position.x;
        let y = looking_at.y - position.y;
        let z = looking_at.z - position.z;
        let length = (x * x + y * y + z * z).sqrt();

        Self {
            yaw: x.atan2(z),
            pitch: (y / length).asin(),
            speed: 150.0,
            last_update: Instant::now(),
        }
    }

    fn update_camera(&mut self, camera: &mut CameraState, input: &InputState) {
        const MOUSE_SENSITIVITY: f32 = 0.0025;
        const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32().min(0.05);
        self.last_update = now;

        let (mouse_x, mouse_y) = input.mouse_delta();
        self.yaw += mouse_x as f32 * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - mouse_y as f32 * MOUSE_SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH);

        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();

        let forward = Vec3 {
            x: sin_yaw * cos_pitch,
            y: sin_pitch,
            z: cos_yaw * cos_pitch,
        };
        let right = Vec3 {
            x: cos_yaw,
            y: 0.0,
            z: -sin_yaw,
        };

        let mut movement = Vec3::default();

        if input.pressed(InputButton::Key(KeyCode::KeyW)) {
            movement.add(forward);
        }

        if input.pressed(InputButton::Key(KeyCode::KeyS)) {
            movement.subtract(forward);
        }

        if input.pressed(InputButton::Key(KeyCode::KeyA)) {
            movement.subtract(right);
        }

        if input.pressed(InputButton::Key(KeyCode::KeyD)) {
            movement.add(right);
        }

        if input.pressed(InputButton::Key(KeyCode::Space)) {
            movement.y += 1.0;
        }

        if input.pressed(InputButton::Modifier(Modifier::Shift)) {
            movement.y -= 1.0;
        }

        let movement_length = movement.length();

        if movement_length > 0.0 {
            let distance = self.speed * delta_time / movement_length;
            camera.position.x += movement.x * distance;
            camera.position.y += movement.y * distance;
            camera.position.z += movement.z * distance;
        }

        camera.looking_at.x = camera.position.x + forward.x;
        camera.looking_at.y = camera.position.y + forward.y;
        camera.looking_at.z = camera.position.z + forward.z;
    }

    pub(crate) fn update(mut camera: Camera, input: Input, mut controller: ResMut<NoClipCamera>) {
        controller.update_camera(&mut camera, &input);
    }
}

#[derive(Clone, Copy, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn add(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }

    fn subtract(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}
