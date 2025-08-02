use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::screens::after_death::prelude::*;
use crate::screens::splash::prelude::*;
use crate::screens::{camera_setup::camera_setup_plugin, gameplay::gameplay_plugin};

mod after_death;
mod camera_setup;
mod gameplay;
mod splash;

pub mod prelude {
    pub use super::GameScreen;
    pub use super::camera_setup::prelude::*;
    pub use super::screens_plugin;
}

pub fn screens_plugin(app: &mut App) {
    app.init_state::<GameScreen>()
        .add_loading_state(
            LoadingState::new(GameScreen::SplashFirst).continue_to_state(GameScreen::SplashNext),
        )
        .add_plugins(camera_setup_plugin)
        .add_plugins(gameplay_plugin)
        .add_plugins(splash_screen_plugin)
        .add_plugins(after_death_plugin);
}

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum GameScreen {
    #[default]
    SplashFirst,
    SplashNext,
    Gameplay,
    AfterDeath,
}
