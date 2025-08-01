use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_trauma_shake::prelude::*;
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
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EnokiPlugin)
        .add_plugins(TraumaPlugin)
        .add_plugins(screens_plugin);
    app.run();
}

pub const COLORS: &[Color] = &[
    Color::srgb(1.000, 1.000, 1.000),
    Color::srgb(0.996, 0.424, 0.565),
    Color::srgb(0.816, 0.216, 0.569),
    Color::srgb(0.529, 0.157, 0.416),
    Color::srgb(0.271, 0.141, 0.349),
    Color::srgb(0.149, 0.051, 0.204),
];

trait ShakeExt {
    fn apply_trauma(&mut self, value: f32);
}

impl ShakeExt for Shake {
    fn apply_trauma(&mut self, value: f32) {
        if self.trauma() < value {
            self.set_trauma(value);
        }
    }
}
