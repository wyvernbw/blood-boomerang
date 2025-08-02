pub mod ghost_sprite;

pub mod prelude {
    pub use super::ghost_sprite::prelude::*;
    use bevy::prelude::*;

    pub fn effects_plugin(app: &mut App) {
        app.add_plugins(ghost_sprite_plugin_default);
    }
}
