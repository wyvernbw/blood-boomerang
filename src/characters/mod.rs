use std::{f32::consts::PI, ops::Add};

use crate::{characters::player::player_plugin, exp_decay::ExpDecay, screens::prelude::*};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use color_eyre::owo_colors::OwoColorize;

pub mod player;

pub mod prelude {
    pub use super::characters_plugin;
}

pub fn characters_plugin(app: &mut App) {
    app.add_plugins(player_plugin).add_systems(
        Update,
        (
            apply_character_velocity,
            flip_character_sprite,
            character_bobbing,
        )
            .run_if(in_state(GameScreen::Gameplay)),
    );
}

pub fn character_base() -> impl Bundle {
    (
        Transform::default(),
        Character,
        Health(20),
        AimDir(Vec2::ZERO),
        RigidBody::KinematicVelocityBased,
        Collider::ball(8.0),
        KinematicCharacterController::default(),
        Velocity::default(),
        PrevVelocity::default(),
    )
}

#[derive(Component)]
pub struct Character;

#[derive(Component, Deref, DerefMut, Default)]
pub struct PrevVelocity(Velocity);

#[derive(Component, Deref, DerefMut)]
pub struct Health(i32);

#[derive(Component, Deref, DerefMut)]
pub struct Speed(f32);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct AimDir(Vec2);

fn apply_character_velocity(
    mut query: Query<
        (
            &mut KinematicCharacterController,
            &Velocity,
            &mut PrevVelocity,
        ),
        With<Character>,
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut controller, velocity, mut prev_velocity) in query.iter_mut() {
        controller.translation = Some(velocity.linvel * dt);
        **prev_velocity = *velocity;
    }
}

fn flip_character_sprite(mut query: Query<(&mut Sprite, &AimDir), With<Character>>) {
    for (mut sprite, aim_dir) in query.iter_mut() {
        sprite.flip_x = aim_dir.x < 0.0;
    }
}

fn character_bobbing(
    mut query: Query<(&mut Transform, &Velocity, &PrevVelocity, &Speed), With<Character>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (mut transform, velocity, _, speed) in query.iter_mut() {
        let speed_factor = velocity.linvel.length_squared() / (speed.0 * speed.0);
        let angle = ((t * 16.0).sin() - 0.5) * PI / 12.0 * speed_factor;
        transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
    }
}
