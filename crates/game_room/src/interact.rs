//! Hover, highlight and click for the room's objects.
//!
//! Anything with [`Interactive`] gets picking observers attached. Hover lifts
//! the emissive of every material under the object, swaps the cursor to a
//! pointer and tells the site to show a tooltip. Click flies the camera to the
//! object's viewpoint and, on arrival, runs its [`Action`].

use bevy::prelude::*;
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};
use bridge::{Hotspot, Mode, RoomEvent};

use crate::audio::Sfx;
use crate::camera::{CurrentPose, Gesture, Rig, cabinet_pose, dartboard_pose, hotspot_pose};
use crate::link::emit;
use crate::neon::CycleNeon;
use crate::{Paused, RoomMode, View};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Hotspot(Hotspot),
    Cabinet {
        index: usize,
        path: String,
    },
    Dartboard,
    NeonCycle,
    /// A framed print of a project: fly up to it, then open the case study.
    Print {
        path: String,
        pose: crate::camera::Pose,
    },
}

#[derive(Component)]
pub struct Interactive {
    pub label: String,
    pub action: Action,
}

impl Interactive {
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
}

/// The camera finished a flight; run the pending action, if any.
#[derive(Message)]
pub struct Arrived {
    pub action: Option<Action>,
}

#[derive(Resource, Default)]
struct Hovered {
    entity: Option<Entity>,
    /// Meshes showing a highlighted copy of their material, with the
    /// original to put back. Materials are shared across the room, so the
    /// highlight never edits them in place.
    swapped: Vec<(Entity, Handle<StandardMaterial>)>,
}

/// Current hover label (the native HUD draws it; the site gets an event).
#[derive(Resource, Default)]
pub struct HoverLabel(pub Option<String>);

/// Last thing the room asked the site to do (shown by the native HUD).
#[derive(Resource, Default)]
pub struct LastSiteEvent(pub Option<String>);

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hovered>()
            .init_resource::<HoverLabel>()
            .init_resource::<LastSiteEvent>()
            .add_message::<Arrived>()
            .init_resource::<PendingClick>()
            .add_systems(
                Update,
                (
                    wire_observers,
                    fire_pending_click,
                    on_arrival,
                    drop_hover_when_inactive,
                ),
            );
    }
}

fn wire_observers(mut commands: Commands, added: Query<Entity, Added<Interactive>>) {
    for e in &added {
        commands
            .entity(e)
            .observe(on_over)
            .observe(on_out)
            .observe(on_click);
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct Gate<'w> {
    mode: Res<'w, RoomMode>,
    view: Res<'w, State<View>>,
    paused: Res<'w, Paused>,
    gesture: Res<'w, Gesture>,
}

impl Gate<'_> {
    /// Hover and click are live: the full room, not playing pool, and no
    /// drag in progress (orbiting shouldn't light things up or click them).
    fn open(&self) -> bool {
        self.mode.0 == Mode::Full
            && *self.view.get() == View::Room
            && !self.paused.0
            && !self.gesture.dragging
    }
}

/// A single click waits this long in case it's the first half of a
/// double-click (which refocuses the camera instead).
const CLICK_DELAY: f32 = 0.25;

/// The object clicked, and when; acted on after [`CLICK_DELAY`] unless a
/// double-click cancels it.
#[derive(Resource, Default)]
pub struct PendingClick(pub Option<(Entity, f32)>);

/// Walk up from a picked mesh to the object that owns it.
fn owner(mut e: Entity, items: &Query<&Interactive>, parents: &Query<&ChildOf>) -> Option<Entity> {
    loop {
        if items.contains(e) {
            return Some(e);
        }
        e = parents.get(e).ok()?.parent();
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct Highlighter<'w, 's> {
    hovered: ResMut<'w, Hovered>,
    label: ResMut<'w, HoverLabel>,
    children: Query<'w, 's, &'static Children>,
    mesh_mats: Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>>,
    mats: ResMut<'w, Assets<StandardMaterial>>,
    window: Query<'w, 's, Entity, With<PrimaryWindow>>,
    commands: Commands<'w, 's>,
}

impl Highlighter<'_, '_> {
    fn set(&mut self, target: Option<(Entity, &str)>) {
        if self.hovered.entity == target.map(|t| t.0) {
            return;
        }
        // Restore the previous object (its highlight copies drop with the handles).
        for (e, original) in std::mem::take(&mut self.hovered.swapped) {
            if let Ok(mut ec) = self.commands.get_entity(e) {
                ec.insert(MeshMaterial3d(original));
            }
        }
        self.hovered.entity = target.map(|t| t.0);

        let cursor = if target.is_some() {
            SystemCursorIcon::Pointer
        } else {
            SystemCursorIcon::Default
        };
        if let Ok(w) = self.window.single() {
            self.commands.entity(w).insert(CursorIcon::from(cursor));
        }

        let label = target.map(|t| t.1.to_string());
        emit(RoomEvent::Hover {
            label: label.clone(),
        });
        self.label.0 = label;

        let Some((e, _)) = target else { return };
        let lift = LinearRgba::rgb(0.22, 0.2, 0.17);
        let mut copies: Vec<(AssetId<StandardMaterial>, Handle<StandardMaterial>)> = Vec::new();
        for d in std::iter::once(e).chain(self.children.iter_descendants(e)) {
            let Ok(mm) = self.mesh_mats.get(d) else {
                continue;
            };
            let original = mm.0.clone();
            let copy = match copies.iter().find(|(id, _)| *id == original.id()) {
                Some((_, h)) => h.clone(),
                None => {
                    let Some(m) = self.mats.get(&original) else {
                        continue;
                    };
                    let mut m = m.clone();
                    m.emissive += lift;
                    let h = self.mats.add(m);
                    copies.push((original.id(), h.clone()));
                    h
                }
            };
            self.commands.entity(d).insert(MeshMaterial3d(copy));
            self.hovered.swapped.push((d, original));
        }
    }
}

fn on_over(
    mut ev: On<PointerOver>,
    items: Query<&Interactive>,
    parents: Query<&ChildOf>,
    gate: Gate,
    mut hl: Highlighter,
) {
    ev.propagate(false);
    if !gate.open() {
        return;
    }
    if let Some(e) = owner(ev.entity, &items, &parents) {
        let label = items.get(e).map(|i| i.label.clone()).unwrap_or_default();
        hl.set(Some((e, &label)));
    }
}

fn on_out(
    mut ev: On<PointerOut>,
    items: Query<&Interactive>,
    parents: Query<&ChildOf>,
    mut hl: Highlighter,
) {
    ev.propagate(false);
    if owner(ev.entity, &items, &parents) == hl.hovered.entity {
        hl.set(None);
    }
}

#[allow(clippy::too_many_arguments)]
fn on_click(
    mut ev: On<PointerClick>,
    items: Query<&Interactive>,
    parents: Query<&ChildOf>,
    gate: Gate,
    time: Res<Time<Real>>,
    mut pending: ResMut<PendingClick>,
) {
    ev.propagate(false);
    if !gate.open() || gate.gesture.suppress_click || ev.button != PointerButton::Primary {
        return;
    }
    // The second click of a double-click refocuses the camera instead.
    if ev.count > 1 {
        pending.0 = None;
        return;
    }
    if let Some(e) = owner(ev.entity, &items, &parents) {
        pending.0 = Some((e, time.elapsed_secs()));
    }
}

/// Act on a single click once it's clearly not a double-click.
#[allow(clippy::too_many_arguments)]
fn fire_pending_click(
    time: Res<Time<Real>>,
    mut pending: ResMut<PendingClick>,
    items: Query<&Interactive>,
    gate: Gate,
    mut rig: ResMut<Rig>,
    cur: Res<CurrentPose>,
    mut neon: MessageWriter<CycleNeon>,
    mut sfx: MessageWriter<Sfx>,
) {
    let Some((e, at)) = pending.0 else { return };
    if time.elapsed_secs() - at < CLICK_DELAY {
        return;
    }
    pending.0 = None;
    let (Ok(item), true) = (items.get(e), gate.open()) else {
        return;
    };
    sfx.write(Sfx::Click);
    let action = item.action.clone();
    match &action {
        Action::Hotspot(h) => rig.fly(cur.0, hotspot_pose(*h), Some(*h), Some(action.clone())),
        Action::Cabinet { index, .. } => rig.fly(
            cur.0,
            cabinet_pose(*index),
            Some(Hotspot::ArcadeRow),
            Some(action.clone()),
        ),
        Action::Dartboard => rig.fly(cur.0, dartboard_pose(), None, Some(action.clone())),
        Action::NeonCycle => {
            neon.write(CycleNeon(e));
        }
        Action::Print { pose, .. } => rig.fly(cur.0, *pose, None, Some(action.clone())),
    }
}

fn navigate(last: &mut LastSiteEvent, path: &str) {
    last.0 = Some(format!("→ {path}"));
    emit(RoomEvent::Navigate { path: path.into() });
}

fn on_arrival(
    mut arrived: MessageReader<Arrived>,
    mut next_view: ResMut<NextState<View>>,
    mut sfx: MessageWriter<Sfx>,
    mut last: ResMut<LastSiteEvent>,
    mut rig: ResMut<Rig>,
    cur: Res<CurrentPose>,
) {
    for a in arrived.read() {
        let Some(action) = &a.action else { continue };
        match action {
            Action::Hotspot(h) => match h {
                Hotspot::PoolTable => next_view.set(View::Pool),
                Hotspot::TrophyShelf => navigate(&mut last, "/about"),
                Hotspot::Chalkboard => navigate(&mut last, "/skills"),
                Hotspot::Bar => navigate(&mut last, "/contact"),
                Hotspot::Jukebox => {
                    sfx.write(Sfx::ToggleMusic);
                }
                Hotspot::Tv => {
                    last.0 = Some("→ showreel".into());
                    emit(RoomEvent::ShowreelOpen);
                }
                Hotspot::Exit => {
                    last.0 = Some("→ exit".into());
                    emit(RoomEvent::Exit);
                    // Natively there's nowhere to go; head back in.
                    #[cfg(not(target_arch = "wasm32"))]
                    rig.fly(
                        cur.0,
                        hotspot_pose(Hotspot::Overview),
                        Some(Hotspot::Overview),
                        None,
                    );
                }
                Hotspot::Overview | Hotspot::ArcadeRow => {}
            },
            Action::Cabinet { path, .. } => navigate(&mut last, path),
            Action::Dartboard => navigate(&mut last, "/lab"),
            Action::Print { path, .. } => navigate(&mut last, path),
            Action::NeonCycle => {}
        }
    }
    let _ = (&mut rig, &cur);
}

fn drop_hover_when_inactive(gate: Gate, mut hl: Highlighter) {
    if !gate.open() && hl.hovered.entity.is_some() {
        hl.set(None);
    }
}
