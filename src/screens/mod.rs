use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::screens::camera_setup::camera_setup_plugin;

mod camera_setup;

pub mod prelude {
    pub use super::GameScreen;
    pub use super::camera_setup::prelude::*;
    pub use super::screens_plugin;
}

pub fn screens_plugin(app: &mut App) {
    app.init_state::<GameScreen>()
        .add_plugins(camera_setup_plugin)
        .add_loading_state(
            LoadingState::new(GameScreen::Splash).continue_to_state(GameScreen::Gameplay),
        );
}

#[derive(States, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum GameScreen {
    #[default]
    Splash,
    Gameplay,
}
