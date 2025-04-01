use bevy::{
    ecs::system::{StaticSystemParam, lifetimeless::SRes},
    prelude::*,
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
            InputMap::<NetworkedInput>::default()
                .with_dual_axis(NetworkedInput::Aim, AimInput)
                .with_dual_axis(NetworkedInput::Aim, GamepadStick::LEFT),
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

fn update_aim_direction(mut aim_direction: ResMut<AimDirection>, windows: Query<&Window>) {
    let window = windows.single();

    if let Some(cursor_position) = window.cursor_position() {
        // Normalize cursor position to -1 to 1 range
        let normalized_x = (cursor_position.x / window.width()) * 2.0 - 1.0;
        let normalized_y = (cursor_position.y / window.height()) * 2.0 - 1.0;

        aim_direction.direction = Vec2::new(normalized_x, -normalized_y);
    }
}
