//! PBR surface textures, baked offline by `crates/texbake` into
//! `assets/room/tex/<set>.<map>.<codec>.ktx2`.
//!
//! The codec is chosen once from what the GPU can sample: BC on desktop, ETC2
//! on phones. Mobile skips normal maps (cheaper, and ETC1-encoded normals add
//! little). With neither codec, surfaces fall back to plain colours.

use std::collections::HashMap;

use bevy::image::{CompressedImageFormatSupport, CompressedImageFormats};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Surf {
    Floor,
    Felt,
    Brick,
    Plaster,
    Wood,
    Leather,
}

impl Surf {
    fn set(self) -> &'static str {
        match self {
            Surf::Floor => "floor",
            Surf::Felt => "felt",
            Surf::Brick => "brick",
            Surf::Plaster => "plaster",
            Surf::Wood => "wood",
            Surf::Leather => "leather",
        }
    }
}

struct Maps {
    albedo: Handle<Image>,
    normal: Option<Handle<Image>>,
    orm: Handle<Image>,
}

#[derive(Resource)]
pub struct Textures {
    codec: Option<&'static str>,
    normals: bool,
    maps: HashMap<Surf, Maps>,
}

/// Materials whose meshes get world-space box UVs when merged, with their
/// texture density (tiles per metre).
#[derive(Resource, Default)]
pub struct BoxUv(pub HashMap<AssetId<StandardMaterial>, f32>);

impl Textures {
    pub fn detect(support: Option<&CompressedImageFormatSupport>) -> Self {
        let f = support.map(|s| s.0).unwrap_or(CompressedImageFormats::NONE);
        let (codec, normals) = if f.contains(CompressedImageFormats::BC) {
            (Some("bc"), true)
        } else if f.contains(CompressedImageFormats::ETC2) {
            (Some("etc"), false)
        } else {
            (None, false)
        };
        info!("room textures: {}", codec.unwrap_or("none (plain colours)"));
        Self {
            codec,
            normals,
            maps: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn codec(&self) -> Option<&'static str> {
        self.codec
    }

    fn maps(&mut self, assets: &AssetServer, s: Surf) -> Option<&Maps> {
        let codec = self.codec?;
        let normals = self.normals;
        Some(self.maps.entry(s).or_insert_with(|| {
            let path = |map: &str| format!("room/tex/{}.{map}.{codec}.ktx2", s.set());
            Maps {
                albedo: assets.load(path("albedo")),
                normal: normals.then(|| assets.load(path("normal"))),
                orm: assets.load(path("orm")),
            }
        }))
    }

    /// A textured material tinted by `tint`. `density` is texture repeats
    /// per metre, applied through the world-space UVs the merge generates.
    #[allow(clippy::too_many_arguments)]
    pub fn material(
        &mut self,
        assets: &AssetServer,
        mats: &mut Assets<StandardMaterial>,
        box_uv: &mut BoxUv,
        s: Surf,
        tint: Color,
        density: f32,
        roughness: f32,
    ) -> Handle<StandardMaterial> {
        let mut m = StandardMaterial {
            base_color: tint,
            perceptual_roughness: roughness,
            ..default()
        };
        if let Some(maps) = self.maps(assets, s) {
            m.base_color_texture = Some(maps.albedo.clone());
            m.normal_map_texture = maps.normal.clone();
            m.metallic_roughness_texture = Some(maps.orm.clone());
            // Roughness comes from the map; this scales it.
            m.perceptual_roughness = 1.0;
        }
        let h = mats.add(m);
        box_uv.0.insert(h.id(), density);
        h
    }

    pub fn ball_atlas(&self, assets: &AssetServer) -> Option<Handle<Image>> {
        self.codec
            .map(|c| assets.load(format!("room/tex/balls.albedo.{c}.ktx2")))
    }
}

pub fn setup(mut commands: Commands, support: Option<Res<CompressedImageFormatSupport>>) {
    commands.insert_resource(Textures::detect(support.as_deref()));
    commands.init_resource::<BoxUv>();
}
