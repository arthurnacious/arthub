use super::*;

/// A game with every object ball pocketed (out of play) and the cue ball at
/// `cue`, for setting up isolated scenarios.
fn empty_game(cue: Vec2) -> PoolGame {
    let mut g = PoolGame::new();
    for b in g.balls.iter_mut().skip(1) {
        b.pocketed = true;
        b.vel = Vec2::ZERO;
    }
    g.balls[0].pos = cue;
    g
}

fn place(g: &mut PoolGame, id: u8, pos: Vec2) {
    let b = &mut g.balls[id as usize];
    b.pos = pos;
    b.vel = Vec2::ZERO;
    b.pocketed = false;
}

/// Run until settled (or `max_t` seconds); returns all events and elapsed time.
fn run(g: &mut PoolGame, max_t: f32) -> (Vec<GameEvent>, f32) {
    let mut all = Vec::new();
    let mut t = 0.0;
    let dt = 1.0 / 60.0;
    while t < max_t {
        let ev = g.step(dt);
        t += dt;
        let settled = ev.contains(&GameEvent::Settled);
        all.extend(ev);
        if settled {
            break;
        }
    }
    (all, t)
}

fn assert_no_overlaps(g: &PoolGame, tol: f32) {
    let live: Vec<&Ball> = g.balls.iter().filter(|b| !b.pocketed).collect();
    for (i, a) in live.iter().enumerate() {
        for b in &live[i + 1..] {
            let d = a.pos.distance(b.pos);
            assert!(
                d >= 2.0 * BALL_RADIUS - tol,
                "balls {} and {} overlap: dist {d}",
                a.id,
                b.id
            );
        }
    }
}

fn assert_inside_rails(g: &PoolGame) {
    let t = &g.table;
    for b in g.balls.iter().filter(|b| !b.pocketed) {
        assert!(
            b.pos.x.abs() <= t.half_length - BALL_RADIUS + 1e-4
                && b.pos.y.abs() <= t.half_width - BALL_RADIUS + 1e-4,
            "ball {} outside rails at {:?}",
            b.id,
            b.pos
        );
    }
}

#[test]
fn kinds() {
    let g = PoolGame::new();
    assert_eq!(g.balls[0].kind(), BallKind::Cue);
    assert_eq!(g.balls[1].kind(), BallKind::Solid);
    assert_eq!(g.balls[7].kind(), BallKind::Solid);
    assert_eq!(g.balls[8].kind(), BallKind::Eight);
    assert_eq!(g.balls[9].kind(), BallKind::Stripe);
    assert_eq!(g.balls[15].kind(), BallKind::Stripe);
}

#[test]
fn rack_is_valid() {
    let g = PoolGame::new();
    assert_eq!(g.balls.len(), 16);
    for (i, b) in g.balls.iter().enumerate() {
        assert_eq!(b.id as usize, i);
        assert!(!b.pocketed);
        assert_eq!(b.vel, Vec2::ZERO);
    }
    assert_no_overlaps(&g, 0.0);
    assert_inside_rails(&g);
    let t = &g.table;
    assert_eq!(g.balls[0].pos, Vec2::new(-t.half_length / 2.0, 0.0));
    // Apex on the foot spot, the 8 in the centre of the third row.
    let apex = g
        .balls
        .iter()
        .skip(1)
        .min_by(|a, b| a.pos.x.total_cmp(&b.pos.x))
        .unwrap();
    assert!((apex.pos - Vec2::new(t.half_length / 2.0, 0.0)).length() < 1e-6);
    assert!(g.balls[8].pos.y.abs() < 1e-6);
    assert!(g.balls[8].pos.x > apex.pos.x);
    assert_eq!(g.potted, 0);
    assert_eq!(g.shots, 0);
    assert!(g.can_shoot());
}

#[test]
fn straight_shot_transfers_velocity() {
    let mut g = empty_game(Vec2::new(-0.5, 0.0));
    place(&mut g, 3, Vec2::new(0.0, 0.0));
    assert!(g.shoot(Vec2::X, 0.4));
    let mut hit = None;
    for _ in 0..120 {
        for e in g.step(1.0 / 60.0) {
            if let GameEvent::BallHit { a, b, .. } = e {
                hit = Some((a, b));
            }
        }
        if hit.is_some() {
            break;
        }
    }
    assert_eq!(hit, Some((0, 3)));
    let cue = g.balls[0].vel;
    let obj = g.balls[3].vel;
    assert!(obj.x > 1.0, "object ball should move fast, got {obj:?}");
    assert!(obj.y.abs() < 1e-3);
    assert!(cue.length() < 0.1 * obj.length(), "cue {cue:?} obj {obj:?}");
}

#[test]
fn cushion_bounce() {
    let mut g = empty_game(Vec2::new(0.6, 0.0));
    assert!(g.shoot(Vec2::X, 1.0));
    let mut bounces = 0;
    for _ in 0..1800 {
        let v_before = g.balls[0].vel;
        let ev = g.step(1.0 / 60.0);
        if ev
            .iter()
            .any(|e| matches!(e, GameEvent::Cushion { ball: 0, .. }))
        {
            if bounces == 0 {
                let v = g.balls[0].vel;
                assert!(v_before.x > 0.0);
                assert!(v.x < 0.0, "should travel back, got {v:?}");
                assert!(v.length() < v_before.length() * 0.8);
            }
            bounces += 1;
        }
        assert_inside_rails(&g);
        assert!(!g.balls[0].pocketed);
        if ev.contains(&GameEvent::Settled) {
            break;
        }
    }
    assert!(bounces >= 1);
    assert!(!g.is_moving());
}

#[test]
fn no_tunnelling_at_high_speed() {
    let mut g = empty_game(Vec2::new(-1.0, 0.1));
    place(&mut g, 5, Vec2::new(-0.85, 0.1));
    // Drive the cue ball faster than a normal shot to be sure.
    assert!(g.shoot(Vec2::X, 1.0));
    g.balls[0].vel = Vec2::new(6.0, 0.0);
    let ev = g.step(0.1);
    assert!(
        ev.iter()
            .any(|e| matches!(e, GameEvent::BallHit { a: 0, b: 5, .. }))
    );
    assert!(g.balls[0].pos.x < g.balls[5].pos.x);
    assert!(g.balls[5].vel.x > 4.0);
    assert_no_overlaps(&g, 1e-5);
}

#[test]
fn corner_pocket() {
    let t = Table::standard();
    let mut g = empty_game(Vec2::new(-t.half_length / 2.0, 0.0));
    let start = Vec2::new(0.6, 0.2);
    place(&mut g, 4, start);
    let corner = t.pockets[5];
    // Cue ball behind ball 4 on the line to the corner.
    let dir = (corner - start).normalize();
    g.balls[0].pos = start - dir * 0.3;
    assert!(g.shoot(dir, 0.6));
    let (ev, _) = run(&mut g, 30.0);
    assert!(ev.contains(&GameEvent::Pocketed { ball: 4 }), "{ev:?}");
    assert!(g.balls[4].pocketed);
    assert_eq!(g.balls[4].vel, Vec2::ZERO);
    assert_eq!(g.potted, 1);
    assert!(ev.contains(&GameEvent::Settled));
}

#[test]
fn scratch_respots_cue() {
    let t = Table::standard();
    let start = Vec2::new(-0.8, -0.3);
    let mut g = empty_game(start);
    let dir = (t.pockets[0] - start).normalize();
    assert!(g.shoot(dir, 0.5));
    let (ev, _) = run(&mut g, 30.0);
    let scratch = ev.iter().position(|e| *e == GameEvent::Scratch);
    let settled = ev.iter().position(|e| *e == GameEvent::Settled);
    assert!(scratch.is_some() && settled.is_some(), "{ev:?}");
    assert!(scratch < settled);
    assert_eq!(g.potted, 0);
    assert!(!g.balls[0].pocketed);
    assert_eq!(g.balls[0].pos, Vec2::new(-t.half_length / 2.0, 0.0));
    assert!(g.can_shoot());
}

#[test]
fn respot_nudges_when_occupied() {
    let t = Table::standard();
    let mut g = empty_game(Vec2::ZERO);
    let spot = Vec2::new(-t.half_length / 2.0, 0.0);
    place(&mut g, 2, spot);
    g.balls[0].pocketed = true;
    g.respot_cue();
    assert!(!g.balls[0].pocketed);
    assert!(g.balls[0].pos.y == 0.0);
    assert_no_overlaps(&g, 0.0);
}

#[test]
fn break_settles() {
    let mut g = PoolGame::new();
    assert!(g.shoot(Vec2::X, 1.0));
    assert!(!g.shoot(Vec2::X, 1.0), "cannot shoot while moving");
    let (ev, t) = run(&mut g, 30.0);
    assert!(ev.contains(&GameEvent::Settled), "did not settle in 30 s");
    assert!(t > 3.0, "break settled suspiciously fast: {t}s");
    assert_eq!(ev.iter().filter(|e| **e == GameEvent::Settled).count(), 1);
    assert!(matches!(ev[0], GameEvent::Cue { .. }));
    let hits = ev
        .iter()
        .filter(|e| matches!(e, GameEvent::BallHit { .. }))
        .count();
    assert!(hits > 15, "break should scatter the rack, {hits} hits");
    assert_no_overlaps(&g, 1e-4);
    assert_inside_rails(&g);
    assert!(!g.is_moving());
    assert!(g.can_shoot());
    assert_eq!(g.shots, 1);
    // Settled is emitted once only.
    assert!(g.step(1.0 / 60.0).is_empty());
}

#[test]
fn predict_matches_first_hit() {
    let mut g = empty_game(Vec2::new(-0.6, 0.0));
    place(&mut g, 7, Vec2::new(0.0, 0.03));
    place(&mut g, 12, Vec2::new(0.3, -0.02));
    place(&mut g, 2, Vec2::new(-0.2, 0.25));
    let dir = Vec2::new(1.0, 0.02).normalize();
    let p = g.predict(dir);
    assert_eq!(p.target, Some(7));
    let td = p.target_dir.unwrap();
    assert!((td.length() - 1.0).abs() < 1e-5);
    assert!(
        (p.ghost.distance(g.balls[7].pos) - 2.0 * BALL_RADIUS).abs() < 1e-4,
        "ghost should touch target"
    );

    assert!(g.shoot(dir, 0.5));
    let mut first = None;
    'outer: for _ in 0..600 {
        for e in g.step(1.0 / 60.0) {
            if let GameEvent::BallHit { a, b, .. } = e {
                first = Some(if a == 0 { b } else { a });
                break 'outer;
            }
        }
    }
    assert_eq!(first, p.target);
    // Object ball travels roughly along the predicted direction.
    let v = g.balls[7].vel.normalize();
    assert!(v.dot(td) > 0.999, "v {v:?} predicted {td:?}");
}

#[test]
fn predict_cushion_when_clear() {
    let g = empty_game(Vec2::new(0.0, 0.0));
    let p = g.predict(Vec2::new(0.0, 1.0));
    assert_eq!(p.target, None);
    assert_eq!(p.target_dir, None);
    assert!((p.ghost.y - (g.table.half_width - BALL_RADIUS)).abs() < 1e-5);
}

#[test]
fn determinism() {
    let shoot = || {
        let mut g = PoolGame::new();
        g.shoot(Vec2::new(1.0, 0.013), 0.95);
        // Irregular frame times must not matter either.
        let mut frames = [0.016f32, 0.033, 0.007, 0.05].iter().cycle();
        let mut t = 0.0;
        while t < 30.0 {
            let dt = *frames.next().unwrap();
            t += dt;
            if g.step(dt).contains(&GameEvent::Settled) {
                break;
            }
        }
        g
    };
    let a = shoot();
    let b = shoot();
    for (x, y) in a.balls.iter().zip(&b.balls) {
        assert_eq!(x.pos, y.pos);
        assert_eq!(x.pocketed, y.pocketed);
    }
    assert_eq!(a.potted, b.potted);
}

#[test]
fn shoot_rejects_when_cue_pocketed() {
    let mut g = PoolGame::new();
    g.balls[0].pocketed = true;
    assert!(!g.can_shoot());
    assert!(!g.shoot(Vec2::X, 1.0));
    assert_eq!(g.shots, 0);
}
