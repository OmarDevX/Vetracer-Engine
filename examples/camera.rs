extern crate glm;

use glm::*;
use glm::ext::look_at;
use glm::ext::perspective;

// Function to compute cross product of two Vec3 vectors
fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub fov: f32, // Field of View in radians
    pub acceleration: f32,
    pub deceleration: f32,
    pub friction: f32,
    pub velocity: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, up: Vec3, yaw: f32, pitch: f32, fov: f32) -> Camera {
        let front = glm::vec3(
            yaw.to_radians().cos() * pitch.to_radians().cos(),
            pitch.to_radians().sin(),
            yaw.to_radians().sin() * pitch.to_radians().cos(),
        );

        let right = glm::normalize(cross(&front, &up));
        let up = glm::normalize(cross(&right, &front));

        Camera {
            position,
            front,
            up,
            right,
            world_up: glm::normalize(up),
            yaw,
            pitch,
            speed: 0.4,
            sensitivity: 0.1,
            fov: fov.to_radians(), // Convert FOV to radians
            acceleration: 0.5,
            deceleration: 2.0,
            friction: 0.85,
            velocity: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn view_matrix(&self, aspect_ratio: f32) -> Mat4 {
        perspective(self.fov, aspect_ratio, 0.1, 1000.0) * look_at(self.position, self.position + self.front, self.up)
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let acceleration = self.acceleration * delta_time;
        let deceleration = self.deceleration * delta_time;

        match direction {
            CameraMovement::Forward => self.velocity =self.velocity + self.front * acceleration,
            CameraMovement::Backward => self.velocity =self.velocity - self.front * acceleration,
            CameraMovement::Left => self.velocity =self.velocity - self.right * acceleration,
            CameraMovement::Right => self.velocity =self.velocity + self.right * acceleration,
        }
    }

    pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32, constrain_pitch: bool) {
        let x_offset = x_offset * self.sensitivity;
        let y_offset = y_offset * self.sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        // Calculate new front vector
        let front = glm::vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );

        // Recalculate right and up vectors based on the new front vector and world up
        self.right = glm::normalize(cross(&front, &self.world_up));
        self.up = glm::normalize(cross(&self.right, &front));
        self.front = front;
    }

    pub fn update_motion(&mut self, delta_time: f32) {
        self.velocity =self.velocity * self.friction.powf(delta_time);

        if dot(self.velocity,self.velocity) < 0.001 {
            self.velocity = vec3(0.0,0.0,0.0);
        }

        self.position =self.position + self.velocity * delta_time;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.update_motion(delta_time);
    }
}

pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
}
