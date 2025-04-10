use avian3d::{prelude::{NarrowPhaseConfig, PhysicsInterpolationPlugin, PhysicsLayer}, sync::SyncConfig, PhysicsPlugins};
use bevy::prelude::*;
use lightyear::prelude::{
    client::{Interpolated, Predicted, VisualInterpolateStatus}, server::ReplicationTarget, PreSpawnedPlayerObject, ReplicationGroup
};
use mygame_assets::AssetPlugin;
use mygame_protocol::ProtocolPlugin;

pub mod level;
pub mod ship;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                AssetPlugin,
                ProtocolPlugin,
                PhysicsPlugins::new(FixedPostUpdate)
                    .build()
                    .disable::<PhysicsInterpolationPlugin>(),
                level::LevelPlugin,
                ship::ShipPlugin,
            ))
            .insert_resource(NarrowPhaseConfig {
                contact_tolerance: 0.1,
                ..default()
            })
            .insert_resource(SyncConfig {
                transform_to_position: false,
                position_to_transform: true,
                ..default()
            });
    }
}

pub type Simulated = Or<(With<Predicted>, With<ReplicationTarget>, With<PreSpawnedPlayerObject>)>;
pub type Rendered = Or<(Simulated, With<Interpolated>)>;

#[derive(PhysicsLayer, Default)]
pub enum CollisionMask {
    #[default]
    Nothing,
    Ship,
    Environment,
    Projectile
}

pub const REPLICATION_GROUP_PREDICTED: ReplicationGroup = ReplicationGroup::new_id(42);
pub const LEFT_PROJECTILE_ID: u64 = 23895723;
pub const RIGHT_PROJECTILE_ID: u64 = 105715186;
