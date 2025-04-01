use avian3d::{prelude::{NarrowPhaseConfig, PhysicsInterpolationPlugin}, PhysicsPlugins};
use bevy::prelude::*;
use lightyear::prelude::{
    client::{Interpolated, Predicted, VisualInterpolateStatus},
    server::ReplicationTarget,
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
