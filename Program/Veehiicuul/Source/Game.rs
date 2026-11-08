// The game: the document, which has the scenes and the vehicles, the models,
// the car in the current scene, the camera, and what each frame's controls
// do to them in Play. Drawing.rs draws it, and the editors change the
// document.
//
// The car starts behind the start line, and the player drives it around the
// track. If it touches an edge of the road, it goes back to the start, at
// rest, and so does switching to another vehicle or scene.

use std::path::Path;

use json::Value;

use crate::camera::{self, Camera};
use crate::collision::Footprint;
use crate::controls::Controls;
use crate::document::Document;
use crate::driving::{Car, DeadZones, drive};
use crate::models::Models;
use crate::scene::Scene;
use crate::settings::GameSettings;
use crate::track::{Shape, Track};
use crate::vehicles::Vehicle;

// After the car is put back at the start, the controls don't move it for
// this long, so that the player can let go of the stick first.
pub const PAUSE: f64 = 0.35;
// The longest that the car moves in one frame, in seconds, so that a frame
// that comes late, like after the window was minimized, doesn't carry it
// far.
pub const LONGEST_STEP: f64 = 0.1;
// The car drives in steps of this long, as many as a frame's time holds,
// so that it drives the same at any frame rate: the same push for the same
// time puts it in the same place. What's left of a frame's time waits for
// the next.
pub const DRIVE_STEP: f64 = 1.0 / 240.0;

pub struct Game {
    pub document: Document,
    pub models: Models,
    pub dead_zones: DeadZones,
    // The current scene's and the vehicle's indices.
    pub scene: usize,
    pub vehicle: usize,
    pub car: Car,
    pub camera: Camera,
    // How many times the car touched an edge, since the game started.
    pub crashes: u32,
    pub show_card: bool,
    // When the car was last put at the start, and the last frame's time, in
    // seconds.
    restarted: f64,
    last: Option<f64>,
    // Time that the last frame's drive steps left over.
    pending: f64,
    // The current scene's track's shape, and the track it's of.
    shape: Shape,
    shape_of: Track,
}

impl Game {
    // The game, in the scene and with the vehicle that the settings name,
    // or the first of each, with a notice about Settings.json, if they name
    // none.
    pub fn new(document: Document, models: Models, settings: &GameSettings, notices: &mut Vec<String>) -> Game {
        let scene = document
            .scenes
            .iter()
            .position(|scene| scene.name.eq_ignore_ascii_case(&settings.scene))
            .unwrap_or_else(|| {
                notices.push(format!(
                    "Scene is {}, which isn't in Scenes, so the game starts in {}.",
                    settings.scene, document.scenes[0].name
                ));
                0
            });
        let vehicle = document.vehicles.find(&settings.vehicle).unwrap_or_else(|| {
            notices.push(format!(
                "Vehicle is {}, which isn't in Vehicles.json, so the game starts with {}.",
                settings.vehicle, document.vehicles.list[0].name
            ));
            0
        });
        let track = document.scenes[scene].track.clone();
        let mut game = Game {
            shape: track.shape(),
            shape_of: track,
            document,
            models,
            dead_zones: settings.dead_zones,
            scene,
            vehicle,
            car: Car::default(),
            camera: Camera {
                follow: settings.follow,
                height: 0.0,
            },
            crashes: 0,
            show_card: true,
            restarted: f64::NEG_INFINITY,
            last: None,
            pending: 0.0,
        };
        game.camera.height = game.default_height();
        game.put_at_start(f64::NEG_INFINITY);
        game
    }

    // The game with the scenes, vehicles and models in `folder`.
    pub fn load(folder: &Path, settings: &GameSettings, notices: &mut Vec<String>) -> Result<Game, String> {
        let models = Models::load(&folder.join("Models"))?;
        let document = Document::load(folder, &models)?;
        Ok(Game::new(document, models, settings, notices))
    }

    pub fn current_scene(&self) -> &Scene {
        &self.document.scenes[self.scene]
    }

    pub fn current_vehicle(&self) -> &Vehicle {
        &self.document.vehicles.list[self.vehicle]
    }

    pub fn footprint(&self) -> Footprint {
        self.current_vehicle().footprint
    }

    // The current scene's track's shape, made again if the track changed.
    pub fn shape(&mut self) -> &Shape {
        let track = &self.document.scenes[self.scene].track;
        if *track != self.shape_of {
            self.shape = track.shape();
            self.shape_of = track.clone();
        }
        &self.shape
    }

    // The shape as of the last call to shape.
    pub fn last_shape(&self) -> &Shape {
        &self.shape
    }

    // The height of the camera's view before the player zooms it.
    pub fn default_height(&self) -> f64 {
        let scene = self.current_scene();
        if self.camera.follow {
            scene.follow_height
        } else {
            scene.camera_height
        }
    }

    // Where the camera looks, on the ground.
    pub fn camera_center(&self) -> [f64; 2] {
        if self.camera.follow {
            self.car.position
        } else {
            self.current_scene().camera_center
        }
    }

    // Whether the controls can't move the car yet, as it was just put back
    // at the start.
    pub fn paused(&self) -> bool {
        self.last.is_some_and(|last| last - self.restarted <= PAUSE)
    }

    // Puts the car on the start, at rest, at `seconds`.
    pub fn put_at_start(&mut self, seconds: f64) {
        let start = self.current_scene().track.start;
        self.car = Car {
            position: start.position,
            heading: start.radians(),
            velocity: [0.0, 0.0],
        };
        self.restarted = seconds;
    }

    // Goes to a scene, from the start.
    pub fn go_to_scene(&mut self, scene: usize, seconds: f64) {
        self.scene = scene.min(self.document.scenes.len() - 1);
        self.camera.height = self.default_height();
        self.put_at_start(seconds);
    }

    // Keeps the indices in the document, after an editor changed it.
    pub fn keep_in_document(&mut self) {
        self.scene = self.scene.min(self.document.scenes.len() - 1);
        self.vehicle = self.vehicle.min(self.document.vehicles.list.len() - 1);
    }

    // Moves the game to a frame's time, `seconds`, with its controls.
    pub fn update(&mut self, controls: &Controls, seconds: f64) {
        let step = self.last.map_or(0.0, |last| (seconds - last).clamp(0.0, LONGEST_STEP));
        self.last = Some(seconds);
        if controls.toggle_card {
            self.show_card = !self.show_card;
        }
        if controls.toggle_camera {
            self.camera.follow = !self.camera.follow;
            self.camera.height = self.default_height();
        }
        if controls.default_zoom {
            self.camera.height = self.default_height();
        }
        let zoomed = (self.camera.height - controls.zoom * camera::ZOOM_SPEED * step) / camera::NOTCH.powf(controls.notches);
        self.camera.height = zoomed.clamp(camera::LOWEST, camera::HIGHEST);
        if controls.scene != 0 {
            let count = self.document.scenes.len() as i32;
            let scene = (self.scene as i32 + controls.scene).rem_euclid(count) as usize;
            self.go_to_scene(scene, seconds);
        }
        if controls.vehicle != 0 {
            let count = self.document.vehicles.list.len() as i32;
            self.vehicle = (self.vehicle as i32 + controls.vehicle).rem_euclid(count) as usize;
            self.put_at_start(seconds);
        }
        if controls.restart {
            self.put_at_start(seconds);
        }
        if self.paused() {
            self.pending = 0.0;
        } else {
            let push = camera::to_ground(controls.push);
            let acceleration = self.current_vehicle().acceleration;
            let footprint = self.footprint();
            self.shape();
            self.pending += step;
            while self.pending >= DRIVE_STEP {
                drive(&mut self.car, push, controls.brake, &acceleration, &self.dead_zones, DRIVE_STEP);
                self.pending -= DRIVE_STEP;
                // The very step that touches an edge crashes, and the pause
                // starts at that step's own moment on the drive's clock, so
                // where the car stops, and when it may move again, don't
                // depend on the frame rate.
                let car = self.car;
                if self.last_shape().edges.touch(&footprint, car.position, car.heading) {
                    self.crashes += 1;
                    self.put_at_start(seconds - self.pending);
                    self.pending = 0.0;
                    break;
                }
            }
        }
    }

    // What scripts save and check about Play.
    pub fn state(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("Scene", self.current_scene().name.clone().into()),
            ("Vehicle", self.current_vehicle().name.clone().into()),
            ("Position", self.car.position.into()),
            ("Heading", self.car.heading.to_degrees().into()),
            ("Velocity", self.car.velocity.into()),
            ("Speed", self.car.speed().into()),
            ("Paused", self.paused().into()),
            ("Camera", if self.camera.follow { "Follow" } else { "Fixed" }.into()),
            ("ViewHeight", self.camera.height.into()),
            ("Crashes", self.crashes.into()),
            ("CardShown", self.show_card.into()),
        ]
    }
}
