use bevy::{prelude::*, window::WindowBackendScaleFactorChanged};
use leafwing_input_manager::prelude::*;
use moonshine_save::prelude::*;
use tracing::instrument;

use crate::{
    COLORS,
    characters::player::{despawn_player, spawn_player},
    screens::{GameScreen, prelude::InGameCamera},
};

pub mod prelude {}

pub fn tutorial_plugin(app: &mut App) {
    app.register_type::<Tutorial>()
        .register_type::<TutorialState>()
        .add_observer(save_on_default_event)
        .add_observer(load_on_default_event)
        .add_plugins(InputManagerPlugin::<TutorialScreenConfirm>::default())
        .add_systems(OnEnter(GameScreen::SplashFirst), load_save)
        .add_systems(OnExit(GameScreen::Tutorial), finish_tutorial)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Tutorial(TutorialState::Active));
        })
        .add_systems(
            Update,
            (advance_state_if_tutorial_done, check_tutorial_confirm)
                .run_if(in_state(GameScreen::Tutorial)),
        )
        .add_systems(
            OnEnter(GameScreen::Tutorial),
            (spawn_player, spawn_tutorial_text),
        )
        .add_systems(
            OnExit(GameScreen::Tutorial),
            (despawn_player, despawn_tutorial_scene),
        );
}

#[derive(Component, Reflect, Deref, DerefMut)]
#[require(Save, Unload)]
struct Tutorial(TutorialState);

#[derive(Default, Reflect, PartialEq, Eq)]
enum TutorialState {
    #[default]
    Active,
    Done,
}

fn load_save(mut commands: Commands) {
    commands.trigger_load(LoadWorld::default_from_file("save.ron"));
}

fn finish_tutorial(mut commands: Commands, mut tutorial: Single<&mut Tutorial>) {
    **tutorial = Tutorial(TutorialState::Done);
    commands.trigger_save(SaveWorld::default_into_file("save.ron"));
}

fn advance_state_if_tutorial_done(
    tutorial: Single<(Entity, &Tutorial)>,
    mut next: ResMut<NextState<GameScreen>>,
) {
    let (entity, tutorial) = tutorial.into_inner();
    if tutorial.0 == TutorialState::Done {
        next.set(GameScreen::Gameplay);
    }
}

#[derive(Component)]
struct TutorialScene;

fn spawn_tutorial_text(mut commands: Commands, camera: Single<Entity, With<InGameCamera>>) {
    let font = TextFont {
        font_size: 10.0,
        ..default()
    };
    commands.spawn((
        TutorialScene,
        TutorialScreenConfirm::input_map(),
        UiTargetCamera(*camera),
        font.clone(),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(2.0)),
            row_gap: Val::Px(2.0),
            ..default()
        },
        children![
            (
                Text::new("wasd OR left stick OR dpad to move"),
                TextColor(COLORS[3]),
                font.clone(),
            ),
            (
                Text::new("mouse OR right stick to aim"),
                TextColor(COLORS[3]),
                font.clone(),
            ),
            (
                Text::new("left click OR right bumper to shoot"),
                TextColor(COLORS[3]),
                font.clone(),
            ),
            (
                Text::new("space OR left trigger to dash"),
                TextColor(COLORS[3]),
                font.clone(),
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
                Text::new("Remember, horrid aberrations await."),
                TextColor(COLORS[3]),
                font.clone(),
            ),
            (
                Text::new("press ENTER or R3 to PLAY"),
                TextColor(COLORS[2]),
                font.clone(),
            )
        ],
    ));
}

#[derive(Component, Actionlike, Reflect, Clone, PartialEq, Eq, Hash, Debug)]
struct TutorialScreenConfirm;

impl TutorialScreenConfirm {
    fn input_map() -> InputMap<TutorialScreenConfirm> {
        InputMap::default()
            .with(TutorialScreenConfirm, KeyCode::Enter)
            .with(TutorialScreenConfirm, GamepadButton::C)
            .with(TutorialScreenConfirm, GamepadButton::RightThumb)
    }
}

#[instrument(skip_all)]
fn check_tutorial_confirm(
    query: Single<&ActionState<TutorialScreenConfirm>, With<TutorialScene>>,
    mut tutorial: Single<&mut Tutorial>,
) {
    if query.pressed(&TutorialScreenConfirm) {
        ***tutorial = TutorialState::Done;
        tracing::info!("tutorial confirm!");
    }
}

fn despawn_tutorial_scene(mut commands: Commands, scene: Single<Entity, With<TutorialScene>>) {
    commands.entity(*scene).despawn();
}
