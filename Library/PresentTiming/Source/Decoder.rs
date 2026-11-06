// Finds an ETW event's fields by name in its payload, with the layout that
// Windows' trace data helper (TDH) gives for the event's provider, ID and
// version, from the manifest that the provider registered. Each layout is
// asked for once. A trace file can have the layouts of its events too, as
// metadata events, for versions of events that this computer's manifests
// don't have, which then come first. Fields follow each other without
// padding, so a field's offset is the sizes of the ones before it, some of
// which, like arrays, are as long as an earlier field says.

use std::collections::HashMap;

use windows_api::*;
use windows_core::GUID;

struct Property {
    name: String,
    flags: PROPERTY_FLAGS,
    in_type: u16,
    // The length or count itself, or the index of the property that has it,
    // as the flags say.
    length: u16,
    count: u16,
    // A struct's members, which follow the top-level properties.
    struct_start: u16,
    struct_members: u16,
}

struct Layout {
    properties: Vec<Property>,
    top_level: usize,
}

#[derive(Default)]
pub struct Decoder {
    layouts: HashMap<(GUID, u16, u8), Option<Layout>>,
}

// The provider of a trace file's metadata events, and their opcode.
pub const METADATA: GUID = GUID::from_u128(0xbbccf6c1_6cd1_48c4_80ff_839482e37671);
const EVENT_INFO: u8 = 0x20;
// TDH's decoding source for TraceLogging, whose layouts aren't kept.
const DECODING_SOURCE_TRACE_LOGGING: i32 = 3;

// One event's top-level fields: where each is in the payload, how big one of
// its elements is, and how many there are.
pub struct Fields<'a> {
    data: &'a [u8],
    fields: Vec<(&'a str, usize, usize, usize)>,
}

impl Decoder {
    // Keeps the layout in a trace file's metadata event, a TRACE_EVENT_INFO,
    // for the events it's of.
    pub fn add_metadata(&mut self, record: &EVENT_RECORD) {
        if record.EventHeader.EventDescriptor.Opcode != EVENT_INFO {
            return;
        }
        let data = payload(record);
        if data.len() < size_of::<TRACE_EVENT_INFO>() {
            return;
        }
        // In u64s, for TRACE_EVENT_INFO's alignment.
        let mut buffer = vec![0u64; data.len().div_ceil(8)];
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), buffer.as_mut_ptr() as *mut u8, data.len());
            let info = &*(buffer.as_ptr() as *const TRACE_EVENT_INFO);
            if info.DecodingSource == DECODING_SOURCE_TRACE_LOGGING || info.EventDescriptor.Channel == 0xB {
                return;
            }
            let key = (info.ProviderGuid, info.EventDescriptor.Id, info.EventDescriptor.Version);
            let bytes = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, data.len());
            self.layouts.insert(key, parse(bytes));
        }
    }

    // The event's fields, or None if TDH has no layout for it, like for a
    // provider without a registered manifest.
    pub fn fields<'a>(&'a mut self, record: &'a EVENT_RECORD) -> Option<Fields<'a>> {
        let header = &record.EventHeader;
        let key = (header.ProviderId, header.EventDescriptor.Id, header.EventDescriptor.Version);
        let layout = self.layouts.entry(key).or_insert_with(|| unsafe { layout(record) }).as_ref()?;
        let data = payload(record);
        let pointer = if header.Flags as i32 & EVENT_HEADER_FLAG_64_BIT_HEADER != 0 { 8 } else { 4 };
        let mut fields = Vec::with_capacity(layout.top_level);
        let mut values = Vec::with_capacity(layout.top_level);
        let mut offset = 0;
        for index in 0..layout.top_level {
            let property = &layout.properties[index];
            let count = if property.flags & PropertyParamCount != 0 {
                *values.get(property.count as usize)? as usize
            } else {
                property.count.max(1) as usize
            };
            let Some(size) = element_size(layout, property, pointer, data, offset, &values) else {
                // A field that this can't size ends the fields that it finds.
                break;
            };
            values.push(if count == 1 { read(data, offset, size).unwrap_or(0) } else { 0 });
            fields.push((property.name.as_str(), offset, size, count));
            offset += size * count;
        }
        Some(Fields { data, fields })
    }
}

impl Fields<'_> {
    // A whole number field, of up to 8 bytes, zero-extended.
    pub fn u64(&self, name: &str) -> Option<u64> {
        let (_, offset, size, _) = self.fields.iter().find(|(field, ..)| *field == name)?;
        read(self.data, *offset, *size)
    }

    pub fn u32(&self, name: &str) -> Option<u32> {
        self.u64(name).map(|value| value as u32)
    }

    // An array of whole numbers.
    pub fn array(&self, name: &str) -> Option<Vec<u64>> {
        let (_, offset, size, count) = self.fields.iter().find(|(field, ..)| *field == name)?;
        (0..*count).map(|index| read(self.data, offset + index * size, *size)).collect()
    }
}

// The event's payload.
pub fn payload(record: &EVENT_RECORD) -> &[u8] {
    if record.UserData.is_null() || record.UserDataLength == 0 {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(record.UserData as *const u8, record.UserDataLength as usize) }
}

// A little-endian whole number of 1, 2, 4 or 8 bytes.
pub fn read(data: &[u8], offset: usize, size: usize) -> Option<u64> {
    let bytes = data.get(offset..offset + size)?;
    match size {
        1 | 2 | 4 | 8 => Some(bytes.iter().rev().fold(0u64, |value, byte| (value << 8) | *byte as u64)),
        _ => None,
    }
}

// The size of one element of a property, or None if it's of a type that
// these events don't have.
fn element_size(layout: &Layout, property: &Property, pointer: usize, data: &[u8], offset: usize, values: &[u64]) -> Option<usize> {
    if property.flags & PropertyStruct != 0 {
        let start = property.struct_start as usize;
        let mut size = 0;
        for member in layout.properties.get(start..start + property.struct_members as usize)? {
            size += element_size(layout, member, pointer, data, offset + size, &[])? * member.count.max(1) as usize;
        }
        return Some(size);
    }
    let length = if property.flags & PropertyParamLength != 0 {
        *values.get(property.length as usize)? as usize
    } else {
        property.length as usize
    };
    let size = match property.in_type as i32 {
        TDH_INTYPE_INT8 | TDH_INTYPE_UINT8 => 1,
        TDH_INTYPE_INT16 | TDH_INTYPE_UINT16 => 2,
        TDH_INTYPE_INT32 | TDH_INTYPE_UINT32 | TDH_INTYPE_HEXINT32 | TDH_INTYPE_BOOLEAN | TDH_INTYPE_FLOAT => 4,
        TDH_INTYPE_INT64 | TDH_INTYPE_UINT64 | TDH_INTYPE_HEXINT64 | TDH_INTYPE_DOUBLE | TDH_INTYPE_FILETIME => 8,
        TDH_INTYPE_POINTER | TDH_INTYPE_SIZET => pointer,
        TDH_INTYPE_GUID | TDH_INTYPE_SYSTEMTIME => 16,
        TDH_INTYPE_BINARY => length,
        TDH_INTYPE_UNICODESTRING if length > 0 => length * 2,
        TDH_INTYPE_UNICODESTRING => {
            let rest = data.get(offset..)?;
            rest.as_chunks::<2>().0.iter().position(|unit| *unit == [0, 0]).map_or(rest.len(), |units| (units + 1) * 2)
        }
        TDH_INTYPE_ANSISTRING if length > 0 => length,
        TDH_INTYPE_ANSISTRING => {
            let rest = data.get(offset..)?;
            rest.iter().position(|byte| *byte == 0).map_or(rest.len(), |bytes| bytes + 1)
        }
        _ => return None,
    };
    Some(size)
}

// The event's layout from TDH, or None if TDH has none.
unsafe fn layout(record: &EVENT_RECORD) -> Option<Layout> {
    unsafe {
        let mut size = 0u32;
        let _ = TdhGetEventInformation(record, None, None, &mut size);
        if size == 0 {
            return None;
        }
        // In u64s, for TRACE_EVENT_INFO's alignment.
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let info = buffer.as_mut_ptr() as *mut TRACE_EVENT_INFO;
        if TdhGetEventInformation(record, None, Some(info), &mut size).0 != 0 {
            return None;
        }
        parse(std::slice::from_raw_parts(buffer.as_ptr() as *const u8, size as usize))
    }
}

// A layout from a TRACE_EVENT_INFO, in bytes aligned for it.
unsafe fn parse(bytes: &[u8]) -> Option<Layout> {
    unsafe {
        let info = &*(bytes.as_ptr() as *const TRACE_EVENT_INFO);
        let array = std::ptr::addr_of!(info.EventPropertyInfoArray) as *const EVENT_PROPERTY_INFO;
        let mut properties = Vec::with_capacity(info.PropertyCount as usize);
        for index in 0..info.PropertyCount as usize {
            let property = &*array.add(index);
            let is_struct = property.Flags & PropertyStruct != 0;
            properties.push(Property {
                name: wide_string(bytes, property.NameOffset as usize),
                flags: property.Flags,
                in_type: if is_struct { 0 } else { property.Anonymous.nonStructType.InType },
                length: property.Anonymous3.length,
                count: property.Anonymous2.count,
                struct_start: if is_struct { property.Anonymous.structType.StructStartIndex } else { 0 },
                struct_members: if is_struct { property.Anonymous.structType.NumOfStructMembers } else { 0 },
            });
        }
        Some(Layout {
            top_level: (info.TopLevelPropertyCount as usize).min(properties.len()),
            properties,
        })
    }
}

// A UTF-16 string that ends with a zero, at an offset in the bytes.
fn wide_string(bytes: &[u8], offset: usize) -> String {
    let units: Vec<u16> = bytes
        .get(offset..)
        .unwrap_or_default()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .take_while(|unit| *unit != 0)
        .collect();
    String::from_utf16_lossy(&units)
}
