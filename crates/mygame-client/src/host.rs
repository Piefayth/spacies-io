use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use lightyear::prelude::client::{Authentication, NetConfig};
use lightyear::prelude::server::ServerTransport;
use lightyear::prelude::*;
use lightyear::server::config::ServerConfig;
use lightyear::{client::config::ClientConfig, prelude::client::ClientConnection};
use mygame_common::LaunchConfigurations;
use std::{
    net::{Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

use crate::app::{AssetPath};
use crate::game_state::GameState;
use mygame_server::app::{ServerMode, build_server_app};

pub (crate) struct HostPlugin;

impl Plugin for HostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Hosting), on_client_begin_hosting);
    }
}

struct SendApp(App);

unsafe impl Send for SendApp {}
impl SendApp {
    fn run(&mut self) {
        self.0.run();
    }
}

fn on_client_begin_hosting(
    mut commands: Commands,
    launch_configurations: ResMut<LaunchConfigurations>,
    asset_path: Res<AssetPath>,
    client: Res<ClientConnection>,
) {
    let client_id = match launch_configurations
        .client_local_config
        .clone()
        .expect("There must be a server config if we are in host mode.")
        .net
    {
        NetConfig::Netcode { auth, config, io } => match auth {
            Authentication::Token(connect_token) => {
                panic!("ClientHost Authentication should not require a connect token.")
            }
            Authentication::Manual {
                server_addr,
                client_id,
                private_key,
                protocol_id,
            } => client_id,
            Authentication::None => panic!("Authentication is required."),
        },
        NetConfig::Local { id } => panic!("Only networked configurations are supported."),
    };

    {
        let server_app = build_server_app(
            launch_configurations
                .server_config
                .clone()
                .expect("There must be a server config if we are in host mode."),
            asset_path.0.clone(),
            ServerMode::ClientHost(ClientId::Netcode(client_id)),
        );

        let mut send_server_app = SendApp(server_app);
        std::thread::spawn(move || send_server_app.run());
    }

    commands.set_state(GameState::ConnectingSelf);
}
