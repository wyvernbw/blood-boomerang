use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::characters::enemies::prelude::*;
use crate::characters::prelude::*;
use crate::screens::prelude::*;

pub mod prelude {
    pub use super::ghost_plugin;
    pub use super::spawn_ghost;
    pub use super::{Ghost, GhostAssets};
}

pub fn ghost_plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(GameScreen::Splash).load_collection::<GhostAssets>(),
    );
}

#[derive(Component)]
pub struct Ghost;

#[derive(AssetCollection, Resource)]
pub struct GhostAssets {
    #[asset(path = "enemies/ghost.png")]
    sprite: Handle<Image>,
}

#[bon::builder]
pub fn spawn_ghost<'a>(
    commands: &'a mut Commands<'_, '_>,
    assets: &Res<'_, GhostAssets>,
) -> EntityCommands<'a> {
    let mut commands = commands.spawn(enemy_base());
    commands
        .insert((
            Sprite {
                image: assets.sprite.clone(),
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
