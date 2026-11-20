// Tests that the decoder takes a trace file's metadata events, which give
// the layouts of events that this computer's manifests may not have, only
// when every property that one says it has is in it: a trace file can say
// anything, and read_file, which reads one, is safe to call.

use present_timing::Decoder;
use present_timing::events::{DXGI, Event, Kind, Runtime, decode};
use windows_api_raw::*;
use windows_core::GUID;

// The provider of a trace file's metadata events, and their opcode.
const METADATA: GUID = GUID::from_u128(0xbbccf6c1_6cd1_48c4_80ff_839482e37671);
const EVENT_INFO: u8 = 0x20;

// DXGI's Present_Stop, of a version that no manifest has.
const PRESENT_STOP: u16 = 0x2B;
const VERSION: u8 = 200;

// A metadata event's payload, a TRACE_EVENT_INFO, for DXGI's Present_Stop,
// whose one property is Result, a UINT32, and which says it has `count`
// properties. Its name is in the TRACE_EVENT_INFO's own bytes, where the
// names' offsets are, which the decoder doesn't read, so that the payload is
// no longer than a TRACE_EVENT_INFO with one property.
fn metadata(count: u32) -> Vec<u64> {
    metadata_with(count, |_| {})
}

// The same, with its property as `change` leaves it.
fn metadata_with(count: u32, change: impl FnOnce(&mut EVENT_PROPERTY_INFO)) -> Vec<u64> {
    let mut info = TRACE_EVENT_INFO {
        ProviderGuid: DXGI,
        PropertyCount: count,
        TopLevelPropertyCount: 1,
        ..Default::default()
    };
    info.EventDescriptor.Id = PRESENT_STOP;
    info.EventDescriptor.Version = VERSION;
    let name = std::mem::offset_of!(TRACE_EVENT_INFO, ProviderNameOffset);
    let property = &mut info.EventPropertyInfoArray[0];
    property.NameOffset = name as u32;
    property.Anonymous.nonStructType.InType = TDH_INTYPE_UINT32 as u16;
    property.Anonymous2.count = 1;
    property.Anonymous3.length = 4;
    change(property);
    // In u64s, for TRACE_EVENT_INFO's alignment.
    let mut words = vec![0u64; size_of::<TRACE_EVENT_INFO>().div_ceil(8)];
    unsafe {
        let bytes = words.as_mut_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(&info as *const TRACE_EVENT_INFO as *const u8, bytes, size_of::<TRACE_EVENT_INFO>());
        for (index, unit) in "Result\0".encode_utf16().enumerate() {
            std::ptr::copy_nonoverlapping(unit.to_le_bytes().as_ptr(), bytes.add(name + index * 2), 2);
        }
    }
    words
}

fn record(provider: GUID, id: u16, opcode: u8, payload: &mut [u8]) -> EVENT_RECORD {
    let mut record = EVENT_RECORD::default();
    record.EventHeader.ProviderId = provider;
    record.EventHeader.EventDescriptor.Id = id;
    record.EventHeader.EventDescriptor.Version = VERSION;
    record.EventHeader.EventDescriptor.Opcode = opcode;
    record.EventHeader.Flags = EVENT_HEADER_FLAG_64_BIT_HEADER as u16;
    record.UserData = payload.as_mut_ptr() as *mut _;
    record.UserDataLength = payload.len() as u16;
    record
}

// The Present_Stop that a decoder makes of an event with Result 5, after a
// metadata event that says the event has `count` properties.
fn present_stop(count: u32) -> Option<Event> {
    present_stop_of(metadata(count))
}

fn present_stop_of(mut words: Vec<u64>) -> Option<Event> {
    let mut decoder = Decoder::default();
    let length = size_of::<TRACE_EVENT_INFO>();
    let payload = unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, length) };
    assert_eq!(decode(&mut decoder, &record(METADATA, 0, EVENT_INFO, payload)), None);
    let mut result = 5u32.to_le_bytes();
    decode(&mut decoder, &record(DXGI, PRESENT_STOP, 0, &mut result))
}

#[test]
fn a_trace_file_s_layout_is_taken_when_its_properties_are_in_it() {
    let event = present_stop(1).unwrap();
    assert_eq!(event.kind, Kind::PresentStop { runtime: Runtime::Dxgi, result: 5 });
}

// One that says it has a second property, past its end, gives its event no
// layout, rather than reading past it.
#[test]
fn a_trace_file_s_layout_whose_properties_go_past_its_end_is_not_taken() {
    assert_eq!(present_stop(2), None);
    assert_eq!(present_stop(u32::MAX), None);
}

// One whose property is a struct that holds itself, whose size would never
// be found, gives its event no layout, rather than looking for it forever.
#[test]
fn a_trace_file_s_layout_with_a_struct_inside_itself_is_not_taken() {
    let words = metadata_with(1, |property| {
        property.Flags = PropertyStruct;
        property.Anonymous.structType = EVENT_PROPERTY_INFO_0_1 {
            StructStartIndex: 0,
            NumOfStructMembers: 1,
            padding: 0,
        };
    });
    assert_eq!(present_stop_of(words), None);
}
