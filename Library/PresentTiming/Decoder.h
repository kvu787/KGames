// Finds an ETW event's fields by name in its payload, with the layout that
// Windows' trace data helper (TDH) gives for the event's provider, ID and
// version, from the manifest that the provider registered. Each layout is
// asked for once. A trace file can have the layouts of its events too, as
// metadata events, for versions of events that this computer's manifests
// don't have, which then come first. Fields follow each other without
// padding, so a field's offset is the sizes of the ones before it, some of
// which, like arrays, are as long as an earlier field says.

#pragma once

#include "Base/Windows.h"

#include <tdh.h>

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <unordered_map>
#include <vector>

namespace present_timing::decoder {

// A field of an event's layout. Only the decoder uses it, and Layout: they're
// here for the Decoder's members.
struct Property {
    std::string name;
    PROPERTY_FLAGS flags;
    std::uint16_t in_type;
    // The length or count itself, or the index of the property that has it,
    // as the flags say.
    std::uint16_t length;
    std::uint16_t count;
    // A struct's members, which follow the top-level properties.
    std::uint16_t struct_start;
    std::uint16_t struct_members;
};

struct Layout {
    std::vector<Property> properties;
    size_t top_level;
};

// The provider of a trace file's metadata events.
inline constexpr GUID Metadata{0xbbccf6c1, 0x6cd1, 0x48c4, {0x80, 0xff, 0x83, 0x94, 0x82, 0xe3, 0x76, 0x71}};

// One event's top-level fields: where each is in the payload, how big one of
// its elements is, and how many there are.
class Fields {
public:
    Fields(Fields&&) = default;
    Fields& operator=(Fields&&) = default;
    Fields(const Fields&) = delete;
    Fields& operator=(const Fields&) = delete;

    // A whole number field, of up to 8 bytes, zero-extended.
    std::optional<std::uint64_t> u64(std::string_view name) const;

    std::optional<std::uint32_t> u32(std::string_view name) const;

    // An array of whole numbers.
    std::optional<std::vector<std::uint64_t>> array(std::string_view name) const;

private:
    // The decoder makes them.
    friend class Decoder;
    Fields() = default;

    struct Field {
        std::string_view name;
        size_t offset;
        size_t size;
        size_t count;
    };

    std::span<const std::uint8_t> data;
    std::vector<Field> fields;
};

class Decoder {
public:
    Decoder() = default;
    Decoder(Decoder&&) = default;
    Decoder& operator=(Decoder&&) = default;
    Decoder(const Decoder&) = delete;
    Decoder& operator=(const Decoder&) = delete;

    // Keeps the layout in a trace file's metadata event, a TRACE_EVENT_INFO,
    // for the events it's of.
    void add_metadata(const EVENT_RECORD& record);

    // The event's fields, or nothing if TDH has no layout for it, like for a
    // provider without a registered manifest. They're good for as long as
    // the decoder and the record are.
    std::optional<Fields> fields(const EVENT_RECORD& record);

private:
    // An event's provider, ID and version.
    struct Key {
        GUID provider;
        std::uint16_t id;
        std::uint8_t version;

        bool operator==(const Key& other) const {
            return provider == other.provider && id == other.id && version == other.version;
        }
    };

    struct KeyHash {
        size_t operator()(const Key& key) const noexcept;
    };

    std::unordered_map<Key, std::optional<Layout>, KeyHash> layouts;
};

// The event's payload.
std::span<const std::uint8_t> payload(const EVENT_RECORD& record);

// A little-endian whole number of 1, 2, 4 or 8 bytes.
std::optional<std::uint64_t> read(std::span<const std::uint8_t> data, size_t offset, size_t size);

} // namespace present_timing::decoder
