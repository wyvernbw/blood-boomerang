use std::time::Duration;

use bevy::prelude::*;
use bon::Builder;
use rand::Rng;

use crate::characters::enemies::ghost::prelude::*;
use crate::characters::enemies::prelude::*;
use crate::characters::player::prelude::*;
use crate::characters::prelude::*;
use crate::screens::prelude::*;

pub fn gameplay_plugin(app: &mut App) {
    app.init_resource::<CurrentWave>()
        .init_resource::<CurrentWaveTime>()
        .add_event::<SpawnWaveEvent>()
        .add_plugins(characters_plugin().screen(GameScreen::Gameplay).call())
        .add_systems(OnEnter(GameScreen::Gameplay), (spawn_player, reset_wave))
        .add_systems(
            OnExit(GameScreen::Gameplay),
            (despawn_player, despawn_enemies),
        )
        .add_systems(
            Update,
            (spawn_waves, spawn_wave_event_loop.after(spawn_waves))
                .run_if(in_state(GameScreen::Gameplay)),
        );
}

#[derive(Resource, DerefMut, Deref, Default)]
pub struct CurrentWave(usize);

#[derive(Resource, DerefMut, Deref, Default)]
pub struct CurrentWaveTime(Duration);

#[derive(Builder, Debug, Clone)]
#[builder(const)]
struct Wave {
    timestamp: Duration,
    #[builder(default = 0)]
    ghost_count: usize,
}

const WAVES: &[Wave] = &[
    Wave::builder()
        .timestamp(Duration::from_secs(2))
        .ghost_count(3)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(5))
        .ghost_count(10)
        .build(),
];

fn random_point_on_rectangle_perimeter(center: Vec2, width: f32, height: f32) -> Vec2 {
    let mut rng = rand::rng();

    let perimeter = 2.0 * (width + height);

    let p = rng.random_range(0.0..perimeter);

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    if p < width {
        // Top edge: left to right
        Vec2::new(center.x - half_width + p, center.y + half_height)
    } else if p < width + height {
        // Right edge: top to bottom
        Vec2::new(center.x + half_width, center.y + half_height - (p - width))
    } else if p < 2.0 * width + height {
        // Bottom edge: right to left
        Vec2::new(
            center.x + half_width - (p - width - height),
            center.y - half_height,
        )
    } else {
        // Left edge: bottom to top
        Vec2::new(
            center.x - half_width,
            center.y - half_height + (p - 2.0 * width - height),
        )
    }
}

fn rand_on_screen_outline() -> Vec2 {
    random_point_on_rectangle_perimeter(Vec2::ZERO, RES_WIDTH as f32, RES_HEIGHT as f32)
}

fn spawn_waves(
    time: Res<Time>,
    mut current_wave: ResMut<CurrentWave>,
    mut current_wave_time: ResMut<CurrentWaveTime>,
    mut events: EventWriter<SpawnWaveEvent>,
) {
    let Some(wave) = WAVES.get(**current_wave) else {
        return;
    };
    **current_wave_time += time.delta();
    if **current_wave_time > wave.timestamp {
        events.write(SpawnWaveEvent(**current_wave));
        **current_wave += 1;
    }
}

#[derive(Event, Debug, Deref, DerefMut, Clone)]
struct SpawnWaveEvent(usize);

fn spawn_wave_event_loop(
    mut commands: Commands,
    mut events: EventReader<SpawnWaveEvent>,
    ghost_assets: Res<GhostAssets>,
) {
    for event in events.read() {
        if let Some(wave) = WAVES.get(**event) {
            for _ in 0..(wave.ghost_count) {
                let pos = rand_on_screen_outline();
                spawn_ghost()
                    .commands(&mut commands)
                    .assets(&ghost_assets)
                    .call()
                    .insert(Transform::from_translation(pos.extend(0.0)));
            }
        }
    }
}

fn reset_wave(mut wave: ResMut<CurrentWave>, mut wave_time: ResMut<CurrentWaveTime>) {
    **wave = 0;
    **wave_time = Duration::ZERO;
}
