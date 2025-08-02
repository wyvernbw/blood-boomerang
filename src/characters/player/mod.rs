use std::marker::PhantomData;
use std::time::Duration;

use crate::COLORS;
use crate::ShakeExt;
use crate::autotimer::prelude::*;
use crate::characters::enemies::PlayerHitEvent;
use crate::characters::player::shoot::PlayerShoot;
use crate::characters::player::shoot::player_shoot_plugin;
use crate::characters::prelude::*;
use crate::screens::prelude::*;

use bevy::ecs::system::ScheduleSystem;
use bevy::tasks::TaskPool;
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_asset_loader::prelude::*;
use bevy_enoki::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_trauma_shake::Shake;
use leafwing_input_manager::prelude::*;
use tracing::instrument;

pub mod shoot;

pub mod prelude {
    pub use super::despawn_player;
    pub use super::player_plugin;
    pub use super::spawn_player;
}

pub fn player_plugin(app: &mut App) {
    app.add_plugins(player_shoot_plugin)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        // Defined below, detects whether MKB or gamepad are active
        .add_plugins(InputModeManagerPlugin)
        .init_resource::<ActionState<PlayerAction>>()
        .insert_resource(PlayerAction::default_input_map())
        // Set up the input processing
        .add_systems(
            OnEnter(GameScreen::SplashNext),
            update_boomerang_activation_particles_color,
        )
        .add_systems(
            Update,
            (
                control_player,
                player_mouse_aim
                    .before(player_aim)
                    .run_if(in_state(ActiveInput::MouseKeyboard)),
                player_aim,
                player_handle_hit_events,
                player_die_if_out_of_health.after(player_handle_hit_events),
                on_player_died,
                player_died_effects
                    .after(player_die_if_out_of_health)
                    .after(on_player_died),
            )
                .run_if(in_state(GameScreen::Gameplay)),
        )
        .configure_loading_state(
            LoadingStateConfig::new(GameScreen::SplashFirst).load_collection::<PlayerAssets>(),
        );
}

#[derive(Resource, AssetCollection)]
pub struct PlayerAssets {
    #[asset(path = "player/player.png")]
    sprite: Handle<Image>,
    #[asset(path = "player/boomerang.png")]
    boomerang_sprite: Handle<Image>,
    #[asset(path = "player/boomerang_activation.particles.ron")]
    boomerang_activation_particles: Handle<Particle2dEffect>,
}

fn update_boomerang_activation_particles_color(
    mut effects: ResMut<Assets<Particle2dEffect>>,
    assets: Res<PlayerAssets>,
) {
    if let Some(effect) = effects.get_mut(assets.boomerang_activation_particles.id()) {
        effect.color = Some(COLORS[2].into());
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Component, Debug)]
#[require(Hitbox)]
pub struct PlayerHitbox;

#[derive(Component, Debug)]
#[require(Hurtbox)]
pub struct PlayerHurtbox;

pub fn spawn_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands
        .spawn(character_base())
        .insert(CollisionGroups::new(
            PLAYER_HURTBOX_GROUP,
            ENEMY_HITBOX_GROUP,
        ))
        .insert(Health(1))
        .insert(ColliderDebugColor(Hsla::hsl(210.0, 1.0, 0.8)))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(PlayerHurtbox)
        .insert(Player)
        .insert(PlayerShoot {
            rate: 0.025,
            spread: 5.0_f32.to_radians(),
        })
        .insert(Speed(96.0))
        .insert(Bobbing)
        .insert(Sprite {
            anchor: bevy::sprite::Anchor::BottomCenter,
            image: player_assets.sprite.clone(),
            ..default()
        });
}

pub fn despawn_player(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands.entity(*player).try_despawn();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
enum PlayerAction {
    Move,
    Aim,
    #[actionlike(Button)]
    Shoot,
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::Move, GamepadStick::LEFT);
        input_map.insert_dual_axis(Self::Move, VirtualDPad::dpad());
        input_map.insert_dual_axis(Self::Aim, GamepadStick::RIGHT);
        input_map.insert(Self::Shoot, GamepadButton::RightTrigger);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::Move, VirtualDPad::wasd());
        input_map.insert_dual_axis(Self::Aim, MouseMove::default());
        input_map.insert(Self::Shoot, MouseButton::Left);

        input_map
    }
}

pub struct InputModeManagerPlugin;

impl Plugin for InputModeManagerPlugin {
    fn build(&self, app: &mut App) {
        // Init a state to record the current active input
        app.init_state::<ActiveInput>()
            // System to switch to gamepad as active input
            .add_systems(
                Update,
                activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
            )
            // System to switch to MKB as active input
            .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)));
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum ActiveInput {
    #[default]
    MouseKeyboard,
    Gamepad,
}

/// Switch the gamepad when any button is pressed or any axis input used
fn activate_gamepad(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.read() {
        match ev {
            GamepadEvent::Button(_) | GamepadEvent::Axis(_) => {
                tracing::info!("Switching to gamepad input");
                next_state.set(ActiveInput::Gamepad);
                return;
            }
            _ => (),
        }
    }
}

/// Switch to mouse and keyboard input when any keyboard button is pressed
fn activate_mkb(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut kb_evr: EventReader<KeyboardInput>,
) {
    for _ev in kb_evr.read() {
        tracing::info!("Switching to mouse and keyboard input");
        next_state.set(ActiveInput::MouseKeyboard);
    }
}

#[instrument(skip_all)]
fn control_player(
    mut commands: Commands,
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    player_query: Single<(Entity, &mut Velocity, &Speed), (With<Player>, Without<Dead>)>,
) {
    let (player, mut player_velocity, player_speed) = player_query.into_inner();
    let dt = time.delta_secs();
    if action_state.axis_pair(&PlayerAction::Move) != Vec2::ZERO {
        player_velocity.linvel = player_velocity.linvel.move_towards(
            action_state
                .clamped_axis_pair(&PlayerAction::Move)
                .clamp_length_max(1.0)
                * **player_speed,
            **player_speed * dt * 20.0,
        );
        commands.entity(player).try_insert(Moving);
        tracing::trace!("{:?}", action_state.axis_pair(&PlayerAction::Move));
    } else {
        commands.entity(player).try_remove::<Moving>();
    }
}

fn player_aim(
    action_state: Res<ActionState<PlayerAction>>,
    mut aim_dir: Single<&mut AimDir, With<Player>>,
) {
    if action_state.axis_pair(&PlayerAction::Aim) != Vec2::ZERO {
        let look = action_state.axis_pair(&PlayerAction::Aim).normalize();
        **aim_dir = AimDir(look);
    }
}

fn player_mouse_aim(
    camera_query: Query<(&GlobalTransform, &Camera), With<OuterCamera>>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut action_state: ResMut<ActionState<PlayerAction>>,
) {
    let (camera_transform, camera) = camera_query.single().expect("Need a single camera");
    let player_transform = player_query.single().expect("Need a single player");
    let window = window_query.single().expect("Need a single primary window");

    // Many steps can fail here, so we'll wrap in an option pipeline
    // First check if the cursor is in window
    // Then check if the ray intersects the plane defined by the player
    // Then finally compute the point along the ray to look at
    let player_position = player_transform.translation;
    if let Some(p) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            Some(ray).zip(ray.intersect_plane(player_position, InfinitePlane3d::new(Vec3::Z)))
        })
        .map(|(ray, p)| ray.get_point(p))
    {
        let diff = (p - player_position).xy();
        if diff.length_squared() > 1e-3f32 {
            // Get the mutable action data to set the axis
            let action_data = action_state.dual_axis_data_mut_or_default(&PlayerAction::Aim);

            // Flipping y sign here to be consistent with gamepad input.
            // We could also invert the gamepad y-axis
            action_data.pair = Vec2::new(diff.x, diff.y);
        }
    }
}

#[instrument(skip_all)]
fn player_handle_hit_events(
    mut commands: Commands,
    player: Single<(Entity, &mut Health, &Transform), With<Player>>,
    mut event_reader: EventReader<PlayerHitEvent>,
    mut shake: Single<&mut Shake>,
) {
    let (player, mut health, transform) = player.into_inner();
    for PlayerHitEvent {
        damage,
        source_transform,
    } in event_reader.read()
    {
        **health -= **damage;
        if let Some(source_transform) = source_transform {
            let from_source = (transform.translation - source_transform.translation)
                .normalize()
                .xy()
                * -1.;
            tracing::info!(?from_source);
            commands.entity(player).insert(Knockback(from_source));
            shake.apply_trauma(0.3);
        }
    }
}

fn player_die_if_out_of_health(
    mut commands: Commands,
    player: Single<(Entity, &Health), With<Player>>,
) {
    let (player_id, player_health) = *player;
    if **player_health <= 0 {
        commands.entity(player_id).try_insert(Dead);
    }
}

#[derive(Debug, Resource, Default, Component, Deref, DerefMut)]
pub struct PlayerDeathEffectsTimer(AutoTimer<800, TimerOnce>);

fn on_player_died(mut commands: Commands, player: Single<Entity, (With<Player>, Added<Dead>)>) {
    commands
        .entity(*player)
        .insert(PlayerDeathEffectsTimer::default());
}

fn player_died_effects(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(Entity, &mut PlayerDeathEffectsTimer), (With<Player>, With<Dead>)>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    let (player_id, mut anim_timer) = player.into_inner();
    anim_timer.tick(time.delta());
    if anim_timer.just_finished() {
        next_screen.set(GameScreen::AfterDeath);
    }
}
