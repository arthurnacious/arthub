//! The room's end of the site bridge.
//!
//! Incoming: commands arrive from JS callbacks outside the Bevy schedule, so
//! they land in a global queue and get drained into [`Cmd`] messages each
//! frame. If the app is paused (reactive mode), the callback also wakes the
//! event loop so the command is processed immediately.
//!
//! Outgoing: [`emit`] dispatches a `CustomEvent` on the web and logs natively.

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{EventLoopProxyWrapper, UpdateMode, WinitSettings, WinitUserEvent};
use bridge::{Command, Mode, RoomEvent};

use crate::{Cmd, Paused, RoomMode, View};

static QUEUE: Mutex<Vec<Command>> = Mutex::new(Vec::new());
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// Queue a command from outside the schedule and wake the loop.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn push_command(cmd: Command) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push(cmd);
    }
    if let Some(wake) = WAKE.get() {
        wake();
    }
}

pub fn emit(ev: RoomEvent) {
    #[cfg(target_arch = "wasm32")]
    bridge::web::send_room_event(&ev);
    #[cfg(not(target_arch = "wasm32"))]
    info!("room → site: {ev:?}");
}

pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, install)
            .add_systems(PreUpdate, drain)
            .add_systems(Update, (apply_lifecycle, announce_ready));
    }
}

fn install(proxy: Option<Res<EventLoopProxyWrapper>>) {
    if let Some(proxy) = proxy {
        let proxy = (**proxy).clone();
        let _ = WAKE.set(Box::new(move || {
            let _ = proxy.send_event(WinitUserEvent::WakeUp);
        }));
    }
    #[cfg(target_arch = "wasm32")]
    bridge::web::on_command(push_command);
}

fn drain(mut out: MessageWriter<Cmd>) {
    let cmds = QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for c in cmds {
        out.write(Cmd(c));
    }
}

fn apply_lifecycle(
    mut cmds: MessageReader<Cmd>,
    mut paused: ResMut<Paused>,
    mut winit: ResMut<WinitSettings>,
    mut mode: ResMut<RoomMode>,
    mut time: ResMut<Time<Virtual>>,
    mut next_view: ResMut<NextState<View>>,
) {
    for Cmd(c) in cmds.read() {
        match c {
            Command::Pause => {
                paused.0 = true;
                time.pause();
                // Sleep until something (a command) wakes us.
                let idle = UpdateMode::reactive_low_power(Duration::from_secs(3600));
                winit.focused_mode = idle;
                winit.unfocused_mode = idle;
            }
            Command::Resume => {
                paused.0 = false;
                time.unpause();
                winit.focused_mode = UpdateMode::Continuous;
                winit.unfocused_mode = UpdateMode::Continuous;
            }
            Command::SetMode(m) => {
                mode.0 = *m;
                if *m == Mode::Preview {
                    next_view.set(View::Room);
                }
            }
            _ => {}
        }
    }
}

fn announce_ready(mut frames: Local<u32>) {
    // A few frames in, the scene is built and the first images are on screen.
    *frames += 1;
    if *frames == 3 {
        emit(RoomEvent::Ready);
    }
}
