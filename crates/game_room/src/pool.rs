//! Playable pool. The simulation is `pool_physics` (pure Rust, unit tested);
//! this module renders it, turns pointer drags into shots, and reports score.
//!
//! Aiming: press anywhere on the table and drag back. The shot goes from the
//! pointer through the cue ball, power grows with drag distance, release fires.

use std::f32::consts::FRAC_PI_2;

use bevy::asset::RenderAssetUsages;
use bevy::input::touch::Touches;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bridge::{Command, Hotspot, Mode, RoomEvent};
use pool_physics::{BALL_RADIUS, BallKind, GameEvent, PoolGame, Vec2 as PVec2};

use crate::audio::Sfx;
use crate::camera::{CurrentPose, Gesture, Rig, RoomCamera, pool_pose};
use crate::link::emit;
use crate::scene::{Chalkboard, LampShade, TABLE_CENTER};
use crate::textures::Textures;
use crate::{Cmd, RoomMode, View};

const R: f32 = BALL_RADIUS;
/// Drag distance (m on the table) for a full-power shot.
const FULL_DRAG: f32 = 0.45;

#[derive(Resource)]
pub struct Pool {
    pub game: PoolGame,
    aim: Option<Aim>,
    strike: Option<Strike>,
    last_dir: Vec2,
    drop: [f32; 16],
    reported_power: f32,
    tallies_drawn: u32,
}

struct Aim {
    start: Vec2,
    dir: Vec2,
    power: f32,
}

struct Strike {
    t: f32,
    dir: Vec2,
    power: f32,
}

#[derive(Component)]
struct PoolBall(u8);

#[derive(Component)]
struct CueStick;

#[derive(Resource)]
struct TallyKit {
    mesh: Handle<Mesh>,
}

pub struct PoolPlugin;

impl Plugin for PoolPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pool {
            game: PoolGame::new(),
            aim: None,
            strike: None,
            last_dir: Vec2::X,
            drop: [0.0; 16],
            reported_power: 0.0,
            tallies_drawn: 0,
        })
        .add_systems(Startup, spawn_balls)
        .add_systems(OnEnter(View::Pool), enter)
        .add_systems(OnExit(View::Pool), exit)
        .add_systems(
            Update,
            (commands, simulate, sync_balls, draw_cue, draw_tallies)
                .chain()
                .after(crate::camera::CameraSet),
        )
        // Aiming decides first whether a press belongs to the cue or the camera.
        .add_systems(
            Update,
            aim.run_if(in_state(View::Pool))
                .before(crate::camera::CameraSet),
        );
    }
}

fn to_world(p: PVec2) -> Vec3 {
    TABLE_CENTER + Vec3::new(p.x, R, p.y)
}

fn pv(v: Vec2) -> PVec2 {
    PVec2::new(v.x, v.y)
}

fn vv(v: PVec2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

fn ball_color(id: u8) -> Color {
    match if id > 8 { id - 8 } else { id } {
        1 => Color::srgb(0.98, 0.76, 0.05),
        2 => Color::srgb(0.05, 0.18, 0.72),
        3 => Color::srgb(0.85, 0.08, 0.06),
        4 => Color::srgb(0.33, 0.09, 0.52),
        5 => Color::srgb(0.98, 0.42, 0.05),
        6 => Color::srgb(0.04, 0.42, 0.18),
        7 => Color::srgb(0.45, 0.06, 0.06),
        _ => Color::srgb(0.02, 0.02, 0.02),
    }
}

/// A sphere whose vertex colours paint a band round the equator.
fn stripe_mesh(band: Color) -> Mesh {
    let mut mesh = Sphere::new(R).mesh().uv(32, 18);
    let band = band.to_linear().to_f32_array();
    let white = [0.95, 0.93, 0.88, 1.0];
    let colors: Vec<[f32; 4]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(ps)) => ps
            .iter()
            .map(|p| if p[1].abs() < R * 0.5 { band } else { white })
            .collect(),
        _ => Vec::new(),
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
    mesh
}

/// A UV sphere whose UVs point at ball `id`'s cell in the 4×4 atlas.
fn atlas_ball(id: u8) -> Mesh {
    let mut mesh = Sphere::new(R).mesh().uv(32, 18);
    let (col, row) = ((id % 4) as f32, (id / 4) as f32);
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs.iter_mut() {
            *uv = [(col + uv[0]) / 4.0, (row + uv[1]) / 4.0];
        }
    }
    mesh
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    pool: Res<Pool>,
    textures: Res<Textures>,
    assets: Res<AssetServer>,
) {
    // Numbered balls from the baked atlas: one glossy clearcoat material for
    // all sixteen. Without compressed-texture support, plain colours instead.
    let atlas = textures.ball_atlas(&assets).map(|tex| {
        mats.add(StandardMaterial {
            base_color_texture: Some(tex),
            perceptual_roughness: 0.18,
            reflectance: 0.6,
            clearcoat: 1.0,
            clearcoat_perceptual_roughness: 0.05,
            ..default()
        })
    });
    let solid = meshes.add(Sphere::new(R).mesh().uv(32, 18));
    let shiny = |c: Color| StandardMaterial {
        base_color: c,
        perceptual_roughness: 0.12,
        reflectance: 0.7,
        clearcoat: 1.0,
        ..default()
    };
    let white = mats.add(shiny(Color::WHITE));
    for b in &pool.game.balls {
        let (mesh, mat) = match (&atlas, b.kind()) {
            (Some(m), _) => (meshes.add(atlas_ball(b.id)), m.clone()),
            (None, kind) => match kind {
                BallKind::Cue => (solid.clone(), mats.add(shiny(Color::srgb(0.96, 0.95, 0.9)))),
                BallKind::Stripe => (meshes.add(stripe_mesh(ball_color(b.id))), white.clone()),
                _ => (solid.clone(), mats.add(shiny(ball_color(b.id)))),
            },
        };
        commands.spawn((
            PoolBall(b.id),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(to_world(b.pos)),
        ));
    }

    // Cue: tapered shaft with a pale tip.
    let shaft = meshes.add(ConicalFrustum {
        radius_top: 0.0065,
        radius_bottom: 0.015,
        height: 1.45,
    });
    let wood = mats.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.55, 0.32),
        perceptual_roughness: 0.3,
        clearcoat: 0.6,
        ..default()
    });
    let butt = mats.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.03, 0.02),
        perceptual_roughness: 0.3,
        ..default()
    });
    let tip = mats.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.45, 0.8),
        perceptual_roughness: 0.8,
        ..default()
    });
    let cyl = meshes.add(Cylinder::new(1.0, 1.0));
    commands
        .spawn((CueStick, Transform::default(), Visibility::Hidden))
        .with_children(|c| {
            c.spawn((
                Mesh3d(shaft),
                MeshMaterial3d(wood),
                Transform::from_xyz(0.0, -0.725, 0.0),
            ));
            c.spawn((
                Mesh3d(cyl.clone()),
                MeshMaterial3d(butt),
                Transform::from_xyz(0.0, -1.2, 0.0).with_scale(Vec3::new(0.0148, 0.5, 0.0148)),
            ));
            c.spawn((
                Mesh3d(cyl.clone()),
                MeshMaterial3d(tip),
                Transform::from_xyz(0.0, -0.004, 0.0).with_scale(Vec3::new(0.0066, 0.008, 0.0066)),
            ));
        });

    commands.insert_resource(TallyKit { mesh: cyl });
}

fn report(pool: &Pool) {
    emit(RoomEvent::PoolScore {
        potted: pool.game.potted,
        shots: pool.game.shots,
    });
}

fn enter(
    mut rig: ResMut<Rig>,
    cur: Res<CurrentPose>,
    pool: Res<Pool>,
    windows: Query<&Window>,
    mut lamp: Query<&mut Visibility, With<LampShade>>,
) {
    let aspect = windows
        .single()
        .map(|w| w.width() / w.height().max(1.0))
        .unwrap_or(1.6);
    rig.fly(cur.0, pool_pose(aspect), Some(Hotspot::PoolTable), None);
    for mut v in &mut lamp {
        *v = Visibility::Hidden;
    }
    emit(RoomEvent::PoolActive(true));
    report(&pool);
}

fn exit(mut pool: ResMut<Pool>, mut lamp: Query<&mut Visibility, With<LampShade>>) {
    for mut v in &mut lamp {
        *v = Visibility::Inherited;
    }
    pool.aim = None;
    pool.strike = None;
    emit(RoomEvent::PoolActive(false));
    emit(RoomEvent::PoolPower(0.0));
}

fn rack(pool: &mut Pool) {
    pool.game.rack();
    pool.aim = None;
    pool.strike = None;
    pool.drop = [0.0; 16];
    report(pool);
}

fn commands(
    mut cmds: MessageReader<Cmd>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<View>>,
    mut pool: ResMut<Pool>,
) {
    for Cmd(c) in cmds.read() {
        if *c == Command::PoolReset {
            rack(&mut pool);
        }
    }
    // Shift+R re-racks (plain R resets the camera everywhere).
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if *view.get() == View::Pool && shift && keys.just_pressed(KeyCode::KeyR) {
        rack(&mut pool);
    }
}

/// Where the pointer ray meets the plane through the ball centres, in table
/// coordinates (x along the length, y across).
fn pointer_on_table(
    windows: &Query<&Window>,
    touches: &Touches,
    cam: &Query<(&Camera, &GlobalTransform), With<RoomCamera>>,
) -> Option<Vec2> {
    let screen = touches
        .iter()
        .next()
        .map(|t| t.position())
        .or_else(|| windows.single().ok()?.cursor_position())?;
    let (camera, gt) = cam.single().ok()?;
    let ray = camera.viewport_to_world(gt, screen).ok()?;
    let plane_y = TABLE_CENTER.y + R;
    let d = ray.direction.y;
    if d.abs() < 1e-4 {
        return None;
    }
    let t = (plane_y - ray.origin.y) / d;
    (t > 0.0).then(|| {
        let p = ray.origin + *ray.direction * t;
        Vec2::new(p.x - TABLE_CENTER.x, p.z - TABLE_CENTER.z)
    })
}

#[allow(clippy::too_many_arguments)]
/// How close (screen pixels) a press must be to the cue ball to aim; further
/// away, the press orbits the camera instead.
const GRAB_PX: f32 = 70.0;
const GRAB_PX_TOUCH: f32 = 95.0;

#[allow(clippy::too_many_arguments)]
fn aim(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    cam: Query<(&Camera, &GlobalTransform), With<RoomCamera>>,
    rig: Res<Rig>,
    mode: Res<RoomMode>,
    mut pool: ResMut<Pool>,
    mut gesture: ResMut<Gesture>,
) {
    let held = mouse.pressed(MouseButton::Left) || touches.iter().next().is_some();
    if !held {
        gesture.captured = false;
    }
    if mode.0 != Mode::Full || rig.moving || pool.strike.is_some() {
        return;
    }
    let pressed = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    let Some(p) = pointer_on_table(&windows, &touches, &cam) else {
        return;
    };
    let cue = vv(pool.game.balls[0].pos);

    // Only a press on (or right by) the cue ball aims; anywhere else, the
    // camera gets the drag. Shift+drag always pans.
    let on_cue = || {
        let screen = touches
            .iter()
            .next()
            .map(|t| t.position())
            .or_else(|| windows.single().ok()?.cursor_position())?;
        let (camera, gt) = cam.single().ok()?;
        let ball = camera
            .world_to_viewport(gt, to_world(pool.game.balls[0].pos))
            .ok()?;
        let reach = if touches.iter().next().is_some() {
            GRAB_PX_TOUCH
        } else {
            GRAB_PX
        };
        Some(ball.distance(screen) <= reach)
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if pressed && pool.game.can_shoot() && !shift && on_cue() == Some(true) {
        gesture.captured = true;
        pool.aim = Some(Aim {
            start: p,
            dir: pool.last_dir,
            power: 0.0,
        });
    }
    if held {
        if let Some(a) = pool.aim.as_mut() {
            let back = cue - p;
            if back.length() > 0.02 {
                a.dir = back.normalize();
            }
            a.power = (p.distance(a.start) / FULL_DRAG).clamp(0.0, 1.0);
            let d = a.dir;
            let pw = a.power;
            pool.last_dir = d;
            if (pw - pool.reported_power).abs() > 0.02 {
                pool.reported_power = pw;
                emit(RoomEvent::PoolPower(pw));
            }
        }
    } else if let Some(a) = pool.aim.take() {
        if a.power > 0.03 {
            pool.strike = Some(Strike {
                t: 0.0,
                dir: a.dir,
                power: a.power,
            });
        }
        pool.reported_power = 0.0;
        emit(RoomEvent::PoolPower(0.0));
    }
}

fn simulate(time: Res<Time>, mut pool: ResMut<Pool>, mut sfx: MessageWriter<Sfx>) {
    let dt = time.delta_secs();

    // The cue's forward stroke, then the actual shot.
    if let Some(s) = pool.strike.as_mut() {
        s.t += dt / 0.09;
        if s.t >= 1.0 {
            let (dir, power) = (s.dir, s.power);
            pool.strike = None;
            pool.game.shoot(pv(dir), power);
        }
    }

    let mut score_changed = false;
    for ev in pool.game.step(dt) {
        match ev {
            GameEvent::BallHit { speed, .. } => {
                sfx.write(Sfx::Clack(speed));
            }
            GameEvent::Cushion { speed, .. } => {
                sfx.write(Sfx::Cushion(speed));
            }
            GameEvent::Cue { speed } => {
                sfx.write(Sfx::Cue(speed));
                score_changed = true;
            }
            GameEvent::Pocketed { ball } => {
                pool.drop[ball as usize] = 0.001;
                sfx.write(Sfx::Pocket);
                score_changed = true;
            }
            GameEvent::Scratch => {}
            GameEvent::Settled => score_changed = true,
        }
    }
    if score_changed {
        report(&pool);
    }
}

fn sync_balls(
    time: Res<Time>,
    mut pool: ResMut<Pool>,
    mut balls: Query<(&PoolBall, &mut Transform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    for (PoolBall(id), mut tf, mut vis) in &mut balls {
        let i = *id as usize;
        let b = &pool.game.balls[i];
        let target = to_world(b.pos);
        if b.pocketed {
            // Sink into the pocket, then disappear.
            let t = pool.drop[i];
            if t > 0.0 && t < 1.0 {
                tf.translation.y = TABLE_CENTER.y + R - t * 0.12;
                pool.drop[i] = (t + dt / 0.35).min(1.0);
            } else {
                *vis = Visibility::Hidden;
            }
            continue;
        }
        pool.drop[i] = 0.0;
        *vis = Visibility::Inherited;
        // Roll: rotate about the axis perpendicular to travel.
        let delta = target - tf.translation;
        let flat = Vec3::new(delta.x, 0.0, delta.z);
        let dist = flat.length();
        if dist > 1e-6 && dist < 0.5 {
            let axis = Vec3::Y.cross(flat / dist);
            tf.rotation = (Quat::from_axis_angle(axis, dist / R) * tf.rotation).normalize();
        }
        tf.translation = target;
    }
}

fn draw_cue(
    pool: Res<Pool>,
    view: Res<State<View>>,
    rig: Res<Rig>,
    mut cue: Query<(&mut Transform, &mut Visibility), With<CueStick>>,
    mut gizmos: Gizmos,
) {
    let Ok((mut tf, mut vis)) = cue.single_mut() else {
        return;
    };
    let show = *view.get() == View::Pool
        && !rig.moving
        && (pool.game.can_shoot() || pool.strike.is_some());
    if !show {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Inherited;

    let cue_ball = to_world(pool.game.balls[0].pos);
    let (dir, pull) = match (&pool.strike, &pool.aim) {
        (Some(s), _) => (s.dir, s.power * 0.25 * (1.0 - s.t)),
        (None, Some(a)) => (a.dir, a.power * 0.25),
        _ => (pool.last_dir, 0.0),
    };
    let d3 = Vec3::new(dir.x, 0.0, dir.y);
    // Butt raised a few degrees, as when cueing over the rail.
    let axis = (d3 - Vec3::Y * 0.1).normalize();
    let tip = cue_ball - axis * (R + 0.012 + pull);
    *tf = Transform::from_translation(tip).with_rotation(Quat::from_rotation_arc(Vec3::Y, axis));

    if let Some(a) = &pool.aim {
        let pred = pool.game.predict(pv(a.dir));
        let ghost = to_world(pred.ghost);
        let line = Color::srgba(1.0, 0.95, 0.85, 0.9);
        gizmos.line(cue_ball, ghost, line);
        let flat = Isometry3d::new(ghost, Quat::from_rotation_x(FRAC_PI_2));
        gizmos.circle(flat, R, Color::srgba(1.0, 1.0, 1.0, 0.8));
        if let (Some(t), Some(td)) = (pred.target, pred.target_dir) {
            let tp = to_world(pool.game.balls[t as usize].pos);
            gizmos.line(
                tp,
                tp + Vec3::new(td.x, 0.0, td.y) * 0.35,
                Color::srgb(0.13, 0.9, 1.0),
            );
        }
        // Power meter: a bar behind the cue ball that fills toward hot pink.
        let back = cue_ball - d3 * 0.08;
        let side = Vec3::new(-d3.z, 0.0, d3.x) * 0.06;
        let len = 0.3;
        gizmos.line(
            back + side,
            back + side - d3 * len,
            Color::srgba(1.0, 1.0, 1.0, 0.25),
        );
        let hot = Color::srgb(1.0, 0.95 - a.power * 0.8, 0.8 - a.power * 0.3);
        gizmos.line(back + side, back + side - d3 * len * a.power, hot);
    }
}

fn draw_tallies(
    mut commands: Commands,
    mut pool: ResMut<Pool>,
    board: Option<Res<Chalkboard>>,
    kit: Option<Res<TallyKit>>,
    children: Query<&Children>,
) {
    let (Some(board), Some(kit)) = (board, kit) else {
        return;
    };
    let n = pool.game.potted;
    if n == pool.tallies_drawn {
        return;
    }
    pool.tallies_drawn = n;
    if let Ok(kids) = children.get(board.root) {
        for k in kids.iter() {
            commands.entity(k).despawn();
        }
    }
    // Gates of five: four uprights and a slash.
    let stroke = |c: &mut Commands, a: Vec3, b: Vec3| {
        let d = b - a;
        c.spawn((
            Mesh3d(kit.mesh.clone()),
            MeshMaterial3d(board.chalk.clone()),
            Transform::from_translation((a + b) / 2.0)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, d.normalize()))
                .with_scale(Vec3::new(0.005, d.length(), 0.005)),
            ChildOf(board.root),
        ));
    };
    for i in 0..n {
        let gate = i / 5;
        let (gx, gy) = ((gate % 3) as f32 * 0.24, -((gate / 3) as f32) * 0.2);
        let j = i % 5;
        if j < 4 {
            let x = gx + j as f32 * 0.04;
            stroke(
                &mut commands,
                Vec3::new(x, gy - 0.07, 0.0),
                Vec3::new(x + 0.008, gy + 0.07, 0.0),
            );
        } else {
            stroke(
                &mut commands,
                Vec3::new(gx - 0.02, gy - 0.05, 0.0),
                Vec3::new(gx + 0.15, gy + 0.05, 0.0),
            );
        }
    }
}
