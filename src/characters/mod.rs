use std::f32::consts::PI;

use crate::characters::bullet::bullet_plugin;
use crate::characters::enemies::prelude::*;
use crate::characters::player::player_plugin;
use crate::screens::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod bullet;
pub mod enemies;
pub mod player;

pub mod prelude {
    pub use super::characters_plugin;
}

pub fn characters_plugin(app: &mut App) {
    app.add_event::<ScreenWrapEvent>()
        .add_plugins(player_plugin)
        .add_plugins(bullet_plugin)
        .add_plugins(enemies_plugin)
        .add_systems(
            Update,
            (
                apply_character_velocity,
                screen_wrap_system,
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
        ScreenWrap,
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

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct PrevVelocity(Velocity);

#[derive(Component, Deref, DerefMut, Clone, Copy)]
pub struct Health(i32);

#[derive(Component, Deref, DerefMut, Clone, Copy)]
pub struct Speed(f32);

#[derive(Component, Deref, DerefMut, Debug, Clone, Copy)]
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

#[derive(Component, Debug, Default)]
pub struct ScreenWrap;

#[derive(Event)]
pub struct ScreenWrapEvent {
    entity: Entity,
}

impl ScreenWrapEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

const HALF_WIDTH: u32 = RES_WIDTH / 2;
const HALF_HEIGHT: u32 = RES_HEIGHT / 2;

fn screen_wrap_system(
    mut query: Query<(Entity, &mut Transform), With<ScreenWrap>>,
    mut events: EventWriter<ScreenWrapEvent>,
) {
    for (entity, mut transform) in query.iter_mut() {
        if transform.translation.y < -(HALF_HEIGHT as f32) - 8.0 {
            transform.translation.y = HALF_HEIGHT as f32 + 8.0;
            transform.translation.x *= -1.0;
            events.write(ScreenWrapEvent::new(entity));
        }
        if transform.translation.y > HALF_HEIGHT as f32 + 8.0 {
            transform.translation.y = -(HALF_HEIGHT as f32) - 8.0;
            transform.translation.x *= -1.0;
            events.write(ScreenWrapEvent::new(entity));
        }
        if transform.translation.x < -(HALF_WIDTH as f32) - 8.0 {
            transform.translation.x = HALF_WIDTH as f32 + 8.0;
            transform.translation.y *= -1.0;
            events.write(ScreenWrapEvent::new(entity));
        }
        if transform.translation.x > HALF_WIDTH as f32 + 8.0 {
            transform.translation.x = -(HALF_WIDTH as f32) - 8.0;
            transform.translation.y *= -1.0;
            events.write(ScreenWrapEvent::new(entity));
        }
    }
}
