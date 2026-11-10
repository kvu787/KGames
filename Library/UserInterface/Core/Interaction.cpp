#include "UserInterface/Core/Interaction.h"

#include "Base/Panic.h"
#include "Base/Text.h"

#include <format>

namespace user_interface {

bool Sense::takes(MouseButton button) const {
    switch (button) {
    case MouseButton::Left:
        return left;
    case MouseButton::Right:
        return right;
    case MouseButton::Middle:
        return middle;
    }
    base::unreachable();
}

std::optional<size_t> Part::index() const {
    if (const Index* indexed = base::as<Index>(*this)) {
        return indexed->index;
    }
    return std::nullopt;
}

bool Part::is_named(std::string_view name) const {
    const Named* named = base::as<Named>(*this);
    return named != nullptr && named->name == name;
}

std::string describe_value(const Part& part) {
    return base::match(
        part, [](const Part::Whole&) { return std::string("Whole"); }, [](const Part::Index& indexed) { return std::format("Index({})", indexed.index); },
        [](const Part::Named& named) { return std::format("Named({})", base::debug(named.name)); });
}

bool Hit::pressing() const {
    return held && held_button == MouseButton::Left;
}

ControlState Hit::state(bool enabled) const {
    if (!enabled) {
        return ControlState::Disabled;
    } else if (pressing() && hovered) {
        return ControlState::Pressed;
    } else if (hovered || pressing()) {
        return ControlState::PointerOver;
    } else {
        return ControlState::Normal;
    }
}

const Hit* Interaction::hit(const Part& part) const {
    for (const auto& [at, hit] : hits) {
        if (at == part) {
            return &hit;
        }
    }
    return nullptr;
}

Hit Interaction::whole() const {
    const Hit* hit = this->hit(Part::Whole{});
    return hit != nullptr ? *hit : Hit{};
}

const Part* Interaction::hovered_part() const {
    for (const auto& [part, hit] : hits) {
        if (hit.hovered) {
            return &part;
        }
    }
    return nullptr;
}

const Part* Interaction::held_part() const {
    for (const auto& [part, hit] : hits) {
        if (hit.held) {
            return &part;
        }
    }
    return nullptr;
}

const Part* Interaction::pressed_part() const {
    for (const auto& [part, hit] : hits) {
        if (hit.pressed) {
            return &part;
        }
    }
    return nullptr;
}

const Part* Interaction::activated_part() const {
    for (const auto& [part, hit] : hits) {
        if (hit.activated) {
            return &part;
        }
    }
    return nullptr;
}

bool Interaction::activated(const Part& part) const {
    const Hit* hit = this->hit(part);
    return hit != nullptr && hit->activated;
}

bool Interaction::pressed(const Part& part) const {
    const Hit* hit = this->hit(part);
    return hit != nullptr && hit->pressed;
}

bool Interaction::held(const Part& part) const {
    const Hit* hit = this->hit(part);
    return hit != nullptr && hit->held;
}

bool Interaction::hovered(const Part& part) const {
    const Hit* hit = this->hit(part);
    return hit != nullptr && hit->hovered;
}

bool Interaction::any_held() const {
    for (const auto& [part, hit] : hits) {
        if (hit.held) {
            return true;
        }
    }
    return false;
}

Point Interaction::wheel() const {
    Point sum = Point::Zero;
    for (const auto& [part, hit] : hits) {
        sum = sum + hit.wheel;
    }
    return sum;
}

} // namespace user_interface
