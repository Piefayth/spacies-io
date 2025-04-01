use avian3d::prelude::{collider, Collider, CollisionMargin, LinearVelocity, Position, RigidBody, Rotation};
use bevy::{gltf::GltfMesh, prelude::*};
use leafwing_input_manager::{axislike::DualAxisType, prelude::{ActionState, GamepadStick, InputMap, MouseMove, VirtualDPad}};
use lightyear::prelude::{
    client::{Confirmed, Interpolated, Predicted},
    server::ReplicationTarget,
};
use mygame_assets::{LevelState, assets::GlobalAssets};
use mygame_protocol::{component::Player, input::NetworkedInput};

use crate::{Rendered, Simulated};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_player_gameplay_components).run_if(in_state(LevelState::Loaded)),
        );

        app.add_systems(FixedUpdate, move_player);
    }
}

fn add_player_gameplay_components(
    mut commands: Commands,
    q_rendered_player: Query<Entity, (Rendered, Without<RigidBody>, With<Player>)>,
    global_assets: Res<GlobalAssets>,
) {
    if q_rendered_player.is_empty() {
        return;
    }

    for player_entity in &q_rendered_player {
        commands.entity(player_entity).insert((
            RigidBody::Kinematic,
            Collider::sphere(1.0),
            CollisionMargin(0.1),
            ActionState::<NetworkedInput>::default(),
            SceneRoot(global_assets.character.clone()),
        ));
    }
}

const PLAYER_MOVE_SPEED: f32 = 10.0;
const MAX_ROLL_ANGLE: f32 = std::f32::consts::FRAC_PI_2;  // 90 degrees
const MAX_PITCH_ANGLE: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees
const TURN_RATE: f32 = 1.0;
const PITCH_RATE: f32 = 1.0;

// given some kind of boundary
// -1 on the x axis of the aim input would mean "look all the way to the left of that boundary"
// now
// "where" in world space is that? your "crosshair" should be on the far left of the boundary
// so where does the spaceship have to turn to look "at" the crosshair in that position?
    // what it's really looking at is not the physical crosshair itself but what is underneath that crosshair
    // so when we are calculating a "look direction" for the ship we need to know what we consider the plane of intersection, maybe?
    // cause otherwise we dont know how "deep" to look?
// would this be a better driver of my spaceship movement...?

fn move_player(
    mut q_player: Query<
        (&ActionState<NetworkedInput>, &mut LinearVelocity, &mut Rotation),
        (Simulated, With<Player>),
    >,
    time: Res<Time<Fixed>>,
) {
    for (action_state, mut velocity, mut rotation) in q_player.iter_mut() {
        if let Some(movement) = action_state.dual_axis_data(&NetworkedInput::Aim) {
            // Get current orientation vectors
            let forward = (rotation.0 * -Vec3::Z).normalize();
            let up = (rotation.0 * Vec3::Y).normalize();
           
            // Calculate current pitch angle - CRITICAL FIX #1
            let forward_xz_raw = Vec3::new(forward.x, 0.0, forward.z);
            // Check if vector is too small to normalize
            let forward_xz = if forward_xz_raw.length_squared() > 1e-6 {
                forward_xz_raw.normalize()
            } else {
                // We're looking nearly straight up/down - use a fallback direction
                Vec3::new(0.0, 0.0, 1.0)
            };
            
            // Ensure dot product is in valid range for acos
            let dot = forward.dot(forward_xz).clamp(-1.0, 1.0);
            let current_pitch = dot.acos() * if forward.y < 0.0 { -1.0 } else { 1.0 };
           
            // Calculate new pitch amount, respecting limits
            let pitch_input = movement.pair.y * PITCH_RATE * time.delta_secs();
            let new_pitch = (current_pitch + pitch_input).clamp(-MAX_PITCH_ANGLE, MAX_PITCH_ANGLE);
            let pitch_change = new_pitch - current_pitch;
           
            // Apply yaw around world up
            let yaw_quat = Quat::from_rotation_y(-movement.pair.x * TURN_RATE * time.delta_secs());
            let forward_after_yaw = yaw_quat.mul_vec3(forward).normalize();
           
            // Calculate right vector - CRITICAL FIX #2
            let right_raw = forward_after_yaw.cross(Vec3::Y);
            let right = if right_raw.length_squared() > 1e-6 {
                right_raw.normalize()
            } else {
                // Forward is aligned with Y axis, use X as fallback
                Vec3::X
            };
           
            // Apply constrained pitch around right vector
            let pitch_quat = Quat::from_axis_angle(right, pitch_change);
            let final_forward = pitch_quat.mul_vec3(forward_after_yaw).normalize();
           
            // Calculate desired roll based on input (constrained)
            let roll_angle = movement.pair.x * MAX_ROLL_ANGLE;
           
            // Create a basis with the correct forward direction but no roll
            let no_roll_right_raw = final_forward.cross(Vec3::Y);
            let no_roll_right = if no_roll_right_raw.length_squared() > 1e-6 {
                no_roll_right_raw.normalize()
            } else {
                Vec3::X
            };
            
            let no_roll_up = no_roll_right.cross(final_forward).normalize();
           
            // Apply roll around forward axis
            let roll_quat = Quat::from_axis_angle(final_forward, roll_angle);
            let rolled_up = roll_quat.mul_vec3(no_roll_up).normalize();
           
            // Create final quaternion from orthonormal basis
            let final_right = final_forward.cross(rolled_up).normalize();
            let rot_matrix = Mat3::from_cols(final_right, rolled_up, -final_forward);
            rotation.0 = Quat::from_mat3(&rot_matrix).normalize(); // Normalize the result to be safe
        }
       
        // Always move forward in the direction the ship is facing
        let forward = (rotation.0 * -Vec3::Z).normalize();
        velocity.0 = forward * PLAYER_MOVE_SPEED;
    }
}
