use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use rand::Rng;

use crate::{
    characters::{
        AimDir,
        bullet::{BulletMaxWrap, bullet_base},
        player::{Player, PlayerAction, PlayerAssets},
    },
    screens::GameScreen,
};

pub fn player_shoot_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (player_shoot_system, spin_boomerangs).run_if(in_state(GameScreen::Gameplay)),
    );
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerShoot {
    pub rate: f32,
    pub spread: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerBoomerang;

fn player_shoot_system(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(&Transform, &AimDir, &PlayerShoot), With<Player>>,
    action_state: Res<ActionState<PlayerAction>>,
    mut cooldown: Local<f32>,
    player_assets: Res<PlayerAssets>,
) {
    let dt = time.delta_secs();
    let (transform, &aim_dir, &PlayerShoot { rate, spread }) = player.into_inner();
    if action_state.pressed(&PlayerAction::Shoot) {
        *cooldown -= dt;
        if *cooldown <= 0.0 {
            *cooldown += rate;
            // shoot
            let half_spread = spread / 2.0;
            let angle = rand::rng().random_range(-half_spread..half_spread);
            commands
                .spawn(bullet_base(4.0))
                .insert(BulletMaxWrap(1))
                .insert(PlayerBoomerang)
                .insert(Collider::ball(4.0))
                .insert(Sensor)
                .insert(Transform::from_translation(
                    transform.translation + aim_dir.extend(0.0) * 8.0 + vec3(0.0, 8.0, 0.0),
                ))
                .insert(Velocity {
                    linvel: Vec2::from_angle(aim_dir.to_angle() + angle) * 200.0,
                    ..default()
                })
                .insert(Sprite {
                    anchor: bevy::sprite::Anchor::Center,
                    image: player_assets.boomerang_sprite.clone(),
                    ..default()
                });
        }
    } else {
        *cooldown = 0.0;
    }
}

fn spin_boomerangs(mut query: Query<&mut Transform, With<PlayerBoomerang>>, time: Res<Time>) {
    let dt = time.delta_secs();
    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(Vec3::Z, PI * 2.0 * dt * 8.0))
    }
}
