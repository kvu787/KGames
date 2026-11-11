// The levels: rectangles in world units, +X right and +Y up, with one tile
// 1 unit. Platforms to stand on, coins to collect, springs that bounce,
// spikes that kill, checkpoints that move the respawn, a door that opens
// once every coin is collected, and the goal behind it. The player's jump
// reaches about 2.5 units, so anything higher needs a spring or a step.

// An axis-aligned rectangle in world units, from its bottom left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Zone {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Zone {
        Zone { x, y, width, height }
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn top(&self) -> f64 {
        self.y + self.height
    }

    pub fn center(&self) -> [f64; 2] {
        [self.x + self.width / 2.0, self.y + self.height / 2.0]
    }

    // Whether a centered box overlaps it.
    pub fn overlaps(&self, center: [f64; 2], half: [f64; 2]) -> bool {
        center[0] + half[0] > self.x && center[0] - half[0] < self.right() && center[1] + half[1] > self.y && center[1] - half[1] < self.top()
    }
}

// A platform that patrols between where its zone is and `to`, there and back
// over `period` seconds, carrying whoever stands on it. Its place is a pure
// function of the game's time, so it's where it should be at any frame rate.
#[derive(Clone, Copy, Debug)]
pub struct Patrol {
    pub zone: Zone,
    pub to: [f64; 2],
    pub period: f64,
}

impl Patrol {
    // Where the platform is at a time, as a zone.
    pub fn at(&self, seconds: f64) -> Zone {
        let phase = (seconds / self.period).rem_euclid(1.0);
        // A triangle wave: out over the first half, back over the second.
        let along = if phase < 0.5 { phase * 2.0 } else { 2.0 - phase * 2.0 };
        Zone {
            x: self.zone.x + (self.to[0] - self.zone.x) * along,
            y: self.zone.y + (self.to[1] - self.zone.y) * along,
            ..self.zone
        }
    }
}

// An sRGB color, a byte a channel, for the level's own look.
pub type LevelColor = [u8; 3];

pub struct Level {
    pub name: &'static str,
    // Where the player starts, as the feet's middle on the ground.
    pub spawn: [f64; 2],
    // What the camera stays inside. Falling below its bottom is a death.
    pub bounds: Zone,
    pub sky: LevelColor,
    pub rock: LevelColor,
    pub solids: Vec<Zone>,
    pub coins: Vec<[f64; 2]>,
    pub spikes: Vec<Zone>,
    pub springs: Vec<Zone>,
    pub checkpoints: Vec<Zone>,
    // Solid until every coin is collected.
    pub doors: Vec<Zone>,
    pub goal: Zone,
    pub patrols: Vec<Patrol>,
}

pub fn levels() -> Vec<Level> {
    vec![first_steps(), ridgeline()]
}

// Level 1: one of everything. The last coin sits right at the door, so
// collecting it can open the door and win the run in the same breath.
fn first_steps() -> Level {
    Level {
        name: "First Steps",
        spawn: [1.5, 0.0],
        bounds: Zone::new(-2.0, -4.0, 68.0, 24.0),
        sky: [0x8E, 0xC9, 0xE8],
        rock: [0x4A, 0x55, 0x68],
        solids: vec![
            // The ground, with a pit at 14..19 and another at 44..52.
            Zone::new(-2.0, -2.0, 16.0, 2.0),
            Zone::new(19.0, -2.0, 25.0, 2.0),
            Zone::new(52.0, -2.0, 14.0, 2.0),
            // A step, a floating platform over the first pit, and a ledge
            // that only the spring reaches.
            Zone::new(9.0, 0.0, 2.0, 1.0),
            Zone::new(15.5, 1.5, 3.0, 0.5),
            Zone::new(24.0, 2.5, 3.0, 0.5),
        ],
        coins: vec![[5.5, 0.7], [17.0, 2.7], [25.5, 3.8], [40.0, 0.7], [57.0, 0.7]],
        spikes: vec![Zone::new(31.0, 0.0, 2.0, 0.6)],
        springs: vec![Zone::new(21.5, 0.0, 1.2, 0.4)],
        checkpoints: vec![Zone::new(36.0, 0.0, 0.8, 1.6)],
        doors: vec![Zone::new(58.0, 0.0, 0.8, 3.0)],
        // The goal starts at the door's face: strictly behind it while the
        // door is closed, and right past it once the door opens, so a run
        // that the closed door stopped at the doorstep, where the last coin
        // opens it, reaches the goal on the next frame.
        goal: Zone::new(58.0, 0.0, 2.8, 3.0),
        patrols: vec![Patrol {
            zone: Zone::new(44.0, -0.5, 3.0, 0.5),
            to: [49.0, -0.5],
            period: 4.0,
        }],
    }
}

// Level 2: longer, higher, and meaner, with two checkpoints.
fn ridgeline() -> Level {
    Level {
        name: "Ridgeline",
        spawn: [1.5, 0.0],
        bounds: Zone::new(-2.0, -4.0, 92.0, 28.0),
        sky: [0xF2, 0xC1, 0x8C],
        rock: [0x5C, 0x4A, 0x52],
        solids: vec![
            Zone::new(-2.0, -2.0, 14.0, 2.0),
            Zone::new(16.0, -1.0, 6.0, 1.0),
            Zone::new(25.0, 0.0, 8.0, 2.0),
            Zone::new(36.0, -2.0, 18.0, 2.0),
            Zone::new(57.0, -1.0, 5.0, 1.5),
            Zone::new(65.0, -2.0, 25.0, 2.0),
            // Steps up the ridge.
            Zone::new(40.0, 0.0, 2.0, 1.0),
            Zone::new(44.0, 1.0, 2.0, 1.5),
            Zone::new(48.0, 3.8, 3.0, 0.5),
            // A high shelf over the last stretch.
            Zone::new(72.0, 2.8, 4.0, 0.5),
        ],
        coins: vec![
            [5.0, 0.7],
            [18.5, 0.7],
            [29.0, 2.7],
            [45.0, 3.2],
            [49.5, 5.0],
            [59.5, 1.2],
            [74.0, 4.0],
        ],
        spikes: vec![Zone::new(36.5, 0.0, 2.5, 0.6), Zone::new(67.0, 0.0, 2.0, 0.6)],
        springs: vec![Zone::new(47.0, 0.0, 1.2, 0.4), Zone::new(70.5, 0.0, 1.2, 0.4)],
        checkpoints: vec![Zone::new(27.0, 2.0, 0.8, 1.6), Zone::new(55.0, 0.0, 0.8, 1.6)],
        doors: vec![Zone::new(84.0, 0.0, 0.8, 3.0)],
        goal: Zone::new(84.0, 0.0, 3.0, 3.0),
        patrols: vec![
            Patrol {
                zone: Zone::new(14.5, -0.5, 1.5, 0.5),
                to: [14.5, 2.5],
                period: 3.0,
            },
            Patrol {
                zone: Zone::new(62.0, -0.5, 3.0, 0.5),
                to: [62.0, -0.5],
                period: 1.0,
            },
        ],
    }
}
