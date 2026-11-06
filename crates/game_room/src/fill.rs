//! The room fill: the back-bar on the front wall (behind the default view),
//! the window onto the city, the back-room door, the ceiling and the corners.
//! All static and merged, except the fan rotor. Small clutter goes under a
//! `Clutter` root that the Low tier hides.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bridge::{FeaturedProject, Quality};

use crate::camera::Pose;
use crate::interact::{Action, Interactive};
use crate::merge::MergeRoot;
use crate::neon::{self, Canvas, NeonColor};
use crate::quality::QualityState;
use crate::scene::{
    DARTBOARD_POS, Kit, PendingCover, ROOM_HALF_X, ROOM_HALF_Z, ROOM_HEIGHT, TABLE_CENTER,
};
use crate::textures::Surf;

/// The window in the front wall: x and y extents.
const WIN_X: (f32, f32) = (-5.5, -4.0);
const WIN_Y: (f32, f32) = (1.35, 2.45);
/// Inner face of the front wall.
const FZ: f32 = 5.0;

/// Minor props: hidden on the Low tier.
#[derive(Component)]
pub struct Clutter;

#[derive(Component)]
struct FanRotor;

pub struct FillPlugin;

impl Plugin for FillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spin_fan, clutter_visibility));
    }
}

fn spin_fan(time: Res<Time>, mut fans: Query<&mut Transform, With<FanRotor>>) {
    for mut t in &mut fans {
        t.rotate_y(time.delta_secs() * 0.9);
    }
}

fn clutter_visibility(q: Res<QualityState>, mut roots: Query<&mut Visibility, With<Clutter>>) {
    let want = if q.tier == Quality::Low {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut v in &mut roots {
        if *v != want {
            *v = want;
        }
    }
}

/// The front wall, with a hole for the window.
pub fn front_wall(
    k: &mut Kit,
    root: Entity,
    centre: Vec3,
    size: Vec3,
    m: &Handle<StandardMaterial>,
) {
    let (hx, h) = (size.x / 2.0, size.y);
    let (x0, x1) = WIN_X;
    let (y0, y1) = WIN_Y;
    let piece = |k: &mut Kit, xa: f32, xb: f32, ya: f32, yb: f32| {
        k.cube(
            root,
            Vec3::new((xa + xb) / 2.0, (ya + yb) / 2.0, centre.z),
            Vec3::new(xb - xa, yb - ya, size.z),
            m,
        );
    };
    piece(k, -hx, x0, 0.0, h);
    piece(k, x1, hx, 0.0, h);
    piece(k, x0, x1, 0.0, y0);
    piece(k, x0, x1, y1, h);
}

pub fn fill(k: &mut Kit, root: Entity, featured: &[FeaturedProject], images: &mut Assets<Image>) {
    let clutter = k.group(Some(root), Transform::IDENTITY);
    k.c.entity(clutter).insert((MergeRoot, Clutter));

    back_bar(k, root, clutter, images);
    posters(k, root, featured);
    vinyl(k, root);
    window(k, root, clutter);
    skyline(k, root);
    back_room_door(k, root);
    corner_crates(k, root, images);
    pinball(k, root);
    coat_stand(k, root);
    darts(k, root, clutter);
    ceiling(k, root, clutter);
    floor(k, root, clutter, images);
    black_label(k, root, images);
    portrait(k, root);
}

// ---------------------------------------------------------------------------
// Back-bar: mirror, lit shelves, the house bottle, a neon line
// ---------------------------------------------------------------------------

fn back_bar(k: &mut Kit, root: Entity, clutter: Entity, images: &mut Assets<Image>) {
    let wood = k.surf(Surf::Wood, Color::srgb(0.45, 0.3, 0.22), 1.6);
    let slat = k.surf(Surf::Wood, Color::srgb(0.32, 0.2, 0.14), 1.6);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let dark = k.mat(Color::srgb(0.02, 0.018, 0.016), 0.5, 0.0);
    let top = k.surf(Surf::Wood, Color::srgb(0.22, 0.13, 0.09), 1.6);
    let mirror = k.mat(Color::srgb(0.035, 0.035, 0.04), 0.06, 1.0);
    let shelf_glass = k.mat(Color::srgb(0.1, 0.12, 0.12), 0.05, 0.0);
    let led = k.glow(Color::srgb(1.0, 0.6, 0.28), 5.0);
    let back_led = k.glow(Color::srgb(1.0, 0.55, 0.25), 2.2);

    // Slatted wood surround above the wainscot, the mirror strip on it.
    k.cube(
        root,
        Vec3::new(0.0, 2.07, FZ - 0.01),
        Vec3::new(4.8, 2.06, 0.02),
        &slat,
    );
    let mut x = -2.36;
    while x <= 2.37 {
        k.cube(
            root,
            Vec3::new(x, 2.07, FZ - 0.027),
            Vec3::new(0.05, 2.06, 0.015),
            &wood,
        );
        x += 0.118;
    }
    k.cube(
        root,
        Vec3::new(0.0, 1.875, FZ - 0.04),
        Vec3::new(4.1, 1.45, 0.012),
        &mirror,
    );
    for (p, s) in [
        (Vec3::new(0.0, 2.605, 0.0), Vec3::new(4.14, 0.02, 0.02)),
        (Vec3::new(0.0, 1.145, 0.0), Vec3::new(4.14, 0.02, 0.02)),
        (Vec3::new(-2.06, 1.875, 0.0), Vec3::new(0.02, 1.48, 0.02)),
        (Vec3::new(2.06, 1.875, 0.0), Vec3::new(0.02, 1.48, 0.02)),
    ] {
        k.cube(root, p + Vec3::Z * (FZ - 0.045), s, &brass);
    }

    // Low cabinet with a polished top.
    let (cz, depth) = (FZ - 0.28, 0.5);
    k.bevel(
        root,
        Vec3::new(0.0, 0.48, cz),
        Vec3::new(4.4, 0.92, depth),
        0.015,
        &wood,
    );
    k.bevel(
        root,
        Vec3::new(0.0, 0.96, cz - 0.03),
        Vec3::new(4.5, 0.045, depth + 0.08),
        0.012,
        &top,
    );
    let front = cz - depth / 2.0;
    for x in [-1.1, 0.0, 1.1] {
        k.cube(
            root,
            Vec3::new(x, 0.49, front - 0.004),
            Vec3::new(0.018, 0.78, 0.01),
            &dark,
        );
    }
    for x in [-1.65, -0.55, 0.55, 1.65] {
        k.cube(
            root,
            Vec3::new(x, 0.49, front - 0.004),
            Vec3::new(0.94, 0.74, 0.006),
            &slat,
        );
        let hx = if x < 0.0 { x + 0.4 } else { x - 0.4 };
        k.cyl_t(
            root,
            Transform::from_xyz(hx, 0.62, front - 0.02),
            0.008,
            0.14,
            &brass,
        );
    }
    k.cube(
        root,
        Vec3::new(0.0, 0.04, front - 0.01),
        Vec3::new(4.4, 0.08, 0.02),
        &dark,
    );
    k.shadow(root, Vec3::new(0.0, 0.0, cz - 0.05), Vec2::new(4.8, 0.9));

    // Three glass shelves: warm LED under each front edge, a backlight
    // strip behind the bottles.
    let shelves = [1.4, 1.86, 2.32];
    let sz = FZ - 0.16;
    for y in shelves {
        k.cube(
            root,
            Vec3::new(0.0, y, sz),
            Vec3::new(3.9, 0.018, 0.22),
            &shelf_glass,
        );
        k.cube(
            root,
            Vec3::new(0.0, y - 0.013, sz - 0.1),
            Vec3::new(3.86, 0.006, 0.012),
            &led,
        );
        k.cube(
            root,
            Vec3::new(0.0, y + 0.012, FZ - 0.055),
            Vec3::new(3.86, 0.008, 0.01),
            &back_led,
        );
        for x in [-1.8, 0.0, 1.8] {
            k.cube(
                root,
                Vec3::new(x, y - 0.03, FZ - 0.1),
                Vec3::new(0.02, 0.05, 0.1),
                &brass,
            );
        }
    }

    // Generic bottles: four shapes, a handful of glass colours, no brands.
    let glass: Vec<_> = [
        Color::srgb(0.32, 0.14, 0.03),
        Color::srgb(0.05, 0.2, 0.07),
        Color::srgb(0.5, 0.52, 0.5),
        Color::srgb(0.3, 0.03, 0.04),
        Color::srgb(0.04, 0.08, 0.3),
        Color::srgb(0.45, 0.3, 0.08),
    ]
    .into_iter()
    .map(|c| k.mat(c, 0.08, 0.0))
    .collect();
    let caps = [
        k.mat(Color::srgb(0.8, 0.6, 0.25), 0.3, 1.0),
        k.mat(Color::srgb(0.02, 0.02, 0.02), 0.4, 0.0),
        k.mat(Color::srgb(0.55, 0.05, 0.05), 0.4, 0.0),
    ];
    let labels = [
        k.mat(Color::srgb(0.85, 0.8, 0.68), 0.8, 0.0),
        k.mat(Color::srgb(0.08, 0.08, 0.08), 0.8, 0.0),
        k.mat(Color::srgb(0.75, 0.6, 0.3), 0.8, 0.0),
    ];
    let mut r = Rng(11);
    for (si, y) in shelves.into_iter().enumerate() {
        let mut x = -1.85 + r.f() * 0.05;
        while x < 1.85 {
            // The house bottle has the middle of the middle shelf to itself.
            if si == 1 && x.abs() < 0.32 {
                x = 0.32;
                continue;
            }
            let kind = (r.f() * 4.0) as u8;
            let g = &glass[(r.f() * glass.len() as f32) as usize % glass.len()];
            let cap = &caps[(r.f() * 3.0) as usize % 3];
            let label = (r.f() < 0.7).then(|| labels[(r.f() * 3.0) as usize % 3].clone());
            let base = Vec3::new(x, y + 0.009, sz + 0.02 - r.f() * 0.04);
            bottle(k, root, base, kind, g, cap, label.as_ref());
            x += 0.13 + r.f() * 0.09;
        }
    }
    reserve(k, root, Vec3::new(0.0, shelves[1] + 0.009, sz), images);

    // A neon line above it all.
    neon::sign(
        k,
        root,
        "BUILT IN RUST",
        Transform::from_xyz(0.0, 2.86, FZ - 0.04).with_rotation(Quat::from_rotation_y(PI)),
        0.03,
        NeonColor::Amber,
        None,
    );
    // Warm, low light on the bottles; the corner stays calm otherwise.
    k.c.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.68, 0.4),
            intensity: 9_000.0,
            range: 2.2,
            ..default()
        },
        Transform::from_xyz(0.0, 1.95, FZ - 0.6),
        ChildOf(root),
    ));

    // Bar-top bits: a mat, two upturned tumblers, a shaker.
    let mat = k.mat(Color::srgb(0.03, 0.03, 0.03), 1.0, 0.0);
    let tumbler = k.mat(Color::srgb(0.2, 0.22, 0.24), 0.05, 0.0);
    let steel = k.mat(Color::srgb(0.75, 0.75, 0.78), 0.25, 1.0);
    let top_y = 0.985;
    k.cube(
        clutter,
        Vec3::new(1.2, top_y + 0.004, cz - 0.08),
        Vec3::new(0.5, 0.008, 0.16),
        &mat,
    );
    for dx in [1.05, 1.2] {
        k.cyl(
            clutter,
            Vec3::new(dx, top_y + 0.05, cz - 0.08),
            0.035,
            0.085,
            &tumbler,
        );
    }
    k.cyl(
        clutter,
        Vec3::new(1.4, top_y + 0.1, cz - 0.08),
        0.045,
        0.19,
        &steel,
    );
    k.cyl(
        clutter,
        Vec3::new(1.4, top_y + 0.215, cz - 0.08),
        0.03,
        0.04,
        &steel,
    );
    k.cyl(
        clutter,
        Vec3::new(-1.3, top_y + 0.06, cz - 0.1),
        0.05,
        0.11,
        &tumbler,
    );
}

fn bottle(
    k: &mut Kit,
    parent: Entity,
    base: Vec3,
    kind: u8,
    glass: &Handle<StandardMaterial>,
    cap: &Handle<StandardMaterial>,
    label: Option<&Handle<StandardMaterial>>,
) {
    let neck = |k: &mut Kit, y: f32, r: f32, h: f32| {
        k.cyl(parent, base + Vec3::Y * (y + h / 2.0), r, h, glass);
        k.cyl(
            parent,
            base + Vec3::Y * (y + h + 0.012),
            r * 1.15,
            0.024,
            cap,
        );
    };
    match kind {
        // Wine-ish: round body, sloped shoulder.
        0 => {
            k.cyl(parent, base + Vec3::Y * 0.095, 0.033, 0.19, glass);
            let sh = k.meshes.add(ConicalFrustum {
                radius_top: 0.013,
                radius_bottom: 0.033,
                height: 0.05,
            });
            k.mesh(
                parent,
                sh,
                Transform::from_translation(base + Vec3::Y * 0.215),
                glass,
            );
            neck(k, 0.24, 0.012, 0.07);
            if let Some(l) = label {
                k.cube(
                    parent,
                    base + Vec3::new(0.0, 0.09, -0.033),
                    Vec3::new(0.045, 0.06, 0.004),
                    l,
                );
            }
        }
        // Square-shouldered spirit.
        1 => {
            k.bevel(
                parent,
                base + Vec3::Y * 0.075,
                Vec3::new(0.075, 0.15, 0.075),
                0.01,
                glass,
            );
            neck(k, 0.15, 0.015, 0.035);
            if let Some(l) = label {
                k.cube(
                    parent,
                    base + Vec3::new(0.0, 0.08, -0.039),
                    Vec3::new(0.058, 0.07, 0.004),
                    l,
                );
            }
        }
        // Round liqueur.
        2 => {
            k.ball(parent, base + Vec3::Y * 0.058, 0.058, glass);
            neck(k, 0.11, 0.012, 0.06);
        }
        // Tall and slim.
        _ => {
            k.cyl(parent, base + Vec3::Y * 0.125, 0.024, 0.25, glass);
            neck(k, 0.25, 0.009, 0.06);
            if let Some(l) = label {
                k.cube(
                    parent,
                    base + Vec3::new(0.0, 0.14, -0.024),
                    Vec3::new(0.034, 0.08, 0.004),
                    l,
                );
            }
        }
    }
}

/// ARTHUB RESERVE: a squat decanter on a brass plinth, label and all.
fn reserve(k: &mut Kit, root: Entity, at: Vec3, images: &mut Assets<Image>) {
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let plinth = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.2, 0.0);
    let amber = k.mat(Color::srgb(0.42, 0.17, 0.03), 0.05, 0.0);
    let pin = k.glow(Color::srgb(1.0, 0.75, 0.45), 12.0);
    k.bevel(
        root,
        at + Vec3::Y * 0.025,
        Vec3::new(0.26, 0.05, 0.17),
        0.006,
        &plinth,
    );
    k.cube(
        root,
        at + Vec3::new(0.0, 0.051, -0.086),
        Vec3::new(0.24, 0.004, 0.004),
        &brass,
    );
    k.cube(
        root,
        at + Vec3::new(0.0, 0.002, -0.086),
        Vec3::new(0.24, 0.004, 0.004),
        &pin,
    );
    let b = at + Vec3::Y * 0.05;
    k.bevel(
        root,
        b + Vec3::Y * 0.1,
        Vec3::new(0.13, 0.2, 0.11),
        0.03,
        &amber,
    );
    let sh = k.meshes.add(ConicalFrustum {
        radius_top: 0.02,
        radius_bottom: 0.05,
        height: 0.04,
    });
    k.mesh(
        root,
        sh,
        Transform::from_translation(b + Vec3::Y * 0.215),
        &amber,
    );
    k.cyl(root, b + Vec3::Y * 0.255, 0.018, 0.05, &amber);
    k.cyl(root, b + Vec3::Y * 0.285, 0.026, 0.012, &brass);
    k.ball(root, b + Vec3::Y * 0.315, 0.032, &brass);

    let mut c = Canvas::new(120, 150, [222, 212, 188]);
    let ink = [24, 18, 14];
    let gold = [150, 105, 40];
    c.rect(6.0, 6.0, 108.0, 3.0, gold);
    c.rect(6.0, 141.0, 108.0, 3.0, gold);
    c.text("ARTHUB", 60.0, 30.0, 22.0, ink);
    c.rect(30.0, 64.0, 60.0, 2.0, gold);
    c.text("RESERVE", 60.0, 76.0, 13.0, ink);
    c.text("SMALL BATCH", 60.0, 106.0, 8.0, gold);
    let label = k.mats.add(StandardMaterial {
        base_color_texture: Some(images.add(c.image())),
        perceptual_roughness: 0.7,
        ..default()
    });
    // Its own group facing the room, so the texture reads the right way round.
    let g = k.group(
        Some(root),
        Transform::from_translation(b + Vec3::new(0.0, 0.1, -0.057))
            .with_rotation(Quat::from_rotation_y(PI)),
    );
    k.cube(g, Vec3::ZERO, Vec3::new(0.095, 0.12, 0.003), &label);
}

// ---------------------------------------------------------------------------
// Flanks: posters of the featured work on the left, records on the right
// ---------------------------------------------------------------------------

fn posters(k: &mut Kit, root: Entity, featured: &[FeaturedProject]) {
    if featured.is_empty() {
        return;
    }
    let frame = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.4, 0.2);
    let lamp = k.glow(Color::srgb(1.0, 0.8, 0.55), 3.0);
    let size = Vec2::new(0.74, 0.46);
    for (i, y) in [2.12, 1.48].into_iter().enumerate() {
        let p = &featured[(i + 1) % featured.len()];
        let g = k.group(
            Some(root),
            Transform::from_xyz(-3.05, y, FZ - 0.005).with_rotation(Quat::from_rotation_y(PI)),
        );
        k.bevel(
            g,
            Vec3::Z * 0.015,
            Vec3::new(size.x + 0.05, size.y + 0.05, 0.03),
            0.006,
            &frame,
        );
        let art = k.mats.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25),
            perceptual_roughness: 0.35,
            ..default()
        });
        k.hints.exact.insert(art.id());
        k.cube(g, Vec3::Z * 0.032, Vec3::new(size.x, size.y, 0.004), &art);
        if let Some(path) = &p.cover {
            let image = k.assets.load::<Image>(path.clone());
            k.c.spawn((
                PendingCover {
                    image,
                    mat: art,
                    crt: false,
                },
                ChildOf(g),
            ));
        }
        if i == 0 {
            k.bevel(
                g,
                Vec3::new(0.0, size.y / 2.0 + 0.08, 0.08),
                Vec3::new(size.x * 0.5, 0.03, 0.05),
                0.01,
                &frame,
            );
            k.cube(
                g,
                Vec3::new(0.0, size.y / 2.0 + 0.064, 0.08),
                Vec3::new(size.x * 0.46, 0.004, 0.03),
                &lamp,
            );
        }
    }
}

fn sleeve_mats(k: &mut Kit) -> Vec<Handle<StandardMaterial>> {
    [
        Color::srgb(0.75, 0.3, 0.2),
        Color::srgb(0.15, 0.2, 0.3),
        Color::srgb(0.85, 0.75, 0.55),
        Color::srgb(0.2, 0.35, 0.25),
        Color::srgb(0.08, 0.08, 0.08),
        Color::srgb(0.6, 0.15, 0.35),
        Color::srgb(0.9, 0.55, 0.15),
    ]
    .into_iter()
    .map(|c| k.mat(c, 0.8, 0.0))
    .collect()
}

fn vinyl(k: &mut Kit, root: Entity) {
    let wood = k.surf(Surf::Wood, Color::srgb(0.7, 0.45, 0.3), 2.0);
    let back = k.mat(Color::srgb(0.03, 0.025, 0.02), 0.9, 0.0);
    let black = k.mat(Color::srgb(0.015, 0.015, 0.015), 0.35, 0.0);
    let vinyl = k.mat(Color::srgb(0.01, 0.01, 0.012), 0.25, 0.0);
    let chrome = k.mat(Color::srgb(0.8, 0.8, 0.82), 0.2, 1.0);
    let label = k.mat(Color::srgb(0.85, 0.2, 0.15), 0.7, 0.0);
    let sleeves = sleeve_mats(k);

    // Open record cabinet: two cubbies of spines.
    let (cx, w, h, d) = (3.1, 1.06, 0.72, 0.42);
    let cz = FZ - d / 2.0 - 0.01;
    let t = 0.025;
    k.cube(
        root,
        Vec3::new(cx, h - t / 2.0, cz),
        Vec3::new(w, t, d),
        &wood,
    );
    k.cube(root, Vec3::new(cx, 0.04, cz), Vec3::new(w, 0.08, d), &wood);
    k.cube(
        root,
        Vec3::new(cx, h / 2.0, cz),
        Vec3::new(w - 0.04, 0.02, d - 0.02),
        &wood,
    );
    for sx in [-1.0, 0.0, 1.0] {
        k.cube(
            root,
            Vec3::new(cx + sx * (w - t) / 2.0, h / 2.0, cz),
            Vec3::new(t, h, d),
            &wood,
        );
    }
    k.cube(
        root,
        Vec3::new(cx, h / 2.0, FZ - 0.015),
        Vec3::new(w, h, 0.01),
        &back,
    );
    let mut r = Rng(5);
    for (y0, y1) in [(0.08, 0.35), (0.37, h - t)] {
        for (xa, xb) in [
            (cx - w / 2.0 + t, cx - t / 2.0),
            (cx + t / 2.0, cx + w / 2.0 - t),
        ] {
            let mut x = xa + 0.004;
            while x < xb - 0.006 {
                let th = 0.004 + r.f() * 0.004;
                let tall = (y1 - y0) * (0.9 + r.f() * 0.08);
                let m = &sleeves[(r.f() * sleeves.len() as f32) as usize % sleeves.len()];
                k.cube(
                    root,
                    Vec3::new(x + th / 2.0, y0 + tall / 2.0, cz + 0.01),
                    Vec3::new(th, tall, d - 0.07),
                    m,
                );
                x += th + 0.0015;
            }
        }
    }
    k.shadow(root, Vec3::new(cx, 0.0, cz), Vec2::new(1.3, 0.7));

    // Turntable on top.
    let tt = Vec3::new(cx - 0.12, h, cz);
    k.bevel(
        root,
        tt + Vec3::Y * 0.04,
        Vec3::new(0.46, 0.08, 0.36),
        0.01,
        &black,
    );
    k.cyl(root, tt + Vec3::new(-0.04, 0.09, 0.0), 0.15, 0.02, &chrome);
    k.cyl(
        root,
        tt + Vec3::new(-0.04, 0.102, 0.0),
        0.147,
        0.004,
        &vinyl,
    );
    k.cyl(
        root,
        tt + Vec3::new(-0.04, 0.105, 0.0),
        0.045,
        0.002,
        &label,
    );
    let pivot = tt + Vec3::new(0.16, 0.1, 0.12);
    k.cyl(root, pivot, 0.018, 0.03, &chrome);
    k.rod(
        root,
        pivot + Vec3::Y * 0.01,
        tt + Vec3::new(0.04, 0.115, -0.08),
        0.004,
        &chrome,
    );
    // A few sleeves lying flat beside it.
    for i in 0..3 {
        let t = Transform::from_translation(Vec3::new(cx + 0.33, h + 0.004 + i as f32 * 0.006, cz))
            .with_rotation(Quat::from_rotation_y(0.1 * i as f32 - 0.05))
            .with_scale(Vec3::new(0.31, 0.005, 0.31));
        k.cube_t(root, t, &sleeves[i + 2]);
    }

    // Two picture ledges with sleeves face-out.
    let face = Quat::from_rotation_x(0.09);
    for (row, y) in [1.5, 2.08].into_iter().enumerate() {
        k.cube(
            root,
            Vec3::new(cx, y, FZ - 0.04),
            Vec3::new(1.1, 0.03, 0.08),
            &wood,
        );
        k.cube(
            root,
            Vec3::new(cx, y + 0.03, FZ - 0.078),
            Vec3::new(1.1, 0.03, 0.006),
            &wood,
        );
        let n = 3 - row;
        for i in 0..n {
            let x = cx + (i as f32 - (n - 1) as f32 / 2.0) * 0.35;
            let m = &sleeves[(i * 2 + row) % sleeves.len()];
            let at = Vec3::new(x, y + 0.17, FZ - 0.05);
            k.cube_t(
                root,
                Transform::from_translation(at)
                    .with_rotation(face)
                    .with_scale(Vec3::new(0.31, 0.31, 0.006)),
                m,
            );
            // Simple sleeve art: a disc and a bar in a contrasting colour.
            let ink = &sleeves[(i * 2 + row + 3) % sleeves.len()];
            let front = face * -Vec3::Z * 0.0045;
            k.cyl_t(
                root,
                Transform::from_translation(at + front + face * Vec3::new(0.0, 0.03, 0.0))
                    .with_rotation(face * Quat::from_rotation_x(FRAC_PI_2)),
                0.085,
                0.002,
                ink,
            );
            k.cube_t(
                root,
                Transform::from_translation(at + front + face * Vec3::new(0.0, -0.11, 0.0))
                    .with_rotation(face)
                    .with_scale(Vec3::new(0.24, 0.025, 0.002)),
                ink,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Window and the city beyond it
// ---------------------------------------------------------------------------

fn window(k: &mut Kit, root: Entity, clutter: Entity) {
    let (x0, x1) = WIN_X;
    let (y0, y1) = WIN_Y;
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let (w, h) = (x1 - x0, y1 - y0);
    let paint = k.mat(Color::srgb(0.05, 0.045, 0.045), 0.6, 0.0);
    let wood = k.surf(Surf::Wood, Color::srgb(0.55, 0.38, 0.28), 2.0);
    let wz = FZ + 0.1;
    // Reveals inside the wall thickness.
    k.cube(
        root,
        Vec3::new(x0 + 0.02, cy, wz),
        Vec3::new(0.04, h, 0.2),
        &paint,
    );
    k.cube(
        root,
        Vec3::new(x1 - 0.02, cy, wz),
        Vec3::new(0.04, h, 0.2),
        &paint,
    );
    k.cube(
        root,
        Vec3::new(cx, y1 - 0.02, wz),
        Vec3::new(w, 0.04, 0.2),
        &paint,
    );
    // Architrave and a deep sill.
    for sx in [x0 - 0.04, x1 + 0.04] {
        k.cube(
            root,
            Vec3::new(sx, cy + 0.04, FZ - 0.012),
            Vec3::new(0.08, h + 0.16, 0.025),
            &wood,
        );
    }
    k.cube(
        root,
        Vec3::new(cx, y1 + 0.04, FZ - 0.012),
        Vec3::new(w + 0.16, 0.08, 0.025),
        &wood,
    );
    k.bevel(
        root,
        Vec3::new(cx, y0 - 0.005, FZ + 0.02),
        Vec3::new(w + 0.24, 0.05, 0.36),
        0.01,
        &wood,
    );
    // Frame and mullions.
    let fz = FZ + 0.14;
    for (p, s) in [
        (Vec3::new(cx, cy, fz), Vec3::new(0.045, h, 0.05)),
        (Vec3::new(cx, cy + 0.12, fz), Vec3::new(w, 0.045, 0.05)),
        (Vec3::new(x0 + 0.06, cy, fz), Vec3::new(0.04, h, 0.05)),
        (Vec3::new(x1 - 0.06, cy, fz), Vec3::new(0.04, h, 0.05)),
        (Vec3::new(cx, y0 + 0.04, fz), Vec3::new(w, 0.04, 0.05)),
        (Vec3::new(cx, y1 - 0.06, fz), Vec3::new(w, 0.04, 0.05)),
    ] {
        k.cube(root, p, s, &paint);
    }
    let glass = k.mats.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.14, 0.18, 0.16),
        perceptual_roughness: 0.04,
        reflectance: 0.7,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    k.cube(
        root,
        Vec3::new(cx, cy, fz + 0.005),
        Vec3::new(w - 0.1, h - 0.1, 0.004),
        &glass,
    );

    // A candle in a jar on the sill.
    let jar = k.mat(Color::srgb(0.3, 0.12, 0.08), 0.1, 0.0);
    let flame = k.glow(Color::srgb(1.0, 0.6, 0.25), 6.0);
    k.cyl(
        clutter,
        Vec3::new(x0 + 0.25, y0 + 0.07, FZ - 0.05),
        0.04,
        0.09,
        &jar,
    );
    k.ball(
        clutter,
        Vec3::new(x0 + 0.25, y0 + 0.13, FZ - 0.05),
        0.01,
        &flame,
    );

    // A stool parked under it.
    let leather = k.surf(Surf::Leather, Color::srgb(0.6, 0.18, 0.12), 4.0);
    let steel = k.mat(Color::srgb(0.08, 0.08, 0.09), 0.4, 1.0);
    let s = Vec3::new(cx + 0.1, 0.0, FZ - 0.55);
    k.cyl(root, s + Vec3::Y * 0.72, 0.17, 0.06, &leather);
    k.cyl(root, s + Vec3::Y * 0.37, 0.025, 0.7, &steel);
    k.cyl(root, s + Vec3::Y * 0.012, 0.19, 0.024, &steel);
    k.cyl(root, s + Vec3::Y * 0.25, 0.14, 0.012, &steel);
    k.shadow(root, s, Vec2::splat(0.55));
}

/// Silhouettes and lit windows in three layers outside: real depth, so the
/// view through the window shifts as the camera moves.
fn skyline(k: &mut Kit, root: Entity) {
    let mass = k.mat(Color::srgb(0.01, 0.01, 0.014), 0.9, 0.0);
    // Sky: a backdrop wall and a side wall, bands brightening to the horizon.
    let bands = [
        (-30.0, -2.0, Color::srgb(0.08, 0.04, 0.08), 2.0),
        (-2.0, 3.0, Color::srgb(0.6, 0.2, 0.38), 3.0),
        (3.0, 7.0, Color::srgb(0.35, 0.14, 0.4), 2.4),
        (7.0, 12.0, Color::srgb(0.14, 0.08, 0.3), 2.0),
        (12.0, 40.0, Color::srgb(0.04, 0.04, 0.1), 2.0),
    ];
    for (ya, yb, col, s) in bands {
        let m = k.glow(col, s);
        let (cy, hh) = ((ya + yb) / 2.0, yb - ya);
        k.cube(
            root,
            Vec3::new(-25.0, cy, 24.0),
            Vec3::new(70.0, hh, 0.1),
            &m,
        );
        k.cube(
            root,
            Vec3::new(-40.0, cy, 14.5),
            Vec3::new(0.1, hh, 19.0),
            &m,
        );
    }
    let warm = k.glow(Color::srgb(1.0, 0.7, 0.35), 3.0);
    let cool = k.glow(Color::srgb(0.5, 0.75, 1.0), 2.6);
    let red = k.glow(Color::srgb(1.0, 0.1, 0.05), 6.0);
    let mut r = Rng(42);
    // (depth of the building fronts, x range)
    for (layer, z) in [(0, 8.0), (1, 12.5), (2, 18.0)] {
        let mut x = -40.0 + r.f() * 3.0;
        while x < 14.0 {
            let bw = 2.0 + r.f() * (3.0 + layer as f32 * 2.0);
            // The nearest roofs sit below the sill, so the view clears them.
            let top = [-3.0, -1.0, 1.0][layer] + r.f() * [4.0, 5.0, 7.0][layer];
            let bd = 4.0;
            let bottom = -25.0;
            k.cube(
                root,
                Vec3::new(x + bw / 2.0, (top + bottom) / 2.0, z + bd / 2.0),
                Vec3::new(bw, top - bottom, bd),
                &mass,
            );
            // A grid of windows, a quarter of them lit.
            let (cw, ch) = (0.55, 0.8);
            let cols = ((bw - 0.4) / cw) as i32;
            let mut wy = top - 0.6;
            while wy > -6.0 {
                for c in 0..cols {
                    if r.f() < 0.32 {
                        let wx = x + 0.2 + (c as f32 + 0.5) * cw;
                        let m = if r.f() < 0.75 { &warm } else { &cool };
                        k.cube(
                            root,
                            Vec3::new(wx, wy, z - 0.02),
                            Vec3::new(0.3, 0.42, 0.04),
                            m,
                        );
                    }
                }
                wy -= ch;
            }
            if top > 5.0 && r.f() < 0.6 {
                k.ball(
                    root,
                    Vec3::new(x + bw / 2.0, top + 0.3, z + bd / 2.0),
                    0.12,
                    &red,
                );
            }
            x += bw + 0.3 + r.f() * 1.5;
        }
    }
}

// ---------------------------------------------------------------------------
// Back-room door, corners
// ---------------------------------------------------------------------------

fn back_room_door(k: &mut Kit, root: Entity) {
    let door = k.group(
        Some(root),
        Transform::from_xyz(5.15, 0.0, FZ - 0.005).with_rotation(Quat::from_rotation_y(PI)),
    );
    let frame = k.mat(Color::srgb(0.04, 0.035, 0.03), 0.5, 0.0);
    let panel = k.surf(Surf::Wood, Color::srgb(0.4, 0.22, 0.14), 1.6);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    k.cube(
        door,
        Vec3::new(0.0, 1.11, 0.02),
        Vec3::new(1.16, 2.22, 0.04),
        &frame,
    );
    k.cube(
        door,
        Vec3::new(0.0, 1.055, 0.05),
        Vec3::new(0.94, 2.07, 0.04),
        &panel,
    );
    for y in [0.58, 1.5] {
        k.bevel(
            door,
            Vec3::new(0.0, y, 0.072),
            Vec3::new(0.7, 0.78, 0.02),
            0.01,
            &panel,
        );
    }
    k.cube(
        door,
        Vec3::new(0.0, 0.12, 0.072),
        Vec3::new(0.9, 0.18, 0.005),
        &brass,
    );
    k.ball(door, Vec3::new(0.38, 1.0, 0.09), 0.03, &brass);
    k.cube(
        door,
        Vec3::new(0.38, 1.0, 0.075),
        Vec3::new(0.05, 0.16, 0.006),
        &brass,
    );
    k.cube(
        door,
        Vec3::new(0.0, 1.03, 0.085),
        Vec3::new(0.3, 0.075, 0.004),
        &brass,
    );
    // A caged bulkhead lamp over it.
    let lamp = k.glow(Color::srgb(1.0, 0.75, 0.45), 4.0);
    k.cyl_t(
        door,
        Transform::from_xyz(0.0, 2.42, 0.03).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        0.07,
        0.03,
        &frame,
    );
    k.ball(door, Vec3::new(0.0, 2.42, 0.07), 0.045, &lamp);
    for a in [0.0, FRAC_PI_2] {
        k.cube_t(
            door,
            Transform::from_xyz(0.0, 2.42, 0.075)
                .with_rotation(Quat::from_rotation_z(a))
                .with_scale(Vec3::new(0.11, 0.006, 0.006)),
            &frame,
        );
    }
    // Light leaking from under it.
    let leak = k.glow(Color::srgb(1.0, 0.62, 0.3), 9.0);
    let spill = k.glow(Color::srgb(1.0, 0.6, 0.3), 0.5);
    k.cube(
        door,
        Vec3::new(0.0, 0.009, 0.05),
        Vec3::new(0.92, 0.014, 0.03),
        &leak,
    );
    k.cube(
        door,
        Vec3::new(0.0, 0.0012, 0.13),
        Vec3::new(0.94, 0.002, 0.12),
        &spill,
    );
    k.c.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.62, 0.32),
            intensity: 2_000.0,
            range: 1.2,
            ..default()
        },
        Transform::from_xyz(0.0, 0.05, 0.3),
        ChildOf(door),
    ));
}

fn corner_crates(k: &mut Kit, root: Entity, images: &mut Assets<Image>) {
    let wood = k.surf(Surf::Wood, Color::srgb(0.75, 0.55, 0.38), 2.2);
    let slat = k.surf(Surf::Wood, Color::srgb(0.55, 0.38, 0.25), 2.2);
    let crate_ = |k: &mut Kit, at: Vec3, s: f32, yaw: f32| {
        let g = k.group(
            Some(root),
            Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw)),
        );
        k.bevel(g, Vec3::Y * s / 2.0, Vec3::splat(s), 0.01, &wood);
        for f in 0..4 {
            let rot = Quat::from_rotation_y(f as f32 * FRAC_PI_2);
            for i in 0..3 {
                let y = s * (0.2 + i as f32 * 0.3);
                k.cube_t(
                    g,
                    Transform::from_translation(Vec3::Y * y + rot * Vec3::Z * (s / 2.0 + 0.004))
                        .with_rotation(rot)
                        .with_scale(Vec3::new(s * 0.94, s * 0.2, 0.012)),
                    &slat,
                );
            }
        }
    };
    crate_(k, Vec3::new(6.45, 0.0, 4.5), 0.62, 0.08);
    crate_(k, Vec3::new(6.4, 0.62, 4.52), 0.46, -0.25);
    k.shadow(root, Vec3::new(6.45, 0.0, 4.5), Vec2::splat(0.95));

    // A combo amp against the wall, facing the room.
    let amp = k.group(
        Some(root),
        Transform::from_xyz(6.78, 0.0, 3.85).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    );
    let tolex = k.surf(Surf::Leather, Color::srgb(0.12, 0.1, 0.09), 4.0);
    let cloth = k.surf(Surf::Felt, Color::srgb(0.25, 0.2, 0.15), 3.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let knob = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.4, 0.0);
    let jewel = k.glow(Color::srgb(1.0, 0.1, 0.05), 5.0);
    k.bevel(
        amp,
        Vec3::new(0.0, 0.3, 0.0),
        Vec3::new(0.66, 0.56, 0.3),
        0.03,
        &tolex,
    );
    k.cube(
        amp,
        Vec3::new(0.0, 0.25, 0.151),
        Vec3::new(0.56, 0.38, 0.004),
        &cloth,
    );
    k.cube(
        amp,
        Vec3::new(0.0, 0.5, 0.151),
        Vec3::new(0.56, 0.08, 0.004),
        &brass,
    );
    for i in 0..6 {
        k.cyl_t(
            amp,
            Transform::from_xyz(-0.2 + i as f32 * 0.07, 0.5, 0.16)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            0.014,
            0.02,
            &knob,
        );
    }
    k.ball(amp, Vec3::new(0.24, 0.5, 0.156), 0.01, &jewel);
    k.cube(
        amp,
        Vec3::new(0.0, 0.6, 0.0),
        Vec3::new(0.16, 0.025, 0.03),
        &knob,
    );
    k.shadow(amp, Vec3::ZERO, Vec2::new(0.8, 0.45));

    // House rules, above the crates.
    let mut c = Canvas::new(128, 176, [18, 16, 20]);
    let cream = [226, 214, 190];
    let pink = [255, 60, 130];
    c.text("HOUSE", 64.0, 14.0, 20.0, pink);
    c.text("RULES", 64.0, 42.0, 20.0, pink);
    for i in 0..5 {
        let y = 82.0 + i as f32 * 16.0;
        c.rect(16.0, y, 8.0, 8.0, cream);
        c.rect(30.0, y + 2.0, 50.0 + ((i * 37) % 40) as f32, 4.0, cream);
    }
    c.rect(8.0, 4.0, 112.0, 2.0, cream);
    c.rect(8.0, 170.0, 112.0, 2.0, cream);
    gig_poster(
        k,
        root,
        Transform::from_xyz(6.98, 1.95, 4.25).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        Vec2::new(0.5, 0.69),
        c.image(),
        images,
    );
}

fn gig_poster(
    k: &mut Kit,
    root: Entity,
    t: Transform,
    size: Vec2,
    image: Image,
    images: &mut Assets<Image>,
) {
    let g = k.group(Some(root), t);
    let frame = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.4, 0.2);
    let art = k.mats.add(StandardMaterial {
        base_color_texture: Some(images.add(image)),
        perceptual_roughness: 0.6,
        ..default()
    });
    k.bevel(
        g,
        Vec3::Z * 0.012,
        Vec3::new(size.x + 0.04, size.y + 0.04, 0.024),
        0.005,
        &frame,
    );
    k.cube(g, Vec3::Z * 0.026, Vec3::new(size.x, size.y, 0.003), &art);
}

/// A pinball machine in the front-left corner, beside the dartboard.
fn pinball(k: &mut Kit, root: Entity) {
    let g = k.group(
        Some(root),
        Transform::from_xyz(-6.25, 0.0, 4.55).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    );
    let body = k.mat(Color::srgb(0.08, 0.02, 0.05), 0.35, 0.0);
    let trim = k.mat(Color::srgb(0.8, 0.8, 0.82), 0.25, 1.0);
    let glass = k.mat(Color::srgb(0.02, 0.02, 0.03), 0.05, 0.0);
    let pink = k.glow(Color::srgb(1.0, 0.18, 0.53), 5.0);
    let cyan = k.glow(Color::srgb(0.13, 0.9, 1.0), 4.0);
    let amber = k.glow(Color::srgb(1.0, 0.62, 0.12), 4.0);
    for (x, z) in [(-0.3, -0.58), (0.3, -0.58), (-0.3, 0.58), (0.3, 0.58)] {
        k.cyl(g, Vec3::new(x, 0.36, z), 0.025, 0.72, &trim);
    }
    // Cabinet, tilted down toward the player.
    let tilt = Quat::from_rotation_x(0.06);
    k.bevel_t(
        g,
        Transform::from_xyz(0.0, 0.88, 0.0).with_rotation(tilt),
        Vec3::new(0.7, 0.3, 1.32),
        0.02,
        &body,
    );
    let top = Transform::from_xyz(0.0, 1.035, 0.0).with_rotation(tilt);
    k.cube_t(g, top.with_scale(Vec3::new(0.62, 0.01, 1.24)), &glass);
    let mut r = Rng(3);
    for i in 0..14 {
        let p = Vec3::new(r.f() * 0.5 - 0.25, 0.012, r.f() * 1.0 - 0.5);
        let m = [&pink, &cyan, &amber][i % 3];
        k.cube_t(
            g,
            Transform::from_translation(top.transform_point(p))
                .with_rotation(tilt)
                .with_scale(Vec3::new(0.03, 0.004, 0.05)),
            m,
        );
    }
    // Backbox with its lettered backglass.
    k.bevel(
        g,
        Vec3::new(0.0, 1.5, -0.6),
        Vec3::new(0.7, 0.72, 0.2),
        0.02,
        &body,
    );
    k.cube(
        g,
        Vec3::new(0.0, 1.52, -0.497),
        Vec3::new(0.6, 0.56, 0.006),
        &glass,
    );
    neon::stroke_text(
        k,
        g,
        "PINBALL",
        Transform::from_xyz(0.0, 1.6, -0.49),
        0.016,
        0.004,
        &pink,
    );
    for y in [1.32, 1.76] {
        k.cube(
            g,
            Vec3::new(0.0, y, -0.49),
            Vec3::new(0.54, 0.008, 0.004),
            &cyan,
        );
    }
    // Coin door and flipper buttons.
    k.cube(
        g,
        Vec3::new(0.0, 0.62, 0.665),
        Vec3::new(0.3, 0.32, 0.02),
        &trim,
    );
    for x in [-0.06, 0.06] {
        k.cube(
            g,
            Vec3::new(x, 0.7, 0.678),
            Vec3::new(0.035, 0.06, 0.006),
            &amber,
        );
    }
    for x in [-0.36, 0.36] {
        k.ball(g, Vec3::new(x, 0.9, 0.5), 0.02, &pink);
    }
    k.shadow(g, Vec3::ZERO, Vec2::new(0.9, 1.5));
}

fn coat_stand(k: &mut Kit, root: Entity) {
    let at = Vec3::new(-6.5, 0.0, -4.5);
    let wood = k.mat(Color::srgb(0.08, 0.04, 0.02), 0.45, 0.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let coat = k.surf(Surf::Leather, Color::srgb(0.18, 0.12, 0.09), 3.0);
    let wool = k.mat(Color::srgb(0.5, 0.12, 0.1), 0.95, 0.0);
    k.cyl(root, at + Vec3::Y * 0.9, 0.022, 1.8, &wood);
    k.ball(root, at + Vec3::Y * 1.82, 0.035, &wood);
    for i in 0..3 {
        let a = i as f32 * TAU / 3.0 + 0.3;
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        k.rod(
            root,
            at + Vec3::Y * 0.3,
            at + d * 0.3 + Vec3::Y * 0.02,
            0.016,
            &wood,
        );
    }
    for i in 0..4 {
        let a = i as f32 * TAU / 4.0;
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        k.rod(
            root,
            at + Vec3::Y * 1.66,
            at + d * 0.13 + Vec3::Y * 1.74,
            0.008,
            &brass,
        );
    }
    // A jacket on one hook, a beanie on another.
    let jacket = Transform::from_translation(at + Vec3::new(0.12, 1.3, 0.05))
        .with_rotation(Quat::from_rotation_y(0.4) * Quat::from_rotation_z(-0.06));
    k.bevel_t(root, jacket, Vec3::new(0.4, 0.75, 0.14), 0.05, &coat);
    k.bevel_t(
        root,
        jacket * Transform::from_xyz(0.0, 0.36, 0.0),
        Vec3::new(0.3, 0.08, 0.18),
        0.03,
        &coat,
    );
    k.ball(root, at + Vec3::new(-0.1, 1.73, 0.0), 0.075, &wool);
    k.shadow(root, at, Vec2::splat(0.7));
}

fn darts(k: &mut Kit, root: Entity, clutter: Entity) {
    let face = Quat::from_rotation_y(FRAC_PI_2); // local +z → world +x
    let cork = k.mat(Color::srgb(0.12, 0.08, 0.05), 1.0, 0.0);
    let ring = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.6, 0.0);
    let p = DARTBOARD_POS;
    // A scuffed cork surround behind the board.
    k.cyl_t(
        root,
        Transform::from_translation(Vec3::new(-ROOM_HALF_X + 0.03, p.y, p.z))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        0.5,
        0.02,
        &cork,
    );
    k.cyl_t(
        root,
        Transform::from_translation(Vec3::new(-ROOM_HALF_X + 0.025, p.y, p.z))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        0.53,
        0.02,
        &ring,
    );
    // Missed throws: pocks in the cork and the plaster.
    let pock = k.mat(Color::srgb(0.03, 0.02, 0.015), 1.0, 0.0);
    let mut r = Rng(17);
    for _ in 0..45 {
        let a = r.f() * TAU;
        let d = 0.3 + r.f() * r.f() * 0.4;
        let at = Vec3::new(-ROOM_HALF_X + 0.042, p.y + a.sin() * d, p.z + a.cos() * d);
        k.cube_t(
            clutter,
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_x(r.f() * PI))
                .with_scale(Vec3::new(0.004, 0.006 + r.f() * 0.01, 0.006)),
            &pock,
        );
    }
    // Three darts stuck in the board.
    let steel = k.mat(Color::srgb(0.7, 0.7, 0.72), 0.3, 1.0);
    let flight = k.mat(Color::srgb(0.85, 0.2, 0.15), 0.7, 0.0);
    for (dy, dz) in [(0.05, -0.02), (-0.03, 0.06), (0.11, 0.09)] {
        let tip = Vec3::new(p.x + 0.04, p.y + dy, p.z + dz);
        let dir = Vec3::new(1.0, 0.08, 0.04).normalize();
        k.rod(clutter, tip, tip + dir * 0.13, 0.004, &steel);
        let fl = tip + dir * 0.14;
        let rot = Quat::from_rotation_arc(Vec3::Z, dir);
        for q in [0.0, FRAC_PI_2] {
            k.cube_t(
                clutter,
                Transform::from_translation(fl)
                    .with_rotation(rot * Quat::from_rotation_z(q))
                    .with_scale(Vec3::new(0.035, 0.002, 0.04)),
                &flight,
            );
        }
    }
    // The oche, taped to the floor 2.37 m out.
    let tape = k.mat(Color::srgb(0.8, 0.6, 0.25), 0.4, 1.0);
    k.cube(
        root,
        Vec3::new(p.x + 2.37, 0.003, p.z),
        Vec3::new(0.04, 0.006, 0.8),
        &tape,
    );
    neon::sign(
        k,
        root,
        "DARTS",
        Transform::from_xyz(-ROOM_HALF_X + 0.06, p.y + 0.78, p.z).with_rotation(face),
        0.045,
        NeonColor::Pink,
        None,
    );
}

// ---------------------------------------------------------------------------
// Ceiling and floor
// ---------------------------------------------------------------------------

fn ceiling(k: &mut Kit, root: Entity, clutter: Entity) {
    let h = ROOM_HEIGHT;
    let beam = k.surf(Surf::Wood, Color::srgb(0.55, 0.4, 0.3), 1.2);
    for x in [-5.2, -2.6, 2.6, 5.2] {
        k.cube(
            root,
            Vec3::new(x, h - 0.12, 0.0),
            Vec3::new(0.2, 0.24, 10.0),
            &beam,
        );
    }
    for z in [-4.9, 4.9] {
        k.cube(
            root,
            Vec3::new(0.0, h - 0.07, z),
            Vec3::new(14.0, 0.14, 0.2),
            &beam,
        );
    }

    // Festoon strings sagging between the beams: the ceiling's own light.
    let wire = k.mat(Color::srgb(0.01, 0.01, 0.01), 0.6, 0.0);
    let festoon = k.glow(Color::srgb(1.0, 0.72, 0.4), 6.0);
    for z in [-1.2, 1.2] {
        let anchors = [-6.95, -5.2, -2.6, 0.0, 2.6, 5.2, 6.95];
        for w in anchors.windows(2) {
            let (a, b) = (w[0], w[1]);
            let n = 4;
            let pt = |t: f32| Vec3::new(a + (b - a) * t, h - 0.2 - 0.22 * 4.0 * t * (1.0 - t), z);
            for i in 0..n {
                let (t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                k.rod_lo(root, pt(t0), pt(t1), 0.003, &wire);
            }
            for t in [0.25, 0.5, 0.75] {
                k.ball(root, pt(t) - Vec3::Y * 0.04, 0.024, &festoon);
                k.cyl(root, pt(t) - Vec3::Y * 0.012, 0.008, 0.025, &wire);
            }
        }
    }

    // A galvanised duct over the arcade half, turning up into the ceiling.
    let duct = k.mat(Color::srgb(0.38, 0.38, 0.4), 0.45, 1.0);
    let (dz, dy, r) = (-2.8, 3.0, 0.15);
    let (xa, xb) = (0.6, ROOM_HALF_X - 0.02);
    let along = Quat::from_rotation_z(FRAC_PI_2);
    k.cyl_t(
        root,
        Transform::from_xyz((xa + xb) / 2.0, dy, dz).with_rotation(along),
        r,
        xb - xa,
        &duct,
    );
    k.ball(root, Vec3::new(xa, dy, dz), r, &duct);
    k.cyl(root, Vec3::new(xa, (dy + h) / 2.0, dz), r, h - dy, &duct);
    let mut x = xa + 0.5;
    while x < xb {
        k.cyl_t(
            root,
            Transform::from_xyz(x, dy, dz).with_rotation(along),
            r + 0.008,
            0.03,
            &duct,
        );
        k.rod(
            root,
            Vec3::new(x, dy + r, dz),
            Vec3::new(x, h, dz),
            0.006,
            &duct,
        );
        x += 1.2;
    }
    let grille = k.mat(Color::srgb(0.05, 0.05, 0.055), 0.6, 0.0);
    k.cube(
        root,
        Vec3::new(3.2, dy - r - 0.02, dz),
        Vec3::new(0.34, 0.04, 0.22),
        &duct,
    );
    for i in 0..5 {
        k.cube(
            root,
            Vec3::new(3.07 + i as f32 * 0.065, dy - r - 0.041, dz),
            Vec3::new(0.03, 0.004, 0.18),
            &grille,
        );
    }

    // Pendants over the bar and flanking the back-bar.
    let cord = k.mat(Color::srgb(0.015, 0.015, 0.015), 0.6, 0.0);
    let shade_m = k.mat(Color::srgb(0.05, 0.08, 0.06), 0.35, 0.6);
    let bulb = k.glow(Color::srgb(1.0, 0.75, 0.45), 8.0);
    let shade = k.meshes.add(ConicalFrustum {
        radius_top: 0.03,
        radius_bottom: 0.15,
        height: 0.17,
    });
    let pendants = [
        (Vec3::new(5.25, 0.0, -3.6), 2.4),
        (Vec3::new(5.25, 0.0, -2.4), 2.4),
        (Vec3::new(5.25, 0.0, -1.2), 2.4),
        (Vec3::new(-1.7, 0.0, 4.35), 2.62),
        (Vec3::new(1.7, 0.0, 4.35), 2.62),
    ];
    for (p, y) in pendants {
        k.rod(root, p.with_y(h), p.with_y(y + 0.085), 0.005, &cord);
        k.mesh(
            root,
            shade.clone(),
            Transform::from_translation(p.with_y(y)),
            &shade_m,
        );
        k.ball(root, p.with_y(y - 0.09), 0.045, &bulb);
    }
    // Their cable run: along the ceiling to the wall, down to a switch.
    let conduit = k.mat(Color::srgb(0.3, 0.3, 0.32), 0.5, 1.0);
    let cy = h - 0.015;
    let run = [
        Vec3::new(5.25, cy, -3.6),
        Vec3::new(5.25, cy, -0.3),
        Vec3::new(ROOM_HALF_X - 0.015, cy, -0.3),
        Vec3::new(ROOM_HALF_X - 0.015, 1.4, -0.3),
    ];
    for w in run.windows(2) {
        k.rod(clutter, w[0], w[1], 0.011, &conduit);
    }
    k.cube(
        clutter,
        Vec3::new(ROOM_HALF_X - 0.015, 1.36, -0.3),
        Vec3::new(0.03, 0.12, 0.08),
        &conduit,
    );

    // Slow ceiling fan over the lounge.
    let fan = Vec3::new(-3.9, h, -2.4);
    let bronze = k.mat(Color::srgb(0.18, 0.12, 0.07), 0.4, 1.0);
    k.cyl(root, fan - Vec3::Y * 0.17, 0.015, 0.34, &bronze);
    k.cyl(root, fan - Vec3::Y * 0.02, 0.07, 0.04, &bronze);
    k.cyl(root, fan - Vec3::Y * 0.4, 0.1, 0.12, &bronze);
    let rotor = k.group(
        Some(root),
        Transform::from_translation(fan - Vec3::Y * 0.47),
    );
    k.c.entity(rotor).insert((MergeRoot, FanRotor));
    let blade = k.mat(Color::srgb(0.2, 0.12, 0.07), 0.5, 0.0);
    let iron = k.mat(Color::srgb(0.12, 0.08, 0.05), 0.5, 0.0);
    k.cyl(rotor, Vec3::ZERO, 0.08, 0.04, &iron);
    for i in 0..4 {
        let rot = Quat::from_rotation_y(i as f32 * FRAC_PI_2);
        k.bevel_t(
            rotor,
            Transform::from_translation(rot * Vec3::X * 0.45)
                .with_rotation(rot * Quat::from_rotation_x(0.14)),
            Vec3::new(0.6, 0.012, 0.14),
            0.006,
            &blade,
        );
        k.cube_t(
            rotor,
            Transform::from_translation(rot * Vec3::X * 0.13)
                .with_rotation(rot)
                .with_scale(Vec3::new(0.16, 0.012, 0.04)),
            &iron,
        );
    }
    let run = [
        fan.with_y(cy),
        Vec3::new(-4.85, cy, -2.4),
        Vec3::new(-4.85, cy, -FZ + 0.015),
        Vec3::new(-4.85, 1.4, -FZ + 0.015),
    ];
    for w in run.windows(2) {
        k.rod(clutter, w[0], w[1], 0.011, &conduit);
    }
    k.cube(
        clutter,
        Vec3::new(-4.85, 1.36, -FZ + 0.015),
        Vec3::new(0.08, 0.12, 0.03),
        &conduit,
    );
}

fn floor(k: &mut Kit, root: Entity, clutter: Entity, images: &mut Assets<Image>) {
    // A runner in front of the back-bar.
    let rug = k.surf(Surf::Felt, Color::srgb(0.22, 0.12, 0.06), 1.6);
    let edge = k.mat(Color::srgb(0.08, 0.05, 0.03), 0.85, 0.0);
    k.cube(
        root,
        Vec3::new(0.0, 0.006, 3.95),
        Vec3::new(3.2, 0.012, 0.85),
        &rug,
    );
    k.cube(
        root,
        Vec3::new(0.0, 0.005, 3.95),
        Vec3::new(3.32, 0.01, 0.97),
        &edge,
    );

    // Worn patches where people stand: a pale, soft-edged film.
    let n = 64u32;
    let mut data = Vec::with_capacity((n * n * 4) as usize);
    let mut r = Rng(9);
    for y in 0..n {
        for x in 0..n {
            let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / n as f32 * 2.0 - Vec2::ONE;
            let a = (1.0 - d.length()).clamp(0.0, 1.0).powf(1.2) * (0.7 + r.f() * 0.3);
            data.extend([255, 255, 255, (a * 255.0) as u8]);
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
    let wear = k.mats.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.7, 0.58, 0.08),
        base_color_texture: Some(tex),
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    k.hints.to_room.insert(wear.id());
    let quad = k.blob.mesh.clone();
    let t = TABLE_CENTER;
    for (at, size) in [
        (Vec3::new(t.x - 1.95, 0.0125, t.z), Vec2::new(1.0, 1.7)),
        (Vec3::new(t.x + 1.95, 0.0125, t.z), Vec2::new(1.0, 1.7)),
        (Vec3::new(t.x, 0.0125, t.z + 1.25), Vec2::new(2.2, 0.7)),
        (Vec3::new(4.8, 0.003, -2.4), Vec2::new(1.1, 3.6)),
        (Vec3::new(-5.3, 0.003, 4.55), Vec2::new(0.8, 0.9)),
        (Vec3::new(-4.35, 0.003, 3.7), Vec2::new(0.9, 1.1)),
        (Vec3::new(5.15, 0.003, 4.3), Vec2::new(1.0, 0.8)),
    ] {
        k.c.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(wear.clone()),
            Transform::from_translation(at).with_scale(Vec3::new(size.x, 1.0, size.y)),
            NotShadowCaster,
            ChildOf(root),
        ));
    }

    // Strays: a cube of chalk, a dropped coaster, a cue case against the wall.
    let chalk = k.mat(Color::srgb(0.15, 0.35, 0.75), 0.9, 0.0);
    k.cube_t(
        clutter,
        Transform::from_xyz(t.x + 1.75, 0.013, t.z + 1.45)
            .with_rotation(Quat::from_rotation_y(0.5))
            .with_scale(Vec3::splat(0.022)),
        &chalk,
    );
    let coaster = k.mat(Color::srgb(0.6, 0.55, 0.45), 0.9, 0.0);
    k.cyl(
        clutter,
        Vec3::new(4.55, 0.003, -0.25),
        0.05,
        0.004,
        &coaster,
    );
    let case = k.surf(Surf::Leather, Color::srgb(0.1, 0.08, 0.07), 4.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let foot = Vec3::new(-3.62, 0.0, FZ - 0.16);
    let top = Vec3::new(-3.62, 0.86, FZ - 0.05);
    k.rod(clutter, foot + Vec3::Y * 0.04, top, 0.045, &case);
    let dir = (top - foot).normalize();
    for p in [foot + dir * 0.06, top] {
        k.cyl_t(
            clutter,
            Transform::from_translation(p).with_rotation(Quat::from_rotation_arc(Vec3::Y, dir)),
            0.047,
            0.03,
            &brass,
        );
    }
}

// ---------------------------------------------------------------------------
// The owner's portrait: low-poly, in a gilt frame beside the ARTHUB sign
// ---------------------------------------------------------------------------

/// Click it to read the About page.
fn portrait(k: &mut Kit, root: Entity) {
    let at = Vec3::new(0.42, 2.4, -ROOM_HALF_Z + 0.03);
    let g = k.group(Some(root), Transform::from_translation(at));
    k.c.entity(g).insert(Interactive::new(
        "Portrait — about me",
        Action::Print {
            path: "/about".into(),
            pose: Pose {
                eye: at + Vec3::new(0.0, -0.1, 1.9),
                look: at,
            },
        },
    ));
    let size = 0.62;
    let gold = k.mat(Color::srgb(0.83, 0.62, 0.28), 0.35, 1.0);
    let gold_dark = k.mat(Color::srgb(0.55, 0.38, 0.15), 0.45, 1.0);
    let liner = k.mat(Color::srgb(0.72, 0.67, 0.56), 0.9, 0.0);
    // Stepped gilt moulding: outer rail, a darker cove, a bright inner lip.
    k.bevel(
        g,
        Vec3::Z * 0.025,
        Vec3::new(size + 0.2, size + 0.2, 0.05),
        0.015,
        &gold,
    );
    k.bevel(
        g,
        Vec3::Z * 0.045,
        Vec3::new(size + 0.12, size + 0.12, 0.02),
        0.006,
        &gold_dark,
    );
    k.cube(
        g,
        Vec3::Z * 0.052,
        Vec3::new(size + 0.06, size + 0.06, 0.012),
        &liner,
    );
    k.bevel(
        g,
        Vec3::Z * 0.058,
        Vec3::new(size + 0.015, size + 0.015, 0.006),
        0.002,
        &gold,
    );
    let art = k.mats.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.25, 0.25),
        perceptual_roughness: 0.55,
        ..default()
    });
    k.hints.exact.insert(art.id());
    k.cube(g, Vec3::Z * 0.062, Vec3::new(size, size, 0.004), &art);
    let image = k.assets.load::<Image>("room/portrait.jpg");
    k.c.spawn((
        PendingCover {
            image,
            mat: art,
            crt: false,
        },
        ChildOf(g),
    ));
    // Brass picture light.
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let lamp = k.glow(Color::srgb(1.0, 0.8, 0.55), 3.5);
    let top = size / 2.0 + 0.1;
    k.cyl_t(
        g,
        Transform::from_xyz(0.0, top + 0.02, 0.06).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        0.008,
        0.12,
        &brass,
    );
    k.bevel(
        g,
        Vec3::new(0.0, top + 0.03, 0.13),
        Vec3::new(0.36, 0.035, 0.05),
        0.01,
        &brass,
    );
    k.cube(
        g,
        Vec3::new(0.0, top + 0.012, 0.13),
        Vec3::new(0.33, 0.004, 0.03),
        &lamp,
    );
}

// ---------------------------------------------------------------------------
// Black Label at the bar: on the counter, in a crate, and a fridge full
// ---------------------------------------------------------------------------

const BL_RED: [u8; 3] = [196, 18, 40];
const BL_BLACK: [u8; 3] = [16, 14, 14];
const BL_CREAM: [u8; 3] = [240, 232, 214];

/// The label, drawn twice across the width so it reads from any side once
/// it's wrapped round a bottle or can. `h` keeps the lettering in proportion:
/// a can's wrap is taller for its girth than a bottle's label band.
fn black_label_art(h: u32) -> Canvas {
    let w = 256u32;
    let mut c = Canvas::new(w, h, BL_RED);
    let s = h as f32 / 96.0;
    for half in 0..2 {
        let x0 = half as f32 * w as f32 / 2.0;
        let cx = x0 + w as f32 / 4.0;
        c.rect(
            x0 + 8.0,
            30.0 * s,
            w as f32 / 2.0 - 16.0,
            40.0 * s,
            BL_BLACK,
        );
        c.text("CARLING", cx, 12.0 * s, 11.0, BL_CREAM);
        c.text("BLACK", cx, 30.0 * s + 6.0, 14.0, BL_CREAM);
        c.text("LABEL", cx, 30.0 * s + 24.0, 14.0, BL_CREAM);
        c.rect(x0 + 8.0, 80.0 * s, w as f32 / 2.0 - 16.0, 3.0, BL_CREAM);
    }
    c
}

struct BlackLabel {
    glass: Handle<StandardMaterial>,
    band: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    cap: Handle<StandardMaterial>,
    shoulder: Handle<Mesh>,
}

impl BlackLabel {
    /// A 750 ml quart: brown glass, wrapped label, red neck foil, crown cap.
    fn quart(&self, k: &mut Kit, parent: Entity, base: Vec3) {
        k.cyl(parent, base + Vec3::Y * 0.1, 0.037, 0.2, &self.glass);
        k.cyl(parent, base + Vec3::Y * 0.1, 0.0375, 0.085, &self.band);
        k.mesh(
            parent,
            self.shoulder.clone(),
            Transform::from_translation(base + Vec3::Y * 0.225),
            &self.glass,
        );
        k.cyl(parent, base + Vec3::Y * 0.285, 0.0135, 0.07, &self.glass);
        k.cyl(parent, base + Vec3::Y * 0.3, 0.0145, 0.04, &self.red);
        k.cyl(parent, base + Vec3::Y * 0.325, 0.015, 0.012, &self.cap);
    }

    /// A 440 ml can.
    fn can(&self, k: &mut Kit, parent: Entity, base: Vec3, m: &Handle<StandardMaterial>) {
        k.cyl(parent, base + Vec3::Y * 0.062, 0.033, 0.124, m);
        k.cyl(parent, base + Vec3::Y * 0.127, 0.028, 0.006, &self.cap);
    }
}

fn black_label(k: &mut Kit, root: Entity, images: &mut Assets<Image>) {
    let band_art = images.add(black_label_art(96).image_wrapped());
    let art = images.add(black_label_art(150).image_wrapped());
    let band = k.mats.add(StandardMaterial {
        base_color_texture: Some(band_art),
        perceptual_roughness: 0.45,
        ..default()
    });
    let can = k.mats.add(StandardMaterial {
        base_color_texture: Some(art.clone()),
        perceptual_roughness: 0.3,
        metallic: 0.6,
        ..default()
    });
    // In the fridge: lit by the cabinet, so they carry a little of their own.
    let can_lit = k.mats.add(StandardMaterial {
        base_color_texture: Some(art.clone()),
        emissive_texture: Some(art.clone()),
        emissive: LinearRgba::WHITE * 0.35,
        perceptual_roughness: 0.3,
        metallic: 0.4,
        ..default()
    });
    let bl = BlackLabel {
        glass: k.mat(Color::srgb(0.22, 0.09, 0.02), 0.08, 0.0),
        band,
        red: k.mat(Color::srgb(0.6, 0.04, 0.08), 0.35, 0.3),
        cap: k.mat(Color::srgb(0.8, 0.78, 0.72), 0.3, 1.0),
        shoulder: k.meshes.add(ConicalFrustum {
            radius_top: 0.0135,
            radius_bottom: 0.037,
            height: 0.05,
        }),
    };

    // On the counter: two quarts and a poured pint, a six-pack further down.
    let (x, top) = (5.42, 1.105);
    for (dz, dx) in [(-2.6, 0.0), (-2.45, 0.07)] {
        bl.quart(k, root, Vec3::new(x + dx, top, dz));
    }
    let pint = k.mat(Color::srgb(0.2, 0.22, 0.24), 0.05, 0.0);
    let lager = k.mat(Color::srgb(0.75, 0.45, 0.08), 0.1, 0.0);
    let foam = k.mat(Color::srgb(0.95, 0.92, 0.85), 0.8, 0.0);
    let p = Vec3::new(x - 0.12, top, -2.3);
    k.cyl(root, p + Vec3::Y * 0.004, 0.038, 0.008, &pint);
    k.cyl(root, p + Vec3::Y * 0.066, 0.037, 0.116, &lager);
    k.cyl(root, p + Vec3::Y * 0.134, 0.038, 0.02, &foam);
    k.cyl(root, p + Vec3::Y * 0.149, 0.04, 0.01, &pint);
    let carton = k.mat(Color::srgb(0.6, 0.05, 0.09), 0.8, 0.0);
    let sp = Vec3::new(x + 0.05, top, -0.85);
    k.cube(
        root,
        sp + Vec3::Y * 0.025,
        Vec3::new(0.15, 0.05, 0.23),
        &carton,
    );
    k.cube(
        root,
        sp + Vec3::Y * 0.11,
        Vec3::new(0.004, 0.17, 0.2),
        &carton,
    );
    for i in 0..6 {
        let (cx, cz) = ((i % 2) as f32 - 0.5, (i / 2) as f32 - 1.0);
        bl.can(k, root, sp + Vec3::new(cx * 0.072, 0.004, cz * 0.072), &can);
    }

    // A crate of twelve at the end of the bar.
    let crate_red = k.mat(Color::srgb(0.55, 0.04, 0.07), 0.6, 0.0);
    let grip = k.mat(Color::srgb(0.02, 0.02, 0.02), 0.6, 0.0);
    let c = Vec3::new(5.55, 0.0, -0.18);
    let (cw, cd, ch) = (0.32, 0.42, 0.24);
    k.cube(
        root,
        c + Vec3::Y * 0.01,
        Vec3::new(cw, 0.02, cd),
        &crate_red,
    );
    for sx in [-1.0, 1.0] {
        k.cube(
            root,
            c + Vec3::new(sx * cw / 2.0, ch / 2.0, 0.0),
            Vec3::new(0.015, ch, cd),
            &crate_red,
        );
    }
    for sz in [-1.0, 1.0] {
        k.cube(
            root,
            c + Vec3::new(0.0, ch / 2.0, sz * cd / 2.0),
            Vec3::new(cw, ch, 0.015),
            &crate_red,
        );
        k.cube(
            root,
            c + Vec3::new(0.0, ch * 0.75, sz * (cd / 2.0 + 0.004)),
            Vec3::new(0.09, 0.03, 0.006),
            &grip,
        );
    }
    for i in 0..12 {
        let (ix, iz) = ((i % 3) as f32 - 1.0, (i / 3) as f32 - 1.5);
        bl.quart(k, root, c + Vec3::new(ix * 0.095, 0.02, iz * 0.095));
    }
    k.shadow(root, c, Vec2::new(0.5, 0.6));

    // An upright glass-door fridge past the end of the counter, stocked
    // with nothing else.
    let f = k.group(
        Some(root),
        Transform::from_xyz(ROOM_HALF_X - 0.3, 0.0, -0.05)
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    );
    let (fw, fh, fd) = (0.62, 1.86, 0.56);
    let shell = k.mat(Color::srgb(0.03, 0.03, 0.035), 0.4, 0.6);
    let inner = k.glow(Color::srgb(0.75, 0.85, 1.0), 1.4);
    let wire = k.mat(Color::srgb(0.6, 0.6, 0.62), 0.3, 1.0);
    let t = 0.03;
    // Shell: back, sides, top, plinth; the inside back panel glows.
    k.cube(
        f,
        Vec3::new(0.0, fh / 2.0, -fd / 2.0 + t / 2.0),
        Vec3::new(fw, fh, t),
        &shell,
    );
    for sx in [-1.0, 1.0] {
        k.cube(
            f,
            Vec3::new(sx * (fw - t) / 2.0, fh / 2.0, 0.0),
            Vec3::new(t, fh, fd),
            &shell,
        );
    }
    k.cube(
        f,
        Vec3::new(0.0, fh - 0.11, 0.0),
        Vec3::new(fw, 0.22, fd),
        &shell,
    );
    k.cube(
        f,
        Vec3::new(0.0, 0.06, 0.0),
        Vec3::new(fw, 0.12, fd),
        &shell,
    );
    k.cube(
        f,
        Vec3::new(0.0, (fh - 0.22 + 0.12) / 2.0, -fd / 2.0 + t + 0.002),
        Vec3::new(fw - 2.0 * t, fh - 0.34, 0.004),
        &inner,
    );
    // Five shelves: quarts on the bottom two, cans above.
    let shelves = [0.14, 0.47, 0.8, 1.11, 1.38];
    let mut n = 0;
    for (si, y) in shelves.into_iter().enumerate() {
        k.cube(
            f,
            Vec3::new(0.0, y, 0.0),
            Vec3::new(fw - 2.0 * t, 0.012, fd - 0.08),
            &wire,
        );
        let (step, count) = if si < 2 { (0.088, 6) } else { (0.074, 7) };
        for row in 0..2 {
            for i in 0..count {
                let px = (i as f32 - (count - 1) as f32 / 2.0) * step;
                let pz = 0.12 - row as f32 * 0.16;
                let base = Vec3::new(px, y + 0.006, pz);
                if si < 2 {
                    bl.quart(k, f, base);
                } else {
                    bl.can(k, f, base, &can_lit);
                }
                n += 1;
            }
        }
    }
    debug!("black label fridge: {n} drinks");
    // Glass door with a chrome handle, and the lit header sign.
    let glass = k.mats.add(StandardMaterial {
        base_color: Color::srgba(0.7, 0.8, 0.9, 0.1),
        perceptual_roughness: 0.03,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    k.cube(
        f,
        Vec3::new(0.0, (fh - 0.22 + 0.12) / 2.0, fd / 2.0 - 0.01),
        Vec3::new(fw - 0.06, fh - 0.36, 0.006),
        &glass,
    );
    for sx in [-1.0, 1.0] {
        k.cube(
            f,
            Vec3::new(sx * (fw / 2.0 - 0.02), fh / 2.0, fd / 2.0 - 0.01),
            Vec3::new(0.04, fh - 0.22, 0.02),
            &shell,
        );
    }
    let chrome = k.mat(Color::srgb(0.85, 0.85, 0.88), 0.15, 1.0);
    k.cube(
        f,
        Vec3::new(-fw / 2.0 + 0.07, 0.95, fd / 2.0 + 0.03),
        Vec3::new(0.02, 0.5, 0.02),
        &chrome,
    );
    let mut sign = Canvas::new(256, 64, BL_RED);
    sign.rect(0.0, 10.0, 256.0, 44.0, BL_BLACK);
    sign.text("BLACK LABEL", 128.0, 20.0, 24.0, BL_CREAM);
    let header = k.mats.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive_texture: Some(images.add(sign.image())),
        emissive: LinearRgba::WHITE * 2.2,
        ..default()
    });
    let g = k.group(
        Some(f),
        Transform::from_xyz(0.0, fh - 0.11, fd / 2.0 + 0.002),
    );
    k.cube(g, Vec3::ZERO, Vec3::new(fw - 0.08, 0.16, 0.004), &header);
    k.shadow(f, Vec3::ZERO, Vec2::new(0.8, 0.75));
}

/// A tiny deterministic generator, so the clutter lands the same every load.
struct Rng(u32);

impl Rng {
    fn f(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (self.0 >> 9) as f32 / (1u32 << 23) as f32
    }
}
