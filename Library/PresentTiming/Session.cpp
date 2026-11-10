#include "PresentTiming/Session.h"

#include "Base/Text.h"
#include "PresentTiming/Events.h"

#include <algorithm>
#include <chrono>
#include <cstddef>
#include <format>
#include <iterator>
#include <span>
#include <system_error>
#include <utility>
#include <vector>

#pragma comment(lib, "advapi32.lib")

namespace present_timing::session {

using events::Providers;

namespace {

// How often the session's buffers are flushed to the reader: 8 ms, the
// shortest period that Intel's PresentMon service allows, and its overlay's.
constexpr std::chrono::milliseconds FlushPeriod{8};

// EVENT_TRACE_PROPERTIES, with room after it for the session's name and its
// log file's, as long as the ones that QueryAllTraces's sample makes room for.
struct Properties {
    EVENT_TRACE_PROPERTIES properties;
    wchar_t name[1024];
    wchar_t log_file[1024];

    // For a session without a log file, as a real-time one is.
    static std::unique_ptr<Properties> new_() {
        std::unique_ptr<Properties> properties = std::make_unique<Properties>();
        properties->properties.Wnode.BufferSize = static_cast<ULONG>(sizeof(Properties));
        properties->properties.LoggerNameOffset = static_cast<ULONG>(offsetof(Properties, name));
        return properties;
    }

    // For QueryAllTraces, which needs room for each session's log file's name
    // too.
    static std::unique_ptr<Properties> for_query() {
        std::unique_ptr<Properties> properties = Properties::new_();
        properties->properties.LogFileNameOffset = static_cast<ULONG>(offsetof(Properties, log_file));
        return properties;
    }
};

// The callback, and whether it threw, after which no event goes to it: an
// exception out of ETW's callback would end the process, and a tracker's
// mistake should lose the stats, not the program.
struct Context {
    Callback callback;
    bool failed;
};

// What every session here is named: this, the program, and the process ID.
constexpr std::string_view Prefix = "KGames ";

// Sleeps with a high-resolution waitable timer, as Rust's std::thread::sleep
// does. Windows' 15.6 ms timer tick doesn't round its time up, as it rounds
// up the time of Sleep, which std::this_thread::sleep_for calls.
void sleep(std::chrono::nanoseconds duration) {
    bool slept = false;
    if (duration > std::chrono::nanoseconds::zero()) {
        HANDLE timer = CreateWaitableTimerExW(nullptr, nullptr, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS);
        if (timer != nullptr) {
            // In 100 ns, and negative for a time from now.
            LARGE_INTEGER time;
            time.QuadPart = -(duration.count() / 100);
            slept = SetWaitableTimer(timer, &time, 0, nullptr, nullptr, FALSE) && WaitForSingleObject(timer, INFINITE) != WAIT_FAILED;
            CloseHandle(timer);
        }
    }
    if (!slept) {
        std::this_thread::sleep_for(duration);
    }
}

void WINAPI on_event(EVENT_RECORD* record) {
    Context& context = *static_cast<Context*>(record->UserContext);
    if (context.failed) {
        return;
    }
    try {
        context.callback(*record);
    } catch (...) {
        context.failed = true;
    }
}

void stop(CONTROLTRACE_ID control, const std::wstring& name) {
    std::unique_ptr<Properties> properties = Properties::new_();
    ControlTraceW(control, name.c_str(), &properties->properties, EVENT_TRACE_CONTROL_STOP);
}

// A whole number that fits 32 bits: str::parse::<u32>.
std::optional<std::uint32_t> parse_u32(std::string_view text) {
    std::optional<std::uint64_t> number = base::parse_u64(text);
    if (!number || *number > UINT32_MAX) {
        return std::nullopt;
    }
    return static_cast<std::uint32_t>(*number);
}

// Whether no process has the ID any more, or the one that has it has
// exited. One that can't be opened at all, like a protected process, is
// taken as alive.
bool process_is_gone(std::uint32_t process) {
    HANDLE handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, process);
    if (handle == nullptr) {
        // ERROR_INVALID_PARAMETER, which E_INVALIDARG is as an HRESULT:
        // no process has the ID.
        return HRESULT_FROM_WIN32(GetLastError()) == E_INVALIDARG;
    }
    DWORD code = 0;
    bool running = GetExitCodeProcess(handle, &code) && code == STILL_ACTIVE;
    CloseHandle(handle);
    return !running;
}

// Stops the sessions named here whose processes are gone, which they
// outlived: a process that's killed doesn't stop its own. A session's
// process is the number at the end of its name.
void stop_leftovers() {
    // QueryAllTracesW fills what fits, and says how many sessions there are,
    // which can be more than 64, so it's asked again with room for them all.
    size_t room = 64;
    std::vector<std::unique_ptr<Properties>> sessions;
    size_t count = 0;
    while (true) {
        sessions.clear();
        sessions.reserve(room);
        for (size_t index = 0; index < room; index++) {
            sessions.push_back(Properties::for_query());
        }
        std::vector<EVENT_TRACE_PROPERTIES*> pointers;
        pointers.reserve(room);
        for (const std::unique_ptr<Properties>& session : sessions) {
            pointers.push_back(&session->properties);
        }
        ULONG found = 0;
        ULONG status = QueryAllTracesW(pointers.data(), static_cast<ULONG>(room), &found);
        if (status == ERROR_MORE_DATA && found > room) {
            room = found;
            continue;
        }
        if (status != ERROR_SUCCESS && status != ERROR_MORE_DATA) {
            return;
        }
        count = std::min<size_t>(found, room);
        break;
    }
    for (size_t index = 0; index < count; index++) {
        const Properties& session = *sessions[index];
        size_t end = std::wstring_view(session.name, std::size(session.name)).find(L'\0');
        if (end == std::wstring_view::npos) {
            end = std::size(session.name);
        }
        std::string name = base::narrow(std::wstring_view(session.name, end));
        if (!name.starts_with(Prefix)) {
            continue;
        }
        std::string_view rest = std::string_view(name).substr(Prefix.size());
        std::optional<std::uint32_t> process = parse_u32(rest.substr(rest.rfind(' ') + 1));
        if (!process) {
            continue;
        }
        if (!process_is_gone(*process)) {
            continue;
        }
        std::wstring wide = base::widen(name);
        stop(0, wide);
    }
}

// The session's properties, as PresentMon's: real-time, with times on the
// performance counter, and buffers that a busy system doesn't fill up. The
// flush timer hands the buffers to the reader once a second, if the flusher
// hasn't. The session is independent: a session that a killed process left,
// whose buffers no one reads, fills up, and without this, an event that
// can't be written to one of the sessions that its provider is enabled for
// isn't written to any, like PresentMon's.
std::unique_ptr<Properties> session_properties() {
    std::unique_ptr<Properties> properties = Properties::new_();
    EVENT_TRACE_PROPERTIES& session = properties->properties;
    session.Wnode.ClientContext = 1;
    session.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    session.LogFileMode = EVENT_TRACE_REAL_TIME_MODE | EVENT_TRACE_INDEPENDENT_SESSION_MODE;
    session.BufferSize = 64;
    session.MinimumBuffers = 64;
    session.MaximumBuffers = 1024;
    session.FlushTimer = 1;
    return properties;
}

// Enables a provider for the session, with only the events listed.
ULONG enable(CONTROLTRACE_ID control, GUID source, GUID provider, std::uint8_t level, std::uint64_t any, std::uint64_t all, std::span<const std::uint16_t> events) {
    // EVENT_FILTER_EVENT_ID: FilterIn, a reserved byte, the count, and the
    // event IDs, as 16-bit words.
    std::vector<std::uint16_t> filter{1, static_cast<std::uint16_t>(events.size())};
    filter.insert(filter.end(), events.begin(), events.end());
    EVENT_FILTER_DESCRIPTOR descriptor{
        .Ptr = reinterpret_cast<ULONGLONG>(filter.data()),
        .Size = static_cast<ULONG>(4 + 2 * events.size()),
        .Type = EVENT_FILTER_TYPE_EVENT_ID,
    };
    ENABLE_TRACE_PARAMETERS parameters{
        .Version = ENABLE_TRACE_PARAMETERS_VERSION_2,
        .EnableProperty = EVENT_ENABLE_PROPERTY_IGNORE_KEYWORD_0,
        .ControlFlags = 0,
        .SourceId = source,
        .EnableFilterDesc = &descriptor,
        .FilterDescCount = 1,
    };
    return EnableTraceEx2(control, &provider, EVENT_CONTROL_CODE_ENABLE_PROVIDER, level, any, all, 0, &parameters);
}

} // namespace

std::optional<Session::Flusher> Session::Flusher::start(CONTROLTRACE_ID control) {
    std::shared_ptr<std::atomic<bool>> stop = std::make_shared<std::atomic<bool>>(false);
    try {
        std::thread thread([stopping = stop, control] {
            SetThreadDescription(GetCurrentThread(), L"PresentTiming flush");
            std::chrono::steady_clock::time_point next = std::chrono::steady_clock::now();
            while (true) {
                // After a late wake, the cadence starts again from now.
                next += FlushPeriod;
                std::chrono::steady_clock::time_point now = std::chrono::steady_clock::now();
                if (next > now) {
                    // A high-resolution waitable timer, which Windows'
                    // 15.6 ms timer tick doesn't round up.
                    sleep(next - now);
                } else {
                    next = now;
                }
                if (stopping->load(std::memory_order_relaxed)) {
                    return;
                }
                std::unique_ptr<Properties> properties = Properties::new_();
                ControlTraceW(control, nullptr, &properties->properties, EVENT_TRACE_CONTROL_FLUSH);
            }
        });
        return Flusher{std::move(stop), std::move(thread)};
    } catch (const std::system_error&) {
        return std::nullopt;
    }
}

void Session::Flusher::stop() && {
    stop_->store(true, std::memory_order_relaxed);
    thread.join();
}

std::expected<Session, StartError> Session::start(std::string_view program, Callback callback) {
    std::wstring name = base::widen(std::format("{}{} {}", Prefix, program, GetCurrentProcessId()));
    stop_leftovers();
    CONTROLTRACE_ID control = 0;
    std::unique_ptr<Properties> properties = session_properties();
    ULONG status = StartTraceW(&control, name.c_str(), &properties->properties);
    if (status == ERROR_ALREADY_EXISTS) {
        // Left over by a run that didn't stop it.
        stop(0, name);
        properties = session_properties();
        status = StartTraceW(&control, name.c_str(), &properties->properties);
    }
    if (status == ERROR_ACCESS_DENIED) {
        return std::unexpected<StartError>(StartError::AccessDenied{});
    }
    if (status != ERROR_SUCCESS) {
        return std::unexpected<StartError>(StartError::Failed{std::format("StartTrace failed with error {}", status)});
    }
    GUID source = properties->properties.Wnode.Guid;
    for (const events::Provider& provider : Providers) {
        ULONG enabled = enable(control, source, provider.guid, provider.level, provider.any, provider.all, provider.events);
        // NVIDIA's provider is only there with NVIDIA's driver.
        if (enabled != ERROR_SUCCESS && provider.guid != events::Nvidia) {
            stop(control, name);
            return std::unexpected<StartError>(StartError::Failed{std::format("EnableTraceEx2 failed with error {}", enabled)});
        }
    }

    // The reading thread owns the context, and deletes it when it's done.
    Context* context = new Context{std::move(callback), false};
    EVENT_TRACE_LOGFILEW logfile{};
    std::wstring logger_name = name;
    logfile.LoggerName = logger_name.data();
    logfile.ProcessTraceMode = PROCESS_TRACE_MODE_EVENT_RECORD | PROCESS_TRACE_MODE_RAW_TIMESTAMP | PROCESS_TRACE_MODE_REAL_TIME;
    logfile.EventRecordCallback = on_event;
    logfile.Context = context;
    PROCESSTRACE_HANDLE trace = OpenTraceW(&logfile);
    if (trace == INVALID_PROCESSTRACE_HANDLE) {
        delete context;
        stop(control, name);
        return std::unexpected<StartError>(StartError::Failed{std::format("OpenTrace failed: {}", base::last_error().what())});
    }
    std::optional<std::thread> thread;
    try {
        thread.emplace([trace, context] {
            SetThreadDescription(GetCurrentThread(), L"PresentTiming");
            // Returns once the session stops, or the trace closes.
            PROCESSTRACE_HANDLE handle = trace;
            ProcessTrace(&handle, 1, nullptr, nullptr);
            delete context;
        });
    } catch (const std::system_error& error) {
        CloseTrace(trace);
        delete context;
        stop(control, name);
        return std::unexpected<StartError>(StartError::Failed{std::format("couldn't start a thread: {}", error.what())});
    }
    std::optional<Flusher> flusher = Flusher::start(control);
    return Session(control, std::move(name), trace, std::move(*thread), std::move(flusher));
}

Session::Session(CONTROLTRACE_ID control, std::wstring name, PROCESSTRACE_HANDLE trace, std::thread thread, std::optional<Flusher> flusher) :
    control(control), name(std::move(name)), trace(trace), thread(std::move(thread)), flusher(std::move(flusher)) {}

Session::Session(Session&& other) noexcept :
    control(other.control),
    name(std::move(other.name)),
    trace(other.trace),
    thread(std::exchange(other.thread, std::nullopt)),
    flusher(std::exchange(other.flusher, std::nullopt)) {}

Session& Session::operator=(Session&& other) noexcept {
    if (this != &other) {
        drop();
        control = other.control;
        name = std::move(other.name);
        trace = other.trace;
        thread = std::exchange(other.thread, std::nullopt);
        flusher = std::exchange(other.flusher, std::nullopt);
    }
    return *this;
}

Session::~Session() {
    drop();
}

void Session::drop() noexcept {
    if (!thread) {
        return;
    }
    // Before the session stops, so that nothing flushes a stopped one.
    if (flusher) {
        std::move(*flusher).stop();
        flusher.reset();
    }
    for (const events::Provider& provider : Providers) {
        EnableTraceEx2(control, &provider.guid, EVENT_CONTROL_CODE_DISABLE_PROVIDER, 0, 0, 0, 0, nullptr);
    }
    stop(control, name);
    CloseTrace(trace);
    thread->join();
    thread.reset();
}

std::expected<std::int64_t, std::string> read_file(const std::filesystem::path& path, Callback callback) {
    std::wstring wide = path.native();
    Context context{std::move(callback), false};
    EVENT_TRACE_LOGFILEW logfile{};
    logfile.LogFileName = wide.data();
    logfile.ProcessTraceMode = PROCESS_TRACE_MODE_EVENT_RECORD | PROCESS_TRACE_MODE_RAW_TIMESTAMP;
    logfile.EventRecordCallback = on_event;
    logfile.Context = &context;
    PROCESSTRACE_HANDLE trace = OpenTraceW(&logfile);
    if (trace == INVALID_PROCESSTRACE_HANDLE) {
        return std::unexpected(std::format("OpenTrace failed: {}", base::last_error().what()));
    }
    std::expected<std::int64_t, std::string> frequency = file_clock(logfile.LogfileHeader.ReservedFlags, logfile.LogfileHeader.PerfFreq.QuadPart);
    if (!frequency) {
        CloseTrace(trace);
        return frequency;
    }
    ULONG status = ProcessTrace(&trace, 1, nullptr, nullptr);
    CloseTrace(trace);
    if (status != ERROR_SUCCESS) {
        return std::unexpected(std::format("ProcessTrace failed with error {}", status));
    }
    if (context.failed) {
        return std::unexpected(std::string("reading the events panicked"));
    }
    return frequency;
}

std::expected<std::int64_t, std::string> file_clock(std::uint32_t reserved_flags, std::int64_t performance_frequency) {
    switch (reserved_flags) {
    case 1:
        if (performance_frequency > 0) {
            return performance_frequency;
        }
        return std::unexpected(std::string("the trace's performance counter frequency is missing"));
    case 2:
        return std::unexpected(std::string("the trace's events are timed with the system time, not the performance counter"));
    case 3:
        return std::unexpected(std::string("the trace's events are timed with the CPU's cycles, not the performance counter"));
    default:
        return std::unexpected(std::format("the trace's events are timed with an unknown clock, {}", reserved_flags));
    }
}

} // namespace present_timing::session
