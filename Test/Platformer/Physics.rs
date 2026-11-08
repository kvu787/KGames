// The movement, in fixed steps: jumping only from the ground, the buffered
// jump firing on the very step of the landing, coyote time, the jump cut,
// the spring, and the moving platform that carries the player.

use platformer::controls::Controls;
use platformer::game::Game;
use platformer::level::{self, Zone};
use platformer::physics::{BUFFER, COYOTE, JUMP_SPEED, STEP};
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

// A press just before the landing jumps on the very step of the landing.
#[test]
fn a_buffered_jump_fires_on_the_landing_step() {
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

// One fixed step at a time: STEP never varies, so a frame's work is a whole
// number of steps and the clock never drifts from the step count.
#[test]
fn the_clock_moves_in_whole_steps() {
    let mut game = game("Clock");
    let before = game.simulated;
    frame(&mut game, Controls::default());
    let steps = (game.simulated - before) / STEP;
    assert!((steps - steps.round()).abs() < 1e-9, "{steps}");
}
