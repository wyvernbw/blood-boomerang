use bevy::prelude::*;
use bevy::render::camera::ScalingMode as CameraScalingMode;
use bevy::render::view::RenderLayers;
use bevy_simple_screen_boxing::CameraBox;
use bevy_simple_screen_boxing::CameraBoxingPlugin;

/// In-game resolution width.
pub const RES_WIDTH: u32 = 160 * 2;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 144 * 2;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

pub(super) fn camera_setup_plugin(app: &mut App) {
    app.add_plugins(CameraBoxingPlugin)
        .add_systems(Startup, spawn_camera);
}

pub(super) fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = CameraScalingMode::Fixed {
        width: RES_WIDTH as f32,
        height: RES_HEIGHT as f32,
    };
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.5, 0.5, 0.9)),
            ..default()
        },
        CameraBox::ResolutionIntegerScale {
            resolution: Vec2::new(RES_WIDTH as f32, RES_HEIGHT as f32),
            allow_imperfect_aspect_ratios: false,
        },
        Projection::Orthographic(projection),
    ));

    // background
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::from_size(vec2(
            RES_WIDTH as f32,
            RES_HEIGHT as f32,
        )))),
        MeshMaterial2d(materials.add(Color::srgba(0.75, 0.5, 0.5, 1.0))),
        Transform::default(),
        PIXEL_PERFECT_LAYERS,
    ));
}
