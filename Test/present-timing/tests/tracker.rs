// Tests that the tracker follows presents through Windows' events to when
// the display took them, as PresentMon does, with made-up events in the
// order that Windows sends them, for each mode: the display taking the
// buffer itself, with and without tearing, and on an overlay plane, DWM
// composing it into its own frame, fullscreen exclusive flips and copies,
// and copies that DWM composes, made on the GPU and on the CPU.

use present_timing::events::{CONFIRMED, Event, IN_FRAME, Kind, Runtime};
use present_timing::tracker::Tracker;
use present_timing::{Corrector, Present, PresentMode};

const PROCESS: u32 = 100;
const THREAD: u32 = 7;
const DWM_PROCESS: u32 = 50;
const DWM_THREAD: u32 = 51;
const WINDOW: u64 = 0x99;
const SWAP_CHAIN: u64 = 0xABC;
const ALLOW_TEARING: u32 = 0x200;
const FLIP_WAIT_VSYNC: u32 = 5;
const FLIP_WAIT_COMPLETE: u32 = 11;

fn event(time: i64, process: u32, thread: u32, kind: Kind) -> Event {
    Event { time, process, thread, kind }
}

// What the process's thread sends for a Present call that starts at a time,
// with the submit sequence of its queue packet.
fn present(tracker: &mut Tracker, start: i64, sequence: u32, flags: u32) {
    let key = (1, sequence as u64, 1);
    for (offset, kind) in [
        (0, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags, sync_interval: 0 }),
        (1, Kind::TokenCompositionSurfaceObject { key }),
        (2, Kind::PresentHistoryStart { token: 1000 + sequence as u64, model: 3, token_data: 0 }),
        (3, Kind::QueuePacketStart { packet_type: 0, submit_sequence: sequence, context: 0x5, present: true }),
        (4, Kind::PresentInfo { window: WINDOW }),
        (5, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 }),
    ] {
        tracker.handle(&event(start + offset, PROCESS, THREAD, kind));
    }
}

fn in_frame(tracker: &mut Tracker, time: i64, sequence: u32, independent_flip: bool) {
    let key = (1, sequence as u64, 1);
    tracker.handle(&event(time, 4, 4, Kind::TokenStateChanged { key, state: IN_FRAME, independent_flip }));
}

// The display's interrupt that says which flips are on the screen.
fn sync_dpc(tracker: &mut Tracker, time: i64, sequences: &[u32]) {
    tracker.handle(&event(
        time,
        0,
        0,
        Kind::SyncDpcMultiPlane {
            planes: vec![0x1000],
            flip_sequences: sequences.iter().map(|sequence| (*sequence as u64) << 32).collect(),
        },
    ));
}

// A present that the display takes at once, mid-refresh: on the screen as
// the flip happens.
fn torn(tracker: &mut Tracker, start: i64, sequence: u32) {
    present(tracker, start, sequence, ALLOW_TEARING);
    in_frame(tracker, start + 6, sequence, true);
    tracker.handle(&event(
        start + 10,
        0,
        0,
        Kind::MmioFlipMultiPlaneOverlay { submit_sequence: sequence, status: Some(FLIP_WAIT_COMPLETE) },
    ));
    sync_dpc(tracker, start + 20, &[sequence]);
}

#[test]
fn the_first_present_to_complete_and_those_under_way_are_lost() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    assert_eq!(tracker.take(), Vec::new());
    torn(&mut tracker, 200, 2);
    assert_eq!(tracker.take().len(), 1);
}

#[test]
fn a_torn_flip_is_on_the_screen_when_it_flips() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    torn(&mut tracker, 200, 2);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::IndependentFlip,
            tearing: true,
            screen: Some(210),
            flip_delay: 0,
        }]
    );
}

#[test]
fn a_flip_that_waits_for_vertical_sync_is_on_the_screen_at_the_sync() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    present(&mut tracker, 200, 2, 0);
    in_frame(&mut tracker, 206, 2, true);
    tracker.handle(&event(210, 0, 0, Kind::MmioFlipMultiPlaneOverlay { submit_sequence: 2, status: Some(FLIP_WAIT_VSYNC) }));
    assert_eq!(tracker.take(), Vec::new());
    tracker.handle(&event(250, 0, 0, Kind::VSyncDpc { fence: 2 << 32 }));
    sync_dpc(&mut tracker, 251, &[2]);
    let presents = tracker.take();
    assert_eq!(presents.len(), 1);
    assert_eq!(presents[0].screen, Some(250));
    assert_eq!(presents[0].mode, PresentMode::IndependentFlip);
    assert!(!presents[0].tearing);
}

#[test]
fn a_flip_with_other_planes_on_the_screen_is_on_an_overlay_plane() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    present(&mut tracker, 200, 2, ALLOW_TEARING);
    in_frame(&mut tracker, 206, 2, true);
    tracker.handle(&event(210, 0, 0, Kind::MmioFlipMultiPlaneOverlay { submit_sequence: 2, status: Some(FLIP_WAIT_COMPLETE) }));
    tracker.handle(&event(220, 0, 0, Kind::SyncDpcMultiPlane { planes: vec![0x1000, 0x2000], flip_sequences: vec![2 << 32] }));
    assert_eq!(tracker.take()[0].mode, PresentMode::OverlayPlane);
}

// DWM's frame: it schedules its present, flips on an overlay plane's path,
// queues the flip with a submit sequence, waits for vertical sync, and the
// display takes it at the sync.
fn dwm_frame(tracker: &mut Tracker, time: i64, sequence: u32) {
    tracker.handle(&event(time, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    tracker.handle(&event(time + 1, DWM_PROCESS, DWM_THREAD, Kind::FlipMultiPlaneOverlay { source: 0, layer: 0 }));
    tracker.handle(&event(
        time + 2,
        DWM_PROCESS,
        DWM_THREAD,
        Kind::QueuePacketStart { packet_type: 3, submit_sequence: sequence, context: 0x9, present: false },
    ));
    tracker.handle(&event(time + 3, 0, 0, Kind::MmioFlipMultiPlaneOverlay { submit_sequence: sequence, status: Some(FLIP_WAIT_VSYNC) }));
    tracker.handle(&event(time + 10, 0, 0, Kind::VSyncDpc { fence: (sequence as u64) << 32 }));
    sync_dpc(tracker, time + 11, &[sequence]);
}

// A present that DWM will compose: in its next frame, and scheduled with it.
fn composed(tracker: &mut Tracker, start: i64, sequence: u32) {
    present(tracker, start, sequence, 0);
    in_frame(tracker, start + 6, sequence, false);
    tracker.handle(&event(start + 7, DWM_PROCESS, DWM_THREAD, Kind::DwmScheduleSurfaceUpdate { key: (1, sequence as u64, 1) }));
}

#[test]
fn a_composed_present_is_on_the_screen_with_dwm_s_frame() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    composed(&mut tracker, 200, 2);
    dwm_frame(&mut tracker, 300, 900);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::ComposedFlip,
            tearing: false,
            screen: Some(310),
            flip_delay: 0,
        }]
    );
}

#[test]
fn of_two_presents_in_one_dwm_frame_only_the_newer_is_on_the_screen() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    composed(&mut tracker, 200, 2);
    composed(&mut tracker, 250, 3);
    dwm_frame(&mut tracker, 300, 900);
    let mut presents = tracker.take();
    presents.sort_by_key(|present| present.start);
    let screens: Vec<(i64, Option<i64>)> = presents.iter().map(|present| (present.start, present.screen)).collect();
    assert_eq!(screens, vec![(200, None), (250, Some(310))]);
}

#[test]
fn a_present_that_dxgi_says_is_occluded_never_reaches_the_screen() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    tracker.handle(&event(200, PROCESS, THREAD, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: 0 }));
    tracker.handle(&event(205, PROCESS, THREAD, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0x087A_0001 }));
    let presents = tracker.take();
    assert_eq!(presents.len(), 1);
    assert_eq!(presents[0].screen, None);
}

#[test]
fn another_process_s_presents_are_not_reported() {
    let mut tracker = Tracker::new(PROCESS + 1);
    torn(&mut tracker, 100, 1);
    torn(&mut tracker, 200, 2);
    assert_eq!(tracker.take(), Vec::new());
}

#[test]
fn a_confirmed_token_lets_the_window_s_next_present_in() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    composed(&mut tracker, 200, 2);
    tracker.handle(&event(260, 4, 4, Kind::TokenStateChanged { key: (1, 2, 1), state: CONFIRMED, independent_flip: false }));
    dwm_frame(&mut tracker, 300, 900);
    assert_eq!(tracker.take()[0].screen, Some(310));
}

#[test]
fn a_present_that_direct3d_9_says_is_occluded_never_reaches_the_screen() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    tracker.handle(&event(200, PROCESS, THREAD, Kind::PresentStart { runtime: Runtime::D3d9, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: -1 }));
    tracker.handle(&event(205, PROCESS, THREAD, Kind::PresentStop { runtime: Runtime::D3d9, result: 0x0876_0878 }));
    let presents = tracker.take();
    assert_eq!(presents.len(), 1);
    assert_eq!(presents[0].screen, None);
}

#[test]
fn a_present_of_an_api_without_events_of_its_own_is_reported() {
    // Like OpenGL's or Vulkan's, which only the kernel's and Win32k's
    // events tell of.
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    let key = (1, 2, 1);
    tracker.handle(&event(200, PROCESS, THREAD, Kind::TokenCompositionSurfaceObject { key }));
    tracker.handle(&event(201, PROCESS, THREAD, Kind::PresentHistoryStart { token: 1002, model: 3, token_data: 0 }));
    tracker.handle(&event(
        202,
        PROCESS,
        THREAD,
        Kind::QueuePacketStart { packet_type: 0, submit_sequence: 2, context: 0x5, present: true },
    ));
    tracker.handle(&event(203, PROCESS, THREAD, Kind::PresentInfo { window: WINDOW }));
    in_frame(&mut tracker, 206, 2, false);
    tracker.handle(&event(207, DWM_PROCESS, DWM_THREAD, Kind::DwmScheduleSurfaceUpdate { key }));
    dwm_frame(&mut tracker, 300, 900);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::ComposedFlip,
            tearing: false,
            screen: Some(310),
            flip_delay: 0,
        }]
    );
}

// A fullscreen exclusive flip: the kernel's flip on the present's thread,
// its queue packet, and the display taking it at the sync.
fn legacy_flip(tracker: &mut Tracker, start: i64, sequence: u32) {
    for (offset, kind) in [
        (0, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: 1 }),
        (1, Kind::Flip { interval: 1, mmio: true }),
        (2, Kind::QueuePacketStart { packet_type: 3, submit_sequence: sequence, context: 0x5, present: false }),
        (3, Kind::PresentInfo { window: WINDOW }),
        (4, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 }),
    ] {
        tracker.handle(&event(start + offset, PROCESS, THREAD, kind));
    }
    tracker.handle(&event(start + 10, 0, 0, Kind::MmioFlip { submit_sequence: sequence, flags: 0 }));
    tracker.handle(&event(start + 20, 0, 0, Kind::VSyncDpc { fence: (sequence as u64) << 32 }));
}

#[test]
fn a_fullscreen_exclusive_flip_is_a_legacy_flip_on_the_screen_at_the_sync() {
    let mut tracker = Tracker::new(PROCESS);
    legacy_flip(&mut tracker, 100, 1);
    legacy_flip(&mut tracker, 200, 2);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::LegacyFlip,
            tearing: false,
            screen: Some(220),
            flip_delay: 0,
        }]
    );
}

// What a bitblt model swap chain's Present call sends: its copy, and its
// queue packet, and, for a copy to the window's surface for DWM, a present
// history token, whose data is the flip chain and serial number that DWM's
// events name a copy made on the CPU by.
fn blit(tracker: &mut Tracker, start: i64, sequence: u32, redirected: bool, history: Option<u64>) {
    let mut kinds = vec![
        Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: 0 },
        Kind::Blit { window: WINDOW, redirected },
    ];
    if let Some(token_data) = history {
        kinds.push(Kind::PresentHistoryStart { token: 1000 + sequence as u64, model: 3, token_data });
    }
    kinds.extend([
        Kind::QueuePacketStart { packet_type: 0, submit_sequence: sequence, context: 0x5, present: true },
        Kind::PresentInfo { window: WINDOW },
        Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 },
    ]);
    for (offset, kind) in kinds.into_iter().enumerate() {
        tracker.handle(&event(start + offset as i64, PROCESS, THREAD, kind));
    }
}

#[test]
fn a_fullscreen_exclusive_copy_is_on_the_screen_when_its_queue_packet_is_done() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    blit(&mut tracker, 200, 2, false, None);
    tracker.handle(&event(230, 0, 0, Kind::QueuePacketStop { context: 0x5, submit_sequence: 2 }));
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::LegacyCopy,
            tearing: true,
            screen: Some(230),
            flip_delay: 0,
        }]
    );
}

#[test]
fn a_copy_whose_queue_packet_is_done_before_its_present_info_waits_for_it() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    for (time, kind) in [
        (200, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: 0 }),
        (201, Kind::Blit { window: WINDOW, redirected: false }),
        (202, Kind::QueuePacketStart { packet_type: 0, submit_sequence: 2, context: 0x5, present: true }),
    ] {
        tracker.handle(&event(time, PROCESS, THREAD, kind));
    }
    tracker.handle(&event(203, 0, 0, Kind::QueuePacketStop { context: 0x5, submit_sequence: 2 }));
    tracker.handle(&event(204, PROCESS, THREAD, Kind::PresentInfo { window: WINDOW }));
    tracker.handle(&event(205, PROCESS, THREAD, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 }));
    let presents = tracker.take();
    assert_eq!(presents.len(), 1);
    assert_eq!(presents[0].mode, PresentMode::LegacyCopy);
    assert_eq!(presents[0].screen, Some(203));
}

#[test]
fn a_copy_that_the_kernel_cancels_never_reaches_the_screen() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    for (time, kind) in [
        (200, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: 0, sync_interval: 0 }),
        (201, Kind::Blit { window: WINDOW, redirected: false }),
        (202, Kind::BlitCancel),
        (203, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 }),
    ] {
        tracker.handle(&event(time, PROCESS, THREAD, kind));
    }
    let presents = tracker.take();
    assert_eq!(presents.len(), 1);
    assert_eq!(presents[0].screen, None);
}

#[test]
fn a_copy_made_on_the_gpu_for_dwm_is_on_the_screen_with_dwm_s_frame() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    blit(&mut tracker, 200, 2, false, Some(0));
    tracker.handle(&event(210, 0, 0, Kind::PresentHistoryInfo { token: 1002 }));
    tracker.handle(&event(290, DWM_PROCESS, DWM_THREAD, Kind::DwmGetPresentHistory));
    dwm_frame(&mut tracker, 300, 900);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::ComposedGpuCopy,
            tearing: false,
            screen: Some(310),
            flip_delay: 0,
        }]
    );
}

#[test]
fn of_two_copies_in_one_dwm_frame_only_the_newer_is_on_the_screen() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    blit(&mut tracker, 200, 2, false, Some(0));
    tracker.handle(&event(210, 0, 0, Kind::PresentHistoryInfo { token: 1002 }));
    blit(&mut tracker, 250, 3, false, Some(0));
    tracker.handle(&event(260, 0, 0, Kind::PresentHistoryInfo { token: 1003 }));
    tracker.handle(&event(290, DWM_PROCESS, DWM_THREAD, Kind::DwmGetPresentHistory));
    dwm_frame(&mut tracker, 300, 900);
    let mut presents = tracker.take();
    presents.sort_by_key(|present| present.start);
    let screens: Vec<(i64, Option<i64>)> = presents.iter().map(|present| (present.start, present.screen)).collect();
    assert_eq!(screens, vec![(200, None), (250, Some(310))]);
}

#[test]
fn a_copy_made_on_the_cpu_for_dwm_is_on_the_screen_with_dwm_s_frame() {
    let mut tracker = Tracker::new(PROCESS);
    tracker.handle(&event(10, DWM_PROCESS, DWM_THREAD, Kind::DwmSchedulePresent));
    torn(&mut tracker, 100, 1);
    tracker.take();
    let token_data = (7 << 32) | 2;
    blit(&mut tracker, 200, 2, true, Some(token_data));
    tracker.handle(&event(210, 0, 0, Kind::PresentHistoryInfo { token: 1002 }));
    tracker.handle(&event(280, DWM_PROCESS, DWM_THREAD, Kind::DwmFlipChain { token_data, window: WINDOW }));
    tracker.handle(&event(290, DWM_PROCESS, DWM_THREAD, Kind::DwmGetPresentHistory));
    dwm_frame(&mut tracker, 300, 900);
    assert_eq!(
        tracker.take(),
        vec![Present {
            start: 200,
            thread: THREAD,
            mode: PresentMode::ComposedCpuCopy,
            tearing: false,
            screen: Some(310),
            flip_delay: 0,
        }]
    );
}

#[test]
fn nvidia_s_flip_delay_puts_a_flip_later() {
    let mut tracker = Tracker::new(PROCESS);
    torn(&mut tracker, 100, 1);
    tracker.take();
    present(&mut tracker, 200, 2, ALLOW_TEARING);
    in_frame(&mut tracker, 206, 2, true);
    // The driver means to flip at 230, 22 ticks after its request, which the
    // kernel's event on the same thread then comes 2 ticks after.
    tracker.handle(&event(208, 0, 33, Kind::NvidiaFlipRequest { alloc: 0, source: 0, time: 230, token: 1 }));
    tracker.handle(&event(210, 0, 33, Kind::MmioFlipMultiPlaneOverlay { submit_sequence: 2, status: Some(FLIP_WAIT_COMPLETE) }));
    sync_dpc(&mut tracker, 240, &[2]);
    let presents = tracker.take();
    assert_eq!(presents[0].flip_delay, 22);
    assert_eq!(presents[0].screen, Some(232));
}

#[test]
fn a_flip_delayed_past_the_next_puts_the_next_on_the_screen_with_it() {
    let present = |start: i64, screen: i64, flip_delay: i64| Present {
        start,
        thread: THREAD,
        mode: PresentMode::IndependentFlip,
        tearing: true,
        screen: Some(screen),
        flip_delay,
    };
    // In the order they started, whatever order they came in.
    let mut corrector = Corrector::default();
    let corrected = corrector.correct(vec![present(200, 260, 50), present(100, 150, 0)]);
    assert_eq!(corrected.iter().map(|present| present.start).collect::<Vec<_>>(), vec![100, 200]);
    // The one after the delayed one goes on the screen with it.
    let corrected = corrector.correct(vec![present(300, 250, 0)]);
    assert_eq!(corrected[0].screen, Some(260));
    assert_eq!(corrected[0].flip_delay, 10);
    // And so does the one after that, which now has a delay too.
    let corrected = corrector.correct(vec![present(400, 255, 0)]);
    assert_eq!(corrected[0].screen, Some(260));
    // But not after one without a delay.
    let mut corrector = Corrector::default();
    corrector.correct(vec![present(200, 260, 0)]);
    let corrected = corrector.correct(vec![present(300, 250, 0)]);
    assert_eq!(corrected[0].screen, Some(250));
}
