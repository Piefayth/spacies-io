use bevy::prelude::*;

mod main_menu;
pub (crate) mod respawn_menu;
pub (crate) mod system_menu;

pub(crate) struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::MainMenuPlugin,
            system_menu::SystemMenuPlugin,
            respawn_menu::RespawnMenuPlugin,
        ));
    }
}
