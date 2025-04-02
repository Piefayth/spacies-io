use avian3d::prelude::{LinearVelocity, Position};
use bevy::{math::VectorSpace, prelude::*};
use lightyear::prelude::client::InterpolationSet;

pub (crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn(Camera3d::default())
                .insert(Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y))
                .insert(MainCamera)
                .insert(CameraVelocity::default());
        })
        .add_systems(PostUpdate, follow_camera_target.after(InterpolationSet::VisualInterpolation).before(TransformSystem::TransformPropagate));
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Default)]
pub struct CameraTarget {
    pub follow_distance: f32, 
    pub smooth_time: f32,
}

// Storage for the smooth damp velocity
#[derive(Component, Default)]
pub struct CameraVelocity {
    pub position_velocity: Vec3,
    pub rotation_velocity: Quat,
}

// SmoothDamp implementation for Vec3
fn smooth_damp_vec3(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    smooth_time: f32,
    delta_time: f32,
) -> Vec3 {
    // If smooth time is very small, just return target
    if smooth_time < 0.0001 {
        *velocity = Vec3::ZERO;
        return target;
    }

    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    
    let change = current - target;
    let temp = (*velocity + omega * change) * delta_time;
    *velocity = (*velocity - omega * temp) * exp;
    
    let output = target + (change + temp) * exp;
    
    // Prevent overshooting
    let original_minus_target = current - target;
    let output_minus_target = output - target;
    
    if original_minus_target.dot(output_minus_target) > 0.0 {
        return output;
    } else {
        *velocity = Vec3::ZERO;
        return target;
    }
}

// SmoothDamp implementation for quaternion rotations
fn smooth_damp_quat(
    current: Quat,
    target: Quat,
    velocity: &mut Quat,
    smooth_time: f32,
    delta_time: f32,
) -> Quat {
    if smooth_time < 0.0001 {
        *velocity = Quat::IDENTITY;
        return target;
    }
    
    let dot = current.dot(target);
    let adjusted_target = if dot < 0.0 {
        -target
    } else {
        target
    };
    
    let t = (delta_time / smooth_time).clamp(0.0, 1.0);
    current.slerp(adjusted_target, t).normalize()
}


fn follow_camera_target(
    time: Res<Time>,
    q_camera_targets: Query<(&Transform, &CameraTarget, Option<&LinearVelocity>)>,
    mut q_cameras: Query<(&mut Transform, &mut CameraVelocity), (With<MainCamera>, Without<CameraTarget>)>,
) {
    let Some((target_transform, target, velocity_opt)) = q_camera_targets.iter().next() else {
        return;
    };
    
    let Ok((mut camera_transform, mut camera_velocity)) = q_cameras.get_single_mut() else {
        return;
    };
    
    // Get the basic desired position (like your original code)
    let desired_position = target_transform.translation +
        -target_transform.forward() * target.follow_distance +
        Vec3::Y * (target.follow_distance * 0.3);  // Add some height
    
    // Apply smooth damping to camera position instead of direct assignment
    camera_transform.translation = smooth_damp_vec3(
        camera_transform.translation,
        desired_position,
        &mut camera_velocity.position_velocity,
        target.smooth_time,
        time.delta_secs()
    );
    
    // Create desired rotation (looking at the ship)
    let desired_rotation = Transform::from_translation(camera_transform.translation)
        .looking_at(target_transform.translation, Vec3::Y)
        .rotation;
    
    // Apply smooth damping to camera rotation
    camera_transform.rotation = smooth_damp_quat(
        camera_transform.rotation,
        desired_rotation,
        &mut camera_velocity.rotation_velocity,
        target.smooth_time * 0.5, // Make rotation smoother than position
        time.delta_secs()
    );
}
