// The persistent store: which levels are unlocked and each level's best
// time, in Save.json in the data folder - the program's folder, or a
// script's output folder, so scripted runs and tests leave the player's
// progress alone. A missing file is a fresh start; a wrong one keeps the
// defaults, and the program shows a notice saying what's wrong.

use std::path::Path;

use json::Value;

pub const FILE: &str = "Save.json";

#[derive(Clone, Debug, PartialEq)]
pub struct Progress {
    // How many levels are playable, at least 1.
    pub unlocked: usize,
    // Each level's best time, in seconds.
    pub best: Vec<Option<f64>>,
    // Whether there's something the file doesn't have yet. Consumers set it;
    // the first SaveProgress in a cascade writes and clears it, so one
    // cascade saves once however many things it changed.
    pub dirty: bool,
}

impl Progress {
    pub fn new(levels: usize) -> Progress {
        Progress {
            unlocked: 1,
            best: vec![None; levels],
            dirty: false,
        }
    }

    // Reads the file, or starts fresh. A file that can't be used says why.
    pub fn load(path: &Path, levels: usize) -> Result<Progress, String> {
        let mut progress = Progress::new(levels);
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(progress),
            Err(error) => return Err(format!("Couldn't read {FILE}: {error}")),
        };
        let value = json::parse(&text).map_err(|error| format!("{FILE} line {}: {}", error.line, error.message))?;
        match value.get("Unlocked") {
            Some(Value::Number(number)) if *number >= 1.0 => {
                progress.unlocked = (*number as usize).clamp(1, levels);
            }
            Some(other) => return Err(format!("{FILE}: Unlocked must be a number from 1, not {}", other.describe())),
            None => {}
        }
        if let Some(best) = value.get("Best") {
            let Value::Array(times) = best else {
                return Err(format!("{FILE}: Best must be an array, not {}", best.describe()));
            };
            for (level, time) in times.iter().enumerate().take(levels) {
                match time {
                    Value::Number(seconds) if seconds.is_finite() && *seconds > 0.0 => progress.best[level] = Some(*seconds),
                    Value::Null => {}
                    other => return Err(format!("{FILE}: Best[{level}] must be seconds or null, not {}", other.describe())),
                }
            }
        }
        Ok(progress)
    }

    // Writes the file and comes back clean. Returns the file's name.
    pub fn save(&mut self, path: &Path) -> Result<String, String> {
        let best: Vec<Value> = self.best.iter().map(|time| time.map_or(Value::Null, Value::Number)).collect();
        let value = json::object([("Unlocked", Value::Number(self.unlocked as f64)), ("Best", Value::Array(best))]);
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(|error| format!("Couldn't make {}: {error}", folder.display()))?;
        }
        std::fs::write(path, value.to_json()).map_err(|error| format!("Couldn't write {FILE}: {error}"))?;
        self.dirty = false;
        Ok(FILE.to_string())
    }

    // A finished run's time, if it's the level's best. Marks dirty when it is.
    pub fn record(&mut self, level: usize, seconds: f64) -> Option<Option<f64>> {
        let previous = self.best[level];
        if previous.is_none_or(|best| seconds < best) {
            self.best[level] = Some(seconds);
            self.dirty = true;
            Some(previous)
        } else {
            None
        }
    }

    // Unlocks a level, if it isn't. Marks dirty when it does.
    pub fn unlock(&mut self, level: usize) -> bool {
        if level >= self.unlocked && level < self.best.len() {
            self.unlocked = level + 1;
            self.dirty = true;
            true
        } else {
            false
        }
    }
}
