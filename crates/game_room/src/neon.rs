//! Neon signs from a tiny stroke font: each letter is a set of polylines on a
//! 4×6 grid, drawn as glowing tubes (rods) with spheres at the joints. The same
//! font in a matte material does the chalk lettering on the chalkboard.

use bevy::prelude::*;

use crate::scene::Kit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeonColor {
    Accent,
    Pink,
    Cyan,
    Amber,
    Red,
}

impl NeonColor {
    const CYCLE: [NeonColor; 4] = [
        NeonColor::Accent,
        NeonColor::Pink,
        NeonColor::Cyan,
        NeonColor::Amber,
    ];

    pub fn color(self) -> Color {
        match self {
            // The site's chalk accent, pushed a little warmer to read as neon.
            NeonColor::Accent => Color::srgb(1.0, 0.93, 0.8),
            NeonColor::Pink => Color::srgb(1.0, 0.18, 0.53),
            NeonColor::Cyan => Color::srgb(0.13, 0.9, 1.0),
            NeonColor::Amber => Color::srgb(1.0, 0.62, 0.12),
            NeonColor::Red => Color::srgb(1.0, 0.1, 0.08),
        }
    }

    pub fn next(self) -> Self {
        let i = Self::CYCLE.iter().position(|c| *c == self).unwrap_or(0);
        Self::CYCLE[(i + 1) % Self::CYCLE.len()]
    }
}

/// A lit sign. The flicker system drives its material and light from this.
#[derive(Component)]
pub struct Neon {
    pub mat: Handle<StandardMaterial>,
    pub color: NeonColor,
    pub strength: f32,
    pub light: Option<Entity>,
    pub light_intensity: f32,
    pub seed: u32,
    /// Only one sign misbehaves; the rest burn steadily.
    pub flickers: bool,
    /// Last brightness written, so steady signs don't touch their material.
    pub last: f32,
}

/// Cycle a sign to its next colour (clicking the big ARTHUB sign).
#[derive(Message)]
pub struct CycleNeon(pub Entity);

pub struct NeonPlugin;

impl Plugin for NeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CycleNeon>()
            .add_systems(Update, (cycle, flicker).chain());
    }
}

type Glyph = &'static [&'static [(i8, i8)]];

fn glyph(c: char) -> (Glyph, f32) {
    const A: Glyph = &[&[(0, 0), (0, 4), (2, 6), (4, 4), (4, 0)], &[(0, 3), (4, 3)]];
    const B: Glyph = &[
        &[(0, 0), (0, 6), (3, 6), (4, 5), (4, 4), (3, 3), (0, 3)],
        &[(3, 3), (4, 2), (4, 1), (3, 0), (0, 0)],
    ];
    const C: Glyph = &[&[(4, 6), (1, 6), (0, 5), (0, 1), (1, 0), (4, 0)]];
    const E: Glyph = &[&[(4, 6), (0, 6), (0, 0), (4, 0)], &[(0, 3), (3, 3)]];
    const H: Glyph = &[&[(0, 0), (0, 6)], &[(4, 0), (4, 6)], &[(0, 3), (4, 3)]];
    const I: Glyph = &[&[(0, 6), (2, 6)], &[(1, 6), (1, 0)], &[(0, 0), (2, 0)]];
    const K: Glyph = &[&[(0, 0), (0, 6)], &[(4, 6), (0, 2)], &[(1, 3), (4, 0)]];
    const L: Glyph = &[&[(0, 6), (0, 0), (4, 0)]];
    const N: Glyph = &[&[(0, 0), (0, 6), (4, 0), (4, 6)]];
    const O: Glyph = &[&[
        (1, 0),
        (0, 1),
        (0, 5),
        (1, 6),
        (3, 6),
        (4, 5),
        (4, 1),
        (3, 0),
        (1, 0),
    ]];
    const P: Glyph = &[&[(0, 0), (0, 6), (3, 6), (4, 5), (4, 4), (3, 3), (0, 3)]];
    const R: Glyph = &[
        &[(0, 0), (0, 6), (3, 6), (4, 5), (4, 4), (3, 3), (0, 3)],
        &[(2, 3), (4, 0)],
    ];
    const S: Glyph = &[&[
        (4, 6),
        (1, 6),
        (0, 5),
        (0, 4),
        (1, 3),
        (3, 3),
        (4, 2),
        (4, 1),
        (3, 0),
        (0, 0),
    ]];
    const T: Glyph = &[&[(0, 6), (4, 6)], &[(2, 6), (2, 0)]];
    const U: Glyph = &[&[(0, 6), (0, 1), (1, 0), (3, 0), (4, 1), (4, 6)]];
    const X: Glyph = &[&[(0, 0), (4, 6)], &[(0, 6), (4, 0)]];
    const V: Glyph = &[&[(0, 6), (2, 0), (4, 6)]];
    const DOT: Glyph = &[&[(1, 3), (1, 3)]];
    const G: Glyph = &[&[
        (4, 5),
        (3, 6),
        (1, 6),
        (0, 5),
        (0, 1),
        (1, 0),
        (3, 0),
        (4, 1),
        (4, 3),
        (2, 3),
    ]];
    const D: Glyph = &[&[(0, 0), (0, 6), (2, 6), (4, 4), (4, 2), (2, 0), (0, 0)]];
    match c {
        'A' => (A, 4.0),
        'B' => (B, 4.0),
        'C' => (C, 4.0),
        'E' => (E, 4.0),
        'H' => (H, 4.0),
        'I' => (I, 2.0),
        'K' => (K, 4.0),
        'L' => (L, 4.0),
        'N' => (N, 4.0),
        'O' => (O, 4.0),
        'P' => (P, 4.0),
        'R' => (R, 4.0),
        'S' => (S, 4.0),
        'T' => (T, 4.0),
        'U' => (U, 4.0),
        'X' => (X, 4.0),
        'V' => (V, 4.0),
        'D' => (D, 4.0),
        'G' => (G, 4.0),
        '·' => (DOT, 2.0),
        _ => (&[], 3.0),
    }
}

const SPACING: f32 = 1.6;

/// A small RGBA canvas for flat signage (labels, posters), lettered with the
/// same stroke font. Cube faces sample V bottom-up, so `image` flips rows.
pub struct Canvas {
    w: u32,
    h: u32,
    px: Vec<[u8; 3]>,
}

impl Canvas {
    pub fn new(w: u32, h: u32, paper: [u8; 3]) -> Self {
        Self {
            w,
            h,
            px: vec![paper; (w * h) as usize],
        }
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, ink: [u8; 3]) {
        let (x0, y0) = (x.max(0.0) as u32, y.max(0.0) as u32);
        let x1 = ((x + w) as u32).min(self.w);
        let y1 = ((y + h) as u32).min(self.h);
        for py in y0..y1 {
            for px in x0..x1 {
                self.px[(py * self.w + px) as usize] = ink;
            }
        }
    }

    /// A thick line from `a` to `b` (pixels, y down).
    pub fn line(&mut self, a: Vec2, b: Vec2, r: f32, ink: [u8; 3]) {
        let lo = a.min(b) - Vec2::splat(r + 1.0);
        let hi = a.max(b) + Vec2::splat(r + 1.0);
        let d = b - a;
        let len2 = d.length_squared().max(1e-6);
        for py in lo.y.max(0.0) as u32..(hi.y.max(0.0) as u32).min(self.h) {
            for px in lo.x.max(0.0) as u32..(hi.x.max(0.0) as u32).min(self.w) {
                let p = Vec2::new(px as f32 + 0.5, py as f32 + 0.5);
                let t = ((p - a).dot(d) / len2).clamp(0.0, 1.0);
                if p.distance(a + d * t) <= r {
                    self.px[(py * self.w + px) as usize] = ink;
                }
            }
        }
    }

    /// `text` centred on `cx`, its capitals `cap` px tall from `top`.
    pub fn text(&mut self, text: &str, cx: f32, top: f32, cap: f32, ink: [u8; 3]) {
        let unit = cap / 6.0;
        let width: f32 = text.chars().map(|c| glyph(c).1 + SPACING).sum::<f32>() - SPACING;
        let mut x = cx / unit - width / 2.0;
        for c in text.chars() {
            let (strokes, w) = glyph(c);
            for line in strokes {
                let pts: Vec<Vec2> = line
                    .iter()
                    .map(|&(gx, gy)| {
                        Vec2::new((x + gx as f32) * unit, top + (6 - gy) as f32 * unit)
                    })
                    .collect();
                for seg in pts.windows(2) {
                    self.line(seg[0], seg[1], unit * 0.5, ink);
                }
            }
            x += w + SPACING;
        }
    }

    pub fn image(&self) -> Image {
        self.export(false)
    }

    /// For wrapping round a cylinder, whose U runs the other way round.
    pub fn image_wrapped(&self) -> Image {
        self.export(true)
    }

    fn export(&self, mirror: bool) -> Image {
        use bevy::asset::RenderAssetUsages;
        use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
        let mut data = Vec::with_capacity((self.w * self.h * 4) as usize);
        for y in (0..self.h).rev() {
            for x in 0..self.w {
                let x = if mirror { self.w - 1 - x } else { x };
                let [r, g, b] = self.px[(y * self.w + x) as usize];
                data.extend([r, g, b, 255]);
            }
        }
        Image::new(
            Extent3d {
                width: self.w,
                height: self.h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// Draw `text` centred on `t` (local XY plane, facing +Z). Returns the group.
pub fn stroke_text(
    k: &mut Kit,
    parent: Entity,
    text: &str,
    t: Transform,
    unit: f32,
    radius: f32,
    mat: &Handle<StandardMaterial>,
) -> Entity {
    let group = k.group(Some(parent), t);
    let width: f32 = text.chars().map(|c| glyph(c).1 + SPACING).sum::<f32>() - SPACING;
    let mut x = -width / 2.0;
    for c in text.chars() {
        let (strokes, w) = glyph(c);
        for line in strokes {
            let pts: Vec<Vec3> = line
                .iter()
                .map(|&(gx, gy)| Vec3::new((x + gx as f32) * unit, (gy as f32 - 3.0) * unit, 0.0))
                .collect();
            for seg in pts.windows(2) {
                k.rod_lo(group, seg[0], seg[1], radius, mat);
            }
            for p in &pts {
                k.ball_lo(group, *p, radius, mat);
            }
        }
        x += w + SPACING;
    }
    group
}

fn tube_mat(k: &mut Kit, color: NeonColor, strength: f32) -> Handle<StandardMaterial> {
    k.mats.add(StandardMaterial {
        base_color: color.color().with_luminance(0.3),
        emissive: color.color().to_linear() * strength,
        perceptual_roughness: 0.3,
        ..default()
    })
}

fn attach_light(
    k: &mut Kit,
    parent: Entity,
    at: Option<Vec3>,
    color: NeonColor,
    intensity: f32,
) -> Option<Entity> {
    at.map(|at| {
        k.c.spawn((
            PointLight {
                color: color.color(),
                intensity,
                range: 3.5,
                ..default()
            },
            Transform::from_translation(at),
            ChildOf(parent),
        ))
        .id()
    })
}

/// A neon word with an optional coloured point light (in `parent` space).
pub fn sign(
    k: &mut Kit,
    parent: Entity,
    text: &str,
    t: Transform,
    unit: f32,
    color: NeonColor,
    light_at: Option<Vec3>,
) -> Entity {
    const STRENGTH: f32 = 14.0;
    let mat = tube_mat(k, color, STRENGTH);
    let group = stroke_text(k, parent, text, t, unit, unit * 0.11, &mat);
    let intensity = 60_000.0 * unit / 0.12;
    let light = attach_light(k, parent, light_at, color, intensity);
    let seed = text
        .bytes()
        .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    k.c.entity(group).insert(Neon {
        mat,
        color,
        strength: STRENGTH,
        light,
        light_intensity: intensity,
        seed,
        flickers: text == "OPEN",
        last: -1.0,
    });
    group
}

/// The 8-ball: a cyan ring with a white "8" inside.
pub fn eight_ball(k: &mut Kit, parent: Entity, t: Transform, radius: f32) {
    let group = k.group(Some(parent), t);
    let ring = tube_mat(k, NeonColor::Cyan, 12.0);
    let eight = tube_mat(k, NeonColor::Accent, 12.0);
    let circle = |k: &mut Kit, c: Vec3, r: f32, tube: f32, m: &Handle<StandardMaterial>| {
        let n = 28;
        for i in 0..n {
            let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
            let p0 = c + Vec3::new(a0.cos(), a0.sin(), 0.0) * r;
            let p1 = c + Vec3::new(a1.cos(), a1.sin(), 0.0) * r;
            k.rod_lo(group, p0, p1, tube, m);
            k.ball_lo(group, p0, tube, m);
        }
    };
    circle(k, Vec3::ZERO, radius, 0.012, &ring);
    circle(
        k,
        Vec3::new(0.0, radius * 0.22, 0.0),
        radius * 0.2,
        0.01,
        &eight,
    );
    circle(
        k,
        Vec3::new(0.0, -radius * 0.26, 0.0),
        radius * 0.26,
        0.01,
        &eight,
    );
    let light = attach_light(
        k,
        parent,
        Some(t.translation + t.rotation * Vec3::Z * 0.5),
        NeonColor::Cyan,
        40_000.0,
    );
    k.c.entity(group).insert(Neon {
        mat: ring,
        color: NeonColor::Cyan,
        strength: 12.0,
        light,
        light_intensity: 40_000.0,
        seed: 88,
        flickers: false,
        last: -1.0,
    });
}

fn hash(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x & 0xffff) as f32 / 65535.0
}

fn cycle(
    mut msgs: MessageReader<CycleNeon>,
    mut signs: Query<&mut Neon>,
    mut lights: Query<&mut PointLight>,
) {
    for CycleNeon(e) in msgs.read() {
        if let Ok(mut n) = signs.get_mut(*e) {
            n.color = n.color.next();
            if let Some(mut l) = n.light.and_then(|l| lights.get_mut(l).ok()) {
                l.color = n.color.color();
            }
        }
    }
}

/// Mostly steady with a faint hum, plus the occasional stutter.
fn flicker(
    time: Res<Time>,
    mut signs: Query<&mut Neon>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut lights: Query<&mut PointLight>,
) {
    let t = time.elapsed_secs();
    let slot = (t / 0.07) as u32;
    for mut n in &mut signs {
        let f = if n.flickers {
            let r = hash(slot.wrapping_add(n.seed.wrapping_mul(9973)));
            let burst = hash((t / 3.0) as u32 ^ n.seed) < 0.25;
            if burst && r < 0.4 { 0.1 + r } else { 1.0 }
        } else {
            1.0
        };
        if f == n.last {
            continue;
        }
        n.last = f;
        if let Some(m) = mats.get_mut(&n.mat) {
            let mut m = m;
            m.emissive = n.color.color().to_linear() * n.strength * f;
            m.base_color = n.color.color().with_luminance(0.3);
        }
        if let Some(mut l) = n.light.and_then(|l| lights.get_mut(l).ok()) {
            l.intensity = n.light_intensity * f;
        }
    }
}
