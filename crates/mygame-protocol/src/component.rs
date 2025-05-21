use avian3d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use leafwing_input_manager::prelude::ActionState;
use lightyear::{
    prelude::{
        client::{ComponentSyncMode, LerpFn},
        *,
    },
    utils::bevy::TransformLinearInterpolation,
};

use crate::input::NetworkedInput;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player(pub ClientId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Bot(pub u64);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Projectile {
    pub owner: Entity
}

impl MapEntities for Projectile {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.owner = entity_mapper.get_mapped(self.owner);
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ship;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: u16,
    pub max: u16
}

pub fn register_components(app: &mut App) {
    app.register_component::<Player>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Once)
        .add_interpolation(ComponentSyncMode::Once);

    app.register_component::<Bot>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Once)
        .add_interpolation(ComponentSyncMode::Once);
    
    app.register_component::<Ship>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Once)
        .add_interpolation(ComponentSyncMode::Once);

    app.register_component::<Projectile>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Once)
        .add_interpolation(ComponentSyncMode::Once)
        .add_map_entities();

    app.register_component::<LinearVelocity>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Full);

    app.register_component::<Health>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Simple);

    app.register_component::<Position>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Full)
        .add_interpolation(ComponentSyncMode::Full)
        .add_interpolation_fn(|start, end, t| Position(start.lerp(**end, t)))
        .add_correction_fn(|start, end, t| Position(start.lerp(**end, t)));

    app.register_component::<Rotation>(ChannelDirection::ServerToClient)
        .add_prediction(ComponentSyncMode::Full)
        .add_interpolation(ComponentSyncMode::Full)
        .add_interpolation_fn(|start, end, t| Rotation(*start.slerp(*end, t)))
        .add_correction_fn(|start, end, t| Rotation(*start.slerp(*end, t)));

    app.add_interpolation_fn::<Transform>(TransformLinearInterpolation::lerp);
}
