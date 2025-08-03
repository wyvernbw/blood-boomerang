use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use bon::Builder;

use crate::characters::enemies::prelude::*;
use crate::characters::prelude::*;
use crate::screens::prelude::*;

pub mod prelude {
    pub use super::ghost_plugin;
    pub use super::{Ghost, GhostArgs, GhostAssets};
}

pub fn ghost_plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(GameScreen::SplashFirst).load_collection::<GhostAssets>(),
    );
}

#[derive(Component, Clone)]
pub struct Ghost;

#[derive(AssetCollection, Resource)]
pub struct GhostAssets {
    #[asset(path = "enemies/ghost.png")]
    pub sprite: Handle<Image>,
}

#[derive(Builder, Clone)]
pub struct GhostArgs<'a> {
    assets: &'a Res<'a, GhostAssets>,
}

pub trait CommandsGhost<T> {
    fn spawn_ghost(&'_ mut self, args: T) -> EntityCommands<'_>;
}

impl<'w, 's> CommandsGhost<GhostArgs<'_>> for Commands<'w, 's> {
    fn spawn_ghost(&'_ mut self, args: GhostArgs) -> EntityCommands<'_> {
        let mut commands = self.spawn(enemy_base());
        commands
            .insert((
                Sprite {
                    image: args.assets.sprite.clone(),
                    ..default()
                },
                Ghost,
            ))
            .insert(Health(2))
            .insert(Speed(64.0))
            .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 0.0)))
            .insert(EnemyClass::Melee)
            .with_children(|parent| {
                parent.spawn((
                    EnemyHitbox,
                    Transform::default(),
                    Sensor,
                    Collider::ball(8.0),
                    Damage(1),
                    CollisionGroups::new(ENEMY_HITBOX_GROUP, PLAYER_HURTBOX_GROUP),
                    ActiveEvents::COLLISION_EVENTS,
                    ColliderDebugColor(Hsla::hsl(340., 1.0, 0.8)),
                ));
            });
        commands
    }
}
