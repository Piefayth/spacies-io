use assets::{FxAssets, GlobalAssets, LevelAssets};
use avian3d::prelude::{Collider, ColliderConstructor, RigidBody};
use bevy::{
    asset::{AssetPlugin as BevyAssetPlugin, LoadState}, color::palettes::css::{BLUE, GREEN, RED, WHITE}, gltf::{GltfMesh, GltfPlugin}, prelude::*, render::RenderApp, ui::UiPlugin
};
use bevy_hanabi::HanabiPlugin;
use images::hemispherical_gradient;
use materials::{GradientMaterial, SharedMaterialPlugin, SkyboxMaterial};
use meshes::skybox_mesh;
use mygame_protocol::message::Level;

pub mod assets;
mod effects;
mod materials;
mod meshes;
mod images;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, on_level_change)
            .add_systems(
                Update,
                check_asset_loading.run_if(in_state(LevelState::Loading)),
            )
            .add_systems(OnEnter(LevelState::Postprocess), postprocess_assets)
            .init_state::<LevelState>()
            .init_resource::<CurrentLevel>()
            .init_resource::<LoadingAssets>()
            .init_resource::<LevelAssets>()
            .init_resource::<GlobalAssets>()
            .init_resource::<FxAssets>()
            .register_type::<Geometry>();
        
        // certain assets and asset processing steps require that rendering is enabled, we are using UiPlugin as a cheat-y way to check
        if app.is_plugin_added::<UiPlugin>() {
            app.add_systems(Startup, effects::register_fx);

            app.add_plugins(SharedMaterialPlugin);

            app.add_systems(OnEnter(LevelState::Postprocess), (
                postprocess_render_assets,
            ));

        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LevelState {
    #[default]
    Unloaded,
    Loading,
    Postprocess,
    Loaded,
}

/// Resource to track the current handles being loaded
#[derive(Resource, Default)]
pub struct LoadingAssets {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Resource, Clone, Deref, DerefMut, Default)]
pub struct CurrentLevel(pub Level);

/// Tag component to let external systems identify "what" kind of thing got loaded
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Geometry;

/// When CurrentLevel changes, load the assets required.
/// Queue the resultant Handles to be polled for completion in `check_asset_loading`
fn on_level_change(
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut level_assets: ResMut<LevelAssets>,
    mut global_assets: ResMut<GlobalAssets>,
    mut next_level_state: ResMut<NextState<LevelState>>,
) {
    if !current_level.is_changed() {
        return;
    }

    // TODO: need to drop all handles from the loaded level
    // and despawn everything from the loaded level

    global_assets.character =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("scenes/craft_speederB.glb"));
    global_assets.bot =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("scenes/craft_speederC.glb"));
    global_assets.laser =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("scenes/weapon-ammo-arrow-scaled.glb"));
    global_assets.target =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("scenes/target-large.glb"));
        
    global_assets.skybox_mesh = meshes.add(skybox_mesh(10000.));
    global_assets.skybox_image = images.add(hemispherical_gradient(BLUE.into(), RED.into()));

    match **current_level {
        Level::Example => {
            level_assets.example_level = asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("scenes/Cylinder.glb"));

            loading_assets
                .handles
                .push(level_assets.example_level.clone().untyped());
        }
        Level::Void => {}
    }

    next_level_state.set(LevelState::Loading);
}

/// Sets the AssetState to Loaded once all queued Handles have finished loading
/// Downstream systems should consume this state change as part of their loading sequence
fn check_asset_loading(
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    let all_loaded = loading_assets
        .handles
        .iter()
        .all(|handle| matches!(asset_server.get_load_state(handle), Some(LoadState::Loaded)));

    if all_loaded {
        info!("All assets loaded successfully");
        next_state.set(LevelState::Postprocess);
        loading_assets.handles.clear();
    }
}

fn postprocess_assets(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    mut scenes: ResMut<Assets<Scene>>,
    level_assets: Res<LevelAssets>,
    global_assets: Res<GlobalAssets>,
    meshes: Res<Assets<Mesh>>,
) {
    // "Generate" colliders for the level
    // In the future, when Avian3d's Collision component is #[reflect], it would be nice
    //  to actually construct the colliders here, rather than defer them with ColliderConstructor
    match **current_level {
        Level::Example => {
            // After the GLTF finishes loading, it's now a bevy Scene
            // that contains a World we can mutate freely
            if let Some(scene) = scenes.get_mut(&level_assets.example_level) {
                let mut entities_to_process = Vec::new();

                for entity_ref in scene.world.iter_entities() {
                    let entity = entity_ref.id();
                    if let Some(mesh_handle) = scene.world.get::<Mesh3d>(entity) {
                        entities_to_process.push((entity, mesh_handle.clone()));
                    }
                }

                for (entity, mesh_handle) in entities_to_process {
                    if let Some(mesh) = meshes.get(&mesh_handle) {
                        scene
                            .world
                            .entity_mut(entity)
                            .insert((ColliderConstructor::TrimeshFromMesh, Geometry));
                    }
                }
            }
        }
        Level::Void => (),
    }
    
    commands.set_state(LevelState::Loaded);
}

fn postprocess_render_assets(
    mut scenes: ResMut<Assets<Scene>>,
    mut global_assets: ResMut<GlobalAssets>,
    mut gradient_materials: ResMut<Assets<GradientMaterial>>,
    mut skybox_materials: ResMut<Assets<SkyboxMaterial>>,
) {
    global_assets.skybox_material = skybox_materials.add(SkyboxMaterial {
        sky_texture: global_assets.skybox_image.clone(),
    });

    // Swap out the material on the "laser"
    if let Some(scene) = scenes.get_mut(&global_assets.laser) {
        let mut material_having_entity = Entity::PLACEHOLDER;

        for entity_ref in scene.world.iter_entities() {
            if let Some(_) = scene.world.get::<MeshMaterial3d<StandardMaterial>>(entity_ref.id()) {
                material_having_entity = entity_ref.id();
            }
        }

        if material_having_entity != Entity::PLACEHOLDER {
            scene.world.entity_mut(material_having_entity).remove::<MeshMaterial3d<StandardMaterial>>();
            scene.world.entity_mut(material_having_entity).insert(MeshMaterial3d(gradient_materials.add(GradientMaterial {
                axis: 2,
                start_color: LinearRgba::new(0.0, 1.2, 0.0, 1.0),
                end_color: WHITE.into(),
                start: -0.5,
                end: 1.0,
            })));
        }
    }
}
