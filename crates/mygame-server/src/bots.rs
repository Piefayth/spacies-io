use avian3d::{math::TAU, prelude::{Position, Rotation}};
use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use bevy_rand::{global::GlobalEntropy, prelude::{Entropy, WyRand}, traits::ForkableRng};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{server::{ControlledBy, Lifetime, SyncTarget}, DisableReplicateHierarchy, NetworkTarget, ServerReplicate, TickManager};
use mygame_common::REPLICATION_GROUP_PREDICTED;
use mygame_protocol::{component::{Bot, Health, Ship}, input::NetworkedInput};
use rand_core::RngCore;

pub struct BotsPlugin;

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (spawn_bots, control_bots));
    }
}

#[derive(Component)]
struct BotAI {
    target_location: Vec3,
    target_ship: Option<Entity>,
    chase_end_tick: u16,
    entropy: Entropy<WyRand>
}

const BOT_SPAWN_TICK_INTERVAL: u16 = 7;
const MAX_BOTS: usize = 0;
const SPAWN_RADIUS: f32 = 100.0;
const CEILING_HEIGHT: f32 = 50.0;
const TARGET_REACH_DISTANCE: f32 = 2.0;
const MAX_CHASE_TICKS: u32 = 300;

fn spawn_bots(
    mut commands: Commands,
    tick_manager: Res<TickManager>,
    q_bots: Query<Entity, With<Bot>>,
    mut global_rng: GlobalEntropy<WyRand>,
) {
    if *tick_manager.tick() % BOT_SPAWN_TICK_INTERVAL != 0 {
        return;
    }
    let bot_count = q_bots.iter().count();
    if bot_count >= MAX_BOTS {
        return;
    }
    
    let spawn_position = random_position_in_area(&mut global_rng, SPAWN_RADIUS, CEILING_HEIGHT);
    let initial_target = random_position_in_area(&mut global_rng, SPAWN_RADIUS, CEILING_HEIGHT);
    
    commands.spawn((
        Ship,
        Health {
            current: 6,
            max: 6
        },
        Bot(global_rng.next_u64()),
        BotAI {
            target_location: initial_target,
            chase_end_tick: 0,
            entropy: global_rng.fork_rng(),
            target_ship: None,
        },
        Position(spawn_position),
        Rotation::default(),
        ServerReplicate {
            group: REPLICATION_GROUP_PREDICTED,
            controlled_by: ControlledBy {
                target: NetworkTarget::None,
                ..default()
            },
            sync: SyncTarget {
                prediction: NetworkTarget::All,
                interpolation: NetworkTarget::None,
            },
            // sync: SyncTarget {
            //     prediction: NetworkTarget::None,
            //     interpolation: NetworkTarget::All,
            // },
            ..default()
        },
        DisableReplicateHierarchy,
        ActionState::<NetworkedInput>::default(),
    ));
}

fn control_bots(
    mut query: Query<(&BotAI, &Position, &Rotation, &mut ActionState<NetworkedInput>, Entity)>,
    mut previous_aims: Local<HashMap<Entity, Vec2>>,
) {
    // Define lerp factor (0.0 = no change, 1.0 = immediate change)
    const LERP_FACTOR: f32 = 0.1; // Adjust for desired smoothness
    
    for (bot_ai, position, rotation, mut action_state, entity) in query.iter_mut() {
        let to_target = bot_ai.target_location - position.0;
        
        // Calculate raw aim values
        let forward = *rotation * Vec3::Z;
        let right = *rotation * Vec3::X;
        let up = *rotation * Vec3::Y;
        
        // Project the to_target vector onto our local coordinate system
        let projected_right = to_target.dot(right);
        let projected_forward = to_target.dot(forward);
        let projected_up = to_target.dot(up);
        
        // Create normalized 2D aim vector (X = horizontal, Y = vertical)
        let horizontal_angle = projected_right.atan2(projected_forward);
        let distance_xz = (projected_right * projected_right + projected_forward * projected_forward).sqrt();
        let vertical_angle = projected_up.atan2(distance_xz);
        
        // Normalize to -1.0 to 1.0 range
        // Horizontal: left/right maps to -1.0/1.0
        // Vertical: down/up maps to -1.0/1.0
        let raw_aim = Vec2::new(
            (horizontal_angle / std::f32::consts::PI).clamp(-1.0, 1.0),
            (vertical_angle / (std::f32::consts::PI / 2.0)).clamp(-1.0, 1.0)
        );
        
        // Get previous aim or use current as fallback
        let previous_aim = previous_aims.get(&entity).copied().unwrap_or(raw_aim);
        
        // Smoothly interpolate between previous and current aim (lerp)
        let final_aim = Vec2::new(
            previous_aim.x * (1.0 - LERP_FACTOR) + raw_aim.x * LERP_FACTOR,
            previous_aim.y * (1.0 - LERP_FACTOR) + raw_aim.y * LERP_FACTOR
        );
        
        // Store current aim for next tick
        previous_aims.insert(entity, final_aim);
        
        // Apply the aim
        action_state.set_axis_pair(&NetworkedInput::Aim, final_aim);
    }
}




fn random_position_in_area(rng: &mut GlobalEntropy<WyRand>, radius: f32, height: f32) -> Vec3 {
    let sqrt_random = (rng.next_u32() as f32 / u32::MAX as f32).sqrt();
    let angle = (rng.next_u32() as f32 / u32::MAX as f32) * TAU;
    let distance = sqrt_random * radius;
    
    let x = distance * angle.cos();
    let z = distance * angle.sin();
    let y = (rng.next_u32() as f32 / u32::MAX as f32) * height;
   
    Vec3::new(x, y, z)
}
