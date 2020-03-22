use imgui::Io;

pub struct Camera {
    pub position: glam::Vec3,
    pub target_position: glam::Vec3,
    pub azimuth: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(start_position: glam::Vec3) -> Camera {
        Camera { position: start_position, target_position: start_position, azimuth: 0.0, pitch: 0.0 }
    }

    pub fn update(&mut self, dt: f32, translation: glam::Vec2, rotation: glam::Vec2, sprint: f32) {
        let pitch_transform = glam::Mat4::from_axis_angle(glam::Vec3::new(1.0, 0.0, 0.0), self.pitch);
        let azimuth_transform = glam::Mat4::from_axis_angle(glam::Vec3::new(0.0, 1.0, 0.0), self.azimuth);
        let camera_rotation = pitch_transform * azimuth_transform;

        let forward = camera_rotation.inverse().transform_vector3(glam::Vec3::new(0.0, 0.0, 1.0));
        let left = forward.cross(glam::Vec3::new(0.0, 1.0, 0.0));

        self.azimuth -= 50.0 * rotation.x() * dt;
        self.pitch -= 50.0 * rotation.y() * dt;
        self.target_position -= sprint * forward * dt * translation.y() + sprint * left * dt * translation.x();
        self.position = self.position * 0.8 + self.target_position * 0.2;
    }

    pub fn update_from_io(&mut self, io: &Io) {
        let dt = io.delta_time;

        let delta_mouse_x = io.mouse_delta[0];
        let delta_mouse_y = io.mouse_delta[1];

        let w = io.keys_down[('w' as usize - 'a' as usize) + 10];
        let a = io.keys_down[('a' as usize - 'a' as usize) + 10];
        let s = io.keys_down[('s' as usize - 'a' as usize) + 10];
        let d = io.keys_down[('d' as usize - 'a' as usize) + 10];

        let rotation = if io.mouse_down[1] {
            glam::Vec2::new(-delta_mouse_x * dt * 0.1, -delta_mouse_y * dt * 0.1)
        } else {
            glam::Vec2::zero()
        };

        let speed = if io.key_shift { 60.0 } else { 10.0 };
        let translation_x = if a { -1.0 } else if d { 1.0 } else { 0.0 };
        let translation_y = if w { 1.0 } else if s { -1.0 } else { 0.0 };

        self.update(dt, glam::Vec2::new(translation_x, translation_y), rotation, speed)
    }

    pub fn transform(&self) -> glam::Mat4 {
        let pitch_transform = glam::Mat4::from_axis_angle(glam::Vec3::new(1.0, 0.0, 0.0), self.pitch);
        let azimuth_transform = glam::Mat4::from_axis_angle(glam::Vec3::new(0.0, 1.0, 0.0), self.azimuth);
        return (pitch_transform * azimuth_transform) * glam::Mat4::from_translation(-self.position);
    }
}