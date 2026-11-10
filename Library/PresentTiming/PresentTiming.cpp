#include "PresentTiming/PresentTiming.h"

#include "Base/Windows.h"
#include "PresentTiming/Decoder.h"

#include <algorithm>
#include <format>
#include <mutex>

namespace present_timing {

using decoder::Decoder;
using session::Session;
using tracker::Tracker;

struct SharedPresents {
    std::mutex mutex;
    std::vector<Present> presents;
};

std::string describe_value(PresentMode mode) {
    switch (mode) {
    case PresentMode::Unknown:
        return "Unknown";
    case PresentMode::LegacyFlip:
        return "LegacyFlip";
    case PresentMode::LegacyCopy:
        return "LegacyCopy";
    case PresentMode::IndependentFlip:
        return "IndependentFlip";
    case PresentMode::OverlayPlane:
        return "OverlayPlane";
    case PresentMode::ComposedFlip:
        return "ComposedFlip";
    case PresentMode::ComposedGpuCopy:
        return "ComposedGpuCopy";
    case PresentMode::ComposedCpuCopy:
        return "ComposedCpuCopy";
    }
    base::unreachable();
}

std::string describe_value(const Present& present) {
    return std::format("Present {{ start: {}, thread: {}, mode: {}, tearing: {}, screen: {}, flip_delay: {} }}", present.start, present.thread, describe_value(present.mode),
                       present.tearing, present.screen ? std::format("Some({})", *present.screen) : "None", present.flip_delay);
}

std::string StartError::to_string() const {
    return base::match(
        *this, [](const AccessDenied&) { return std::string("reading ETW's events needs the Performance Log Users group"); }, [](const Failed& failed) { return failed.message; });
}

PresentTiming::PresentTiming(Session session, std::shared_ptr<SharedPresents> presents) : presents(std::move(presents)), session_(std::move(session)) {}

std::expected<PresentTiming, StartError> PresentTiming::start(std::string_view name, std::uint32_t process) {
    std::shared_ptr<SharedPresents> presents = std::make_shared<SharedPresents>();
    Decoder decoder;
    Tracker tracker(process);
    std::expected<Session, StartError> session =
        Session::start(name, [output = presents, decoder = std::move(decoder), tracker = std::move(tracker)](const EVENT_RECORD& record) mutable {
            if (std::optional<events::Event> event = events::decode(decoder, record)) {
                tracker.handle(*event);
                std::vector<Present> known = tracker.take();
                if (!known.empty()) {
                    std::lock_guard lock(output->mutex);
                    output->presents.insert(output->presents.end(), known.begin(), known.end());
                }
            }
        });
    if (!session) {
        return std::unexpected(std::move(session.error()));
    }
    return PresentTiming(std::move(*session), std::move(presents));
}

std::vector<Present> PresentTiming::take() {
    std::vector<Present> taken;
    {
        std::lock_guard lock(presents->mutex);
        taken = std::exchange(presents->presents, {});
    }
    return corrector.correct(std::move(taken));
}

std::expected<std::pair<std::vector<Present>, std::int64_t>, std::string> read_file(const std::filesystem::path& path, std::uint32_t process) {
    std::shared_ptr<SharedPresents> presents = std::make_shared<SharedPresents>();
    Decoder decoder;
    Tracker tracker(process);
    std::expected<std::int64_t, std::string> frequency =
        session::read_file(path, [output = presents, decoder = std::move(decoder), tracker = std::move(tracker)](const EVENT_RECORD& record) mutable {
            if (std::optional<events::Event> event = events::decode(decoder, record)) {
                tracker.handle(*event);
                std::lock_guard lock(output->mutex);
                std::vector<Present> known = tracker.take();
                output->presents.insert(output->presents.end(), known.begin(), known.end());
            }
        });
    if (!frequency) {
        return std::unexpected(std::move(frequency.error()));
    }
    std::vector<Present> taken;
    {
        std::lock_guard lock(presents->mutex);
        taken = std::exchange(presents->presents, {});
    }
    return std::pair(Corrector().correct(std::move(taken)), *frequency);
}

std::vector<Present> Corrector::correct(std::vector<Present> presents) {
    std::ranges::stable_sort(presents, {}, &Present::start);
    for (Present& present : presents) {
        if (!present.screen) {
            continue;
        }
        if (present.start == last_start) {
            continue;
        }
        std::int64_t screen = *present.screen;
        if (last_flip_delay != 0 && last_screen > screen) {
            present.flip_delay += last_screen - screen;
            screen = last_screen;
            present.screen = screen;
        }
        last_screen = screen;
        last_flip_delay = present.flip_delay;
        last_start = present.start;
    }
    return presents;
}

std::int64_t frequency() {
    static const std::int64_t Frequency = [] {
        LARGE_INTEGER value{};
        QueryPerformanceFrequency(&value);
        return std::max<std::int64_t>(value.QuadPart, 1);
    }();
    return Frequency;
}

} // namespace present_timing
