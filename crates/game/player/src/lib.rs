//! Player controller with physics, movement states, and camera

use lithos_engine_math::{FixedVec3, Fixed, Vec3, Mat4};
use lithos_simulation_voxel::{VoxelStorage, VoxelCollider, Capsule, is_on_ground, get_block_at, BlockId};
use serde::{Serialize, Deserialize};

/// Player movement state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementState {
    /// Standing on ground
    Grounded,
    /// Walking on ground
    Walking,
    /// Sprinting
    Sprinting,
    /// Sneaking/crouching
    Sneaking,
    /// Jumping
    Jumping,
    /// Falling
    Falling,
    /// Swimming
    Swimming,
    /// Flying (creative/spectator)
    Flying,
    /// Climbing (ladders/vines)
    Climbing,
}

/// Player physics constants
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PlayerPhysics {
    /// Capsule height when standing
    pub height_standing: Fixed,
    /// Capsule height when sneaking
    pub height_sneaking: Fixed,
    /// Capsule radius
    pub radius: Fixed,
    /// Eye height offset from feet
    pub eye_height: Fixed,
    /// Eye height when sneaking
    pub eye_height_sneaking: Fixed,
    
    /// Walk speed (blocks per second)
    pub walk_speed: Fixed,
    /// Sprint speed multiplier
    pub sprint_multiplier: Fixed,
    /// Sneak speed multiplier
    pub sneak_multiplier: Fixed,
    /// Swim speed
    pub swim_speed: Fixed,
    /// Fly speed
    pub fly_speed: Fixed,
    
    /// Jump velocity
    pub jump_velocity: Fixed,
    /// Gravity acceleration
    pub gravity: Fixed,
    /// Terminal velocity
    pub terminal_velocity: Fixed,
    /// Air control factor (0-1)
    pub air_control: Fixed,
    /// Ground friction
    pub ground_friction: Fixed,
    /// Air friction
    pub air_friction: Fixed,
    /// Water friction
    pub water_friction: Fixed,
    
    /// Step height (auto-step up blocks)
    pub step_height: Fixed,
    /// Max slope angle (radians) for walking
    pub max_slope: Fixed,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            height_standing: Fixed::from_f32(1.8),
            height_sneaking: Fixed::from_f32(1.5),
            radius: Fixed::from_f32(0.3),
            eye_height: Fixed::from_f32(1.62),
            eye_height_sneaking: Fixed::from_f32(1.27),
            
            walk_speed: Fixed::from_f32(4.317),  // ~4.3 m/s
            sprint_multiplier: Fixed::from_f32(1.3),
            sneak_multiplier: Fixed::from_f32(0.3),
            swim_speed: Fixed::from_f32(2.0),
            fly_speed: Fixed::from_f32(10.0),
            
            jump_velocity: Fixed::from_f32(6.5),
            gravity: Fixed::from_f32(20.0),  // m/s²
            terminal_velocity: Fixed::from_f32(78.0),
            air_control: Fixed::from_f32(0.02),
            ground_friction: Fixed::from_f32(10.0),
            air_friction: Fixed::from_f32(0.02),
            water_friction: Fixed::from_f32(5.0),
            
            step_height: Fixed::from_f32(0.6),
            max_slope: Fixed::from_f32(0.785),  // ~45 degrees
        }
    }
}

/// Player input state
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub sneak: bool,
    pub fly_up: bool,
    pub fly_down: bool,
}

/// Player controller
pub struct PlayerController {
    /// Current position (feet)
    pub position: FixedVec3,
    /// Current velocity
    pub velocity: FixedVec3,
    /// Camera rotation (yaw, pitch)
    pub rotation: (Fixed, Fixed),  // yaw, pitch in radians
    /// Current movement state
    pub movement_state: MovementState,
    /// Physics parameters
    pub physics: PlayerPhysics,
    /// Whether player is in creative mode (fly/no-clip)
    pub creative: bool,
    /// Whether player is on ground (cached)
    on_ground: bool,
    /// Jump cooldown ticks
    jump_cooldown: u32,
    /// Tick counter
    tick: u64,
}

impl PlayerController {
    /// Create new player controller at position
    pub fn new(position: FixedVec3, physics: PlayerPhysics) -> Self {
        Self {
            position,
            velocity: FixedVec3::zero(),
            rotation: (Fixed::ZERO, Fixed::ZERO),
            movement_state: MovementState::Grounded,
            physics,
            creative: false,
            on_ground: false,
            jump_cooldown: 0,
            tick: 0,
        }
    }

    /// Create with default physics
    pub fn default_at(position: FixedVec3) -> Self {
        Self::new(position, PlayerPhysics::default())
    }

    /// Get capsule for current state
    fn capsule(&self) -> Capsule {
        let height = if self.movement_state == MovementState::Sneaking {
            self.physics.height_sneaking
        } else {
            self.physics.height_standing
        };
        Capsule::from_center_height_radius(
            self.position + FixedVec3::new(Fixed::ZERO, height / Fixed::from_int(2), Fixed::ZERO),
            height,
            self.physics.radius,
        )
    }

    /// Get eye position
    pub fn eye_position(&self) -> FixedVec3 {
        let height = if self.movement_state == MovementState::Sneaking {
            self.physics.eye_height_sneaking
        } else {
            self.physics.eye_height
        };
        self.position + FixedVec3::new(Fixed::ZERO, height, Fixed::ZERO)
    }

    /// Get forward vector from yaw
    pub fn forward_vector(&self) -> FixedVec3 {
        let (yaw, _) = self.rotation;
        let cos = yaw.cos();
        let sin = yaw.sin();
        FixedVec3::new(-sin, Fixed::ZERO, cos)
    }

    /// Get right vector from yaw
    pub fn right_vector(&self) -> FixedVec3 {
        let (yaw, _) = self.rotation;
        let cos = yaw.cos();
        let sin = yaw.sin();
        FixedVec3::new(cos, Fixed::ZERO, sin)
    }

    /// Update player state for one tick
    pub fn update(&mut self, input: PlayerInput, storage: &VoxelStorage, dt: Fixed) {
        self.tick += 1;
        
        if self.jump_cooldown > 0 {
            self.jump_cooldown -= 1;
        }

        // Handle creative flying
        if self.creative && input.fly_up {
            self.velocity.y = self.physics.fly_speed;
            self.movement_state = MovementState::Flying;
            self.position = self.position + self.velocity * dt;
            return;
        }
        if self.creative && input.fly_down {
            self.velocity.y = -self.physics.fly_speed;
            self.movement_state = MovementState::Flying;
            self.position = self.position + self.velocity * dt;
            return;
        }
        if self.creative && (input.forward || input.backward || input.left || input.right) {
            self.movement_state = MovementState::Flying;
            let move_dir = self.move_direction(input);
            if move_dir.length_squared() > Fixed::ZERO {
                self.velocity = move_dir.normalize() * self.physics.fly_speed;
            } else {
                self.velocity = FixedVec3::zero();
            }
            self.position = self.position + self.velocity * dt;
            return;
        }

        // Check ground state
        self.on_ground = is_on_ground(storage, self.position, self.physics.radius, self.capsule().height());
        
        // Check if in water
        let in_water = self.is_in_water(storage);
        let in_lava = false; // TODO: implement lava check

        // Update movement state
        self.update_movement_state(input, in_water);

        // Apply physics
        self.apply_physics(input, storage, dt, in_water, in_lava);
        
        // Collision resolution
        self.resolve_collisions(storage, dt);
    }

    /// Update movement state based on input and environment
    fn update_movement_state(&mut self, input: PlayerInput, in_water: bool) {
        if in_water {
            self.movement_state = MovementState::Swimming;
            return;
        }

        if !self.on_ground {
            if self.velocity.y > Fixed::ZERO {
                self.movement_state = MovementState::Jumping;
            } else {
                self.movement_state = MovementState::Falling;
            }
            return;
        }

        if input.sneak {
            self.movement_state = MovementState::Sneaking;
        } else if input.sprint && (input.forward || input.backward || input.left || input.right) {
            self.movement_state = MovementState::Sprinting;
        } else if input.forward || input.backward || input.left || input.right {
            self.movement_state = MovementState::Walking;
        } else {
            self.movement_state = MovementState::Grounded;
        }
    }

    /// Calculate movement direction from input
    fn move_direction(&self, input: PlayerInput) -> FixedVec3 {
        let mut dir = FixedVec3::zero();
        let forward = self.forward_vector();
        let right = self.right_vector();

        if input.forward {
            dir = dir + forward;
        }
        if input.backward {
            dir = dir - forward;
        }
        if input.right {
            dir = dir + right;
        }
        if input.left {
            dir = dir - right;
        }

        dir
    }

    /// Get current move speed based on state
    fn move_speed(&self) -> Fixed {
        match self.movement_state {
            MovementState::Sprinting => self.physics.walk_speed * self.physics.sprint_multiplier,
            MovementState::Sneaking => self.physics.walk_speed * self.physics.sneak_multiplier,
            MovementState::Swimming => self.physics.swim_speed,
            MovementState::Flying => self.physics.fly_speed,
            _ => self.physics.walk_speed,
        }
    }

    /// Apply physics forces
    fn apply_physics(&mut self, input: PlayerInput, _storage: &VoxelStorage, dt: Fixed, in_water: bool, _in_lava: bool) {
        let move_dir = self.move_direction(input);
        let speed = self.move_speed();

        // Horizontal movement
        if move_dir.length_squared() > Fixed::ZERO {
            let move_dir_norm = move_dir.normalize();
            let target_vel = move_dir_norm * speed;
            
            let control = if self.on_ground || in_water {
                Fixed::ONE
            } else {
                self.physics.air_control
            };
            
            self.velocity.x = self.velocity.x + (target_vel.x - self.velocity.x) * control;
            self.velocity.z = self.velocity.z + (target_vel.z - self.velocity.z) * control;
        } else {
            // Apply friction
            let friction = if in_water {
                self.physics.water_friction
            } else if self.on_ground {
                self.physics.ground_friction
            } else {
                self.physics.air_friction
            };
            
            self.velocity.x = self.velocity.x * (Fixed::ONE - friction * dt);
            self.velocity.z = self.velocity.z * (Fixed::ONE - friction * dt);
        }

        // Jumping
        if input.jump && self.on_ground && self.jump_cooldown == 0 {
            self.velocity.y = self.physics.jump_velocity;
            self.jump_cooldown = 5; // 5 ticks cooldown
            self.on_ground = false;
        }

        // Gravity
        if !in_water {
            self.velocity.y = self.velocity.y - self.physics.gravity * dt;
            
            // Terminal velocity
            if self.velocity.y < -self.physics.terminal_velocity {
                self.velocity.y = -self.physics.terminal_velocity;
            }
        } else {
            // Water buoyancy
            if input.jump {
                self.velocity.y = self.physics.swim_speed * Fixed::from_f32(0.5);
            } else {
                self.velocity.y = self.velocity.y - self.physics.gravity * dt * Fixed::from_f32(0.1);
            }
        }

        // Apply velocity
        self.position = self.position + self.velocity * dt;
    }

    /// Resolve collisions with voxel world
    fn resolve_collisions(&mut self, storage: &VoxelStorage, dt: Fixed) {
        let mut collider = VoxelCollider::new(storage);
        let mut capsule = self.capsule();
        
        // X axis
        let mut test_capsule = capsule.translate(FixedVec3::new(self.velocity.x * dt, Fixed::ZERO, Fixed::ZERO));
        let result = collider.check_capsule(test_capsule);
        if result.collided {
            // Slide along wall
            let slide = result.collision_normal * self.velocity.x.abs() * dt;
            self.velocity.x = Fixed::ZERO;
            test_capsule = test_capsule.translate(slide);
            let result2 = collider.check_capsule(test_capsule);
            if !result2.collided {
                capsule = test_capsule;
            }
        } else {
            capsule = test_capsule;
        }

        // Z axis
        test_capsule = capsule.translate(FixedVec3::new(Fixed::ZERO, Fixed::ZERO, self.velocity.z * dt));
        let result = collider.check_capsule(test_capsule);
        if result.collided {
            let slide = result.collision_normal * self.velocity.z.abs() * dt;
            self.velocity.z = Fixed::ZERO;
            test_capsule = test_capsule.translate(slide);
            let result2 = collider.check_capsule(test_capsule);
            if !result2.collided {
                capsule = test_capsule;
            }
        } else {
            capsule = test_capsule;
        }

        // Y axis (vertical)
        test_capsule = capsule.translate(FixedVec3::new(Fixed::ZERO, self.velocity.y * dt, Fixed::ZERO));
        let result = collider.check_capsule(test_capsule);
        if result.collided {
            if self.velocity.y > Fixed::ZERO {
                // Hit ceiling
                self.velocity.y = Fixed::ZERO;
            } else {
                // Hit floor
                self.velocity.y = Fixed::ZERO;
                self.on_ground = true;
            }
        } else {
            capsule = test_capsule;
        }

        // Update position from final capsule
        let height = if self.movement_state == MovementState::Sneaking {
            self.physics.height_sneaking
        } else {
            self.physics.height_standing
        };
        self.position = capsule.center() - FixedVec3::new(Fixed::ZERO, height / Fixed::from_int(2), Fixed::ZERO);
    }

    /// Check if player is in water
    fn is_in_water(&self, storage: &VoxelStorage) -> bool {
        let head_pos = self.eye_position();
        let block = get_block_at(storage, head_pos);
        matches!(block.block_id, BlockId::WATER)
    }

    /// Set rotation (yaw, pitch in radians)
    pub fn set_rotation(&mut self, yaw: Fixed, pitch: Fixed) {
        self.rotation.0 = yaw;
        // Clamp pitch to [-pi/2, pi/2]
        let half_pi = Fixed::from_f32(std::f32::consts::FRAC_PI_2);
        self.rotation.1 = pitch.clamp(-half_pi, half_pi);
    }

    /// Add to rotation (for mouse input)
    pub fn add_rotation(&mut self, yaw_delta: Fixed, pitch_delta: Fixed) {
        self.rotation.0 = self.rotation.0 + yaw_delta;
        let half_pi = Fixed::from_f32(std::f32::consts::FRAC_PI_2);
        self.rotation.1 = (self.rotation.1 + pitch_delta).clamp(-half_pi, half_pi);
    }

    /// Set creative mode
    pub fn set_creative(&mut self, creative: bool) {
        self.creative = creative;
        if creative {
            self.velocity = FixedVec3::zero();
        }
    }

    /// Teleport to position
    pub fn teleport(&mut self, position: FixedVec3) {
        self.position = position;
        self.velocity = FixedVec3::zero();
    }

    /// Get view matrix for rendering
    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.eye_position().to_f32_vec3();
        let (yaw, pitch) = self.rotation;
        
        let forward = Vec3::new(
            (-yaw.to_f32()).sin() * pitch.to_f32().cos(),
            pitch.to_f32().sin(),
            yaw.to_f32().cos() * pitch.to_f32().cos(),
        );
        
        let center = eye + forward;
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        Mat4::look_at(eye, center, up)
    }

    /// Get projection matrix
    pub fn projection_matrix(&self, aspect: f32, fov: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective(fov, aspect, near, far)
    }
}

/// Camera system (first/third person)
pub struct Camera {
    /// Camera mode
    pub mode: CameraMode,
    /// Distance from player (third person)
    pub distance: Fixed,
    /// Pitch offset (third person)
    pub pitch_offset: Fixed,
    /// Yaw offset (third person)
    pub yaw_offset: Fixed,
    /// Smooth interpolation factor
    pub smooth_factor: Fixed,
    /// Current smoothed position
    smoothed_position: FixedVec3,
    /// Current smoothed rotation
    smoothed_rotation: (Fixed, Fixed),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            mode: CameraMode::FirstPerson,
            distance: Fixed::from_f32(4.0),
            pitch_offset: Fixed::from_f32(-0.3),
            yaw_offset: Fixed::ZERO,
            smooth_factor: Fixed::from_f32(0.1),
            smoothed_position: FixedVec3::zero(),
            smoothed_rotation: (Fixed::ZERO, Fixed::ZERO),
        }
    }
}

impl Camera {
    /// Update camera based on player state
    pub fn update(&mut self, player: &PlayerController, dt: Fixed) {
        let target_pos = match self.mode {
            CameraMode::FirstPerson => player.eye_position(),
            CameraMode::ThirdPerson | CameraMode::ThirdPersonBack | CameraMode::ThirdPersonFront => {
                let (yaw, pitch) = player.rotation;
                let yaw = yaw + self.yaw_offset;
                let pitch = pitch + self.pitch_offset;
                
                let cos_pitch = pitch.cos();
                let offset = FixedVec3::new(
                    (-yaw).sin() * cos_pitch * self.distance,
                    pitch.sin() * self.distance,
                    yaw.cos() * cos_pitch * self.distance,
                );
                
                player.eye_position() - offset
            }
        };

        let target_rot = match self.mode {
            CameraMode::FirstPerson => player.rotation,
            CameraMode::ThirdPerson | CameraMode::ThirdPersonBack => {
                (player.rotation.0 + self.yaw_offset, player.rotation.1 + self.pitch_offset)
            }
            CameraMode::ThirdPersonFront => {
                (player.rotation.0 + Fixed::from_f32(std::f32::consts::PI) + self.yaw_offset, 
                 -player.rotation.1 + self.pitch_offset)
            }
        };

        // Smooth interpolation
        let t = self.smooth_factor * dt * Fixed::from_int(60); // 60 Hz reference
        self.smoothed_position = self.smoothed_position.lerp(target_pos, t);
        self.smoothed_rotation.0 = self.smoothed_rotation.0.lerp(target_rot.0, t);
        self.smoothed_rotation.1 = self.smoothed_rotation.1.lerp(target_rot.1, t);
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.smoothed_position.to_f32_vec3();
        let (yaw, pitch) = self.smoothed_rotation;
        
        let forward = Vec3::new(
            (-yaw.to_f32()).sin() * pitch.to_f32().cos(),
            pitch.to_f32().sin(),
            yaw.to_f32().cos() * pitch.to_f32().cos(),
        );
        
        let center = eye + forward;
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        Mat4::look_at(eye, center, up)
    }

    /// Get projection matrix
    pub fn projection_matrix(&self, aspect: f32, fov: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective(fov, aspect, near, far)
    }

    /// Set camera mode
    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }

    /// Cycle camera mode
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            CameraMode::FirstPerson => CameraMode::ThirdPersonBack,
            CameraMode::ThirdPersonBack => CameraMode::ThirdPersonFront,
            CameraMode::ThirdPersonFront => CameraMode::FirstPerson,
            CameraMode::ThirdPerson => CameraMode::FirstPerson,
        };
    }

    /// Set distance (third person)
    pub fn set_distance(&mut self, distance: Fixed) {
        self.distance = distance.max(Fixed::from_f32(0.5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_player_creation() {
        let player = PlayerController::default_at(FixedVec3::new(Fixed::ZERO, Fixed::from_int(100), Fixed::ZERO));
        assert_eq!(player.movement_state, MovementState::Grounded);
        assert_eq!(player.position.y, Fixed::from_int(100));
    }

    #[test]
    fn test_capsule() {
        let physics = PlayerPhysics::default();
        let player = PlayerController::new(FixedVec3::zero(), physics);
        let capsule = player.capsule();
        assert_eq!(capsule.height(), physics.height_standing);
        assert_eq!(capsule.radius, physics.radius);
    }

    #[test]
    fn test_eye_position() {
        let physics = PlayerPhysics::default();
        let mut player = PlayerController::new(FixedVec3::zero(), physics);
        let eye = player.eye_position();
        assert_eq!(eye.y, physics.eye_height);
        
        player.movement_state = MovementState::Sneaking;
        let eye_sneak = player.eye_position();
        assert_eq!(eye_sneak.y, physics.eye_height_sneaking);
    }

    #[test]
    fn test_movement_direction() {
        let physics = PlayerPhysics::default();
        let mut player = PlayerController::new(FixedVec3::zero(), physics);
        player.rotation.0 = Fixed::from_f32(0.0); // Looking +Z
        
        let input = PlayerInput { forward: true, ..Default::default() };
        let dir = player.move_direction(input);
        assert!(dir.z > Fixed::ZERO); // Moving forward in +Z
        
        player.rotation.0 = Fixed::from_f32(std::f32::consts::FRAC_PI_2); // Looking +X
        let dir = player.move_direction(input);
        assert!(dir.x > Fixed::ZERO); // Moving forward in +X
    }

    #[test]
    fn test_camera_first_person() {
        let physics = PlayerPhysics::default();
        let player = PlayerController::default_at(FixedVec3::zero());
        let mut camera = Camera::default();
        camera.set_mode(CameraMode::FirstPerson);
        camera.update(&player, Fixed::from_f32(1.0/60.0));
        
        let view = camera.view_matrix();
        // Just verify it produces a valid matrix
        assert!(view.m[3][3] != 0.0);
    }
}