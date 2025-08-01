use std::time::Duration;

use bevy::prelude::*;

use crate::screens::GameScreen;

pub mod prelude {
    pub use super::splash_screen_plugin;
}

pub fn splash_screen_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameScreen::SplashFirst), spawn_splash_screen)
        .add_systems(OnExit(GameScreen::SplashFirst), continue_to_splash_next)
        .add_systems(
            Update,
            splash_next_system.run_if(in_state(GameScreen::SplashNext)),
        )
        .add_systems(OnExit(GameScreen::SplashNext), despawn_splash_screen);
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
        next_screen.set(GameScreen::Gameplay);
    }
}

fn despawn_splash_screen(mut commands: Commands, splash: Single<Entity, With<SplashScreen>>) {
    commands.entity(*splash).try_despawn();
}
