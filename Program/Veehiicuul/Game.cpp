#include "Veehiicuul/Game.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Veehiicuul/Angles.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <limits>

namespace veehiicuul::game {

using collision::Footprint;
using controls::Controls;
using document::Document;
using driving::Car;
using models::Models;
using scene::Scene;
using settings::GameSettings;
using track::Shape;
using vehicles::Vehicle;

namespace {

// The scene that the settings name, or the first, with a notice.
size_t starting_scene(const Document& document, const GameSettings& settings, std::vector<std::string>& notices) {
    for (size_t index = 0; index < document.scenes.size(); index++) {
        if (base::eq_ignore_ascii_case(document.scenes[index].name, settings.scene)) {
            return index;
        }
    }
    notices.push_back(std::format("Scene is {}, which isn't in Scenes, so the game starts in {}.", settings.scene, document.scenes[0].name));
    return 0;
}

// The vehicle that the settings name, or the first, with a notice.
size_t starting_vehicle(const Document& document, const GameSettings& settings, std::vector<std::string>& notices) {
    if (std::optional<size_t> index = document.vehicles.find(settings.vehicle)) {
        return *index;
    }
    notices.push_back(std::format("Vehicle is {}, which isn't in Vehicles.json, so the game starts with {}.", settings.vehicle, document.vehicles.list[0].name));
    return 0;
}

} // namespace

Game::Game(Document game_document, Models game_models, const GameSettings& settings, std::vector<std::string>& notices) :
    document(std::move(game_document)),
    models(std::move(game_models)),
    dead_zones(settings.dead_zones),
    scene(starting_scene(document, settings, notices)),
    vehicle(starting_vehicle(document, settings, notices)),
    car(),
    camera{
        .follow = settings.follow,
        .height = 0.0,
    },
    crashes(0),
    show_card(true),
    restarted(-std::numeric_limits<double>::infinity()),
    last(),
    pending(0.0),
    shape_(document.scenes[scene].track.shape()),
    shape_of(document.scenes[scene].track) {
    camera.height = default_height();
    put_at_start(-std::numeric_limits<double>::infinity());
}

std::expected<Game, std::string> Game::load(const std::filesystem::path& folder, const GameSettings& settings, std::vector<std::string>& notices) {
    std::expected<Models, std::string> loaded_models = Models::load(folder / "Models");
    if (!loaded_models) {
        return std::unexpected(std::move(loaded_models.error()));
    }
    std::expected<Document, std::string> loaded_document = Document::load(folder, *loaded_models);
    if (!loaded_document) {
        return std::unexpected(std::move(loaded_document.error()));
    }
    return Game(std::move(*loaded_document), std::move(*loaded_models), settings, notices);
}

const Scene& Game::current_scene() const {
    return document.scenes[scene];
}

const Vehicle& Game::current_vehicle() const {
    return document.vehicles.list[vehicle];
}

Footprint Game::footprint() const {
    return current_vehicle().footprint;
}

const Shape& Game::shape() {
    const track::Track& track = document.scenes[scene].track;
    if (track != shape_of) {
        shape_ = track.shape();
        shape_of = track;
    }
    return shape_;
}

const Shape& Game::last_shape() const {
    return shape_;
}

double Game::default_height() const {
    const Scene& current = current_scene();
    return camera.follow ? current.follow_height : current.camera_height;
}

std::array<double, 2> Game::camera_center() const {
    return camera.follow ? car.position : current_scene().camera_center;
}

bool Game::paused() const {
    return last && *last - restarted <= Pause;
}

void Game::put_at_start(double seconds) {
    track::Pose start = current_scene().track.start;
    car = Car{
        .position = start.position,
        .heading = start.radians(),
        .velocity = {0.0, 0.0},
    };
    restarted = seconds;
}

void Game::go_to_scene(size_t index, double seconds) {
    scene = std::min(index, document.scenes.size() - 1);
    camera.height = default_height();
    put_at_start(seconds);
}

void Game::keep_in_document() {
    scene = std::min(scene, document.scenes.size() - 1);
    vehicle = std::min(vehicle, document.vehicles.list.size() - 1);
}

void Game::update(const Controls& controls, double seconds) {
    double step = last ? std::clamp(seconds - *last, 0.0, LongestStep) : 0.0;
    last = seconds;
    if (controls.toggle_card) {
        show_card = !show_card;
    }
    if (controls.toggle_camera) {
        camera.follow = !camera.follow;
        camera.height = default_height();
    }
    if (controls.default_zoom) {
        camera.height = default_height();
    }
    double zoomed = (camera.height - controls.zoom * camera::ZoomSpeed * step) / std::pow(camera::Notch, controls.notches);
    camera.height = std::clamp(zoomed, camera::Lowest, camera::Highest);
    if (controls.scene != 0) {
        std::int32_t count = static_cast<std::int32_t>(document.scenes.size());
        size_t next = static_cast<size_t>(base::rem_euclid(static_cast<std::int32_t>(scene) + controls.scene, count));
        go_to_scene(next, seconds);
    }
    if (controls.vehicle != 0) {
        std::int32_t count = static_cast<std::int32_t>(document.vehicles.list.size());
        vehicle = static_cast<size_t>(base::rem_euclid(static_cast<std::int32_t>(vehicle) + controls.vehicle, count));
        put_at_start(seconds);
    }
    if (controls.restart) {
        put_at_start(seconds);
    }
    if (paused()) {
        pending = 0.0;
    } else {
        std::array<double, 2> push = camera::to_ground(controls.push);
        driving::Acceleration acceleration = current_vehicle().acceleration;
        Footprint car_footprint = footprint();
        shape();
        pending += step;
        while (pending >= DriveStep) {
            driving::drive(car, push, controls.brake, acceleration, dead_zones, DriveStep);
            pending -= DriveStep;
            // The very step that touches an edge crashes, and the pause
            // starts at that step's own moment on the drive's clock, so
            // where the car stops, and when it may move again, don't
            // depend on the frame rate.
            if (last_shape().edges.touch(car_footprint, car.position, car.heading)) {
                crashes += 1;
                put_at_start(seconds - pending);
                pending = 0.0;
                break;
            }
        }
    }
}

std::vector<std::pair<std::string_view, json::Value>> Game::state() const {
    std::vector<std::pair<std::string_view, json::Value>> members;
    members.reserve(11);
    members.emplace_back("Scene", current_scene().name);
    members.emplace_back("Vehicle", current_vehicle().name);
    members.emplace_back("Position", car.position);
    members.emplace_back("Heading", to_degrees(car.heading));
    members.emplace_back("Velocity", car.velocity);
    members.emplace_back("Speed", car.speed());
    members.emplace_back("Paused", paused());
    members.emplace_back("Camera", camera.follow ? "Follow" : "Fixed");
    members.emplace_back("ViewHeight", camera.height);
    members.emplace_back("Crashes", crashes);
    members.emplace_back("CardShown", show_card);
    return members;
}

} // namespace veehiicuul::game
