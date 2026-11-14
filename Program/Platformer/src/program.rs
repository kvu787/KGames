// The platformer as a Program, which Library/UserInterface's main loop runs.
// Each step of a frame's input is one message: the step's controls, which
// the game turns into its one-off actions, a step by the time since the step
// before, which only the frame's last step has, and events, drained to
// quiescence before the step's view.
// The app bar goes between the unlocked levels and starts a run over, and
// the cards show the run and the events as they happen.

use json::Value;
use user_interface::{Context, Program, Severity, Startup, Tree, Ui};

use crate::cards;
use crate::controls::Controls;
use crate::game::Game;
use crate::level;
use crate::progress::{self, Progress};
use crate::NAME;

// What a step of a frame's input means.
#[derive(Clone, Debug)]
pub enum Message {
    // Once a step: the step's controls, which move the game on by the
    // step's `ui.delta()`, the frame's time in its last step, and 0 before.
    Tick(Controls),
    // The app bar: a level to go to, and starting the run over.
    GoLevel(usize),
    Restart,
}

pub struct Platformer {
    pub game: Game,
    // The game's clock: the last frame's time in a frame's steps before its
    // last, and the frame's time in its last, which has the frame's delta,
    // so that the game moves once a frame, by the frame's delta, after the
    // frame's input.
    clock: f64,
}

impl Platformer {
    // The game as it starts: with the progress from Save.json in the data
    // folder, which is the program's folder, or a script's output folder, so
    // scripted runs leave the player's progress alone.
    pub fn start(startup: &mut Startup) -> Platformer {
        let levels = level::levels();
        let save_path = startup.data_folder.join(progress::FILE);
        let progress = match Progress::load(&save_path, levels.len()) {
            Ok(progress) => progress,
            Err(problem) => {
                startup.notify(Severity::Warning, progress::FILE, &problem);
                Progress::new(levels.len())
            }
        };
        Platformer {
            game: Game::new(levels, progress, save_path),
            clock: 0.0,
        }
    }
}

impl Program for Platformer {
    type Message = Message;

    fn title(&self) -> String {
        NAME.to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        vec![Message::Tick(Controls::read(ui))]
    }

    fn update(&mut self, message: Message, ui: &mut Context<Message>) {
        match message {
            Message::Tick(controls) => {
                if ui.last_step() {
                    self.clock = ui.time();
                }
                self.game.update(&controls, self.clock);
            }
            Message::GoLevel(level) => self.game.go_to_level(level),
            Message::Restart => self.game.restart(),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(&self.game, ui)
    }

    fn state(&self) -> Value {
        self.game.state()
    }
}
