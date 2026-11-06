//! Native-only automated tour, for checking the room without clicking around:
//!
//! ```sh
//! ARTHUB_TOUR=/tmp/shots cargo run -p game_room --release
//! ```
//!
//! Visits every hotspot and screenshots it, plays a break shot, measures the
//! frame rate in the overview, prints a summary and exits.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bridge::{Command, Hotspot};

use crate::View;
use crate::camera::RoomCamera;
use crate::interact::Interactive;
use crate::link::push_command;
use crate::pool::Pool;

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(dir) = std::env::var("ARTHUB_TOUR") {
            let _ = std::fs::create_dir_all(&dir);
            app.insert_resource(Tour {
                dir,
                step: 0,
                at: 0.0,
                frames: 0,
                fps_from: None,
            })
            .add_systems(Update, tour);
        }
    }
}

#[derive(Resource)]
struct Tour {
    dir: String,
    step: usize,
    at: f32,
    frames: u32,
    fps_from: Option<f32>,
}

enum Step {
    Wait(f32),
    Focus(Hotspot),
    Shot(&'static str),
    EnterPool,
    Break,
    MeasureFps(f32),
    Quit,
}

fn script() -> Vec<Step> {
    use Step::*;
    if std::env::var_os("ARTHUB_TOUR_QUICK").is_some() {
        return vec![
            Wait(3.0),
            Shot("overview"),
            Wait(0.5),
            MeasureFps(4.0),
            Quit,
        ];
    }
    let mut s = vec![Wait(4.0), Shot("overview")];
    for h in Hotspot::ALL.into_iter().skip(1) {
        s.extend([Focus(h), Wait(2.2), Shot(h.id())]);
    }
    s.extend([
        Focus(Hotspot::Overview),
        Wait(2.0),
        EnterPool,
        Wait(2.5),
        Shot("pool-ready"),
        Break,
        Wait(0.35),
        Shot("pool-break"),
        Wait(9.0),
        Shot("pool-settled"),
        Focus(Hotspot::Overview),
        Wait(3.0),
        MeasureFps(8.0),
        Quit,
    ]);
    s
}

#[allow(clippy::too_many_arguments)]
fn tour(
    time: Res<Time<Real>>,
    mut tour: ResMut<Tour>,
    mut commands: Commands,
    mut next_view: ResMut<NextState<View>>,
    mut pool: ResMut<Pool>,
    mut exit: MessageWriter<AppExit>,
    cam: Query<(&Camera, &GlobalTransform), With<RoomCamera>>,
    windows: Query<&Window>,
    items: Query<(Entity, &Interactive)>,
    children: Query<&Children>,
    globals: Query<&GlobalTransform, With<Mesh3d>>,
) {
    let now = time.elapsed_secs();
    tour.frames += 1;
    let steps = script();
    let Some(step) = steps.get(tour.step) else {
        return;
    };
    match step {
        Step::Wait(d) => {
            if tour.at == 0.0 {
                tour.at = now;
            }
            if now - tour.at < *d {
                return;
            }
            tour.at = 0.0;
        }
        Step::Focus(h) => push_command(Command::FocusHotspot(*h)),
        Step::Shot(name) if *name == "overview" => {
            let path = format!("{}/{name}.png", tour.dir);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            print_spots(&cam, &windows, &items, &children, &globals);
        }
        Step::Shot(name) => {
            let path = format!("{}/{name}.png", tour.dir);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
        Step::EnterPool => next_view.set(View::Pool),
        Step::Break => {
            pool.game.shoot(pool_physics::Vec2::new(1.0, 0.012), 1.0);
        }
        Step::MeasureFps(d) => match tour.fps_from {
            None => {
                tour.fps_from = Some(now);
                tour.frames = 0;
                return;
            }
            Some(t0) if now - t0 < *d => return,
            Some(t0) => {
                let fps = tour.frames as f32 / (now - t0);
                println!(
                    "TOUR fps={fps:.1} frame_ms={:.2} over {:.1}s",
                    1000.0 / fps,
                    now - t0
                );
                println!(
                    "TOUR pool potted={} shots={}",
                    pool.game.potted, pool.game.shots
                );
            }
        },
        Step::Quit => {
            exit.write(AppExit::Success);
        }
    }
    tour.step += 1;
}

/// Where each interactive object lands on screen from the overview camera,
/// as percentages: the hotspot map for the static fallback image.
fn print_spots(
    cam: &Query<(&Camera, &GlobalTransform), With<RoomCamera>>,
    windows: &Query<&Window>,
    items: &Query<(Entity, &Interactive)>,
    children: &Query<&Children>,
    globals: &Query<&GlobalTransform, With<Mesh3d>>,
) {
    let (Ok((camera, cam_gt)), Ok(w)) = (cam.single(), windows.single()) else {
        return;
    };
    for (e, item) in items {
        let pts: Vec<Vec3> = children
            .iter_descendants(e)
            .filter_map(|d| globals.get(d).ok())
            .map(|g| g.translation())
            .collect();
        if pts.is_empty() {
            continue;
        }
        let centre = pts.iter().copied().sum::<Vec3>() / pts.len() as f32;
        if let Ok(p) = camera.world_to_viewport(cam_gt, centre) {
            println!(
                "TOUR spot {:?} {:.1}% {:.1}%",
                item.label,
                p.x / w.width() * 100.0,
                p.y / w.height() * 100.0
            );
        }
    }
}
