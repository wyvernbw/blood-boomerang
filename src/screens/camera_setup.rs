use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::camera::ScalingMode as CameraScalingMode;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::RenderLayers;
use bevy_simple_screen_boxing::CameraBox;
use bevy_simple_screen_boxing::CameraBoxingPlugin;

use crate::COLORS;

pub mod prelude {
    pub use super::InGameCamera;
    pub use super::OuterCamera;
    pub use super::RES_HEIGHT;
    pub use super::RES_WIDTH;
}

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
pub struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
pub struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
pub struct OuterCamera;

pub(super) fn camera_setup_plugin(app: &mut App) {
    app.add_plugins(CameraBoxingPlugin)
        .add_systems(Startup, spawn_camera);
}

pub(super) fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // This Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // Fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = CameraScalingMode::Fixed {
        width: RES_WIDTH as f32,
        height: RES_HEIGHT as f32,
    };
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(*COLORS.last().unwrap()),
            target: RenderTarget::Image(image_handle.clone().into()),
            ..default()
        },
        Msaa::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((
        Sprite::from_image(image_handle.clone()),
        Canvas,
        HIGH_RES_LAYERS,
    ));

    commands.spawn((
        Camera2d,
        CameraBox::ResolutionIntegerScale {
            resolution: Vec2::new(RES_WIDTH as f32, RES_HEIGHT as f32),
            allow_imperfect_aspect_ratios: false,
        },
        Projection::Orthographic(projection),
        Msaa::Off,
        OuterCamera,
        HIGH_RES_LAYERS,
    ));
}
