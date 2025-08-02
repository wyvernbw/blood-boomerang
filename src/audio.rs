use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bon::Builder;

pub mod prelude {
    pub use super::VolumeSettings;
    pub use super::my_audio_plugin;
    pub use bevy_kira_audio::prelude::*;
}

#[derive(Component, Resource, Debug, Builder)]
pub struct VolumeSettings {
    sfx: f64,
    music: f64,
}

impl VolumeSettings {
    pub fn calc_sfx(&self, value: f64) -> f64 {
        self.sfx * value
    }
    pub fn calc_music(&self, value: f64) -> f64 {
        self.music * value
    }
}

pub fn my_audio_plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .insert_resource(VolumeSettings::builder().sfx(1.0).music(1.0).build());
}
