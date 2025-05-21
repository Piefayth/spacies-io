use avian3d::prelude::Position;
use bevy::prelude::*;
use bevy_hanabi::{EffectProperties, ParticleEffect, Value, VectorValue};
use lightyear::{client::message::ClientMessage, prelude::{is_client, FromServer, Message, TickManager}};
use mygame_assets::assets::FxAssets;
use mygame_common::{ship::{DespawnAfter, ProjectileHitNonShip}, Rendered};
use mygame_protocol::{component::Ship, message::ServerShipHit};

pub (crate) struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_ship_hit_fx).run_if(is_client));
        app.add_observer(render_ship_destroy_fx);
        app.add_observer(render_non_ship_hit_fx);
    }
}

fn render_ship_hit_fx(
    mut commands: Commands,
    fx_assets: Res<FxAssets>,
    tick_manager: Res<TickManager>,
    mut ship_hit_event_reader: EventReader<FromServer<ServerShipHit>>,
) {
    for ev in ship_hit_event_reader.read() {
        commands.spawn((
            ParticleEffect::new(fx_assets.laser_hit_vfx_large.clone()),
            Transform::from_translation(ev.message.position),
            DespawnAfter {
                created_at_tick: *tick_manager.tick(),
                lifetime_ticks: 62,
                is_server_controlled: false,
            },
        ));
    }
}

fn render_non_ship_hit_fx(
    trigger: Trigger<ProjectileHitNonShip>,
    mut commands: Commands,
    fx_assets: Res<FxAssets>,
    tick_manager: Res<TickManager>,
) {
    commands.spawn((
        ParticleEffect::new(fx_assets.laser_hit_vfx_small.clone()),
        Transform::from_translation(trigger.position),
        DespawnAfter {
            created_at_tick: *tick_manager.tick(),
            lifetime_ticks: 62,
            is_server_controlled: false,
        },
    ));
}

fn render_ship_destroy_fx(
    trigger: Trigger<OnRemove, Ship>,
    mut commands: Commands,
    fx_assets: Res<FxAssets>,
    tick_manager: Res<TickManager>,
    q_rendered_ships: Query<&Position, (Rendered, With<Ship>)>,
) {
    if let Ok(ship_position) = q_rendered_ships.get(trigger.target()) {
        commands.spawn((
            ParticleEffect::new(fx_assets.ship_destroy_vfx.clone()),
            Transform::from_translation(ship_position.0),
            DespawnAfter {
                created_at_tick: *tick_manager.tick(),
                lifetime_ticks: 62,
                is_server_controlled: false,
            },
        ));
    }
}
