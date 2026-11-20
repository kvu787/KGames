// Writes the bindings that windows-bindgen generates for the part of
// GameInput's v4 header that KGames uses: keyboards, mice and gamepads. main.rs writes them to GameInputRaw's src/lib.rs, and
// Test/game-input writes them somewhere else, to check that the committed file
// is still exactly what this writes.
//
// GameInput isn't in the Windows metadata that windows-bindgen reads, so this
// describes GameInput.h as metadata, the way Windows' metadata describes
// Windows' headers, and gives windows-bindgen that too:
//
// - An enum is an enum of i32, which MSVC gives every unscoped enum, so its
//   0xFFFFFFFF is -1.
// - A struct is a struct, and the union that GameInputForceFeedbackParams
//   ends with is a struct nested in it.
// - GameInputCallbackToken, a typedef of a uint64_t, is a native typedef.
// - A callback is a delegate, with CALLBACK's calling convention.
// - An interface derives from IUnknown, with its GUID, and is agile, which
//   makes windows-bindgen write Send and Sync for it, as Microsoft's
//   documentation says that GameInput's objects are.
// - A parameter's SAL annotations give its direction, whether it's optional,
//   the parameter with its buffer's length, and whether it's a COM out
//   pointer. A buffer of void is a buffer of bytes.
// - The constants and GameInputInitialize are in the namespace's Apis class.
//   GAMEINPUT_API_VERSION, an integer #define, is an int.
//
// The scope, below, names what's bound, and windows-bindgen's filters take
// the rest out. A method that's out of scope keeps its slot in its
// interface's vtable, as a usize, so the slots after it stay where GameInput
// has them. An interface with such a slot has no trait to implement it with.
//
// windows-bindgen declares a function with link!, whose raw-dylib takes it
// from a DLL, but GameInputInitialize is in GameInput.lib, a static library
// that GameInputRaw's build script links. So it's declared with extern, as
// windows-bindgen declares functions with --sys --extern.

mod header;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use header::{Callback, Enum, Field, Header, Interface, Param, Struct, Typedef};
use windows_metadata::writer::{AttributeType, File, HasAttribute, HasConstant, MemberRefParent, TypeDefOrRef};
use windows_metadata::{
    FieldAttributes, MethodAttributes, MethodCallAttributes, MethodImplAttributes, PInvokeAttributes, ParamAttributes,
    Signature, Type, TypeAttributes, TypeName, Value,
};

// The namespace that the metadata puts GameInput in, under Windows' own.
const NAMESPACE: &str = "Windows.Win32.GameInput";

// Where Windows' metadata has the types from Windows' headers that GameInput.h
// uses, and those of them that it has as types of their own.
// windows-bindgen writes the others, HRESULT, GUID and PCWSTR, as
// windows-core's.
const WIN32: &str = "Windows.Win32";
const WIN32_TYPES: [&str; 2] = ["APP_LOCAL_DEVICE_ID", "HANDLE"];

// The scope: what the bindings have of GameInput.h, which is what reads
// keyboards, mice and gamepads, finds and identifies their devices, and
// rumbles a gamepad. The rest of the header is out: the other kinds of device
// (arcade sticks, flight sticks, racing wheels, sensors, touch surfaces),
// controllers that GameInput doesn't map to a gamepad, with the mapper, force
// feedback, haptics, lighting, raw device reports, aggregate devices and the
// dispatcher.
//
// The interfaces, each with its methods that are bound.
pub const INTERFACES: [(&str, &[&str]); 3] = [
    (
        "IGameInput",
        &[
            "GetCurrentTimestamp",
            "GetCurrentReading",
            "GetNextReading",
            "GetPreviousReading",
            "RegisterReadingCallback",
            "RegisterDeviceCallback",
            "UnregisterCallback",
            "FindDeviceFromId",
            "FindDeviceFromPlatformString",
            "EnableBackgroundInput",
        ],
    ),
    (
        "IGameInputDevice",
        &[
            "GetVendorId",
            "GetProductId",
            "GetRevisionNumber",
            "GetUsage",
            "GetHardwareVersion",
            "GetFirmwareVersion",
            "GetDeviceId",
            "GetDeviceRootId",
            "GetDeviceFamily",
            "GetSupportedInput",
            "GetSupportedRumbleMotors",
            "GetSupportedSystemButtons",
            "GetContainerId",
            "GetDisplayName",
            "GetPnpPath",
            "GetXInputDeviceIndex",
            "GetKeyboardInfo",
            "GetMouseInfo",
            "GetGamepadInfo",
            "IsConnected",
            "SetRumbleState",
        ],
    ),
    (
        "IGameInputReading",
        &[
            "GetInputKind",
            "GetTimestamp",
            "GetDevice",
            "GetKeyCount",
            "GetKeyState",
            "GetMouseState",
            "GetGamepadState",
            "GetSystemButtonState",
        ],
    ),
];

// The enums, each with all of its values, since GameInput gives any of them,
// the structs, the typedef and the callbacks.
pub const TYPES: [&str; 22] = [
    "GameInputCallbackToken",
    "GameInputDeviceCallback",
    "GameInputDeviceEvent",
    "GameInputDeviceFamily",
    "GameInputGamepadButtons",
    "GameInputGamepadInfo",
    "GameInputGamepadState",
    "GameInputKeyState",
    "GameInputKeyboardInfo",
    "GameInputKeyboardKind",
    "GameInputKind",
    "GameInputLabel",
    "GameInputMouseButtons",
    "GameInputMouseInfo",
    "GameInputMousePositions",
    "GameInputMouseState",
    "GameInputReadingCallback",
    "GameInputRumbleMotors",
    "GameInputRumbleParams",
    "GameInputSystemButtons",
    "GameInputUsage",
    "GameInputVersion",
];

// The constants: the version, the sets of a gamepad's buttons, and the errors
// of what's bound.
pub const CONSTANTS: [&str; 20] = [
    "GAMEINPUT_API_VERSION",
    "GameInputGamepadModuleSystemDuo",
    "GameInputGamepadModuleDpad",
    "GameInputGamepadModuleShoulders",
    "GameInputGamepadModuleTriggers",
    "GameInputGamepadModuleThumbsticks",
    "GameInputGamepadModulePaddles2",
    "GameInputGamepadModulePaddles4",
    "GameInputGamepadLayoutBasic",
    "GameInputGamepadLayoutButtons",
    "GameInputGamepadLayoutStandard",
    "GameInputGamepadLayoutElite",
    "GAMEINPUT_FACILITY",
    "GAMEINPUT_E_DEVICE_DISCONNECTED",
    "GAMEINPUT_E_DEVICE_NOT_FOUND",
    "GAMEINPUT_E_READING_NOT_FOUND",
    "GAMEINPUT_E_REFERENCE_READING_TOO_OLD",
    "GAMEINPUT_E_OBJECT_NO_LONGER_EXISTS",
    "GAMEINPUT_E_CALLBACK_NOT_FOUND",
    "GAMEINPUT_E_INPUT_KIND_NOT_PRESENT",
];

// The functions.
pub const FUNCTIONS: [&str; 1] = ["GameInputInitialize"];

// The type from Windows' headers.
pub const WIN32_TYPE: &str = "APP_LOCAL_DEVICE_ID";
const METADATA: &str = "Windows.Win32.Foundation.Metadata";

// What starts the bindings.
const PREAMBLE: &str = "\
// The bindings for the part of GameInput's v4 API that reads keyboards, mice
// and gamepads, from native/include/v4/GameInput.h in the Microsoft.GameInput
// NuGet package that NuGetPackages.csproj declares, and nothing else: what
// windows-bindgen generates for bindings-generator's scope of the header,
// which bindings-generator describes as Windows metadata for it. A usize in a
// vtable is the slot of a method that's out of scope. Don't edit it by hand:
// regenerate it with `cargo run -p game-input-bindings-generator`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

";

// GameInput.h, in the Microsoft.GameInput package that GameInputRaw's
// NuGetPackages.csproj declares, where GameInputRaw's build script had dotnet
// restore it.
pub fn header() -> PathBuf {
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../NuGetPackages.csproj");
    let output = Command::new("dotnet")
        .arg("msbuild")
        .arg(&project)
        .args(["-getProperty:PkgMicrosoft_GameInput", "--nologo"])
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .output()
        .expect("couldn't run dotnet");
    let package = String::from_utf8(output.stdout).unwrap();
    let header = Path::new(package.trim()).join(r"native\include\v4\GameInput.h");
    assert!(
        header.is_file(),
        "{} isn't there: build game-input-raw, whose build script restores the package",
        header.display()
    );
    header
}

// Writes the bindings for the header to the output.
pub fn generate(header: &Path, output: &Path) {
    let text = std::fs::read_to_string(header).unwrap_or_else(|error| panic!("{}: {error}", header.display()));
    let metadata = Metadata::write(&Header::parse(&text));
    windows_bindgen::builder()
        .input_default()
        .input_bytes(&metadata)
        .filters(filters())
        .output(output)
        .flat()
        .write();
    let bindings = std::fs::read_to_string(output).unwrap();
    std::fs::write(output, rustfmt(&format!("{PREAMBLE}{}", extern_functions(&bindings)))).unwrap();
}

// windows-bindgen's filters for the scope: each name alone, and an interface
// with its methods, like Windows.Win32.GameInput.IGameInput::{A, B}.
fn filters() -> Vec<String> {
    let names = TYPES.iter().chain(&CONSTANTS).chain(&FUNCTIONS).map(|name| format!("{NAMESPACE}.{name}"));
    let interfaces = INTERFACES.iter().map(|(name, methods)| format!("{NAMESPACE}.{name}::{{{}}}", methods.join(", ")));
    names.chain(interfaces).chain([WIN32_TYPE.to_string()]).collect()
}

// The bindings, with each function that link! declares declared with extern.
fn extern_functions(bindings: &str) -> String {
    let lines = bindings.lines().map(|line| match line.trim_start().strip_prefix("windows_core::link!(") {
        // windows_core::link!("library" "abi" fn Name(param : Type, ...) -> Type);
        Some(link) => {
            let (_, rest) = link.split_once("\" ").unwrap();
            let (abi, function) = rest.split_once(' ').unwrap();
            assert!(function.starts_with("fn "), "a function that links by another name: {line}");
            let function = function.strip_suffix(");").unwrap().replace(" : ", ": ");
            format!("unsafe extern {abi} {{ pub {function}; }}")
        }
        None => line.to_string(),
    });
    lines.collect::<Vec<_>>().join("\n")
}

// Code formatted as windows-bindgen formats it.
fn rustfmt(code: &str) -> String {
    let mut rustfmt = Command::new("rustfmt")
        .args(["--edition", "2024", "--config", "newline_style=Unix"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("couldn't run rustfmt");
    rustfmt.stdin.take().unwrap().write_all(code.as_bytes()).unwrap();
    let output = rustfmt.wait_with_output().unwrap();
    assert!(output.status.success(), "rustfmt couldn't format the bindings");
    String::from_utf8(output.stdout).unwrap()
}

// A parameter as metadata has it.
struct ParamRow {
    name: String,
    ty: Type,
    flags: ParamAttributes,
    // The index of the parameter with the buffer's length, in elements or in
    // bytes.
    count: Option<i16>,
    bytes: Option<i16>,
    com_out_pointer: bool,
}

// A winmd file that describes the header.
struct Metadata<'a> {
    file: File,
    header: &'a Header,
}

impl Metadata<'_> {
    fn write(header: &Header) -> Vec<u8> {
        let mut metadata = Metadata { file: File::new(NAMESPACE), header };
        for item in &header.enums {
            metadata.enumeration(item);
        }
        for item in &header.structs {
            metadata.structure(item);
        }
        for item in &header.typedefs {
            metadata.typedef(item);
        }
        for item in &header.callbacks {
            metadata.callback(item);
        }
        for item in &header.interfaces {
            metadata.interface(item);
        }
        metadata.apis();
        metadata.file.into_stream()
    }

    // A type that the header names, as C spells it, as metadata has it. An
    // interface is a reference type, so one of its pointers is the interface.
    fn ty(&self, text: &str) -> Type {
        let pointers = text.matches('*').count();
        let name = text.replace('*', "").replace("const ", "");
        let name = name.trim();
        let header = self.header;
        let (ty, reference) = match name {
            "void" => (Type::Void, false),
            "bool" => (Type::Bool, false),
            "float" => (Type::F32, false),
            "char" | "uint8_t" => (Type::U8, false),
            "uint16_t" => (Type::U16, false),
            "wchar_t" => (Type::Char, false),
            "int32_t" | "LONG" => (Type::I32, false),
            "uint32_t" => (Type::U32, false),
            "int64_t" => (Type::I64, false),
            "uint64_t" => (Type::U64, false),
            "size_t" => (Type::USize, false),
            "GUID" => (value("System", "Guid"), false),
            "REFIID" => (Type::PtrMut(Box::new(value("System", "Guid")), 1), false),
            "LPVOID" => (Type::PtrMut(Box::new(Type::Void), 1), false),
            "LPCWSTR" => (value(WIN32, "PCWSTR"), false),
            "HRESULT" => (value(WIN32, "HRESULT"), false),
            name if WIN32_TYPES.contains(&name) => (value(WIN32, name), false),
            name if header.interfaces.iter().any(|item| item.name == name) => (class(NAMESPACE, name), true),
            name if header.callbacks.iter().any(|item| item.name == name) => (class(NAMESPACE, name), false),
            name if header.enums.iter().any(|item| item.name == name)
                || header.structs.iter().any(|item| item.name == name)
                || header.typedefs.iter().any(|item| item.name == name) =>
            {
                (value(NAMESPACE, name), false)
            }
            name => panic!("GameInput.h uses a type that this doesn't know: {name}"),
        };
        match pointers - usize::from(reference) {
            0 => ty,
            pointers => match ty {
                Type::PtrMut(ty, more) => Type::PtrMut(ty, pointers + more),
                ty => Type::PtrMut(Box::new(ty), pointers),
            },
        }
    }

    fn attribute(&mut self, parent: HasAttribute, namespace: &str, name: &str, ctor: Vec<Type>, arguments: Vec<(String, Value)>) {
        let attribute = self.file.TypeRef(namespace, name);
        let signature = Signature { flags: MethodCallAttributes::HASTHIS, return_type: Type::Void, types: ctor };
        let ctor = self.file.MemberRef(".ctor", &signature, MemberRefParent::TypeRef(attribute));
        self.file.Attribute(parent, AttributeType::MemberRef(ctor), &arguments);
    }

    fn guid(&mut self, parent: HasAttribute, guid: u128) {
        let bytes = guid.to_be_bytes();
        let mut ctor = vec![Type::U32, Type::U16, Type::U16];
        let mut arguments = vec![
            Value::U32(u32::from_be_bytes(bytes[0..4].try_into().unwrap())),
            Value::U16(u16::from_be_bytes(bytes[4..6].try_into().unwrap())),
            Value::U16(u16::from_be_bytes(bytes[6..8].try_into().unwrap())),
        ];
        for byte in &bytes[8..] {
            ctor.push(Type::U8);
            arguments.push(Value::U8(*byte));
        }
        let arguments = arguments.into_iter().map(|argument| (String::new(), argument)).collect();
        self.attribute(parent, METADATA, "GuidAttribute", ctor, arguments);
    }

    // An attribute whose one argument is a value of an enum.
    fn enum_attribute(&mut self, parent: HasAttribute, namespace: &str, name: &str, ty: TypeName, value: i32) {
        let argument = Value::EnumValue(ty.clone(), Box::new(Value::I32(value)));
        self.attribute(parent, namespace, name, vec![Type::ValueName(ty)], vec![(String::new(), argument)]);
    }

    fn extends(&mut self, name: &str) -> TypeDefOrRef {
        TypeDefOrRef::TypeRef(self.file.TypeRef("System", name))
    }

    fn enumeration(&mut self, item: &Enum) {
        let extends = self.extends("Enum");
        let def = self.file.TypeDef(NAMESPACE, &item.name, extends, TypeAttributes::Public | TypeAttributes::Sealed);
        let underlying = FieldAttributes::Public | FieldAttributes::SpecialName | FieldAttributes::RTSpecialName;
        self.file.Field("value__", &Type::I32, underlying);
        for (member, number) in &item.members {
            let field = self.file.Field(member, &value(NAMESPACE, &item.name), literal());
            self.file.Constant(HasConstant::Field(field), &Value::I32(*number as u32 as i32));
        }
        if item.flags {
            self.attribute(HasAttribute::TypeDef(def), "System", "FlagsAttribute", vec![], vec![]);
        }
    }

    fn fields(&mut self, fields: &[Field], flags: FieldAttributes, offset: Option<u32>) {
        for field in fields {
            let mut ty = self.ty(&field.ty);
            if let Some(length) = field.length {
                ty = Type::ArrayFixed(Box::new(ty), length);
            }
            let row = self.file.Field(&field.name, &ty, flags);
            if let Some(offset) = offset {
                self.file.FieldLayout(row, offset);
            }
        }
    }

    fn structure(&mut self, item: &Struct) {
        let extends = self.extends("ValueType");
        let def = self.file.TypeDef(NAMESPACE, &item.name, extends, TypeAttributes::Public | TypeAttributes::SequentialLayout);
        self.fields(&item.fields, FieldAttributes::Public, None);
        if let Some((field, fields)) = &item.union {
            // The union, which C++ leaves unnamed, as win32metadata names it.
            let name = format!("_{field}_e__Union");
            self.file.Field(field, &value(NAMESPACE, &format!("{}/{name}", item.name)), FieldAttributes::Public);
            let extends = self.extends("ValueType");
            let flags = TypeAttributes::NestedPublic | TypeAttributes::ExplicitLayout | TypeAttributes::Sealed;
            let union = self.file.TypeDef("", &name, extends, flags);
            self.fields(fields, FieldAttributes::Public, Some(0));
            self.file.NestedClass(union, def);
        }
    }

    // A typedef of a primitive type: a struct of one field, Value.
    fn typedef(&mut self, item: &Typedef) {
        let extends = self.extends("ValueType");
        let def = self.file.TypeDef(NAMESPACE, &item.name, extends, TypeAttributes::Public | TypeAttributes::SequentialLayout);
        let ty = self.ty(&item.ty);
        self.file.Field("Value", &ty, FieldAttributes::Public);
        self.attribute(HasAttribute::TypeDef(def), METADATA, "NativeTypedefAttribute", vec![], vec![]);
    }

    fn callback(&mut self, item: &Callback) {
        let params = self.params(&item.params);
        let extends = self.extends("MulticastDelegate");
        let def = self.file.TypeDef(NAMESPACE, &item.name, extends, TypeAttributes::Public | TypeAttributes::Sealed);
        let flags = MethodAttributes::Public | MethodAttributes::HideBySig | MethodAttributes::NewSlot | MethodAttributes::Virtual;
        self.method("Invoke", Type::Void, &params, MethodCallAttributes::HASTHIS, flags, MethodImplAttributes::Runtime);
        // CALLBACK is __stdcall: CallingConvention.Winapi.
        let convention = TypeName::named("System.Runtime.InteropServices", "CallingConvention");
        self.enum_attribute(HasAttribute::TypeDef(def), "System.Runtime.InteropServices", "UnmanagedFunctionPointerAttribute", convention, 1);
    }

    fn interface(&mut self, item: &Interface) {
        let flags = TypeAttributes::Public | TypeAttributes::Interface | TypeAttributes::Abstract;
        let def = self.file.TypeDef(NAMESPACE, &item.name, TypeDefOrRef::default(), flags);
        for method in &item.methods {
            let params = self.params(&method.params);
            let returns = self.ty(&method.returns);
            let flags = MethodAttributes::Public
                | MethodAttributes::Virtual
                | MethodAttributes::HideBySig
                | MethodAttributes::NewSlot
                | MethodAttributes::Abstract;
            self.method(&method.name, returns, &params, MethodCallAttributes::HASTHIS, flags, MethodImplAttributes::default());
        }
        self.file.InterfaceImpl(def, &class(WIN32, "IUnknown"));
        self.guid(HasAttribute::TypeDef(def), item.guid);
        // MarshalingType.Agile.
        let marshaling = TypeName::named("Windows.Foundation.Metadata", "MarshalingType");
        self.enum_attribute(HasAttribute::TypeDef(def), "Windows.Foundation.Metadata", "MarshalingBehaviorAttribute", marshaling, 2);
    }

    // The constants and the functions.
    fn apis(&mut self) {
        let extends = self.extends("Object");
        let flags = TypeAttributes::Public | TypeAttributes::Abstract | TypeAttributes::Sealed;
        self.file.TypeDef(NAMESPACE, "Apis", extends, flags);
        let version = self.file.Field("GAMEINPUT_API_VERSION", &Type::I32, literal());
        self.file.Constant(HasConstant::Field(version), &Value::I32(self.header.version));
        let header = self.header;
        for constant in &header.constants {
            let ty = self.ty(&constant.ty);
            match constant.value {
                header::Value::Integer(integer) => {
                    let field = self.file.Field(&constant.name, &ty, literal());
                    let integer = match ty {
                        Type::U32 => Value::U32(integer as u32),
                        _ => Value::I32(integer as u32 as i32),
                    };
                    self.file.Constant(HasConstant::Field(field), &integer);
                }
                header::Value::Guid(guid) => {
                    let field = self.file.Field(&constant.name, &ty, FieldAttributes::Public | FieldAttributes::Static);
                    self.guid(HasAttribute::Field(field), guid);
                }
            }
        }
        for function in &header.functions {
            let params = self.params(&function.params);
            let returns = self.ty(&function.returns);
            let flags = MethodAttributes::Public | MethodAttributes::Static | MethodAttributes::HideBySig | MethodAttributes::PInvokeImpl;
            let method =
                self.method(&function.name, returns, &params, MethodCallAttributes(0), flags, MethodImplAttributes::PreserveSig);
            let convention = PInvokeAttributes::NoMangle | PInvokeAttributes::CallConvPlatformapi;
            self.file.ImplMap(method, convention, &function.name, "GameInput.lib");
        }
    }

    fn method(
        &mut self,
        name: &str,
        returns: Type,
        params: &[ParamRow],
        call: MethodCallAttributes,
        flags: MethodAttributes,
        implementation: MethodImplAttributes,
    ) -> windows_metadata::writer::MethodDef {
        let signature = Signature { flags: call, return_type: returns, types: params.iter().map(|param| param.ty.clone()).collect() };
        let method = self.file.MethodDef(name, &signature, flags, implementation);
        for (index, param) in params.iter().enumerate() {
            let row = HasAttribute::Param(self.file.Param(&param.name, index as u16 + 1, param.flags));
            let index = |name: &str, value: Option<i16>| value.map(|value| (name.to_string(), Value::I16(value)));
            if let Some(count) = index("CountParamIndex", param.count) {
                self.attribute(row, METADATA, "NativeArrayInfoAttribute", vec![], vec![count]);
            }
            if let Some(bytes) = index("BytesParamIndex", param.bytes) {
                self.attribute(row, METADATA, "MemorySizeAttribute", vec![], vec![bytes]);
            }
            if param.com_out_pointer {
                self.attribute(row, METADATA, "ComOutPtrAttribute", vec![], vec![]);
            }
        }
        method
    }

    // Parameters, with what their SAL annotations say about them.
    fn params(&self, params: &[Param]) -> Vec<ParamRow> {
        let position = |argument: &Option<String>| -> Option<i16> {
            // _Out_writes_to_opt_(contactArrayCount, return)'s first argument.
            let name = argument.as_deref().expect("a buffer without its length").split(',').next().unwrap().trim();
            let position = params.iter().position(|param| param.name == name);
            Some(position.unwrap_or_else(|| panic!("no parameter {name}")) as i16)
        };
        params
            .iter()
            .map(|param| {
                let mut row = ParamRow {
                    name: param.name.clone(),
                    ty: self.ty(&param.ty),
                    flags: ParamAttributes::In,
                    count: None,
                    bytes: None,
                    com_out_pointer: false,
                };
                let (input, output, optional) = (ParamAttributes::In, ParamAttributes::Out, ParamAttributes::Optional);
                for (annotation, argument) in &param.annotations {
                    match annotation.as_str() {
                        "_In_" => row.flags = input,
                        "_In_opt_" => row.flags = input | optional,
                        "_Out_" | "_Outptr_" | "_Outptr_result_maybenull_" | "_Outptr_result_nullonfailure_" => row.flags = output,
                        "_Out_opt_" => row.flags = output | optional,
                        "_COM_Outptr_" => (row.flags, row.com_out_pointer) = (output, true),
                        "_In_reads_" => (row.flags, row.count) = (input, position(argument)),
                        "_Out_writes_" => (row.flags, row.count) = (output, position(argument)),
                        "_Out_writes_to_opt_" => (row.flags, row.count) = (output | optional, position(argument)),
                        "_In_reads_bytes_" => (row.flags, row.bytes) = (input, position(argument)),
                        "_Out_writes_bytes_" => (row.flags, row.bytes) = (output, position(argument)),
                        "_Out_writes_bytes_opt_" => (row.flags, row.bytes) = (output | optional, position(argument)),
                        // What a failure leaves in an output, which the
                        // bindings don't say.
                        "_Result_zeroonfailure_" => {}
                        _ => panic!("{} has a SAL annotation that this doesn't know: {annotation}", param.name),
                    }
                }
                // A buffer of void is a buffer of bytes.
                if (row.count.is_some() || row.bytes.is_some())
                    && matches!(&row.ty, Type::PtrMut(ty, 1) if **ty == Type::Void)
                {
                    row.ty = Type::PtrMut(Box::new(Type::U8), 1);
                }
                row
            })
            .collect()
    }
}

fn literal() -> FieldAttributes {
    FieldAttributes::Public | FieldAttributes::Static | FieldAttributes::Literal | FieldAttributes::HasDefault
}

fn value(namespace: &str, name: &str) -> Type {
    Type::ValueName(TypeName::named(namespace, name))
}

fn class(namespace: &str, name: &str) -> Type {
    Type::ClassName(TypeName::named(namespace, name))
}
