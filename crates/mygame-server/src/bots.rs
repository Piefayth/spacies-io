use avian3d::{math::TAU, prelude::{Position, Rotation}};
use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_rand::{global::GlobalEntropy, prelude::{Entropy, WyRand}, traits::ForkableRng};
use lightyear::prelude::{server::{ControlledBy, Lifetime, SyncTarget}, NetworkTarget, ReplicateHierarchy, ServerReplicate, TickManager};
use mygame_common::REPLICATION_GROUP_PREDICTED;
use mygame_protocol::component::{Bot, Ship};
use rand_core::RngCore;

pub struct BotsPlugin;

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, spawn_bots);
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
const MAX_BOTS: usize = 555;
const SPAWN_RADIUS: f32 = 100.0;
const CEILING_HEIGHT: f32 = 100.0;
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
            hierarchy: ReplicateHierarchy {
                enabled: false,
                ..default()
            },
            ..default()
        },
    ));
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
