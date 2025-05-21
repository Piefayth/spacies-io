use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hanabi::EffectAsset;

use crate::materials::SkyboxMaterial;

#[derive(Resource, Default)]
pub struct GlobalAssets {
    pub character: Handle<Scene>,
    pub bot: Handle<Scene>,
    pub laser: Handle<Scene>,
    pub target: Handle<Scene>,

    pub skybox_mesh: Handle<Mesh>,
    pub skybox_image: Handle<Image>,
    pub skybox_material: Handle<SkyboxMaterial>,
}

#[derive(Resource, Default)]
pub struct FxAssets {
    pub laser_hit_vfx_large: Handle<EffectAsset>,
    pub laser_hit_vfx_small: Handle<EffectAsset>,
    pub ship_destroy_vfx: Handle<EffectAsset>,
    pub ship_damage_vfx: Handle<EffectAsset>,
}

#[derive(Resource, Default)]
pub struct LevelAssets {
    pub example_level: Handle<Scene>,
}
