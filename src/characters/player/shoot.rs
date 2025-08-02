use std::f32::consts::PI;

use crate::ShakeExt;
use crate::autotimer::prelude::*;
use crate::effects::prelude::*;
use crate::{audio::prelude::*, exp_decay::ExpDecay};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};
use bevy_enoki::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_trauma_shake::Shake;
use leafwing_input_manager::prelude::ActionState;
use rand::Rng;
use tracing::instrument;

use crate::characters::{player::PlayerHitbox, prelude::*};
use crate::{
    COLORS,
    characters::{
        AimDir, PLAYER_HITBOX_GROUP,
        bullet::{BulletMaxWrap, BulletWrapCount, bullet_base},
        player::{Player, PlayerAction, PlayerAssets},
    },
    screens::GameScreen,
};

pub fn player_shoot_plugin(app: &mut App) {
    app.add_plugins(ghost_sprite_plugin::<PlayerBoomerangGhostSprite>)
        .add_plugins(Material2dPlugin::<PlayerBoomerangMaterial>::default())
        .add_systems(Startup, setup_boomerang_mesh)
        .add_systems(
            FixedUpdate,
            (spin_boomerangs, boomerang_fly).run_if(in_state(GameScreen::Gameplay)),
        )
        .add_systems(
            FixedUpdate,
            (
                player_shoot_system,
                boomerang_activate_after_wrap,
                boomerang_activate_effects.after(boomerang_fly),
                boomerang_material_update,
                boomerang_material_update_no_damage,
                fade_out_boomerang_ghosts,
            )
                .chain()
                .run_if(in_state(GameScreen::Gameplay))
                .after(setup_boomerang_mesh),
        )
        .add_systems(
            PostUpdate,
            (
                boomerang_destroy_on_contact,
                boomerang_destroy_close_to_player,
            )
                .run_if(in_state(GameScreen::Gameplay)),
        );
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerShoot {
    pub rate: f32,
    pub spread: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerBoomerang {
    traveled: f32,
    max_distance: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct PlayerBoomerangGhostSprite;

impl PlayerBoomerang {
    fn new(max_distance: f32) -> Self {
        Self {
            max_distance,
            ..default()
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct BoomerangMesh(Handle<Mesh>);

fn setup_boomerang_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(BoomerangMesh(meshes.add(Rectangle::new(16.0, 16.0))));
}

fn player_shoot_system(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<PlayerBoomerangMaterial>>,
    meshes: Res<BoomerangMesh>,
    player: Single<(&Transform, &AimDir, &PlayerShoot), With<Player>>,
    action_state: Res<ActionState<PlayerAction>>,
    mut cooldown: Local<f32>,
    player_assets: Res<PlayerAssets>,
    audio: Res<Audio>,
    volume: Res<VolumeSettings>,
    mut shoot_timer: Local<AutoTimer<100, TimerRepeating>>,
    mut shake: Single<&mut Shake>,
) {
    let dt = time.delta_secs();
    let mesh_handle = meshes.clone();
    let (transform, &aim_dir, &PlayerShoot { rate, spread }) = player.into_inner();
    if action_state.pressed(&PlayerAction::Shoot) {
        *cooldown -= dt;
        shoot_timer.tick(time.delta());
        if shoot_timer.just_finished() {
            audio
                .play(player_assets.shoot_sound.clone())
                .with_volume(volume.calc_sfx(1.0));
        }
        if *cooldown <= 0.0 {
            *cooldown += rate;
            // shoot
            let half_spread = spread / 2.0;
            let angle = rand::rng().random_range(-half_spread..half_spread);
            let material = materials.add(PlayerBoomerangMaterial {
                color_amount: 0.0,
                color: COLORS[2].into(),
                disabled_color: COLORS[4].with_alpha(0.8).into(),
                base_sampler: player_assets.boomerang_sprite.clone(),
            });
            let direction = Vec2::from_angle(aim_dir.to_angle() + angle);
            shake.apply_trauma(0.1);
            commands
                .spawn(bullet_base(2.0))
                // .remove::<BulletLifetime>()
                .insert(BulletMaxWrap(1))
                .insert(PlayerBoomerang::new(64.))
                .insert(Mesh2d(mesh_handle))
                .insert(BoomerangMaterialId(material.id()))
                .insert(MeshMaterial2d(material))
                .insert(Collider::ball(4.0))
                .insert(Sensor)
                .insert(CollidingEntities::default())
                .insert(CollisionGroups::new(
                    PLAYER_HITBOX_GROUP,
                    ENEMY_HURTBOX_GROUP,
                ))
                .insert(PlayerHitbox)
                .insert(Transform::from_translation(
                    transform.translation + aim_dir.extend(0.0) * 8.0 + vec3(0.0, 8.0, 4.0),
                ))
                .insert(Velocity {
                    linvel: direction * 200.0,
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

fn boomerang_activate_after_wrap(
    query: Query<(Entity, &BulletWrapCount), With<PlayerBoomerang>>,
    mut commands: Commands,
) {
    for (boomerang, wrap_count) in query.iter() {
        if **wrap_count > 0 {
            commands.entity(boomerang).insert_if_new(Damage(1));
        }
    }
}

fn boomerang_fly(
    time: Res<Time>,
    commands: ParallelCommands,
    mut query: Query<(Entity, &mut PlayerBoomerang, &mut Velocity, &Transform)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let dt = time.delta_secs();
    query
        .par_iter_mut()
        .for_each(|(boomerang_id, mut boomerang, mut velocity, transform)| {
            let speed = velocity.linvel.length();
            boomerang.traveled += speed * dt;
            if boomerang.traveled > boomerang.max_distance {
                // go towards player
                let to_player = (player_transform.translation - transform.translation).normalize();
                velocity.linvel = to_player.xy() * speed;
                commands.command_scope(|mut commands| {
                    commands.entity(boomerang_id).insert_if_new(Damage(1));
                })
            }
        });
}

fn boomerang_activate_effects(
    mut commands: Commands,
    assets: Res<PlayerAssets>,
    query: Query<(Entity, &Transform), (With<PlayerBoomerang>, Added<Damage>)>,
) {
    for (boomerang, transform) in query.iter() {
        commands.spawn((
            ParticleSpawner::default(),
            ParticleEffectHandle(assets.boomerang_activation_particles.clone()),
            Transform::from_translation(transform.translation.with_z(10.0)),
            OneShot::Despawn,
        ));
        // TODO: play sounds
        commands.entity(boomerang).insert((
            GhostSpriteSpawnerGeneric::<PlayerBoomerangGhostSprite>::builder()
                .kind(GhostSpriteSpawnerKind::Infinite)
                .rate(0.02)
                .ghost_decay(16.0)
                .build(),
        ));
    }
}

fn boomerang_destroy_close_to_player(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut PlayerBoomerang,
        &Transform,
        Option<&GhostSpriteSpawnerGeneric>,
    )>,
    player_transform: Single<&Transform, With<Player>>,
) {
    for (boomerang_id, boomerang, transform, spawner) in query.iter_mut() {
        if boomerang.traveled > boomerang.max_distance
            && transform
                .translation
                .distance_squared(player_transform.translation)
                < 256.0
        {
            commands.entity(boomerang_id).despawn();
            if let Some(spawner) = spawner {
                commands.spawn(spawner.clone());
            }
        }
    }
}

#[instrument(skip_all)]
fn boomerang_destroy_on_contact(
    mut enemies: Query<(Entity, &CollidingEntities), (With<PlayerBoomerang>, With<Damage>)>,
    mut spawners: Query<&GhostSpriteSpawnerGeneric, With<PlayerBoomerang>>,
    mut commands: Commands,
) {
    for (boomerang, colliding_entities) in enemies.iter_mut() {
        if !colliding_entities.is_empty() {
            commands.entity(boomerang).try_despawn();
            if let Ok(spawner) = spawners.get(boomerang) {
                commands.spawn(spawner.clone());
            }
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct PlayerBoomerangMaterial {
    #[uniform(0)]
    color_amount: f32,
    #[uniform(1)]
    color: LinearRgba,
    #[texture(2)]
    #[sampler(3)]
    base_sampler: Handle<Image>,
    #[uniform(4)]
    disabled_color: LinearRgba,
}

impl Material2d for PlayerBoomerangMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/player/boomerang.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct BoomerangMaterialId(AssetId<PlayerBoomerangMaterial>);

fn boomerang_material_update(
    mut materials: ResMut<Assets<PlayerBoomerangMaterial>>,
    mut query: Query<(&BoomerangMaterialId, &Damage), With<PlayerBoomerang>>,
) {
    for (material, damage) in query.iter_mut() {
        if **damage < 1 {
            continue;
        }
        if let Some(material) = materials.get_mut(**material) {
            material.color_amount = 1.0;
        }
    }
}

fn boomerang_material_update_no_damage(
    mut materials: ResMut<Assets<PlayerBoomerangMaterial>>,
    mut query: Query<&BoomerangMaterialId, (With<PlayerBoomerang>, Without<Damage>)>,
) {
    for material in query.iter_mut() {
        if let Some(material) = materials.get_mut(**material) {
            material.color_amount = 0.0;
        }
    }
}

fn fade_out_boomerang_ghosts(
    mut commands: Commands,
    mut materials: ResMut<Assets<PlayerBoomerangMaterial>>,
    ghosts: Query<(
        Entity,
        &GhostSpriteGeneric<PlayerBoomerangGhostSprite>,
        Option<&BoomerangMaterialId>,
    )>,
    player_assets: Res<PlayerAssets>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (ghost, GhostSpriteGeneric { decay, .. }, material) in ghosts.iter() {
        let id = match material {
            Some(BoomerangMaterialId(id)) => *id,
            None => {
                let material = materials.add(PlayerBoomerangMaterial {
                    color_amount: 1.0,
                    color: COLORS[2].with_alpha(0.8).into(),
                    disabled_color: COLORS[4].with_alpha(0.8).into(),
                    base_sampler: player_assets.boomerang_sprite.clone(),
                });
                let id = material.id();
                commands
                    .entity(ghost)
                    .insert(BoomerangMaterialId(id))
                    .insert(MeshMaterial2d(material));
                id
            }
        };
        let material = materials.get_mut(id).expect("error adding material asset");
        let alpha = material.color.alpha().exp_decay(0.0, *decay, dt);
        material.color.set_alpha(alpha);
        if alpha < 0.005 {
            commands.entity(ghost).try_despawn();
        }
    }
}
