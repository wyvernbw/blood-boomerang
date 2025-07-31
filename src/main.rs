use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use characters::prelude::*;
use screens::prelude::*;

mod characters;
mod exp_decay;
mod screens;

pub fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(screens_plugin)
        .add_plugins(characters_plugin);
    app.run();
}
