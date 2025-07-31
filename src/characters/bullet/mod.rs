use crate::characters::player::player_plugin;
use crate::characters::{ScreenWrap, ScreenWrapEvent};
use crate::screens::prelude::*;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn bullet_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_bullet_velocity,
            bullet_lifetime_tick,
            bullet_count_wraps,
            bullets_despawn_from_wraps,
        )
            .run_if(in_state(GameScreen::Gameplay)),
    );
}

#[derive(Component)]
pub struct Bullet;

#[derive(Component, Clone, Copy, DerefMut, Deref)]
pub struct BulletLifetime(pub f32);

#[derive(Component, Clone, Copy, DerefMut, Deref, Default)]
pub struct BulletWrapCount(pub usize);

#[derive(Component, Clone, Copy, DerefMut, Deref)]
#[require(ScreenWrap, BulletWrapCount)]
pub struct BulletMaxWrap(pub usize);

#[must_use]
pub fn bullet_base(lifetime: f32) -> impl Bundle {
    (
        Transform::default(),
        Bullet,
        BulletLifetime(lifetime),
        ScreenWrap,
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController::default(),
        Velocity::default(),
    )
}

pub fn apply_bullet_velocity(
    mut query: Query<(&mut KinematicCharacterController, &Velocity), With<Bullet>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut controller, velocity) in query.iter_mut() {
        controller.translation = Some(velocity.linvel * dt);
    }
}

fn bullet_lifetime_tick(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BulletLifetime), With<Bullet>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (bullet, mut lifetime) in query.iter_mut() {
        **lifetime -= dt;
        if **lifetime < 0.0 {
            commands.entity(bullet).despawn();
        }
    }
}

fn bullet_count_wraps(
    mut query: Query<&mut BulletWrapCount, With<Bullet>>,
    mut events: EventReader<ScreenWrapEvent>,
) {
    for ScreenWrapEvent { entity } in events.read() {
        if let Ok(mut wrap_count) = query.get_mut(*entity) {
            **wrap_count += 1;
        }
    }
}

fn bullets_despawn_from_wraps(
    mut commands: Commands,
    query: Query<(Entity, &BulletWrapCount, &BulletMaxWrap), With<Bullet>>,
) {
    for (entity, wrap_count, max_wrap) in query.iter() {
        if **wrap_count > **max_wrap {
            commands.entity(entity).despawn();
        }
    }
}
