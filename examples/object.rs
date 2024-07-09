use glm::{dot, vec3};

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Object {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub roughness: f32,
    pub emission: f32,
    pub is_static: bool,
    pub angular_velocity: [f32; 3], // Angular velocity in radians per second
    pub angular_acceleration: [f32; 3], // Angular acceleration in radians per second^2
    pub orientation: [f32; 4], // Quaternion representing rotation
    pub mass:f32,
    pub is_cube:bool,
    pub size:[f32;3],
    pub is_glass:bool,
    pub reflectness:f32,
}

impl Object {
    pub fn new(position: [f32; 3], radius: f32, color: [f32; 3], roughness: f32, emission: f32, is_static: bool) -> Self {
        Object {
            position,
            velocity: [0.0;3],
            acceleration: [0.0;3],
            radius,
            color,
            roughness,
            emission,
            is_static,
            angular_velocity: [0.0;3],
            angular_acceleration: [0.0;3],
            orientation: [1.0, 0.0, 0.0, 0.0], // Identity quaternion (no rotation)
            mass:1.0,
            is_cube:true,
            size: [1.0;3],
            is_glass: false,
            reflectness: 0.0,
        }
    }

pub fn process_physics(&mut self, delta_time: f32, spheres: &mut [Object]) {
    let mut apply_gravity = true;

    if !self.is_static {
        for other in spheres.iter_mut() {
            if self.position != other.position&&self!=other {
                let distance = Object::distance_between_spheres(self, other);
                if distance < self.radius + other.radius {
                    self.resolve_collision(other);
                    apply_gravity=false
                }
            }
        }
        if apply_gravity{
        // Apply gravity
        self.acceleration[1] = -9.81;

        self.velocity[0] += self.acceleration[0] * delta_time;
        self.velocity[1] += self.acceleration[1] * delta_time;
        self.velocity[2] += self.acceleration[2] * delta_time;

        // Limit velocity to prevent excessive speed
        let max_speed: f32 = 5.0; // Adjust as needed
        let speed_squared = self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2);
        if speed_squared > max_speed.powi(2) {
            let speed = speed_squared.sqrt();
            self.velocity[0] = (self.velocity[0] / speed) * max_speed;
            self.velocity[1] = (self.velocity[1] / speed) * max_speed;
            self.velocity[2] = (self.velocity[2] / speed) * max_speed;
        }

        // Update position using Verlet integration
        self.position[0] += self.velocity[0] * delta_time;
        self.position[1] += self.velocity[1] * delta_time;
        self.position[2] += self.velocity[2] * delta_time;
    }
        // Update angular velocity and orientation
        self.angular_velocity[0] += self.angular_acceleration[0] * delta_time;
        self.angular_velocity[1] += self.angular_acceleration[1] * delta_time;
        self.angular_velocity[2] += self.angular_acceleration[2] * delta_time;

        // Limit angular velocity to prevent excessive rotation
        let max_angular_speed: f32 = 10.0; // Adjust as needed
        let angular_speed_squared = self.angular_velocity[0].powi(2) + self.angular_velocity[1].powi(2) + self.angular_velocity[2].powi(2);
        if angular_speed_squared > max_angular_speed.powi(2) {
            let angular_speed = angular_speed_squared.sqrt();
            self.angular_velocity[0] = self.angular_velocity[0] / angular_speed * max_angular_speed;
            self.angular_velocity[1] = self.angular_velocity[1] / angular_speed * max_angular_speed;
            self.angular_velocity[2] = self.angular_velocity[2] / angular_speed * max_angular_speed;
        }
        let tVec=vec3(self.velocity[0], self.velocity[1], self.velocity[2]);
        if dot(tVec,tVec) < 0.5 {
            self.velocity=[0.0;3];
        }

        self.update_orientation(delta_time);
    }

    // self.check_collision(spheres);
}



    fn update_orientation(&mut self, delta_time: f32) {
        let angle = (self.angular_velocity[0].powi(2) + self.angular_velocity[1].powi(2) + self.angular_velocity[2].powi(2)).sqrt() * delta_time;
        if angle != 0.0 {
            let axis = [
                self.angular_velocity[0] / angle,
                self.angular_velocity[1] / angle,
                self.angular_velocity[2] / angle,
            ];
            let half_angle = angle * 0.5;
            let sin_half_angle = half_angle.sin();
            let delta_orientation = [
                half_angle.cos(),
                axis[0] * sin_half_angle,
                axis[1] * sin_half_angle,
                axis[2] * sin_half_angle,
            ];

            self.orientation = Object::quaternion_multiply(self.orientation, delta_orientation);
        }
    }

    fn quaternion_multiply(q1: [f32; 4], q2: [f32; 4]) -> [f32; 4] {
        [
            q1[0] * q2[0] - q1[1] * q2[1] - q1[2] * q2[2] - q1[3] * q2[3],
            q1[0] * q2[1] + q1[1] * q2[0] + q1[2] * q2[3] - q1[3] * q2[2],
            q1[0] * q2[2] - q1[1] * q2[3] + q1[2] * q2[0] + q1[3] * q2[1],
            q1[0] * q2[3] + q1[1] * q2[2] - q1[2] * q2[1] + q1[3] * q2[0],
        ]
    }

    // fn check_collision(&mut self, spheres: &mut [Sphere]) {
    //     for other in spheres.iter_mut() {
    //         if self.position != other.position&&self!=other {
    //             let distance = Sphere::distance_between_spheres(self, other);
    //             if distance < self.radius + other.radius {
    //                 self.resolve_collision(other);
    //             }
    //         }
    //     }
    // }
    
fn resolve_collision(&mut self, other: &mut Object) {
    if self != other {
        let distance = Object::distance_between_spheres(self, other);
        let penetration_depth = self.radius + other.radius - distance;
        if penetration_depth > 0.0 {
            // Collision normal (direction from self to other)
            let collision_normal = [
                (other.position[0] - self.position[0]) / distance,
                (other.position[1] - self.position[1]) / distance,
                (other.position[2] - self.position[2]) / distance,
            ];
            let mut myPos=self.position;
            let mut otherPos=other.position;
            if !self.is_static && other.is_static {
                // Non-static sphere hits static sphere, adjust non-static sphere's position
                myPos[0] -= collision_normal[0] * penetration_depth;
                myPos[1] -= collision_normal[1] * penetration_depth;
                myPos[2] -= collision_normal[2] * penetration_depth;
            } else if !other.is_static && self.is_static {
                // Static sphere hits non-static sphere, adjust non-static sphere's position
                otherPos[0] += collision_normal[0] * penetration_depth;
                otherPos[1] += collision_normal[1] * penetration_depth;
                otherPos[2] += collision_normal[2] * penetration_depth;
            } else {
                // Both spheres are non-static, adjust both positions
                let correction_factor = penetration_depth / (self.radius + other.radius);
                myPos[0] -= collision_normal[0] * self.radius * correction_factor;
                myPos[1] -= collision_normal[1] * self.radius * correction_factor;
                myPos[2] -= collision_normal[2] * self.radius * correction_factor;
                otherPos[0] += collision_normal[0] * other.radius * correction_factor;
                otherPos[1] += collision_normal[1] * other.radius * correction_factor;
                otherPos[2] += collision_normal[2] * other.radius * correction_factor;
            }
            self.position=myPos;
            other.position=otherPos;
        }
    }
}



    
pub fn update(&mut self, delta_time: f32, spheres: &mut [Object]) {
    let fixed_delta_time = 0.016; // Example: Fixed time step of 0.016 seconds (60 FPS)
    let mut time_accumulator = delta_time;

    while time_accumulator >= fixed_delta_time {
        self.process_physics(fixed_delta_time, spheres);
        time_accumulator -= fixed_delta_time;
    }

    // Process remaining time (if any) with a smaller time step
    if time_accumulator > 0.0 {
        self.process_physics(time_accumulator, spheres);
    }
}



    fn distance_between_spheres(s1: &Object, s2: &Object) -> f32 {
        let dx = s1.position[0] - s2.position[0];
        let dy = s1.position[1] - s2.position[1];
        let dz = s1.position[2] - s2.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
