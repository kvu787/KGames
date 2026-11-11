// The event graph, against the two laws: a frame's input produces the
// correct state by the end of that frame, and every event's whole graph of
// consequences - however deep - resolves inside the frame that published it.
// Each test runs the Game::update that causes something and asserts
// everything it should have caused, the file on disk included.

use std::path::PathBuf;

use platformer::controls::Controls;
use platformer::game::{Game, LIVES, RunState};
use platformer::level;
use platformer::physics::HALF;
use platformer::progress::Progress;

// A frame at 60 Hz.
const FRAME: f64 = 1.0 / 60.0;

fn save_folder(test: &str) -> PathBuf {
    std::env::temp_dir().join("PlatformerEventTests").join(test)
}

// A game on level 1, saving to a fresh folder of the test's own, settled on
// the ground after a few frames.
fn game(test: &str) -> Game {
    let _ = std::fs::remove_dir_all(save_folder(test));
    let levels = level::levels();
    let progress = Progress::new(levels.len());
    let mut game = Game::new(levels, progress, save_folder(test).join("Save.json"));
    for frame in 1..=5 {
        game.update(&Controls::default(), frame as f64 * FRAME);
    }
    assert!(game.player.grounded);
    game
}

// One more frame, with these controls.
fn frame(game: &mut Game, controls: Controls) {
    let next = game.simulated + FRAME;
    game.update(&controls, next);
}

// Whether the log, newest first, holds lines starting with the labels, in
// order, oldest first.
fn logged(game: &Game, labels: &[&str]) -> bool {
    let log: Vec<&str> = game.log.iter().map(|(_, label)| label.as_str()).rev().collect();
    let mut rest = &log[..];
    for label in labels {
        match rest.iter().position(|line| line.starts_with(label)) {
            Some(index) => rest = &rest[index + 1..],
            None => return false,
        }
    }
    true
}

fn count(game: &Game, label: &str) -> usize {
    game.log.iter().filter(|(_, line)| line == label).count()
}

// Puts the player's feet at a point, standing still.
fn place(game: &mut Game, feet: [f64; 2]) {
    game.player.position = [feet[0], feet[1] + HALF[1]];
    game.player.velocity = [0.0, 0.0];
}

// Collects every coin but the last by hand, as if the run had.
fn collect_all_but_last(game: &mut Game) {
    let total = game.total_coins();
    for coin in 0..total - 1 {
        game.collected[coin] = true;
    }
    game.coins = total - 1;
}

// Runs into the closed door at the doorstep, where the last coin is: the
// frame collects it, which opens the door, and the door stops the frame's
// run, as it was closed when the frame began. So the run isn't over yet.
fn open_the_door_at_the_doorstep(game: &mut Game) {
    collect_all_but_last(game);
    place(game, [57.6, 0.0]);
    frame(game, Controls { push: 1.0, ..Controls::default() });
    assert!(game.doors_open[0]);
    assert_eq!(game.state, RunState::Running);
}

#[test]
fn collecting_the_last_coin_opens_the_door_in_the_same_frame() {
    let mut game = game("LastCoin");
    collect_all_but_last(&mut game);
    assert!(!game.doors_open[0]);
    let coin = *game.current().coins.last().unwrap();
    place(&mut game, [coin[0] - 0.2, 0.0]);
    frame(&mut game, Controls::default());
    assert_eq!(game.coins, game.total_coins());
    assert!(game.doors_open[0], "the chain coin -> all coins -> door open ran inside the frame");
    assert!(logged(&game, &["CoinCollected 4", "AllCoinsCollected", "DoorOpened 0"]), "{:?}", game.log);
}

// The deepest chain: a frame's input collects the last coin, which opens
// the door, all in that frame. The door was closed as the frame began, so it
// stops that frame's run, and the next frame's run reaches the goal, which
// ends the run, sets the record, unlocks the next level, saves the file,
// and flashes the save light - all before that frame's view.
#[test]
fn the_last_coin_opens_the_door_and_the_goal_resolves_in_its_frame() {
    let mut game = game("OneFrame");
    collect_all_but_last(&mut game);
    place(&mut game, [57.6, 0.0]);
    let run = Controls {
        push: 1.0,
        ..Controls::default()
    };
    frame(&mut game, run);
    assert_eq!(game.coins, game.total_coins());
    assert!(game.doors_open[0], "the coin opened the door in its frame");
    assert!((game.player.position[0] - (game.current().doors[0].x - HALF[0])).abs() < 1e-9, "stopped at the door's face");
    assert_eq!(game.state, RunState::Running);
    frame(&mut game, run);

    // The whole graph, in the goal's frame.
    assert!(matches!(game.state, RunState::Finished(_)));
    assert!(game.progress.best[0].is_some(), "the record was set");
    assert_eq!(game.progress.unlocked, 2, "the next level was unlocked");
    assert!(
        logged(
            &game,
            &["CoinCollected 4", "AllCoinsCollected", "DoorOpened 0", "GoalReached", "RecordSet", "ProgressSaved Save.json"]
        ),
        "{:?}",
        game.log
    );
    // The persistent store was really written, inside the frame.
    let saved = std::fs::read_to_string(save_folder("OneFrame").join("Save.json"));
    let _ = std::fs::remove_dir_all(save_folder("OneFrame"));
    let text = saved.expect("Save.json was written in the goal's frame");
    assert!(text.contains("\"Unlocked\": 2"), "{text}");
    // And the frame's effects are already showing.
    use platformer::game::EffectKind;
    let kinds: Vec<EffectKind> = game.effects.iter().map(|effect| effect.kind).collect();
    assert!(kinds.contains(&EffectKind::Sparkle), "{kinds:?}");
    assert!(kinds.contains(&EffectKind::Burst), "{kinds:?}");
    assert!(kinds.contains(&EffectKind::Confetti), "{kinds:?}");
    assert!(kinds.contains(&EffectKind::SaveFlash), "{kinds:?}");
    // RecordSet's label carries the seconds, so match it loosely.
    assert!(game.log.iter().any(|(_, line)| line.starts_with("RecordSet ")), "{:?}", game.log);
}

// Two consumers mark the progress dirty in one cascade - the record and the
// unlock - but the cascade saves once: the second SaveProgress finds the
// store clean.
#[test]
fn one_cascade_saves_once() {
    let mut game = game("SaveOnce");
    open_the_door_at_the_doorstep(&mut game);
    frame(&mut game, Controls { push: 1.0, ..Controls::default() });
    assert!(matches!(game.state, RunState::Finished(_)));
    assert_eq!(count(&game, "SaveProgress"), 2, "the record and the unlock each asked, {:?}", game.log);
    assert_eq!(count(&game, "ProgressSaved Save.json"), 1, "but one write served both, {:?}", game.log);
    let _ = std::fs::remove_dir_all(save_folder("SaveOnce"));
}

// A run that doesn't beat the best sets no record and writes nothing.
#[test]
fn a_slower_run_sets_no_record_and_saves_nothing() {
    let mut game = game("Slower");
    game.progress.best[0] = Some(0.001);
    game.progress.unlocked = 2;
    open_the_door_at_the_doorstep(&mut game);
    frame(&mut game, Controls { push: 1.0, ..Controls::default() });
    assert!(matches!(game.state, RunState::Finished(_)));
    assert_eq!(game.progress.best[0], Some(0.001));
    assert_eq!(count(&game, "ProgressSaved Save.json"), 0, "{:?}", game.log);
    assert!(!save_folder("Slower").join("Save.json").exists());
}

#[test]
fn death_and_respawn_resolve_in_the_frame() {
    let mut game = game("Death");
    // The checkpoint is reached, so the respawn goes there.
    game.reached[0] = true;
    game.checkpoint = Some(0);
    let spikes = game.current().spikes[0];
    place(&mut game, [spikes.center()[0], spikes.y + 0.1]);
    frame(&mut game, Controls::default());
    assert_eq!(game.lives, LIVES - 1);
    let checkpoint = game.current().checkpoints[0];
    assert!((game.player.position[0] - checkpoint.center()[0]).abs() < 1e-9, "respawned at the checkpoint, in the frame");
    assert!(logged(&game, &["Died Spikes", "Respawned"]), "{:?}", game.log);
    assert_eq!(game.state, RunState::Running);
}

#[test]
fn the_last_death_fails_the_run_in_the_frame() {
    let mut game = game("OutOfLives");
    game.lives = 1;
    let spikes = game.current().spikes[0];
    place(&mut game, [spikes.center()[0], spikes.y + 0.1]);
    frame(&mut game, Controls::default());
    assert_eq!(game.lives, 0);
    assert_eq!(game.state, RunState::Failed);
    assert!(logged(&game, &["Died Spikes", "OutOfLives"]), "{:?}", game.log);
    // Failed means frozen: more frames change nothing but the clock.
    let position = game.player.position;
    frame(&mut game, Controls { push: 1.0, ..Controls::default() });
    assert_eq!(game.player.position, position);
}

#[test]
fn falling_out_of_the_level_is_a_death() {
    let mut game = game("Fell");
    place(&mut game, [16.5, -8.0]);
    frame(&mut game, Controls::default());
    assert_eq!(game.lives, LIVES - 1);
    assert!(logged(&game, &["Died Fell", "Respawned"]), "{:?}", game.log);
    // No checkpoint yet, so the respawn is the spawn.
    assert!((game.player.position[0] - game.current().spawn[0]).abs() < 1e-9);
}

#[test]
fn a_checkpoint_counts_once() {
    let mut game = game("Checkpoint");
    let checkpoint = game.current().checkpoints[0];
    place(&mut game, [checkpoint.center()[0], checkpoint.y]);
    frame(&mut game, Controls::default());
    frame(&mut game, Controls::default());
    assert_eq!(count(&game, "CheckpointReached 0"), 1, "{:?}", game.log);
    assert_eq!(game.checkpoint, Some(0));
}

#[test]
fn a_spring_bounces_in_its_frame() {
    let mut game = game("Spring");
    let spring = game.current().springs[0];
    // Falling onto the spring, a step above it.
    game.player.position = [spring.center()[0], spring.top() + 0.57];
    game.player.velocity = [0.0, -6.0];
    frame(&mut game, Controls::default());
    assert!(game.player.velocity[1] > 0.0, "bounced up in the frame, {:?}", game.player);
    assert_eq!(count(&game, "SpringBounced 0"), 1, "{:?}", game.log);
}

#[test]
fn restarting_resets_the_run_and_keeps_the_progress() {
    let mut game = game("Restart");
    game.progress.best[0] = Some(9.0);
    game.collected[0] = true;
    game.coins = 1;
    game.lives = 1;
    frame(&mut game, Controls { restart: true, ..Controls::default() });
    assert_eq!(game.coins, 0);
    assert_eq!(game.lives, LIVES);
    assert_eq!(game.state, RunState::Running);
    assert_eq!(game.progress.best[0], Some(9.0));
    assert!(logged(&game, &["RunStarted"]), "{:?}", game.log);
}
