use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use rand::Rng;

use crate::{
    characters::{
        AimDir,
        bullet::{BulletMaxWrap, BulletWrapCount, bullet_base},
        player::{Player, PlayerAction, PlayerAssets},
    },
    screens::GameScreen,
};

pub fn player_shoot_plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<PlayerBoomerangMaterial>::default())
        .add_systems(Startup, setup_boomerang_mesh)
        .add_systems(
            Update,
            (
                player_shoot_system,
                spin_boomerangs,
                boomerang_material_update,
            )
                .run_if(in_state(GameScreen::Gameplay))
                .after(setup_boomerang_mesh),
        );
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerShoot {
    pub rate: f32,
    pub spread: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerBoomerang;

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
) {
    let dt = time.delta_secs();
    let mesh_handle = meshes.clone();
    let (transform, &aim_dir, &PlayerShoot { rate, spread }) = player.into_inner();
    if action_state.pressed(&PlayerAction::Shoot) {
        *cooldown -= dt;
        if *cooldown <= 0.0 {
            *cooldown += rate;
            // shoot
            let half_spread = spread / 2.0;
            let angle = rand::rng().random_range(-half_spread..half_spread);
            let material = materials.add(PlayerBoomerangMaterial {
                color_amount: 0.0,
                color: LinearRgba::rgb(0.0, 0.6, 0.6),
                disabled_color: LinearRgba::rgb(0.1, 0.1, 0.1),
                base_sampler: player_assets.boomerang_sprite.clone(),
            });
            commands
                .spawn(bullet_base(4.0))
                .insert(BulletMaxWrap(1))
                .insert(PlayerBoomerang)
                .insert(Mesh2d(mesh_handle))
                .insert(BoomerangMaterialId(material.id()))
                .insert(MeshMaterial2d(material))
                .insert(Collider::ball(4.0))
                .insert(Sensor)
                .insert(Transform::from_translation(
                    transform.translation + aim_dir.extend(0.0) * 8.0 + vec3(0.0, 8.0, 0.0),
                ))
                .insert(Velocity {
                    linvel: Vec2::from_angle(aim_dir.to_angle() + angle) * 200.0,
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
    mut query: Query<(&BoomerangMaterialId, &BulletWrapCount), With<PlayerBoomerang>>,
) {
    for (material, count) in query.iter_mut() {
        if **count < 1 {
            continue;
        }
        if let Some(material) = materials.get_mut(**material) {
            material.color_amount = 1.0;
        }
    }
}
