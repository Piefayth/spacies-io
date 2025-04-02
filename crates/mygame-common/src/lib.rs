use avian3d::{prelude::{NarrowPhaseConfig, PhysicsInterpolationPlugin}, PhysicsPlugins};
use bevy::prelude::*;
use lightyear::prelude::{
    client::{Interpolated, Predicted, VisualInterpolateStatus},
    server::ReplicationTarget, ReplicationGroup,
};
use mygame_assets::AssetPlugin;
use mygame_protocol::ProtocolPlugin;

pub mod level;
pub mod player;

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
                player::PlayerPlugin,
            ))
            .insert_resource(NarrowPhaseConfig {
                contact_tolerance: 0.1,
                ..default()
            });
    }
}

pub type Simulated = Or<(With<Predicted>, With<ReplicationTarget>)>;
pub type Rendered = Or<(Simulated, With<Interpolated>)>;


pub const REPLICATION_GROUP_PREDICTED: ReplicationGroup = ReplicationGroup::new_id(42);
pub const PRE_SPAWNED_PROJECTILE: u64 = 23895723;
