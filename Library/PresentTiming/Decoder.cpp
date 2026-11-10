#include "PresentTiming/Decoder.h"

#include "Base/Text.h"

#include <algorithm>
#include <cstring>

#pragma comment(lib, "tdh.lib")

namespace present_timing::decoder {

namespace {

// The opcode of a trace file's metadata events.
constexpr std::uint8_t EventInfo = 0x20;

// The size of one element of a property, or nothing if it's of a type that
// these events don't have.
std::optional<size_t> element_size(const Layout& layout, const Property& property, size_t pointer, std::span<const std::uint8_t> data, size_t offset,
                                   std::span<const std::uint64_t> values) {
    if ((property.flags & PropertyStruct) != 0) {
        size_t start = property.struct_start;
        size_t end = start + property.struct_members;
        if (end > layout.properties.size()) {
            return std::nullopt;
        }
        size_t size = 0;
        for (size_t index = start; index < end; index++) {
            const Property& member = layout.properties[index];
            std::optional<size_t> member_size = element_size(layout, member, pointer, data, offset + size, {});
            if (!member_size) {
                return std::nullopt;
            }
            size += *member_size * std::max<std::uint16_t>(member.count, 1);
        }
        return size;
    }
    size_t length = property.length;
    if ((property.flags & PropertyParamLength) != 0) {
        if (property.length >= values.size()) {
            return std::nullopt;
        }
        length = static_cast<size_t>(values[property.length]);
    }
    switch (property.in_type) {
    case TDH_INTYPE_INT8:
    case TDH_INTYPE_UINT8:
        return 1;
    case TDH_INTYPE_INT16:
    case TDH_INTYPE_UINT16:
        return 2;
    case TDH_INTYPE_INT32:
    case TDH_INTYPE_UINT32:
    case TDH_INTYPE_HEXINT32:
    case TDH_INTYPE_BOOLEAN:
    case TDH_INTYPE_FLOAT:
        return 4;
    case TDH_INTYPE_INT64:
    case TDH_INTYPE_UINT64:
    case TDH_INTYPE_HEXINT64:
    case TDH_INTYPE_DOUBLE:
    case TDH_INTYPE_FILETIME:
        return 8;
    case TDH_INTYPE_POINTER:
    case TDH_INTYPE_SIZET:
        return pointer;
    case TDH_INTYPE_GUID:
    case TDH_INTYPE_SYSTEMTIME:
        return 16;
    case TDH_INTYPE_BINARY:
        return length;
    case TDH_INTYPE_UNICODESTRING: {
        if (length > 0) {
            return length * 2;
        }
        if (offset > data.size()) {
            return std::nullopt;
        }
        std::span<const std::uint8_t> rest = data.subspan(offset);
        for (size_t unit = 0; unit < rest.size() / 2; unit++) {
            if (rest[unit * 2] == 0 && rest[unit * 2 + 1] == 0) {
                return (unit + 1) * 2;
            }
        }
        return rest.size();
    }
    case TDH_INTYPE_ANSISTRING: {
        if (length > 0) {
            return length;
        }
        if (offset > data.size()) {
            return std::nullopt;
        }
        std::span<const std::uint8_t> rest = data.subspan(offset);
        for (size_t byte = 0; byte < rest.size(); byte++) {
            if (rest[byte] == 0) {
                return byte + 1;
            }
        }
        return rest.size();
    }
    default:
        return std::nullopt;
    }
}

// A UTF-16 string that ends with a zero, at an offset in the bytes.
std::string wide_string(std::span<const std::uint8_t> bytes, size_t offset) {
    std::wstring units;
    if (offset <= bytes.size()) {
        std::span<const std::uint8_t> rest = bytes.subspan(offset);
        for (size_t pair = 0; pair < rest.size() / 2; pair++) {
            wchar_t unit = static_cast<wchar_t>(rest[pair * 2] | (rest[pair * 2 + 1] << 8));
            if (unit == 0) {
                break;
            }
            units.push_back(unit);
        }
    }
    return base::narrow(units);
}

// A layout from a TRACE_EVENT_INFO, in bytes aligned for it.
std::optional<Layout> parse(std::span<const std::uint8_t> bytes) {
    const TRACE_EVENT_INFO& info = *reinterpret_cast<const TRACE_EVENT_INFO*>(bytes.data());
    const EVENT_PROPERTY_INFO* array = info.EventPropertyInfoArray;
    std::vector<Property> properties;
    properties.reserve(info.PropertyCount);
    for (size_t index = 0; index < info.PropertyCount; index++) {
        const EVENT_PROPERTY_INFO& property = array[index];
        bool is_struct = (property.Flags & PropertyStruct) != 0;
        properties.push_back(Property{
            .name = wide_string(bytes, property.NameOffset),
            .flags = property.Flags,
            .in_type = is_struct ? std::uint16_t{0} : property.nonStructType.InType,
            .length = property.length,
            .count = property.count,
            .struct_start = is_struct ? property.structType.StructStartIndex : std::uint16_t{0},
            .struct_members = is_struct ? property.structType.NumOfStructMembers : std::uint16_t{0},
        });
    }
    size_t top_level = std::min<size_t>(info.TopLevelPropertyCount, properties.size());
    return Layout{
        .properties = std::move(properties),
        .top_level = top_level,
    };
}

// The event's layout from TDH, or nothing if TDH has none.
std::optional<Layout> layout(const EVENT_RECORD& record) {
    // TDH takes the record as one that it could change, and doesn't.
    EVENT_RECORD* event = const_cast<EVENT_RECORD*>(&record);
    ULONG size = 0;
    TdhGetEventInformation(event, 0, nullptr, nullptr, &size);
    if (size == 0) {
        return std::nullopt;
    }
    // In 64-bit words, for TRACE_EVENT_INFO's alignment.
    std::vector<std::uint64_t> buffer((static_cast<size_t>(size) + 7) / 8);
    TRACE_EVENT_INFO* info = reinterpret_cast<TRACE_EVENT_INFO*>(buffer.data());
    if (TdhGetEventInformation(event, 0, nullptr, info, &size) != ERROR_SUCCESS) {
        return std::nullopt;
    }
    return parse(std::span(reinterpret_cast<const std::uint8_t*>(buffer.data()), size));
}

} // namespace

size_t Decoder::KeyHash::operator()(const Key& key) const noexcept {
    // FNV-1a, over the provider's bytes, the ID and the version.
    size_t hash = 14695981039346656037ull;
    auto add = [&](std::uint8_t byte) {
        hash = (hash ^ byte) * 1099511628211ull;
    };
    const std::uint8_t* provider = reinterpret_cast<const std::uint8_t*>(&key.provider);
    for (size_t index = 0; index < sizeof(GUID); index++) {
        add(provider[index]);
    }
    add(static_cast<std::uint8_t>(key.id));
    add(static_cast<std::uint8_t>(key.id >> 8));
    add(key.version);
    return hash;
}

void Decoder::add_metadata(const EVENT_RECORD& record) {
    if (record.EventHeader.EventDescriptor.Opcode != EventInfo) {
        return;
    }
    std::span<const std::uint8_t> data = payload(record);
    if (data.size() < sizeof(TRACE_EVENT_INFO)) {
        return;
    }
    // In 64-bit words, for TRACE_EVENT_INFO's alignment.
    std::vector<std::uint64_t> buffer((data.size() + 7) / 8);
    std::memcpy(buffer.data(), data.data(), data.size());
    const TRACE_EVENT_INFO& info = *reinterpret_cast<const TRACE_EVENT_INFO*>(buffer.data());
    // TraceLogging's layouts aren't kept.
    if (info.DecodingSource == DecodingSourceTlg || info.EventDescriptor.Channel == 0xB) {
        return;
    }
    Key key{info.ProviderGuid, info.EventDescriptor.Id, info.EventDescriptor.Version};
    std::span<const std::uint8_t> bytes(reinterpret_cast<const std::uint8_t*>(buffer.data()), data.size());
    layouts.insert_or_assign(key, parse(bytes));
}

std::optional<Fields> Decoder::fields(const EVENT_RECORD& record) {
    const EVENT_HEADER& header = record.EventHeader;
    Key key{header.ProviderId, header.EventDescriptor.Id, header.EventDescriptor.Version};
    auto known = layouts.find(key);
    if (known == layouts.end()) {
        known = layouts.emplace(key, layout(record)).first;
    }
    if (!known->second) {
        return std::nullopt;
    }
    const Layout& layout = *known->second;
    std::span<const std::uint8_t> data = payload(record);
    size_t pointer = (header.Flags & EVENT_HEADER_FLAG_64_BIT_HEADER) != 0 ? 8 : 4;
    Fields fields;
    fields.data = data;
    fields.fields.reserve(layout.top_level);
    std::vector<std::uint64_t> values;
    values.reserve(layout.top_level);
    size_t offset = 0;
    for (size_t index = 0; index < layout.top_level; index++) {
        const Property& property = layout.properties[index];
        size_t count = std::max<std::uint16_t>(property.count, 1);
        if ((property.flags & PropertyParamCount) != 0) {
            if (property.count >= values.size()) {
                return std::nullopt;
            }
            count = static_cast<size_t>(values[property.count]);
        }
        std::optional<size_t> size = element_size(layout, property, pointer, data, offset, values);
        if (!size) {
            // A field that this can't size ends the fields that it finds.
            break;
        }
        values.push_back(count == 1 ? read(data, offset, *size).value_or(0) : 0);
        fields.fields.push_back({property.name, offset, *size, count});
        offset += *size * count;
    }
    return fields;
}

std::optional<std::uint64_t> Fields::u64(std::string_view name) const {
    auto field = std::ranges::find(fields, name, &Field::name);
    if (field == fields.end()) {
        return std::nullopt;
    }
    return read(data, field->offset, field->size);
}

std::optional<std::uint32_t> Fields::u32(std::string_view name) const {
    std::optional<std::uint64_t> value = u64(name);
    if (!value) {
        return std::nullopt;
    }
    return static_cast<std::uint32_t>(*value);
}

std::optional<std::vector<std::uint64_t>> Fields::array(std::string_view name) const {
    auto field = std::ranges::find(fields, name, &Field::name);
    if (field == fields.end()) {
        return std::nullopt;
    }
    std::vector<std::uint64_t> values;
    for (size_t index = 0; index < field->count; index++) {
        std::optional<std::uint64_t> value = read(data, field->offset + index * field->size, field->size);
        if (!value) {
            return std::nullopt;
        }
        values.push_back(*value);
    }
    return values;
}

std::span<const std::uint8_t> payload(const EVENT_RECORD& record) {
    if (record.UserData == nullptr || record.UserDataLength == 0) {
        return {};
    }
    return std::span(static_cast<const std::uint8_t*>(record.UserData), record.UserDataLength);
}

std::optional<std::uint64_t> read(std::span<const std::uint8_t> data, size_t offset, size_t size) {
    if (offset > data.size() || size > data.size() - offset) {
        return std::nullopt;
    }
    switch (size) {
    case 1:
    case 2:
    case 4:
    case 8: {
        std::uint64_t value = 0;
        for (size_t index = size; index > 0; index--) {
            value = (value << 8) | data[offset + index - 1];
        }
        return value;
    }
    default:
        return std::nullopt;
    }
}

} // namespace present_timing::decoder
