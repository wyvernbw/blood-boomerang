use crate::COLORS;
use crate::autotimer::prelude::*;
use crate::characters::enemies::prelude::*;
use crate::characters::player::prelude::*;
use crate::screens::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use tracing::instrument;

pub mod prelude {
    pub use super::after_death_plugin;
}

pub fn after_death_plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<DeathScreenAction>::default())
        .add_systems(OnEnter(GameScreen::AfterDeath), spawn_black_screen)
        .add_systems(OnExit(GameScreen::AfterDeath), despawn_black_screen)
        .add_systems(
            Update,
            (spawn_black_screen_ui, input_system).run_if(in_state(GameScreen::AfterDeath)),
        );
}

type TextDelayTimer = AutoTimer<500, TimerOnce>;

#[derive(Component)]
#[require(TextDelayTimer)]
struct BlackScreen;

const MARGIN: Val = Val::Px(4.0);

fn spawn_black_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            // fill the entire window
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(MARGIN),
            row_gap: MARGIN,
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        BlackScreen,
        Transform::from_xyz(0.0, 0.0, 100.0),
        DeathScreenAction::default_input_map(),
    ));
}

#[instrument(skip_all)]
fn spawn_black_screen_ui(
    mut commands: Commands,
    time: Res<Time>,
    black_screen: Single<(Entity, &mut TextDelayTimer), With<BlackScreen>>,
) {
    let (black_screen, mut text_delay) = black_screen.into_inner();
    text_delay.tick(time.delta());
    if !text_delay.just_finished() {
        return;
    }
    tracing::info!("spawning ui");
    commands.entity(black_screen).insert(children![
        (Text::new("DIED"), TextColor(COLORS[3])),
        (Text::new("Press ENTER to restart."), TextColor(COLORS[4]))
    ]);
}

#[derive(Actionlike, Debug, Reflect, PartialEq, Eq, Clone, Copy, Hash)]
enum DeathScreenAction {
    Continue,
}

impl DeathScreenAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(DeathScreenAction::Continue, KeyCode::Enter)
            .with(DeathScreenAction::Continue, GamepadButton::C)
            .with(DeathScreenAction::Continue, KeyCode::Space)
            .with(DeathScreenAction::Continue, GamepadButton::RightTrigger)
    }
}

#[instrument(skip_all)]
fn input_system(
    mut next_state: ResMut<NextState<GameScreen>>,
    inputs: Single<&ActionState<DeathScreenAction>, With<BlackScreen>>,
) {
    if inputs.pressed(&DeathScreenAction::Continue) {
        tracing::info!("{:?}", DeathScreenAction::Continue);
        next_state.set(GameScreen::Gameplay)
    }
}

fn despawn_black_screen(mut commands: Commands, black_screen: Single<Entity, With<BlackScreen>>) {
    commands.entity(*black_screen).try_despawn();
}
