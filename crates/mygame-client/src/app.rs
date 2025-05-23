use bevy::asset::AssetMetaCheck;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use lightyear::prelude::client::{NetConfig, VisualInterpolationPlugin};
use lightyear::{
    client::{config::ClientConfig, plugin::ClientPlugins},
    server::config::ServerConfig,
};
use mygame_common::CommonPlugin;
use mygame_render::RenderPlugin;

use crate::crosshair::CrosshairPlugin;
use crate::game_state::{GameLifecyclePlugin, GameState};
use crate::input::InputPlugin;
use crate::throwaway::ThrowawayPlugin;
use crate::{
    interpolation::InterpolationPlugin, network::NetworkPlugin, replication::ReplicationPlugin,
    ui::UiPlugin,
};
use mygame_common::LaunchConfigurations;

#[cfg(feature = "host")]
use crate::host::HostPlugin;
#[cfg(feature = "host")]
use lightyear::prelude::client::IoConfig;

/// The root asset path is preserved here by the client at startup so it can be forwarded
/// to the client server, should they choose to host.
#[derive(Resource)]
pub struct AssetPath(pub String);

fn build_core_client_app(
    app: &mut App,
    client_remote_config: ClientConfig,
    asset_path: String,
) -> &mut App {
    app.add_plugins((
        DefaultPlugins.build().set(AssetPlugin {
            file_path: asset_path.clone(),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(LogPlugin {
            level: Level::INFO,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=info,offset_allocator=error,naga=warn,bevy_hanabi=error".into(),
            ..default()
        }),
        ClientPlugins {
            config: client_remote_config.clone(),
        },
        RenderPlugin,
        CommonPlugin,
        GameLifecyclePlugin,
        UiPlugin,
        NetworkPlugin,
        ReplicationPlugin,
        InputPlugin,
        InterpolationPlugin,
        CrosshairPlugin,
        ThrowawayPlugin
    ));

    app.insert_resource(AssetPath(asset_path));
    app.enable_state_scoped_entities::<GameState>();
    
    app
}

#[cfg(not(feature = "host"))]
pub fn build_client_app(client_config: ClientConfig, asset_path: String) -> App {
    let mut app = App::new();

    build_core_client_app(&mut app, client_config.clone(), asset_path);

    app.insert_resource(LaunchConfigurations {
        server_config: None,
        client_local_config: None,
        client_remote_config: Some(client_config),
    });

    app
}

/// The "host" feature has its own signature for build_client_app so it may
/// obtain a ServerConfig to configure the server with.
#[cfg(feature = "host")]
pub fn build_client_app(
    client_remote_config: ClientConfig,
    client_local_config: ClientConfig,
    asset_path: String,
    server_config: ServerConfig,
) -> App {
    let mut app = App::new();

    build_core_client_app(&mut app, client_remote_config.clone(), asset_path.clone());

    app.add_plugins((HostPlugin,));

    app.insert_resource(LaunchConfigurations {
        server_config: Some(server_config),
        client_local_config: Some(client_local_config),
        client_remote_config: Some(client_remote_config),
    });

    app
}
