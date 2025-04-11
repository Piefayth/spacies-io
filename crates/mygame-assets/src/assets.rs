use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hanabi::EffectAsset;

#[derive(Resource, Default)]
pub struct GlobalAssets {
    pub character: Handle<Scene>,
    pub bot: Handle<Scene>,
    pub laser: Handle<Scene>,
    pub target: Handle<Scene>,
}

#[derive(Resource, Default)]
pub struct FxAssets {
    pub laser_hit_vfx: Handle<EffectAsset>,
    pub ship_destroy_vfx: Handle<EffectAsset>,
}

#[derive(Resource, Default)]
pub struct LevelAssets {
    pub example_level: Handle<Scene>,
}
