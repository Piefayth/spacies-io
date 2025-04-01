use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum NetworkedInput {
    #[actionlike(DualAxis)]
    Aim,
}

pub fn register_input(app: &mut App) {
    app.add_plugins(LeafwingInputPlugin {
        config: InputConfig::<NetworkedInput> {
            lag_compensation: true, // good default?
            ..default()
        },
    });
}
