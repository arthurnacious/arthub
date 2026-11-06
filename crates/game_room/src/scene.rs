//! The lounge, built from primitives. No external models: every prop is a
//! handful of scaled unit meshes, so the whole room costs nothing to download.
//!
//! Layout (metres, Y up, floor at 0). The room spans x ∈ [-7, 7], z ∈ [-5, 5]
//! and the overview camera looks from the front (+z) toward the back wall.
//!
//! * back wall (z = -5):  ARTHUB neon, couch, jukebox (left), arcade row (right)
//! * left wall (x = -7):  trophy shelf, TV, 8-ball neon, chalkboard, dartboard
//! * right wall (x = 7):  bar + OPEN neon, wall phone, exit door + EXIT neon
//! * centre:              pool table under its lamp

use bevy::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use bridge::{FeaturedProject, Hotspot};

use crate::atmosphere::Blob;
use crate::interact::{Action, Interactive};
use crate::merge::{CastsShadows, MergeHints, RoomRoot, rounded_box};
use crate::neon::{self, NeonColor};
use crate::textures::{BoxUv, Surf, Textures};

pub const ROOM_HALF_X: f32 = 7.0;
pub const ROOM_HALF_Z: f32 = 5.0;
pub const ROOM_HEIGHT: f32 = 3.4;

/// Centre of the pool table's playing surface (top of the felt).
pub const TABLE_CENTER: Vec3 = Vec3::new(0.0, 0.80, 0.3);
pub const TABLE_HALF_LEN: f32 = 1.27;
pub const TABLE_HALF_WID: f32 = 0.635;
const RAIL: f32 = 0.16;

pub const CABINET_Z: f32 = -4.55;
/// Most cabinets that fit along the back wall between the couch and the bar.
pub const MAX_CABINETS: usize = 6;
static CABINETS: AtomicUsize = AtomicUsize::new(0);

/// The featured projects, one cabinet each (first [`MAX_CABINETS`]).
#[derive(Resource, Clone, Default)]
pub struct Featured(pub Vec<FeaturedProject>);

/// Cabinets are centred on x = 3.2, spaced to fit however many there are.
pub fn cabinet_x(i: usize) -> f32 {
    let n = CABINETS.load(Ordering::Relaxed).max(1);
    let spacing = if n <= 5 { 0.95 } else { 0.84 };
    3.2 + (i as f32 - (n as f32 - 1.0) / 2.0) * spacing
}

pub const TV_POS: Vec3 = Vec3::new(-6.94, 1.8, -1.2);
pub const CHALKBOARD_POS: Vec3 = Vec3::new(-6.94, 1.65, 1.6);
pub const DARTBOARD_POS: Vec3 = Vec3::new(-6.94, 1.73, 3.7);
pub const TROPHY_POS: Vec3 = Vec3::new(-6.85, 1.45, -3.75);
pub const JUKEBOX_POS: Vec3 = Vec3::new(-5.5, 0.0, -4.45);
pub const PHONE_POS: Vec3 = Vec3::new(6.94, 1.45, 0.7);
pub const DOOR_POS: Vec3 = Vec3::new(6.97, 0.0, 2.9);

/// The pool lamp's shade, hidden while playing.
#[derive(Component)]
pub struct LampShade;

/// Tags lights that are allowed to cast shadows (on medium/high quality).
#[derive(Component)]
pub struct ShadowCaster;

/// The animated TV screen material.
#[derive(Component)]
pub struct TvScreen(pub Handle<StandardMaterial>);

/// Jukebox colour bands that pulse; `phase` offsets each band.
#[derive(Component)]
pub struct JukeboxBand {
    pub mat: Handle<StandardMaterial>,
    pub phase: f32,
}

/// An arcade screen waiting for its optional cover image to load.
#[derive(Component)]
pub struct PendingCover {
    pub image: Handle<Image>,
    pub mat: Handle<StandardMaterial>,
    /// Arcade screens get the CRT treatment; framed prints don't.
    pub crt: bool,
}

/// Where the chalk tally marks for potted balls are drawn.
#[derive(Resource)]
pub struct Chalkboard {
    pub root: Entity,
    pub chalk: Handle<StandardMaterial>,
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_room)
            .add_systems(Update, (animate_tv, animate_jukebox, swap_in_covers));
    }
}

// ---------------------------------------------------------------------------
// Kit: tiny helpers for spawning scaled primitives
// ---------------------------------------------------------------------------

pub struct Kit<'a, 'w, 's> {
    pub c: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub mats: &'a mut Assets<StandardMaterial>,
    pub assets: &'a AssetServer,
    pub tex: &'a mut Textures,
    pub box_uv: &'a mut BoxUv,
    pub blob: &'a Blob,
    pub hints: &'a mut MergeHints,
    /// CRT test card for the TV.
    pub test_card: Handle<Image>,
    /// Identical materials are shared so the static merge can batch them.
    cache: std::collections::HashMap<String, Handle<StandardMaterial>>,
    cube: Handle<Mesh>,
    cyl: Handle<Mesh>,
    sphere: Handle<Mesh>,
    /// Low-poly versions for thin tubes and joints (neon, wires).
    cyl_lo: Handle<Mesh>,
    sphere_lo: Handle<Mesh>,
}

impl<'a, 'w, 's> Kit<'a, 'w, 's> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c: &'a mut Commands<'w, 's>,
        meshes: &'a mut Assets<Mesh>,
        mats: &'a mut Assets<StandardMaterial>,
        assets: &'a AssetServer,
        tex: &'a mut Textures,
        box_uv: &'a mut BoxUv,
        blob: &'a Blob,
        hints: &'a mut MergeHints,
    ) -> Self {
        let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let cyl = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(24));
        let sphere = meshes.add(Sphere::new(1.0).mesh().uv(20, 14));
        let cyl_lo = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(8));
        let sphere_lo = meshes.add(Sphere::new(1.0).mesh().uv(8, 6));
        Self {
            c,
            meshes,
            mats,
            assets,
            tex,
            box_uv,
            blob,
            hints,
            test_card: Handle::default(),
            cache: Default::default(),
            cube,
            cyl,
            sphere,
            cyl_lo,
            sphere_lo,
        }
    }

    pub fn group(&mut self, parent: Option<Entity>, t: Transform) -> Entity {
        let mut e = self.c.spawn((t, Visibility::default()));
        if let Some(p) = parent {
            e.insert(ChildOf(p));
        }
        e.id()
    }

    pub fn mat(&mut self, color: Color, roughness: f32, metallic: f32) -> Handle<StandardMaterial> {
        let key = format!(
            "m{:?}{roughness}/{metallic}",
            color.to_srgba().to_u8_array()
        );
        if let Some(h) = self.cache.get(&key) {
            return h.clone();
        }
        let h = self.mats.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: roughness,
            metallic,
            ..default()
        });
        self.cache.insert(key, h.clone());
        h
    }

    /// A static self-lit material. `strength` multiplies the colour; bloom
    /// picks up anything well above 1.0. The merge folds these into one
    /// unlit batch per object.
    pub fn glow(&mut self, color: Color, strength: f32) -> Handle<StandardMaterial> {
        let h = self.glow_live(color, strength);
        self.hints.glows.insert(h.id());
        h
    }

    /// A self-lit material that gets animated, so it keeps its own batch.
    pub fn glow_live(&mut self, color: Color, strength: f32) -> Handle<StandardMaterial> {
        self.mats.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: color.to_linear() * strength,
            perceptual_roughness: 0.4,
            ..default()
        })
    }

    pub fn mesh(
        &mut self,
        parent: Entity,
        mesh: Handle<Mesh>,
        t: Transform,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        self.c
            .spawn((Mesh3d(mesh), MeshMaterial3d(m.clone()), t, ChildOf(parent)))
            .id()
    }

    pub fn cube(
        &mut self,
        parent: Entity,
        at: Vec3,
        size: Vec3,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.cube.clone();
        self.mesh(
            parent,
            mesh,
            Transform::from_translation(at).with_scale(size),
            m,
        )
    }

    pub fn cube_t(&mut self, parent: Entity, t: Transform, m: &Handle<StandardMaterial>) -> Entity {
        let mesh = self.cube.clone();
        self.mesh(parent, mesh, t, m)
    }

    /// Upright cylinder centred at `at`.
    pub fn cyl(
        &mut self,
        parent: Entity,
        at: Vec3,
        radius: f32,
        height: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        self.cyl_t(parent, Transform::from_translation(at), radius, height, m)
    }

    /// Cylinder with an arbitrary transform; radius/height multiply its scale.
    pub fn cyl_t(
        &mut self,
        parent: Entity,
        t: Transform,
        radius: f32,
        height: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.cyl.clone();
        let t = t.with_scale(t.scale * Vec3::new(radius, height, radius));
        self.mesh(parent, mesh, t, m)
    }

    pub fn ball(
        &mut self,
        parent: Entity,
        at: Vec3,
        radius: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.sphere.clone();
        self.mesh(
            parent,
            mesh,
            Transform::from_translation(at).with_scale(Vec3::splat(radius)),
            m,
        )
    }

    /// A PBR-textured material (wood, brick…) tinted by `tint`, tiling
    /// `density` times per metre once merged.
    pub fn surf(&mut self, s: Surf, tint: Color, density: f32) -> Handle<StandardMaterial> {
        self.tex
            .material(self.assets, self.mats, self.box_uv, s, tint, density, 0.8)
    }

    /// A box with rounded edges (radius `r`).
    pub fn bevel(
        &mut self,
        parent: Entity,
        at: Vec3,
        size: Vec3,
        r: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        self.bevel_t(parent, Transform::from_translation(at), size, r, m)
    }

    pub fn bevel_t(
        &mut self,
        parent: Entity,
        t: Transform,
        size: Vec3,
        r: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.meshes.add(rounded_box(size, r));
        self.mesh(parent, mesh, t, m)
    }

    /// A soft contact shadow on the floor under `parent`-space point `at`.
    pub fn shadow(&mut self, parent: Entity, at: Vec3, size: Vec2) {
        let b = self.blob.bundle(at, size, 0.0);
        self.c.spawn((b, ChildOf(parent)));
    }

    /// A rod from `a` to `b` (used for neon tubes, chalk marks, chains).
    pub fn rod(
        &mut self,
        parent: Entity,
        a: Vec3,
        b: Vec3,
        radius: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let d = b - a;
        let t = Transform::from_translation((a + b) / 2.0)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, d.normalize_or(Vec3::Y)));
        self.cyl_t(parent, t, radius, d.length(), m)
    }

    /// `rod` with an 8-sided tube: for neon strokes and wires.
    pub fn rod_lo(
        &mut self,
        parent: Entity,
        a: Vec3,
        b: Vec3,
        radius: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let d = b - a;
        let t = Transform::from_translation((a + b) / 2.0)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, d.normalize_or(Vec3::Y)))
            .with_scale(Vec3::new(radius, d.length(), radius));
        let mesh = self.cyl_lo.clone();
        self.mesh(parent, mesh, t, m)
    }

    /// A low-poly ball for joints between `rod_lo` segments.
    pub fn ball_lo(
        &mut self,
        parent: Entity,
        at: Vec3,
        radius: f32,
        m: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.sphere_lo.clone();
        self.mesh(
            parent,
            mesh,
            Transform::from_translation(at).with_scale(Vec3::splat(radius)),
            m,
        )
    }
}

// ---------------------------------------------------------------------------
// Room
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
    featured: Res<Featured>,
    mut tex: ResMut<Textures>,
    mut box_uv: ResMut<BoxUv>,
    blob: Res<Blob>,
    mut hints: ResMut<MergeHints>,
) {
    hints.to_room.insert(blob.mat.id());
    let mut k = Kit::new(
        &mut commands,
        &mut meshes,
        &mut mats,
        &assets,
        &mut tex,
        &mut box_uv,
        &blob,
        &mut hints,
    );
    k.test_card = images.add(test_card());
    let root = k.group(None, Transform::IDENTITY);
    k.c.entity(root).insert(RoomRoot);

    shell(&mut k, root);
    lighting(&mut k, root);
    pool_table(&mut k, root);
    arcade_row(&mut k, root, &mut images, &assets, &featured.0);
    neon_signs(&mut k, root);
    jukebox(&mut k, root);
    left_wall(&mut k, root);
    bar(&mut k, root);
    lounge(&mut k, root);
    exit_door(&mut k, root);
    crate::dressing::dress(&mut k, root, &featured.0);
    crate::fill::fill(&mut k, root, &featured.0, &mut images);
}

fn shell(k: &mut Kit, root: Entity) {
    let (hx, hz, h) = (ROOM_HALF_X, ROOM_HALF_Z, ROOM_HEIGHT);

    // Dark varnished planks; the lamp and neon streak across them.
    let floor = k.surf(Surf::Floor, Color::srgb(0.62, 0.55, 0.5), 0.42);
    k.cube(
        root,
        Vec3::new(0.0, -0.05, 0.0),
        Vec3::new(hx * 2.0, 0.1, hz * 2.0),
        &floor,
    );

    // Rug under the table: a worn red field with a gold border.
    let rug = k.surf(Surf::Felt, Color::srgb(0.2, 0.035, 0.04), 1.6);
    k.cube(
        root,
        TABLE_CENTER.with_y(0.005),
        Vec3::new(4.6, 0.01, 3.2),
        &rug,
    );
    let rug_edge = k.mat(Color::srgb(0.5, 0.38, 0.18), 0.85, 0.0);
    k.cube(
        root,
        TABLE_CENTER.with_y(0.004),
        Vec3::new(4.8, 0.008, 3.4),
        &rug_edge,
    );

    let brick = k.surf(Surf::Brick, Color::srgb(0.55, 0.45, 0.42), 0.55);
    let plaster = k.surf(Surf::Plaster, Color::srgb(0.24, 0.22, 0.27), 0.6);
    let wood = k.surf(Surf::Wood, Color::srgb(0.55, 0.42, 0.34), 0.9);
    let trim = k.mat(Color::srgb(0.6, 0.45, 0.22), 0.35, 1.0);
    let ceiling = k.mat(Color::srgb(0.045, 0.04, 0.04), 1.0, 0.0);
    k.cube(
        root,
        Vec3::new(0.0, h + 0.05, 0.0),
        Vec3::new(hx * 2.0, 0.1, hz * 2.0),
        &ceiling,
    );

    // (centre, size, inward normal, surface): brick front and back, plaster on the sides.
    let walls = [
        (
            Vec3::new(0.0, h / 2.0, -hz - 0.1),
            Vec3::new(hx * 2.0, h, 0.2),
            Vec3::Z,
            &brick,
        ),
        (
            Vec3::new(0.0, h / 2.0, hz + 0.1),
            Vec3::new(hx * 2.0, h, 0.2),
            -Vec3::Z,
            &brick,
        ),
        (
            Vec3::new(-hx - 0.1, h / 2.0, 0.0),
            Vec3::new(0.2, h, hz * 2.0),
            Vec3::X,
            &plaster,
        ),
        (
            Vec3::new(hx + 0.1, h / 2.0, 0.0),
            Vec3::new(0.2, h, hz * 2.0),
            -Vec3::X,
            &plaster,
        ),
    ];
    for (centre, size, n, surface) in walls {
        let surface = surface.clone();
        if n == -Vec3::Z {
            crate::fill::front_wall(k, root, centre, size, &surface);
        } else {
            k.cube(root, centre, size, &surface);
        }
        let face = centre + n * 0.1;
        let along = if n.x == 0.0 { Vec3::X } else { Vec3::Z };
        let span = if n.x == 0.0 { hx * 2.0 } else { hz * 2.0 };
        let thick = |v: f32| n.abs() * v;
        // Wainscot to 1 m with a brass rail; skirting board at the floor.
        k.cube(
            root,
            face.with_y(0.5) + n * 0.02,
            along * span + Vec3::Y + thick(0.04),
            &wood,
        );
        k.cube(
            root,
            face.with_y(1.02) + n * 0.045,
            along * span + Vec3::Y * 0.04 + thick(0.03),
            &trim,
        );
        k.cube(
            root,
            face.with_y(0.06) + n * 0.05,
            along * span + Vec3::Y * 0.12 + thick(0.02),
            &wood,
        );
        // Vertical wainscot panels.
        let mut s = -span / 2.0 + 0.35;
        while s < span / 2.0 {
            k.cube(
                root,
                (face + along * s + n * 0.045).with_y(0.5),
                Vec3::Y * 0.9 + along * 0.025 + thick(0.012),
                &wood,
            );
            s += 0.7;
        }
    }
}

fn lighting(k: &mut Kit, root: Entity) {
    // Barely-there ambient: the lamp, the neon and the environment map do the
    // lighting; corners are allowed to fall into darkness.
    k.c.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.5, 0.45, 0.7),
        brightness: 8.0,
        ..default()
    });
    // One low warm fill over the bar end so the back wall doesn't vanish.
    k.c.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.7, 0.45),
            intensity: 22_000.0,
            range: 5.5,
            ..default()
        },
        Transform::from_xyz(3.6, 2.9, -3.2),
        ChildOf(root),
    ));
}

fn pool_table(k: &mut Kit, root: Entity) {
    let c = TABLE_CENTER;
    let table = k.group(Some(root), Transform::from_translation(c.with_y(0.0)));
    k.c.entity(table).insert((
        Interactive::new(
            "Pool Table — play a round",
            Action::Hotspot(Hotspot::PoolTable),
        ),
        CastsShadows,
    ));

    let felt = k.surf(Surf::Felt, Color::srgb(0.10, 0.52, 0.30), 3.0);
    let wood = k.surf(Surf::Wood, Color::srgb(0.75, 0.48, 0.32), 2.2);
    let leather = k.surf(Surf::Leather, Color::srgb(0.35, 0.22, 0.15), 6.0);
    let black = k.mat(Color::srgb(0.01, 0.01, 0.01), 0.8, 0.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.28, 1.0);
    let pearl = k.mat(Color::srgb(0.95, 0.92, 0.85), 0.25, 0.0);
    let chalk = k.mat(Color::srgb(0.12, 0.38, 0.85), 0.95, 0.0);

    let (hl, hw, top) = (TABLE_HALF_LEN, TABLE_HALF_WID, c.y);
    let (ol, ow) = (hl + RAIL, hw + RAIL);

    // Slate bed in felt, a bevelled apron, turned legs on brass feet.
    k.cube(
        table,
        Vec3::new(0.0, top - 0.02, 0.0),
        Vec3::new(hl * 2.0 + 0.02, 0.04, hw * 2.0 + 0.02),
        &felt,
    );
    k.bevel(
        table,
        Vec3::new(0.0, top - 0.16, 0.0),
        Vec3::new(ol * 2.0, 0.24, ow * 2.0),
        0.03,
        &wood,
    );
    k.bevel(
        table,
        Vec3::new(0.0, top - 0.3, 0.0),
        Vec3::new(ol * 2.0 - 0.12, 0.05, ow * 2.0 - 0.12),
        0.02,
        &wood,
    );
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let x = sx * (ol - 0.18);
        let z = sz * (ow - 0.16);
        k.bevel(
            table,
            Vec3::new(x, (top - 0.3) / 2.0 + 0.05, z),
            Vec3::new(0.15, top - 0.4, 0.15),
            0.03,
            &wood,
        );
        k.cyl(table, Vec3::new(x, 0.12, z), 0.095, 0.08, &wood);
        k.cyl(table, Vec3::new(x, 0.035, z), 0.1, 0.07, &brass);
        k.shadow(table, Vec3::new(x, 0.0, z), Vec2::splat(0.45));
    }
    k.shadow(table, Vec3::ZERO, Vec2::new(ol * 2.0 + 0.5, ow * 2.0 + 0.5));

    // Rails (bevelled) and felt cushions, split around the six pockets.
    let gap = 0.075;
    let rail_h = 0.06;
    for sz in [-1.0f32, 1.0] {
        for (a, b) in [(-hl + gap, -gap), (gap, hl - gap)] {
            let mid = (a + b) / 2.0;
            k.bevel(
                table,
                Vec3::new(
                    mid,
                    top + rail_h / 2.0 - 0.01,
                    sz * (hw + RAIL / 2.0 + 0.02),
                ),
                Vec3::new(b - a, rail_h, RAIL - 0.03),
                0.018,
                &wood,
            );
            k.bevel(
                table,
                Vec3::new(mid, top + 0.018, sz * (hw + 0.022)),
                Vec3::new(b - a, 0.036, 0.045),
                0.012,
                &felt,
            );
        }
    }
    for sx in [-1.0f32, 1.0] {
        let len = hw * 2.0 - gap * 2.0;
        k.bevel(
            table,
            Vec3::new(
                sx * (hl + RAIL / 2.0 + 0.02),
                top + rail_h / 2.0 - 0.01,
                0.0,
            ),
            Vec3::new(RAIL - 0.03, rail_h, len),
            0.018,
            &wood,
        );
        k.bevel(
            table,
            Vec3::new(sx * (hl + 0.022), top + 0.018, 0.0),
            Vec3::new(0.045, 0.036, len),
            0.012,
            &felt,
        );
    }
    // Mother-of-pearl diamond sights inlaid in the rails.
    let sight = |k: &mut Kit, at: Vec3| {
        let t = Transform::from_translation(at)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4));
        k.cube_t(table, t.with_scale(Vec3::new(0.018, 0.004, 0.018)), &pearl);
    };
    for i in (1..8).filter(|i| *i != 4) {
        let x = -hl + i as f32 * hl * 2.0 / 8.0;
        for sz in [-1.0, 1.0] {
            sight(
                k,
                Vec3::new(x, top + rail_h - 0.008, sz * (hw + RAIL * 0.6)),
            );
        }
    }
    for i in [1, 3] {
        let z = -hw + i as f32 * hw * 2.0 / 4.0;
        for sx in [-1.0, 1.0] {
            sight(
                k,
                Vec3::new(sx * (hl + RAIL * 0.6), top + rail_h - 0.008, z),
            );
        }
    }
    // Pockets: leather cups under brass-capped corners.
    for (px, pz) in [
        (-hl, -hw),
        (0.0, -hw),
        (hl, -hw),
        (-hl, hw),
        (0.0, hw),
        (hl, hw),
    ] {
        let p = Vec3::new(px * 1.01, top - 0.04, pz * 1.03);
        k.cyl(table, p, 0.072, 0.11, &leather);
        k.cyl(table, p.with_y(top + 0.002), 0.06, 0.012, &black);
        if px != 0.0 {
            k.bevel(
                table,
                Vec3::new(
                    px.signum() * (ol - 0.02),
                    top + rail_h - 0.005,
                    pz.signum() * (ow - 0.02),
                ),
                Vec3::new(0.11, 0.012, 0.11),
                0.005,
                &brass,
            );
        }
    }
    // Two cubes of blue chalk left on the rails.
    for (x, z, yaw) in [
        (-hl + 0.32, -(hw + RAIL * 0.55), 0.3f32),
        (hl - 0.5, hw + RAIL * 0.55, -0.5),
    ] {
        let t = Transform::from_xyz(x, top + rail_h + 0.001, z)
            .with_rotation(Quat::from_rotation_y(yaw));
        k.bevel_t(table, t, Vec3::splat(0.022), 0.003, &chalk);
    }

    // Hanging lamp: long shade, glowing underside, chains, warm spot. The
    // shade hides while you play (the camera looks down past it); the light stays.
    let lamp_y = 1.85;
    let lamp = k.group(Some(table), Transform::IDENTITY);
    k.c.entity(lamp).insert(LampShade);
    let shade = k.mat(Color::srgb(0.04, 0.22, 0.12), 0.35, 0.1);
    k.bevel(
        lamp,
        Vec3::new(0.0, lamp_y, 0.0),
        Vec3::new(1.9, 0.14, 0.42),
        0.035,
        &shade,
    );
    k.bevel(
        lamp,
        Vec3::new(0.0, lamp_y + 0.08, 0.0),
        Vec3::new(1.95, 0.025, 0.46),
        0.01,
        &brass,
    );
    let bulb = k.glow(Color::srgb(1.0, 0.82, 0.55), 9.0);
    k.cube(
        lamp,
        Vec3::new(0.0, lamp_y - 0.071, 0.0),
        Vec3::new(1.82, 0.004, 0.36),
        &bulb,
    );
    // A warm lip around the shade so the lamp reads from above, too.
    let lip = k.glow(Color::srgb(1.0, 0.7, 0.4), 3.5);
    k.cube(
        lamp,
        Vec3::new(0.0, lamp_y - 0.06, 0.0),
        Vec3::new(1.92, 0.018, 0.44),
        &lip,
    );
    for sx in [-0.8, 0.8] {
        k.rod(
            lamp,
            Vec3::new(sx, lamp_y + 0.09, 0.0),
            Vec3::new(sx, ROOM_HEIGHT, 0.0),
            0.006,
            &brass,
        );
        k.cyl(
            lamp,
            Vec3::new(sx, ROOM_HEIGHT - 0.01, 0.0),
            0.04,
            0.02,
            &brass,
        );
    }
    // The hero light: one shadowed spot does the work of a row of bulbs.
    k.c.spawn((
        SpotLight {
            color: Color::srgb(1.0, 0.82, 0.6),
            intensity: 320_000.0,
            range: 5.0,
            radius: 0.3,
            outer_angle: 1.05,
            inner_angle: 0.55,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, lamp_y - 0.1, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ShadowCaster,
        ChildOf(table),
    ));
}

fn arcade_row(
    k: &mut Kit,
    root: Entity,
    images: &mut Assets<Image>,
    assets: &AssetServer,
    featured: &[FeaturedProject],
) {
    if featured.len() > MAX_CABINETS {
        warn!(
            "{} featured projects; only the first {MAX_CABINETS} get cabinets",
            featured.len()
        );
    }
    let featured = &featured[..featured.len().min(MAX_CABINETS)];
    CABINETS.store(featured.len(), Ordering::Relaxed);
    let body = k.mat(Color::srgb(0.035, 0.035, 0.045), 0.45, 0.1);
    let panel = k.mat(Color::srgb(0.09, 0.09, 0.11), 0.35, 0.2);
    let black = k.mat(Color::srgb(0.005, 0.005, 0.005), 0.2, 0.0);
    let stick = k.mat(Color::srgb(0.85, 0.1, 0.12), 0.3, 0.0);

    for (i, p) in featured.iter().enumerate() {
        let [r, g, b] = p.color;
        let accent = Color::srgb_u8(r, g, b);
        let cab = k.group(
            Some(root),
            Transform::from_xyz(cabinet_x(i), 0.0, CABINET_Z),
        );
        k.c.entity(cab).insert(Interactive::new(
            p.title.clone(),
            Action::Cabinet {
                index: i,
                path: format!("/work/{}", p.slug),
            },
        ));

        // Body: bevelled base and hood, coloured T-moulding on every edge.
        k.bevel(
            cab,
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(0.72, 1.0, 0.75),
            0.02,
            &body,
        );
        k.bevel(
            cab,
            Vec3::new(0.0, 1.42, -0.12),
            Vec3::new(0.72, 0.84, 0.5),
            0.02,
            &body,
        );
        let trim = k.glow(accent, 0.7);
        for sx in [-0.363, 0.363] {
            k.cube(
                cab,
                Vec3::new(sx, 0.5, 0.375),
                Vec3::new(0.012, 1.0, 0.012),
                &trim,
            );
            k.cube(
                cab,
                Vec3::new(sx, 1.42, 0.13),
                Vec3::new(0.012, 0.84, 0.012),
                &trim,
            );
            k.cube(
                cab,
                Vec3::new(sx, 0.95, 0.0),
                Vec3::new(0.006, 1.7, 0.08),
                &trim,
            );
        }
        // Coin door with two lit slots, and a speaker grille under the marquee.
        let metal = k.mat(Color::srgb(0.12, 0.12, 0.13), 0.35, 0.9);
        k.bevel(
            cab,
            Vec3::new(0.0, 0.42, 0.378),
            Vec3::new(0.3, 0.32, 0.01),
            0.004,
            &metal,
        );
        let slot = k.glow(Color::srgb(1.0, 0.45, 0.12), 3.0);
        for sx in [-0.06, 0.06] {
            k.cube(
                cab,
                Vec3::new(sx, 0.48, 0.385),
                Vec3::new(0.035, 0.05, 0.004),
                &slot,
            );
        }
        for j in 0..4 {
            k.cube(
                cab,
                Vec3::new(0.0, 1.72 + j as f32 * 0.018, 0.13),
                Vec3::new(0.5, 0.006, 0.006),
                &metal,
            );
        }
        k.shadow(cab, Vec3::new(0.0, 0.0, 0.05), Vec2::new(1.0, 1.0));

        // Control panel, tilted toward the player.
        let deck = Transform::from_xyz(0.0, 1.03, 0.28)
            .with_rotation(Quat::from_rotation_x(-0.35))
            .with_scale(Vec3::new(0.7, 0.06, 0.32));
        k.cube_t(cab, deck, &panel);
        k.cyl(cab, Vec3::new(-0.15, 1.1, 0.29), 0.008, 0.08, &black);
        k.ball(cab, Vec3::new(-0.15, 1.15, 0.29), 0.022, &stick);
        let btn = k.glow(accent, 2.5);
        for (j, bx) in [0.02, 0.1, 0.18].into_iter().enumerate() {
            k.cyl(
                cab,
                Vec3::new(bx, 1.08 + j as f32 * 0.0, 0.3),
                0.022,
                0.03,
                &btn,
            );
        }

        // Screen: a procedural glow until the project's cover image loads.
        let cover = images.add(cover_image(p.color, i as u32));
        let screen = k.mats.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::WHITE * 2.2,
            emissive_texture: Some(cover),
            perceptual_roughness: 0.1,
            ..default()
        });
        let st = Transform::from_xyz(0.0, 1.42, 0.135)
            .with_rotation(Quat::from_rotation_x(-0.22))
            .with_scale(Vec3::new(0.58, 0.44, 0.01));
        k.cube_t(cab, st, &screen);
        let bezel = Transform::from_xyz(0.0, 1.42, 0.128)
            .with_rotation(Quat::from_rotation_x(-0.22))
            .with_scale(Vec3::new(0.66, 0.52, 0.01));
        k.cube_t(cab, bezel, &black);
        if let Some(path) = &p.cover {
            let image = assets.load::<Image>(path.clone());
            k.c.spawn((
                PendingCover {
                    image,
                    mat: screen,
                    crt: true,
                },
                ChildOf(cab),
            ));
        }

        // Marquee on top.
        let marquee = k.glow(accent, 4.5);
        k.cube(
            cab,
            Vec3::new(0.0, 1.93, 0.0),
            Vec3::new(0.7, 0.16, 0.04),
            &marquee,
        );
        k.cube(
            cab,
            Vec3::new(0.0, 1.93, -0.1),
            Vec3::new(0.72, 0.2, 0.22),
            &body,
        );

        k.c.spawn((
            PointLight {
                color: accent,
                intensity: 9_000.0,
                range: 2.5,
                ..default()
            },
            Transform::from_xyz(0.0, 1.5, 0.7),
            ChildOf(cab),
        ));
    }
}

/// A synthwave-ish placeholder cover: gradient, sun, scanlines.
fn cover_image([r, g, b]: [u8; 3], seed: u32) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let (w, h) = (128u32, 96u32);
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            // Row 0 is the top of the texture; the cube face samples it upright.
            let v = 1.0 - y as f32 / h as f32;
            let u = x as f32 / w as f32;
            // Sun disc with slices cut out of the lower half.
            let (dx, dy) = (u - 0.5, (v - 0.55) * 0.75);
            let sun = (dx * dx + dy * dy).sqrt() < 0.22
                && !(v > 0.5 && ((v * 40.0) as u32 + seed).is_multiple_of(3));
            // Perspective grid floor.
            let grid = v > 0.72
                && (((u - 0.5) / (v - 0.6) * 6.0).fract().abs() < 0.08
                    || ((1.0 / (v - 0.6)) * 2.0).fract() < 0.12);
            let scan = if y % 2 == 0 { 1.0 } else { 0.7 };
            let base = 0.12 + 0.35 * (1.0 - v);
            let (fr, fg, fb) = if sun {
                (1.0, 0.85, 0.6)
            } else if grid {
                (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
            } else {
                (
                    base * r as f32 / 255.0,
                    base * g as f32 / 255.0,
                    base * b as f32 / 255.0 + 0.05,
                )
            };
            for c in [fr, fg, fb] {
                data.push((c * scan * 255.0).clamp(0.0, 255.0) as u8);
            }
            data.push(255);
        }
    }
    let data = crt(&data, w, h);
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Width the cabinet screens actually need.
const SCREEN_TEX_WIDTH: u32 = 320;

/// Box-filter a loaded RGBA8 cover down to screen size, flipping it upright
/// for the cube face (whose V runs bottom-up). Without this a 1600 px cover
/// minified onto ~200 px of screen with no mipmaps turns to moiré.
fn screen_texture(src: &Image, crt_look: bool) -> Option<Image> {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension};

    let data = src.data.as_ref()?;
    let (w, h) = (src.width(), src.height());
    if src.texture_descriptor.format.block_copy_size(None) != Some(4)
        || data.len() < (w * h * 4) as usize
    {
        return None;
    }
    let target = if crt_look { SCREEN_TEX_WIDTH } else { 512 };
    let f = w.div_ceil(target).max(1);
    let (nw, nh) = (w / f, h / f);
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            let mut acc = [0u32; 4];
            for dy in 0..f {
                for dx in 0..f {
                    let i = (((y * f + dy) * w + x * f + dx) * 4) as usize;
                    for c in 0..4 {
                        acc[c] += data[i + c] as u32;
                    }
                }
            }
            let o = (((nh - 1 - y) * nw + x) * 4) as usize;
            for c in 0..4 {
                out[o + c] = (acc[c] / (f * f)) as u8;
            }
        }
    }
    let out = if crt_look { crt(&out, nw, nh) } else { out };
    Some(Image::new(
        Extent3d {
            width: nw,
            height: nh,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        out,
        src.texture_descriptor.format,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

/// SMPTE-ish colour bars with an "ON AIR" block, through the CRT filter.
fn test_card() -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let (w, h) = (256u32, 144u32);
    let bars = [
        [192, 192, 192],
        [192, 192, 0],
        [0, 192, 192],
        [0, 192, 0],
        [192, 0, 192],
        [192, 0, 0],
        [0, 0, 192],
    ];
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let c: [u8; 3] = if y < h * 2 / 3 {
                bars[(x * 7 / w) as usize]
            } else if y < h * 3 / 4 {
                bars[6 - (x * 7 / w) as usize]
            } else if x < w / 3 {
                [12, 12, 40]
            } else if x < w * 2 / 3 {
                [235, 235, 235]
            } else {
                [20, 20, 20]
            };
            data.extend([c[0], c[1], c[2], 255]);
        }
    }
    // Flip rows for the cube face.
    let row = (w * 4) as usize;
    let mut flipped = Vec::with_capacity(data.len());
    for r in (0..h as usize).rev() {
        flipped.extend_from_slice(&data[r * row..(r + 1) * row]);
    }
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        crt(&flipped, w, h),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Make a flat RGBA image look like it's on a tube: barrel curvature (with
/// black, rounded corners where the glass curves away), vignette, scanlines
/// and a faint RGB phosphor mask.
pub fn crt(src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; src.len()];
    let sample = |x: f32, y: f32, c: usize| -> f32 {
        let (x, y) = (x.clamp(0.0, (w - 1) as f32), y.clamp(0.0, (h - 1) as f32));
        let (x0, y0) = (x.floor() as u32, y.floor() as u32);
        let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let px = |x: u32, y: u32| src[((y * w + x) * 4) as usize + c] as f32;
        let top = px(x0, y0) * (1.0 - fx) + px(x1, y0) * fx;
        let bot = px(x0, y1) * (1.0 - fx) + px(x1, y1) * fx;
        top * (1.0 - fy) + bot * fy
    };
    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / h as f32 * 2.0 - 1.0;
            let r2 = u * u + v * v;
            let (bu, bv) = (u * (1.0 + 0.07 * r2), v * (1.0 + 0.1 * r2));
            let o = ((y * w + x) * 4) as usize;
            out[o + 3] = 255;
            if bu.abs() > 1.0 || bv.abs() > 1.0 {
                continue;
            }
            let edge = ((1.0 - bu.abs()).min(1.0 - bv.abs()) * 40.0).clamp(0.0, 1.0);
            let vignette = (1.0 - 0.45 * (bu * bu + bv * bv)).max(0.0) * edge;
            let scan = if y % 2 == 0 { 1.0 } else { 0.62 };
            let sx = (bu + 1.0) / 2.0 * w as f32 - 0.5;
            let sy = (bv + 1.0) / 2.0 * h as f32 - 0.5;
            for c in 0..3 {
                let mask = if x % 3 == c as u32 { 1.0 } else { 0.82 };
                out[o + c] = (sample(sx, sy, c) * vignette * scan * mask).min(255.0) as u8;
            }
        }
    }
    out
}

fn swap_in_covers(
    mut commands: Commands,
    pending: Query<(Entity, &PendingCover)>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    use bevy::asset::LoadState;
    for (e, p) in &pending {
        match assets.load_state(&p.image) {
            LoadState::Loaded => {
                let small = images.get(&p.image).and_then(|i| screen_texture(i, p.crt));
                let tex = small
                    .map(|i| images.add(i))
                    .unwrap_or_else(|| p.image.clone());
                if let Some(mut m) = mats.get_mut(&p.mat) {
                    if p.crt {
                        m.emissive_texture = Some(tex);
                        m.emissive = LinearRgba::WHITE * 1.6;
                    } else {
                        m.base_color_texture = Some(tex);
                        m.base_color = Color::WHITE;
                        m.emissive_texture = m.base_color_texture.clone();
                        m.emissive = LinearRgba::WHITE * 0.06;
                    }
                }
                commands.entity(e).despawn();
            }
            // No cover on disk: keep the procedural one.
            LoadState::Failed(_) => {
                commands.entity(e).despawn();
            }
            _ => {}
        }
    }
}

fn neon_signs(k: &mut Kit, root: Entity) {
    // Main sign on the back wall, above the couch.
    let backing = k.mat(Color::srgb(0.02, 0.02, 0.025), 0.15, 0.0);
    k.cube(
        root,
        Vec3::new(-2.3, 2.45, -4.97),
        Vec3::new(4.3, 1.15, 0.03),
        &backing,
    );
    let sign = neon::sign(
        k,
        root,
        "ARTHUB",
        Transform::from_xyz(-2.3, 2.45, -4.92),
        0.12,
        NeonColor::Accent,
        Some(Vec3::new(-2.3, 2.3, -4.2)),
    );
    k.c.entity(sign).insert(Interactive::new(
        "Neon — click to change colour",
        Action::NeonCycle,
    ));

    // OPEN above the bar, facing into the room.
    neon::sign(
        k,
        root,
        "OPEN",
        Transform::from_xyz(6.93, 2.55, -2.3)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        0.09,
        NeonColor::Pink,
        Some(Vec3::new(6.3, 2.5, -2.3)),
    );

    // 8-ball ring on the left wall.
    neon::eight_ball(
        k,
        root,
        Transform::from_xyz(-6.93, 2.6, 0.2)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        0.32,
    );
}

fn jukebox(k: &mut Kit, root: Entity) {
    let jb = k.group(Some(root), Transform::from_translation(JUKEBOX_POS));
    k.c.entity(jb).insert(Interactive::new(
        "Jukebox — toggle music",
        Action::Hotspot(Hotspot::Jukebox),
    ));

    let wood = k.surf(Surf::Wood, Color::srgb(0.95, 0.5, 0.3), 2.0);
    let chrome = k.mat(Color::srgb(0.92, 0.92, 0.95), 0.08, 1.0);
    let glass = k.mat(Color::srgb(0.04, 0.03, 0.05), 0.04, 0.0);
    let half_pi = std::f32::consts::FRAC_PI_2;

    k.bevel(
        jb,
        Vec3::new(0.0, 0.6, 0.0),
        Vec3::new(0.95, 1.2, 0.6),
        0.04,
        &wood,
    );
    // Arched top: a cylinder lying along z, framed in chrome.
    let arch = Transform::from_xyz(0.0, 1.2, 0.0).with_rotation(Quat::from_rotation_x(half_pi));
    k.cyl_t(jb, arch, 0.475, 0.6, &wood);
    let trim = k.meshes.add(Torus::new(0.462, 0.49));
    for z in [-0.29, 0.29] {
        k.mesh(
            jb,
            trim.clone(),
            Transform::from_xyz(0.0, 1.2, z).with_rotation(Quat::from_rotation_x(half_pi)),
            &chrome,
        );
    }
    // Record window, selector strip, speaker grille, chrome kick plate.
    k.bevel(
        jb,
        Vec3::new(0.0, 1.07, 0.3),
        Vec3::new(0.6, 0.36, 0.02),
        0.01,
        &glass,
    );
    let labels = k.glow(Color::srgb(1.0, 0.9, 0.7), 1.2);
    k.cube(
        jb,
        Vec3::new(0.0, 0.8, 0.305),
        Vec3::new(0.56, 0.08, 0.01),
        &labels,
    );
    for j in 0..7 {
        k.rod(
            jb,
            Vec3::new(-0.3, 0.45 + j as f32 * 0.035, 0.31),
            Vec3::new(0.3, 0.45 + j as f32 * 0.035, 0.31),
            0.006,
            &chrome,
        );
    }
    k.bevel(
        jb,
        Vec3::new(0.0, 0.08, 0.3),
        Vec3::new(0.95, 0.14, 0.03),
        0.01,
        &chrome,
    );
    for sx in [-0.475, 0.475] {
        k.rod(
            jb,
            Vec3::new(sx, 0.02, 0.3),
            Vec3::new(sx, 1.2, 0.3),
            0.012,
            &chrome,
        );
    }
    k.shadow(jb, Vec3::new(0.0, 0.0, 0.05), Vec2::new(1.4, 1.0));

    // Glowing bands: an arch ring and two vertical tubes, each pulsing.
    let palette = [
        Color::srgb(1.0, 0.18, 0.53),
        Color::srgb(1.0, 0.65, 0.15),
        Color::srgb(0.13, 0.9, 1.0),
    ];
    let torus = k.meshes.add(Torus::new(0.36, 0.40));
    for (i, col) in palette.into_iter().enumerate() {
        let m = k.glow_live(col, 3.0);
        k.c.spawn((
            JukeboxBand {
                mat: m.clone(),
                phase: i as f32 * 2.1,
            },
            ChildOf(jb),
        ));
        match i {
            0 => {
                let t = Transform::from_xyz(0.0, 1.2, 0.31)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
                k.mesh(jb, torus.clone(), t, &m);
            }
            _ => {
                let sx = if i == 1 { -0.4 } else { 0.4 };
                k.cube(
                    jb,
                    Vec3::new(sx, 0.62, 0.31),
                    Vec3::new(0.035, 1.05, 0.02),
                    &m,
                );
            }
        }
    }
    k.c.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.4, 0.6),
            intensity: 25_000.0,
            range: 3.5,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.8),
        ChildOf(jb),
    ));
}

fn left_wall(k: &mut Kit, root: Entity) {
    use std::f32::consts::FRAC_PI_2;
    let face_x = Quat::from_rotation_y(FRAC_PI_2); // local +z → world +x

    // TV
    let tv = k.group(
        Some(root),
        Transform::from_translation(TV_POS).with_rotation(face_x),
    );
    k.c.entity(tv).insert(Interactive::new(
        "TV — showreel",
        Action::Hotspot(Hotspot::Tv),
    ));
    let frame = k.mat(Color::srgb(0.015, 0.015, 0.015), 0.3, 0.2);
    k.cube(tv, Vec3::ZERO, Vec3::new(1.7, 0.98, 0.06), &frame);
    let screen = k.mats.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: LinearRgba::WHITE * 0.9,
        emissive_texture: Some(k.test_card.clone()),
        perceptual_roughness: 0.15,
        ..default()
    });
    k.cube(
        tv,
        Vec3::new(0.0, 0.0, 0.031),
        Vec3::new(1.6, 0.9, 0.002),
        &screen,
    );
    k.c.spawn((TvScreen(screen), ChildOf(tv)));
    k.c.spawn((
        PointLight {
            color: Color::srgb(0.5, 0.5, 1.0),
            intensity: 15_000.0,
            range: 3.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.6),
        ChildOf(tv),
    ));

    // Trophy shelf
    let shelf = k.group(
        Some(root),
        Transform::from_translation(TROPHY_POS).with_rotation(face_x),
    );
    k.c.entity(shelf).insert(Interactive::new(
        "Trophy shelf — about me",
        Action::Hotspot(Hotspot::TrophyShelf),
    ));
    let oak = k.mat(Color::srgb(0.32, 0.17, 0.08), 0.5, 0.0);
    let gold = k.mat(Color::srgb(1.0, 0.76, 0.33), 0.22, 1.0);
    let silver = k.mat(Color::srgb(0.85, 0.86, 0.9), 0.2, 1.0);
    let marble = k.mat(Color::srgb(0.08, 0.08, 0.08), 0.3, 0.0);
    for y in [0.0, 0.55] {
        k.cube(
            shelf,
            Vec3::new(0.0, y, 0.12),
            Vec3::new(1.4, 0.04, 0.28),
            &oak,
        );
    }
    let cup = k.meshes.add(ConicalFrustum {
        radius_top: 0.07,
        radius_bottom: 0.025,
        height: 0.12,
    });
    let handle = k.meshes.add(Torus::new(0.022, 0.032));
    for (i, (x, y, s)) in [
        (-0.45, 0.0, 1.0),
        (-0.1, 0.0, 1.35),
        (0.3, 0.0, 0.9),
        (-0.3, 0.55, 1.1),
        (0.25, 0.55, 1.2),
    ]
    .into_iter()
    .enumerate()
    {
        let m = if i % 2 == 0 { &gold } else { &silver };
        let base = Vec3::new(x, y + 0.02, 0.12);
        k.cube(
            shelf,
            base + Vec3::Y * 0.03 * s,
            Vec3::new(0.09, 0.06, 0.09) * s,
            &marble,
        );
        k.cyl(shelf, base + Vec3::Y * 0.1 * s, 0.012 * s, 0.08 * s, m);
        let t = Transform::from_translation(base + Vec3::Y * 0.2 * s).with_scale(Vec3::splat(s));
        k.mesh(shelf, cup.clone(), t, m);
        for hx in [-0.07, 0.07] {
            let t = Transform::from_translation(base + Vec3::new(hx * s, 0.21 * s, 0.0))
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(Vec3::splat(s));
            k.mesh(shelf, handle.clone(), t, m);
        }
    }
    k.c.spawn((
        SpotLight {
            color: Color::srgb(1.0, 0.85, 0.6),
            intensity: 40_000.0,
            range: 3.0,
            outer_angle: 0.7,
            ..default()
        },
        Transform::from_xyz(0.0, 1.3, 0.6).looking_at(Vec3::new(0.0, 0.2, 0.1), Vec3::Y),
        ChildOf(shelf),
    ));

    // Chalkboard
    let board = k.group(
        Some(root),
        Transform::from_translation(CHALKBOARD_POS).with_rotation(face_x),
    );
    k.c.entity(board).insert(Interactive::new(
        "Chalkboard — skills",
        Action::Hotspot(Hotspot::Chalkboard),
    ));
    k.cube(board, Vec3::ZERO, Vec3::new(1.9, 1.15, 0.05), &oak);
    let slate = k.mat(Color::srgb(0.035, 0.05, 0.045), 0.95, 0.0);
    k.cube(
        board,
        Vec3::new(0.0, 0.0, 0.026),
        Vec3::new(1.78, 1.03, 0.004),
        &slate,
    );
    k.cube(
        board,
        Vec3::new(0.0, -0.6, 0.06),
        Vec3::new(1.8, 0.03, 0.08),
        &oak,
    );
    let chalk = k.mat(Color::srgb(0.85, 0.85, 0.8), 1.0, 0.0);
    // Headings in chalk: SKILLS on the left half, POOL tallies on the right.
    neon::stroke_text(
        k,
        board,
        "SKILLS",
        Transform::from_xyz(-0.45, 0.36, 0.03),
        0.022,
        0.0035,
        &chalk,
    );
    neon::stroke_text(
        k,
        board,
        "POOL",
        Transform::from_xyz(0.45, 0.36, 0.03),
        0.022,
        0.0035,
        &chalk,
    );
    k.rod(
        board,
        Vec3::new(0.0, 0.48, 0.03),
        Vec3::new(0.0, -0.45, 0.03),
        0.004,
        &chalk,
    );
    // Skill tallies: rows of chalk strokes, longer = stronger.
    for (row, n) in [7, 6, 6, 5, 4].into_iter().enumerate() {
        let y = 0.12 - row as f32 * 0.12;
        k.rod(
            board,
            Vec3::new(-0.8, y, 0.03),
            Vec3::new(-0.55, y, 0.03),
            0.004,
            &chalk,
        );
        for j in 0..n {
            let x = -0.48 + j as f32 * 0.05;
            k.rod(
                board,
                Vec3::new(x, y - 0.035, 0.03),
                Vec3::new(x + 0.01, y + 0.035, 0.03),
                0.004,
                &chalk,
            );
        }
    }
    let tallies = k.group(Some(board), Transform::from_xyz(0.12, 0.12, 0.03));
    k.c.insert_resource(Chalkboard {
        root: tallies,
        chalk,
    });

    // Dartboard (goes to the lab)
    let dart = k.group(
        Some(root),
        Transform::from_translation(DARTBOARD_POS).with_rotation(face_x),
    );
    k.c.entity(dart)
        .insert(Interactive::new("Dartboard — the lab", Action::Dartboard));
    let face = Quat::from_rotation_x(FRAC_PI_2);
    let rings = [
        (0.27, Color::srgb(0.02, 0.02, 0.02)),
        (0.23, Color::srgb(0.75, 0.08, 0.08)),
        (0.215, Color::srgb(0.9, 0.85, 0.7)),
        (0.15, Color::srgb(0.05, 0.4, 0.15)),
        (0.135, Color::srgb(0.9, 0.85, 0.7)),
        (0.035, Color::srgb(0.05, 0.4, 0.15)),
        (0.016, Color::srgb(0.75, 0.08, 0.08)),
    ];
    for (i, (r, col)) in rings.into_iter().enumerate() {
        let m = k.mat(col, 0.9, 0.0);
        let t = Transform::from_xyz(0.0, 0.0, 0.02 + i as f32 * 0.002).with_rotation(face);
        k.cyl_t(dart, t, r, 0.03, &m);
    }
    let wire = k.mat(Color::srgb(0.7, 0.7, 0.7), 0.3, 1.0);
    for i in 0..10 {
        let a = i as f32 * std::f32::consts::PI / 10.0;
        let d = Vec3::new(a.cos(), a.sin(), 0.0) * 0.23;
        k.rod(dart, Vec3::Z * 0.05 - d, Vec3::Z * 0.05 + d, 0.0015, &wire);
    }
}

fn bar(k: &mut Kit, root: Entity) {
    let bar = k.group(Some(root), Transform::IDENTITY);
    k.c.entity(bar).insert(Interactive::new(
        "Bar — reserve a table",
        Action::Hotspot(Hotspot::Bar),
    ));

    let wood = k.surf(Surf::Wood, Color::srgb(0.7, 0.4, 0.28), 1.5);
    let top = k.mats.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.03, 0.02),
        perceptual_roughness: 0.12,
        clearcoat: 1.0,
        ..default()
    });
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let leather = k.surf(Surf::Leather, Color::srgb(0.6, 0.18, 0.12), 4.0);

    // Counter runs along the right wall.
    let (x, z0, z1) = (5.6, -4.3, -0.5);
    let zm = (z0 + z1) / 2.0;
    k.bevel(
        bar,
        Vec3::new(x, 0.53, zm),
        Vec3::new(0.6, 1.06, z1 - z0),
        0.02,
        &wood,
    );
    k.bevel(
        bar,
        Vec3::new(x - 0.05, 1.08, zm),
        Vec3::new(0.75, 0.05, z1 - z0 + 0.1),
        0.02,
        &top,
    );
    k.shadow(
        bar,
        Vec3::new(x - 0.1, 0.0, zm),
        Vec2::new(1.0, z1 - z0 + 0.4),
    );
    k.cyl_t(
        bar,
        Transform::from_xyz(x - 0.42, 0.18, zm)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        0.02,
        z1 - z0,
        &brass,
    );
    for i in 0..4 {
        let z = z0 + 0.5 + i as f32 * 0.95;
        let s = Vec3::new(x - 0.85, 0.0, z);
        k.cyl(bar, s + Vec3::Y * 0.38, 0.025, 0.76, &brass);
        k.cyl(bar, s + Vec3::Y * 0.02, 0.2, 0.04, &brass);
        k.cyl(bar, s + Vec3::Y * 0.78, 0.2, 0.07, &leather);
        k.shadow(bar, s, Vec2::splat(0.55));
    }

    // Back bar: shelves of bottles, lit from below.
    for y in [1.4, 1.8] {
        k.cube(
            bar,
            Vec3::new(6.82, y, zm),
            Vec3::new(0.3, 0.03, z1 - z0),
            &wood,
        );
    }
    let glass_cols = [
        Color::srgb(0.1, 0.5, 0.15),
        Color::srgb(0.6, 0.3, 0.05),
        Color::srgb(0.7, 0.75, 0.8),
        Color::srgb(0.4, 0.05, 0.1),
        Color::srgb(0.85, 0.6, 0.15),
    ];
    let bottle_mats: Vec<_> = glass_cols
        .iter()
        .map(|c| {
            k.mats.add(StandardMaterial {
                base_color: *c,
                perceptual_roughness: 0.05,
                reflectance: 0.9,
                ..default()
            })
        })
        .collect();
    for (row, y) in [1.4f32, 1.8].into_iter().enumerate() {
        let mut z = z0 + 0.2;
        let mut i = row * 3;
        while z < z1 - 0.15 {
            let m = bottle_mats[i % bottle_mats.len()].clone();
            let h = 0.24 + (i % 3) as f32 * 0.04;
            k.cyl(bar, Vec3::new(6.82, y + 0.015 + h / 2.0, z), 0.035, h, &m);
            k.cyl(bar, Vec3::new(6.82, y + h + 0.05, z), 0.012, 0.08, &m);
            z += 0.17;
            i += 1;
        }
    }
    k.c.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.6, 0.3),
            intensity: 45_000.0,
            range: 4.0,
            ..default()
        },
        Transform::from_xyz(6.4, 1.25, zm),
        ChildOf(bar),
    ));

    // Wall phone, part of the same hotspot.
    let red = k.mat(Color::srgb(0.7, 0.05, 0.05), 0.3, 0.0);
    let chrome = k.mat(Color::srgb(0.9, 0.9, 0.92), 0.15, 1.0);
    let phone = k.group(
        Some(bar),
        Transform::from_translation(PHONE_POS)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    );
    k.cube(
        phone,
        Vec3::new(0.0, 0.0, 0.04),
        Vec3::new(0.2, 0.32, 0.08),
        &red,
    );
    k.cube(
        phone,
        Vec3::new(0.0, 0.03, 0.085),
        Vec3::new(0.12, 0.12, 0.01),
        &chrome,
    );
    let cap = k.meshes.add(Capsule3d::new(0.025, 0.2));
    k.mesh(phone, cap, Transform::from_xyz(0.13, 0.0, 0.07), &red);
    for i in 0..6 {
        let y = 0.08 - i as f32 * 0.025;
        k.ball(
            phone,
            Vec3::new(0.155 + (i as f32 * 1.7).sin() * 0.01, y - 0.12, 0.07),
            0.006,
            &chrome,
        );
    }
}

fn lounge(k: &mut Kit, root: Entity) {
    // Chesterfield-ish couch under the neon sign, low marble table in front.
    let leather = k.surf(Surf::Leather, Color::srgb(0.62, 0.2, 0.14), 2.5);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    let couch = k.group(Some(root), Transform::from_xyz(-2.3, 0.0, -4.4));
    k.bevel(
        couch,
        Vec3::new(0.0, 0.3, 0.0),
        Vec3::new(2.6, 0.32, 0.9),
        0.05,
        &leather,
    );
    k.bevel(
        couch,
        Vec3::new(0.0, 0.72, -0.35),
        Vec3::new(2.6, 0.62, 0.22),
        0.08,
        &leather,
    );
    for sx in [-1.0, 1.0] {
        k.bevel(
            couch,
            Vec3::new(sx * 1.2, 0.58, 0.0),
            Vec3::new(0.24, 0.5, 0.92),
            0.1,
            &leather,
        );
    }
    for sx in [-0.88, 0.0, 0.88] {
        k.bevel(
            couch,
            Vec3::new(sx, 0.5, 0.06),
            Vec3::new(0.86, 0.14, 0.72),
            0.06,
            &leather,
        );
        k.bevel(
            couch,
            Vec3::new(sx, 0.78, -0.21),
            Vec3::new(0.82, 0.38, 0.1),
            0.05,
            &leather,
        );
    }
    for (sx, sz) in [(-1.2, -0.38), (1.2, -0.38), (-1.2, 0.38), (1.2, 0.38)] {
        k.cyl(couch, Vec3::new(sx, 0.07, sz), 0.03, 0.14, &brass);
    }
    k.shadow(couch, Vec3::new(0.0, 0.0, 0.05), Vec2::new(3.1, 1.4));

    let marble = k.mat(Color::srgb(0.13, 0.13, 0.14), 0.12, 0.0);
    k.bevel(
        root,
        Vec3::new(-2.3, 0.38, -3.3),
        Vec3::new(1.3, 0.04, 0.6),
        0.015,
        &marble,
    );
    for (sx, sz) in [(-0.6, -0.25), (0.6, -0.25), (-0.6, 0.25), (0.6, 0.25)] {
        k.cyl(
            root,
            Vec3::new(-2.3 + sx, 0.18, -3.3 + sz),
            0.015,
            0.36,
            &brass,
        );
    }
    k.shadow(root, Vec3::new(-2.3, 0.0, -3.3), Vec2::new(1.6, 0.9));
}

fn exit_door(k: &mut Kit, root: Entity) {
    let door = k.group(
        Some(root),
        Transform::from_translation(DOOR_POS)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    );
    k.c.entity(door).insert(Interactive::new(
        "Exit — back to the site",
        Action::Hotspot(Hotspot::Exit),
    ));
    let frame = k.mat(Color::srgb(0.05, 0.05, 0.055), 0.4, 0.3);
    let panel = k.mat(Color::srgb(0.12, 0.05, 0.03), 0.45, 0.0);
    let brass = k.mat(Color::srgb(0.85, 0.62, 0.3), 0.3, 1.0);
    k.cube(
        door,
        Vec3::new(0.0, 1.1, 0.02),
        Vec3::new(1.2, 2.25, 0.05),
        &frame,
    );
    k.cube(
        door,
        Vec3::new(0.0, 1.05, 0.05),
        Vec3::new(1.0, 2.1, 0.04),
        &panel,
    );
    k.cube(
        door,
        Vec3::new(0.0, 1.05, 0.075),
        Vec3::new(0.8, 0.02, 0.02),
        &brass,
    );
    k.ball(door, Vec3::new(0.38, 1.0, 0.09), 0.03, &brass);
    neon::sign(
        k,
        door,
        "EXIT",
        Transform::from_xyz(0.0, 2.45, 0.05),
        0.05,
        NeonColor::Red,
        Some(Vec3::new(0.0, 2.4, 0.5)),
    );
}

// ---------------------------------------------------------------------------
// Ambient animation
// ---------------------------------------------------------------------------

fn animate_tv(time: Res<Time>, tv: Query<&TvScreen>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let t = time.elapsed_secs();
    for TvScreen(h) in &tv {
        if let Some(mut m) = mats.get_mut(h) {
            // The tube breathes and occasionally rolls a brighter frame.
            let flick = 0.85
                + 0.08 * (t * 7.3).sin() * (t * 2.1).cos()
                + 0.15 * ((t * 0.7).sin().max(0.0)).powi(8);
            m.emissive = LinearRgba::WHITE * flick;
        }
    }
}

/// Music playing → bands chase quickly; otherwise a slow breath.
#[derive(Resource, Default)]
pub struct MusicPlaying(pub bool);

fn animate_jukebox(
    time: Res<Time>,
    music: Option<Res<MusicPlaying>>,
    bands: Query<&JukeboxBand>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let playing = music.is_some_and(|m| m.0);
    let t = time.elapsed_secs() * if playing { 2.4 } else { 0.45 };
    for b in &bands {
        if let Some(mut m) = mats.get_mut(&b.mat) {
            let pulse = 0.55 + 0.45 * (t + b.phase).sin().abs();
            let base = m.emissive;
            let len = (base.red.max(base.green).max(base.blue)).max(1e-3);
            let strength = if playing { 5.0 } else { 2.6 } * pulse;
            m.emissive =
                LinearRgba::rgb(base.red / len, base.green / len, base.blue / len) * strength;
        }
    }
}
