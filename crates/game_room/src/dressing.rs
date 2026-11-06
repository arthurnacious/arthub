//! Set dressing: the clutter that makes the room look used. All static, so it
//! folds into a handful of merged batches.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;
use bridge::FeaturedProject;

use crate::camera::Pose;
use crate::interact::{Action, Interactive};
use crate::scene::{JUKEBOX_POS, Kit, PendingCover, TV_POS};
use crate::textures::Surf;

pub fn dress(k: &mut Kit, root: Entity, featured: &[FeaturedProject]) {
    cue_rack(k, root);
    prints(k, root, featured);
    cables(k, root);
    plant(k, root, Vec3::new(4.0, 0.0, 4.55), 1.0);
    plant(k, root, Vec3::new(-6.45, 0.0, -2.55), 0.8);
    drinks(k, root);
    couch_rug(k, root);
    records(k, root);
}

fn cue_rack(k: &mut Kit, root: Entity) {
    let wood = k.surf(Surf::Wood, Color::srgb(0.8, 0.5, 0.32), 2.0);
    let maple = k.mat(Color::srgb(0.78, 0.6, 0.38), 0.35, 0.0);
    let ebony = k.mat(Color::srgb(0.04, 0.025, 0.02), 0.3, 0.0);
    let ferrule = k.mat(Color::srgb(0.92, 0.9, 0.84), 0.4, 0.0);
    let tip = k.mat(Color::srgb(0.15, 0.3, 0.6), 0.9, 0.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);

    let x0 = 0.3;
    k.bevel(
        root,
        Vec3::new(x0, 1.45, -4.96),
        Vec3::new(0.72, 0.08, 0.06),
        0.015,
        &wood,
    );
    k.bevel(
        root,
        Vec3::new(x0, 0.32, -4.95),
        Vec3::new(0.72, 0.06, 0.1),
        0.015,
        &wood,
    );
    k.shadow(root, Vec3::new(x0, 0.0, -4.85), Vec2::new(0.9, 0.35));
    let butt = k.meshes.add(ConicalFrustum {
        radius_top: 0.011,
        radius_bottom: 0.015,
        height: 0.7,
    });
    let shaft = k.meshes.add(ConicalFrustum {
        radius_top: 0.0065,
        radius_bottom: 0.011,
        height: 0.75,
    });
    for i in 0..5 {
        let x = x0 - 0.24 + i as f32 * 0.12;
        let lean = Quat::from_rotation_z((i as f32 - 2.0) * 0.012);
        let base = Vec3::new(x, 0.34, -4.92);
        let up = lean * Vec3::Y;
        let butt_mat = if i % 2 == 0 { &ebony } else { &maple };
        k.mesh(
            root,
            butt.clone(),
            Transform::from_translation(base + up * 0.35).with_rotation(lean),
            butt_mat,
        );
        k.mesh(
            root,
            shaft.clone(),
            Transform::from_translation(base + up * 1.075).with_rotation(lean),
            &maple,
        );
        k.cyl_t(
            root,
            Transform::from_translation(base + up * 1.46).with_rotation(lean),
            0.0066,
            0.02,
            &ferrule,
        );
        k.cyl_t(
            root,
            Transform::from_translation(base + up * 1.475).with_rotation(lean),
            0.0062,
            0.01,
            &tip,
        );
        k.cyl_t(
            root,
            Transform::from_translation(base + up * 0.66).with_rotation(lean),
            0.0115,
            0.03,
            &brass,
        );
    }
    // The triangle rack, hanging from a peg beside the cues.
    let peg = Vec3::new(x0 + 0.5, 1.75, -4.93);
    k.cyl_t(
        root,
        Transform::from_translation(peg).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        0.008,
        0.08,
        &brass,
    );
    let pts: Vec<Vec3> = (0..3)
        .map(|j| {
            let a = -FRAC_PI_2 + j as f32 * TAU / 3.0 + PI;
            peg + Vec3::new(a.cos(), a.sin() - 0.5, 0.0) * 0.2 + Vec3::Z * 0.02
        })
        .collect();
    for j in 0..3 {
        k.rod(root, pts[j], pts[(j + 1) % 3], 0.012, &wood);
    }
}

/// Framed prints of the featured projects. Click one to open its case study.
fn prints(k: &mut Kit, root: Entity, featured: &[FeaturedProject]) {
    // (position, facing as a yaw that turns local +Z into the room, size)
    let spots = [
        (Vec3::new(-4.3, 1.55, -4.97), 0.0, Vec2::new(0.78, 0.49)),
        (
            Vec3::new(-6.97, 1.6, 0.15),
            FRAC_PI_2,
            Vec2::new(0.82, 0.51),
        ),
        (
            Vec3::new(6.97, 1.6, 1.55),
            -FRAC_PI_2,
            Vec2::new(0.88, 0.55),
        ),
    ];
    let frame = k.mat(Color::srgb(0.03, 0.03, 0.03), 0.4, 0.2);
    let mount = k.mat(Color::srgb(0.85, 0.83, 0.78), 0.9, 0.0);
    let lamp = k.glow(Color::srgb(1.0, 0.8, 0.55), 3.0);
    for (p, (at, yaw, size)) in featured.iter().zip(spots) {
        let rot = Quat::from_rotation_y(yaw);
        let g = k.group(
            Some(root),
            Transform::from_translation(at).with_rotation(rot),
        );
        let facing = rot * Vec3::Z;
        let pose = Pose {
            eye: at + facing * 1.6 + Vec3::Y * 0.05,
            look: at,
        };
        k.c.entity(g).insert(Interactive::new(
            format!("{} — print", p.title),
            Action::Print {
                path: format!("/work/{}", p.slug),
                pose,
            },
        ));
        k.bevel(
            g,
            Vec3::new(0.0, 0.0, 0.015),
            Vec3::new(size.x + 0.08, size.y + 0.08, 0.03),
            0.008,
            &frame,
        );
        k.cube(
            g,
            Vec3::new(0.0, 0.0, 0.032),
            Vec3::new(size.x + 0.02, size.y + 0.02, 0.004),
            &mount,
        );
        let art = k.mats.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25),
            perceptual_roughness: 0.35,
            ..default()
        });
        k.hints.exact.insert(art.id());
        k.cube(
            g,
            Vec3::new(0.0, 0.0, 0.036),
            Vec3::new(size.x - 0.06, size.y - 0.06, 0.004),
            &art,
        );
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
        // A brass picture light, glowing down onto the print.
        k.bevel(
            g,
            Vec3::new(0.0, size.y / 2.0 + 0.09, 0.08),
            Vec3::new(size.x * 0.5, 0.03, 0.05),
            0.01,
            &frame,
        );
        k.cube(
            g,
            Vec3::new(0.0, size.y / 2.0 + 0.074, 0.08),
            Vec3::new(size.x * 0.46, 0.004, 0.03),
            &lamp,
        );
    }
}

fn cables(k: &mut Kit, root: Entity) {
    let rubber = k.mat(Color::srgb(0.015, 0.015, 0.015), 0.6, 0.0);
    let r = 0.007;
    // From the back of the arcade row, along the skirting to the corner socket.
    let y = 0.012;
    k.rod(
        root,
        Vec3::new(1.0, y, -4.88),
        Vec3::new(5.4, y, -4.88),
        r,
        &rubber,
    );
    k.rod(
        root,
        Vec3::new(5.4, y, -4.88),
        Vec3::new(5.4, y, -4.35),
        r,
        &rubber,
    );
    // The TV feed runs down the wall and across to the jukebox corner.
    let tv = TV_POS + Vec3::new(0.06, -0.49, 0.25);
    k.rod(root, tv, tv.with_y(y), r, &rubber);
    k.rod(root, tv.with_y(y), Vec3::new(-6.88, y, -2.0), r, &rubber);
    // Jukebox power, in a lazy curve.
    let j = JUKEBOX_POS + Vec3::new(0.3, y, -0.28);
    let pts = [
        j,
        j + Vec3::new(0.25, 0.0, -0.05),
        j + Vec3::new(0.45, 0.0, -0.2),
        Vec3::new(-4.6, y, -4.9),
    ];
    for w in pts.windows(2) {
        k.rod(root, w[0], w[1], r, &rubber);
    }
    let plate = k.mat(Color::srgb(0.75, 0.72, 0.66), 0.5, 0.0);
    k.bevel(
        root,
        Vec3::new(5.4, 0.3, -4.97),
        Vec3::new(0.08, 0.12, 0.012),
        0.004,
        &plate,
    );
    k.bevel(
        root,
        Vec3::new(-4.6, 0.3, -4.97),
        Vec3::new(0.08, 0.12, 0.012),
        0.004,
        &plate,
    );
}

fn plant(k: &mut Kit, root: Entity, at: Vec3, scale: f32) {
    let clay = k.mat(Color::srgb(0.45, 0.2, 0.12), 0.85, 0.0);
    let soil = k.mat(Color::srgb(0.05, 0.035, 0.025), 1.0, 0.0);
    let leaf_a = k.mat(Color::srgb(0.05, 0.22, 0.08), 0.55, 0.0);
    let leaf_b = k.mat(Color::srgb(0.08, 0.3, 0.1), 0.55, 0.0);
    let g = k.group(
        Some(root),
        Transform::from_translation(at).with_scale(Vec3::splat(scale)),
    );
    let pot = k.meshes.add(ConicalFrustum {
        radius_top: 0.2,
        radius_bottom: 0.14,
        height: 0.38,
    });
    k.mesh(g, pot, Transform::from_xyz(0.0, 0.19, 0.0), &clay);
    k.cyl(g, Vec3::new(0.0, 0.37, 0.0), 0.185, 0.02, &soil);
    k.shadow(g, Vec3::ZERO, Vec2::splat(0.6));
    // Fronds: flattened spheres fanned out on stems.
    let mut seed = 7u32;
    let mut rnd = move || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (seed >> 9) as f32 / (1u32 << 23) as f32
    };
    for i in 0..14 {
        let a = i as f32 / 14.0 * TAU + rnd() * 0.4;
        let tilt = 0.35 + rnd() * 0.6;
        let len = 0.35 + rnd() * 0.35;
        let dir = Vec3::new(a.cos() * tilt.sin(), tilt.cos(), a.sin() * tilt.sin());
        let base = Vec3::new(0.0, 0.38, 0.0);
        let tipp = base + dir * len;
        k.rod(g, base, tipp, 0.006, &leaf_a);
        let leaf = Transform::from_translation(base + dir * len * 0.75)
            .with_rotation(
                Quat::from_rotation_arc(Vec3::X, dir) * Quat::from_rotation_x(rnd() * PI),
            )
            .with_scale(Vec3::new(len * 0.45, 0.004, 0.06));
        let m = if i % 2 == 0 { &leaf_a } else { &leaf_b };
        let mesh = k.meshes.add(Sphere::new(1.0).mesh().uv(10, 6));
        k.mesh(g, mesh, leaf, m);
    }
}

fn drinks(k: &mut Kit, root: Entity) {
    let top = 1.105;
    let x = 5.42;
    let glass = k.mats.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.1),
        perceptual_roughness: 0.04,
        reflectance: 0.9,
        ..default()
    });
    let whisky = k.mats.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.15, 0.03),
        perceptual_roughness: 0.1,
        ..default()
    });
    let coaster = k.mat(Color::srgb(0.6, 0.55, 0.45), 0.9, 0.0);
    let cherry = k.mat(Color::srgb(0.6, 0.02, 0.04), 0.2, 0.0);
    let napkin = k.mat(Color::srgb(0.9, 0.88, 0.82), 0.9, 0.0);

    // Two whiskies on coasters.
    for z in [-3.75, -1.1] {
        k.cyl(root, Vec3::new(x, top + 0.002, z), 0.05, 0.004, &coaster);
        k.cyl(root, Vec3::new(x, top + 0.045, z), 0.037, 0.085, &glass);
        k.cyl(root, Vec3::new(x, top + 0.025, z), 0.032, 0.035, &whisky);
    }
    // A cocktail with a cherry.
    let bowl = k.meshes.add(ConicalFrustum {
        radius_top: 0.06,
        radius_bottom: 0.005,
        height: 0.07,
    });
    let p = Vec3::new(x - 0.05, top, -1.75);
    k.cyl(root, p + Vec3::Y * 0.003, 0.035, 0.006, &glass);
    k.cyl(root, p + Vec3::Y * 0.05, 0.004, 0.09, &glass);
    k.mesh(
        root,
        bowl,
        Transform::from_translation(p + Vec3::Y * 0.13),
        &glass,
    );
    k.ball(root, p + Vec3::new(0.01, 0.16, 0.0), 0.012, &cherry);
    // Napkin stack.
    k.bevel(
        root,
        Vec3::new(x + 0.08, top + 0.012, -3.2),
        Vec3::new(0.12, 0.024, 0.12),
        0.004,
        &napkin,
    );
}

fn couch_rug(k: &mut Kit, root: Entity) {
    let rug = k.surf(Surf::Felt, Color::srgb(0.06, 0.2, 0.22), 1.4);
    let edge = k.mat(Color::srgb(0.55, 0.42, 0.22), 0.85, 0.0);
    k.cube(
        root,
        Vec3::new(-2.3, 0.006, -3.75),
        Vec3::new(3.3, 0.012, 2.1),
        &rug,
    );
    k.cube(
        root,
        Vec3::new(-2.3, 0.005, -3.75),
        Vec3::new(3.45, 0.01, 2.25),
        &edge,
    );
}

fn records(k: &mut Kit, root: Entity) {
    let wood = k.surf(Surf::Wood, Color::srgb(0.85, 0.55, 0.35), 2.5);
    let vinyl = k.mat(Color::srgb(0.01, 0.01, 0.012), 0.25, 0.0);
    let labels = [
        k.mat(Color::srgb(0.85, 0.2, 0.15), 0.7, 0.0),
        k.mat(Color::srgb(0.95, 0.8, 0.3), 0.7, 0.0),
        k.mat(Color::srgb(0.2, 0.45, 0.85), 0.7, 0.0),
    ];
    let sleeve_cols = [
        Color::srgb(0.75, 0.3, 0.2),
        Color::srgb(0.15, 0.2, 0.3),
        Color::srgb(0.85, 0.75, 0.55),
        Color::srgb(0.2, 0.35, 0.25),
    ];
    let sleeves: Vec<_> = sleeve_cols
        .into_iter()
        .map(|c| k.mat(c, 0.8, 0.0))
        .collect();

    // A crate of sleeves beside the jukebox.
    let c = JUKEBOX_POS + Vec3::new(0.95, 0.0, 0.05);
    let crate_ = k.group(
        Some(root),
        Transform::from_translation(c).with_rotation(Quat::from_rotation_y(-0.15)),
    );
    let (w, d, h) = (0.36, 0.42, 0.3);
    k.bevel(
        crate_,
        Vec3::new(0.0, 0.01, 0.0),
        Vec3::new(w, 0.02, d),
        0.005,
        &wood,
    );
    for sx in [-1.0, 1.0] {
        k.bevel(
            crate_,
            Vec3::new(sx * w / 2.0, h / 2.0, 0.0),
            Vec3::new(0.02, h, d),
            0.005,
            &wood,
        );
    }
    for sz in [-1.0, 1.0] {
        k.bevel(
            crate_,
            Vec3::new(0.0, h / 2.0, sz * d / 2.0),
            Vec3::new(w, h, 0.02),
            0.005,
            &wood,
        );
    }
    for i in 0..13 {
        let z = -d / 2.0 + 0.04 + i as f32 * 0.027;
        let tilt = Quat::from_rotation_x(-0.18 + (i % 3) as f32 * 0.04);
        let t = Transform::from_xyz(0.0, 0.18, z).with_rotation(tilt);
        k.cube_t(
            crate_,
            t.with_scale(Vec3::new(0.31, 0.31, 0.004)),
            &sleeves[i % sleeves.len()],
        );
    }
    k.shadow(crate_, Vec3::ZERO, Vec2::new(0.6, 0.65));

    // Two records out of their sleeves, leaning on the jukebox.
    for (j, dz) in [0.0f32, 0.05].into_iter().enumerate() {
        let p = JUKEBOX_POS + Vec3::new(-0.6, 0.155, 0.1 + dz);
        let rot = Quat::from_rotation_y(FRAC_PI_2)
            * Quat::from_rotation_z(FRAC_PI_2)
            * Quat::from_rotation_x(0.12);
        k.cyl_t(
            root,
            Transform::from_translation(p).with_rotation(rot),
            0.15,
            0.003,
            &vinyl,
        );
        k.cyl_t(
            root,
            Transform::from_translation(p + rot * Vec3::Y * 0.002).with_rotation(rot),
            0.05,
            0.002,
            &labels[j],
        );
    }
}
