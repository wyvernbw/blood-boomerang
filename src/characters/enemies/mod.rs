use std::f32::consts::PI;
use std::sync::Arc;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enoki::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_trauma_shake::prelude::*;
use tracing::instrument;

use crate::ShakeExt;
use crate::characters::Speed;
use crate::characters::character_base;
use crate::characters::enemies::ghost::prelude::*;
use crate::characters::player::Player;
use crate::characters::prelude::*;
use crate::screens::GameScreen;

pub mod prelude {
    pub use super::despawn_enemies;
    pub use super::enemies_plugin;
    pub use super::enemy_base;
    pub use super::{Enemy, EnemyClass, EnemyHitbox, EnemyHurtbox};
}

pub mod ghost;

pub fn enemies_plugin(app: &mut App) {
    app.insert_resource(BoidSeparationUpdateRate::PerFrame)
        .add_plugins(ghost_plugin)
        .configure_loading_state(
            LoadingStateConfig::new(GameScreen::SplashFirst).load_collection::<EnemyAssets>(),
        )
        .add_event::<PlayerHitEvent>()
        .add_event::<EnemyHitEvent>()
        .add_event::<EnemyDiedEvent>()
        .add_systems(OnEnter(GameScreen::SplashNext), setup_hit_particle_material)
        .add_systems(
            Update,
            (
                boids_calculate_separation,
                boids_move_towards_player.after(boids_calculate_separation),
                enemy_check_for_player_collisions,
                enemies_take_damage,
                handle_enemy_died_events.after(enemies_take_damage),
                handle_enemy_hit_events
                    .after(enemies_take_damage)
                    .before(handle_enemy_died_events),
            )
                .run_if(in_state(GameScreen::Gameplay)),
        );
}

#[derive(Component, Debug)]
#[require(Boid, EnemyClass)]
pub struct Enemy;

#[derive(Component, Debug)]
#[require(Hitbox)]
pub struct EnemyHitbox;

#[derive(Component, Debug)]
#[require(Hurtbox, CollidingEntities)]
pub struct EnemyHurtbox;

pub fn enemy_base() -> impl Bundle {
    (
        character_base(),
        Enemy,
        Sensor,
        CollisionGroups::new(ENEMY_HURTBOX_GROUP, PLAYER_HITBOX_GROUP),
        ActiveEvents::COLLISION_EVENTS,
        EnemyHurtbox,
        LookAtPlayer,
    )
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Boid {
    flock_detection_range: f32,
    current_separation: Vec2,
    separation_speed: f32,
}

impl Boid {
    fn calculate_separation<'a>(
        &self,
        transform: &Transform,
        transforms: impl Iterator<Item = &'a Transform>,
    ) -> Vec2 {
        let flock_detection_range_squared = self.flock_detection_range * self.flock_detection_range;
        let (count, separation) = transforms
            .filter_map(|other_transform| {
                if other_transform
                    .translation
                    .distance_squared(transform.translation)
                    < flock_detection_range_squared
                {
                    // close, avoid
                    Some((transform.translation - other_transform.translation).normalize())
                } else {
                    None
                }
            })
            .filter(|vec| !vec.is_nan())
            .fold((0.0f32, Vec2::ZERO), |(count, acc), vec| {
                (count + 1.0, acc + vec.xy())
            });
        if count == 0.0 {
            return Vec2::ZERO;
        }
        separation / count
    }
    #[instrument(skip_all)]
    fn update_separation<'a>(
        &mut self,
        transform: &Transform,
        transforms: impl Iterator<Item = &'a Transform>,
    ) {
        let separation = self.calculate_separation(transform, transforms);
        tracing::trace!(?separation);
        self.current_separation = separation;
    }
}

impl Default for Boid {
    fn default() -> Self {
        Self {
            separation_speed: 64.,
            flock_detection_range: 20.,
            current_separation: default(),
        }
    }
}

#[derive(Resource, Debug)]
pub enum BoidSeparationUpdateRate {
    PerFrame,
    Rate(f32),
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub enum EnemyClass {
    // will go towards the player
    #[default]
    Melee,
    // will keep its distance
    Ranged {
        max_range: f32,
    },
}

fn boids_calculate_separation(
    mut query: Query<(&Transform, &EnemyClass, &mut Boid), With<Enemy>>,
    time: Res<Time>,
    mut tick_time: Local<f32>,
    update_rate: Res<BoidSeparationUpdateRate>,
) {
    match *update_rate {
        BoidSeparationUpdateRate::PerFrame => {}
        BoidSeparationUpdateRate::Rate(rate) => {
            let dt = time.delta_secs();
            *tick_time += dt;
            if *tick_time > rate {
                *tick_time -= rate;
            } else {
                return;
            }
        }
    }
    // par iter cause boids are expensive
    // TODO: spread calculations across frames
    let enemies = query
        .iter()
        .map(|el| (*el.0, *el.1, *el.2))
        .collect::<Arc<[_]>>();
    query.par_iter_mut().for_each(|(transform, _, mut boid)| {
        boid.update_separation(transform, enemies.iter().map(|(transform, _, _)| transform));
    });
}

fn boids_move_towards_player(
    mut query: Query<(&mut Velocity, &Transform, &EnemyClass, &mut Boid, &Speed), With<Enemy>>,
    player_transform: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut velocity, transform, enemy_class, boid, speed) in query.iter_mut() {
        // apply separation
        velocity.linvel += dt * boid.current_separation * boid.separation_speed;
        let distance_to_player = player_transform.translation.distance(transform.translation);
        let mut move_towards_player = || {
            let dir_to_player = (player_transform.translation - transform.translation).normalize();
            velocity.linvel = velocity
                .linvel
                .move_towards(dir_to_player.xy() * speed.0, dt * speed.0 * 2.0);
        };
        match enemy_class {
            EnemyClass::Melee => {
                move_towards_player();
            }
            EnemyClass::Ranged { max_range } if distance_to_player < *max_range => {
                move_towards_player();
            }
            EnemyClass::Ranged { .. } => {}
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct PlayerHitEvent {
    pub damage: Damage,
}

impl PlayerHitEvent {
    pub fn new(damage: Damage) -> Self {
        Self { damage }
    }
}

#[instrument(skip_all)]
fn enemy_check_for_player_collisions(
    mut events: EventReader<CollisionEvent>,
    mut hit_events: EventWriter<PlayerHitEvent>,
    hitboxes: Query<&Damage, With<EnemyHitbox>>,
    player: Single<Entity, With<Player>>,
) {
    for event in events.read() {
        tracing::trace!(?event);
        if let CollisionEvent::Started(entity_1, entity_2, _) = *event {
            let (_, enemy_id) = if entity_1 == *player {
                (entity_1, entity_2)
            } else if entity_2 == *player {
                (entity_2, entity_1)
            } else {
                continue;
            };

            if let Ok(damage) = hitboxes.get(enemy_id) {
                hit_events.write(PlayerHitEvent::new(*damage));
            };
        }
    }
}

#[derive(Event, Debug)]
pub struct EnemyHitEvent(Entity, Transform);

#[derive(Event, Debug)]
pub struct EnemyDiedEvent(Entity);

#[instrument(skip_all)]
fn enemies_take_damage(
    mut enemies: Query<(Entity, &mut Health, &CollidingEntities), With<EnemyHurtbox>>,
    hitboxes: Query<(&Damage, &Transform), With<Hitbox>>,
    mut hit_events: EventWriter<EnemyHitEvent>,
    mut died_events: EventWriter<EnemyDiedEvent>,
) {
    for (enemy, mut health, colliding_entities) in enemies.iter_mut() {
        for hitbox in colliding_entities.iter() {
            let Ok((damage, transform)) = hitboxes.get(hitbox) else {
                continue;
            };

            **health -= **damage;
            hit_events.write(EnemyHitEvent(enemy, *transform));
            if **health <= 0 {
                died_events.write(EnemyDiedEvent(enemy));
            }
        }
    }
}

fn handle_enemy_died_events(
    mut events: EventReader<EnemyDiedEvent>,
    mut commands: Commands,
    mut shake: Single<&mut Shake>,
) {
    for EnemyDiedEvent(enemy) in events.read() {
        // TODO: Some effects
        commands.entity(*enemy).despawn();
        shake.apply_trauma(0.25);
    }
}

#[derive(Resource, AssetCollection)]
struct EnemyAssets {
    #[asset(path = "enemies/hit.particles.ron")]
    hit_particles: Handle<Particle2dEffect>,
    #[asset(path = "enemies/hit_particle.png")]
    hit_particles_texture: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
struct HitParticleMaterial(Handle<SpriteParticle2dMaterial>);

fn setup_hit_particle_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    assets: Res<EnemyAssets>,
) {
    commands.insert_resource(HitParticleMaterial(materials.add(
        SpriteParticle2dMaterial::new(assets.hit_particles_texture.clone(), 1, 1),
    )));
}

#[instrument(err, skip_all)]
fn handle_enemy_hit_events(
    mut events: EventReader<EnemyHitEvent>,
    mut commands: Commands,
    assets: Res<EnemyAssets>,
    mut query: Query<&Transform, With<Enemy>>,
    mut shake: Single<&mut Shake>,
    hit_particles_material: Res<HitParticleMaterial>,
) -> Result {
    for EnemyHitEvent(enemy, hitbox_transform) in events.read() {
        let Ok(enemy_transform) = query.get(*enemy) else {
            continue;
        };
        let from_hitbox = (enemy_transform.translation - hitbox_transform.translation).normalize();
        commands
            .spawn((
                ParticleSpawner(hit_particles_material.clone()),
                ParticleEffectHandle(assets.hit_particles.clone()),
                OneShot::Despawn,
            ))
            .insert(
                Transform::from_translation(enemy_transform.translation)
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, from_hitbox.xy().to_angle())),
            );
        shake.apply_trauma(0.1);
    }
    Ok(())
}

pub fn despawn_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for enemy in query.iter() {
        commands.entity(enemy).try_despawn();
    }
}
