use std::{
    hash::{Hash, Hasher},
    time::Duration,
};

use avian3d::prelude::{
    collider, Collider, CollisionMargin, LinearVelocity, PhysicsSet, Position, RigidBody, RigidBodyDisabled, Rotation
};
use bevy::{app::FixedMain, gltf::GltfMesh, prelude::*};
use leafwing_input_manager::{
    axislike::DualAxisType,
    buttonlike::ButtonState,
    prelude::{ActionState, GamepadStick, InputMap, MouseMove, VirtualDPad},
};
use lightyear::{
    client::prediction::rollback::DisableRollback,
    prelude::{
        client::{is_in_rollback, Confirmed, Interpolated, Predicted, PredictionDespawnCommandsExt, Rollback}, server::{ControlledBy, Lifetime, ReplicationTarget, SyncTarget}, NetworkIdentity, NetworkTarget, PreSpawnedPlayerObject, ReplicateHierarchy, ReplicateOnceComponent, ServerReplicate, TickManager
    },
};
use mygame_assets::{LevelState, assets::GlobalAssets};
use mygame_protocol::{
    component::{Bot, Player, Projectile, Ship},
    input::NetworkedInput,
};

use crate::{
    LEFT_PROJECTILE_ID, REPLICATION_GROUP_PREDICTED, RIGHT_PROJECTILE_ID,
    Rendered, Simulated,
};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_ship_gameplay_components,
            )
                .run_if(in_state(LevelState::Loaded))
                .after(RunFixedMainLoopSystem::AfterFixedMainLoop),
        );

        app.add_systems(FixedUpdate, (/*move_ship,*/ (fire, add_projectile_gameplay_components, move_projectiles).chain(), despawn_after_lifetime));
    }
}

fn add_ship_gameplay_components(
    mut commands: Commands,
    q_rendered_ship: Query<Entity, (Rendered, Without<RigidBody>, With<Ship>)>,
    global_assets: Res<GlobalAssets>,
) {
    if q_rendered_ship.is_empty() {
        return;
    }

    for ship_entity in &q_rendered_ship {
        commands.entity(ship_entity).insert((
            RigidBody::Kinematic,
            Collider::sphere(1.0),
            CollisionMargin(0.1),
            ActionState::<NetworkedInput>::default(),
            SceneRoot(global_assets.character.clone()),
            ShipWeapon {
                cooldown_ticks: 7,
                last_fired_tick: 0,
            },
        ));
    }
}

fn add_projectile_gameplay_components(
    mut commands: Commands,
    q_projectile: Query<Entity, (Rendered, Without<Collider>, With<Projectile>)>,
    global_assets: Res<GlobalAssets>,
) {
    for projectile_entity in &q_projectile {
        commands.entity(projectile_entity).insert((
            RigidBody::Kinematic,
            Collider::cylinder(1.0, 2.0),
            SceneRoot(global_assets.laser.clone()),
            //DisableRollback,
        ));
    }
}

#[derive(Component)]
pub struct ShipWeapon {
    pub cooldown_ticks: u16,
    pub last_fired_tick: u16,
}

#[derive(Component)]
pub struct DespawnAfter {
    pub created_at_tick: u16,
    pub lifetime_ticks: u16,
}

impl DespawnAfter {
    pub fn should_despawn(&self, current_tick: u16) -> bool {
        let despawn_at_tick = self.created_at_tick.wrapping_add(self.lifetime_ticks);
        
        if self.created_at_tick <= despawn_at_tick {
            current_tick >= despawn_at_tick
        } else {
            current_tick >= despawn_at_tick || current_tick < self.created_at_tick
        }
    }
}

pub fn despawn_after_lifetime(
    mut commands: Commands,
    tick_manager: Res<TickManager>,
    q_despawn: Query<(Entity, &DespawnAfter)>,
    network_identity: NetworkIdentity
) {
    let current_tick = tick_manager.tick();
    
    for (entity, despawn_after) in q_despawn.iter() {
        if despawn_after.should_despawn(*current_tick) {
            if network_identity.is_client() {
                commands.entity(entity).insert(Visibility::Hidden); // instead, "Disabled" to prevent collision?
            } else {
                commands.entity(entity).despawn();
            }
            
        }
    }
}

const PROJECTILE_VELOCITY: f32 = 50.;

#[derive(Component)]
pub struct ProjectileVelocity(pub Vec3);

fn fire(
    mut commands: Commands,
    mut q_ship: Query<
        (
            &mut ActionState<NetworkedInput>,
            &Position,
            &Rotation,
            &LinearVelocity,
            &mut ShipWeapon,
            Option<&Player>,
            Option<&Bot>,
        ),
        (Simulated, With<Ship>),
    >,
    network_identity: NetworkIdentity,
    tick_manager: Res<TickManager>,
    rollback_manager: Option<Res<Rollback>>,
) {
    let tick = tick_manager.tick();
    let rollback = if let Some(rollback_manager) = rollback_manager {
        rollback_manager.is_rollback()
    } else {
        false
    };

    for (mut action_state, ship_position, ship_rotation, ship_velocity, mut ship_weapon, maybe_player, maybe_bot) in
        q_ship.iter_mut()
    {
        if let Some(fire) = action_state.button_data_mut(&NetworkedInput::Fire) {
            let shooter_id = if let Some(player) = maybe_player {
                player.0.to_bits()
            } else if let Some(bot) = maybe_bot {
                bot.0
            } else {
                warn!("Simulated ship exists that is neither a bot nor a player?");
                0
            };


            if fire.pressed()
                && *tick > ship_weapon.last_fired_tick + ship_weapon.cooldown_ticks
            {
                ship_weapon.last_fired_tick = *tick;
                
                let offset_distance = 0.5;
                let ship_right = ship_rotation.0 * Vec3::X;
                let ship_up = ship_rotation.0 * Vec3::Y;
                let ship_forward = ship_rotation * -Vec3::Z;
                
                let left_offset = ship_position.0 - (ship_right * offset_distance) + ship_forward * offset_distance;
                let right_offset = ship_position.0 + (ship_right * offset_distance) + ship_forward * offset_distance;
                
                let projectile_velocity =
                    ship_velocity.0 + (ship_forward.normalize() * PROJECTILE_VELOCITY);

                let left_hash = compute_hash(LEFT_PROJECTILE_ID, shooter_id, &tick_manager);

                let right_hash =
                    compute_hash(RIGHT_PROJECTILE_ID, shooter_id, &tick_manager);

                let left_projectile_base = (
                    Position(left_offset),
                    ship_rotation.clone(),
                    Projectile,
                    LinearVelocity(projectile_velocity),
                    PreSpawnedPlayerObject::new(left_hash),
                    DespawnAfter {
                        created_at_tick: *tick,
                        lifetime_ticks: 60,
                    },
                );

                let right_projectile_base = (
                    Position(right_offset),
                    ship_rotation.clone(),
                    Projectile,
                    LinearVelocity(projectile_velocity),
                    PreSpawnedPlayerObject::new(right_hash),
                    DespawnAfter {
                        created_at_tick: *tick,
                        lifetime_ticks: 60,
                    },
                );

                let server_components = (
                    ServerReplicate {
                        group: REPLICATION_GROUP_PREDICTED,
                        controlled_by: ControlledBy {
                            target: match maybe_player {
                                Some(player) => NetworkTarget::Single(player.0),
                                None => NetworkTarget::None,
                            },
                            lifetime: Lifetime::SessionBased,
                        },
                        sync: SyncTarget {
                            prediction: NetworkTarget::All,
                            interpolation: NetworkTarget::None,
                        },
                        hierarchy: ReplicateHierarchy {
                            enabled: false,
                            ..default()
                        },
                        ..default()
                    },
                    // ReplicateOnceComponent::<Position>::default(),
                    // ReplicateOnceComponent::<Rotation>::default(),
                );

                commands
                    .spawn(left_projectile_base)
                    .insert_if(server_components.clone(), || network_identity.is_server());

                commands
                    .spawn(right_projectile_base)
                    .insert_if(server_components, || network_identity.is_server());
            }
        }
    }
}

fn move_projectiles(
    mut q_projectile: Query<(&mut Position, &LinearVelocity, Option<&PreSpawnedPlayerObject>), (With<Projectile>, Or<(With<Predicted>, With<PreSpawnedPlayerObject>)>)>,
    time: Res<Time<Fixed>>,
    tick_manager: Res<TickManager>,
    network_identity: NetworkIdentity,
    maybe_rollback: Option<Res<Rollback>>
) {
    let (tick, is_rollback) = match maybe_rollback {
        Some(rb) => {
            (tick_manager.tick_or_rollback_tick(rb.as_ref()), rb.is_rollback())
        },
        None => (tick_manager.tick(), false),
    };

    for (mut position, velocity, maybe_prespawn) in q_projectile.iter_mut() {
        //position.0 += velocity.0 * time.delta_secs();

        if network_identity.is_client() {
            if is_rollback {
                println!("      Rollback Tick ({}) projectile pos: {}, is prespawn: {}", tick.0, position.0, maybe_prespawn.is_some());
            } else {
                println!("Tick ({}) projectile pos: {}, is prespawn: {}", tick.0, position.0, maybe_prespawn.is_some());
            }
        }
        
    }
}

fn compute_hash(object_id: u64, client_id: u64, tick_manager: &TickManager) -> u64 {
    let mut hasher = seahash::SeaHasher::new();

    tick_manager.tick().hash(&mut hasher);
    object_id.hash(&mut hasher);
    client_id.hash(&mut hasher);

    hasher.finish()
}

const PLAYER_MOVE_SPEED: f32 = 10.0;
const MAX_ROLL_ANGLE: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees
const MAX_PITCH_ANGLE: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees
const TURN_RATE: f32 = 1.0;
const PITCH_RATE: f32 = 1.0;

fn move_ship(
    mut q_player: Query<
        (
            &ActionState<NetworkedInput>,
            &mut LinearVelocity,
            &mut Rotation,
        ),
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
