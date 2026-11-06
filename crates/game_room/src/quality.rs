//! Quality tiers and the frame-time watchdog.
//!
//! | tier   | shadows   | bloom | MSAA |
//! |--------|-----------|-------|------|
//! | low    | off       | off   | off  |
//! | medium | pool lamp | on    | off  |
//! | high   | pool lamp | on    | 4×   |
//!
//! Resolution is deliberately *not* scaled via the window's scale-factor
//! override: bevy_winit converts cursor and touch positions with that factor
//! while the browser reports them at the real device pixel ratio, so any
//! override breaks picking on high-DPI screens.
//!
//! If frames average over 20 ms for a sustained stretch, we drop a tier and
//! tell the site. Picking a tier by hand switches the watchdog off.

use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bridge::{Command, Mode, Quality, RoomEvent};

use crate::camera::RoomCamera;
use crate::link::emit;
use crate::scene::ShadowCaster;
use crate::{Cmd, Paused, RoomMode};

const BUDGET: f32 = 0.020;

#[derive(Resource)]
pub struct QualityState {
    pub tier: Quality,
    pub auto: bool,
    ema: f32,
    slow: f32,
    grace: f32,
    applied: Option<(Quality, Mode)>,
}

pub struct QualityPlugin;

impl Plugin for QualityPlugin {
    fn build(&self, app: &mut App) {
        let tier = if cfg!(target_arch = "wasm32") {
            Quality::Medium
        } else {
            Quality::High
        };
        // Natively, ARTHUB_QUALITY=low|medium|high pins a tier (handy for profiling).
        #[cfg(not(target_arch = "wasm32"))]
        let (tier, auto) = match std::env::var("ARTHUB_QUALITY")
            .ok()
            .and_then(|q| Quality::from_id(&q))
        {
            Some(q) => (q, false),
            None => (tier, true),
        };
        #[cfg(target_arch = "wasm32")]
        let auto = true;
        app.insert_resource(QualityState {
            tier,
            auto,
            ema: 0.016,
            slow: 0.0,
            grace: 4.0,
            applied: None,
        })
        .add_systems(Update, (commands, watchdog, apply).chain());
    }
}

fn commands(
    mut cmds: MessageReader<Cmd>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: ResMut<QualityState>,
    mode: Res<RoomMode>,
) {
    for Cmd(c) in cmds.read() {
        match c {
            Command::SetQuality(t) => {
                q.tier = *t;
                q.auto = false;
            }
            Command::Resume | Command::SetMode(_) => q.grace = 3.0,
            _ => {}
        }
    }
    if mode.0 == Mode::Full && keys.just_pressed(KeyCode::KeyQ) {
        let i = Quality::ALL.iter().position(|t| *t == q.tier).unwrap_or(0);
        q.tier = Quality::ALL[(i + 1) % 3];
        q.auto = false;
        emit(RoomEvent::Quality(q.tier));
    }
}

fn watchdog(time: Res<Time<Real>>, mut q: ResMut<QualityState>, paused: Res<Paused>) {
    let dt = time.delta_secs();
    if paused.0 || dt <= 0.0 || dt > 0.5 {
        // Ignore hitches from tab switches and loading.
        return;
    }
    q.ema = q.ema * 0.92 + dt * 0.08;
    if q.grace > 0.0 {
        q.grace -= dt;
        return;
    }
    q.slow = if q.ema > BUDGET {
        q.slow + dt
    } else {
        (q.slow - dt * 0.5).max(0.0)
    };
    if q.auto && q.slow > 2.5 {
        if let Some(lower) = q.tier.lower() {
            info!(
                "frame time {:.1} ms over budget, dropping to {:?}",
                q.ema * 1000.0,
                lower
            );
            q.tier = lower;
            emit(RoomEvent::Quality(lower));
        }
        q.slow = 0.0;
        q.grace = 4.0;
    }
}

fn apply(
    mut q: ResMut<QualityState>,
    mode: Res<RoomMode>,
    mut commands: Commands,
    mut cam: Query<(Entity, &mut Msaa), With<RoomCamera>>,
    mut spots: Query<&mut SpotLight, With<ShadowCaster>>,
) {
    if q.applied == Some((q.tier, mode.0)) {
        return;
    }
    let Ok((cam_e, mut msaa)) = cam.single_mut() else {
        return;
    };
    // The hero preview is a teaser: keep the glow, drop the expensive bits.
    let preview = mode.0 == Mode::Preview;
    let tier = q.tier;

    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut shadows = tier != Quality::Low && !preview;
    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut bloom = tier != Quality::Low;
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Debug switches for isolating render issues.
        let off = |k: &str| std::env::var_os(k).is_some();
        shadows &= !off("ARTHUB_NO_SHADOWS");
        bloom &= !off("ARTHUB_NO_BLOOM");
    }
    *msaa = if tier == Quality::High && !preview {
        Msaa::Sample4
    } else {
        Msaa::Off
    };
    if bloom {
        commands.entity(cam_e).insert(Bloom {
            intensity: 0.22,
            ..Bloom::NATURAL
        });
    } else {
        commands.entity(cam_e).remove::<Bloom>();
    }
    for mut s in &mut spots {
        s.shadow_maps_enabled = shadows;
    }
    q.applied = Some((tier, mode.0));
}
