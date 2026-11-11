// The game: a run through a level, publishing an Event for everything that
// happens. Each frame moves the world by the frame's own time, in one step,
// detects what the movement did - coins touched, springs landed on, spikes
// hit, the goal reached - and then drains the event queue to quiescence:
// every consumer runs, and whatever they publish runs too. So a frame's
// input produces the whole graph of its consequences inside that frame,
// however deep it goes, and the view only ever describes settled state.

use std::collections::VecDeque;
use std::path::PathBuf;

use json::Value;

use crate::controls::Controls;
use crate::events::{Cause, Event, MOST_EVENTS};
use crate::level::{Level, Zone};
use crate::physics::{self, BUFFER, HALF, LONGEST_FRAME, Player, SPRING_SPEED, Solid, Standing};
use crate::progress::Progress;

// Lives per run.
pub const LIVES: u32 = 3;
// How many recent events the log keeps for the Events card.
const LOG_LENGTH: usize = 40;

// Where a run stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    Running,
    // Finished, in this many seconds.
    Finished(f64),
    // Out of lives.
    Failed,
}

// A moment drawn over the world for a while, like a coin's sparkle. State,
// not an action: spawned by an event's consumer, timed on the game's clock,
// pruned once over, drawn by the view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectKind {
    Sparkle,
    Burst,
    Bounce,
    Flag,
    Poof,
    Shimmer,
    Confetti,
    SaveFlash,
}

impl EffectKind {
    pub fn life(self) -> f64 {
        match self {
            EffectKind::Confetti => 2.0,
            EffectKind::SaveFlash => 1.2,
            _ => 0.6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    pub kind: EffectKind,
    pub at: [f64; 2],
    pub started: f64,
}

pub struct Game {
    pub levels: Vec<Level>,
    pub level: usize,
    pub progress: Progress,
    save_path: PathBuf,
    // The run.
    pub player: Player,
    pub collected: Vec<bool>,
    pub coins: usize,
    pub lives: u32,
    pub doors_open: Vec<bool>,
    pub reached: Vec<bool>,
    pub checkpoint: Option<usize>,
    pub state: RunState,
    // When the run started, on the game's clock.
    pub started: f64,
    // The game's clock: the frames' times added up, each at most
    // LONGEST_FRAME, from 0 as the game starts.
    pub simulated: f64,
    // The last frame's time, on the frames' clock, which update is given,
    // and which starts at 0 too.
    last: f64,
    // The events: the queue that drains inside the frame, the log the Events
    // card shows, and how many drained this frame, against the cap.
    queue: VecDeque<Event>,
    pub log: VecDeque<(f64, String)>,
    drained_this_frame: usize,
    pub effects: Vec<Effect>,
    // What the last save said, for the status bar.
    pub status: String,
    pub show_cards: bool,
}

impl Game {
    pub fn new(levels: Vec<Level>, progress: Progress, save_path: PathBuf) -> Game {
        let level = &levels[0];
        let mut game = Game {
            player: Player::at(level.spawn),
            collected: vec![false; level.coins.len()],
            coins: 0,
            lives: LIVES,
            doors_open: vec![false; level.doors.len()],
            reached: vec![false; level.checkpoints.len()],
            checkpoint: None,
            state: RunState::Running,
            started: 0.0,
            simulated: 0.0,
            last: 0.0,
            queue: VecDeque::new(),
            log: VecDeque::new(),
            drained_this_frame: 0,
            effects: Vec::new(),
            status: String::new(),
            show_cards: true,
            levels,
            level: 0,
            progress,
            save_path,
        };
        game.publish(Event::RunStarted);
        game.drain();
        game
    }

    pub fn current(&self) -> &Level {
        &self.levels[self.level]
    }

    pub fn total_coins(&self) -> usize {
        self.current().coins.len()
    }

    // The run's clock: how long it took, or has taken so far.
    pub fn run_seconds(&self) -> f64 {
        match self.state {
            RunState::Finished(seconds) => seconds,
            _ => self.simulated - self.started,
        }
    }

    // Starts the current level over: a fresh run on the same clock.
    pub fn restart(&mut self) {
        let level = &self.levels[self.level];
        self.player = Player::at(level.spawn);
        self.collected = vec![false; level.coins.len()];
        self.coins = 0;
        self.lives = LIVES;
        self.doors_open = vec![false; level.doors.len()];
        self.reached = vec![false; level.checkpoints.len()];
        self.checkpoint = None;
        self.state = RunState::Running;
        self.started = self.simulated;
        self.effects.clear();
        self.publish(Event::RunStarted);
        self.drain();
    }

    // Goes to an unlocked level and starts a run there.
    pub fn go_to_level(&mut self, level: usize) {
        if level < self.progress.unlocked.min(self.levels.len()) && level != self.level {
            self.level = level;
            self.restart();
        }
    }

    // A frame, at `seconds` on the frames' clock: the frame's one-off
    // controls, then one step by the frame's time, the time since the frame
    // before, at most LONGEST_FRAME, then its events, drained to quiescence.
    pub fn update(&mut self, controls: &Controls, seconds: f64) {
        self.drained_this_frame = 0;
        let frame = (seconds - self.last).clamp(0.0, LONGEST_FRAME);
        self.last = seconds;
        if controls.toggle_cards {
            self.show_cards = !self.show_cards;
        }
        if controls.restart {
            self.restart();
        }
        if controls.level != 0 {
            let target = self.level as i64 + i64::from(controls.level);
            if target >= 0 && (target as usize) < self.progress.unlocked.min(self.levels.len()) {
                self.level = target as usize;
                self.restart();
            }
        }
        if controls.jump_pressed {
            // The press counts for a moment, from the frame's start, so one
            // just before a landing jumps on the very frame of the landing.
            self.player.jump_until = self.simulated + BUFFER;
        }
        self.simulated += frame;
        self.step(controls, frame);
        self.drain();
        let simulated = self.simulated;
        self.effects.retain(|effect| effect.started + effect.kind.life() > simulated);
    }

    // One frame's step, `seconds` long: the platforms move, the player moves
    // against them, and what the movement did becomes events.
    fn step(&mut self, controls: &Controls, seconds: f64) {
        if self.state != RunState::Running {
            return;
        }
        let now = self.simulated;
        let level = &self.levels[self.level];

        // The platform the player stood on carries them by how far it moved.
        let carry = match self.player.standing {
            Standing::Patrol(patrol) => {
                let before = level.patrols[patrol].at(now - seconds);
                let after = level.patrols[patrol].at(now);
                [after.x - before.x, after.y - before.y]
            }
            Standing::Still => [0.0, 0.0],
        };

        // What's solid this frame: the ground, the doors still closed, and
        // the platforms where they are now.
        let mut solids: Vec<Solid> = level.solids.iter().map(|&zone| Solid { zone, standing: Standing::Still }).collect();
        for (door, &zone) in level.doors.iter().enumerate() {
            if !self.doors_open[door] {
                solids.push(Solid { zone, standing: Standing::Still });
            }
        }
        for (patrol, moving) in level.patrols.iter().enumerate() {
            solids.push(Solid {
                zone: moving.at(now),
                standing: Standing::Patrol(patrol),
            });
        }

        physics::step(&mut self.player, now, seconds, controls.push, controls.jump_held, carry, &solids);

        // What the movement did, as events. Each check reads the state as
        // the last drain left it, so nothing is published twice.
        let position = self.player.position;
        let falling = self.player.velocity[1] < -1.0;
        let mut found = Vec::new();
        for (coin, &at) in level.coins.iter().enumerate() {
            if !self.collected[coin] && Zone::new(at[0] - 0.3, at[1] - 0.3, 0.6, 0.6).overlaps(position, HALF) {
                found.push(Event::CoinCollected { coin });
            }
        }
        for (spring, &zone) in level.springs.iter().enumerate() {
            if falling && zone.overlaps(position, HALF) {
                found.push(Event::SpringBounced { spring });
            }
        }
        for (checkpoint, &zone) in level.checkpoints.iter().enumerate() {
            if !self.reached[checkpoint] && zone.overlaps(position, HALF) {
                found.push(Event::CheckpointReached { checkpoint });
            }
        }
        if level.goal.overlaps(position, HALF) {
            found.push(Event::GoalReached);
        }
        let on_spikes = level.spikes.iter().any(|zone| {
            // A touch of the points themselves, not a brush past the base.
            let shrunk = Zone::new(zone.x + 0.1, zone.y, zone.width - 0.2, zone.height - 0.1);
            shrunk.overlaps(position, HALF)
        });
        if on_spikes {
            found.push(Event::Died { cause: Cause::Spikes });
        } else if position[1] < level.bounds.y {
            found.push(Event::Died { cause: Cause::Fell });
        }
        for event in found {
            self.publish(event);
        }
    }

    fn publish(&mut self, event: Event) {
        self.queue.push_back(event);
    }

    // Runs every queued event, and everything they publish, now. A cascade
    // that doesn't settle within MOST_EVENTS in one frame is a bug in the
    // event graph, and stops the program rather than hiding.
    fn drain(&mut self) {
        while let Some(event) = self.queue.pop_front() {
            self.drained_this_frame += 1;
            assert!(
                self.drained_this_frame <= MOST_EVENTS,
                "the event cascade didn't settle: {} events in one frame, last {}",
                self.drained_this_frame,
                event.label()
            );
            self.log.push_front((self.simulated, event.label()));
            self.log.truncate(LOG_LENGTH);
            self.apply(event);
        }
    }

    // One event's consequences: state changes, effects, and more events.
    // Facts ride in the event; everything else is read from the game as it
    // is now, never from a snapshot.
    fn apply(&mut self, event: Event) {
        match event {
            Event::RunStarted => {}
            Event::CoinCollected { coin } => {
                if !self.collected[coin] {
                    self.collected[coin] = true;
                    self.coins += 1;
                    let at = self.levels[self.level].coins[coin];
                    self.effect(EffectKind::Sparkle, at);
                    if self.coins == self.total_coins() {
                        self.publish(Event::AllCoinsCollected);
                    }
                }
            }
            Event::AllCoinsCollected => {
                for door in 0..self.doors_open.len() {
                    if !self.doors_open[door] {
                        self.publish(Event::DoorOpened { door });
                    }
                }
            }
            Event::DoorOpened { door } => {
                self.doors_open[door] = true;
                let at = self.levels[self.level].doors[door].center();
                self.effect(EffectKind::Burst, at);
            }
            Event::SpringBounced { spring } => {
                self.player.velocity[1] = SPRING_SPEED;
                self.player.grounded = false;
                self.player.standing = Standing::Still;
                let at = self.levels[self.level].springs[spring].center();
                self.effect(EffectKind::Bounce, at);
            }
            Event::CheckpointReached { checkpoint } => {
                self.reached[checkpoint] = true;
                self.checkpoint = Some(checkpoint);
                let at = self.levels[self.level].checkpoints[checkpoint].center();
                self.effect(EffectKind::Flag, at);
            }
            Event::Died { cause: _ } => {
                if self.state == RunState::Running {
                    self.effect(EffectKind::Poof, self.player.position);
                    self.lives -= 1;
                    if self.lives > 0 {
                        self.publish(Event::Respawned);
                    } else {
                        self.publish(Event::OutOfLives);
                    }
                }
            }
            Event::Respawned => {
                let level = &self.levels[self.level];
                let spawn = match self.checkpoint {
                    Some(checkpoint) => {
                        let zone = level.checkpoints[checkpoint];
                        [zone.center()[0], zone.y]
                    }
                    None => level.spawn,
                };
                self.player = Player::at(spawn);
                self.effect(EffectKind::Shimmer, self.player.position);
            }
            Event::OutOfLives => {
                self.state = RunState::Failed;
            }
            Event::GoalReached => {
                if self.state == RunState::Running {
                    self.publish(Event::RunFinished {
                        seconds: self.simulated - self.started,
                    });
                }
            }
            Event::RunFinished { seconds } => {
                self.state = RunState::Finished(seconds);
                let at = self.levels[self.level].goal.center();
                self.effect(EffectKind::Confetti, at);
                let previous = self.progress.best[self.level];
                if previous.is_none_or(|best| seconds < best) {
                    self.publish(Event::RecordSet { seconds, previous });
                }
                let next = self.level + 1;
                if next < self.levels.len() && next >= self.progress.unlocked {
                    self.publish(Event::LevelUnlocked { level: next });
                }
            }
            Event::RecordSet { seconds, previous: _ } => {
                self.progress.record(self.level, seconds);
                self.publish(Event::SaveProgress);
            }
            Event::LevelUnlocked { level } => {
                self.progress.unlock(level);
                self.publish(Event::SaveProgress);
            }
            Event::SaveProgress => {
                // The first save in a cascade writes; the rest find the
                // progress clean and do nothing, so one cascade saves once.
                if self.progress.dirty {
                    match self.progress.save(&self.save_path) {
                        Ok(file) => self.publish(Event::ProgressSaved { file }),
                        Err(error) => self.publish(Event::SaveFailed { error }),
                    }
                }
            }
            Event::ProgressSaved { file } => {
                self.status = format!("Saved {file}");
                let at = self.player.position;
                self.effect(EffectKind::SaveFlash, [at[0], at[1] + 1.5]);
            }
            Event::SaveFailed { error } => {
                self.status = error;
            }
        }
    }

    fn effect(&mut self, kind: EffectKind, at: [f64; 2]) {
        self.effects.push(Effect {
            kind,
            at,
            started: self.simulated,
        });
    }

    // The game's state, for scripts and tests.
    pub fn state(&self) -> Value {
        let best = self.progress.best[self.level];
        let state = match self.state {
            RunState::Running => "Running",
            RunState::Finished(_) => "Finished",
            RunState::Failed => "Failed",
        };
        let events: Vec<Value> = self.log.iter().take(6).map(|(_, label)| Value::from(label.as_str())).collect();
        json::object([
            ("Level", self.current().name.into()),
            ("Coins", self.coins.into()),
            ("TotalCoins", self.total_coins().into()),
            ("Lives", self.lives.into()),
            ("State", state.into()),
            ("Time", self.run_seconds().into()),
            ("Best", best.map_or(Value::Null, Value::Number)),
            ("DoorsOpen", self.doors_open.iter().all(|&open| open).into()),
            ("Position", Value::Array(vec![self.player.position[0].into(), self.player.position[1].into()])),
            ("Grounded", self.player.grounded.into()),
            ("Unlocked", self.progress.unlocked.into()),
            ("Status", self.status.as_str().into()),
            ("Events", Value::Array(events)),
        ])
    }
}
