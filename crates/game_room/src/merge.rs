//! Static batching.
//!
//! The room is built from hundreds of small primitives, which is pleasant to
//! author but costs per-entity CPU every frame and a draw call per piece. Once
//! the scene exists, every static mesh is baked into one mesh per
//! (merge root, material). A merge root is the nearest ancestor that must stay
//! addressable: an interactive object (for picking and hover highlight), the
//! lamp shade (hidden while playing), a neon sign, or anything tagged
//! [`MergeRoot`]. Everything else folds into the room itself.
//!
//! Textured materials (see [`BoxUv`]) get world-space box-projected UVs here,
//! so wood and brick tile at a constant density instead of stretching with
//! each primitive's scale, plus tangents for their normal maps.
//!
//! Only the pool table casts shadows (it's the only thing under the shadowed
//! lamp); everything else is merged with [`NotShadowCaster`].

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::math::Affine3A;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use crate::interact::Interactive;
use crate::neon::Neon;
use crate::scene::LampShade;
use crate::textures::BoxUv;

/// Keep this subtree as its own merged group.
#[derive(Component)]
pub struct MergeRoot;

/// Leave this mesh (and its subtree) alone: it moves or is replaced at runtime.
#[derive(Component)]
pub struct NoMerge;

/// Merged meshes under this root cast shadows.
#[derive(Component)]
pub struct CastsShadows;

/// The room's top-level entity; everything merges under it by default.
#[derive(Component)]
pub struct RoomRoot;

/// What builders tell the merge about materials.
#[derive(Resource, Default)]
pub struct MergeHints {
    /// Static self-lit materials: folded into one unlit palette material per
    /// root, their emissive colour baked into vertex colours.
    pub glows: HashSet<AssetId<StandardMaterial>>,
    /// Merged into the room root whatever their parent (contact shadows).
    pub to_room: HashSet<AssetId<StandardMaterial>>,
    /// Must keep their own material (it's changed later, e.g. a texture
    /// that finishes loading after the merge).
    pub exact: HashSet<AssetId<StandardMaterial>>,
}

/// How pieces are grouped into batches within a root.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Key {
    /// Its own material (textured, animated, transparent…).
    Exact(AssetId<StandardMaterial>),
    /// Plain lit materials that differ only in colour share one material;
    /// the colour moves to the vertices. (roughness, metallic, clearcoat, reflectance) quantised.
    Lit(u8, u8, u8, u8),
    /// Static glows: one unlit material, HDR colour in the vertices.
    Glow,
    /// Textured surfaces sharing a texture set (albedo, normal, ORM ids);
    /// the tint moves to the vertices, density is already in the UVs.
    Tex(
        AssetId<Image>,
        Option<AssetId<Image>>,
        Option<AssetId<Image>>,
    ),
}

pub struct MergePlugin;

impl Plugin for MergePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MergeHints>()
            .add_systems(PostUpdate, merge_static.after(TransformSystems::Propagate));
    }
}

#[derive(Default)]
struct Builder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn merge_static(
    mut done: Local<bool>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    hints: Res<MergeHints>,
    box_uv: Option<Res<BoxUv>>,
    room: Query<Entity, With<RoomRoot>>,
    pieces: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &GlobalTransform,
        ),
        Without<NoMerge>,
    >,
    parents: Query<&ChildOf>,
    roots: Query<
        (),
        Or<(
            With<Interactive>,
            With<LampShade>,
            With<Neon>,
            With<MergeRoot>,
            With<RoomRoot>,
        )>,
    >,
    blocked: Query<(), With<NoMerge>>,
    globals: Query<&GlobalTransform>,
    shadows: Query<(), With<CastsShadows>>,
    labels: Query<&Interactive>,
) {
    if *done || room.is_empty() {
        return;
    }
    *done = true;
    let empty = BoxUv::default();
    let box_uv = box_uv.as_deref().unwrap_or(&empty);

    let room_e = room.single().expect("one room");
    // Group pieces by (merge root, batch key).
    let mut groups: HashMap<(Entity, Key), (Handle<StandardMaterial>, bool, Builder)> =
        HashMap::new();
    let mut palette: HashMap<Key, Handle<StandardMaterial>> = HashMap::new();
    let mut merged = 0;
    'piece: for (e, mesh3d, mat, gt) in &pieces {
        // Walk up to the nearest merge root; skip pieces outside the room or
        // under a NoMerge ancestor.
        let mut root = None;
        let mut casts = false;
        let mut cur = e;
        while let Ok(p) = parents.get(cur) {
            cur = p.parent();
            if blocked.contains(cur) {
                continue 'piece;
            }
            casts |= shadows.contains(cur);
            if root.is_none() && roots.contains(cur) {
                root = Some(cur);
            }
        }
        let (Some(mut root), true) = (root, room.contains(cur)) else {
            continue;
        };
        if hints.to_room.contains(&mat.0.id()) {
            root = room_e;
        }
        let Some(m) = mats.get(&mat.0) else {
            continue;
        };
        let (key, color) = classify(&mat.0, m, &hints, box_uv);
        let template = m.clone();
        let Some(mesh) = meshes.get(&mesh3d.0) else {
            continue;
        };
        let Ok(root_gt) = globals.get(root) else {
            continue;
        };

        let to_root = root_gt.affine().inverse() * gt.affine();
        let normal_m = Mat3::from(to_root.matrix3).inverse().transpose();
        let world_m = gt.affine();
        let density = box_uv.0.get(&mat.0.id()).copied();

        let handle = match key {
            Key::Exact(_) => mat.0.clone(),
            _ => palette
                .entry(key)
                .or_insert_with(|| {
                    let m = palette_material(key, &template);
                    mats.add(m)
                })
                .clone(),
        };
        let entry = groups
            .entry((root, key))
            .or_insert_with(|| (handle, casts, Builder::default()));
        let before = entry.2.positions.len();
        if !append(&mut entry.2, mesh, to_root, normal_m, world_m, density) {
            continue;
        }
        if let Some(c) = color {
            let added = entry.2.positions.len() - before;
            entry.2.colors.extend(std::iter::repeat_n(c, added));
        }
        commands.entity(e).despawn();
        merged += 1;
    }

    let groups_n = groups.len();
    if std::env::var_os("ARTHUB_MERGE_DEBUG").is_some() {
        let mut per: HashMap<Entity, usize> = HashMap::new();
        for (root, _) in groups.keys() {
            *per.entry(*root).or_default() += 1;
        }
        let mut v: Vec<_> = per.into_iter().collect();
        v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        for (root, n) in v {
            let label = labels
                .get(root)
                .map(|i| i.label.clone())
                .unwrap_or_else(|_| format!("{root:?}"));
            info!("  {n:3} batches under {label}");
        }
    }
    for ((root, _), (mat, casts, b)) in groups {
        let textured = mats
            .get(&mat)
            .is_some_and(|m| m.normal_map_texture.is_some());
        // Kept in the main world too: picking ray-casts against it.
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, b.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, b.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, b.uvs);
        if !b.colors.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, b.colors);
        }
        mesh.insert_indices(Indices::U32(b.indices));
        if textured {
            let _ = mesh.generate_tangents();
        }
        let mut e = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat),
            Transform::IDENTITY,
            ChildOf(root),
        ));
        if !casts {
            e.insert(NotShadowCaster);
        }
    }
    info!("static merge: {merged} meshes → {groups_n} batches");
}

fn q(v: f32, steps: f32) -> u8 {
    (v.clamp(0.0, 1.0) * steps).round() as u8
}

/// Decide how a piece batches, and (for palette keys) its vertex colour.
fn classify(
    h: &Handle<StandardMaterial>,
    m: &StandardMaterial,
    hints: &MergeHints,
    box_uv: &BoxUv,
) -> (Key, Option<[f32; 4]>) {
    let id = h.id();
    if hints.exact.contains(&id) {
        return (Key::Exact(id), None);
    }
    if hints.glows.contains(&id) {
        let e = m.emissive;
        return (Key::Glow, Some([e.red, e.green, e.blue, 1.0]));
    }
    let plain = m.base_color_texture.is_none()
        && m.emissive_texture.is_none()
        && m.normal_map_texture.is_none()
        && m.metallic_roughness_texture.is_none()
        && m.emissive.red + m.emissive.green + m.emissive.blue < 1e-4
        && matches!(m.alpha_mode, AlphaMode::Opaque)
        && !m.unlit
        && !box_uv.0.contains_key(&id);
    if plain {
        // Coarse buckets: nobody can tell roughness 0.3 from 0.35 in this light.
        let key = Key::Lit(
            q(m.perceptual_roughness, 4.0),
            q(m.metallic, 1.0),
            q(m.clearcoat, 1.0),
            q(m.reflectance, 2.0),
        );
        return (key, Some(m.base_color.to_linear().to_f32_array()));
    }
    let textured = box_uv.0.contains_key(&id)
        && m.emissive.red + m.emissive.green + m.emissive.blue < 1e-4
        && matches!(m.alpha_mode, AlphaMode::Opaque);
    if let (true, Some(albedo)) = (textured, &m.base_color_texture) {
        let key = Key::Tex(
            albedo.id(),
            m.normal_map_texture.as_ref().map(|h| h.id()),
            m.metallic_roughness_texture.as_ref().map(|h| h.id()),
        );
        return (key, Some(m.base_color.to_linear().to_f32_array()));
    }
    (Key::Exact(id), None)
}

fn palette_material(key: Key, template: &StandardMaterial) -> StandardMaterial {
    match key {
        Key::Lit(r, me, c, re) => StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: (r as f32 / 4.0).max(0.06),
            metallic: me as f32,
            clearcoat: c as f32,
            reflectance: re as f32 / 2.0,
            ..default()
        },
        Key::Tex(..) => StandardMaterial {
            base_color: Color::WHITE,
            ..template.clone()
        },
        _ => StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        },
    }
}

/// Append `mesh` (transformed into root space) to `b`. Returns false if the
/// mesh isn't an indexed/unindexed triangle list with positions and normals.
fn append(
    b: &mut Builder,
    mesh: &Mesh,
    to_root: Affine3A,
    normal_m: Mat3,
    world: Affine3A,
    density: Option<f32>,
) -> bool {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return false;
    }
    let (Some(VertexAttributeValues::Float32x3(pos)), Some(VertexAttributeValues::Float32x3(nor))) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
    ) else {
        return false;
    };
    let uv = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => Some(uv),
        _ => None,
    };
    let base = b.positions.len() as u32;
    let world_n = Mat3::from(world.matrix3).inverse().transpose();
    for i in 0..pos.len() {
        let p = Vec3::from(pos[i]);
        let n = Vec3::from(nor[i]);
        b.positions.push(to_root.transform_point3(p).to_array());
        b.normals
            .push((normal_m * n).normalize_or_zero().to_array());
        let tex = match density {
            Some(d) => box_project(
                world.transform_point3(p),
                (world_n * n).normalize_or_zero(),
                d,
            ),
            None => uv.map(|u| u[i]).unwrap_or([0.0, 0.0]),
        };
        b.uvs.push(tex);
    }
    match mesh.indices() {
        Some(Indices::U16(ix)) => b.indices.extend(ix.iter().map(|&i| base + i as u32)),
        Some(Indices::U32(ix)) => b.indices.extend(ix.iter().map(|&i| base + i)),
        None => b.indices.extend(base..base + pos.len() as u32),
    }
    true
}

/// Planar projection along the normal's dominant axis, in world metres.
fn box_project(p: Vec3, n: Vec3, density: f32) -> [f32; 2] {
    let a = n.abs();
    let uv = if a.y >= a.x && a.y >= a.z {
        Vec2::new(p.x, p.z)
    } else if a.x >= a.z {
        Vec2::new(p.z * -n.x.signum(), -p.y)
    } else {
        Vec2::new(p.x * n.z.signum(), -p.y)
    };
    (uv * density).to_array()
}

/// A box with rounded edges (radius `r`), `size` overall. Two rings of
/// vertices per edge give a soft bevel that catches highlights.
pub fn rounded_box(size: Vec3, r: f32) -> Mesh {
    let h = size / 2.0;
    let r = r.min(h.min_element() * 0.99);
    let inner = h - Vec3::splat(r);
    let coords = |half: f32| {
        let rr = r.min(half);
        vec![
            -half,
            -half + rr * 0.3,
            -half + rr,
            half - rr,
            half - rr * 0.3,
            half,
        ]
    };
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // Each face: (normal axis, sign, u axis, v axis).
    let faces = [
        (0, 1.0),
        (0, -1.0),
        (1, 1.0),
        (1, -1.0),
        (2, 1.0),
        (2, -1.0),
    ];
    for (axis, sign) in faces {
        let (ua, va) = ((axis + 1) % 3, (axis + 2) % 3);
        let us = coords(h[ua]);
        let vs = coords(h[va]);
        let base = positions.len() as u32;
        for (j, &v) in vs.iter().enumerate() {
            for (i, &u) in us.iter().enumerate() {
                let mut p = Vec3::ZERO;
                p[axis] = h[axis] * sign;
                p[ua] = u;
                p[va] = v;
                let c = p.clamp(-inner, inner);
                let mut face_n = Vec3::ZERO;
                face_n[axis] = sign;
                let n = (p - c).normalize_or(face_n);
                positions.push((c + n * r).to_array());
                normals.push(n.to_array());
                uvs.push([i as f32 / 5.0, j as f32 / 5.0]);
            }
        }
        let w = us.len() as u32;
        for j in 0..vs.len() as u32 - 1 {
            for i in 0..w - 1 {
                let a = base + j * w + i;
                let (b, c, d) = (a + 1, a + w, a + w + 1);
                // Wind counter-clockwise when seen from outside.
                if sign > 0.0 {
                    indices.extend([a, b, d, a, d, c]);
                } else {
                    indices.extend([a, d, b, a, c, d]);
                }
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
