use crate::game_state::GameState;
use bevy::prelude::*;
use lightyear::prelude::{
    client::{ClientCommandsExt, ClientConnection, NetClient},
    *,
};
use mygame_assets::{CurrentLevel, LevelState};
use mygame_common::Rendered;
use mygame_protocol::{
    component::Player,
    message::{ClientRequestRespawn, ServerWelcome, UnorderedReliable},
};
use mygame_render::camera::CameraTarget;

pub (crate) struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_server_welcome.run_if(in_state(GameState::ConnectingRemote)),
                #[cfg(feature = "host")]
                on_server_welcome.run_if(in_state(GameState::ConnectingSelf)),
            ),
        );
        app.add_systems(Update, await_spawn);
        app.add_systems(OnEnter(LevelState::Loaded), on_assets_loaded);
    }
}

/// Tag component to identify the local player
#[derive(Component)]
pub struct LocalPlayer;

/// Once finished loading the assets that the server requested the client to load
/// Signal the completion to the server
fn on_assets_loaded(mut commands: Commands, mut client: ResMut<ClientConnectionManager>) {
    commands.set_state(GameState::Spawning);

    if let Err(e) =
        client.send_message::<UnorderedReliable, ClientRequestRespawn>(&ClientRequestRespawn)
    {
        println!("unable to signal client level load complete due to {}", e);
        commands.disconnect_client();
    }
}

/// Respond to the welcome message from the server by initiating a load of the level requested
fn on_server_welcome(
    mut server_welcome_events: ResMut<Events<ClientReceiveMessage<ServerWelcome>>>,
    game_state: Res<State<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ev in server_welcome_events.drain() {
        next_state.set(GameState::Loading);
        current_level.0 = ev.message.current_level;
    }
}

fn await_spawn(
    mut commands: Commands,
    q_spawned_player: Query<(Entity, &Player), (Rendered, Added<Player>)>,
    q_local_player: Query<&LocalPlayer>, 
    client: Res<ClientConnection>,
) {
    for (entity, player) in &q_spawned_player {
        if !q_local_player.is_empty() {
            warn!("Server spawned a local player when one already existed? Ignoring.");
            return;
        }

        if player.0 == client.id() {
            commands.entity(entity)
                .insert((
                    LocalPlayer,
                    CameraTarget {
                        follow_distance: 6.0,
                        smooth_time: 0.15,
                    }
                ));
            commands.set_state(GameState::Playing);
        }
    }
}
