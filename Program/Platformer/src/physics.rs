// How the player moves, a step a frame, by the frame's own time: run,
// gravity, jump, and axis-separated collision against the solids, where a
// box resting against a face, less than SKIN into it, touches it without
// colliding. Jumps are forgiving the way platformers are: a press up to
// BUFFER seconds before landing jumps on the very frame of the landing, and
// a press up to COYOTE seconds after walking off a ledge still jumps. All of
// it runs on the game's clock, which is the frames' times added up.
//
// There are no fixed steps, which a frame rate they don't divide would fit a
// varying number of into its frames, so that the player would judder. The
// fall moves by the average of the speeds before and after gravity's pull,
// which is exact for a constant pull, so a jump reaches the same height at
// any frame rate.

use crate::level::Zone;

// The longest frame, in seconds, that moves the game by its time: a frame
// that comes later than this, like after the window was minimized, moves it
// by this much. At the fastest fall, a frame this long moves the player 1.2
// units, less than the thinnest platform, 0.5, and the player's height, 1.1,
// together, so that no frame carries the player through a platform.
pub const LONGEST_FRAME: f64 = 0.05;
// How fast the player runs, in units a second.
pub const RUN_SPEED: f64 = 8.0;
pub const GRAVITY: f64 = 36.0;
// The jump's upward speed, which reaches about 2.5 units.
pub const JUMP_SPEED: f64 = 13.4;
// Letting go of jump while rising cuts the rise to this, for short hops.
pub const JUMP_CUT: f64 = 4.0;
// A spring's upward speed, which reaches about 5 units.
pub const SPRING_SPEED: f64 = 19.0;
pub const FASTEST_FALL: f64 = 24.0;
pub const COYOTE: f64 = 0.1;
pub const BUFFER: f64 = 0.12;
// The player's box, as half-widths around its center.
pub const HALF: [f64; 2] = [0.35, 0.55];
// How deep the box has to be in a solid, on both axes, to collide with it.
// The box is stored as its center, and a center resolved against a face
// doesn't always round back to it: a player landed on a top at 2.0 is at
// 2.0 + 0.55, and less 0.55 again that's 1.9999999999999998. Without the
// skin, that hair inside the platform made the next step's run take it for a
// wall, and push the player back to its edge and off it. Rounding at the
// levels' sizes is around 1e-15, and a standing player's 60 Hz frame of
// gravity sinks the box 0.005 into the ground.
pub const SKIN: f64 = 1e-9;

// What the player stands on, so a moving platform can carry them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Standing {
    Still,
    Patrol(usize),
}

#[derive(Clone, Debug)]
pub struct Player {
    // The box's center.
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub grounded: bool,
    // What the player stands on, while grounded.
    pub standing: Standing,
    // Until when a jump press still counts, and until when the ground the
    // player left still does, on the simulation's clock.
    pub jump_until: f64,
    pub coyote_until: f64,
}

impl Player {
    // At the spawn point, which is the feet's middle.
    pub fn at(spawn: [f64; 2]) -> Player {
        Player {
            position: [spawn[0], spawn[1] + HALF[1]],
            velocity: [0.0, 0.0],
            grounded: false,
            standing: Standing::Still,
            jump_until: f64::NEG_INFINITY,
            coyote_until: f64::NEG_INFINITY,
        }
    }
}

// A solid the player can collide with this step, and what it is.
pub struct Solid {
    pub zone: Zone,
    pub standing: Standing,
}

// Whether the player's box at `center` is more than SKIN into a solid, on
// both axes.
pub fn collides(zone: &Zone, center: [f64; 2]) -> bool {
    center[0] + HALF[0] > zone.x + SKIN
        && center[0] - HALF[0] < zone.right() - SKIN
        && center[1] + HALF[1] > zone.y + SKIN
        && center[1] - HALF[1] < zone.top() - SKIN
}

// Jumps, from the ground or from a ledge just left, at `now` on the game's
// clock, if a press still counts.
fn jump(player: &mut Player, now: f64) {
    if player.jump_until >= now && (player.grounded || player.coyote_until >= now) && player.velocity[1] <= 0.0 {
        player.velocity[1] = JUMP_SPEED;
        player.grounded = false;
        player.standing = Standing::Still;
        player.jump_until = f64::NEG_INFINITY;
        player.coyote_until = f64::NEG_INFINITY;
    }
}

// One frame's movement against the solids, `seconds` long, which ends at
// `now` on the game's clock, with the stick's push and the jump's state. The
// solids are where they are at the frame's end, AFTER the platforms moved,
// and `carry` is how far the platform the player stood on moved, which moves
// the player first.
pub fn step(player: &mut Player, now: f64, seconds: f64, push: f64, jump_held: bool, carry: [f64; 2], solids: &[Solid]) {
    // A press jumps from the ground the player stood on as the frame began,
    // so that the frame of the press already rises.
    jump(player, now - seconds);

    // The platform the player stands on carries them.
    player.position[0] += carry[0];
    player.position[1] += carry[1];

    // Run, and resolve along X: walls stop the player where they stand.
    player.velocity[0] = push * RUN_SPEED;
    player.position[0] += player.velocity[0] * seconds;
    for solid in solids {
        if collides(&solid.zone, player.position) {
            if player.velocity[0] > 0.0 {
                player.position[0] = solid.zone.x - HALF[0];
            } else if player.velocity[0] < 0.0 {
                player.position[0] = solid.zone.right() + HALF[0];
            }
        }
    }

    // Fall, with the rise cut short once jump is let go of, by the average
    // of the speeds before and after gravity's pull.
    if !jump_held && player.velocity[1] > JUMP_CUT {
        player.velocity[1] = JUMP_CUT;
    }
    let rising = player.velocity[1];
    player.velocity[1] = (rising - GRAVITY * seconds).max(-FASTEST_FALL);
    player.position[1] += (rising + player.velocity[1]) / 2.0 * seconds;

    // Resolve along Y: land on tops, bump heads on bottoms.
    let was_falling = player.velocity[1] <= 0.0;
    player.grounded = false;
    player.standing = Standing::Still;
    for solid in solids {
        if collides(&solid.zone, player.position) {
            if was_falling {
                player.position[1] = solid.zone.top() + HALF[1];
                player.velocity[1] = 0.0;
                player.grounded = true;
                player.standing = solid.standing;
            } else {
                player.position[1] = solid.zone.y - HALF[1];
                player.velocity[1] = 0.0;
            }
        }
    }

    // The ground just left still counts for a moment, and a press just
    // before landing jumps on the frame of the landing, which rises from
    // the next.
    if player.grounded {
        player.coyote_until = now + COYOTE;
    }
    jump(player, now);
}
