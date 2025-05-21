use avian3d::{prelude::{NarrowPhaseConfig, PhysicsInterpolationPlugin, PhysicsLayer}, sync::SyncConfig, PhysicsPlugins};
use bevy::prelude::*;
use lightyear::{client::config::ClientConfig, prelude::{
    client::{Confirmed, Interpolated, Predicted, VisualInterpolateStatus}, server::ReplicateToClient, PreSpawned, ReplicationGroup
}, server::config::ServerConfig};
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

#[derive(Resource)]
pub struct LaunchConfigurations {
    pub server_config: Option<ServerConfig>,
    pub client_local_config: Option<ClientConfig>,
    pub client_remote_config: Option<ClientConfig>,
}

pub type Simulated = Or<(With<Predicted>, With<ReplicateToClient>, With<PreSpawned>)>;
pub type Rendered = Or<(Simulated, With<Interpolated>)>;

pub const REPLICATION_GROUP_PREDICTED: ReplicationGroup = ReplicationGroup::new_id(42);
pub const LEFT_PROJECTILE_ID: u64 = 23895723;
pub const RIGHT_PROJECTILE_ID: u64 = 105715186;
