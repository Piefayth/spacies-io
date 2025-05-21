use std::{
    hash::{Hash, Hasher},
    time::Duration,
};

use avian3d::prelude::{
    Collider, ColliderDisabled, CollisionLayers, CollisionMargin, CollisionStarted, Collisions,
    LinearVelocity, PhysicsSet, Position, RigidBody, RigidBodyDisabled, Rotation,
};
use bevy::{app::FixedMain, gltf::GltfMesh, prelude::*};
use leafwing_input_manager::{
    action_state::DualAxisData,
    axislike::DualAxisType,
    buttonlike::ButtonState,
    prelude::{ActionState, GamepadStick, InputMap, MouseMove, VirtualDPad},
};
use lightyear::{
    client::prediction::rollback::DisableRollback,
    prelude::{
        client::{
            is_in_rollback, Confirmed, Interpolated, Predicted, PredictionDespawnCommandsExt, Rollback
        }, server::{ControlledBy, Lifetime, SyncTarget}, ClientId, DisableReplicateHierarchy, NetworkIdentity, NetworkTarget, PreSpawned, ReplicateOnce, ServerConnectionManager, ServerReplicate, TickManager
    },
};
use mygame_assets::{CollisionMask, LevelState, assets::GlobalAssets};
use mygame_protocol::{
    component::{Bot, Health, Player, Projectile, Ship},
    input::NetworkedInput,
    message::{Reliable, ServerShipHit},
};

use crate::{
    LEFT_PROJECTILE_ID, REPLICATION_GROUP_PREDICTED, RIGHT_PROJECTILE_ID, Rendered, Simulated,
};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_rendered_ship_components, add_simulated_ship_components)
                .run_if(in_state(LevelState::Loaded))
                .after(RunFixedMainLoopSystem::AfterFixedMainLoop),
        );

        app.add_systems(FixedUpdate, (move_ship, fire, despawn_after_lifetime));

        app.add_systems(
            FixedPostUpdate,
            (handle_projectile_collisions, handle_ship_collisions)
                .after(PhysicsSet::StepSimulation),
        );

        app.add_systems(Last, add_simulated_projectile_components);
    }
}

fn handle_ship_collisions(
    mut commands: Commands,
    collisions: Collisions,
    mut q_ships: Query<(Entity, &Position, &mut Health), With<Ship>>,
    network_identity: NetworkIdentity,
    tick_manager: Res<TickManager>,
) {
    for contact_pair in collisions.iter() {
        let (ship_entity, other_entity) = if let Some(entity1) = contact_pair.body1 {
            if q_ships.contains(entity1) {
                if let Some(entity2) = contact_pair.body2 {
                    (entity1, entity2)
                } else {
                    continue; // No rigidbody in body2 (shouldn't have collider without RB anyway)
                }
            } else if let Some(entity2) = contact_pair.body2 {
                if q_ships.contains(entity2) {
                    (entity2, entity1)
                } else {
                    continue; // Neither entity is a ship
                }
            } else {
                continue; // body2 is None and body1 is not a ship
            }
        } else {
            continue; // No rigidbody in body1
        };
    }
}

#[derive(Event)]
pub struct ProjectileHitNonShip {
    pub position: Vec3,
}

fn handle_projectile_collisions(
    mut commands: Commands,
    collisions: Collisions,
    q_projectile: Query<(Entity, &Projectile, &Position, &Rotation, &LinearVelocity)>,
    mut q_ships: Query<(Entity, &Position, &mut Health), With<Ship>>,
    network_identity: NetworkIdentity,
    time: Res<Time<Fixed>>,
) {
    for contact_pair in collisions.iter() {
        // note that we check the "body entity" for other entity, because collider may not be on the parent
        let (
            projectile_entity,
            other_entity,
            projectile,
            projectile_position,
            projectile_rotation,
            projectile_velocity,
        ) = if let Ok((
            projectile_entity,
            projectile,
            projectile_position,
            projectile_rotation,
            projectile_velocity,
        )) = q_projectile.get(contact_pair.collider1)
        {
            // Check if body_entity2 exists before using it
            let Some(body_entity2) = contact_pair.body2 else {
                continue; // Skip this collision if there's no body entity
            };
            (
                projectile_entity,
                body_entity2,
                projectile,
                projectile_position,
                projectile_rotation,
                projectile_velocity,
            )
        } else if let Ok((
            projectile_entity,
            projectile,
            projectile_position,
            projectile_rotation,
            projectile_velocity,
        )) = q_projectile.get(contact_pair.collider2)
        {
            // Check if body_entity1 exists before using it
            let Some(body_entity1) = contact_pair.body1 else {
                continue; // Skip this collision if there's no body entity
            };
            (
                projectile_entity,
                body_entity1,
                projectile,
                projectile_position,
                projectile_rotation,
                projectile_velocity,
            )
        } else {
            continue;
        };

        // can't shoot yourself
        if projectile.owner == other_entity {
            continue;
        }

        if contact_pair.manifolds.is_empty() {
            continue;
        }

        let is_penetrating = contact_pair.manifolds.iter().any(|manifold| {
            manifold
                .points
                .iter()
                .any(|contact| contact.penetration > 0.0)
        });

        if !is_penetrating {
            continue;
        }

        // is "other_entity" a ship?
        if let Ok((ship, ship_position, mut ship_health)) = q_ships.get_mut(other_entity) {
            if network_identity.is_client() {
                commands
                    .entity(projectile_entity)
                    .insert((ColliderDisabled));
            } else {
                commands.entity(projectile_entity).despawn();

                // We want the despawn to happen EXACTLY once, even if two projectiles hit this frame
                if ship_health.current == 1 {
                    commands.entity(ship).despawn();
                } else {
                    ship_health.current -= 1;
                }

                let projectile_position = *projectile_position.clone();

                commands.queue(move |world: &mut World| {
                    let mut server = world.resource_mut::<ServerConnectionManager>();
                    let mut clients: Vec<ClientId> = vec![];

                    for client in server.connected_clients() {
                        clients.push(client);
                    }

                    for client in clients {
                        let _ = server.send_message::<Reliable, ServerShipHit>(
                            client,
                            &ServerShipHit {
                                position: projectile_position,
                            },
                        );
                    }
                })
            }
        } else {
            if network_identity.is_client() {
                // We don't use prediction despawn because it doesn't despawn the hierarchy
                // Notably this will prevent projectile rollbacks
                commands
                    .entity(projectile_entity)
                    .insert((ColliderDisabled, Visibility::Hidden));

                // Find first penetrating contact and use projectile's position/rotation to get world space
                let contact_position = if let Some(manifold) = contact_pair
                    .manifolds
                    .iter()
                    .find(|m| m.points.iter().any(|c| c.penetration > 0.0))
                {
                    // Get the first penetrating contact
                    if let Some(contact) = manifold.points.iter().find(|c| c.penetration > 0.0) {
                        // Move projectiles backward by their velocity so the VFX will spawn further back
                        if contact_pair.collider1 == projectile_entity {
                            contact.global_point1(projectile_position, projectile_rotation)
                                - projectile_velocity.0 * time.delta_secs() * 3.0
                        } else {
                            contact.global_point2(projectile_position, projectile_rotation)
                                - projectile_velocity.0 * time.delta_secs() * 3.0
                        }
                    } else {
                        projectile_position.0.clone()
                    }
                } else {
                    projectile_position.0.clone()
                };

                // This is an event because it triggers rendered effects
                // Which might not be available to common
                commands.trigger(ProjectileHitNonShip {
                    position: contact_position,
                });
            } else {
                // We disable the collider on the server instead of despawning the projectile so the server does not despawn it
                // before the client has the chance to process the collision and play the vfx
                // The alternative would be to let the server send the hit event, but this saves a lil bandwidth
                commands.entity(projectile_entity).insert((ColliderDisabled, Visibility::Hidden));
            }
        }
    }
}

fn add_rendered_ship_components(
    mut commands: Commands,
    q_rendered_ship: Query<Entity, (Rendered, Without<Children>, With<Ship>)>,
    global_assets: Res<GlobalAssets>,
) {
    if q_rendered_ship.is_empty() {
        return;
    }

    for ship_entity in &q_rendered_ship {
        commands
            .entity(ship_entity)
            .insert((
                // Collision is here instead of in add_simulated_ship_components in case we want to try interpolated ships
                SceneRoot(global_assets.character.clone()),
            ))
            .with_child((
                Collider::cuboid(2.0, 0.75, 2.0),
                //Collider::sphere(1.0),
                CollisionLayers::new(
                    CollisionMask::Ship,
                    [CollisionMask::Environment, CollisionMask::Projectile],
                ),
                Transform::from_translation(Vec3::Y * 0.25), // better alignment vertically
            ));
    }
}

fn add_simulated_ship_components(
    mut commands: Commands,
    q_simulated_ship: Query<Entity, (Simulated, Without<RigidBody>, With<Ship>)>,
    global_assets: Res<GlobalAssets>,
) {
    if q_simulated_ship.is_empty() {
        return;
    }

    for ship_entity in &q_simulated_ship {
        commands.entity(ship_entity).insert((
            RigidBody::Kinematic,
            ShipWeapon {
                cooldown_ticks: 20,
                last_fired_tick: 0,
            },
        ));
    }
}

// All rendered projectiles are simulated, actually! So we treat Rendered as Simulated here
fn add_simulated_projectile_components(
    mut commands: Commands,
    q_projectile: Query<Entity, (Rendered, Without<Collider>, With<Projectile>)>,
    global_assets: Res<GlobalAssets>,
) {
    for projectile_entity in &q_projectile {
        commands.entity(projectile_entity).insert((
            RigidBody::Kinematic,
            Collider::capsule_endpoints(0.5, Vec3::Z * -1., Vec3::Z * 1.),
            CollisionLayers::new(
                CollisionMask::Projectile,
                [CollisionMask::Environment, CollisionMask::Ship],
            ),
            SceneRoot(global_assets.laser.clone()),
            DisableRollback,
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
    pub is_server_controlled: bool,
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
    network_identity: NetworkIdentity,
) {
    let current_tick = tick_manager.tick();

    for (entity, despawn_after) in q_despawn.iter() {
        if despawn_after.should_despawn(*current_tick) {
            if network_identity.is_client() && despawn_after.is_server_controlled {
                //commands.entity(entity).insert(Visibility::Hidden);

                // Maybe just let the server handle despawning?
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

const PROJECTILE_VELOCITY: f32 = 200.;

#[derive(Component)]
pub struct ProjectileVelocity(pub Vec3);

fn fire(
    mut commands: Commands,
    mut q_ship: Query<
        (
            Entity,
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

    for (
        ship_entity,
        mut action_state,
        ship_position,
        ship_rotation,
        ship_velocity,
        mut ship_weapon,
        maybe_player,
        maybe_bot,
    ) in q_ship.iter_mut()
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

            if fire.pressed() && *tick > ship_weapon.last_fired_tick + ship_weapon.cooldown_ticks {
                ship_weapon.last_fired_tick = *tick;

                let offset_distance = 0.5;
                let ship_right = ship_rotation.0 * Vec3::X;
                let ship_up = ship_rotation.0 * Vec3::Y;
                let ship_forward = ship_rotation * -Vec3::Z;

                let left_offset = ship_position.0 - (ship_right * offset_distance)
                    + ship_forward * offset_distance;
                let right_offset = ship_position.0
                    + (ship_right * offset_distance)
                    + ship_forward * offset_distance;

                let projectile_velocity =
                    ship_velocity.0 + (ship_forward.normalize() * PROJECTILE_VELOCITY);

                let left_hash = compute_hash(LEFT_PROJECTILE_ID, shooter_id, &tick_manager);

                let right_hash = compute_hash(RIGHT_PROJECTILE_ID, shooter_id, &tick_manager);

                let left_projectile_base = (
                    Position(left_offset),
                    ship_rotation.clone(),
                    Projectile { owner: ship_entity },
                    LinearVelocity(projectile_velocity),
                    PreSpawned::new(left_hash),
                    DespawnAfter {
                        created_at_tick: *tick,
                        lifetime_ticks: 60,
                        is_server_controlled: false,
                    },
                );

                let right_projectile_base = (
                    Position(right_offset),
                    ship_rotation.clone(),
                    Projectile { owner: ship_entity },
                    LinearVelocity(projectile_velocity),
                    PreSpawned::new(right_hash),
                    DespawnAfter {
                        created_at_tick: *tick,
                        lifetime_ticks: 60,
                        is_server_controlled: false,
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
                        ..default()
                    },
                    DisableReplicateHierarchy,
                    ReplicateOnce::default()
                        .add::<Position>()
                        .add::<Rotation>()
                        .add::<LinearVelocity>(),
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

fn compute_hash(object_id: u64, client_id: u64, tick_manager: &TickManager) -> u64 {
    let mut hasher = seahash::SeaHasher::new();

    tick_manager.tick().hash(&mut hasher);
    object_id.hash(&mut hasher);
    client_id.hash(&mut hasher);

    hasher.finish()
}

const SHIP_MOVE_SPEED: f32 = 10.0;
const MAX_ROLL_ANGLE: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees
const MAX_PITCH_ANGLE: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees
const TURN_RATE: f32 = 1.0;
const PITCH_RATE: f32 = 1.0;
const MIN_HEIGHT: f32 = 5.0; // Minimum allowed height
const MAX_HEIGHT: f32 = 75.0; // Maximum allowed height
const ARENA_RADIUS: f32 = 100.0; // Maximum distance from origin in the XZ plane

fn move_ship(
    mut q_ship: Query<
        (
            &ActionState<NetworkedInput>,
            &mut LinearVelocity,
            &mut Rotation,
            &Transform, // Added position to track height
        ),
        (Simulated, With<Ship>),
    >,
    time: Res<Time<Fixed>>,
) {
    for (action_state, mut velocity, mut rotation, transform) in q_ship.iter_mut() {
        if let Some(movement) = action_state.dual_axis_data(&NetworkedInput::Aim) {
            // Get current orientation vectors
            let forward = (rotation.0 * -Vec3::Z).normalize();
            let up = (rotation.0 * Vec3::Y).normalize();

            // Calculate current pitch angle
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

            // Get current height and determine if we need to override pitch input
            let current_height = transform.translation.y;
            let pitch_input = movement.pair.y * PITCH_RATE * time.delta_secs();

            // Calculate new pitch amount, respecting limits
            let new_pitch = (current_pitch + pitch_input).clamp(-MAX_PITCH_ANGLE, MAX_PITCH_ANGLE);
            let pitch_change = new_pitch - current_pitch;

            // Apply yaw around world up
            let yaw_quat = Quat::from_rotation_y(-movement.pair.x * TURN_RATE * time.delta_secs());
            let forward_after_yaw = yaw_quat.mul_vec3(forward).normalize();

            // Calculate right vector
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
            rotation.0 = Quat::from_mat3(&rot_matrix);
        }

        // Always move forward in the direction the ship is facing
        let forward = (rotation.0 * -Vec3::Z).normalize();

        let mut adjusted_velocity = forward * SHIP_MOVE_SPEED;

        // Check height constraints
        let current_height = transform.translation.y;
        if (current_height <= MIN_HEIGHT && forward.y < 0.0)
            || (current_height >= MAX_HEIGHT && forward.y > 0.0)
        {
            // Remove vertical component of velocity when at height boundaries
            adjusted_velocity.y = 0.0;
        }

        // Check arena radius constraint
        let current_pos_xz = Vec2::new(transform.translation.x, transform.translation.z);
        let distance_from_origin = current_pos_xz.length();

        if distance_from_origin >= ARENA_RADIUS {
            // Calculate direction from origin to ship (normalized)
            let dir_from_origin = if distance_from_origin > 0.001 {
                current_pos_xz / distance_from_origin
            } else {
                Vec2::new(1.0, 0.0) // Default direction if at origin
            };

            // Project the forward direction onto the direction from origin
            let forward_xz = Vec2::new(forward.x, forward.z);
            let outward_component = forward_xz.dot(dir_from_origin);

            // If moving outward at the boundary, remove that component
            if outward_component > 0.0 {
                // Remove the outward component from velocity
                adjusted_velocity.x -= dir_from_origin.x * outward_component * SHIP_MOVE_SPEED;
                adjusted_velocity.z -= dir_from_origin.y * outward_component * SHIP_MOVE_SPEED;
            }
        }

        velocity.0 = adjusted_velocity;
    }
}
