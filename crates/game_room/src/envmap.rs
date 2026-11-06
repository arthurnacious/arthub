//! A procedural environment map: a dark room with a warm pool on the ceiling
//! above the lamp and coloured glows where the neon signs are. It gives
//! glossy surfaces (balls, chrome, the bar top) something to reflect without
//! flooding the room with light. Generated at startup (a few ms), so it
//! costs nothing to download and works on WebGL2.

use bevy::asset::RenderAssetUsages;
use bevy::light::EnvironmentMapLight;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use crate::camera::RoomCamera;

/// Radiance lobes: direction, linear colour, angular width (cosine falloff power).
const LOBES: &[([f32; 3], [f32; 3], f32)] = &[
    ([0.0, 1.0, 0.05], [1.0, 0.72, 0.42], 40.0), // pool lamp hot spot overhead
    ([0.0, -1.0, 0.0], [0.30, 0.10, 0.06], 3.0), // rug and floor bounce
    ([-0.35, 0.3, -1.0], [1.0, 0.95, 0.85], 18.0), // ARTHUB sign
    ([0.6, 0.15, -1.0], [0.9, 0.3, 0.9], 10.0),  // arcade glow
    ([1.0, 0.4, -0.35], [1.0, 0.25, 0.45], 16.0), // OPEN sign over the bar
    ([1.0, -0.1, -0.5], [1.0, 0.55, 0.25], 6.0), // bar back light
    ([-1.0, 0.2, -0.6], [1.0, 0.3, 0.55], 8.0),  // jukebox
    ([-1.0, 0.4, 0.0], [0.2, 0.85, 1.0], 20.0),  // 8-ball sign
];

fn radiance(dir: Vec3, widen: f32) -> Vec3 {
    let mut c = Vec3::splat(0.004);
    for &(d, col, power) in LOBES {
        let d = Vec3::from(d).normalize();
        let p = (power / widen).max(1.0);
        let k = dir.dot(d).max(0.0).powf(p);
        c += Vec3::from(col) * k;
    }
    c
}

/// Direction through texel (u, v) of cube face `face` (+X, -X, +Y, -Y, +Z, -Z).
fn face_dir(face: usize, u: f32, v: f32) -> Vec3 {
    let (a, b) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
    match face {
        0 => Vec3::new(1.0, -b, -a),
        1 => Vec3::new(-1.0, -b, a),
        2 => Vec3::new(a, 1.0, b),
        3 => Vec3::new(a, -1.0, -b),
        4 => Vec3::new(a, -b, 1.0),
        _ => Vec3::new(-a, -b, -1.0),
    }
    .normalize()
}

/// f32 → IEEE half (round-to-nearest; our values are small and positive).
fn f16(x: f32) -> [u8; 2] {
    let b = x.max(0.0).to_bits();
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let h: u16 = if exp <= 0 {
        0
    } else if exp >= 31 {
        0x7bff
    } else {
        ((exp as u16) << 10) | (((b >> 13) & 0x3ff) as u16)
    };
    h.to_le_bytes()
}

#[allow(dead_code)]
fn to_srgb8(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// A cubemap whose mip `m` is the environment blurred by `widen(m)`.
fn cubemap(size: u32, mips: u32, widen: impl Fn(u32) -> f32) -> Image {
    let mut data = Vec::new();
    // Layout: for each face, every mip level (wgpu's layer-major order).
    for face in 0..6 {
        for m in 0..mips {
            let s = (size >> m).max(1);
            for y in 0..s {
                for x in 0..s {
                    let d = face_dir(
                        face,
                        (x as f32 + 0.5) / s as f32,
                        (y as f32 + 0.5) / s as f32,
                    );
                    let c = radiance(d, widen(m));
                    for v in [c.x, c.y, c.z, 1.0] {
                        data.extend(f16(v));
                    }
                }
            }
        }
    }
    // `Image::new` checks the data against the base level only; build it with
    // that, then swap in the full mip chain.
    let base = vec![0u8; (size * size * 8 * 6) as usize];
    let mut img = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        base,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.data = Some(data);
    img.texture_descriptor.mip_level_count = mips;
    img.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    img
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cam: Query<Entity, With<RoomCamera>>,
) {
    let Ok(cam) = cam.single() else { return };
    // Native Vulkan on Intel Mesa loses the device with environment-map
    // lighting (Bevy uses texture binding arrays for light probes there);
    // WebGL2 takes a different path and is fine. Opt in natively with
    // ARTHUB_ENV=1.
    if cfg!(not(target_arch = "wasm32")) && std::env::var_os("ARTHUB_ENV").is_none() {
        info!("environment map disabled natively (set ARTHUB_ENV=1 to try it)");
        return;
    }
    let specular = images.add(cubemap(64, 7, |m| 1.0 + m as f32 * m as f32 * 1.5));
    let diffuse = images.add(cubemap(8, 1, |_| 60.0));
    commands.entity(cam).insert(EnvironmentMapLight {
        diffuse_map: diffuse,
        specular_map: specular,
        // Kept low: reflections should read, the room should stay dark.
        intensity: 110.0,
        ..default()
    });
}
