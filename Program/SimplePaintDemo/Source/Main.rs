// No console window in any build.
#![windows_subsystem = "windows"]

#[path = "Editor.rs"]
mod editor;
#[path = "Panels.rs"]
mod panels;
#[path = "Preview.rs"]
mod preview;

use std::path::Path;

use application::{Arguments, FrameCounter, Settings, Window};
use canvas::{Canvas, Font, theme};
use editor::{Action, Editor};
use graphics::Renderer;
use preview::{IMAGE_SIZE, Preview};
use windows_core::Result;

const NAME: &str = "SimplePaint Demo";
// The em height of the font, in pixels. Programs only support 100% scaling
// (see Library/Application), so this never changes.
const FONT_SIZE: i32 = 15;

fn main() -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings, the command line and Paint.json, which
        // the header lists.
        let mut notices = Vec::new();
        let arguments = Arguments::parse(&mut notices);
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (settings, ()) =
            Settings::for_program::<()>(&folder.join("Settings.json"), &arguments, &mut notices);
        let paint_path = arguments.file(folder, "Paint.json");
        let mut editor = Editor::new(editor::load(&paint_path, &mut notices));
        let mut window = Window::open(NAME, &settings, &arguments, &mut notices)?;

        let font = Font::new(FONT_SIZE)?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, false)?;
        let overlay = canvas::Pipeline::new(&mut renderer, &font)?;
        let preview = Preview::new(&renderer)?;
        // The title bar says how frames are presented, as it affects latency.
        let description = renderer.description();
        window.set_title(&format!("{NAME} - {description}"))?;
        window.show(arguments.activate);
        let mut frames = FrameCounter::default();
        let mut vertices = Vec::new();
        // Where clicks do something in the last frame drawn.
        let mut targets = Vec::new();

        // Handle all pending messages, then render one frame once the
        // renderer is ready for it.
        while window.handle_messages() {
            if let Some([width, height]) = window.take_resize() {
                renderer.resize(width, height)?;
            }

            if window.minimized() {
                // Minimized: sleep until something happens instead of spinning.
                window.wait_for_message()?;
                frames.pause();
                continue;
            }

            // Spin until the GPU and DXGI can take a frame, like GameInputDemo,
            // which explains why. The loop keeps handling window messages
            // through the wait.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }

            // Read input only now, right before drawing, so the frame shows
            // the newest. Without focus, there's none.
            let frame = window.begin_frame();
            frames.frame(frame.seconds);
            for event in &frame.events {
                if let Action::ToggleFullscreen = editor.handle(event, &targets, &paint_path) {
                    window.toggle_fullscreen()?;
                }
            }
            let cursor = frame.cursor;
            editor.follow(cursor, &targets);

            let mut canvas = Canvas::new(&font, &mut vertices);
            let status = panels::Status {
                focused: frame.focused,
                frames: &frames,
                display: &window.display,
                adapter: &description,
                notices: &notices,
                supported: &window.supported,
                window_problems: &window.problems,
                cursor,
            };
            let layout = panels::draw(
                &mut canvas,
                &editor,
                &status,
                renderer.width as f32,
                renderer.height as f32,
            );
            targets = layout.targets;
            renderer.clear(theme::BACKGROUND);
            overlay.draw(&renderer, canvas.vertices);
            // The sphere goes over the preview panel's middle, if it fits.
            if let Some([x, y]) = layout.image
                && x + IMAGE_SIZE <= renderer.width
                && y + IMAGE_SIZE <= renderer.height
            {
                preview.draw(&renderer, &editor.material, [x, y]);
            }
            window.handle_checkpoints(&frame, |path| renderer.capture(path), || editor.state(&targets))?;
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()?;
        window.finish()
    }
}
