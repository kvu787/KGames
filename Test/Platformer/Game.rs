// The platformer through its frames, as the main loop runs them, with input
// that the tests make up, in the cards' real layout: the law is that a
// frame's input shows its whole effect on that frame's own view.

use std::path::PathBuf;

use platformer::physics::HALF;
use platformer::program::Platformer;
use user_interface::json::Value;
use user_interface::{Headless, Key, Startup};

fn save_folder(test: &str) -> PathBuf {
    std::env::temp_dir().join("PlatformerGameTests").join(test)
}

// The game after its first frames, on the ground, saving to a fresh folder
// of the test's own.
fn start(test: &str) -> Headless<Platformer> {
    let _ = std::fs::remove_dir_all(save_folder(test));
    let mut app = Headless::start(Startup::for_tests("{}", &save_folder(test)), |startup| Ok(Platformer::start(startup)));
    app.frames(5);
    assert!(app.program.game.player.grounded);
    app
}

fn state(app: &Headless<Platformer>, member: &str) -> Value {
    app.state().get(member).cloned().unwrap_or(Value::Null)
}

#[test]
fn a_jump_press_rises_on_its_own_frame() {
    let mut app = start("Jump");
    let ground = app.program.game.player.position[1];
    app.hold(Key::SPACE);
    assert!(app.program.game.player.position[1] > ground, "higher on the very frame of the press");
    assert_eq!(state(&app, "Grounded"), Value::Bool(false));
}

#[test]
fn the_frame_that_collects_a_coin_already_counts_it() {
    let mut app = start("Coin");
    let coin = app.program.game.current().coins[0];
    app.program.game.player.position = [coin[0], coin[1] + HALF[1]];
    app.program.game.player.velocity = [0.0, 0.0];
    app.frame();
    assert_eq!(state(&app, "Coins"), Value::Number(1.0));
    let events = state(&app, "Events");
    assert!(events.to_json().contains("CoinCollected 0"), "{}", events.to_json());
}

// The banner, the record and the saved file are all there on the frame that
// reaches the goal.
#[test]
fn the_goal_frame_shows_the_banner_and_saves() {
    let mut app = start("Goal");
    let game = &mut app.program.game;
    let total = game.total_coins();
    for coin in 0..total {
        game.collected[coin] = true;
    }
    game.coins = total;
    game.doors_open[0] = true;
    let goal = game.current().goal;
    game.player.position = [goal.x + 0.5, 0.55];
    game.player.velocity = [0.0, 0.0];
    assert!(!app.has_target("Banner"));
    app.frame();
    assert_eq!(state(&app, "State"), Value::String("Finished".to_string()));
    assert!(app.has_target("Banner"), "the banner is on the goal's own frame");
    assert_eq!(state(&app, "Status"), Value::String("Saved Save.json".to_string()));
    assert!(save_folder("Goal").join("Save.json").is_file(), "the store was written in the frame");
    assert_eq!(state(&app, "Unlocked"), Value::Number(2.0));
}

#[test]
fn f1_hides_the_cards_on_its_frame() {
    let mut app = start("Cards");
    assert!(app.has_target("Run") && app.has_target("Events"));
    app.key(Key::F1);
    assert!(!app.has_target("Run") && !app.has_target("Events"));
    app.key(Key::F1);
    assert!(app.has_target("Run"));
}

#[test]
fn levels_switch_only_once_unlocked() {
    let mut app = start("Levels");
    app.key(Key::PAGE_UP);
    assert_eq!(state(&app, "Level"), Value::String("First Steps".to_string()), "locked levels don't open");
    app.program.game.progress.unlocked = 2;
    app.key(Key::PAGE_UP);
    assert_eq!(state(&app, "Level"), Value::String("Ridgeline".to_string()));
    assert_eq!(state(&app, "Coins"), Value::Number(0.0), "a fresh run there");
    app.key(Key::PAGE_DOWN);
    assert_eq!(state(&app, "Level"), Value::String("First Steps".to_string()));
}

#[test]
fn r_restarts_the_run_on_its_frame() {
    let mut app = start("Restart");
    app.program.game.lives = 1;
    app.program.game.coins = 2;
    app.program.game.collected[0] = true;
    app.program.game.collected[1] = true;
    app.key(Key::R);
    assert_eq!(state(&app, "Lives"), Value::Number(3.0));
    assert_eq!(state(&app, "Coins"), Value::Number(0.0));
    assert_eq!(state(&app, "State"), Value::String("Running".to_string()));
}

#[test]
fn without_focus_nothing_moves() {
    let mut app = start("Focus");
    let before = app.program.game.player.position;
    app.focus(false).hold(Key::RIGHT).hold(Key::SPACE).frames(10);
    assert_eq!(app.program.game.player.position[0], before[0], "the keys went nowhere");
    assert!(app.program.game.player.grounded);
}

#[test]
fn the_run_card_fits_and_the_targets_stay_on_the_page() {
    for fullscreen in [false, true] {
        let mut app = start("Fits");
        app.set_fullscreen(fullscreen).frame();
        for target in app.ui.targets().iter().filter(|target| target.listed) {
            let rect = target.rect;
            assert!(rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= 2560.0, "{} at {rect:?}", target.name);
        }
    }
}
