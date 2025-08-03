use std::time::Duration;

use crate::audio::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use bon::Builder;
use rand::{Rng, rng};

use crate::characters::enemies::ghost::{CommandsGhost, prelude::*};
use crate::characters::enemies::prelude::*;
use crate::characters::{SpeedMod, prelude::*};
use crate::exp_decay::ExpDecay;
use crate::screens::prelude::*;

pub mod prelude {
    pub use super::coffin_plugin;
    pub use super::{Coffin, CoffinArgs, CoffinAssets, CommandsCoffin};
}

pub fn coffin_plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(GameScreen::SplashFirst).load_collection::<CoffinAssets>(),
    )
    .add_systems(
        Update,
        (coffin_spawn_ghosts, coffin_ghosts_speed_up)
            .run_if(not(in_state(GameScreen::SplashFirst))),
    );
}

#[derive(Component, Builder)]
pub struct Coffin {
    #[builder(with = |duration: Duration| Timer::new(duration, TimerMode::Repeating))]
    #[builder(name = rate)]
    spawn_rate_timer: Timer,
    #[builder(with = |duration: Duration| Timer::new(duration, TimerMode::Once))]
    #[builder(name = initial_rate)]
    initial_rate_timer: Timer,
    count: usize,
    spacing: f32,
}

#[derive(AssetCollection, Resource)]
pub struct CoffinAssets {
    #[asset(path = "enemies/coffin.png")]
    pub sprite: Handle<Image>,
    #[asset(path = "enemies/ghostlike.wav")]
    pub spawn_sound: Handle<AudioSource>,
}

#[derive(Component, Builder)]
pub struct CoffinArgs<'a> {
    assets: &'a Res<'a, CoffinAssets>,
    coffin: Coffin,
}

pub trait CommandsCoffin<T> {
    fn spawn_coffin(&'_ mut self, args: T) -> EntityCommands<'_>;
}

impl<'w, 's> CommandsCoffin<CoffinArgs<'_>> for Commands<'w, 's> {
    fn spawn_coffin(&'_ mut self, args: CoffinArgs) -> EntityCommands<'_> {
        let mut commands = self.spawn(enemy_base());
        commands
            .insert((
                Sprite {
                    image: args.assets.sprite.clone(),
                    ..default()
                },
                args.coffin,
            ))
            .insert(Health(15))
            .insert(Speed(48.0))
            .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 0.0)))
            .insert(EnemyClass::Ranged { max_range: 64.0 })
            .with_children(|parent| {
                parent.spawn((
                    EnemyHitbox,
                    Transform::default(),
                    Sensor,
                    Collider::cuboid(8., 16.),
                    CollisionGroups::new(ENEMY_HITBOX_GROUP, PLAYER_HURTBOX_GROUP),
                    ActiveEvents::COLLISION_EVENTS,
                    ColliderDebugColor(Hsla::hsl(340., 1.0, 0.8)),
                ));
            });
        commands
    }
}

#[derive(Component)]
struct SpawnedByCoffin;

fn coffin_spawn_ghosts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Coffin, &Transform)>,
    ghost_assets: Res<GhostAssets>,
    coffin_assets: Res<CoffinAssets>,
    audio: Res<Audio>,
    volume: Res<VolumeSettings>,
) {
    for (mut coffin, transform) in query.iter_mut() {
        coffin.initial_rate_timer.tick(time.delta());
        if coffin.initial_rate_timer.finished() {
            coffin.spawn_rate_timer.tick(time.delta());
        }
        if coffin.spawn_rate_timer.just_finished() || coffin.initial_rate_timer.just_finished() {
            for _ in 0..coffin.count {
                let dir = vec2(rng().random_range(0.0..1.0), rng().random_range(0.0..1.0))
                    .normalize_or(vec2(1.0, 0.0));
                commands
                    .spawn_ghost(GhostArgs::builder().assets(&ghost_assets).build())
                    .insert(transform.with_translation(
                        transform.translation + (dir * coffin.spacing).extend(0.0),
                    ))
                    .insert(SpeedMod(0.0))
                    .insert(SpawnedByCoffin);
                audio
                    .play(coffin_assets.spawn_sound.clone())
                    .with_volume(volume.calc_sfx(1.0));
            }
        }
    }
}

fn coffin_ghosts_speed_up(mut query: Query<&mut SpeedMod, With<SpawnedByCoffin>>, time: Res<Time>) {
    let dt = time.delta_secs();
    for mut speed_mod in query.iter_mut() {
        let new_mod = speed_mod.0.exp_decay(1.0, 8.0, dt);
        **speed_mod = new_mod;
    }
}
