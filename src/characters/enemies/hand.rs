use std::f32::consts::PI;
use std::time::Duration;

use crate::audio::prelude::*;
use crate::autotimer::{AutoTimer, TimerRepeating};
use crate::characters::bullet::{BulletMaxWrap, bullet_base};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use bon::Builder;
use rand::{Rng, rng};

use crate::characters::enemies::ghost::{CommandsGhost, prelude::*};
use crate::characters::enemies::prelude::*;
use crate::characters::{AimDirRotationOffset, SpeedMod, prelude::*};
use crate::effects::prelude::*;
use crate::exp_decay::ExpDecay;
use crate::screens::prelude::*;

pub mod prelude {}

pub fn hand_plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(GameScreen::SplashFirst).load_collection::<HandAssets>(),
    )
    .add_systems(
        Update,
        (hand_shoot_fingers).run_if(not(in_state(GameScreen::SplashFirst))),
    );
}

#[derive(Component, Builder, Clone)]
pub struct Hand {
    #[builder(with = |shoot_rate: Duration| Timer::new(shoot_rate, TimerMode::Repeating))]
    #[builder(name = shoot_rate)]
    shoot_rate_timer: Timer,
    #[builder(default = 5)]
    finger_count: usize,
}

#[derive(AssetCollection, Resource)]
pub struct HandAssets {
    #[asset(path = "enemies/hand.png")]
    pub sprite: Handle<Image>,
    #[asset(path = "enemies/finger.png")]
    pub finger: Handle<Image>,
}

#[derive(Component, Builder)]
pub struct HandArgs<'a> {
    assets: &'a Res<'a, HandAssets>,
    hand: Hand,
}

pub trait CommandsHand<T> {
    fn spawn_hand(&'_ mut self, args: T) -> EntityCommands<'_>;
}

impl<'w, 's> CommandsHand<HandArgs<'_>> for Commands<'w, 's> {
    fn spawn_hand(&'_ mut self, args: HandArgs<'_>) -> EntityCommands<'_> {
        let mut commands = self.spawn(enemy_base());
        commands
            .insert((
                Sprite {
                    image: args.assets.sprite.clone(),
                    ..default()
                },
                args.hand,
            ))
            .insert(Health(20))
            .insert(Speed(48.0))
            .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 0.0)))
            .insert(Collider::cuboid(16., 16.))
            .insert(EnemyClass::Ranged { max_range: 96.0 })
            .insert(AimDirRotationOffset(-PI))
            .with_children(|parent| {
                parent.spawn((
                    EnemyHitbox,
                    Transform::default(),
                    Sensor,
                    Collider::cuboid(8., 8.),
                    CollisionGroups::new(ENEMY_HITBOX_GROUP, PLAYER_HURTBOX_GROUP),
                    ActiveEvents::COLLISION_EVENTS,
                    ColliderDebugColor(Hsla::hsl(340., 1.0, 0.8)),
                ));
            });
        commands
    }
}

#[derive(Component)]
pub struct Finger;

fn hand_shoot_fingers(
    mut commands: Commands,
    time: Res<Time>,
    mut hands: Query<(&mut Hand, &Transform, &AimDir)>,
    assets: Res<HandAssets>,
) {
    for (mut hand, transform, aim_dir) in hands.iter_mut() {
        hand.shoot_rate_timer.tick(time.delta());
        if !hand.shoot_rate_timer.just_finished() {
            continue;
        }
        let spread = 20.0f32.to_radians();
        let half_finger_count = hand.finger_count as i32 / 2;
        for idx in -half_finger_count..half_finger_count {
            let current_angle_base = -spread * (idx) as f32;
            let next_aim_dir = Vec2::from_angle(current_angle_base).rotate(**aim_dir);
            commands
                .spawn(bullet_base(200.0))
                .insert(BulletMaxWrap(1))
                .insert(Damage(1))
                .insert(Finger)
                .insert(Sprite {
                    image: assets.finger.clone(),
                    ..default()
                })
                .insert(Collider::ball(8.0))
                .insert(CollidingEntities::default())
                .insert(CollisionGroups::new(
                    ENEMY_HITBOX_GROUP,
                    PLAYER_HURTBOX_GROUP,
                ))
                .insert(EnemyHitbox)
                .insert(*transform)
                .insert(AimDir(next_aim_dir))
                .insert(AimDirRotationOffset(PI))
                .insert(Velocity {
                    linvel: 100.0 * next_aim_dir,
                    ..default()
                });
        }
    }
}
