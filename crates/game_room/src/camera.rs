//! Free 360° camera with hotspots.
//!
//! The camera orbits a focus point. Hotspot clicks (and the HUD) fly it there
//! with the eased transition and make that object the new focus, so you can
//! look all the way round it.
//!
//! | input                | does                                   |
//! |----------------------|----------------------------------------|
//! | left-drag / 1 finger | orbit (360° around, pitch clamped)     |
//! | right/middle/Shift-drag, 2 fingers | pan focus and camera together |
//! | wheel, trackpad or 2-finger pinch  | zoom toward / away from focus |
//! | arrows / WASD / + −  | orbit / pan / zoom                     |
//! | R                    | reset to the current hotspot's view    |
//! | double-click / tap   | refocus on that point                  |
//! | 1–9, `,` `.`, Esc    | jump / cycle hotspots, back to overview |
//!
//! Everything is damped with inertia. The focus stays inside the room and the
//! camera is pulled in rather than passing through walls, floor or ceiling.
//! A pointer that travels more than 5 px is a drag: it never clicks objects
//! and hover highlights pause while it lasts. After 8 s with no input, the
//! gentle idle drift blends back in.

use bevy::camera::Hdr;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::touch::Touches;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bridge::{Command, Hotspot, Mode, RoomEvent};

use crate::interact::{Action, Arrived, PendingClick};
use crate::link::emit;
use crate::scene::*;
use crate::{Cmd, Paused, RoomMode, View};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: Vec3,
    pub look: Vec3,
}

const fn pose(eye: [f32; 3], look: [f32; 3]) -> Pose {
    Pose {
        eye: Vec3::from_array(eye),
        look: Vec3::from_array(look),
    }
}

pub fn hotspot_pose(h: Hotspot) -> Pose {
    match h {
        Hotspot::Overview => pose([0.0, 3.0, 4.75], [0.0, 0.9, -1.4]),
        Hotspot::PoolTable => pose([0.0, 2.15, 2.75], [0.0, 0.75, 0.25]),
        Hotspot::ArcadeRow => pose([3.2, 1.75, -1.4], [3.2, 1.25, -4.6]),
        Hotspot::TrophyShelf => pose([-4.9, 1.85, -3.75], [-7.0, 1.7, -3.75]),
        Hotspot::Chalkboard => pose([-4.75, 1.7, 1.6], [-7.0, 1.65, 1.6]),
        Hotspot::Jukebox => pose([-4.35, 1.55, -2.5], [-5.5, 1.05, -4.45]),
        Hotspot::Tv => pose([-4.4, 1.75, -1.2], [-7.0, 1.78, -1.2]),
        Hotspot::Bar => pose([3.4, 1.85, 0.6], [6.4, 1.2, -1.6]),
        Hotspot::Exit => pose([4.7, 1.65, 2.9], [7.0, 1.4, 2.9]),
    }
}

/// Close-up of one arcade cabinet's screen.
pub fn cabinet_pose(i: usize) -> Pose {
    let x = cabinet_x(i);
    Pose {
        eye: Vec3::new(x, 1.5, CABINET_Z + 1.25),
        look: Vec3::new(x, 1.38, CABINET_Z),
    }
}

pub fn dartboard_pose() -> Pose {
    Pose {
        eye: DARTBOARD_POS + Vec3::new(2.0, 0.0, 0.0),
        look: DARTBOARD_POS,
    }
}

/// Above the table, angled just enough to read depth while aiming. On
/// portrait screens the camera looks down the table's length so it fits.
pub fn pool_pose(aspect: f32) -> Pose {
    if aspect < 1.0 {
        Pose {
            eye: TABLE_CENTER + Vec3::new(0.55, 2.05, 0.0),
            look: TABLE_CENTER + Vec3::new(0.06, 0.0, 0.0),
        }
    } else {
        Pose {
            eye: TABLE_CENTER + Vec3::new(0.0, 2.35, 1.25),
            look: TABLE_CENTER + Vec3::new(0.0, 0.0, 0.08),
        }
    }
}

// Limits.
const PITCH_MIN: f32 = -0.3;
const PITCH_MAX: f32 = 1.48;
const DIST_MIN: f32 = 0.45;
const DIST_MAX: f32 = 9.0;
/// Camera keeps this far inside the walls, floor and ceiling.
const WALL_MARGIN: f32 = 0.15;
/// The focus point stays inside this box.
const FOCUS_MIN: Vec3 = Vec3::new(-ROOM_HALF_X + 0.3, 0.05, -ROOM_HALF_Z + 0.3);
const FOCUS_MAX: Vec3 = Vec3::new(ROOM_HALF_X - 0.3, ROOM_HEIGHT - 0.3, ROOM_HALF_Z - 0.3);
/// Drag threshold before a press stops being a click.
pub const DRAG_PX: f32 = 5.0;
/// Seconds without input before the idle drift returns.
const IDLE_AFTER: f32 = 8.0;

/// Orbit parameters: focus point, yaw (around Y), pitch (above horizontal)
/// and distance.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Orbit {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    dist: f32,
}

impl Orbit {
    fn from_pose(p: Pose) -> Self {
        let d = p.eye - p.look;
        let dist = d.length().max(0.01);
        Self {
            focus: p.look,
            yaw: d.x.atan2(d.z),
            pitch: (d.y / dist).clamp(-1.0, 1.0).asin(),
            dist,
        }
    }

    fn dir(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    fn clamped(mut self) -> Self {
        self.pitch = self.pitch.clamp(PITCH_MIN, PITCH_MAX);
        self.dist = self.dist.clamp(DIST_MIN, DIST_MAX);
        self.focus = self.focus.clamp(FOCUS_MIN, FOCUS_MAX);
        self
    }

    /// The eye, pulled in toward the focus if a wall, the floor or the
    /// ceiling is in the way.
    fn pose(&self) -> Pose {
        let dir = self.dir();
        let lo = Vec3::new(-ROOM_HALF_X, 0.0, -ROOM_HALF_Z) + WALL_MARGIN;
        let hi = Vec3::new(ROOM_HALF_X, ROOM_HEIGHT, ROOM_HALF_Z) - WALL_MARGIN;
        let mut reach = self.dist;
        for a in 0..3 {
            if dir[a] > 1e-5 {
                reach = reach.min((hi[a] - self.focus[a]) / dir[a]);
            } else if dir[a] < -1e-5 {
                reach = reach.min((lo[a] - self.focus[a]) / dir[a]);
            }
        }
        Pose {
            eye: self.focus + dir * reach.max(0.05),
            look: self.focus,
        }
    }
}

struct Flight {
    from: Pose,
    to: Pose,
    t: f32,
    dur: f32,
}

#[derive(Resource)]
pub struct Rig {
    flight: Option<Flight>,
    /// Flying between views (input is ignored meanwhile).
    pub moving: bool,
    /// The hotspot we're at or flying to (`None` after free looking around
    /// an ad-hoc focus).
    pub hotspot: Option<Hotspot>,
    then: Option<Action>,
    /// Smoothed (what's rendered) and target orbit.
    cur: Orbit,
    target: Orbit,
    /// Inertia, per second: yaw, pitch, zoom (log-distance) and pan.
    v_rot: Vec2,
    v_zoom: f32,
    v_pan: Vec3,
    /// Seconds since the last user input.
    idle: f32,
    /// Where R / "Reset view" returns to.
    home: Pose,
}

impl Default for Rig {
    fn default() -> Self {
        let p = hotspot_pose(Hotspot::Overview);
        let o = Orbit::from_pose(p);
        Self {
            flight: None,
            moving: false,
            hotspot: Some(Hotspot::Overview),
            then: None,
            cur: o,
            target: o,
            v_rot: Vec2::ZERO,
            v_zoom: 0.0,
            v_pan: Vec3::ZERO,
            idle: IDLE_AFTER,
            home: p,
        }
    }
}

impl Rig {
    /// Fly from wherever the camera is now (`current`) to `to`, which then
    /// becomes the orbit focus and the "Reset view" pose.
    pub fn fly(&mut self, current: Pose, to: Pose, hotspot: Option<Hotspot>, then: Option<Action>) {
        let dist = current.eye.distance(to.eye) + current.look.distance(to.look) * 0.5;
        self.flight = Some(Flight {
            from: current,
            to,
            t: 0.0,
            dur: (0.9 + dist * 0.06).clamp(1.0, 1.6),
        });
        self.moving = true;
        self.hotspot = hotspot;
        self.then = then;
        self.home = to;
        self.stop();
    }

    pub fn snap(&mut self, to: Pose, hotspot: Option<Hotspot>) {
        self.flight = None;
        self.moving = false;
        self.hotspot = hotspot;
        self.then = None;
        self.home = to;
        self.cur = Orbit::from_pose(to);
        self.target = self.cur;
        self.stop();
    }

    fn stop(&mut self) {
        self.v_rot = Vec2::ZERO;
        self.v_zoom = 0.0;
        self.v_pan = Vec3::ZERO;
    }

    fn touched(&mut self) {
        self.idle = 0.0;
    }
}

fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// A pose requested from outside (`debug_pose()` on the room module):
/// snapped to on the next frame. For screenshots and audits.
pub static DEBUG_POSE: std::sync::Mutex<Option<Pose>> = std::sync::Mutex::new(None);

/// Last rendered pose `[eye.xyz, look.xyz]`, readable from JS for tests and
/// debugging (`camera_pose()` on the room module).
pub static LAST_POSE: std::sync::Mutex<[f32; 6]> = std::sync::Mutex::new([0.0; 6]);

/// The camera's current pose, for starting new flights.
#[derive(Resource, Clone, Copy)]
pub struct CurrentPose(pub Pose);

/// Pointer gesture state, shared with picking (clicks) and pool (aiming).
#[derive(Resource, Default)]
pub struct Gesture {
    /// A press is in progress and has moved past the drag threshold.
    pub dragging: bool,
    /// The last press turned into a drag: its release must not click.
    pub suppress_click: bool,
    /// The pool cue owns the pointer (dragging from the cue ball aims).
    pub captured: bool,
    press: Option<(Vec2, Grip)>,
    last: Option<Vec2>,
    pinch: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Grip {
    Orbit,
    Pan,
}

#[derive(Component)]
pub struct RoomCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rig>()
            .init_resource::<Gesture>()
            .insert_resource(CurrentPose(hotspot_pose(Hotspot::Overview)))
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (commands, keyboard, pointer, advance)
                    .chain()
                    .in_set(CameraSet),
            )
            .add_systems(
                PreUpdate,
                reset_on_press
                    .after(bevy::input::InputSystems)
                    .before(bevy::picking::PickingSystems::Backend),
            )
            .add_observer(on_double_click);
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CameraSet;

fn spawn_camera(mut commands: Commands) {
    let p = hotspot_pose(Hotspot::Overview);
    commands.spawn((
        Camera3d::default(),
        RoomCamera,
        Hdr,
        Tonemapping::AgX,
        Bloom {
            intensity: 0.22,
            ..Bloom::NATURAL
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 52f32.to_radians(),
            ..default()
        }),
        DistanceFog {
            color: Color::srgb(0.03, 0.025, 0.04),
            falloff: FogFalloff::Exponential { density: 0.045 },
            ..default()
        },
        Msaa::Sample4,
        Transform::from_translation(p.eye).looking_at(p.look, Vec3::Y),
    ));
}

/// Fly to a hotspot. Used by commands, keys and the HUD.
pub fn go_to(rig: &mut Rig, current: Pose, h: Hotspot) {
    if rig.hotspot == Some(h) && !rig.moving && rig.then.is_none() {
        return;
    }
    rig.fly(current, hotspot_pose(h), Some(h), None);
}

/// Return to the view this focus started from.
fn reset(rig: &mut Rig, current: Pose) {
    let (home, h, then) = (rig.home, rig.hotspot, None);
    rig.fly(current, home, h, then);
}

fn commands(
    mut cmds: MessageReader<Cmd>,
    mut rig: ResMut<Rig>,
    cur: Res<CurrentPose>,
    view: Res<State<View>>,
    mut next_view: ResMut<NextState<View>>,
) {
    if let Some(p) = DEBUG_POSE.lock().ok().and_then(|mut d| d.take()) {
        rig.snap(p, None);
    }
    for Cmd(c) in cmds.read() {
        match c {
            Command::FocusHotspot(h) => {
                // The site echoes our own arrivals back via the URL; already
                // there (or on the way) means nothing to do, and must not kick
                // us out of a pool game.
                if rig.hotspot == Some(*h) {
                    continue;
                }
                if *view.get() == View::Pool {
                    next_view.set(View::Room);
                }
                go_to(&mut rig, cur.0, *h);
            }
            Command::ResetView => {
                rig.touched();
                reset(&mut rig, cur.0);
            }
            Command::SetMode(Mode::Preview) => {
                rig.snap(hotspot_pose(Hotspot::Overview), Some(Hotspot::Overview))
            }
            Command::PoolExit if *view.get() == View::Pool => {
                next_view.set(View::Room);
                go_to(&mut rig, cur.0, Hotspot::Overview);
            }
            _ => {}
        }
    }
}

/// First input of the session tells the site to retire its hint overlay.
fn interacted(sent: &mut bool, rig: &mut Rig) {
    rig.touched();
    if !*sent {
        *sent = true;
        emit(RoomEvent::Interacted);
    }
}

#[allow(clippy::too_many_arguments)]
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    mut rig: ResMut<Rig>,
    cur: Res<CurrentPose>,
    view: Res<State<View>>,
    mut next_view: ResMut<NextState<View>>,
    mode: Res<RoomMode>,
    mut sent: Local<bool>,
) {
    if mode.0 != Mode::Full {
        return;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.just_pressed(KeyCode::Escape) {
        if *view.get() == View::Pool {
            next_view.set(View::Room);
        }
        interacted(&mut sent, &mut rig);
        go_to(&mut rig, cur.0, Hotspot::Overview);
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) && !shift {
        interacted(&mut sent, &mut rig);
        reset(&mut rig, cur.0);
        return;
    }
    if rig.moving {
        return;
    }

    // Held keys drive velocities, so they get the same easing as the mouse.
    let dt = time.delta_secs();
    let axis = |neg: &[KeyCode], pos: &[KeyCode]| {
        (keys.any_pressed(pos.iter().copied()) as i32
            - keys.any_pressed(neg.iter().copied()) as i32) as f32
    };
    let orbit = Vec2::new(
        axis(&[KeyCode::ArrowRight], &[KeyCode::ArrowLeft]),
        axis(&[KeyCode::ArrowDown], &[KeyCode::ArrowUp]),
    );
    let pan = Vec2::new(
        axis(&[KeyCode::KeyA], &[KeyCode::KeyD]),
        axis(&[KeyCode::KeyS], &[KeyCode::KeyW]),
    );
    let zoom = axis(
        &[KeyCode::Equal, KeyCode::NumpadAdd],
        &[KeyCode::Minus, KeyCode::NumpadSubtract],
    );
    if orbit != Vec2::ZERO || pan != Vec2::ZERO || zoom != 0.0 {
        interacted(&mut sent, &mut rig);
        if pan != Vec2::ZERO && *view.get() == View::Room {
            rig.hotspot = None;
        }
        let k = (dt * 8.0).min(1.0);
        rig.v_rot = rig.v_rot.lerp(orbit * Vec2::new(1.6, 1.0), k);
        let (right, up) = basis(&rig.cur);
        let pan_v = (right * pan.x + up * pan.y) * (1.0 + rig.cur.dist * 0.35);
        rig.v_pan = rig.v_pan.lerp(pan_v, k);
        rig.v_zoom = rig.v_zoom + (zoom * 1.4 - rig.v_zoom) * k;
    }

    if *view.get() == View::Pool {
        return;
    }
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    for (i, k) in digits.into_iter().enumerate() {
        if keys.just_pressed(k) {
            interacted(&mut sent, &mut rig);
            go_to(&mut rig, cur.0, Hotspot::ALL[i]);
        }
    }
    let here = rig.hotspot.unwrap_or(Hotspot::Overview);
    if keys.just_pressed(KeyCode::Period) {
        go_to(&mut rig, cur.0, here.cycle(true));
    }
    if keys.just_pressed(KeyCode::Comma) {
        go_to(&mut rig, cur.0, here.cycle(false));
    }
}

/// Screen-right and screen-up in world space for the current view.
fn basis(o: &Orbit) -> (Vec3, Vec3) {
    let fwd = -o.dir();
    let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(fwd).normalize_or(Vec3::Y);
    (right, up)
}

#[allow(clippy::too_many_arguments)]
fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    time: Res<Time<Real>>,
    mode: Res<RoomMode>,
    paused: Res<Paused>,
    mut rig: ResMut<Rig>,
    mut g: ResMut<Gesture>,
    mut sent: Local<bool>,
) {
    let dt = time.delta_secs().max(1e-4);
    if mode.0 != Mode::Full || paused.0 {
        wheel.clear();
        *g = Gesture::default();
        return;
    }

    // --- Zoom: wheel, trackpad scroll and trackpad pinch (ctrl+wheel).
    let mut zoom = 0.0;
    for w in wheel.read() {
        zoom += match w.unit {
            MouseScrollUnit::Line => w.y * 0.12,
            MouseScrollUnit::Pixel => w.y * 0.0016,
        };
    }

    // --- Where the pointer is: one finger, or the mouse.
    let fingers: Vec<_> = touches.iter().collect();
    let cursor = match fingers.as_slice() {
        [] => windows.single().ok().and_then(|w| w.cursor_position()),
        [t, ..] => Some(t.position()),
    };
    let pressed_now =
        mouse.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle])
            || touches.any_just_pressed();
    let held = mouse.any_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle])
        || !fingers.is_empty();

    if pressed_now && !g.captured {
        let pan = mouse.any_pressed([MouseButton::Right, MouseButton::Middle])
            || keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if let Some(c) = cursor {
            g.press = Some((c, if pan { Grip::Pan } else { Grip::Orbit }));
            g.last = Some(c);
        }
        g.pinch = None;
    }

    let mut delta = Vec2::ZERO;
    let mut grip = None;
    if held && !g.captured {
        if let (Some(c), Some((start, gr))) = (cursor, g.press) {
            if !g.dragging && c.distance(start) > DRAG_PX {
                g.dragging = true;
                g.suppress_click = true;
            }
            delta = c - g.last.unwrap_or(c);
            g.last = Some(c);
            grip = Some(gr);
        }
        // Two fingers: pan with their midpoint, zoom with their spread.
        if let [a, b, ..] = fingers.as_slice() {
            let spread = a.position().distance(b.position());
            if let Some(prev) = g.pinch {
                zoom += (spread / prev.max(1.0)).ln() * 0.9;
            }
            g.pinch = Some(spread);
            delta = (a.delta() + b.delta()) / 2.0;
            grip = Some(Grip::Pan);
            g.dragging = true;
            g.suppress_click = true;
        }
    }

    if !held {
        // A release that wasn't a drag might be half of a double-click.
        g.press = None;
        g.last = None;
        g.pinch = None;
        g.dragging = false;
    }

    if rig.moving {
        return;
    }

    if g.dragging && delta != Vec2::ZERO {
        interacted(&mut sent, &mut rig);
        if rig.hotspot.is_some() && rig.hotspot != Some(Hotspot::PoolTable) {
            // Looking around a hotspot keeps it as the focus; only a pan
            // detaches the view from it.
            if grip == Some(Grip::Pan) {
                rig.hotspot = None;
            }
        }
        match grip {
            Some(Grip::Orbit) => {
                let d = Vec2::new(-delta.x * 0.0055, delta.y * 0.0045);
                rig.target.yaw += d.x;
                rig.target.pitch += d.y;
                rig.v_rot = rig.v_rot.lerp(d / dt, 0.5);
            }
            Some(Grip::Pan) => {
                let (right, up) = basis(&rig.cur);
                let s = rig.cur.dist * 0.0013;
                let d = -right * delta.x * s + up * delta.y * s;
                rig.target.focus += d;
                rig.v_pan = rig.v_pan.lerp(d / dt, 0.5);
            }
            None => {}
        }
    } else if !held {
        // Let go: coast on inertia.
    } else {
        rig.v_rot = Vec2::ZERO;
        rig.v_pan = Vec3::ZERO;
    }

    if zoom != 0.0 {
        interacted(&mut sent, &mut rig);
        rig.target.dist *= (-zoom).exp();
        rig.v_zoom = 0.0;
    }
    rig.target = rig.target.clamped();
}

/// A new press starts a fresh gesture. Runs before picking turns a release
/// into a click, so a quick click right after a drag isn't mistaken for the
/// end of that drag.
fn reset_on_press(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut g: ResMut<Gesture>,
) {
    if (mouse.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle])
        || touches.any_just_pressed())
        && !g.captured
    {
        g.suppress_click = false;
        g.dragging = false;
    }
}

/// Double-click / double-tap any surface: glide the focus there. Uses
/// picking's own click counting (robust even when both clicks land in one
/// frame) and its hit point.
fn on_double_click(
    ev: On<PointerClick>,
    mode: Res<RoomMode>,
    view: Res<State<View>>,
    g: Res<Gesture>,
    mut rig: ResMut<Rig>,
    mut pending: ResMut<PendingClick>,
) {
    if ev.count < 2
        || ev.button != PointerButton::Primary
        || mode.0 != Mode::Full
        || g.suppress_click
        || rig.moving
    {
        return;
    }
    let Some(hit) = ev.hit.position else { return };
    pending.0 = None;
    rig.touched();
    if *view.get() == View::Room {
        rig.hotspot = None;
    }
    rig.target.focus = hit;
    rig.target.dist = rig.target.dist.min(3.5);
    rig.target = rig.target.clamped();
    emit(RoomEvent::Interacted);
}

#[allow(clippy::too_many_arguments)]
fn advance(
    time: Res<Time>,
    real: Res<Time<Real>>,
    mut rig: ResMut<Rig>,
    mut cur: ResMut<CurrentPose>,
    mut cam: Query<(&mut Transform, &mut Projection), With<RoomCamera>>,
    windows: Query<&Window>,
    view: Res<State<View>>,
    mode: Res<RoomMode>,
    g: Res<Gesture>,
    mut arrived: MessageWriter<Arrived>,
) {
    let dt = real.delta_secs().min(0.1);
    rig.idle += dt;

    let pose = if let Some(f) = rig.flight.as_mut() {
        f.t += dt / f.dur;
        let e = ease_in_out_cubic(f.t);
        let p = Pose {
            eye: f.from.eye.lerp(f.to.eye, e),
            look: f.from.look.lerp(f.to.look, e),
        };
        if f.t >= 1.0 {
            let to = f.to;
            rig.flight = None;
            rig.moving = false;
            // The destination becomes the new orbit, focused on the object.
            rig.cur = Orbit::from_pose(to);
            rig.target = rig.cur;
            if let Some(h) = rig.hotspot {
                emit(RoomEvent::Hotspot(h));
            }
            let action = rig.then.take();
            arrived.write(Arrived { action });
        }
        p
    } else {
        // Inertia carries on after a release, decaying.
        if !g.dragging {
            let decay = (-4.5 * dt).exp();
            let (v_rot, v_pan, v_zoom) = (rig.v_rot, rig.v_pan, rig.v_zoom);
            rig.target.yaw += v_rot.x * dt;
            rig.target.pitch += v_rot.y * dt;
            rig.target.focus += v_pan * dt;
            rig.target.dist *= (v_zoom * dt).exp();
            rig.v_rot *= decay;
            rig.v_pan *= decay;
            rig.v_zoom *= decay;
            rig.target = rig.target.clamped();
        }
        // Damped follow: weighty, never twitchy.
        let k = 1.0 - (-11.0 * dt).exp();
        let (c, t) = (rig.cur, rig.target);
        // Take the short way round when the yaw wraps.
        let mut dyaw = (t.yaw - c.yaw) % std::f32::consts::TAU;
        if dyaw > std::f32::consts::PI {
            dyaw -= std::f32::consts::TAU;
        } else if dyaw < -std::f32::consts::PI {
            dyaw += std::f32::consts::TAU;
        }
        rig.cur = Orbit {
            focus: c.focus.lerp(t.focus, k),
            yaw: c.yaw + dyaw * k,
            pitch: c.pitch + (t.pitch - c.pitch) * k,
            dist: c.dist + (t.dist - c.dist) * k,
        };

        // Idle drift: after 8 s without input (always, in the hero preview),
        // a slow sway blends in over 2 s.
        let still = cfg!(not(target_arch = "wasm32")) && std::env::var_os("ARTHUB_TOUR").is_some();
        let idle = if mode.0 == Mode::Preview {
            f32::MAX
        } else {
            rig.idle
        };
        let blend = if still || *view.get() == View::Pool {
            0.0
        } else {
            ((idle - IDLE_AFTER) / 2.0).clamp(0.0, 1.0)
        };
        let blend = blend * blend * (3.0 - 2.0 * blend);
        let tt = time.elapsed_secs();
        let mut o = rig.cur;
        o.yaw += (tt * 0.17).sin() * 0.07 * blend;
        o.pitch += (tt * 0.23).sin() * 0.02 * blend;
        o.focus.x += (tt * 0.11).sin() * 0.12 * blend;
        o.pose()
    };

    cur.0 = pose;
    if let Ok(mut p) = LAST_POSE.lock() {
        *p = [
            pose.eye.x,
            pose.eye.y,
            pose.eye.z,
            pose.look.x,
            pose.look.y,
            pose.look.z,
        ];
    }
    if let Ok((mut tf, mut proj)) = cam.single_mut() {
        // Keep ~64° across on narrow screens by widening the vertical FOV.
        if let (Projection::Perspective(pp), Ok(w)) = (&mut *proj, windows.single()) {
            let aspect = w.width() / w.height().max(1.0);
            let wanted = (2.0 * ((32f32.to_radians()).tan() / aspect).atan())
                .clamp(52f32.to_radians(), 84f32.to_radians());
            if (pp.fov - wanted).abs() > 1e-3 {
                pp.fov = wanted;
            }
        }
        // Straight down/up has no defined "up"; keep a tiny tilt.
        let look = if (pose.eye - pose.look).normalize_or_zero().y.abs() > 0.999 {
            pose.look + Vec3::Z * 1e-3
        } else {
            pose.look
        };
        *tf = Transform::from_translation(pose.eye).looking_at(look, Vec3::Y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pose_round_trips_through_orbit() {
        for h in Hotspot::ALL {
            let p = hotspot_pose(h);
            let back = Orbit::from_pose(p).pose();
            assert!(
                back.eye.distance(p.eye) < 0.2,
                "{h:?}: {:?} vs {:?}",
                back.eye,
                p.eye
            );
            assert!(back.look.distance(p.look) < 1e-4);
        }
    }

    #[test]
    fn camera_never_leaves_the_room() {
        for yaw in (0..36).map(|i| i as f32 * 10f32.to_radians()) {
            for pitch in [PITCH_MIN, 0.0, 0.7, PITCH_MAX] {
                let o = Orbit {
                    focus: Vec3::new(6.5, 0.2, -4.6),
                    yaw,
                    pitch,
                    dist: DIST_MAX,
                }
                .clamped();
                let e = o.pose().eye;
                assert!(
                    e.x.abs() < ROOM_HALF_X
                        && e.z.abs() < ROOM_HALF_Z
                        && e.y > 0.0
                        && e.y < ROOM_HEIGHT,
                    "{e:?}"
                );
            }
        }
    }

    #[test]
    fn limits_hold() {
        let o = Orbit {
            focus: Vec3::new(99.0, -5.0, 99.0),
            yaw: 0.0,
            pitch: 3.0,
            dist: 100.0,
        }
        .clamped();
        assert_eq!(o.pitch, PITCH_MAX);
        assert_eq!(o.dist, DIST_MAX);
        assert!(o.focus.x <= FOCUS_MAX.x && o.focus.y >= FOCUS_MIN.y && o.focus.z <= FOCUS_MAX.z);
    }
}
