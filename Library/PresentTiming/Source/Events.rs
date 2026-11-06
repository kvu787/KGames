// The ETW events that tell when presents reach the screen, from DXGI, the
// graphics kernel (DxgKrnl), Win32k and the desktop window manager (DWM),
// and NVIDIA's display driver, as plain data. The session enables these
// providers with these events, the levels and the keywords that PresentMon
// uses, so that the times are the same as PresentMon's.

use windows_api::EVENT_RECORD;
use windows_core::GUID;

use crate::decoder::{Decoder, METADATA, payload, read};

pub const DXGKRNL: GUID = GUID::from_u128(0x802ec45a_1e99_4b83_9920_87c98277ba9d);
pub const WIN32K: GUID = GUID::from_u128(0x8c416c79_d49b_4f01_a467_e56d3aa8234c);
pub const DWM: GUID = GUID::from_u128(0x9e9bba3c_2e38_40cb_99f4_9e8281425164);
pub const DXGI: GUID = GUID::from_u128(0xca11c036_0102_4a2d_a6ad_f03cfed5d3c9);
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
pub const PROVIDERS: [Provider; 5] = [
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

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    // DXGI.
    PresentStart { swap_chain: u64, flags: u32, sync_interval: i32 },
    PresentStop { result: u32 },
    // Win32k: a present's token for composition, and its state changes.
    TokenCompositionSurfaceObject { key: (u64, u64, u64) },
    TokenStateChanged { key: (u64, u64, u64), state: u32, independent_flip: bool },
    // DxgKrnl.
    PresentHistoryStart { token: u64, model: u32 },
    PresentHistoryInfo { token: u64 },
    QueuePacketStart { packet_type: u32, submit_sequence: u32, context: u64, present: bool },
    QueuePacketStop { context: u64, submit_sequence: u32 },
    PresentInfo { window: u64 },
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
    // NVIDIA's display driver: when it means to flip.
    NvidiaFlipRequest { alloc: u64, source: u32, time: u64, token: u32 },
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
        let fields = decoder.fields(record)?;
        match (provider, id) {
            (DXGI, 0x2A | 0x37) => Kind::PresentStart {
                swap_chain: fields.u64("pIDXGISwapChain")?,
                flags: fields.u32("Flags")?,
                sync_interval: fields.u32("SyncInterval")? as i32,
            },
            (DXGI, 0x2B | 0x38) => Kind::PresentStop { result: fields.u32("Result")? },
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
