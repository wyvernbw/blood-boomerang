use std::f32::consts::PI;

use crate::characters::bullet::bullet_plugin;
use crate::characters::enemies::prelude::*;
use crate::characters::player::{Player, player_plugin};
use crate::exp_decay::ExpDecay;
use crate::screens::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod bullet;
pub mod enemies;
pub mod player;

pub mod prelude {
    pub use super::Bobbing;
    pub use super::LookAtPlayer;
    pub use super::character_base;
    pub use super::characters_plugin;
    pub use super::{AimDir, Character, Damage, Health, Hitbox, Hurtbox, Speed};
    pub use super::{
        ENEMY_HITBOX_GROUP, ENEMY_HURTBOX_GROUP, PLAYER_HITBOX_GROUP, PLAYER_HURTBOX_GROUP,
    };
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
                aim_at_player,
                point_towards_aim_direction.after(aim_at_player),
                screen_wrap_system,
                flip_character_sprite,
                character_bobbing,
            )
                .run_if(in_state(GameScreen::Gameplay)),
        );
}

pub const PLAYER_HURTBOX_GROUP: Group = Group::GROUP_1;
pub const PLAYER_HITBOX_GROUP: Group = Group::GROUP_2;
pub const ENEMY_HURTBOX_GROUP: Group = Group::GROUP_3;
pub const ENEMY_HITBOX_GROUP: Group = Group::GROUP_4;

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
        ActiveCollisionTypes::all(),
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
pub struct Damage(i32);

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

#[derive(Component)]
pub struct Bobbing;

fn character_bobbing(
    mut query: Query<
        (&mut Transform, &Velocity, &PrevVelocity, &Speed),
        (With<Character>, With<Bobbing>),
    >,
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

#[derive(Component, Debug, Default)]
pub struct Hurtbox;

#[derive(Component, Debug, Default)]
pub struct Hitbox;

#[derive(Component, Debug, Default)]
pub struct LookAtPlayer;

fn aim_at_player(
    mut query: Query<(&mut AimDir, &Transform), With<LookAtPlayer>>,
    player: Single<&Transform, With<Player>>,
) {
    for (mut aim_dir, transform) in query.iter_mut() {
        **aim_dir = (player.translation - transform.translation)
            .normalize()
            .xy();
    }
}

fn point_towards_aim_direction(
    mut query: Query<(&mut Transform, &AimDir), Without<Bobbing>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut transform, aim_dir) in query.iter_mut() {
        let angle = aim_dir.to_angle() + PI / 2.;
        // transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
        transform.rotation =
            transform
                .rotation
                .exp_decay(Quat::from_axis_angle(Vec3::Z, angle), 8.0, dt);
    }
}
