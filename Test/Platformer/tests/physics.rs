// The movement, a step a frame by the frame's own time: each frame moving
// the player by its time, at 100 frames a second and at uneven ones, a jump
// as high at any frame rate, jumping only from the ground, the buffered jump
// firing on the very frame of the landing, coyote time, the jump cut, the
// spring, the moving platform that carries the player, and every top in
// every level holding a player who runs along it.

use platformer::controls::Controls;
use platformer::game::Game;
use platformer::level::{self, Zone};
use platformer::physics::{self, BUFFER, COYOTE, FASTEST_FALL, GRAVITY, HALF, JUMP_SPEED, LONGEST_FRAME, Player, RUN_SPEED, Solid, Standing};
use platformer::progress::Progress;

const FRAME: f64 = 1.0 / 60.0;

fn game(test: &str) -> Game {
    let folder = std::env::temp_dir().join("PlatformerPhysicsTests").join(test);
    let _ = std::fs::remove_dir_all(&folder);
    let mut game = Game::new(level::levels(), Progress::new(2), folder.join("Save.json"));
    for frame in 1..=5 {
        game.update(&Controls::default(), frame as f64 * FRAME);
    }
    assert!(game.player.grounded);
    game
}

fn frame(game: &mut Game, controls: Controls) {
    let next = game.simulated + FRAME;
    game.update(&controls, next);
}

// Running left, toward the first ground's open left edge.
const RUN_LEFT: Controls = Controls {
    push: -1.0,
    jump_pressed: false,
    jump_held: false,
    restart: false,
    level: 0,
    toggle_cards: false,
};

#[test]
fn a_jump_rises_on_the_frame_of_its_press() {
    let mut game = game("Jump");
    let ground = game.player.position[1];
    frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
    assert!(game.player.velocity[1] > 0.0);
    assert!(game.player.position[1] > ground, "already higher on the press's frame");
    assert!(!game.player.grounded);
}

#[test]
fn a_jump_needs_the_ground_or_a_fresh_ledge() {
    let mut game = game("Air");
    // Jump once, then press again mid-air: nothing.
    frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
    let rising = game.player.velocity[1];
    assert!(rising > 0.0);
    frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
    assert!(game.player.velocity[1] < rising, "the second press did nothing but fall along");
}

// A press just before the landing jumps on the very frame of the landing,
// which rises from the next.
#[test]
fn a_buffered_jump_fires_on_the_landing_frame() {
    let mut game = game("Buffer");
    // A short hop without holding, so the fall starts soon.
    frame(&mut game, Controls { jump_pressed: true, ..Controls::default() });
    // Fall until just above the ground, then press while still in the air.
    while game.player.velocity[1] > -3.0 {
        frame(&mut game, Controls::default());
    }
    assert!(!game.player.grounded);
    game.player.jump_until = game.simulated + BUFFER;
    // Within the buffer the landing itself relaunches: watch for a frame
    // where the player is airborne RISING without a press this frame.
    let mut relaunched = false;
    for _ in 0..30 {
        frame(&mut game, Controls::default());
        if game.player.velocity[1] > JUMP_SPEED - 2.0 {
            relaunched = true;
            break;
        }
    }
    assert!(relaunched, "the buffered press jumped on the landing, {:?}", game.player);
}

#[test]
fn coyote_time_jumps_after_the_ledge_but_not_much_after() {
    {
        let mut game = game("Coyote");
        // Walk off the ground's open left edge.
        while game.player.grounded {
            frame(&mut game, RUN_LEFT);
        }
        // Just off the ledge: a press inside the coyote window still jumps.
        assert!(game.simulated <= game.player.coyote_until);
        frame(&mut game, Controls { jump_pressed: true, jump_held: true, push: -1.0, ..Controls::default() });
        assert!(game.player.velocity[1] > 0.0, "the coyote jump fired, {:?}", game.player);
    }

    // Do it again, but wait the window out: the press does nothing.
    let mut game = game("CoyoteLate");
    while game.player.grounded {
        frame(&mut game, RUN_LEFT);
    }
    let frames_past = (COYOTE / FRAME).ceil() as usize + 1;
    for _ in 0..frames_past {
        frame(&mut game, Controls::default());
    }
    frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
    assert!(game.player.velocity[1] < 0.0, "too late: still falling, {:?}", game.player);
}

#[test]
fn letting_go_of_jump_cuts_the_rise() {
    let full = {
        let mut game = game("FullJump");
        let mut top = game.player.position[1];
        frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
        for _ in 0..60 {
            frame(&mut game, Controls { jump_held: true, ..Controls::default() });
            top = top.max(game.player.position[1]);
        }
        top
    };
    let cut = {
        let mut game = game("CutJump");
        let mut top = game.player.position[1];
        frame(&mut game, Controls { jump_pressed: true, jump_held: true, ..Controls::default() });
        // Let go right away.
        for _ in 0..60 {
            frame(&mut game, Controls::default());
            top = top.max(game.player.position[1]);
        }
        top
    };
    assert!(cut < full - 0.5, "a tap hops lower: {cut} against {full}");
}

#[test]
fn a_moving_platform_carries_the_player() {
    let mut game = game("Carry");
    // Stand on level 1's patrol, which moves along X.
    let patrol = game.current().patrols[0];
    let zone = patrol.at(game.simulated);
    game.player.position = [zone.center()[0], zone.top() + 0.56];
    game.player.velocity = [0.0, 0.0];
    // Land on it.
    for _ in 0..10 {
        frame(&mut game, Controls::default());
        if game.player.grounded {
            break;
        }
    }
    assert!(game.player.grounded, "{:?}", game.player);
    let before = game.player.position[0];
    let platform_before = patrol.at(game.simulated).x;
    for _ in 0..30 {
        frame(&mut game, Controls::default());
    }
    let platform_moved = patrol.at(game.simulated).x - platform_before;
    let player_moved = game.player.position[0] - before;
    assert!(platform_moved.abs() > 0.3, "the platform moved, {platform_moved}");
    assert!((player_moved - platform_moved).abs() < 0.05, "the player rode it: {player_moved} against {platform_moved}");
}

#[test]
fn walls_stop_the_run_where_they_stand() {
    let mut game = game("Wall");
    // Run right into the closed door.
    let door = game.current().doors[0];
    game.player.position = [door.x - 1.0, 0.55];
    game.player.velocity = [0.0, 0.0];
    let run = Controls { push: 1.0, ..Controls::default() };
    for _ in 0..30 {
        frame(&mut game, run);
    }
    assert!((game.player.position[0] - (door.x - 0.35)).abs() < 1e-9, "stopped at the door's face, {:?}", game.player.position);
}

// Every top in every level, the doors' and the patrols' at every 240th of a
// second of their way included, holds a player who stands on it and runs
// along it, both ways, through frames of uneven lengths. The player's box is stored as its center, and a center resting
// on a top doesn't always round back to it: 2.0 + 0.55 - 0.55 is
// 1.9999999999999998. That hair of overlap once made the run's X pass take
// the platform under the player for a wall, and push them back to its edge
// and off it: an invisible wall on First Steps' platform over the first pit
// and on Ridgeline's checkpoint block, the two tops at 2.0.
#[test]
fn every_top_holds_a_runner_both_ways() {
    let mut tops: Vec<(String, Zone)> = Vec::new();
    for level in level::levels() {
        for (index, &zone) in level.solids.iter().enumerate() {
            tops.push((format!("{} solid {index}", level.name), zone));
        }
        for (index, &zone) in level.doors.iter().enumerate() {
            tops.push((format!("{} door {index}", level.name), zone));
        }
        for (index, patrol) in level.patrols.iter().enumerate() {
            let samples = (patrol.period * 240.0).round() as usize;
            for sample in 0..samples {
                let seconds = sample as f64 / 240.0;
                tops.push((format!("{} patrol {index} at {seconds} s", level.name), patrol.at(seconds)));
            }
        }
    }
    // A 60 Hz frame, a 240 Hz one, a 100 Hz one, and the longest.
    const FRAMES: [f64; 4] = [1.0 / 60.0, 1.0 / 240.0, 0.01, LONGEST_FRAME];
    for (name, zone) in tops {
        let solids = [Solid { zone, standing: Standing::Still }];
        for push in [1.0, -1.0] {
            // Where a landing puts the player: the top plus the half height.
            let mut player = Player::at([zone.center()[0], zone.top()]);
            let start = player.position;
            let mut now = 0.0;
            for seconds in FRAMES {
                now += seconds;
                physics::step(&mut player, now, seconds, push, false, [0.0, 0.0], &solids);
            }
            let ran = push * RUN_SPEED * FRAMES.iter().sum::<f64>();
            assert!(player.grounded, "{name}, running {push}: still on it, {player:?}");
            assert!(
                (player.position[0] - (start[0] + ran)).abs() < 1e-9 && player.position[1] == start[1],
                "{name}, running {push}: ran {ran} along the top from {start:?}, {player:?}"
            );
        }
    }
}

// The way the bug was found: jump from the first ground onto First Steps'
// platform over the pit, holding right, and keep running along it.
#[test]
fn a_runner_lands_on_the_platform_over_the_pit_and_runs_along_it() {
    let mut game = game("PitPlatform");
    let platform = game.current().solids[4];
    assert_eq!(platform, Zone::new(15.5, 1.5, 3.0, 0.5), "the platform over the first pit");
    game.player.position = [12.0, HALF[1]];
    game.player.velocity = [0.0, 0.0];
    frame(&mut game, Controls { push: 1.0, jump_pressed: true, jump_held: true, ..Controls::default() });
    let mut landed = false;
    let mut farthest = f64::NEG_INFINITY;
    for _ in 0..60 {
        frame(&mut game, Controls { push: 1.0, jump_held: true, ..Controls::default() });
        let on_top = game.player.grounded && game.player.position[1] == platform.top() + HALF[1];
        if on_top {
            landed = true;
            farthest = farthest.max(game.player.position[0]);
        } else if landed {
            break;
        }
    }
    assert!(landed, "landed on the platform, {:?}", game.player);
    assert!(farthest > platform.right(), "ran to its far end, not stopped on it: {farthest}");
}

// A patrol carries the player by how far it moved, which is rounded too.
// Ride Ridgeline's lift, which rises and sinks through heights that round
// every which way, for a whole period, running a frame each way in turn:
// the player stays on it, never pushed off it as if it were a wall.
#[test]
fn a_runner_rides_the_lift_through_every_height() {
    let mut game = game("Lift");
    game.level = 1;
    game.restart();
    let patrol = game.current().patrols[0];
    assert!(patrol.to[1] != patrol.zone.y && patrol.to[0] == patrol.zone.x, "Ridgeline's first patrol goes up and down");
    let zone = patrol.at(game.simulated);
    game.player.position = [zone.center()[0], zone.top() + HALF[1]];
    game.player.velocity = [0.0, 0.0];
    frame(&mut game, Controls::default());
    assert!(game.player.grounded, "{:?}", game.player);
    let frames = (patrol.period / FRAME).round() as usize;
    for index in 0..frames {
        let push = if index % 2 == 0 { 1.0 } else { -1.0 };
        frame(&mut game, Controls { push, ..Controls::default() });
        let zone = patrol.at(game.simulated);
        assert!(
            game.player.standing == Standing::Patrol(0) && (game.player.position[0] - zone.center()[0]).abs() < 0.2,
            "frame {index}: still riding the lift at {zone:?}, {:?}",
            game.player
        );
    }
}

#[test]
fn the_patrol_is_a_pure_function_of_time() {
    let patrol = level::levels()[0].patrols[0];
    let there = patrol.at(1.0);
    let again = patrol.at(1.0 + patrol.period * 7.0);
    assert!((there.x - again.x).abs() < 1e-9 && (there.y - again.y).abs() < 1e-9);
    // The wave reaches both ends.
    let start: Zone = patrol.at(0.0);
    let far: Zone = patrol.at(patrol.period / 2.0);
    assert!((start.x - patrol.zone.x).abs() < 1e-9);
    assert!((far.x - patrol.to[0]).abs() < 1e-9);
}

// Each frame moves the game by its own time, the time since the frame before,
// in one step, as the Godot version's _Process does with its delta: at 100
// frames a second, each frame runs the player 10 milliseconds' worth, and
// pulls a falling player down by 10 milliseconds of gravity, and so does a
// frame of any other length, up to LONGEST_FRAME. In fixed steps of 1/240 of
// a second, a 100 Hz frame held 2 or 3 of them, so the player moved by 8.3
// or 12.5 milliseconds' worth, and juddered.
#[test]
fn each_frame_moves_the_game_by_its_own_time() {
    let mut game = game("FrameTime");
    let run = Controls { push: 1.0, ..Controls::default() };
    let hundred_hertz = [0.01; 10];
    let uneven = [0.007, 0.013, 0.004, 0.016, 0.0101, 0.0069];
    let mut time = game.simulated;
    for frame_time in hundred_hertz.iter().chain(&uneven) {
        let (clock, x) = (game.simulated, game.player.position[0]);
        time += frame_time;
        game.update(&run, time);
        assert!((game.simulated - clock - frame_time).abs() < 1e-12, "the clock moved {}", game.simulated - clock);
        let ran = game.player.position[0] - x;
        assert!((ran - RUN_SPEED * frame_time).abs() < 1e-12, "a frame of {frame_time} seconds ran {ran}");
        assert!(game.player.grounded);
    }

    // In the air, each frame pulls the player down by its time's gravity.
    game.player.position[1] += 5.0;
    game.player.velocity[1] = 0.0;
    for frame_time in hundred_hertz.iter().chain(&uneven) {
        let speed = game.player.velocity[1];
        time += frame_time;
        game.update(&Controls::default(), time);
        assert!(!game.player.grounded);
        let pulled = speed - game.player.velocity[1];
        assert!((pulled - GRAVITY * frame_time).abs() < 1e-12, "a frame of {frame_time} seconds pulled {pulled}");
    }

    // A frame that comes late, like after the window was minimized, counts
    // LONGEST_FRAME.
    let (clock, y) = (game.simulated, game.player.position[1]);
    time += 0.5;
    game.update(&Controls::default(), time);
    assert!((game.simulated - clock - LONGEST_FRAME).abs() < 1e-12, "the late frame moved the clock {}", game.simulated - clock);
    let fell = y - game.player.position[1];
    assert!(fell > 0.0 && fell <= FASTEST_FALL * LONGEST_FRAME + 1e-12, "fell {fell}, no faster than the fastest fall");
}

// Gravity's pull is exact, so a held jump peaks at JUMP_SPEED squared over
// twice GRAVITY above the ground, at any frame rate, instead of lower at
// lower ones.
#[test]
fn a_jump_reaches_the_same_height_at_any_frame_rate() {
    let peak = JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY);
    for rate in [20.0, 30.0, 60.0, 100.0, 144.0, 240.0] {
        let mut game = game(&format!("Peak{rate}"));
        let ground = game.player.position[1];
        let mut time = game.simulated;
        let mut top = ground;
        let mut held = Controls { jump_pressed: true, jump_held: true, ..Controls::default() };
        for _ in 0..(rate as usize) {
            time += 1.0 / rate;
            game.update(&held, time);
            held.jump_pressed = false;
            top = top.max(game.player.position[1]);
        }
        // The frames sample the arc, so the highest of them is within an
        // eighth of a frame's gravity of the peak.
        let sampling = GRAVITY / (rate * rate) / 8.0;
        assert!(
            top - ground <= peak + 1e-9 && top - ground >= peak - sampling - 1e-9,
            "at {rate} frames a second, the jump reached {} of {peak}",
            top - ground
        );
    }
}
