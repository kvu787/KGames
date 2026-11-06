// A real-time ETW session with the providers that Events.rs lists, whose
// events a thread of its own reads as they come, and hands to a callback.
// The session is the system's: it outlives the process if the process ends
// without stopping it, like when the process is killed. So a session's name
// has its process's ID, and a session whose process is gone is stopped
// before a new one starts, while another process's live session is left
// alone.

use std::ffi::c_void;
use std::thread::JoinHandle;

use windows_api::*;
use windows_core::{GUID, PCWSTR, PWSTR};

use crate::StartError;
use crate::events::PROVIDERS;

// EVENT_TRACE_PROPERTIES, with room for the session's name after it.
#[repr(C)]
struct Properties {
    properties: EVENT_TRACE_PROPERTIES,
    name: [u16; 260],
}

impl Properties {
    fn new() -> Box<Properties> {
        let mut properties: Box<Properties> = Box::new(unsafe { std::mem::zeroed() });
        properties.properties.Wnode.BufferSize = size_of::<Properties>() as u32;
        properties.properties.LoggerNameOffset = std::mem::offset_of!(Properties, name) as u32;
        properties
    }
}

pub struct Session {
    control: CONTROLTRACE_ID,
    name: Vec<u16>,
    trace: PROCESSTRACE_HANDLE,
    thread: Option<JoinHandle<()>>,
}

// What the reading thread hands each event to.
pub type Callback = Box<dyn FnMut(&EVENT_RECORD) + Send>;

// The callback, and whether it panicked, after which no event goes to it: an
// unwinding out of ETW's callback would abort the process, and a tracker's
// mistake should lose the stats, not the program.
struct Context {
    callback: Callback,
    failed: bool,
}

// What every session here is named: this, the program, and the process ID.
const PREFIX: &str = "KGames ";

impl Session {
    // Starts the session, named after the program and this process, enables
    // the providers, and starts reading.
    pub fn start(program: &str, callback: Callback) -> Result<Session, StartError> {
        let name: Vec<u16> = format!("{PREFIX}{program} {}", unsafe { GetCurrentProcessId() }).encode_utf16().chain([0]).collect();
        unsafe {
            stop_leftovers();
            let mut control = CONTROLTRACE_ID(0);
            let mut properties = session_properties();
            let mut status = StartTraceW(&mut control, PCWSTR(name.as_ptr()), &mut properties.properties);
            if status == ERROR_ALREADY_EXISTS as u32 {
                // Left over by a run that didn't stop it.
                stop(CONTROLTRACE_ID(0), &name);
                properties = session_properties();
                status = StartTraceW(&mut control, PCWSTR(name.as_ptr()), &mut properties.properties);
            }
            match status {
                0 => {}
                code if code == ERROR_ACCESS_DENIED as u32 => return Err(StartError::AccessDenied),
                code => return Err(StartError::Failed(format!("StartTrace failed with error {code}"))),
            }
            let source = properties.properties.Wnode.Guid;
            for provider in &PROVIDERS {
                let status = enable(control, source, provider.guid, provider.level, provider.any, provider.all, provider.events);
                // NVIDIA's provider is only there with NVIDIA's driver.
                if status != 0 && provider.guid != crate::events::NVIDIA {
                    stop(control, &name);
                    return Err(StartError::Failed(format!("EnableTraceEx2 failed with error {status}")));
                }
            }

            let context = Box::into_raw(Box::new(Context { callback, failed: false }));
            let mut logfile: EVENT_TRACE_LOGFILEW = std::mem::zeroed();
            let mut logger_name = name.clone();
            logfile.LoggerName = PWSTR(logger_name.as_mut_ptr());
            logfile.Anonymous.ProcessTraceMode =
                (PROCESS_TRACE_MODE_EVENT_RECORD | PROCESS_TRACE_MODE_RAW_TIMESTAMP | PROCESS_TRACE_MODE_REAL_TIME) as u32;
            logfile.Anonymous2.EventRecordCallback = Some(on_event);
            logfile.Context = context as *mut c_void;
            let trace = OpenTraceW(&mut logfile);
            if trace.0 == u64::MAX {
                drop(Box::from_raw(context));
                stop(control, &name);
                return Err(StartError::Failed(format!("OpenTrace failed: {}", windows_core::Error::from_thread())));
            }
            let context = context as usize;
            let thread = std::thread::Builder::new()
                .name("PresentTiming".to_string())
                .spawn(move || {
                    // Returns once the session stops, or the trace closes.
                    let _ = ProcessTrace(&[trace], None, None);
                    drop(Box::from_raw(context as *mut Context));
                });
            let thread = match thread {
                Ok(thread) => thread,
                Err(error) => {
                    let _ = CloseTrace(trace);
                    drop(Box::from_raw(context as *mut Context));
                    stop(control, &name);
                    return Err(StartError::Failed(format!("couldn't start a thread: {error}")));
                }
            };
            Ok(Session {
                control,
                name,
                trace,
                thread: Some(thread),
            })
        }
    }
}

// Reads the events in a trace file, like one that xperf or WPR saved, in
// order, and gives the performance counter's frequency on the computer that
// saved it.
pub fn read_file(path: &std::path::Path, callback: Callback) -> Result<i64, String> {
    use std::os::windows::ffi::OsStrExt;
    let mut path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut context = Context { callback, failed: false };
    unsafe {
        let mut logfile: EVENT_TRACE_LOGFILEW = std::mem::zeroed();
        logfile.LogFileName = PWSTR(path.as_mut_ptr());
        logfile.Anonymous.ProcessTraceMode = (PROCESS_TRACE_MODE_EVENT_RECORD | PROCESS_TRACE_MODE_RAW_TIMESTAMP) as u32;
        logfile.Anonymous2.EventRecordCallback = Some(on_event);
        logfile.Context = &mut context as *mut Context as *mut c_void;
        let trace = OpenTraceW(&mut logfile);
        if trace.0 == u64::MAX {
            return Err(format!("OpenTrace failed: {}", windows_core::Error::from_thread()));
        }
        let frequency = logfile.LogfileHeader.PerfFreq;
        let status = ProcessTrace(&[trace], None, None);
        let _ = CloseTrace(trace);
        if status != 0 {
            return Err(format!("ProcessTrace failed with error {status}"));
        }
        if context.failed {
            return Err("reading the events panicked".to_string());
        }
        Ok(frequency)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            for provider in &PROVIDERS {
                let _ = EnableTraceEx2(
                    self.control,
                    &provider.guid,
                    EVENT_CONTROL_CODE_DISABLE_PROVIDER as u32,
                    0,
                    0,
                    0,
                    0,
                    None,
                );
            }
            stop(self.control, &self.name);
            let _ = CloseTrace(self.trace);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

unsafe extern "system" fn on_event(record: *mut EVENT_RECORD) {
    unsafe {
        let record = &*record;
        let context = &mut *(record.UserContext as *mut Context);
        if context.failed {
            return;
        }
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (context.callback)(record))).is_err() {
            context.failed = true;
        }
    }
}

// Stops the sessions named here whose processes are gone, which they
// outlived: a process that's killed doesn't stop its own. A session's
// process is the number at the end of its name.
unsafe fn stop_leftovers() {
    // QueryAllTracesW fills what fits, and says how many sessions there are.
    const MOST: usize = 64;
    let mut sessions: Vec<Box<Properties>> = (0..MOST).map(|_| Properties::new()).collect();
    let mut pointers: Vec<*mut EVENT_TRACE_PROPERTIES> =
        sessions.iter_mut().map(|session| &mut session.properties as *mut EVENT_TRACE_PROPERTIES).collect();
    let mut count = 0u32;
    let status = unsafe { QueryAllTracesW(pointers.as_mut_ptr(), MOST as u32, &mut count) };
    if status != 0 && status != ERROR_MORE_DATA as u32 {
        return;
    }
    for session in sessions.iter().take((count as usize).min(MOST)) {
        let end = session.name.iter().position(|&unit| unit == 0).unwrap_or(session.name.len());
        let name = String::from_utf16_lossy(&session.name[..end]);
        let Some(rest) = name.strip_prefix(PREFIX) else {
            continue;
        };
        let Some(process) = rest.rsplit(' ').next().and_then(|id| id.parse::<u32>().ok()) else {
            continue;
        };
        if !unsafe { process_is_gone(process) } {
            continue;
        }
        let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
        unsafe { stop(CONTROLTRACE_ID(0), &wide) };
    }
}

// Whether no process has the ID any more, or the one that has it has
// exited. One that can't be opened at all, like a protected process, is
// taken as alive.
unsafe fn process_is_gone(process: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION as u32, false, process);
        if handle.0.is_null() {
            // ERROR_INVALID_PARAMETER, which E_INVALIDARG is as an HRESULT:
            // no process has the ID.
            return windows_core::Error::from_thread().code() == E_INVALIDARG;
        }
        let mut code = 0u32;
        let running = GetExitCodeProcess(handle, &mut code).as_bool() && code == STILL_ACTIVE as u32;
        let _ = CloseHandle(handle);
        !running
    }
}

// The session's properties, as PresentMon's: real-time, with times on the
// performance counter, and buffers that a busy system doesn't fill up. The
// buffers go to the reader at least once a second.
fn session_properties() -> Box<Properties> {
    let mut properties = Properties::new();
    let session = &mut properties.properties;
    session.Wnode.ClientContext = 1;
    session.Wnode.Flags = WNODE_FLAG_TRACED_GUID as u32;
    session.LogFileMode = EVENT_TRACE_REAL_TIME_MODE as u32;
    session.BufferSize = 64;
    session.MinimumBuffers = 64;
    session.MaximumBuffers = 1024;
    session.FlushTimer = 1;
    properties
}

unsafe fn stop(control: CONTROLTRACE_ID, name: &[u16]) {
    let mut properties = Properties::new();
    unsafe {
        let _ = ControlTraceW(control, PCWSTR(name.as_ptr()), &mut properties.properties, EVENT_TRACE_CONTROL_STOP as u32);
    }
}

// Enables a provider for the session, with only the events listed.
unsafe fn enable(control: CONTROLTRACE_ID, source: GUID, provider: GUID, level: u8, any: u64, all: u64, events: &[u16]) -> u32 {
    // EVENT_FILTER_EVENT_ID: FilterIn, a reserved byte, the count, and the
    // event IDs, as 16-bit words.
    let mut filter: Vec<u16> = vec![1, events.len() as u16];
    filter.extend_from_slice(events);
    let descriptor = EVENT_FILTER_DESCRIPTOR {
        Ptr: filter.as_ptr() as u64,
        Size: (4 + 2 * events.len()) as u32,
        Type: EVENT_FILTER_TYPE_EVENT_ID,
    };
    let parameters = ENABLE_TRACE_PARAMETERS {
        Version: ENABLE_TRACE_PARAMETERS_VERSION_2 as u32,
        EnableProperty: EVENT_ENABLE_PROPERTY_IGNORE_KEYWORD_0 as u32,
        ControlFlags: 0,
        SourceId: source,
        EnableFilterDesc: &descriptor as *const EVENT_FILTER_DESCRIPTOR as *mut EVENT_FILTER_DESCRIPTOR,
        FilterDescCount: 1,
    };
    unsafe {
        EnableTraceEx2(
            control,
            &provider,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER as u32,
            level,
            any,
            all,
            0,
            Some(&parameters),
        )
    }
}
