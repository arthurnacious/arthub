//! Air: dust drifting in the pool lamp's light and soft blob shadows under
//! furniture. (A fake light-shaft volume was tried and dropped: on WebGL2 it
//! read as a pale slab rather than haze; distance fog does that job.) (SSAO needs compute shaders,
//! which WebGL2 doesn't have, so contact shadows are faked).

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bridge::{Mode, Quality};

use crate::merge::NoMerge;
use crate::quality::QualityState;
use crate::scene::TABLE_CENTER;
use crate::{RoomMode, View};

#[derive(Component)]
struct Mote {
    seed: Vec3,
    speed: f32,
}

#[derive(Component)]
struct Shaft;

/// Soft dark ellipse under a piece of furniture.
#[derive(Resource)]
pub struct Blob {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<StandardMaterial>,
}

impl Blob {
    /// A shadow footprint centred at `at` (floor level), `size` across.
    pub fn bundle(&self, at: Vec3, size: Vec2, yaw: f32) -> impl Bundle {
        (
            Mesh3d(self.mesh.clone()),
            MeshMaterial3d(self.mat.clone()),
            Transform::from_translation(at.with_y(0.004))
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::new(size.x, 1.0, size.y)),
            NotShadowCaster,
        )
    }
}

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, blob_assets)
            .add_systems(Startup, spawn)
            .add_systems(Update, (drift, visibility));
    }
}

const LAMP_Y: f32 = 1.78;
const MOTES: usize = 140;

fn blob_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Radial falloff in the alpha channel.
    let n = 64u32;
    let mut data = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / n as f32 * 2.0 - Vec2::ONE;
            let a = (1.0 - d.length()).clamp(0.0, 1.0).powf(1.6);
            data.extend([0, 0, 0, (a * 255.0) as u8]);
        }
    }
    let tex = images.add(Image::new(
        Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    ));
    commands.insert_resource(Blob {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5))),
        mat: mats.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.75),
            base_color_texture: Some(tex),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // Dust: one tiny mesh + material, so the motes batch together.
    let mote = meshes.add(Sphere::new(0.0035).mesh().ico(1).expect("ico"));
    let glow = mats.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: LinearRgba::rgb(1.0, 0.8, 0.55) * 3.0,
        unlit: true,
        ..default()
    });
    let mut rng = 0x9E37_79B9u32;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        (rng >> 8) as f32 / (1u32 << 24) as f32
    };
    for _ in 0..MOTES {
        let seed = Vec3::new(next(), next(), next());
        commands.spawn((
            Mote {
                seed,
                speed: 0.02 + next() * 0.05,
            },
            Mesh3d(mote.clone()),
            MeshMaterial3d(glow.clone()),
            Transform::default(),
            NotShadowCaster,
            NoMerge,
        ));
    }
}

/// Motes wander inside the lamp's cone on slow, overlapping sine paths.
fn drift(time: Res<Time>, mut motes: Query<(&Mote, &mut Transform, &ViewVisibility)>) {
    let t = time.elapsed_secs();
    for (m, mut tf, vis) in &mut motes {
        if !vis.get() {
            continue;
        }
        let s = m.seed;
        let y = (s.y + t * m.speed).fract();
        let spread = 0.45 + y * 0.45; // wider toward the table
        let x = ((s.x * 2.0 - 1.0) + (t * 0.11 + s.z * 9.0).sin() * 0.08) * 1.25 * spread;
        let z = ((s.z * 2.0 - 1.0) + (t * 0.07 + s.x * 7.0).cos() * 0.08) * 0.55 * spread;
        tf.translation =
            TABLE_CENTER + Vec3::new(x, LAMP_Y - 0.1 - y * (LAMP_Y - 0.9) - TABLE_CENTER.y, z);
    }
}

/// Dust and the shaft are the first things to go: off on Low, in the hero
/// preview, and while playing (they'd sit between the camera and the balls).
fn visibility(
    q: Res<QualityState>,
    mode: Res<RoomMode>,
    view: Res<State<View>>,
    mut motes: Query<&mut Visibility, (With<Mote>, Without<Shaft>)>,
    mut shaft: Query<&mut Visibility, (With<Shaft>, Without<Mote>)>,
) {
    let on = q.tier != Quality::Low && mode.0 == Mode::Full && *view.get() == View::Room;
    let want = if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut motes {
        if *v != want {
            *v = want;
        }
    }
    for mut v in &mut shaft {
        if *v != want {
            *v = want;
        }
    }
}
