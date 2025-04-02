use bevy::prelude::*;
use lightyear::{
    client::config::ClientConfig,
    prelude::{ClientConnectEvent, ClientDisconnectEvent, client::ClientCommandsExt},
};

use crate::app::LaunchConfigurations;
use crate::game_state::GameState;

pub (crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::ConnectingRemote),
            connect_to_remote_server,
        );
        #[cfg(feature = "host")]
        app.add_systems(OnEnter(GameState::ConnectingSelf), connect_to_local_server);

        app.add_observer(on_client_connect_success)
            .add_observer(on_client_disconnect);
    }
}

fn connect_to_remote_server(
    mut commands: Commands,
    host_config: ResMut<LaunchConfigurations>,
    mut client_config: ResMut<ClientConfig>,
) {
    *client_config = host_config
        .client_remote_config
        .clone()
        .expect("There must be a remote client config we are a client.");
    commands.connect_client();
}

#[cfg(feature = "host")]
fn connect_to_local_server(
    mut commands: Commands,
    host_config: ResMut<LaunchConfigurations>,
    mut client_config: ResMut<ClientConfig>,
) {
    *client_config = host_config
        .client_local_config
        .clone()
        .expect("There must be a local client config if we are in host mode.");
    commands.connect_client();
}

fn on_client_connect_success(_trigger: Trigger<ClientConnectEvent>) {
    // No need to do anything, we are waiting for a ServerWelcome message
    info!("successful client connection");
}

fn on_client_disconnect(
    _trigger: Trigger<ClientDisconnectEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    // TODO: Cleanup existing state?
    game_state.set(GameState::MainMenu);
}
