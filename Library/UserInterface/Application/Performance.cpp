#include "UserInterface/Application/Performance.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "UserInterface/Platform/Clock.h"

#include <algorithm>
#include <bit>
#include <cmath>
#include <expected>
#include <format>
#include <limits>
#include <span>

namespace user_interface::performance {

namespace {

using present_timing::Present;
using present_timing::PresentMode;
using present_timing::PresentTiming;
using present_timing::StartError;

// How late Windows' events about presents can come: PresentTiming has ETW
// hand them over every 8 ms, each 7 to 52 ms after it happened, as measured,
// but without that, ETW's flush timer hands them over once a second, each 1
// to 2 seconds after. So a frame waits this long for its present's before
// it's taken as lost.
constexpr double LateSeconds = 5.0;
// How often what the PerfBar says changes, so that it can be read.
constexpr double RefreshSeconds = 0.1;
// How far before its Present call's start a frame's own present time can be,
// to be the same present.
constexpr double MatchSeconds = 0.005;

// How the display took the newest frame: at once, when Windows' events say
// that its flip was immediate, or at its next refresh. An immediate flip
// tears without a variable refresh rate, but not with G-SYNC, which starts a
// new refresh for the frame, so it's immediate with vertical sync on too.
constexpr std::string_view Immediate = "immediate";
constexpr std::string_view AtRefresh = "at refresh";
// The PerfBar's fields' widths, in characters: frames a second and times in
// milliseconds, both with two decimals, the present mode, as wide as the
// longest mode's name, and the flip, as wide as its longer word.
constexpr size_t RateWidth = 8;
constexpr size_t MillisecondsWidth = 7;
constexpr size_t ModeWidth = [] {
    size_t width = 0;
    for (PresentMode mode : present_timing::AllPresentModes) {
        std::string_view name = present_timing::name(mode);
        if (name.size() > width) {
            width = name.size();
        }
    }
    return width;
}();
constexpr size_t FlipWidth = AtRefresh.size();

// Runs, one after another, with a label's words joined into one run.
struct Runs {
    std::vector<Run> runs;

    void label(std::string_view text) {
        if (!runs.empty() && !runs.back().value) {
            runs.back().text += text;
        } else {
            runs.push_back(Run{.text = std::string(text), .value = false});
        }
    }

    void value(std::string text) {
        runs.push_back(Run{.text = std::move(text), .value = true});
    }
};

// A number with so many decimals, right-aligned in a field so many
// characters wide, or a dash where its last digit would go if it isn't
// known. One too wide for its field fills it with #, as a spreadsheet's
// cell does, instead of pushing what comes after it out of place.
std::string number(std::optional<double> value, size_t decimals, size_t width) {
    std::string text;
    if (!value) {
        text = std::format("{:>{}}", "-", width);
    } else if (std::isnan(*value)) {
        // As Rust writes it, where C++ writes "nan".
        text = std::format("{:>{}}", "NaN", width);
    } else {
        text = std::format("{:>{}.{}f}", *value, width, decimals);
    }
    return text.size() > width ? std::string(width, '#') : text;
}

// Words, left-aligned in a field so many characters wide, cut short if
// they're wider, or a dash if they aren't known.
std::string words(std::optional<std::string_view> value, size_t width) {
    return std::format("{:<{}.{}}", value.value_or("-"), width, width);
}

// Words the PerfBar.
void word(Summary& summary) {
    bool frames = summary.frames_per_second > 0.0;
    bool measuring = base::is<LatencyState::Measuring>(summary.latency);
    bool latency = measuring && summary.samples > 0;
    auto rate = [](double value, bool known) {
        return number(known ? std::optional<double>(value) : std::nullopt, 2, RateWidth);
    };
    auto time = [](double value, bool known) {
        return number(known ? std::optional<double>(value) : std::nullopt, 2, MillisecondsWidth);
    };

    Runs stats;
    stats.label("FPS = ");
    stats.value(rate(summary.frames_per_second, frames));
    stats.label(Separator);
    stats.label("Input latency = ");
    stats.value(time(summary.mean, latency));
    stats.label(" ms");
    // How far back the stats look.
    stats.label(Separator);
    stats.label("Lookback = ");
    stats.value(base::display(WindowSeconds));
    stats.label(" s");
    stats.label(Separator);
    stats.label("Mode = ");
    stats.value(words(measuring ? std::optional<std::string_view>(present_timing::name(summary.mode)) : std::nullopt, ModeWidth));
    stats.label(Separator);
    stats.label("Flip = ");
    stats.value(words(measuring ? std::optional<std::string_view>(summary.immediate ? Immediate : AtRefresh) : std::nullopt, FlipWidth));
    summary.stats = std::move(stats.runs);

    Runs note;
    std::optional<std::string_view> reason = base::match(
        summary.latency, [](const LatencyState::Unavailable& unavailable) { return std::optional<std::string_view>(unavailable.reason); },
        [](const LatencyState::Starting&) { return std::optional<std::string_view>("starting"); },
        [](const LatencyState::Measuring&) { return std::optional<std::string_view>(); });
    if (reason) {
        note.label(Separator);
        note.label("Input latency = ");
        note.value(std::string(*reason));
    }
    summary.note = std::move(note.runs);
}

// Where a number is among all of them, NaNs and both zeros included, as
// f64::total_cmp orders them.
std::int64_t total_order(double value) {
    std::int64_t bits = std::bit_cast<std::int64_t>(value);
    bits ^= static_cast<std::int64_t>(static_cast<std::uint64_t>(bits >> 63) >> 1);
    return bits;
}

// The value of sorted values at a fraction of the way through them, between
// the two nearest.
double percentile(std::span<const double> sorted, double fraction) {
    if (sorted.empty()) {
        return 0.0;
    }
    double position = fraction * static_cast<double>(sorted.size() - 1);
    size_t below = base::saturating_cast<size_t>(std::floor(position));
    size_t above = base::saturating_cast<size_t>(std::ceil(position));
    double weight = position - static_cast<double>(below);
    return sorted[below] * (1.0 - weight) + sorted[above] * weight;
}

} // namespace

std::string describe_value(const LatencyState& state) {
    return base::match(
        state, [](const LatencyState::Starting&) { return std::string("Starting"); },
        [](const LatencyState::Unavailable& unavailable) { return std::format("Unavailable({})", base::debug(unavailable.reason)); },
        [](const LatencyState::Measuring&) { return std::string("Measuring"); });
}

std::string describe_value(const Run& run) {
    return std::format("Run {{ text: {}, value: {} }}", base::debug(run.text), run.value);
}

Performance Performance::measure(std::string_view name) {
    std::expected<PresentTiming, StartError> started = PresentTiming::start(name, GetCurrentProcessId());
    if (started) {
        return Performance(Source::Timing{std::move(*started)});
    }
    return base::match(
        started.error(), [](const StartError::AccessDenied&) { return Performance(Source::Unavailable{"needs the Performance Log Users group"}); },
        [](const StartError::Failed& failed) { return Performance(Source::Unavailable{std::format("unavailable: {}", failed.message)}); });
}

Performance Performance::without_latency(std::string_view reason) {
    return Performance(Source::Unavailable{std::string(reason)});
}

Performance Performance::given() {
    return Performance(Source::Given{});
}

Performance::Performance(Source source) : source(std::move(source)), refreshed(-std::numeric_limits<double>::infinity()) {}

void Performance::frame_started(double time) {
    frame_times.push_back(time);
    while (frame_times.front() != nullptr && *frame_times.front() < time - WindowSeconds) {
        frame_times.pop_front();
    }
}

void Performance::presented(Frame frame) {
    if (!base::is<Source::Unavailable>(source)) {
        presented_.push_back(frame);
    }
}

void Performance::known(std::vector<Present> presents) {
    for (const Present& each : presents) {
        present(each);
    }
}

void Performance::update(double time) {
    if (Source::Timing* timing = base::as<Source::Timing>(source)) {
        std::vector<Present> presents = timing->timing.take();
        known(std::move(presents));
    }
    std::int64_t now = clock::now();
    std::int64_t window = base::saturating_cast<std::int64_t>(WindowSeconds * static_cast<double>(clock::frequency()));
    std::int64_t late = base::saturating_cast<std::int64_t>(LateSeconds * static_cast<double>(clock::frequency()));
    // The frames on the screen, and the input latency's samples, over the
    // window before the newest frame known on the screen, as Windows'
    // events about them come late, and none from longer ago than events
    // can be late, like if they stopped.
    std::int64_t newest = displayed.back() != nullptr ? *displayed.back() : now;
    std::int64_t start = std::max(newest - window, now - late - window);
    while (displayed.front() != nullptr && *displayed.front() < start) {
        displayed.pop_front();
    }
    while (samples.front() != nullptr && samples.front()->screen < start) {
        samples.pop_front();
    }
    // Frames whose presents never came, like if ETW lost an event.
    while (presented_.front() != nullptr && presented_.front()->present < now - late) {
        presented_.pop_front();
    }
    if (time - refreshed >= RefreshSeconds || time < refreshed) {
        refreshed = time;
        summary_ = summarize();
    }
}

const Summary& Performance::summary() const {
    return summary_;
}

void Performance::present(const Present& present) {
    std::int64_t slack = base::saturating_cast<std::int64_t>(MatchSeconds * static_cast<double>(clock::frequency()));
    // Frames before this present that no present came for.
    while (presented_.front() != nullptr && presented_.front()->present < present.start - slack) {
        presented_.pop_front();
    }
    if (presented_.front() == nullptr) {
        return;
    }
    Frame frame = *presented_.front();
    if (frame.present > present.start) {
        // Not one of the main loop's presents.
        return;
    }
    presented_.pop_front();
    known_ = true;
    if (!present.screen) {
        if (frame.input) {
            carried = frame;
        }
        return;
    }
    std::int64_t screen = *present.screen;
    displayed.push_back(screen);
    mode = present.mode;
    // PresentTiming calls an immediate flip tearing, as PresentMon does.
    immediate = present.tearing;
    std::optional<Frame> from = frame.input ? std::optional<Frame>(frame) : carried;
    carried.reset();
    if (from && from->input) {
        std::int64_t input = *from->input;
        samples.push_back(Sample{
            .screen = screen,
            .total = clock::milliseconds(screen - input),
            .waiting = clock::milliseconds(from->read - input),
            .building = clock::milliseconds(from->present - from->read),
            .presenting = clock::milliseconds(screen - from->present),
        });
    }
}

Summary Performance::summarize() const {
    Summary summary;
    if (frame_times.front() != nullptr && frame_times.back() != nullptr) {
        double first = *frame_times.front();
        double last = *frame_times.back();
        size_t count = frame_times.size() - 1;
        if (count > 0 && last > first) {
            summary.frames_per_second = static_cast<double>(count) / (last - first);
            summary.frame_time = (last - first) * 1000.0 / static_cast<double>(count);
            double longest = 0.0;
            for (size_t index = 0; index + 1 < frame_times.size(); index++) {
                longest = std::fmax(longest, (frame_times[index + 1] - frame_times[index]) * 1000.0);
            }
            summary.longest_frame = longest;
        }
    }
    if (const Source::Unavailable* unavailable = base::as<Source::Unavailable>(source)) {
        summary.latency = LatencyState::Unavailable{unavailable->reason};
    } else if (!known_) {
        summary.latency = LatencyState::Starting{};
    } else {
        summary.latency = LatencyState::Measuring{};
    }
    if (displayed.front() != nullptr && displayed.back() != nullptr && *displayed.back() > *displayed.front()) {
        summary.displayed_per_second = static_cast<double>(displayed.size() - 1) / clock::seconds(*displayed.back() - *displayed.front());
    }
    summary.mode = mode;
    summary.immediate = immediate;
    summary.samples = samples.size();
    if (!samples.empty()) {
        auto mean = [&](double Sample::* part) {
            // From -0, as Rust's sum of floats is.
            double sum = -0.0;
            for (const Sample& sample : samples) {
                sum += sample.*part;
            }
            return sum / static_cast<double>(samples.size());
        };
        std::vector<double> totals;
        totals.reserve(samples.size());
        for (const Sample& sample : samples) {
            totals.push_back(sample.total);
        }
        std::ranges::stable_sort(totals, [](double a, double b) { return total_order(a) < total_order(b); });
        summary.mean = mean(&Sample::total);
        summary.percentile_99 = percentile(totals, 0.99);
        summary.waiting = mean(&Sample::waiting);
        summary.building = mean(&Sample::building);
        summary.presenting = mean(&Sample::presenting);
    }
    word(summary);
    return summary;
}

} // namespace user_interface::performance
