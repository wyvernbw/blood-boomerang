use std::{marker::PhantomData, time::Duration};

use crate::exp_decay::ExpDecay;
use bevy::prelude::*;
use bon::Builder;
use tracing::instrument;

pub mod prelude {
    pub use super::ghost_sprite_plugin;
    pub use super::ghost_sprite_plugin_default;
    pub use super::{
        GhostSprite, GhostSpriteGeneric, GhostSpriteSpawner, GhostSpriteSpawnerGeneric,
        GhostSpriteSpawnerKind,
    };
}

pub fn ghost_sprite_plugin_default(app: &mut App) {
    app.add_plugins(ghost_sprite_plugin::<()>);
}

pub fn ghost_sprite_plugin<B: Bundle>(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_ghost_sprites::<B>, ghost_sprite_fade_out::<B>).chain(),
    );
}

#[derive(Component, Default, Debug)]
pub struct GhostSpriteGeneric<B: Bundle = ()> {
    pub decay: f32,
    _b: PhantomData<B>,
}

pub type GhostSprite = GhostSpriteGeneric<()>;

impl<B: Bundle> GhostSpriteGeneric<B> {
    pub fn new(decay: f32) -> Self {
        Self {
            decay,
            _b: PhantomData,
        }
    }
}

#[derive(Component, Debug, Builder, Clone)]
pub struct GhostSpriteSpawnerGeneric<B: Bundle = ()> {
    pub kind: GhostSpriteSpawnerKind,
    #[builder(with = |secs: f32| Timer::from_seconds(secs, TimerMode::Repeating))]
    #[builder(name = rate)]
    pub rate_timer: Timer,
    pub ghost_decay: f32,
    #[builder(skip)]
    pub ghosts: Vec<Entity>,
    #[builder(skip)]
    _b: PhantomData<B>,
}

pub type GhostSpriteSpawner = GhostSpriteSpawnerGeneric<()>;

#[derive(Debug, Clone, Copy)]
pub enum GhostSpriteSpawnerKind {
    Count(usize),
    Time(f32),
    Infinite,
}

#[derive(Component, Debug, Deref, DerefMut)]
struct GhostSpriteSpawnerTimer(Timer);

#[instrument(skip_all)]
fn spawn_ghost_sprites<B: Bundle>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut GhostSpriteSpawnerGeneric<B>,
        Option<&GhostSpriteSpawnerTimer>,
        Option<&Sprite>,
        Option<&Mesh2d>,
        &Transform,
    )>,
) {
    for (entity, mut spawner, timer, sprite, mesh, transform) in query.iter_mut() {
        spawner.rate_timer.tick(time.delta());
        let rate = spawner.rate_timer.duration();
        if spawner.rate_timer.just_finished() {
            if let Some(sprite) = sprite {
                let ghost_id = commands
                    .spawn((
                        GhostSpriteGeneric::<B>::new(spawner.ghost_decay),
                        sprite.clone(),
                        *transform,
                    ))
                    .id();
                spawner.ghosts.push(ghost_id);
                tracing::trace!(?ghost_id, "spawned sprite ghost with sprite");
            }
            if let Some(mesh) = mesh {
                let ghost_id = commands
                    .spawn((
                        GhostSpriteGeneric::<B>::new(spawner.ghost_decay),
                        mesh.clone(),
                        *transform,
                    ))
                    .id();
                spawner.ghosts.push(ghost_id);
                tracing::trace!(?ghost_id, "spawned sprite ghost with mesh");
            }
            match (&mut spawner.kind, timer) {
                (GhostSpriteSpawnerKind::Count(count), _) => {
                    *count -= 1;
                    if *count == 0 {
                        commands
                            .entity(entity)
                            .try_remove::<GhostSpriteSpawnerGeneric<B>>();
                    }
                }
                (GhostSpriteSpawnerKind::Time(time), None) => {
                    let mut timer = Timer::new(Duration::from_secs_f32(*time), TimerMode::Once);
                    timer.tick(rate);
                    if !timer.finished() {
                        commands
                            .entity(entity)
                            .insert_if_new(GhostSpriteSpawnerTimer(timer));
                    }
                }
                (GhostSpriteSpawnerKind::Time(_), Some(timer)) => {
                    if timer.finished() {
                        commands
                            .entity(entity)
                            .try_remove::<GhostSpriteSpawnerGeneric<B>>();
                    }
                }
                (GhostSpriteSpawnerKind::Infinite, _) => {}
            }
        }
    }
}

pub fn ghost_sprite_fade_out<B: Bundle>(
    mut commands: Commands,
    mut query: Query<(Entity, &GhostSpriteGeneric<B>, &mut Sprite)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (ghost, GhostSpriteGeneric::<B> { decay, .. }, mut sprite) in query.iter_mut() {
        let alpha = sprite.color.alpha().exp_decay(0.0, *decay, dt);
        sprite.color.set_alpha(alpha);
        if sprite.color.alpha() < 0.005 {
            commands.entity(ghost).try_despawn();
        }
    }
}
