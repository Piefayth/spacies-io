use bevy::{
    ecs::system::{lifetimeless::SRes, StaticSystemParam}, input::mouse::MouseMotion, prelude::*
};
use leafwing_input_manager::{
    clashing_inputs::BasicInputs, plugin::InputManagerPlugin, prelude::{
        updating::{CentralInputStore, InputRegistration, UpdatableInput}, ActionState, DualAxislike, GamepadStick, InputMap, MouseMove, UserInput
    }, Actionlike, InputControlKind
};
use mygame_common::Simulated;
use mygame_protocol::input::NetworkedInput;
use serde::{Deserialize, Serialize};

use crate::{game_state::GameState, replication::LocalPlayer, ui::system_menu::SystemMenuState};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<LocalInput>::default())
            .add_systems(
                Update,
                (
                    add_input_maps,
                    handle_system_menu_or_cancel.run_if(in_state(GameState::Playing)),
                    update_aim_direction,
                ),
            )
            .init_resource::<AimDirection>()
            .register_input_kind::<AimInput>(InputControlKind::DualAxis);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum LocalInput {
    #[actionlike(Button)]
    SystemMenuOrCancel,
}

#[derive(Resource, Default)]
pub struct AimDirection {
    pub direction: Vec2,
}

fn add_input_maps(
    mut commands: Commands,
    q_local_player: Query<Entity, (Simulated, Added<LocalPlayer>)>,
) {
    for player in &q_local_player {
        commands.entity(player).insert((
            InputMap::<LocalInput>::default().with(LocalInput::SystemMenuOrCancel, KeyCode::Escape),
            ActionState::<LocalInput>::default(),
            InputMap::<NetworkedInput>::default()
                .with_dual_axis(NetworkedInput::Aim, AimInput)
                .with_dual_axis(NetworkedInput::Aim, GamepadStick::LEFT)
                .with(NetworkedInput::Fire, MouseButton::Left),
        ));
    }
}

fn handle_system_menu_or_cancel(
    q_local_inputs: Query<&ActionState<LocalInput>>,
    system_menu_state: Res<State<SystemMenuState>>,
    mut next_system_menu_state: ResMut<NextState<SystemMenuState>>,
) {
    for local_input in &q_local_inputs {
        if local_input.just_pressed(&LocalInput::SystemMenuOrCancel) {
            match **system_menu_state {
                SystemMenuState::Open => next_system_menu_state.set(SystemMenuState::Closed),
                SystemMenuState::Closed => next_system_menu_state.set(SystemMenuState::Open),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct AimInput;

impl UserInput for AimInput {
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::None
    }
}

impl UpdatableInput for AimInput {
    type SourceData = SRes<AimDirection>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        central_input_store.update_dualaxislike(
            AimInput,
            Vec2::new(source_data.direction.x, source_data.direction.y),
        );
    }
}

impl DualAxislike for AimInput {
    fn axis_pair(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Vec2 {
        input_store.pair(self)
    }
}

fn update_aim_direction(
    mut aim_direction: ResMut<AimDirection>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    // Get cumulative motion this frame
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }
    
    // Apply sensitivity and update direction
    let sensitivity = 0.001; // Adjust this value to your liking
    
    if delta != Vec2::ZERO {
        // Update the aim direction based on mouse movement
        // You might want to clamp these values to keep them in a certain range
        aim_direction.direction.x += delta.x * sensitivity;
        aim_direction.direction.y -= delta.y * sensitivity; // Invert Y for typical FPS controls
        
        // Optional: Normalize or clamp the direction vector
        // aim_direction.direction = aim_direction.direction.normalize();
        // or
        aim_direction.direction.x = aim_direction.direction.x.clamp(-1.0, 1.0);
        aim_direction.direction.y = aim_direction.direction.y.clamp(-1.0, 1.0);
    }
}
