// Everything that happens in the game, as data: the simulation publishes
// events as it discovers them, and Game::apply consumes each one, which may
// publish more. The queue drains to quiescence inside the frame that
// started it, so however deep the graph goes - the last coin opening the
// door, the goal ending the run, the record saving the file, the save
// flashing its light - all of it has happened by the end of the frame.
// An event carries the facts of its moment, like which coin or how many
// seconds; what a consumer needs beyond that, it reads from the game as it
// is when the event is applied, never from a snapshot.

// A cascade that doesn't settle is a bug, not a wait.
pub const MOST_EVENTS: usize = 1_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    // A run began: a new game, a restart, or arriving in a level.
    RunStarted,
    // The player touched a coin that wasn't collected yet.
    CoinCollected { coin: usize },
    // The last coin was collected, which is what opens doors.
    AllCoinsCollected,
    DoorOpened { door: usize },
    // The player landed on a spring while falling.
    SpringBounced { spring: usize },
    // The player touched a checkpoint for the first time this run.
    CheckpointReached { checkpoint: usize },
    Died { cause: Cause },
    // After a death with lives left: back at the checkpoint, or the spawn.
    Respawned,
    // After a death without lives left: the run failed.
    OutOfLives,
    // The player touched the goal while the run was going.
    GoalReached,
    // The run ended well, in this many seconds.
    RunFinished { seconds: f64 },
    // The run beat the level's best time, which was `previous`.
    RecordSet { seconds: f64, previous: Option<f64> },
    // Finishing a level the first time unlocks the next.
    LevelUnlocked { level: usize },
    // Something made the progress worth saving. Consumers made it dirty;
    // the first SaveProgress writes the file, and any more in the same
    // cascade find it clean and do nothing, so one cascade saves once.
    SaveProgress,
    ProgressSaved { file: String },
    SaveFailed { error: String },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cause {
    Spikes,
    Fell,
}

impl Event {
    // A compact line for the log card and for scripts, like
    // "CoinCollected 2".
    pub fn label(&self) -> String {
        match self {
            Event::RunStarted => "RunStarted".to_string(),
            Event::CoinCollected { coin } => format!("CoinCollected {coin}"),
            Event::AllCoinsCollected => "AllCoinsCollected".to_string(),
            Event::DoorOpened { door } => format!("DoorOpened {door}"),
            Event::SpringBounced { spring } => format!("SpringBounced {spring}"),
            Event::CheckpointReached { checkpoint } => format!("CheckpointReached {checkpoint}"),
            Event::Died { cause: Cause::Spikes } => "Died Spikes".to_string(),
            Event::Died { cause: Cause::Fell } => "Died Fell".to_string(),
            Event::Respawned => "Respawned".to_string(),
            Event::OutOfLives => "OutOfLives".to_string(),
            Event::GoalReached => "GoalReached".to_string(),
            Event::RunFinished { seconds } => format!("RunFinished {seconds:.3}"),
            Event::RecordSet { seconds, .. } => format!("RecordSet {seconds:.3}"),
            Event::LevelUnlocked { level } => format!("LevelUnlocked {level}"),
            Event::SaveProgress => "SaveProgress".to_string(),
            Event::ProgressSaved { file } => format!("ProgressSaved {file}"),
            Event::SaveFailed { error } => format!("SaveFailed {error}"),
        }
    }
}
