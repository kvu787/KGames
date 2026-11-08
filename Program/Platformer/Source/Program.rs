// The platformer as a Program, which Library/UserInterface's main loop runs.
// A frame's input is one message: the controls, which the game turns into
// fixed steps and events, drained to quiescence before the frame's view.
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

// What a frame's input means.
#[derive(Clone, Debug)]
pub enum Message {
    // Once a frame: the frame's controls, which drive the simulation to the
    // frame's time.
    Tick(Controls),
    // The app bar: a level to go to, and starting the run over.
    GoLevel(usize),
    Restart,
}

pub struct Platformer {
    pub game: Game,
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
            Message::Tick(controls) => self.game.update(&controls, ui.time()),
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
