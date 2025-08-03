use std::time::Duration;

use bevy::prelude::*;
use bon::Builder;
use rand::Rng;

use crate::characters::enemies::coffin::prelude::*;
use crate::characters::enemies::ghost::{CommandsGhost, prelude::*};
use crate::characters::enemies::hand::{CommandsHand, Hand, HandArgs, HandAssets};
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
    #[builder(default = 0)]
    coffin_count: usize,
    #[builder(default = 0)]
    hand_count: usize,
}

// const WAVES: &[Wave] = &[
//     Wave::builder()
//         .timestamp(Duration::from_secs(2))
//         .ghost_count(3)
//         .build(),
//     Wave::builder()
//         .timestamp(Duration::from_secs(6))
//         // .ghost_count(5)
//         .coffin_count(1)
//         .build(),
//     Wave::builder()
//         .timestamp(Duration::from_secs(9))
//         // .ghost_count(5)
//         .coffin_count(2)
//         .hand_count(1)
//         .build(),
// ];

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
    random_point_on_rectangle_perimeter(Vec2::ZERO, RES_WIDTH as f32 + 16., RES_HEIGHT as f32 + 16.)
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
    coffin_assets: Res<CoffinAssets>,
    hand_assets: Res<HandAssets>,
) {
    for event in events.read() {
        if let Some(wave) = WAVES.get(**event) {
            for _ in 0..(wave.ghost_count) {
                let pos = rand_on_screen_outline();
                commands
                    .spawn_ghost(GhostArgs::builder().assets(&ghost_assets).build())
                    .insert(Transform::from_translation(pos.extend(0.0)));
            }
            for _ in 0..wave.coffin_count {
                let pos = rand_on_screen_outline();
                commands
                    .spawn_coffin(
                        CoffinArgs::builder()
                            .assets(&coffin_assets)
                            .coffin(
                                Coffin::builder()
                                    .initial_rate(Duration::from_secs_f32(1.0))
                                    .rate(Duration::from_secs_f32(5.0))
                                    .count(5)
                                    .spacing(32.0)
                                    .build(),
                            )
                            .build(),
                    )
                    .insert(Transform::from_translation(pos.extend(0.0)));
            }
            for _ in 0..wave.hand_count {
                let pos = rand_on_screen_outline();
                commands
                    .spawn_hand(
                        HandArgs::builder()
                            .assets(&hand_assets)
                            .hand(
                                Hand::builder()
                                    .shoot_rate(Duration::from_secs_f32(1.5))
                                    .build(),
                            )
                            .build(),
                    )
                    .insert(Transform::from_translation(pos.extend(0.0)));
            }
        }
    }
}

fn reset_wave(mut wave: ResMut<CurrentWave>, mut wave_time: ResMut<CurrentWaveTime>) {
    **wave = 0;
    **wave_time = Duration::ZERO;
}

const WAVES: &[Wave] = &[
    // Initial easy waves
    Wave::builder()
        .timestamp(Duration::from_secs(2))
        .ghost_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(5))
        .ghost_count(3)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(8))
        .ghost_count(4)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(11))
        .ghost_count(5)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(14))
        .ghost_count(6)
        .build(),
    // Begin mixing in coffins
    Wave::builder()
        .timestamp(Duration::from_secs(17))
        .ghost_count(6)
        .coffin_count(1)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(20))
        .ghost_count(7)
        .coffin_count(1)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(23))
        .ghost_count(7)
        .coffin_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(26))
        .ghost_count(8)
        .coffin_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(29))
        .ghost_count(8)
        .coffin_count(3)
        .build(),
    // Introduce hands
    Wave::builder()
        .timestamp(Duration::from_secs(32))
        .ghost_count(9)
        .coffin_count(3)
        .hand_count(1)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(35))
        .ghost_count(9)
        .coffin_count(3)
        .hand_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(38))
        .ghost_count(10)
        .coffin_count(3)
        .hand_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(41))
        .ghost_count(10)
        .coffin_count(4)
        .hand_count(2)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(44))
        .ghost_count(11)
        .coffin_count(4)
        .hand_count(3)
        .build(),
    // Ramp up faster
    Wave::builder()
        .timestamp(Duration::from_secs(47))
        .ghost_count(12)
        .coffin_count(4)
        .hand_count(3)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(50))
        .ghost_count(12)
        .coffin_count(5)
        .hand_count(3)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(53))
        .ghost_count(13)
        .coffin_count(5)
        .hand_count(4)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(56))
        .ghost_count(14)
        .coffin_count(5)
        .hand_count(4)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(59))
        .ghost_count(14)
        .coffin_count(6)
        .hand_count(5)
        .build(),
    // Harder section
    Wave::builder()
        .timestamp(Duration::from_secs(62))
        .ghost_count(15)
        .coffin_count(6)
        .hand_count(5)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(65))
        .ghost_count(16)
        .coffin_count(6)
        .hand_count(6)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(68))
        .ghost_count(16)
        .coffin_count(7)
        .hand_count(6)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(71))
        .ghost_count(17)
        .coffin_count(7)
        .hand_count(6)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(74))
        .ghost_count(18)
        .coffin_count(7)
        .hand_count(7)
        .build(),
    // Very challenging waves
    Wave::builder()
        .timestamp(Duration::from_secs(77))
        .ghost_count(18)
        .coffin_count(8)
        .hand_count(7)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(80))
        .ghost_count(19)
        .coffin_count(8)
        .hand_count(8)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(83))
        .ghost_count(20)
        .coffin_count(8)
        .hand_count(8)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(86))
        .ghost_count(20)
        .coffin_count(9)
        .hand_count(8)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(89))
        .ghost_count(21)
        .coffin_count(9)
        .hand_count(9)
        .build(),
    // Continue up to wave 50
    Wave::builder()
        .timestamp(Duration::from_secs(92))
        .ghost_count(22)
        .coffin_count(9)
        .hand_count(9)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(95))
        .ghost_count(22)
        .coffin_count(10)
        .hand_count(9)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(98))
        .ghost_count(23)
        .coffin_count(10)
        .hand_count(10)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(101))
        .ghost_count(24)
        .coffin_count(10)
        .hand_count(10)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(104))
        .ghost_count(25)
        .coffin_count(10)
        .hand_count(10)
        .build(),
    // Climactic finale waves
    Wave::builder()
        .timestamp(Duration::from_secs(107))
        .ghost_count(25)
        .coffin_count(11)
        .hand_count(11)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(110))
        .ghost_count(26)
        .coffin_count(11)
        .hand_count(12)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(113))
        .ghost_count(26)
        .coffin_count(12)
        .hand_count(12)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(116))
        .ghost_count(27)
        .coffin_count(12)
        .hand_count(13)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(119))
        .ghost_count(28)
        .coffin_count(12)
        .hand_count(14)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(122))
        .ghost_count(28)
        .coffin_count(13)
        .hand_count(14)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(125))
        .ghost_count(29)
        .coffin_count(13)
        .hand_count(15)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(128))
        .ghost_count(30)
        .coffin_count(13)
        .hand_count(15)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(131))
        .ghost_count(30)
        .coffin_count(14)
        .hand_count(16)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(134))
        .ghost_count(31)
        .coffin_count(14)
        .hand_count(17)
        .build(),
    Wave::builder()
        .timestamp(Duration::from_secs(137))
        .ghost_count(32)
        .coffin_count(15)
        .hand_count(18)
        .build(),
];
