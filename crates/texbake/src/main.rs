//! Bake the game room's textures.
//!
//! Reads `art/textures/<set>/{Diffuse,nor_gl,Rough}.jpg` (fetched by
//! `scripts/fetch-textures.sh`) and `art/balls.png`, and writes
//! `assets/room/tex/<set>.<map>.<codec>.ktx2`:
//!
//! * map:   `albedo` (sRGB), `normal` (tangent space, OpenGL Y+), `orm`
//!   (glTF packing: G = roughness, B = metallic)
//! * codec: `bc` for desktop GPUs (BC1 colour, BC5 normals) and `etc` for
//!   mobile (ETC2 RGB, which every WebGL2 phone supports)
//!
//! Every file carries a full mip chain (no shimmer when minified) and is
//! zstd-supercompressed. The room picks the codec the GPU supports.
//!
//! Run with `just bake-textures`.

use std::path::{Path, PathBuf};

use image::{RgbaImage, imageops::FilterType};
use intel_tex_2::{RgSurface, RgbaSurface, bc1, bc5, etc1};

struct Set {
    name: &'static str,
    src: &'static str,
    size: u32,
    normal_size: u32,
    /// Strip colour so the material's base colour tints it (felt).
    grey: bool,
}

const SETS: &[Set] = &[
    Set {
        name: "floor",
        src: "dark_wooden_planks",
        size: 1024,
        normal_size: 512,
        grey: false,
    },
    Set {
        name: "felt",
        src: "velour_velvet",
        size: 1024,
        normal_size: 512,
        grey: true,
    },
    Set {
        name: "brick",
        src: "red_brick_03",
        size: 512,
        normal_size: 512,
        grey: false,
    },
    Set {
        name: "plaster",
        src: "painted_plaster_wall",
        size: 512,
        normal_size: 256,
        grey: false,
    },
    Set {
        name: "wood",
        src: "dark_wood",
        size: 512,
        normal_size: 256,
        grey: false,
    },
    Set {
        name: "leather",
        src: "brown_leather",
        size: 512,
        normal_size: 256,
        grey: false,
    },
];

#[derive(Clone, Copy)]
enum Kind {
    Albedo,
    Normal,
    Orm,
}

#[derive(Clone, Copy)]
enum Codec {
    Bc,
    Etc,
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = root.join("assets/room/tex");
    std::fs::create_dir_all(&out).expect("create assets/room/tex");
    let mut total = 0u64;

    for set in SETS {
        let dir = root.join("art/textures").join(set.src);
        let albedo = load(&dir.join("Diffuse.jpg"), set.size);
        let albedo = if set.grey { greyscale(albedo) } else { albedo };
        let normal = load(&dir.join("nor_gl.jpg"), set.normal_size);
        let rough = load(&dir.join("Rough.jpg"), set.size / 2);
        // glTF ORM: R occlusion (unused, 255), G roughness, B metallic (0).
        let orm = RgbaImage::from_fn(rough.width(), rough.height(), |x, y| {
            let r = rough.get_pixel(x, y)[0];
            image::Rgba([255, r, 0, 255])
        });
        for (kind, img) in [
            (Kind::Albedo, &albedo),
            (Kind::Normal, &normal),
            (Kind::Orm, &orm),
        ] {
            for codec in [Codec::Bc, Codec::Etc] {
                // Phones (ETC2) skip normal maps entirely; don't ship them.
                if matches!((kind, codec), (Kind::Normal, Codec::Etc)) {
                    continue;
                }
                total += write(&out, set.name, kind, codec, img);
            }
        }
    }

    let balls = root.join("art/balls.png");
    if balls.exists() {
        let img = load(&balls, 0);
        for codec in [Codec::Bc, Codec::Etc] {
            total += write(&out, "balls", Kind::Albedo, codec, &img);
        }
    } else {
        eprintln!("! art/balls.png missing: run scripts/make-ball-atlas.sh first");
    }
    println!(
        "baked {:.2} MB into {}",
        total as f64 / 1048576.0,
        out.display()
    );
}

/// Load and resize to `size`×`size` (0 keeps the original size).
fn load(path: &Path, size: u32) -> RgbaImage {
    let img = image::open(path)
        .unwrap_or_else(|e| panic!("{}: {e} (run scripts/fetch-textures.sh)", path.display()));
    let img = if size == 0 {
        img
    } else {
        img.resize_exact(size, size, FilterType::Lanczos3)
    };
    img.to_rgba8()
}

fn greyscale(mut img: RgbaImage) -> RgbaImage {
    // Re-centre around mid-grey so the tint colour reads true.
    let mean =
        img.pixels().map(|p| luma(p) as u64).sum::<u64>() / (img.width() * img.height()) as u64;
    for p in img.pixels_mut() {
        let v = (luma(p) as i32 - mean as i32 + 200).clamp(0, 255) as u8;
        *p = image::Rgba([v, v, v, 255]);
    }
    img
}

fn luma(p: &image::Rgba<u8>) -> u8 {
    ((p[0] as u32 * 54 + p[1] as u32 * 183 + p[2] as u32 * 19) >> 8) as u8
}

/// Halve repeatedly down to 4×4 (the block size). Normals are renormalised.
fn mips(base: &RgbaImage, normal: bool) -> Vec<RgbaImage> {
    let mut out = vec![base.clone()];
    while out.last().unwrap().width() > 4 {
        let prev = out.last().unwrap();
        let (w, h) = (prev.width() / 2, prev.height() / 2);
        let next = RgbaImage::from_fn(w, h, |x, y| {
            let mut acc = [0u32; 4];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let p = prev.get_pixel(x * 2 + dx, y * 2 + dy);
                for c in 0..4 {
                    acc[c] += p[c] as u32;
                }
            }
            let mut px = [0u8; 4];
            for c in 0..4 {
                px[c] = (acc[c] / 4) as u8;
            }
            if normal {
                let v = [
                    px[0] as f32 / 127.5 - 1.0,
                    px[1] as f32 / 127.5 - 1.0,
                    px[2] as f32 / 127.5 - 1.0,
                ];
                let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
                for c in 0..3 {
                    px[c] = ((v[c] / l + 1.0) * 127.5).round() as u8;
                }
            }
            image::Rgba(px)
        });
        out.push(next);
    }
    out
}

fn encode(img: &RgbaImage, kind: Kind, codec: Codec) -> Vec<u8> {
    let (w, h) = img.dimensions();
    let rgba = RgbaSurface {
        data: img.as_raw(),
        width: w,
        height: h,
        stride: w * 4,
    };
    match (codec, kind) {
        (Codec::Bc, Kind::Normal) => {
            let rg: Vec<u8> = img.pixels().flat_map(|p| [p[0], p[1]]).collect();
            bc5::compress_blocks(&RgSurface {
                data: &rg,
                width: w,
                height: h,
                stride: w * 2,
            })
        }
        (Codec::Bc, _) => bc1::compress_blocks(&rgba),
        // Thresholds below ~10 hit an abort in the ISPC kernel's AVX-512 ETC packer.
        (Codec::Etc, _) => etc1::compress_blocks(
            &etc1::EncodeSettings {
                fast_skip_threshold: 10,
            },
            &rgba,
        ),
    }
}

// KTX2 constants: Vulkan formats and Khronos data-format models.
const VK_BC1_RGBA_UNORM: u32 = 133;
const VK_BC1_RGBA_SRGB: u32 = 134;
const VK_BC5_UNORM: u32 = 141;
const VK_ETC2_RGB8_UNORM: u32 = 147;
const VK_ETC2_RGB8_SRGB: u32 = 148;
const MODEL_BC1A: u32 = 128;
const MODEL_BC5: u32 = 132;
const MODEL_ETC2: u32 = 161;

fn write(out: &Path, name: &str, kind: Kind, codec: Codec, img: &RgbaImage) -> u64 {
    let normal = matches!(kind, Kind::Normal);
    let srgb = matches!(kind, Kind::Albedo);
    let levels: Vec<Vec<u8>> = mips(img, normal)
        .iter()
        .map(|m| encode(m, kind, codec))
        .collect();

    let (vk, model, block_bytes, samples): (u32, u32, u32, u32) = match (codec, kind) {
        (Codec::Bc, Kind::Normal) => (VK_BC5_UNORM, MODEL_BC5, 16, 2),
        (Codec::Bc, _) => (
            if srgb {
                VK_BC1_RGBA_SRGB
            } else {
                VK_BC1_RGBA_UNORM
            },
            MODEL_BC1A,
            8,
            1,
        ),
        (Codec::Etc, _) => (
            if srgb {
                VK_ETC2_RGB8_SRGB
            } else {
                VK_ETC2_RGB8_UNORM
            },
            MODEL_ETC2,
            8,
            1,
        ),
    };
    let bytes = ktx2(
        vk,
        model,
        block_bytes,
        samples,
        srgb,
        img.width(),
        img.height(),
        &levels,
    );
    let map = match kind {
        Kind::Albedo => "albedo",
        Kind::Normal => "normal",
        Kind::Orm => "orm",
    };
    let ext = match codec {
        Codec::Bc => "bc",
        Codec::Etc => "etc",
    };
    let path = out.join(format!("{name}.{map}.{ext}.ktx2"));
    std::fs::write(&path, &bytes).expect("write ktx2");
    println!(
        "  {:<34} {:>4}×{:<4} {:>3} mips {:>7.1} KB",
        path.file_name().unwrap().to_string_lossy(),
        img.width(),
        img.height(),
        levels.len(),
        bytes.len() as f64 / 1024.0
    );
    bytes.len() as u64
}

/// A KTX2 file with zstd supercompression. Mip level 0 is the largest; level
/// data is stored smallest-first as the spec requires.
#[allow(clippy::too_many_arguments)]
fn ktx2(
    vk: u32,
    model: u32,
    block_bytes: u32,
    samples: u32,
    srgb: bool,
    w: u32,
    h: u32,
    levels: &[Vec<u8>],
) -> Vec<u8> {
    let compressed: Vec<Vec<u8>> = levels
        .iter()
        .map(|l| zstd::encode_all(&l[..], 19).expect("zstd"))
        .collect();

    // Data Format Descriptor: one basic block.
    let block_size = 24 + 16 * samples;
    let mut dfd = Vec::new();
    let push = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&x.to_le_bytes());
    push(&mut dfd, 4 + block_size); // dfdTotalSize
    push(&mut dfd, 0); // vendorId 0, descriptorType 0
    push(&mut dfd, 2 | (block_size << 16)); // version 2, block size
    push(
        &mut dfd,
        model | (1 << 8) | ((if srgb { 2 } else { 1 }) << 16),
    ); // model, BT709, transfer
    push(&mut dfd, 3 | (3 << 8)); // 4×4 blocks
    push(&mut dfd, block_bytes); // bytesPlane0
    push(&mut dfd, 0); // bytesPlane4..7
    for s in 0..samples {
        push(&mut dfd, (s * 64) | (63 << 16) | (s << 24)); // offset, length-1, channel
        push(&mut dfd, 0); // sample position
        push(&mut dfd, 0); // lower
        push(&mut dfd, u32::MAX); // upper
    }

    let header_len = 12 + 9 * 4 + 4 * 4 + 2 * 8;
    let index_len = 24 * levels.len();
    let dfd_off = header_len + index_len;
    let data_off = dfd_off + dfd.len();

    let mut f = vec![
        0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ];
    for v in [vk, 1, w, h, 0, 0, 1, levels.len() as u32, 2 /* zstd */] {
        push(&mut f, v);
    }
    for v in [dfd_off as u32, dfd.len() as u32, 0, 0] {
        push(&mut f, v);
    }
    f.extend_from_slice(&0u64.to_le_bytes());
    f.extend_from_slice(&0u64.to_le_bytes());

    // Offsets: smallest level first in the file.
    let mut offsets = vec![0usize; levels.len()];
    let mut at = data_off;
    for i in (0..levels.len()).rev() {
        offsets[i] = at;
        at += compressed[i].len();
    }
    for i in 0..levels.len() {
        f.extend_from_slice(&(offsets[i] as u64).to_le_bytes());
        f.extend_from_slice(&(compressed[i].len() as u64).to_le_bytes());
        f.extend_from_slice(&(levels[i].len() as u64).to_le_bytes());
    }
    f.extend_from_slice(&dfd);
    for i in (0..levels.len()).rev() {
        f.extend_from_slice(&compressed[i]);
    }
    f
}
