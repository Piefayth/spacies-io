use bevy::{color::palettes::tailwind::SLATE_800, prelude::*};
use lightyear::prelude::ClientConnectionManager;
use mygame_common::Simulated;
use mygame_protocol::message::{ClientRequestRespawn, UnorderedReliable};

use crate::{game_state::GameState, replication::LocalPlayer};

pub struct RespawnMenuPlugin;

impl Plugin for RespawnMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<RespawnMenuState>()
            .add_observer(set_respawn_menu_state_open)
            .add_observer(set_respawn_menu_state_closed)
            .add_systems(OnEnter(RespawnMenuState::Open), open_respawn_menu)
            .add_systems(OnEnter(RespawnMenuState::Closed), close_respawn_menu)
            .add_systems(OnExit(GameState::Playing), close_respawn_menu);
    }
}

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum RespawnMenuState {
    Open,
    #[default]
    Closed,
}

#[derive(Component)]
pub struct RespawnMenu;

fn set_respawn_menu_state_open(trigger: Trigger<OnRemove, LocalPlayer>, mut commands: Commands) {
    commands.set_state(RespawnMenuState::Open);
}

fn set_respawn_menu_state_closed(trigger: Trigger<OnAdd, LocalPlayer>, mut commands: Commands) {
    commands.set_state(RespawnMenuState::Closed);
}

fn open_respawn_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RespawnMenu,
        ))
        .with_children(|child_builder| {
            child_builder
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(SLATE_800.into()),
                ))
                .with_children(|child_child_builder| {
                    child_child_builder
                        .spawn((
                            Text::new("Respawn"),
                            TextFont {
                                font_size: 30.,
                                ..default()
                            },
                            Node {
                                padding: UiRect::bottom(Val::Px(20.)),
                                ..default()
                            },
                        ))
                        .observe(|_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.queue(|world: &mut World| {
                                if let Some(mut client) = world.get_resource_mut::<ClientConnectionManager>() {
                                    let _ = client.send_message::<UnorderedReliable, ClientRequestRespawn>(&ClientRequestRespawn);
                                }
                            });
                        });
                });
        });
}

fn close_respawn_menu(mut commands: Commands, q_respawn_menu: Query<Entity, With<RespawnMenu>>) {
    for respawn_menu in &q_respawn_menu {
        commands.entity(respawn_menu).despawn_recursive();
    }
}
