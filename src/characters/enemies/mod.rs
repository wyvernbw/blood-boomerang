use std::sync::Arc;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::characters::Speed;
use crate::characters::character_base;
use crate::characters::enemies::ghost::prelude::*;
use crate::characters::player::Player;
use crate::screens::GameScreen;

pub mod prelude {
    pub use super::enemies_plugin;
    pub use super::enemy_base;
    pub use super::{Enemy, EnemyClass};
}

pub mod ghost;

pub fn enemies_plugin(app: &mut App) {
    app.insert_resource(BoidSeparationUpdateRate::PerFrame)
        .add_plugins(ghost_plugin)
        .add_systems(
            Update,
            (
                boids_calculate_separation,
                boids_move_towards_player.after(boids_calculate_separation),
            )
                .run_if(in_state(GameScreen::Gameplay)),
        );
}

#[derive(Component, Debug)]
#[require(Boid, EnemyClass)]
pub struct Enemy;

pub fn enemy_base() -> impl Bundle {
    (character_base(), Enemy)
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
        transform: &GlobalTransform,
        transforms: impl Iterator<Item = &'a GlobalTransform>,
    ) -> Vec2 {
        let flock_detection_range_squared = self.flock_detection_range * self.flock_detection_range;
        let (count, separation) = transforms
            .filter_map(|other_transform| {
                if other_transform
                    .translation()
                    .distance_squared(transform.translation())
                    < flock_detection_range_squared
                {
                    // close, avoid
                    Some(-(transform.translation() - other_transform.translation()).normalize())
                } else {
                    None
                }
            })
            .fold((0.0f32, Vec2::ZERO), |(count, acc), vec| {
                (count + 1.0, acc + vec.xy())
            });
        separation / count
    }
    fn update_separation<'a>(
        &mut self,
        transform: &GlobalTransform,
        transforms: impl Iterator<Item = &'a GlobalTransform>,
    ) {
        let separation = self.calculate_separation(transform, transforms);
        self.current_separation = separation;
    }
}

impl Default for Boid {
    fn default() -> Self {
        Self {
            separation_speed: 32.,
            flock_detection_range: 8.,
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
    mut query: Query<(&GlobalTransform, &EnemyClass, &mut Boid), With<Enemy>>,
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
    mut query: Query<
        (
            &mut Velocity,
            &GlobalTransform,
            &EnemyClass,
            &mut Boid,
            &Speed,
        ),
        With<Enemy>,
    >,
    player_transform: Single<&GlobalTransform, With<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut velocity, transform, enemy_class, boid, speed) in query.iter_mut() {
        // apply separation
        velocity.linvel += dt * boid.current_separation * boid.separation_speed;
        let distance_to_player = player_transform
            .translation()
            .distance(transform.translation());
        let mut move_towards_player = || {
            let dir_to_player =
                (player_transform.translation() - transform.translation()).normalize();
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
