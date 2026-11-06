//! Native-only HUD. In the browser the HUD is DOM (site typography, real
//! buttons, accessible); natively this small Bevy UI stands in for it.

use bevy::prelude::*;
use bridge::Hotspot;

use crate::View;
use crate::audio::Muted;
use crate::camera::Rig;
use crate::interact::{HoverLabel, LastSiteEvent};
use crate::pool::Pool;
use crate::quality::QualityState;
use crate::scene::MusicPlaying;

#[derive(Component)]
struct Status;

#[derive(Component)]
struct Tooltip;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(Update, update);
    }
}

fn spawn(mut commands: Commands) {
    let keys: String = Hotspot::ALL
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{} {}", i + 1, h.label()))
        .collect::<Vec<_>>()
        .join("  ·  ");
    commands.spawn((
        Text::new(format!(
            "ARTHUB GAME ROOM\n{keys}\ndrag orbit · right/shift-drag pan · wheel zoom · arrows/WASD/+- · R reset · double-click focus · , . cycle · Esc back · Q quality · Shift+R re-rack"
        )),
        TextFont { font_size: FontSize::Px(13.0), ..default() },
        TextColor(Color::srgba(0.95, 0.94, 0.9, 0.75)),
        Node { position_type: PositionType::Absolute, left: Val::Px(16.0), top: Val::Px(14.0), ..default() },
    ));
    commands.spawn((
        Status,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.94, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(14.0),
            ..default()
        },
    ));
    commands.spawn((
        Tooltip,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.04, 0.04, 0.04)),
        BackgroundColor(Color::srgb(0.89, 0.87, 0.82)),
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            ..default()
        },
        Visibility::Hidden,
    ));
}

#[allow(clippy::too_many_arguments)]
fn update(
    rig: Res<Rig>,
    view: Res<State<View>>,
    pool: Res<Pool>,
    q: Res<QualityState>,
    muted: Res<Muted>,
    music: Res<MusicPlaying>,
    hover: Res<HoverLabel>,
    last: Res<LastSiteEvent>,
    windows: Query<&Window>,
    mut status: Query<&mut Text, (With<Status>, Without<Tooltip>)>,
    mut tip: Query<(&mut Text, &mut Node, &mut Visibility), With<Tooltip>>,
) {
    if let Ok(mut t) = status.single_mut() {
        let spot = rig.hotspot.map(|h| h.label()).unwrap_or("—");
        let mut s = format!(
            "{}  ·  quality {:?}{}  ·  sound {}  ·  music {}",
            spot,
            q.tier,
            if q.auto { " (auto)" } else { "" },
            if muted.0 { "off" } else { "on" },
            if music.0 { "on" } else { "off" },
        );
        if *view.get() == View::Pool {
            s.push_str(&format!(
                "\nBalls potted: {}  ·  Shots: {}  ·  drag back from the cue ball, release to shoot",
                pool.game.potted, pool.game.shots
            ));
        }
        if let Some(e) = &last.0 {
            s.push_str(&format!("\nsite event: {e}"));
        }
        **t = s;
    }
    if let Ok((mut t, mut node, mut vis)) = tip.single_mut() {
        let cursor = windows.single().ok().and_then(|w| w.cursor_position());
        match (&hover.0, cursor) {
            (Some(label), Some(c)) => {
                **t = label.to_uppercase();
                node.left = Val::Px(c.x + 18.0);
                node.top = Val::Px(c.y + 14.0);
                *vis = Visibility::Inherited;
            }
            _ => *vis = Visibility::Hidden,
        }
    }
}
