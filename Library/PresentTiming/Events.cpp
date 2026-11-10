#include "PresentTiming/Events.h"

// Direct3D 9's Present flags, and the DXGI flags that mean the same.
#include <d3d9.h>
#include <dxgi.h>

#include <algorithm>
#include <initializer_list>

namespace present_timing::events {

using decoder::Decoder;
using decoder::Metadata;
using decoder::payload;
using decoder::read;

namespace {

constexpr std::uint16_t DxgkrnlEvents[] = {0xAB, 0xA6, 0x1F5, 0xA8, 0x10A, 0xFC, 0x17E, 0x111, 0x74, 0x103, 0xB8, 0xAC, 0xD7, 0xB2, 0xF4, 0xB4, 0x11};
constexpr std::uint16_t Win32kEvents[] = {0xC9, 0x12D};
constexpr std::uint16_t DwmEvents[] = {0x40, 0x0F, 0xC4, 0x45, 0x46, 0x65};
constexpr std::uint16_t DxgiEvents[] = {0x2A, 0x2B, 0x37, 0x38};
constexpr std::uint16_t D3d9Events[] = {0x1, 0x2};
constexpr std::uint16_t NvidiaEvents[] = {0x1};

std::uint32_t dxgi_flags(std::uint32_t flags) {
    std::uint32_t dxgi = 0;
    for (auto [d3d9, flag] : {
             std::pair<std::uint32_t, std::uint32_t>{D3DPRESENT_DONOTFLIP, DXGI_PRESENT_DO_NOT_SEQUENCE},
             std::pair<std::uint32_t, std::uint32_t>{D3DPRESENT_DONOTWAIT, DXGI_PRESENT_DO_NOT_WAIT},
             std::pair<std::uint32_t, std::uint32_t>{D3DPRESENT_FLIPRESTART, DXGI_PRESENT_RESTART},
         }) {
        if ((flags & d3d9) != 0) {
            dxgi |= flag;
        }
    }
    return dxgi;
}

} // namespace

const std::array<Provider, 6> Providers = {
    Provider{
        .guid = Dxgkrnl,
        .level = 4,
        .any = 0x0800'0001,
        .all = 0x1,
        .events = DxgkrnlEvents,
    },
    Provider{
        .guid = Win32k,
        .level = 4,
        .any = 0x8000'0004'0000'1000,
        .all = 0x8000'0000'0000'1000,
        .events = Win32kEvents,
    },
    Provider{
        .guid = Dwm,
        .level = 5,
        .any = 0x8000'0000'0000'0081,
        .all = 0,
        .events = DwmEvents,
    },
    Provider{
        .guid = Dxgi,
        .level = 0,
        .any = 0x8000'0000'0000'0002,
        .all = 0x8000'0000'0000'0002,
        .events = DxgiEvents,
    },
    Provider{
        .guid = D3d9,
        .level = 0,
        .any = 0x8000'0000'0000'0002,
        .all = 0x8000'0000'0000'0002,
        .events = D3d9Events,
    },
    Provider{
        .guid = Nvidia,
        .level = 4,
        .any = 0x1000'0000'0000'0000,
        .all = 0x1000'0000'0000'0000,
        .events = NvidiaEvents,
    },
};

std::optional<Event> decode(Decoder& decoder, const EVENT_RECORD& record) {
    const EVENT_HEADER& header = record.EventHeader;
    std::uint16_t id = header.EventDescriptor.Id;
    std::uint8_t version = header.EventDescriptor.Version;
    const GUID& provider = header.ProviderId;
    if (provider == Metadata) {
        decoder.add_metadata(record);
        return std::nullopt;
    }
    auto event = [&](Kind kind) {
        return Event{
            .time = header.TimeStamp.QuadPart,
            .process = header.ProcessId,
            .thread = header.ThreadId,
            .kind = std::move(kind),
        };
    };
    if (provider == Nvidia) {
        // NVIDIA's provider registers no manifest, so its one event is read
        // by its fixed layout.
        std::span<const std::uint8_t> data = payload(record);
        if (id != 1 || data.size() < 44) {
            return std::nullopt;
        }
        std::optional<std::uint64_t> token = read(data, 8, 4);
        std::optional<std::uint64_t> time = read(data, 16, 8);
        std::optional<std::uint64_t> source = read(data, 28, 4);
        std::optional<std::uint64_t> alloc = read(data, 36, 8);
        if (!token || !time || !source || !alloc) {
            return std::nullopt;
        }
        return event(Kind::NvidiaFlipRequest{
            .alloc = *alloc,
            .source = static_cast<std::uint32_t>(*source),
            .time = *time,
            .token = static_cast<std::uint32_t>(*token),
        });
    }
    // Finding an event's fields is most of the cost of decoding, and the
    // providers send events that aren't read, so those go first.
    auto one_of = [id](std::initializer_list<std::uint16_t> ids) {
        return std::ranges::find(ids, id) != ids.end();
    };
    bool wanted = (provider == Dxgi && one_of({0x2A, 0x37, 0x2B, 0x38})) || (provider == D3d9 && one_of({0x1, 0x2})) || (provider == Win32k && one_of({0xC9, 0x12D})) ||
                  (provider == Dxgkrnl && one_of({0xAB, 0xD7, 0xAC, 0xB2, 0xB4, 0xB8, 0xA6, 0x1F5, 0xA8, 0xFC, 0x10A, 0x74, 0x103, 0x11, 0x111, 0x17E})) ||
                  (provider == Dwm && one_of({0x0F, 0x40, 0xC4, 0x45, 0x46, 0x65}));
    if (!wanted) {
        return std::nullopt;
    }
    std::optional<decoder::Fields> fields = decoder.fields(record);
    if (!fields) {
        return std::nullopt;
    }
    if (provider == Dxgi) {
        switch (id) {
        case 0x2A:
        case 0x37: {
            std::optional<std::uint64_t> swap_chain = fields->u64("pIDXGISwapChain");
            std::optional<std::uint32_t> flags = fields->u32("Flags");
            std::optional<std::uint32_t> sync_interval = fields->u32("SyncInterval");
            if (!swap_chain || !flags || !sync_interval) {
                return std::nullopt;
            }
            return event(Kind::PresentStart{
                .runtime = Runtime::Dxgi,
                .swap_chain = *swap_chain,
                .flags = *flags,
                .sync_interval = static_cast<std::int32_t>(*sync_interval),
            });
        }
        case 0x2B:
        case 0x38: {
            std::optional<std::uint32_t> result = fields->u32("Result");
            if (!result) {
                return std::nullopt;
            }
            return event(Kind::PresentStop{.runtime = Runtime::Dxgi, .result = *result});
        }
        }
    } else if (provider == D3d9) {
        switch (id) {
        case 0x1: {
            std::optional<std::uint32_t> flags = fields->u32("Flags");
            std::optional<std::uint64_t> swap_chain = fields->u64("pSwapchain");
            if (!flags || !swap_chain) {
                return std::nullopt;
            }
            return event(Kind::PresentStart{
                .runtime = Runtime::D3d9,
                .swap_chain = *swap_chain,
                .flags = dxgi_flags(*flags),
                .sync_interval = (*flags & D3DPRESENT_FORCEIMMEDIATE) != 0 ? 0 : -1,
            });
        }
        case 0x2: {
            std::optional<std::uint32_t> result = fields->u32("Result");
            if (!result) {
                return std::nullopt;
            }
            return event(Kind::PresentStop{.runtime = Runtime::D3d9, .result = *result});
        }
        }
    } else if (provider == Win32k) {
        switch (id) {
        case 0xC9: {
            std::optional<std::uint64_t> surface = fields->u64("CompositionSurfaceLuid");
            std::optional<std::uint64_t> present_count = fields->u64("PresentCount");
            std::optional<std::uint64_t> bind_id = fields->u64("BindId");
            if (!surface || !present_count || !bind_id) {
                return std::nullopt;
            }
            return event(Kind::TokenCompositionSurfaceObject{.key = {*surface, *present_count, *bind_id}});
        }
        case 0x12D: {
            std::optional<std::uint32_t> state = fields->u32("NewState");
            std::optional<std::uint64_t> surface = fields->u64("CompositionSurfaceLuid");
            std::optional<std::uint64_t> present_count = fields->u64("PresentCount");
            std::optional<std::uint64_t> bind_id = fields->u64("BindId");
            if (!state || !surface || !present_count || !bind_id) {
                return std::nullopt;
            }
            return event(Kind::TokenStateChanged{
                .key = {*surface, *present_count, *bind_id},
                .state = *state,
                .independent_flip = *state == InFrame && fields->u32("IndependentFlip").value_or(0) != 0,
            });
        }
        }
    } else if (provider == Dxgkrnl) {
        switch (id) {
        case 0xAB:
        case 0xD7: {
            std::optional<std::uint64_t> token = fields->u64("Token");
            std::optional<std::uint32_t> model = fields->u32("Model");
            if (!token || !model) {
                return std::nullopt;
            }
            return event(Kind::PresentHistoryStart{
                .token = *token,
                .model = *model,
                .token_data = fields->u64("TokenData").value_or(0),
            });
        }
        case 0xAC: {
            std::optional<std::uint64_t> token = fields->u64("Token");
            if (!token) {
                return std::nullopt;
            }
            return event(Kind::PresentHistoryInfo{.token = *token});
        }
        case 0xB2: {
            std::optional<std::uint32_t> packet_type = fields->u32("PacketType");
            std::optional<std::uint32_t> submit_sequence = fields->u32("SubmitSequence");
            std::optional<std::uint64_t> context = fields->u64("hContext");
            std::optional<std::uint32_t> present = fields->u32("bPresent");
            if (!packet_type || !submit_sequence || !context || !present) {
                return std::nullopt;
            }
            return event(Kind::QueuePacketStart{
                .packet_type = *packet_type,
                .submit_sequence = *submit_sequence,
                .context = *context,
                .present = *present != 0,
            });
        }
        case 0xB4: {
            std::optional<std::uint64_t> context = fields->u64("hContext");
            std::optional<std::uint32_t> submit_sequence = fields->u32("SubmitSequence");
            if (!context || !submit_sequence) {
                return std::nullopt;
            }
            return event(Kind::QueuePacketStop{.context = *context, .submit_sequence = *submit_sequence});
        }
        case 0xB8: {
            std::optional<std::uint64_t> window = fields->u64("hWindow");
            if (!window) {
                return std::nullopt;
            }
            return event(Kind::PresentInfo{.window = *window});
        }
        case 0xA6: {
            std::optional<std::uint64_t> window = fields->u64("hwnd");
            std::optional<std::uint32_t> redirected = fields->u32("bRedirectedPresent");
            if (!window || !redirected) {
                return std::nullopt;
            }
            return event(Kind::Blit{.window = *window, .redirected = *redirected != 0});
        }
        case 0x1F5:
            return event(Kind::BlitCancel{});
        case 0xA8: {
            std::optional<std::uint32_t> interval = fields->u32("FlipInterval");
            std::optional<std::uint32_t> mmio = fields->u32("MMIOFlip");
            if (!interval || !mmio) {
                return std::nullopt;
            }
            return event(Kind::Flip{.interval = *interval, .mmio = *mmio != 0});
        }
        case 0xFC: {
            std::optional<std::uint32_t> source = fields->u32("VidPnSourceId");
            std::optional<std::uint32_t> layer = fields->u32("LayerIndex");
            if (!source || !layer) {
                return std::nullopt;
            }
            return event(Kind::FlipMultiPlaneOverlay{.source = *source, .layer = *layer});
        }
        case 0x10A: {
            std::optional<std::uint32_t> submit_sequence = fields->u32("SubmitSequence");
            std::optional<std::uint32_t> interval = fields->u32("FlipInterval");
            if (!submit_sequence || !interval) {
                return std::nullopt;
            }
            return event(Kind::IndependentFlip{.submit_sequence = *submit_sequence, .interval = *interval});
        }
        case 0x74: {
            std::optional<std::uint32_t> submit_sequence = fields->u32("FlipSubmitSequence");
            std::optional<std::uint32_t> flags = fields->u32("Flags");
            if (!submit_sequence || !flags) {
                return std::nullopt;
            }
            return event(Kind::MmioFlip{.submit_sequence = *submit_sequence, .flags = *flags});
        }
        case 0x103: {
            std::optional<std::uint64_t> submit_sequence = fields->u64("FlipSubmitSequence");
            if (!submit_sequence) {
                return std::nullopt;
            }
            return event(Kind::MmioFlipMultiPlaneOverlay{
                .submit_sequence = static_cast<std::uint32_t>(*submit_sequence >> 32),
                .status = version >= 2 ? fields->u32("FlipEntryStatusAfterFlip") : std::nullopt,
            });
        }
        case 0x11: {
            std::optional<std::uint64_t> fence = fields->u64("FlipFenceId");
            if (!fence) {
                return std::nullopt;
            }
            return event(Kind::VSyncDpc{.fence = *fence});
        }
        case 0x111:
        case 0x17E: {
            std::string_view addresses = id == 0x111 && version >= 1 ? "PresentIdOrPhysicalAddress" : "ScannedPhysicalAddress";
            std::optional<std::vector<std::uint64_t>> planes = fields->array(addresses);
            if (!planes) {
                return std::nullopt;
            }
            std::optional<std::vector<std::uint64_t>> flip_sequences = fields->array("FlipSubmitSequence");
            if (!flip_sequences) {
                return std::nullopt;
            }
            return event(Kind::SyncDpcMultiPlane{
                .planes = std::move(*planes),
                .flip_sequences = std::move(*flip_sequences),
            });
        }
        }
    } else if (provider == Dwm) {
        switch (id) {
        case 0x0F:
            return event(Kind::DwmSchedulePresent{});
        case 0x40:
            return event(Kind::DwmGetPresentHistory{});
        case 0xC4: {
            std::optional<std::uint64_t> surface = fields->u64("luidSurface");
            std::optional<std::uint64_t> present_count = fields->u64("PresentCount");
            std::optional<std::uint64_t> bind_id = fields->u64("bindId");
            if (!surface || !present_count || !bind_id) {
                return std::nullopt;
            }
            return event(Kind::DwmScheduleSurfaceUpdate{.key = {*surface, *present_count, *bind_id}});
        }
        // The flip chain and the serial number are 4 bytes each, or 8 on
        // Windows 8.1, and make the token data's high and low halves.
        case 0x45:
        case 0x46:
        case 0x65: {
            std::optional<std::uint64_t> flip_chain = fields->u64("ulFlipChain");
            std::optional<std::uint64_t> serial_number = fields->u64("ulSerialNumber");
            std::optional<std::uint64_t> window = fields->u64("hwnd");
            if (!flip_chain || !serial_number || !window) {
                return std::nullopt;
            }
            return event(Kind::DwmFlipChain{
                .token_data = (*flip_chain << 32) | (*serial_number & 0xFFFF'FFFF),
                .window = *window,
            });
        }
        }
    }
    return std::nullopt;
}

} // namespace present_timing::events
