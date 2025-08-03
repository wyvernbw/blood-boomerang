use std::time::Duration;

use bevy::prelude::*;

use crate::audio::prelude::*;
use crate::screens::{GameScreen, MenuAssets};

pub mod prelude {
    pub use super::play_menu_sound;
    pub use super::splash_screen_plugin;
}

pub fn splash_screen_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameScreen::SplashFirst), spawn_splash_screen)
        .add_systems(OnExit(GameScreen::SplashFirst), (continue_to_splash_next))
        .add_systems(
            Update,
            splash_next_system.run_if(in_state(GameScreen::SplashNext)),
        )
        .add_systems(OnExit(GameScreen::SplashNext), despawn_splash_screen)
        .add_systems(OnEnter(GameScreen::SplashNext), play_menu_sound)
        .add_systems(OnExit(GameScreen::SplashNext), play_menu_sound)
        .add_systems(OnEnter(GameScreen::Gameplay), play_bg_music);
}

#[derive(Component, Default, Debug)]
struct SplashScreen;

fn spawn_splash_screen(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let splash = asset_server.load::<Image>("splash_art.png");

    commands.spawn((
        Sprite {
            image: splash,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
        SplashScreen,
    ));
}

fn continue_to_splash_next(mut next_screen: ResMut<NextState<GameScreen>>) {
    next_screen.set(GameScreen::SplashNext);
}

fn splash_next_system(
    mut next_screen: ResMut<NextState<GameScreen>>,
    time: Res<Time>,
    mut elapsed: Local<Duration>,
) {
    *elapsed += time.delta();
    if elapsed.as_secs_f32() > 1.0 {
        next_screen.set(GameScreen::Tutorial);
    }
}

fn despawn_splash_screen(mut commands: Commands, splash: Single<Entity, With<SplashScreen>>) {
    commands.entity(*splash).try_despawn();
}

pub fn play_menu_sound(audio: Res<Audio>, volume: Res<VolumeSettings>, assets: Res<MenuAssets>) {
    audio
        .play(assets.menu_sound.clone())
        .with_volume(volume.calc_sfx(1.0));
}

#[derive(Component)]
pub struct BgMusic;

pub fn play_bg_music(
    audio: Res<Audio>,
    assets: Res<MenuAssets>,
    music: Option<Single<&BgMusic>>,
    mut commands: Commands,
) {
    if music.is_some() {
        return;
    }
    commands.spawn(BgMusic);
    audio
        .play(assets.background_music.clone())
        .looped()
        .fade_in(AudioTween::linear(Duration::from_secs_f32(2.0)))
        .with_volume(0.2);
}
