// WinUI's ProgressBar and ProgressRing: how far something has come, or that
// it's going on for a time no one knows, with their indeterminate
// animations' timing. Those move because their motion is what says that the
// work goes on and the program isn't frozen; apart from a text field's
// blinking caret, nothing else in the library animates. A determinate one
// shows its value as it is, without easing to it.

#pragma once

#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"

#include <optional>
#include <string_view>

namespace user_interface {

// A progress bar across its rectangle's middle: a fraction from 0 to 1, or
// nothing while how long is unknown.
class ProgressBar : public Widget<void> {
public:
    explicit ProgressBar(std::optional<float> value);
    ProgressBar(const ProgressBar&) = delete;
    ProgressBar& operator=(const ProgressBar&) = delete;
    ProgressBar(ProgressBar&&) = default;
    ProgressBar& operator=(ProgressBar&&) = default;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    std::optional<float> value;
};

// A progress ring, centered in its rectangle: a fraction, or nothing while
// how long is unknown.
class ProgressRing : public Widget<void> {
public:
    explicit ProgressRing(std::optional<float> value);
    ProgressRing(const ProgressRing&) = delete;
    ProgressRing& operator=(const ProgressRing&) = delete;
    ProgressRing(ProgressRing&&) = default;
    ProgressRing& operator=(ProgressRing&&) = default;

    // Its diameter.
    ProgressRing&& size(float size) &&;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    std::optional<float> value;
    float size_ = controls::ProgressRingSize;
};

} // namespace user_interface
