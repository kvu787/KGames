// How the player moves, in fixed steps of STEP seconds: run, gravity, jump,
// and axis-separated collision against the solids. Jumps are forgiving the
// way platformers are: a press up to BUFFER seconds before landing jumps on
// the very step of the landing, and a press up to COYOTE seconds after
// walking off a ledge still jumps. All of it runs on the simulation's clock,
// so a run is the same at any frame rate.

use crate::level::Zone;

// The fixed step, a 240th of a second: four steps in a 60 Hz frame.
pub const STEP: f64 = 1.0 / 240.0;
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

// One fixed step of movement against the solids, at `now` on the simulation's
// clock, with the stick's push and the jump's state. The solids are where
// they are AFTER the platforms moved this step, and `carry` is how far the
// platform the player stood on moved, which moves the player first.
pub fn step(player: &mut Player, now: f64, push: f64, jump_held: bool, carry: [f64; 2], solids: &[Solid]) {
    // The platform the player stands on carries them.
    player.position[0] += carry[0];
    player.position[1] += carry[1];

    // Run, and resolve along X: walls stop the player where they stand.
    player.velocity[0] = push * RUN_SPEED;
    player.position[0] += player.velocity[0] * STEP;
    for solid in solids {
        if solid.zone.overlaps(player.position, HALF) {
            if player.velocity[0] > 0.0 {
                player.position[0] = solid.zone.x - HALF[0];
            } else if player.velocity[0] < 0.0 {
                player.position[0] = solid.zone.right() + HALF[0];
            }
        }
    }

    // Fall, with the rise cut short once jump is let go of.
    player.velocity[1] -= GRAVITY * STEP;
    if !jump_held && player.velocity[1] > JUMP_CUT {
        player.velocity[1] = JUMP_CUT;
    }
    if player.velocity[1] < -FASTEST_FALL {
        player.velocity[1] = -FASTEST_FALL;
    }
    player.position[1] += player.velocity[1] * STEP;

    // Resolve along Y: land on tops, bump heads on bottoms.
    let was_falling = player.velocity[1] <= 0.0;
    player.grounded = false;
    player.standing = Standing::Still;
    for solid in solids {
        if solid.zone.overlaps(player.position, HALF) {
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
    // before landing jumps on the step of the landing.
    if player.grounded {
        player.coyote_until = now + COYOTE;
    }
    if player.jump_until >= now && (player.grounded || player.coyote_until >= now) && player.velocity[1] <= 0.0 {
        player.velocity[1] = JUMP_SPEED;
        player.grounded = false;
        player.standing = Standing::Still;
        player.jump_until = f64::NEG_INFINITY;
        player.coyote_until = f64::NEG_INFINITY;
    }
}
