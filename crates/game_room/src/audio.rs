//! Sound with zero downloads: every effect and the jukebox loop are
//! synthesised into in-memory WAVs at startup (music lazily, on first play).
//!
//! Master mute defaults to on in the browser (autoplay rules, and nobody wants
//! a portfolio that shouts) and off natively.

use std::sync::Arc;

use bevy::audio::{AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use bridge::{Command, RoomEvent};

use crate::Cmd;
use crate::link::emit;
use crate::scene::MusicPlaying;

#[derive(Message, Clone, Copy, Debug)]
pub enum Sfx {
    /// Ball-on-ball, with impact speed (m/s).
    Clack(f32),
    Cushion(f32),
    Cue(f32),
    Pocket,
    Click,
    ToggleMusic,
}

#[derive(Resource)]
pub struct Muted(pub bool);

#[derive(Resource)]
struct Sounds {
    clack: Handle<AudioSource>,
    cushion: Handle<AudioSource>,
    cue: Handle<AudioSource>,
    pocket: Handle<AudioSource>,
    click: Handle<AudioSource>,
    music: Option<Handle<AudioSource>>,
}

#[derive(Component)]
struct MusicTrack;

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Sfx>()
            .insert_resource(Muted(cfg!(target_arch = "wasm32")))
            .init_resource::<MusicPlaying>()
            .add_systems(Startup, build_sounds)
            .add_systems(Update, (commands, play).chain());
    }
}

const RATE: u32 = 22_050;

fn wav(samples: &[f32]) -> AudioSource {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32_000.0) as i16).to_le_bytes());
    }
    AudioSource {
        bytes: Arc::from(b),
    }
}

/// Deterministic white noise.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

/// Sum of decaying partials plus a noise transient.
fn hit(
    len: f32,
    partials: &[(f32, f32, f32)],
    noise_amt: f32,
    noise_decay: f32,
    seed: u32,
) -> Vec<f32> {
    let n = (len * RATE as f32) as usize;
    let mut rng = Noise(seed);
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let tone: f32 = partials
                .iter()
                .map(|&(f, amp, decay)| {
                    (t * f * std::f32::consts::TAU).sin() * amp * (-t / decay).exp()
                })
                .sum();
            tone + rng.next() * noise_amt * (-t / noise_decay).exp()
        })
        .collect()
}

fn build_sounds(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    let clack = hit(
        0.06,
        &[
            (2350.0, 0.5, 0.007),
            (3900.0, 0.3, 0.005),
            (5600.0, 0.15, 0.004),
        ],
        0.35,
        0.002,
        1,
    );
    let cushion = hit(
        0.08,
        &[(140.0, 0.6, 0.02), (720.0, 0.2, 0.01)],
        0.15,
        0.004,
        2,
    );
    let cue = hit(
        0.08,
        &[(180.0, 0.6, 0.015), (1500.0, 0.3, 0.004)],
        0.3,
        0.003,
        3,
    );
    let click = hit(
        0.04,
        &[(1800.0, 0.25, 0.006), (900.0, 0.15, 0.01)],
        0.05,
        0.002,
        4,
    );
    // Pocket: a thud, then the ball rattling down the return.
    let mut pocket = hit(0.45, &[(90.0, 0.7, 0.06), (210.0, 0.3, 0.03)], 0.2, 0.01, 5);
    for k in 1..5 {
        let at = (0.08 * k as f32 * RATE as f32) as usize;
        for (j, s) in hit(
            0.05,
            &[(1200.0 + k as f32 * 150.0, 0.12 / k as f32, 0.006)],
            0.05,
            0.003,
            9 + k,
        )
        .into_iter()
        .enumerate()
        {
            if let Some(p) = pocket.get_mut(at + j) {
                *p += s;
            }
        }
    }
    commands.insert_resource(Sounds {
        clack: sources.add(wav(&clack)),
        cushion: sources.add(wav(&cushion)),
        cue: sources.add(wav(&cue)),
        pocket: sources.add(wav(&pocket)),
        click: sources.add(wav(&click)),
        music: None,
    });
}

/// A slow lounge loop: Rhodes-ish chords, upright bass, brushed hats.
/// Am9 – Dm9 – G13 – Cmaj9 at 84 bpm, two bars each, seamless.
fn lounge_loop() -> Vec<f32> {
    let beat = 60.0 / 84.0;
    let bar = beat * 4.0;
    let chords: [(f32, [f32; 4]); 4] = [
        (110.0, [261.63, 329.63, 392.0, 493.88]),
        (73.42, [349.23, 440.0, 523.25, 659.25]),
        (98.0, [349.23, 493.88, 659.25, 698.46]),
        (65.41, [329.63, 392.0, 493.88, 587.33]),
    ];
    let total = bar * 2.0 * chords.len() as f32;
    let n = (total * RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    let mut rng = Noise(77);
    let tau = std::f32::consts::TAU;

    for (ci, (bass, notes)) in chords.iter().enumerate() {
        let start = ci as f32 * bar * 2.0;
        // Chord stabs on beats 1 and the "and" of 2, each bar.
        for b in 0..2 {
            for &off in &[0.0, beat * 1.5] {
                let t0 = start + b as f32 * bar + off;
                let i0 = (t0 * RATE as f32) as usize;
                let len = (beat * 2.4 * RATE as f32) as usize;
                for j in 0..len {
                    let t = j as f32 / RATE as f32;
                    let env = (1.0 - (-t / 0.01).exp()) * (-t / 0.9).exp();
                    let mut s = 0.0;
                    for (k, f) in notes.iter().enumerate() {
                        let det = 1.0 + (k as f32 - 1.5) * 0.0015;
                        s += (t * f * det * tau).sin()
                            + 0.25 * (t * f * 2.0 * tau).sin() * (-t / 0.25).exp();
                    }
                    out[(i0 + j) % n] += s * env * 0.045;
                }
            }
            // Walking-ish bass: root on 1, fifth on 3.
            for (q, mult) in [(0.0, 1.0), (beat * 2.0, 1.5)] {
                let i0 = ((start + b as f32 * bar + q) * RATE as f32) as usize;
                let len = (beat * 1.8 * RATE as f32) as usize;
                for j in 0..len {
                    let t = j as f32 / RATE as f32;
                    let env = (1.0 - (-t / 0.005).exp()) * (-t / 0.5).exp();
                    let f = bass * mult;
                    out[(i0 + j) % n] +=
                        ((t * f * tau).sin() + 0.3 * (t * f * 2.0 * tau).sin()) * env * 0.22;
                }
            }
        }
    }
    // Brushed hats on the off-beats.
    let beats = (total / beat) as usize;
    for b in 0..beats {
        let i0 = (((b as f32 + 0.5) * beat) * RATE as f32) as usize;
        let mut prev = 0.0;
        for j in 0..(0.12 * RATE as f32) as usize {
            let t = j as f32 / RATE as f32;
            let x = rng.next();
            let hp = x - prev;
            prev = x;
            out[(i0 + j) % n] += hp * (-t / 0.03).exp() * 0.035;
        }
    }
    out
}

fn commands(
    mut cmds: MessageReader<Cmd>,
    mut muted: ResMut<Muted>,
    mut playing: ResMut<MusicPlaying>,
    mut commands: Commands,
    music: Query<Entity, With<MusicTrack>>,
) {
    for Cmd(c) in cmds.read() {
        if let Command::Mute(m) = c {
            muted.0 = *m;
            if *m && playing.0 {
                for e in &music {
                    commands.entity(e).despawn();
                }
                playing.0 = false;
                emit(RoomEvent::Music { playing: false });
            }
        }
    }
}

fn play(
    mut sfx: MessageReader<Sfx>,
    mut sounds: Option<ResMut<Sounds>>,
    mut sources: ResMut<Assets<AudioSource>>,
    mut muted: ResMut<Muted>,
    mut playing: ResMut<MusicPlaying>,
    music: Query<Entity, With<MusicTrack>>,
    mut commands: Commands,
) {
    let Some(sounds) = sounds.as_mut() else {
        return;
    };
    let mut budget = 6;
    for s in sfx.read() {
        if let Sfx::ToggleMusic = s {
            if playing.0 {
                for e in &music {
                    commands.entity(e).despawn();
                }
            } else {
                // Choosing a track at the jukebox is an explicit opt-in to sound.
                muted.0 = false;
                let track = sounds
                    .music
                    .get_or_insert_with(|| sources.add(wav(&lounge_loop())))
                    .clone();
                commands.spawn((
                    MusicTrack,
                    AudioPlayer(track),
                    PlaybackSettings {
                        volume: Volume::Linear(0.55),
                        ..PlaybackSettings::LOOP
                    },
                ));
            }
            playing.0 = !playing.0;
            emit(RoomEvent::Music { playing: playing.0 });
            continue;
        }
        if muted.0 || budget == 0 {
            continue;
        }
        budget -= 1;
        let (h, v) = match *s {
            Sfx::Clack(speed) => (&sounds.clack, (speed / 3.0).clamp(0.05, 1.0)),
            Sfx::Cushion(speed) => (&sounds.cushion, (speed / 3.0).clamp(0.05, 0.8)),
            Sfx::Cue(speed) => (&sounds.cue, (speed / 4.0).clamp(0.2, 1.0)),
            Sfx::Pocket => (&sounds.pocket, 0.8),
            Sfx::Click => (&sounds.click, 0.5),
            Sfx::ToggleMusic => unreachable!(),
        };
        commands.spawn((
            AudioPlayer(h.clone()),
            PlaybackSettings {
                volume: Volume::Linear(v),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}
