// The game: the tracks and the vehicles, the car on the track, the camera,
// and what each frame's controls do to them. Scene.rs draws it.
//
// The car starts on the start line, and the player drives it around the
// track. If it touches an edge of the road, it goes back to the start, at
// rest, and so does switching to another vehicle or track.

use std::path::Path;

use json::Value;
use meshes::car;

use crate::camera::{self, Camera};
use crate::collision::Footprint;
use crate::controls::Controls;
use crate::driving::{Car, DeadZones, drive};
use crate::settings::GameSettings;
use crate::track::Track;
use crate::vehicles::{Vehicle, Vehicles};

// After the car is put back at the start, the controls don't move it for
// this long, so that the player can let go of the stick first.
pub const PAUSE: f64 = 0.35;
// The longest that the car moves in one frame, in seconds, so that a frame
// that comes late, like after the window was minimized, doesn't carry it
// far.
pub const LONGEST_STEP: f64 = 0.1;

pub struct Game {
    pub tracks: Vec<Track>,
    pub vehicles: Vehicles,
    // Every vehicle's, as they're all the same car.
    pub footprint: Footprint,
    pub dead_zones: DeadZones,
    // The track's and the vehicle's indices.
    pub track: usize,
    pub vehicle: usize,
    pub car: Car,
    pub camera: Camera,
    // How many times the car touched an edge, since the game started.
    pub crashes: u32,
    pub show_text: bool,
    // When the car was last put at the start, and the last frame's time, in
    // seconds.
    restarted: f64,
    last: Option<f64>,
}

impl Game {
    // The game, on the track and with the vehicle that the settings name,
    // or the first of each, with a notice, if they name none.
    pub fn new(tracks: Vec<Track>, vehicles: Vehicles, settings: &GameSettings, notices: &mut Vec<String>) -> Game {
        let track = tracks
            .iter()
            .position(|track| track.name.eq_ignore_ascii_case(&settings.track))
            .unwrap_or_else(|| {
                notices.push(format!(
                    "Settings.json's Track, {}, isn't in Tracks, so the game starts on {}",
                    settings.track, tracks[0].name
                ));
                0
            });
        let vehicle = vehicles.find(&settings.vehicle).unwrap_or_else(|| {
            notices.push(format!(
                "Settings.json's Vehicle, {}, isn't in Vehicles.json, so the game starts with {}",
                settings.vehicle, vehicles.list[0].name
            ));
            0
        });
        let mut game = Game {
            tracks,
            vehicles,
            footprint: Footprint::of_mesh(&car::VERTICES),
            dead_zones: settings.dead_zones,
            track,
            vehicle,
            car: Car::default(),
            camera: Camera {
                follow: settings.follow,
                height: 0.0,
            },
            crashes: 0,
            show_text: true,
            restarted: f64::NEG_INFINITY,
            last: None,
        };
        game.camera.height = game.default_height();
        game.put_at_start(f64::NEG_INFINITY);
        game
    }

    // The game with the tracks in `folder`'s Tracks, and the vehicles in its
    // Vehicles.json.
    pub fn load(folder: &Path, settings: &GameSettings, notices: &mut Vec<String>) -> Result<Game, String> {
        let tracks = Track::load_all(&folder.join("Tracks"))?;
        let vehicles = Vehicles::load(&folder.join("Vehicles.json"))?;
        Ok(Game::new(tracks, vehicles, settings, notices))
    }

    pub fn current_track(&self) -> &Track {
        &self.tracks[self.track]
    }

    pub fn current_vehicle(&self) -> &Vehicle {
        &self.vehicles.list[self.vehicle]
    }

    // The height of the camera's view before the player zooms it.
    pub fn default_height(&self) -> f64 {
        let track = self.current_track();
        if self.camera.follow {
            track.follow_height
        } else {
            track.camera_height
        }
    }

    // Where the camera looks, on the ground.
    pub fn camera_center(&self) -> [f64; 2] {
        if self.camera.follow {
            self.car.position
        } else {
            self.current_track().camera_center
        }
    }

    // Whether the controls can't move the car yet, as it was just put back
    // at the start.
    pub fn paused(&self) -> bool {
        self.last.is_some_and(|last| last - self.restarted <= PAUSE)
    }

    // Puts the car on the start line, at rest, at `seconds`.
    fn put_at_start(&mut self, seconds: f64) {
        let start = self.current_track().start;
        self.car = Car {
            position: start.position,
            heading: start.heading,
            velocity: [0.0, 0.0],
        };
        self.restarted = seconds;
    }

    // Moves the game to a frame's time, `seconds`, with its controls.
    pub fn update(&mut self, controls: &Controls, seconds: f64) {
        let step = self.last.map_or(0.0, |last| (seconds - last).clamp(0.0, LONGEST_STEP));
        self.last = Some(seconds);
        if controls.toggle_text {
            self.show_text = !self.show_text;
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
        if controls.track != 0 {
            let count = self.tracks.len() as i32;
            self.track = (self.track as i32 + controls.track).rem_euclid(count) as usize;
            self.camera.height = self.default_height();
            self.put_at_start(seconds);
        }
        if controls.vehicle != 0 {
            let count = self.vehicles.list.len() as i32;
            self.vehicle = (self.vehicle as i32 + controls.vehicle).rem_euclid(count) as usize;
            self.put_at_start(seconds);
        }
        if controls.restart {
            self.put_at_start(seconds);
        }
        if !self.paused() {
            let push = camera::to_ground(controls.push);
            let acceleration = self.current_vehicle().acceleration;
            drive(&mut self.car, push, controls.brake, &acceleration, &self.dead_zones, step);
        }
        let car = self.car;
        if self.current_track().edges.touch(&self.footprint, car.position, car.heading) {
            self.crashes += 1;
            self.put_at_start(seconds);
        }
    }

    // What scripts save and check.
    pub fn state(&self) -> Value {
        json::object([
            ("Track", self.current_track().name.clone().into()),
            ("Vehicle", self.current_vehicle().name.clone().into()),
            ("Position", self.car.position.into()),
            ("Heading", self.car.heading.to_degrees().into()),
            ("Velocity", self.car.velocity.into()),
            ("Speed", self.car.speed().into()),
            ("Paused", self.paused().into()),
            ("Camera", if self.camera.follow { "Follow" } else { "Fixed" }.into()),
            ("ViewHeight", self.camera.height.into()),
            ("Crashes", self.crashes.into()),
            ("TextShown", self.show_text.into()),
        ])
    }
}
