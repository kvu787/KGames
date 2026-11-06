// The ETW events that tell when presents reach the screen, from DXGI and
// Direct3D 9, the graphics kernel (DxgKrnl), Win32k and the desktop window
// manager (DWM), and NVIDIA's display driver, as plain data. The session
// enables these providers with these events, the levels and the keywords
// that PresentMon uses, so that the times are the same as PresentMon's.

use windows_api::EVENT_RECORD;
use windows_core::GUID;

use crate::decoder::{Decoder, METADATA, payload, read};

pub const DXGKRNL: GUID = GUID::from_u128(0x802ec45a_1e99_4b83_9920_87c98277ba9d);
pub const WIN32K: GUID = GUID::from_u128(0x8c416c79_d49b_4f01_a467_e56d3aa8234c);
pub const DWM: GUID = GUID::from_u128(0x9e9bba3c_2e38_40cb_99f4_9e8281425164);
pub const DXGI: GUID = GUID::from_u128(0xca11c036_0102_4a2d_a6ad_f03cfed5d3c9);
pub const D3D9: GUID = GUID::from_u128(0x783aca0a_790e_4d7f_8451_aa850511c6b9);
pub const NVIDIA: GUID = GUID::from_u128(0xae4f8626_8265_40d1_a70b_11b64240e8e9);

// A provider to enable: its level, keywords, and the events to let through.
pub struct Provider {
    pub guid: GUID,
    pub level: u8,
    pub any: u64,
    pub all: u64,
    pub events: &'static [u16],
}

// In the order that PresentMon enables them: the kernel's first, so that no
// present is under way before its events come.
pub const PROVIDERS: [Provider; 6] = [
    Provider {
        guid: DXGKRNL,
        level: 4,
        any: 0x0800_0001,
        all: 0x1,
        events: &[
            0xAB, 0xA6, 0x1F5, 0xA8, 0x10A, 0xFC, 0x17E, 0x111, 0x74, 0x103, 0xB8, 0xAC, 0xD7, 0xB2, 0xF4, 0xB4, 0x11,
        ],
    },
    Provider {
        guid: WIN32K,
        level: 4,
        any: 0x8000_0004_0000_1000,
        all: 0x8000_0000_0000_1000,
        events: &[0xC9, 0x12D],
    },
    Provider {
        guid: DWM,
        level: 5,
        any: 0x8000_0000_0000_0081,
        all: 0,
        events: &[0x40, 0x0F, 0xC4, 0x45, 0x46, 0x65],
    },
    Provider {
        guid: DXGI,
        level: 0,
        any: 0x8000_0000_0000_0002,
        all: 0x8000_0000_0000_0002,
        events: &[0x2A, 0x2B, 0x37, 0x38],
    },
    Provider {
        guid: D3D9,
        level: 0,
        any: 0x8000_0000_0000_0002,
        all: 0x8000_0000_0000_0002,
        events: &[0x1, 0x2],
    },
    Provider {
        guid: NVIDIA,
        level: 4,
        any: 0x1000_0000_0000_0000,
        all: 0x1000_0000_0000_0000,
        events: &[0x1],
    },
];

// An event, with what's in its header.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    // As the performance counter's ticks.
    pub time: i64,
    pub process: u32,
    pub thread: u32,
    pub kind: Kind,
}

// The API whose Present call a present came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runtime {
    Dxgi,
    D3d9,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    // DXGI's and Direct3D 9's Present calls. Direct3D 9's flags are DXGI's
    // that mean the same.
    PresentStart { runtime: Runtime, swap_chain: u64, flags: u32, sync_interval: i32 },
    PresentStop { runtime: Runtime, result: u32 },
    // Win32k: a present's token for composition, and its state changes.
    TokenCompositionSurfaceObject { key: (u64, u64, u64) },
    TokenStateChanged { key: (u64, u64, u64), state: u32, independent_flip: bool },
    // DxgKrnl.
    PresentHistoryStart { token: u64, model: u32, token_data: u64 },
    PresentHistoryInfo { token: u64 },
    QueuePacketStart { packet_type: u32, submit_sequence: u32, context: u64, present: bool },
    QueuePacketStop { context: u64, submit_sequence: u32 },
    PresentInfo { window: u64 },
    // A bitblt model present's copy, to the screen or else to the window's
    // surface for DWM, and the kernel finding the copy unneeded.
    Blit { window: u64, redirected: bool },
    BlitCancel,
    Flip { interval: u32, mmio: bool },
    FlipMultiPlaneOverlay { source: u32, layer: u32 },
    IndependentFlip { submit_sequence: u32, interval: u32 },
    MmioFlip { submit_sequence: u32, flags: u32 },
    // The status after the flip is None before the event's version 2.
    MmioFlipMultiPlaneOverlay { submit_sequence: u32, status: Option<u32> },
    VSyncDpc { fence: u64 },
    SyncDpcMultiPlane { planes: Vec<u64>, flip_sequences: Vec<u64> },
    // DWM.
    DwmSchedulePresent,
    DwmGetPresentHistory,
    DwmScheduleSurfaceUpdate { key: (u64, u64, u64) },
    // DWM taking a present that was copied on the CPU, by its present
    // history token's data.
    DwmFlipChain { token_data: u64, window: u64 },
    // NVIDIA's display driver: when it means to flip.
    NvidiaFlipRequest { alloc: u64, source: u32, time: u64, token: u32 },
}

// Direct3D 9's Present flags, and the DXGI flags that mean the same.
const D3DPRESENT_DONOTWAIT: u32 = 0x1;
const D3DPRESENT_DONOTFLIP: u32 = 0x4;
const D3DPRESENT_FLIPRESTART: u32 = 0x8;
const D3DPRESENT_FORCEIMMEDIATE: u32 = 0x100;
const DXGI_PRESENT_DO_NOT_SEQUENCE: u32 = 0x2;
const DXGI_PRESENT_RESTART: u32 = 0x4;
const DXGI_PRESENT_DO_NOT_WAIT: u32 = 0x8;

fn dxgi_flags(flags: u32) -> u32 {
    [
        (D3DPRESENT_DONOTFLIP, DXGI_PRESENT_DO_NOT_SEQUENCE),
        (D3DPRESENT_DONOTWAIT, DXGI_PRESENT_DO_NOT_WAIT),
        (D3DPRESENT_FLIPRESTART, DXGI_PRESENT_RESTART),
    ]
    .iter()
    .filter(|(d3d9, _)| flags & d3d9 != 0)
    .fold(0, |dxgi, (_, flag)| dxgi | flag)
}

// Win32k's token states.
pub const IN_FRAME: u32 = 3;
pub const CONFIRMED: u32 = 4;
pub const DISCARDED: u32 = 6;

// The event as plain data, or None if it's not one of these, or its fields
// aren't there.
pub fn decode(decoder: &mut Decoder, record: &EVENT_RECORD) -> Option<Event> {
    let header = &record.EventHeader;
    let id = header.EventDescriptor.Id;
    let version = header.EventDescriptor.Version;
    let provider = header.ProviderId;
    if provider == METADATA {
        decoder.add_metadata(record);
        return None;
    }
    let kind = if provider == NVIDIA {
        // NVIDIA's provider registers no manifest, so its one event is read
        // by its fixed layout.
        let data = payload(record);
        if id != 1 || data.len() < 44 {
            return None;
        }
        Kind::NvidiaFlipRequest {
            token: read(data, 8, 4)? as u32,
            time: read(data, 16, 8)?,
            source: read(data, 28, 4)? as u32,
            alloc: read(data, 36, 8)?,
        }
    } else {
        // Finding an event's fields is most of the cost of decoding, and
        // the providers send events that aren't read, so those go first.
        let read = matches!(
            (provider, id),
            (DXGI, 0x2A | 0x37 | 0x2B | 0x38)
                | (D3D9, 0x1 | 0x2)
                | (WIN32K, 0xC9 | 0x12D)
                | (DXGKRNL, 0xAB | 0xD7 | 0xAC | 0xB2 | 0xB4 | 0xB8 | 0xA6 | 0x1F5 | 0xA8 | 0xFC | 0x10A | 0x74 | 0x103 | 0x11 | 0x111 | 0x17E)
                | (DWM, 0x0F | 0x40 | 0xC4 | 0x45 | 0x46 | 0x65)
        );
        if !read {
            return None;
        }
        let fields = decoder.fields(record)?;
        match (provider, id) {
            (DXGI, 0x2A | 0x37) => Kind::PresentStart {
                runtime: Runtime::Dxgi,
                swap_chain: fields.u64("pIDXGISwapChain")?,
                flags: fields.u32("Flags")?,
                sync_interval: fields.u32("SyncInterval")? as i32,
            },
            (DXGI, 0x2B | 0x38) => Kind::PresentStop { runtime: Runtime::Dxgi, result: fields.u32("Result")? },
            (D3D9, 0x1) => {
                let flags = fields.u32("Flags")?;
                Kind::PresentStart {
                    runtime: Runtime::D3d9,
                    swap_chain: fields.u64("pSwapchain")?,
                    flags: dxgi_flags(flags),
                    sync_interval: if flags & D3DPRESENT_FORCEIMMEDIATE != 0 { 0 } else { -1 },
                }
            }
            (D3D9, 0x2) => Kind::PresentStop { runtime: Runtime::D3d9, result: fields.u32("Result")? },
            (WIN32K, 0xC9) => Kind::TokenCompositionSurfaceObject {
                key: (fields.u64("CompositionSurfaceLuid")?, fields.u64("PresentCount")?, fields.u64("BindId")?),
            },
            (WIN32K, 0x12D) => {
                let state = fields.u32("NewState")?;
                Kind::TokenStateChanged {
                    key: (fields.u64("CompositionSurfaceLuid")?, fields.u64("PresentCount")?, fields.u64("BindId")?),
                    state,
                    independent_flip: state == IN_FRAME && fields.u32("IndependentFlip").unwrap_or(0) != 0,
                }
            }
            (DXGKRNL, 0xAB | 0xD7) => Kind::PresentHistoryStart {
                token: fields.u64("Token")?,
                model: fields.u32("Model")?,
                token_data: fields.u64("TokenData").unwrap_or(0),
            },
            (DXGKRNL, 0xAC) => Kind::PresentHistoryInfo { token: fields.u64("Token")? },
            (DXGKRNL, 0xB2) => Kind::QueuePacketStart {
                packet_type: fields.u32("PacketType")?,
                submit_sequence: fields.u32("SubmitSequence")?,
                context: fields.u64("hContext")?,
                present: fields.u32("bPresent")? != 0,
            },
            (DXGKRNL, 0xB4) => Kind::QueuePacketStop {
                context: fields.u64("hContext")?,
                submit_sequence: fields.u32("SubmitSequence")?,
            },
            (DXGKRNL, 0xB8) => Kind::PresentInfo { window: fields.u64("hWindow")? },
            (DXGKRNL, 0xA6) => Kind::Blit {
                window: fields.u64("hwnd")?,
                redirected: fields.u32("bRedirectedPresent")? != 0,
            },
            (DXGKRNL, 0x1F5) => Kind::BlitCancel,
            (DXGKRNL, 0xA8) => Kind::Flip {
                interval: fields.u32("FlipInterval")?,
                mmio: fields.u32("MMIOFlip")? != 0,
            },
            (DXGKRNL, 0xFC) => Kind::FlipMultiPlaneOverlay {
                source: fields.u32("VidPnSourceId")?,
                layer: fields.u32("LayerIndex")?,
            },
            (DXGKRNL, 0x10A) => Kind::IndependentFlip {
                submit_sequence: fields.u32("SubmitSequence")?,
                interval: fields.u32("FlipInterval")?,
            },
            (DXGKRNL, 0x74) => Kind::MmioFlip {
                submit_sequence: fields.u32("FlipSubmitSequence")?,
                flags: fields.u32("Flags")?,
            },
            (DXGKRNL, 0x103) => Kind::MmioFlipMultiPlaneOverlay {
                submit_sequence: (fields.u64("FlipSubmitSequence")? >> 32) as u32,
                status: if version >= 2 { fields.u32("FlipEntryStatusAfterFlip") } else { None },
            },
            (DXGKRNL, 0x11) => Kind::VSyncDpc { fence: fields.u64("FlipFenceId")? },
            (DXGKRNL, 0x111 | 0x17E) => {
                let addresses = if id == 0x111 && version >= 1 { "PresentIdOrPhysicalAddress" } else { "ScannedPhysicalAddress" };
                Kind::SyncDpcMultiPlane {
                    planes: fields.array(addresses)?,
                    flip_sequences: fields.array("FlipSubmitSequence")?,
                }
            }
            (DWM, 0x0F) => Kind::DwmSchedulePresent,
            (DWM, 0x40) => Kind::DwmGetPresentHistory,
            (DWM, 0xC4) => Kind::DwmScheduleSurfaceUpdate {
                key: (fields.u64("luidSurface")?, fields.u64("PresentCount")?, fields.u64("bindId")?),
            },
            // The flip chain and the serial number are 4 bytes each, or 8 on
            // Windows 8.1, and make the token data's high and low halves.
            (DWM, 0x45 | 0x46 | 0x65) => Kind::DwmFlipChain {
                token_data: (fields.u64("ulFlipChain")? << 32) | (fields.u64("ulSerialNumber")? & 0xFFFF_FFFF),
                window: fields.u64("hwnd")?,
            },
            _ => return None,
        }
    };
    Some(Event {
        time: header.TimeStamp,
        process: header.ProcessId,
        thread: header.ThreadId,
        kind,
    })
}
