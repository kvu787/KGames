// Callback view: a Program whose messages are closures over itself, so that
// what a widget does is written beside it, as in immediate mode, with the
// same frame as a message enum's: the input on the tree the user saw first,
// then one read-only view.
//
//     tree.add("Save", rect, Button::new("Save").on_press(act(|app, _| app.save())));
//     tree.add("Volume", rect, Slider::new(app.volume, 0.0, 1.0).on_change(on(|app, v| app.volume = v)));
//
// A CallbackProgram implements Program with Action<Self> as its message,
// and update runs the action.

use std::rc::Rc;

use graphics::{Frame, Gpu};

use crate::application::program::Program;
use crate::core::interface::{Context, Ui};
use crate::core::tree::Tree;

// What an action does, to the program, with the context.
type Doing<P> = dyn Fn(&mut P, &mut Context<Action<P>>);

// A message that's what to do to the program.
pub struct Action<P>(Rc<Doing<P>>);

impl<P> Clone for Action<P> {
    fn clone(&self) -> Self {
        Action(self.0.clone())
    }
}

impl<P: 'static> Action<P> {
    pub fn new(what: impl Fn(&mut P, &mut Context<Action<P>>) + 'static) -> Self {
        Action(Rc::new(what))
    }

    // Does it to the program.
    pub fn run(&self, program: &mut P, context: &mut Context<Action<P>>) {
        (self.0)(program, context)
    }
}

// An action from what it does.
pub fn act<P: 'static>(what: impl Fn(&mut P, &mut Context<Action<P>>) + 'static) -> Action<P> {
    Action::new(what)
}

// A reaction to a widget's value, like a slider's, from what it does with
// the value: the function that the widget wants, which makes the action.
pub fn on<P: 'static, V: 'static>(what: impl Fn(&mut P, V) + 'static) -> impl Fn(V) -> Action<P> {
    let what = Rc::new(what);
    move |value: V| {
        let what = what.clone();
        let value = Rc::new(std::cell::Cell::new(Some(value)));
        Action::new(move |program: &mut P, _: &mut Context<Action<P>>| {
            if let Some(value) = value.take() {
                what(program, value);
            }
        })
    }
}

// Like `on`, for a reaction that also wants the context, like to close the
// window.
pub fn on_with<P: 'static, V: 'static>(what: impl Fn(&mut P, V, &mut Context<Action<P>>) + 'static) -> impl Fn(V) -> Action<P> {
    let what = Rc::new(what);
    move |value: V| {
        let what = what.clone();
        let value = Rc::new(std::cell::Cell::new(Some(value)));
        Action::new(move |program: &mut P, context: &mut Context<Action<P>>| {
            if let Some(value) = value.take() {
                what(program, value, context);
            }
        })
    }
}

pub trait CallbackProgram: Sized + 'static {
    fn title(&self) -> String;

    fn input(&self, _ui: &Ui) -> Vec<Action<Self>> {
        Vec::new()
    }

    fn view(&self, ui: &mut Ui) -> Tree<Action<Self>>;

    fn prepare(&mut self, _gpu: &mut Gpu) -> windows_core::Result<()> {
        Ok(())
    }

    fn render(&mut self, _gpu: &mut Gpu, _frame: &Frame) -> windows_core::Result<()> {
        Ok(())
    }

    fn may_close(&mut self) -> bool {
        true
    }

    fn state(&self) -> json::Value {
        json::Value::Object(Vec::new())
    }

    fn script(&mut self, step: &json::Value) -> Result<(), String> {
        Err(format!("This program has no Program steps, but got {}.", step.to_json()))
    }
}

impl<P: CallbackProgram> Program for P {
    type Message = Action<P>;

    fn title(&self) -> String {
        CallbackProgram::title(self)
    }

    fn input(&self, ui: &Ui) -> Vec<Action<P>> {
        CallbackProgram::input(self, ui)
    }

    fn update(&mut self, action: Action<P>, ui: &mut Context<Action<P>>) {
        action.run(self, ui);
    }

    fn view(&self, ui: &mut Ui) -> Tree<Action<P>> {
        CallbackProgram::view(self, ui)
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> windows_core::Result<()> {
        CallbackProgram::prepare(self, gpu)
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> windows_core::Result<()> {
        CallbackProgram::render(self, gpu, frame)
    }

    fn may_close(&mut self) -> bool {
        CallbackProgram::may_close(self)
    }

    fn state(&self) -> json::Value {
        CallbackProgram::state(self)
    }

    fn script(&mut self, step: &json::Value) -> Result<(), String> {
        CallbackProgram::script(self, step)
    }
}
