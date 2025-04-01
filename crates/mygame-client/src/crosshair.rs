use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}};

use crate::replication::LocalPlayer;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        /*
            #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
            pub enum NetworkedInput {
                #[actionlike(DualAxis)]
                Aim,
            }
         */
        // using leafwing input manager, we will get the ActionState<NetworkedInput> from the LocalPlayer and use that to draw the crosshair
        // the crosshair will be implemented as two separate quads using the same "crosshair" shader that we will have to implement
        // this will be a "spaceship style" crosshair, hence the two quads (that will be positioned along an imaginary line shooting out from the front of the ship)
        // in order to get the appropriate mesh positions, we will basically take the dual axis normalized "Aim" position, translate into window coordinates
            // then shoot a ray from there into two intersecting vertical planes, one slightly further away from the other
            // the position of intersection at each plane is the location for each mesh

        app
            .add_systems(Update, spawn_crosshair_meshes)
            .add_plugins(
                MaterialPlugin::<CrosshairMaterial>::default(),
            );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CrosshairMaterial {
    #[uniform(0)]
    color: LinearRgba, 
}

impl Material for CrosshairMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/crosshair.wgsl".into()
    }
}

#[derive(Component)]
struct CrosshairNear;

#[derive(Component)]
struct CrosshairFar;

const CROSSHAIR_NEAR_DISTANCE: f32 = 20.0;
const CROSSHAIR_FAR_DISTANCE: f32 = 30.0;

fn spawn_crosshair_meshes(
    mut commands: Commands,
    q_added_local_player: Query<(Entity, &Transform), Added<LocalPlayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CrosshairMaterial>>,
) {
    let near_plane = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.0, 1.0)));
    let far_plane = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.5, 1.5)));
    
    let crosshair_material = materials.add(CrosshairMaterial {
        color: LinearRgba::new(1.0, 0.0, 0.0, 1.0), // Red crosshair
    });
    
    for (player_entity, player_transform) in &q_added_local_player {
        commands.entity(player_entity)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Mesh3d(near_plane.clone()),
                    MeshMaterial3d(crosshair_material.clone()),
                    CrosshairNear,
                    Transform::from_translation(player_transform.forward() * CROSSHAIR_NEAR_DISTANCE)
                ));

                child_builder.spawn((
                    Mesh3d(far_plane.clone()),
                    MeshMaterial3d(crosshair_material.clone()),
                    CrosshairFar,
                    Transform::from_translation(player_transform.forward() * CROSSHAIR_FAR_DISTANCE)
                ));
            });
    }
}
