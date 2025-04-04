use avian3d::prelude::Collider;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GlobalAssets {
    pub character: Handle<Scene>,
    pub bot: Handle<Scene>,
    pub laser: Handle<Scene>,
    pub target: Handle<Scene>,
}

#[derive(Resource, Default)]
pub struct LevelAssets {
    pub example_level: Handle<Scene>,
}
