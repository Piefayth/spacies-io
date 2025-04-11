use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Copy, Default)]
pub enum Level {
    #[default]
    Void, // No level is loaded
    Example,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerWelcome {
    pub current_level: Level,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientLevelLoadComplete;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientHostRequestShutdown;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerShipHit {
    pub position: Vec3,
}

#[derive(Channel)]
pub struct UnorderedReliable;

#[derive(Channel)]
pub struct Reliable;

pub fn register_messages(app: &mut App) {
    app.register_message::<ServerWelcome>(ChannelDirection::ServerToClient);
    app.register_message::<ServerShipHit>(ChannelDirection::ServerToClient);

    app.register_message::<ClientLevelLoadComplete>(ChannelDirection::ClientToServer);
    app.register_message::<ClientHostRequestShutdown>(ChannelDirection::ClientToServer);

    app.add_channel::<UnorderedReliable>(ChannelSettings {
        mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
        ..default()
    });

    app.add_channel::<Reliable>(ChannelSettings {
        mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
        ..default()
    });
}
