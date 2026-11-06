// Follows presents through the events about them, to when the display took
// each, by the rules that PresentMon 2.6.1's consumer follows (its
// PresentMonTraceConsumer.cpp), so that the times are the same as its. Only
// the paths that DXGI flip-model swap chains take are here: the display
// taking the buffer itself (independent flip), doing that on an overlay
// plane, and DWM composing it into its own frame. It follows the process's
// presents, and DWM's, whose frames show the composed ones.
//
// A present is found by different keys as it goes: its thread while its
// Present call runs, Win32k's token for it, the kernel's present history
// token, and the submit sequence of its queue packet, which the flips and
// the display's sync interrupts (DPCs) name.

use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::events::{CONFIRMED, DISCARDED, Event, IN_FRAME, Kind};
use crate::{Present, PresentMode};

// How many presents can be followed at once, before the oldest is lost.
const RING: usize = 2048;

// DXGI's Present flags and results.
const PRESENT_TEST: u32 = 0x1;
const PRESENT_DO_NOT_SEQUENCE: u32 = 0x2;
const STATUS_OCCLUDED: u32 = 0x087A_0001;
const STATUS_NO_DESKTOP_ACCESS: u32 = 0x087A_0005;
const STATUS_MODE_CHANGE_IN_PROGRESS: u32 = 0x087A_0008;

// The kernel's flip statuses, and present models.
const FLIP_WAIT_VSYNC: u32 = 5;
const FLIP_WAIT_HSYNC: u32 = 15;
const FLIP_IMMEDIATE: u32 = 0x2;
const MODEL_REDIRECTED_GDI: u32 = 1;
const MODEL_REDIRECTED_COMPOSITION: u32 = 7;
const PACKET_MMIO_FLIP: u32 = 3;
const PACKET_SOFTWARE: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Unknown,
    LegacyFlip,
    IndependentFlip,
    ComposedIndependentFlip,
    ComposedFlip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Final {
    Unknown,
    Presented,
    Discarded,
}

struct Tracked {
    start: i64,
    process: u32,
    thread: u32,
    driver_thread: u32,
    time_in_present: i64,
    // Whether DXGI's Present_Start made it, or else a kernel event did.
    dxgi: bool,
    swap_chain: u64,
    flags: u32,
    submit_sequence: u32,
    win32k_key: Option<(u64, u64, u64)>,
    token: u64,
    window: u64,
    ring: usize,
    mode: Mode,
    final_state: Final,
    screen: Option<i64>,
    tearing: bool,
    flip_delay: i64,
    dependents: Vec<u64>,
    wait_for_flip: bool,
    wait_for_mpo_flip: bool,
    wait_for_sync_dpc: bool,
    seen_dxgk_present: bool,
    seen_win32k: bool,
    seen_in_frame: bool,
    completed: bool,
    lost: bool,
    failed: bool,
    waiting_for_dwm: bool,
    waiting_for_present_stop: bool,
}

pub struct Tracker {
    // The process whose presents are reported.
    process: u32,
    presents: HashMap<u64, Tracked>,
    next_id: u64,
    ring: Vec<Option<u64>>,
    next_ring: usize,
    by_thread: HashMap<u32, u64>,
    ordered: HashMap<u32, BTreeMap<i64, u64>>,
    // By submit sequence, then by the queue's context, in the order added.
    by_submit_sequence: HashMap<u32, Vec<(u64, u64)>>,
    by_win32k_key: HashMap<(u64, u64, u64), u64>,
    by_token: HashMap<u64, u64>,
    last_by_window: HashMap<u64, u64>,
    waiting_for_dwm: Vec<u64>,
    dwm_process: u32,
    dwm_thread: u32,
    completed_one: bool,
    // Completed presents, in the order they completed.
    completed: VecDeque<u64>,
    // NVIDIA's flip requests.
    last_flip_token: u32,
    last_flip_time: HashMap<u32, i64>,
    flip_requests: HashMap<u32, (i64, u32)>,
    // The process's presents that are known, in the order they completed.
    output: Vec<Present>,
}

impl Tracker {
    pub fn new(process: u32) -> Tracker {
        Tracker {
            process,
            presents: HashMap::new(),
            next_id: 1,
            ring: vec![None; RING],
            next_ring: 0,
            by_thread: HashMap::new(),
            ordered: HashMap::new(),
            by_submit_sequence: HashMap::new(),
            by_win32k_key: HashMap::new(),
            by_token: HashMap::new(),
            last_by_window: HashMap::new(),
            waiting_for_dwm: Vec::new(),
            dwm_process: 0,
            dwm_thread: 0,
            completed_one: false,
            completed: VecDeque::new(),
            last_flip_token: 0,
            last_flip_time: HashMap::new(),
            flip_requests: HashMap::new(),
            output: Vec::new(),
        }
    }

    // The process's presents whose fate became known since the last call.
    pub fn take(&mut self) -> Vec<Present> {
        std::mem::take(&mut self.output)
    }

    fn tracked(&self, process: u32) -> bool {
        process == self.process || (self.dwm_process != 0 && process == self.dwm_process)
    }

    fn get(&self, id: u64) -> &Tracked {
        &self.presents[&id]
    }

    fn get_mut(&mut self, id: u64) -> &mut Tracked {
        self.presents.get_mut(&id).unwrap()
    }

    pub fn handle(&mut self, event: &Event) {
        match &event.kind {
            Kind::PresentStart { swap_chain, flags, sync_interval: _ } => {
                if self.tracked(event.process) && flags & PRESENT_TEST == 0 {
                    let id = self.create(event, true);
                    let present = self.get_mut(id);
                    present.swap_chain = *swap_chain;
                    present.flags = *flags;
                    self.track(id);
                }
            }
            Kind::PresentStop { result } => {
                if self.tracked(event.process) {
                    self.present_stop(event, *result);
                }
            }
            Kind::TokenCompositionSurfaceObject { key } => {
                while let Some(id) = self.find_or_create(event) {
                    if self.get(id).seen_win32k {
                        self.remove_lost(id);
                        continue;
                    }
                    let present = self.get_mut(id);
                    present.mode = Mode::ComposedFlip;
                    present.seen_win32k = true;
                    present.win32k_key = Some(*key);
                    self.by_win32k_key.insert(*key, id);
                    break;
                }
            }
            Kind::PresentHistoryStart { token, model } => {
                if *model != MODEL_REDIRECTED_GDI {
                    self.present_history_start(event, *token, *model);
                }
            }
            Kind::PresentHistoryInfo { token } => {
                if let Some(id) = self.by_token.get(token).copied() {
                    let present = self.get_mut(id);
                    if present.mode == Mode::ComposedFlip && !present.seen_win32k && !present.waiting_for_dwm {
                        present.waiting_for_dwm = true;
                        self.waiting_for_dwm.push(id);
                    }
                    self.by_token.remove(token);
                }
            }
            Kind::QueuePacketStart { packet_type, submit_sequence, context, present } => {
                if (*packet_type == PACKET_MMIO_FLIP || *packet_type == PACKET_SOFTWARE || *present)
                    && let Some(id) = self.by_thread.get(&event.thread).copied()
                    && self.get(id).submit_sequence == 0
                {
                    self.get_mut(id).submit_sequence = *submit_sequence;
                    self.by_submit_sequence.entry(*submit_sequence).or_default().push((*context, id));
                }
            }
            Kind::QueuePacketStop { context, submit_sequence } => {
                let found = self
                    .by_submit_sequence
                    .get(submit_sequence)
                    .and_then(|entries| entries.iter().find(|(known, _)| known == context))
                    .map(|(_, id)| *id);
                if let Some(id) = found
                    && self.get(id).mode == Mode::LegacyFlip
                    && !self.get(id).wait_for_flip
                {
                    self.set_screen_time(id, event.time);
                    self.complete(id);
                }
            }
            Kind::PresentInfo { window } => {
                if let Some(id) = self.by_thread.get(&event.thread).copied() {
                    let present = self.get_mut(id);
                    present.seen_dxgk_present = true;
                    if present.window == 0 {
                        present.window = *window;
                    }
                    if !present.dxgi || present.thread != event.thread {
                        self.by_thread.remove(&event.thread);
                    }
                }
            }
            Kind::Flip { interval, mmio } => {
                if let Some(id) = self.flip(event) {
                    let present = self.get_mut(id);
                    if *mmio {
                        present.wait_for_flip = true;
                    } else if *interval == 0 {
                        present.tearing = true;
                    }
                }
            }
            Kind::FlipMultiPlaneOverlay { source, layer } => {
                if let Some(id) = self.flip(event) {
                    let present = self.get_mut(id);
                    present.wait_for_flip = true;
                    present.wait_for_mpo_flip = true;
                    if present.swap_chain == 0 {
                        present.swap_chain = ((*source as u64) << 32) | *layer as u64;
                    }
                }
            }
            Kind::IndependentFlip { submit_sequence, interval: _ } => {
                if let Some(id) = self.by_submit(*submit_sequence) {
                    self.get_mut(id).mode = Mode::IndependentFlip;
                }
            }
            Kind::MmioFlip { submit_sequence, flags } => {
                if let Some(id) = self.by_submit(*submit_sequence) {
                    let present = self.get_mut(id);
                    if present.mode == Mode::ComposedFlip {
                        present.mode = Mode::IndependentFlip;
                    }
                    if flags & FLIP_IMMEDIATE != 0 {
                        self.set_screen_time(id, event.time);
                        self.get_mut(id).tearing = true;
                        if self.get(id).mode == Mode::LegacyFlip {
                            self.complete(id);
                        }
                    }
                }
            }
            Kind::MmioFlipMultiPlaneOverlay { submit_sequence, status } => {
                if let (Some(id), Some(status)) = (self.by_submit(*submit_sequence), status) {
                    match *status {
                        FLIP_WAIT_VSYNC => self.get_mut(id).wait_for_sync_dpc = true,
                        FLIP_WAIT_HSYNC => {
                            let present = self.get_mut(id);
                            present.wait_for_sync_dpc = true;
                            present.tearing = true;
                        }
                        _ => {
                            self.get_mut(id).tearing = true;
                            // NVIDIA's driver may flip later than the kernel's
                            // event, by the delay that it asked for.
                            if let Some((delay, _)) = self.flip_requests.get(&event.thread).copied() {
                                self.get_mut(id).flip_delay = delay;
                                self.flip_requests.clear();
                            }
                            let delay = self.get(id).flip_delay;
                            self.set_screen_time(id, event.time + delay);
                            if self.get(id).mode == Mode::LegacyFlip {
                                self.complete(id);
                            }
                        }
                    }
                }
            }
            Kind::VSyncDpc { fence } => {
                if *fence != 0
                    && let Some(id) = self.by_submit((fence >> 32) as u32)
                {
                    self.set_screen_time(id, event.time);
                    let present = self.get(id);
                    if present.mode == Mode::LegacyFlip && !present.wait_for_mpo_flip {
                        self.complete(id);
                    }
                }
            }
            Kind::SyncDpcMultiPlane { planes, flip_sequences } => {
                if !flip_sequences.is_empty() {
                    let multi_plane = planes.iter().filter(|address| **address != 0).count() > 1;
                    for sequence in flip_sequences {
                        if *sequence == 0 {
                            continue;
                        }
                        if let Some(id) = self.by_submit((sequence >> 32) as u32) {
                            let present = self.get_mut(id);
                            if multi_plane && matches!(present.mode, Mode::IndependentFlip | Mode::ComposedFlip) {
                                present.mode = Mode::ComposedIndependentFlip;
                            }
                            if present.final_state != Final::Presented {
                                self.set_screen_time(id, event.time);
                            }
                            self.complete(id);
                        }
                    }
                }
            }
            Kind::TokenStateChanged { key, state, independent_flip } => {
                if let Some(id) = self.by_win32k_key.get(key).copied() {
                    self.token_state_changed(id, *key, *state, *independent_flip);
                }
            }
            Kind::DwmSchedulePresent => {
                self.dwm_process = event.process;
                self.dwm_thread = event.thread;
            }
            Kind::DwmGetPresentHistory => self.last_by_window.clear(),
            Kind::DwmScheduleSurfaceUpdate { key } => {
                if let Some(id) = self.by_win32k_key.get(key).copied() {
                    let present = self.get_mut(id);
                    if present.seen_in_frame && !present.waiting_for_dwm {
                        present.waiting_for_dwm = true;
                        self.waiting_for_dwm.push(id);
                    }
                }
            }
            Kind::NvidiaFlipRequest { alloc, source, time, token } => {
                if *token == self.last_flip_token {
                    return;
                }
                self.last_flip_token = *token;
                let (mut proposed, mut delay) = (0i64, 0i64);
                if *alloc == 0 {
                    proposed = *time as i64;
                    delay = (proposed - event.time).max(0);
                    if let Some(last) = self.last_flip_time.get(source).copied()
                        && last != 0
                        && proposed < last
                    {
                        delay = last - proposed;
                        proposed = last;
                    }
                }
                self.last_flip_time.insert(*source, proposed);
                self.flip_requests.entry(event.thread).or_insert((delay, *token));
            }
        }
    }

    fn create(&mut self, event: &Event, dxgi: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.presents.insert(
            id,
            Tracked {
                start: event.time,
                process: event.process,
                thread: event.thread,
                driver_thread: 0,
                time_in_present: 0,
                dxgi,
                swap_chain: 0,
                flags: 0,
                submit_sequence: 0,
                win32k_key: None,
                token: 0,
                window: 0,
                ring: 0,
                mode: Mode::Unknown,
                final_state: Final::Unknown,
                screen: None,
                tearing: false,
                flip_delay: 0,
                dependents: Vec::new(),
                wait_for_flip: false,
                wait_for_mpo_flip: false,
                wait_for_sync_dpc: false,
                seen_dxgk_present: false,
                seen_win32k: false,
                seen_in_frame: false,
                completed: false,
                lost: false,
                failed: false,
                waiting_for_dwm: false,
                waiting_for_present_stop: false,
            },
        );
        id
    }

    fn track(&mut self, id: u64) {
        if let Some(old) = self.ring[self.next_ring] {
            self.remove_lost(old);
        }
        self.ring[self.next_ring] = Some(id);
        self.get_mut(id).ring = self.next_ring;
        self.next_ring = (self.next_ring + 1) % RING;
        let (process, start, thread) = {
            let present = self.get(id);
            (present.process, present.start, present.thread)
        };
        self.ordered.entry(process).or_default().entry(start).or_insert(id);
        self.set_thread_present(thread, id);
    }

    fn set_thread_present(&mut self, thread: u32, id: u64) {
        if let Some(old) = self.by_thread.get(&thread).copied() {
            self.remove_lost(old);
        }
        self.by_thread.entry(thread).or_insert(id);
    }

    // The present that a kernel or Win32k event on a thread is about: the
    // thread's present, or the oldest that hasn't been seen by the kernel
    // yet, which a driver's thread presents later, or else a new one.
    fn find_or_create(&mut self, event: &Event) -> Option<u64> {
        if let Some(id) = self.by_thread.get(&event.thread).copied() {
            return Some(id);
        }
        let deferred = self.ordered.get(&event.process).and_then(|ordered| {
            ordered.values().copied().find(|id| {
                let present = &self.presents[id];
                present.driver_thread == 0 && !present.seen_dxgk_present && !present.seen_win32k && present.mode == Mode::Unknown
            })
        });
        if let Some(id) = deferred {
            self.get_mut(id).driver_thread = event.thread;
            self.set_thread_present(event.thread, id);
            return Some(id);
        }
        if self.tracked(event.process) {
            let id = self.create(event, false);
            self.track(id);
            return Some(id);
        }
        None
    }

    // The oldest present with the submit sequence, of any queue.
    fn by_submit(&self, sequence: u32) -> Option<u64> {
        self.by_submit_sequence
            .get(&sequence)?
            .iter()
            .map(|(_, id)| *id)
            .min_by_key(|id| self.presents[id].start)
    }

    fn remove_from_submit_sequence(&mut self, id: u64) {
        let sequence = self.get(id).submit_sequence;
        if sequence != 0
            && let Some(entries) = self.by_submit_sequence.get_mut(&sequence)
        {
            if entries.len() == 1 {
                if entries[0].1 == id {
                    self.by_submit_sequence.remove(&sequence);
                }
            } else if let Some(index) = entries.iter().position(|(_, known)| *known == id) {
                entries.remove(index);
            }
        }
        self.get_mut(id).submit_sequence = 0;
    }

    fn set_screen_time(&mut self, id: u64, time: i64) {
        let present = self.get_mut(id);
        if present.screen.is_none() {
            present.final_state = Final::Presented;
        }
        present.screen = Some(time);
    }

    fn present_stop(&mut self, event: &Event, result: u32) {
        let Some(id) = self.by_thread.get(&event.thread).copied() else {
            return;
        };
        let present = self.get_mut(id);
        present.dxgi = true;
        present.time_in_present = event.time - present.start;
        if present.waiting_for_present_stop {
            present.waiting_for_present_stop = false;
            self.by_thread.remove(&event.thread);
            self.emit_ready();
            return;
        }
        if (result as i32) < 0 {
            present.failed = true;
            self.complete(id);
            return;
        }
        if matches!(result, STATUS_OCCLUDED | STATUS_MODE_CHANGE_IN_PROGRESS | STATUS_NO_DESKTOP_ACCESS) {
            present.final_state = Final::Discarded;
            self.complete(id);
            return;
        }
        self.by_thread.remove(&event.thread);
    }

    fn present_history_start(&mut self, event: &Event, token: u64, model: u32) {
        let id = loop {
            let Some(id) = self.find_or_create(event) else {
                return;
            };
            if self.get(id).token != 0 {
                self.remove_lost(id);
                continue;
            }
            break id;
        };
        let present = self.get_mut(id);
        present.tearing = false;
        present.token = token;
        if present.screen.is_some() {
            present.screen = None;
            present.final_state = Final::Unknown;
        }
        if let Some(other) = self.by_token.get(&token).copied()
            && other != id
        {
            self.remove_lost(other);
        }
        self.by_token.insert(token, id);
        if self.get(id).mode == Mode::Unknown {
            if model == MODEL_REDIRECTED_COMPOSITION {
                self.remove_lost(id);
                return;
            }
            self.get_mut(id).mode = Mode::ComposedFlip;
        }
    }

    // A flip that DWM, or a legacy fullscreen present, made: the present
    // that it's of.
    fn flip(&mut self, event: &Event) -> Option<u64> {
        let mut usable = None;
        while let Some(id) = self.by_thread.get(&event.thread).copied() {
            let present = self.get(id);
            if present.seen_dxgk_present || (present.submit_sequence != 0 && present.time_in_present != 0 && present.dxgi) {
                self.remove_lost(id);
                continue;
            }
            if present.dxgi {
                if present.mode != Mode::Unknown {
                    return None;
                }
                usable = Some(id);
            } else {
                self.by_thread.remove(&event.thread);
            }
            break;
        }
        let id = match usable {
            Some(id) => id,
            None if self.tracked(event.process) => {
                let id = self.create(event, false);
                self.track(id);
                id
            }
            None => return None,
        };
        self.get_mut(id).mode = Mode::LegacyFlip;
        if event.thread == self.dwm_thread {
            let waiting = std::mem::take(&mut self.waiting_for_dwm);
            for dependent in &waiting {
                self.get_mut(*dependent).waiting_for_dwm = false;
            }
            let present = self.get_mut(id);
            self.waiting_for_dwm = std::mem::replace(&mut present.dependents, waiting);
            for dependent in self.waiting_for_dwm.clone() {
                self.get_mut(dependent).waiting_for_dwm = true;
            }
        }
        Some(id)
    }

    fn token_state_changed(&mut self, id: u64, key: (u64, u64, u64), state: u32, independent_flip: bool) {
        match state {
            IN_FRAME => {
                let present = self.get_mut(id);
                present.seen_in_frame = true;
                if independent_flip && present.mode == Mode::ComposedFlip {
                    present.mode = Mode::IndependentFlip;
                }
                let window = present.window;
                if window != 0 {
                    match self.last_by_window.get(&window).copied() {
                        None => {
                            self.last_by_window.insert(window, id);
                        }
                        Some(previous) if previous != id => {
                            self.last_by_window.insert(window, id);
                            if !self.get(previous).wait_for_sync_dpc {
                                if self.get(previous).screen.is_none() {
                                    self.get_mut(previous).final_state = Final::Discarded;
                                }
                                self.remove_from_submit_sequence(previous);
                            }
                        }
                        Some(_) => {}
                    }
                }
            }
            CONFIRMED => {
                let present = self.get(id);
                if present.final_state == Final::Unknown && present.flags & PRESENT_DO_NOT_SEQUENCE != 0 {
                    self.get_mut(id).final_state = Final::Discarded;
                    self.remove_from_submit_sequence(id);
                }
                let window = self.get(id).window;
                if window != 0 {
                    self.last_by_window.remove(&window);
                }
            }
            DISCARDED => {
                self.by_win32k_key.remove(&key);
                let present = self.get_mut(id);
                present.win32k_key = None;
                if !present.seen_in_frame && present.final_state == Final::Unknown {
                    present.final_state = Final::Discarded;
                    self.complete(id);
                } else if present.mode != Mode::ComposedFlip {
                    self.complete(id);
                }
            }
            _ => {}
        }
    }

    fn remove_lost(&mut self, id: u64) {
        let present = self.get_mut(id);
        present.lost = true;
        if present.completed {
            if present.waiting_for_present_stop {
                present.waiting_for_present_stop = false;
                let (thread, driver_thread) = (present.thread, present.driver_thread);
                self.unmap_thread(thread, id);
                self.unmap_thread(driver_thread, id);
                self.emit_ready();
            }
        } else {
            self.complete(id);
        }
    }

    fn unmap_thread(&mut self, thread: u32, id: u64) {
        if self.by_thread.get(&thread) == Some(&id) {
            self.by_thread.remove(&thread);
        }
    }

    fn complete(&mut self, id: u64) {
        // The first present that completes, and every one under way then,
        // are taken as lost, as their earlier events may not have been seen.
        if !self.completed_one && !self.get(id).lost {
            loop {
                let first = self.ordered.values().find_map(|ordered| ordered.values().next().copied());
                let Some(first) = first else {
                    break;
                };
                if self.get(first).completed {
                    // Already removed from the maps' view; take it out.
                    self.stop_tracking(first);
                    continue;
                }
                self.remove_lost(first);
            }
            self.completed_one = true;
            return;
        }
        if self.get(id).completed {
            return;
        }
        let present = self.get_mut(id);
        present.completed = true;
        if !present.lost && present.dxgi && present.time_in_present == 0 {
            present.waiting_for_present_stop = true;
        }

        // DWM's present: what it showed shows the composed presents with it,
        // only the newest of each window's.
        let dependents = std::mem::take(&mut self.get_mut(id).dependents);
        if !dependents.is_empty() {
            let (lost, final_state, screen) = {
                let present = self.get(id);
                (present.lost, present.final_state, present.screen)
            };
            if lost {
                for dependent in &dependents {
                    if let Some(present) = self.presents.get_mut(dependent) {
                        present.lost = true;
                    }
                }
            } else {
                let mut windows = Vec::new();
                for dependent in dependents.iter().rev() {
                    // One that completed already, and was handed on, is gone.
                    let Some(present) = self.presents.get_mut(dependent) else {
                        continue;
                    };
                    if present.completed {
                        continue;
                    }
                    let composed = present.mode == Mode::ComposedFlip;
                    if composed && windows.contains(&present.window) {
                        present.final_state = Final::Discarded;
                        present.screen = None;
                    } else if present.final_state != Final::Discarded {
                        present.final_state = final_state;
                        if screen.is_some() {
                            present.screen = screen;
                        }
                    }
                    if composed {
                        windows.push(present.window);
                    }
                }
            }
            for dependent in dependents.iter().rev() {
                if self.presents.get(dependent).is_some_and(|present| !present.completed) {
                    self.complete(*dependent);
                }
            }
        }

        // A present on the screen completes the swap chain's earlier ones.
        if self.get(id).final_state == Final::Presented {
            let (process, start, swap_chain, lost) = {
                let present = self.get(id);
                (present.process, present.start, present.swap_chain, present.lost)
            };
            let earlier: Vec<u64> = self
                .ordered
                .get(&process)
                .map(|ordered| ordered.range(..start).map(|(_, id)| *id).collect())
                .unwrap_or_default();
            for other in earlier {
                let still_ordered = self.ordered.get(&process).is_some_and(|ordered| ordered.values().any(|known| *known == other));
                if !still_ordered || self.get(other).swap_chain != swap_chain {
                    continue;
                }
                if lost {
                    self.get_mut(other).lost = true;
                }
                self.complete(other);
            }
        }

        let start = self.get(id).start;
        self.stop_tracking(id);
        self.completed.push_back(id);
        self.emit_ready();

        // A present that waits too long for its Present_Stop stops waiting.
        if let Some(oldest) = self.completed.front().copied()
            && self.get(oldest).waiting_for_present_stop
            && start - self.get(oldest).start > 2 * crate::frequency()
        {
            let present = self.get_mut(oldest);
            present.waiting_for_present_stop = false;
            present.lost = true;
            self.stop_tracking(oldest);
            self.emit_ready();
        }
    }

    fn stop_tracking(&mut self, id: u64) {
        let present = self.get(id);
        let (ring, thread, driver_thread, process, start, key, token, window, waiting_for_stop, waiting_for_dwm) = (
            present.ring,
            present.thread,
            present.driver_thread,
            present.process,
            present.start,
            present.win32k_key,
            present.token,
            present.window,
            present.waiting_for_present_stop,
            present.waiting_for_dwm,
        );
        if self.ring[ring] == Some(id) {
            self.ring[ring] = None;
        }
        if !waiting_for_stop {
            self.unmap_thread(thread, id);
        }
        self.unmap_thread(driver_thread, id);
        if let Some(ordered) = self.ordered.get_mut(&process)
            && ordered.get(&start) == Some(&id)
        {
            ordered.remove(&start);
        }
        self.remove_from_submit_sequence(id);
        if let Some(key) = key
            && self.by_win32k_key.get(&key) == Some(&id)
        {
            self.by_win32k_key.remove(&key);
        }
        if token != 0 && self.by_token.get(&token) == Some(&id) {
            self.by_token.remove(&token);
        }
        if window != 0 && self.last_by_window.get(&window) == Some(&id) {
            self.last_by_window.remove(&window);
        }
        if waiting_for_dwm {
            self.waiting_for_dwm.retain(|known| *known != id);
        }
        let present = self.get_mut(id);
        present.win32k_key = None;
        present.token = 0;
        present.window = 0;
        present.waiting_for_dwm = false;
    }

    // Hands on the completed presents at the front that no longer wait for
    // their Present_Stop: the process's own, as Presents.
    fn emit_ready(&mut self) {
        while let Some(id) = self.completed.front().copied() {
            if self.get(id).waiting_for_present_stop {
                break;
            }
            self.completed.pop_front();
            let present = self.presents.remove(&id).unwrap();
            if present.process == self.process && present.dxgi && !present.lost && !present.failed {
                self.output.push(Present {
                    start: present.start,
                    thread: present.thread,
                    mode: match present.mode {
                        Mode::IndependentFlip => PresentMode::IndependentFlip,
                        Mode::ComposedIndependentFlip => PresentMode::OverlayPlane,
                        Mode::ComposedFlip => PresentMode::Composed,
                        Mode::LegacyFlip | Mode::Unknown => PresentMode::Unknown,
                    },
                    tearing: present.tearing,
                    screen: if present.final_state == Final::Presented { present.screen } else { None },
                    flip_delay: present.flip_delay,
                });
            }
        }
    }
}
