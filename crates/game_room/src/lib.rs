//! ARTHUB game room: a procedural 3D lounge in Bevy.
//!
//! Runs natively (`cargo run -p game_room`) for iteration and as a lazily
//! loaded WASM module on the site, where it renders into
//! `#game-room-canvas` and talks to Leptos through the `bridge` crate.

use bevy::asset::AssetMetaCheck;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::winit::WinitSettings;
use bridge::{Command, Mode};

mod atmosphere;
mod audio;
mod camera;
mod dressing;
mod envmap;
mod fill;
mod interact;
mod link;
mod merge;
mod neon;
mod pool;
mod quality;
mod scene;
mod textures;

#[cfg(not(target_arch = "wasm32"))]
mod dev;
#[cfg(not(target_arch = "wasm32"))]
mod hud;

/// Which experience the room is running: hero teaser or the full room.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomMode(pub Mode);

/// Top-level interaction state.
#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum View {
    #[default]
    Room,
    Pool,
}

/// A command from the site (or the keyboard, natively), as a Bevy message.
#[derive(Message, Clone, Debug)]
pub struct Cmd(pub Command);

/// Rendering is suspended (canvas off-screen or tab hidden).
#[derive(Resource, Default)]
pub struct Paused(pub bool);

pub fn app(mode: Mode, featured: Vec<bridge::FeaturedProject>) -> App {
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut window = Window {
        title: "ARTHUB — Game Room".into(),
        canvas: Some(format!("#{}", bridge::CANVAS_ID)),
        fit_canvas_to_parent: true,
        // Let the page keep scrolling, focusing and handling keys normally.
        prevent_default_event_handling: false,
        // Winit focuses the canvas on creation. As the hero backdrop that
        // would pull keyboard focus past the skip link and the nav; only the
        // full room wants the keys (the site focuses it when it gets there).
        focused: mode == Mode::Full,
        resolution: WindowResolution::new(1280, 800),
        ..default()
    };

    // On the web bevy_winit never learns the device pixel ratio (Bevy thinks
    // it is 1) while the browser reports pointer and touch positions in
    // device pixels, so on a 2× screen every click lands at twice its real
    // position. Telling Bevy the real ratio up front makes the two agree, and
    // renders at full sharpness.
    #[cfg(target_arch = "wasm32")]
    if let Some(dpr) = web_sys::window().map(|w| w.device_pixel_ratio() as f32) {
        window.resolution.set_scale_factor_override(Some(dpr));
    }

    // Native: the workspace `assets/` folder. Web: the site root, where
    // cargo-leptos serves the same folder.
    let file_path = if cfg!(target_arch = "wasm32") {
        "/"
    } else {
        "../../assets"
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            .set(ImagePlugin {
                // Surface textures tile, carry mip chains, and are seen at
                // grazing angles (the floor), so: repeat, trilinear, 4× aniso.
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    anisotropy_clamp: 4,
                    ..default()
                },
            })
            .set(AssetPlugin {
                file_path: file_path.into(),
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(MeshPickingPlugin)
    .insert_resource(ClearColor(Color::srgb(0.02, 0.018, 0.024)))
    .insert_resource(RoomMode(mode))
    .insert_resource(scene::Featured(featured))
    .insert_resource(WinitSettings::continuous())
    .init_resource::<Paused>()
    .init_state::<View>()
    .add_message::<Cmd>()
    .add_systems(PreStartup, textures::setup)
    .add_systems(PostStartup, envmap::setup)
    .add_plugins((
        merge::MergePlugin,
        atmosphere::AtmospherePlugin,
        fill::FillPlugin,
    ))
    .add_plugins((
        link::LinkPlugin,
        scene::ScenePlugin,
        neon::NeonPlugin,
        camera::CameraPlugin,
        interact::InteractPlugin,
        pool::PoolPlugin,
        audio::AudioFxPlugin,
        quality::QualityPlugin,
    ));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((hud::HudPlugin, dev::DevPlugin));

    app
}

pub fn run(mode: Mode, featured: Vec<bridge::FeaturedProject>) {
    app(mode, featured).run();
}

/// Entry point for the browser. `mode` is `"preview"` or `"full"`;
/// `featured` is [`bridge::encode_featured`] output from the site.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(mode: String, featured: String) {
    console_error_panic_hook::set_once();
    run(
        Mode::from_id(&mode).unwrap_or_default(),
        bridge::decode_featured(&featured),
    );
}

/// The camera's current `[eye.xyz, look.xyz]` (read-only; for tests and debugging).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn camera_pose() -> Vec<f32> {
    camera::LAST_POSE
        .lock()
        .map(|p| p.to_vec())
        .unwrap_or_default()
}

/// Snap the camera to `eye` looking at `look` (for screenshots and audits).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn debug_pose(ex: f32, ey: f32, ez: f32, lx: f32, ly: f32, lz: f32) {
    use bevy::math::Vec3;
    if let Ok(mut p) = camera::DEBUG_POSE.lock() {
        *p = Some(camera::Pose {
            eye: Vec3::new(ex, ey, ez),
            look: Vec3::new(lx, ly, lz),
        });
    }
}
