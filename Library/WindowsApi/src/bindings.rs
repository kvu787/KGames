#[inline]
pub unsafe fn AdjustWindowRectExForDpi(
    lprect: *mut RECT,
    dwstyle: u32,
    bmenu: bool,
    dwexstyle: u32,
    dpi: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn AdjustWindowRectExForDpi(lprect : *mut RECT, dwstyle : u32, bmenu : windows_core::BOOL, dwexstyle : u32, dpi : u32) -> windows_core::BOOL);
    unsafe { AdjustWindowRectExForDpi(lprect as _, dwstyle, bmenu.into(), dwexstyle, dpi) }
}
#[inline]
pub unsafe fn BeginPaint(hwnd: HWND, lppaint: *mut PAINTSTRUCT) -> HDC {
    windows_core::link!("user32.dll" "system" fn BeginPaint(hwnd : HWND, lppaint : *mut PAINTSTRUCT) -> HDC);
    unsafe { BeginPaint(hwnd, lppaint as _) }
}
#[inline]
pub unsafe fn CM_Get_DevNode_PropertyW(
    dndevinst: DEVINST,
    propertykey: *const DEVPROPKEY,
    propertytype: *mut DEVPROPTYPE,
    propertybuffer: Option<*mut u8>,
    propertybuffersize: *mut u32,
    ulflags: u32,
) -> CONFIGRET {
    windows_core::link!("cfgmgr32.dll" "system" fn CM_Get_DevNode_PropertyW(dndevinst : DEVINST, propertykey : *const DEVPROPKEY, propertytype : *mut DEVPROPTYPE, propertybuffer : *mut u8, propertybuffersize : *mut u32, ulflags : u32) -> CONFIGRET);
    unsafe {
        CM_Get_DevNode_PropertyW(
            dndevinst,
            propertykey,
            propertytype as _,
            propertybuffer.unwrap_or(core::mem::zeroed()) as _,
            propertybuffersize as _,
            ulflags,
        )
    }
}
#[inline]
pub unsafe fn CM_Get_Device_Interface_PropertyW<P0>(
    pszdeviceinterface: P0,
    propertykey: *const DEVPROPKEY,
    propertytype: *mut DEVPROPTYPE,
    propertybuffer: Option<*mut u8>,
    propertybuffersize: *mut u32,
    ulflags: u32,
) -> CONFIGRET
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("cfgmgr32.dll" "system" fn CM_Get_Device_Interface_PropertyW(pszdeviceinterface : windows_core::PCWSTR, propertykey : *const DEVPROPKEY, propertytype : *mut DEVPROPTYPE, propertybuffer : *mut u8, propertybuffersize : *mut u32, ulflags : u32) -> CONFIGRET);
    unsafe {
        CM_Get_Device_Interface_PropertyW(
            pszdeviceinterface.param().abi(),
            propertykey,
            propertytype as _,
            propertybuffer.unwrap_or(core::mem::zeroed()) as _,
            propertybuffersize as _,
            ulflags,
        )
    }
}
#[inline]
pub unsafe fn CM_Get_Parent(
    pdndevinst: *mut DEVNODE,
    dndevinst: DEVINST,
    ulflags: u32,
) -> CONFIGRET {
    windows_core::link!("setupapi.dll" "system" fn CM_Get_Parent(pdndevinst : *mut DEVNODE, dndevinst : DEVINST, ulflags : u32) -> CONFIGRET);
    unsafe { CM_Get_Parent(pdndevinst as _, dndevinst, ulflags) }
}
#[inline]
pub unsafe fn CM_Locate_DevNodeW(
    pdndevinst: *mut DEVNODE,
    pdeviceid: Option<*const u16>,
    ulflags: u32,
) -> CONFIGRET {
    windows_core::link!("setupapi.dll" "system" fn CM_Locate_DevNodeW(pdndevinst : *mut DEVNODE, pdeviceid : *const u16, ulflags : u32) -> CONFIGRET);
    unsafe {
        CM_Locate_DevNodeW(
            pdndevinst as _,
            pdeviceid.unwrap_or(core::mem::zeroed()) as _,
            ulflags,
        )
    }
}
#[inline]
pub unsafe fn CloseHandle(hobject: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
    unsafe { CloseHandle(hobject) }
}
#[inline]
pub unsafe fn CloseTrace(tracehandle: PROCESSTRACE_HANDLE) -> u32 {
    windows_core::link!("advapi32.dll" "system" fn CloseTrace(tracehandle : PROCESSTRACE_HANDLE) -> u32);
    unsafe { CloseTrace(tracehandle) }
}
#[inline]
pub unsafe fn CoCreateInstance<P1, T>(
    rclsid: *const windows_core::GUID,
    punkouter: P1,
    dwclscontext: u32,
) -> windows_core::Result<T>
where
    P1: windows_core::Param<windows_core::IUnknown>,
    T: windows_core::Interface,
{
    windows_core::link!("ole32.dll" "system" fn CoCreateInstance(rclsid : *const windows_core::GUID, punkouter : *mut core::ffi::c_void, dwclscontext : u32, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        CoCreateInstance(
            rclsid,
            punkouter.param().abi(),
            dwclscontext,
            &T::IID,
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn CoInitializeEx(
    pvreserved: Option<*const core::ffi::c_void>,
    dwcoinit: u32,
) -> windows_core::HRESULT {
    windows_core::link!("ole32.dll" "system" fn CoInitializeEx(pvreserved : *const core::ffi::c_void, dwcoinit : u32) -> windows_core::HRESULT);
    unsafe { CoInitializeEx(pvreserved.unwrap_or(core::mem::zeroed()) as _, dwcoinit) }
}
#[inline]
pub unsafe fn ControlTraceW<P1>(
    traceid: CONTROLTRACE_ID,
    instancename: P1,
    properties: *mut EVENT_TRACE_PROPERTIES,
    controlcode: u32,
) -> u32
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn ControlTraceW(traceid : CONTROLTRACE_ID, instancename : windows_core::PCWSTR, properties : *mut EVENT_TRACE_PROPERTIES, controlcode : u32) -> u32);
    unsafe {
        ControlTraceW(
            traceid,
            instancename.param().abi(),
            properties as _,
            controlcode,
        )
    }
}
#[inline]
pub unsafe fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP {
    windows_core::link!("gdi32.dll" "system" fn CreateCompatibleBitmap(hdc : HDC, cx : i32, cy : i32) -> HBITMAP);
    unsafe { CreateCompatibleBitmap(hdc, cx, cy) }
}
#[inline]
pub unsafe fn CreateCompatibleDC(hdc: Option<HDC>) -> HDC {
    windows_core::link!("gdi32.dll" "system" fn CreateCompatibleDC(hdc : HDC) -> HDC);
    unsafe { CreateCompatibleDC(hdc.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn CreateDXGIFactory2<T>(flags: u32) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("dxgi.dll" "system" fn CreateDXGIFactory2(flags : u32, riid : *const windows_core::GUID, ppfactory : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        CreateDXGIFactory2(flags, &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn CreateEventW<P3>(
    lpeventattributes: Option<*const SECURITY_ATTRIBUTES>,
    bmanualreset: bool,
    binitialstate: bool,
    lpname: P3,
) -> HANDLE
where
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateEventW(lpeventattributes : *const SECURITY_ATTRIBUTES, bmanualreset : windows_core::BOOL, binitialstate : windows_core::BOOL, lpname : windows_core::PCWSTR) -> HANDLE);
    unsafe {
        CreateEventW(
            lpeventattributes.unwrap_or(core::mem::zeroed()) as _,
            bmanualreset.into(),
            binitialstate.into(),
            lpname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn CreateToolhelp32Snapshot(dwflags: u32, th32processid: u32) -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn CreateToolhelp32Snapshot(dwflags : u32, th32processid : u32) -> HANDLE);
    unsafe { CreateToolhelp32Snapshot(dwflags, th32processid) }
}
#[inline]
pub unsafe fn CreateWindowExW<P1, P2>(
    dwexstyle: u32,
    lpclassname: P1,
    lpwindowname: P2,
    dwstyle: u32,
    x: i32,
    y: i32,
    nwidth: i32,
    nheight: i32,
    hwndparent: Option<HWND>,
    hmenu: Option<HMENU>,
    hinstance: Option<HINSTANCE>,
    lpparam: Option<*const core::ffi::c_void>,
) -> HWND
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn CreateWindowExW(dwexstyle : u32, lpclassname : windows_core::PCWSTR, lpwindowname : windows_core::PCWSTR, dwstyle : u32, x : i32, y : i32, nwidth : i32, nheight : i32, hwndparent : HWND, hmenu : HMENU, hinstance : HINSTANCE, lpparam : *const core::ffi::c_void) -> HWND);
    unsafe {
        CreateWindowExW(
            dwexstyle,
            lpclassname.param().abi(),
            lpwindowname.param().abi(),
            dwstyle,
            x,
            y,
            nwidth,
            nheight,
            hwndparent.unwrap_or(core::mem::zeroed()) as _,
            hmenu.unwrap_or(core::mem::zeroed()) as _,
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            lpparam.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn D3D12CreateDevice<P0, T>(
    padapter: P0,
    minimumfeaturelevel: D3D_FEATURE_LEVEL,
    result__: *mut Option<T>,
) -> windows_core::Result<()>
where
    P0: windows_core::Param<windows_core::IUnknown>,
    T: windows_core::Interface,
{
    windows_core::link!("d3d12.dll" "system" fn D3D12CreateDevice(padapter : *mut core::ffi::c_void, minimumfeaturelevel : D3D_FEATURE_LEVEL, riid : *const windows_core::GUID, ppdevice : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        D3D12CreateDevice(
            padapter.param().abi(),
            minimumfeaturelevel,
            &T::IID,
            result__ as *mut _ as *mut _,
        )
        .ok()
    }
}
#[inline]
pub unsafe fn D3D12GetDebugInterface<T>(result__: *mut Option<T>) -> windows_core::Result<()>
where
    T: windows_core::Interface,
{
    windows_core::link!("d3d12.dll" "system" fn D3D12GetDebugInterface(riid : *const windows_core::GUID, ppvdebug : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe { D3D12GetDebugInterface(&T::IID, result__ as *mut _ as *mut _).ok() }
}
#[inline]
pub unsafe fn D3DKMTGetProcessSchedulingPriorityClass(
    param0: HANDLE,
    param1: *mut D3DKMT_SCHEDULINGPRIORITYCLASS,
) -> windows_core::NTSTATUS {
    windows_core::link!("gdi32.dll" "system" fn D3DKMTGetProcessSchedulingPriorityClass(param0 : HANDLE, param1 : *mut D3DKMT_SCHEDULINGPRIORITYCLASS) -> windows_core::NTSTATUS);
    unsafe { D3DKMTGetProcessSchedulingPriorityClass(param0, param1 as _) }
}
#[inline]
pub unsafe fn D3DKMTSetProcessSchedulingPriorityClass(
    param0: HANDLE,
    param1: D3DKMT_SCHEDULINGPRIORITYCLASS,
) -> windows_core::NTSTATUS {
    windows_core::link!("gdi32.dll" "system" fn D3DKMTSetProcessSchedulingPriorityClass(param0 : HANDLE, param1 : D3DKMT_SCHEDULINGPRIORITYCLASS) -> windows_core::NTSTATUS);
    unsafe { D3DKMTSetProcessSchedulingPriorityClass(param0, param1) }
}
#[inline]
pub unsafe fn DWriteCreateFactory<T>(factorytype: DWRITE_FACTORY_TYPE) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("dwrite.dll" "system" fn DWriteCreateFactory(factorytype : DWRITE_FACTORY_TYPE, iid : *const windows_core::GUID, factory : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        DWriteCreateFactory(factorytype, &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn DefWindowProcW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn DefWindowProcW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn DeleteDC(hdc: HDC) -> windows_core::BOOL {
    windows_core::link!("gdi32.dll" "system" fn DeleteDC(hdc : HDC) -> windows_core::BOOL);
    unsafe { DeleteDC(hdc) }
}
#[inline]
pub unsafe fn DeleteObject(ho: HGDIOBJ) -> windows_core::BOOL {
    windows_core::link!("gdi32.dll" "system" fn DeleteObject(ho : HGDIOBJ) -> windows_core::BOOL);
    unsafe { DeleteObject(ho) }
}
#[inline]
pub unsafe fn DestroyWindow(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn DestroyWindow(hwnd : HWND) -> windows_core::BOOL);
    unsafe { DestroyWindow(hwnd) }
}
#[inline]
pub unsafe fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn DispatchMessageW(lpmsg : *const MSG) -> LRESULT);
    unsafe { DispatchMessageW(lpmsg) }
}
#[inline]
pub unsafe fn DisplayConfigGetDeviceInfo(
    requestpacket: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
) -> i32 {
    windows_core::link!("user32.dll" "system" fn DisplayConfigGetDeviceInfo(requestpacket : *mut DISPLAYCONFIG_DEVICE_INFO_HEADER) -> i32);
    unsafe { DisplayConfigGetDeviceInfo(requestpacket as _) }
}
#[inline]
pub unsafe fn DwmSetWindowAttribute(
    hwnd: HWND,
    dwattribute: u32,
    pvattribute: *const core::ffi::c_void,
    cbattribute: u32,
) -> windows_core::HRESULT {
    windows_core::link!("dwmapi.dll" "system" fn DwmSetWindowAttribute(hwnd : HWND, dwattribute : u32, pvattribute : *const core::ffi::c_void, cbattribute : u32) -> windows_core::HRESULT);
    unsafe { DwmSetWindowAttribute(hwnd, dwattribute, pvattribute, cbattribute) }
}
#[inline]
pub unsafe fn EnableTraceEx2(
    traceid: CONTROLTRACE_ID,
    providerid: *const windows_core::GUID,
    controlcode: u32,
    level: u8,
    matchanykeyword: u64,
    matchallkeyword: u64,
    timeout: u32,
    enableparameters: Option<*const ENABLE_TRACE_PARAMETERS>,
) -> u32 {
    windows_core::link!("advapi32.dll" "system" fn EnableTraceEx2(traceid : CONTROLTRACE_ID, providerid : *const windows_core::GUID, controlcode : u32, level : u8, matchanykeyword : u64, matchallkeyword : u64, timeout : u32, enableparameters : *const ENABLE_TRACE_PARAMETERS) -> u32);
    unsafe {
        EnableTraceEx2(
            traceid,
            providerid,
            controlcode,
            level,
            matchanykeyword,
            matchallkeyword,
            timeout,
            enableparameters.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn EndPaint(hwnd: HWND, lppaint: *const PAINTSTRUCT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn EndPaint(hwnd : HWND, lppaint : *const PAINTSTRUCT) -> windows_core::BOOL);
    unsafe { EndPaint(hwnd, lppaint) }
}
#[inline]
pub unsafe fn EnumDisplayMonitors(
    hdc: Option<HDC>,
    lprcclip: Option<*const RECT>,
    lpfnenum: MONITORENUMPROC,
    dwdata: LPARAM,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn EnumDisplayMonitors(hdc : HDC, lprcclip : *const RECT, lpfnenum : MONITORENUMPROC, dwdata : LPARAM) -> windows_core::BOOL);
    unsafe {
        EnumDisplayMonitors(
            hdc.unwrap_or(core::mem::zeroed()) as _,
            lprcclip.unwrap_or(core::mem::zeroed()) as _,
            lpfnenum,
            dwdata,
        )
    }
}
#[inline]
pub unsafe fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32 {
    windows_core::link!("user32.dll" "system" fn FillRect(hdc : HDC, lprc : *const RECT, hbr : HBRUSH) -> i32);
    unsafe { FillRect(hdc, lprc, hbr) }
}
#[inline]
pub unsafe fn GetCapture() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetCapture() -> HWND);
    unsafe { GetCapture() }
}
#[inline]
pub unsafe fn GetClientRect(hwnd: HWND, lprect: *mut RECT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetClientRect(hwnd : HWND, lprect : *mut RECT) -> windows_core::BOOL);
    unsafe { GetClientRect(hwnd, lprect as _) }
}
#[inline]
pub unsafe fn GetCurrentProcess() -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
    unsafe { GetCurrentProcess() }
}
#[inline]
pub unsafe fn GetCurrentProcessId() -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentProcessId() -> u32);
    unsafe { GetCurrentProcessId() }
}
#[inline]
pub unsafe fn GetCursorPos(lppoint: *mut POINT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetCursorPos(lppoint : *mut POINT) -> windows_core::BOOL);
    unsafe { GetCursorPos(lppoint as _) }
}
#[inline]
pub unsafe fn GetDC(hwnd: Option<HWND>) -> HDC {
    windows_core::link!("user32.dll" "system" fn GetDC(hwnd : HWND) -> HDC);
    unsafe { GetDC(hwnd.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetDisplayConfigBufferSizes(
    flags: u32,
    numpatharrayelements: *mut u32,
    nummodeinfoarrayelements: *mut u32,
) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetDisplayConfigBufferSizes(flags : u32, numpatharrayelements : *mut u32, nummodeinfoarrayelements : *mut u32) -> i32);
    unsafe {
        GetDisplayConfigBufferSizes(
            flags,
            numpatharrayelements as _,
            nummodeinfoarrayelements as _,
        )
    }
}
#[inline]
pub unsafe fn GetDpiForMonitor(
    hmonitor: HMONITOR,
    dpitype: MONITOR_DPI_TYPE,
    dpix: *mut u32,
    dpiy: *mut u32,
) -> windows_core::HRESULT {
    windows_core::link!("api-ms-win-shcore-scaling-l1-1-1.dll" "system" fn GetDpiForMonitor(hmonitor : HMONITOR, dpitype : MONITOR_DPI_TYPE, dpix : *mut u32, dpiy : *mut u32) -> windows_core::HRESULT);
    unsafe { GetDpiForMonitor(hmonitor, dpitype, dpix as _, dpiy as _) }
}
#[inline]
pub unsafe fn GetDpiForWindow(hwnd: HWND) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetDpiForWindow(hwnd : HWND) -> u32);
    unsafe { GetDpiForWindow(hwnd) }
}
#[inline]
pub unsafe fn GetExitCodeProcess(hprocess: HANDLE, lpexitcode: *mut u32) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GetExitCodeProcess(hprocess : HANDLE, lpexitcode : *mut u32) -> windows_core::BOOL);
    unsafe { GetExitCodeProcess(hprocess, lpexitcode as _) }
}
#[inline]
pub unsafe fn GetKeyNameTextW(lparam: i32, lpstring: windows_core::PWSTR, cchsize: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetKeyNameTextW(lparam : i32, lpstring : windows_core::PWSTR, cchsize : i32) -> i32);
    unsafe { GetKeyNameTextW(lparam, lpstring, cchsize) }
}
#[inline]
pub unsafe fn GetKeyState(nvirtkey: i32) -> i16 {
    windows_core::link!("user32.dll" "system" fn GetKeyState(nvirtkey : i32) -> i16);
    unsafe { GetKeyState(nvirtkey) }
}
#[inline]
pub unsafe fn GetKeyboardLayout(idthread: u32) -> HKL {
    windows_core::link!("user32.dll" "system" fn GetKeyboardLayout(idthread : u32) -> HKL);
    unsafe { GetKeyboardLayout(idthread) }
}
#[inline]
pub unsafe fn GetModuleFileNameW(
    hmodule: Option<HMODULE>,
    lpfilename: windows_core::PWSTR,
    nsize: u32,
) -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetModuleFileNameW(hmodule : HMODULE, lpfilename : windows_core::PWSTR, nsize : u32) -> u32);
    unsafe {
        GetModuleFileNameW(
            hmodule.unwrap_or(core::mem::zeroed()) as _,
            lpfilename,
            nsize,
        )
    }
}
#[inline]
pub unsafe fn GetModuleHandleExW<P1>(
    dwflags: u32,
    lpmodulename: P1,
    phmodule: *mut HMODULE,
) -> windows_core::BOOL
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetModuleHandleExW(dwflags : u32, lpmodulename : windows_core::PCWSTR, phmodule : *mut HMODULE) -> windows_core::BOOL);
    unsafe { GetModuleHandleExW(dwflags, lpmodulename.param().abi(), phmodule as _) }
}
#[inline]
pub unsafe fn GetModuleHandleW<P0>(lpmodulename: P0) -> HMODULE
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetModuleHandleW(lpmodulename : windows_core::PCWSTR) -> HMODULE);
    unsafe { GetModuleHandleW(lpmodulename.param().abi()) }
}
#[inline]
pub unsafe fn GetMonitorInfoW(hmonitor: HMONITOR, lpmi: *mut MONITORINFO) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetMonitorInfoW(hmonitor : HMONITOR, lpmi : *mut MONITORINFO) -> windows_core::BOOL);
    unsafe { GetMonitorInfoW(hmonitor, lpmi as _) }
}
#[inline]
pub unsafe fn GetPixel(hdc: HDC, x: i32, y: i32) -> COLORREF {
    windows_core::link!("gdi32.dll" "system" fn GetPixel(hdc : HDC, x : i32, y : i32) -> COLORREF);
    unsafe { GetPixel(hdc, x, y) }
}
#[inline]
pub unsafe fn GetProcessMemoryInfo(
    process: HANDLE,
    ppsmemcounters: *mut PROCESS_MEMORY_COUNTERS,
    cb: u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" "K32GetProcessMemoryInfo" fn GetProcessMemoryInfo(process : HANDLE, ppsmemcounters : *mut PROCESS_MEMORY_COUNTERS, cb : u32) -> windows_core::BOOL);
    unsafe { GetProcessMemoryInfo(process, ppsmemcounters as _, cb) }
}
#[inline]
pub unsafe fn GetStockObject(i: i32) -> HGDIOBJ {
    windows_core::link!("gdi32.dll" "system" fn GetStockObject(i : i32) -> HGDIOBJ);
    unsafe { GetStockObject(i) }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[inline]
pub unsafe fn GetWindowLongPtrW(hwnd: HWND, nindex: i32) -> isize {
    windows_core::link!("user32.dll" "system" fn GetWindowLongPtrW(hwnd : HWND, nindex : i32) -> isize);
    unsafe { GetWindowLongPtrW(hwnd, nindex) }
}
#[cfg(target_pointer_width = "32")]
pub use GetWindowLongW as GetWindowLongPtrW;
#[inline]
pub unsafe fn GetWindowLongW(hwnd: HWND, nindex: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetWindowLongW(hwnd : HWND, nindex : i32) -> i32);
    unsafe { GetWindowLongW(hwnd, nindex) }
}
#[inline]
pub unsafe fn IsZoomed(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn IsZoomed(hwnd : HWND) -> windows_core::BOOL);
    unsafe { IsZoomed(hwnd) }
}
#[inline]
pub unsafe fn LoadCursorW<P1>(hinstance: Option<HINSTANCE>, lpcursorname: P1) -> HCURSOR
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn LoadCursorW(hinstance : HINSTANCE, lpcursorname : windows_core::PCWSTR) -> HCURSOR);
    unsafe {
        LoadCursorW(
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            lpcursorname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn MapVirtualKeyW(ucode: u32, umaptype: u32) -> u32 {
    windows_core::link!("user32.dll" "system" fn MapVirtualKeyW(ucode : u32, umaptype : u32) -> u32);
    unsafe { MapVirtualKeyW(ucode, umaptype) }
}
#[inline]
pub unsafe fn Module32FirstW(hsnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn Module32FirstW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
    unsafe { Module32FirstW(hsnapshot, lpme as _) }
}
#[inline]
pub unsafe fn Module32NextW(hsnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn Module32NextW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
    unsafe { Module32NextW(hsnapshot, lpme as _) }
}
#[inline]
pub unsafe fn MonitorFromPoint(pt: POINT, dwflags: u32) -> HMONITOR {
    windows_core::link!("user32.dll" "system" fn MonitorFromPoint(pt : POINT, dwflags : u32) -> HMONITOR);
    unsafe { MonitorFromPoint(pt, dwflags) }
}
#[inline]
pub unsafe fn MonitorFromWindow(hwnd: HWND, dwflags: u32) -> HMONITOR {
    windows_core::link!("user32.dll" "system" fn MonitorFromWindow(hwnd : HWND, dwflags : u32) -> HMONITOR);
    unsafe { MonitorFromWindow(hwnd, dwflags) }
}
#[inline]
pub unsafe fn MsgWaitForMultipleObjects(
    phandles: Option<&[HANDLE]>,
    fwaitall: bool,
    dwmilliseconds: u32,
    dwwakemask: u32,
) -> u32 {
    windows_core::link!("user32.dll" "system" fn MsgWaitForMultipleObjects(ncount : u32, phandles : *const HANDLE, fwaitall : windows_core::BOOL, dwmilliseconds : u32, dwwakemask : u32) -> u32);
    unsafe {
        MsgWaitForMultipleObjects(
            phandles.map_or(0, |slice| slice.len().try_into().unwrap()),
            phandles.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            fwaitall.into(),
            dwmilliseconds,
            dwwakemask,
        )
    }
}
#[inline]
pub unsafe fn OpenProcess(dwdesiredaccess: u32, binherithandle: bool, dwprocessid: u32) -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn OpenProcess(dwdesiredaccess : u32, binherithandle : windows_core::BOOL, dwprocessid : u32) -> HANDLE);
    unsafe { OpenProcess(dwdesiredaccess, binherithandle.into(), dwprocessid) }
}
#[inline]
pub unsafe fn OpenTraceW(logfile: *mut EVENT_TRACE_LOGFILEW) -> PROCESSTRACE_HANDLE {
    windows_core::link!("advapi32.dll" "system" fn OpenTraceW(logfile : *mut EVENT_TRACE_LOGFILEW) -> PROCESSTRACE_HANDLE);
    unsafe { OpenTraceW(logfile as _) }
}
#[inline]
pub unsafe fn PeekMessageW(
    lpmsg: *mut MSG,
    hwnd: Option<HWND>,
    wmsgfiltermin: u32,
    wmsgfiltermax: u32,
    wremovemsg: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn PeekMessageW(lpmsg : *mut MSG, hwnd : HWND, wmsgfiltermin : u32, wmsgfiltermax : u32, wremovemsg : u32) -> windows_core::BOOL);
    unsafe {
        PeekMessageW(
            lpmsg as _,
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            wmsgfiltermin,
            wmsgfiltermax,
            wremovemsg,
        )
    }
}
#[inline]
pub unsafe fn Process32FirstW(hsnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn Process32FirstW(hsnapshot : HANDLE, lppe : *mut PROCESSENTRY32W) -> windows_core::BOOL);
    unsafe { Process32FirstW(hsnapshot, lppe as _) }
}
#[inline]
pub unsafe fn Process32NextW(hsnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn Process32NextW(hsnapshot : HANDLE, lppe : *mut PROCESSENTRY32W) -> windows_core::BOOL);
    unsafe { Process32NextW(hsnapshot, lppe as _) }
}
#[inline]
pub unsafe fn ProcessTrace(
    handlearray: &[PROCESSTRACE_HANDLE],
    starttime: Option<*const FILETIME>,
    endtime: Option<*const FILETIME>,
) -> u32 {
    windows_core::link!("advapi32.dll" "system" fn ProcessTrace(handlearray : *const PROCESSTRACE_HANDLE, handlecount : u32, starttime : *const FILETIME, endtime : *const FILETIME) -> u32);
    unsafe {
        ProcessTrace(
            handlearray.as_ptr(),
            handlearray.len().try_into().unwrap(),
            starttime.unwrap_or(core::mem::zeroed()) as _,
            endtime.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn QueryAllTracesW(
    propertyarray: *mut PEVENT_TRACE_PROPERTIES,
    propertyarraycount: u32,
    loggercount: *mut u32,
) -> u32 {
    windows_core::link!("advapi32.dll" "system" fn QueryAllTracesW(propertyarray : *mut PEVENT_TRACE_PROPERTIES, propertyarraycount : u32, loggercount : *mut u32) -> u32);
    unsafe { QueryAllTracesW(propertyarray as _, propertyarraycount, loggercount as _) }
}
#[inline]
pub unsafe fn QueryDisplayConfig(
    flags: u32,
    numpatharrayelements: *mut u32,
    patharray: *mut DISPLAYCONFIG_PATH_INFO,
    nummodeinfoarrayelements: *mut u32,
    modeinfoarray: *mut DISPLAYCONFIG_MODE_INFO,
    currenttopologyid: *mut DISPLAYCONFIG_TOPOLOGY_ID,
) -> i32 {
    windows_core::link!("user32.dll" "system" fn QueryDisplayConfig(flags : u32, numpatharrayelements : *mut u32, patharray : *mut DISPLAYCONFIG_PATH_INFO, nummodeinfoarrayelements : *mut u32, modeinfoarray : *mut DISPLAYCONFIG_MODE_INFO, currenttopologyid : *mut DISPLAYCONFIG_TOPOLOGY_ID) -> i32);
    unsafe {
        QueryDisplayConfig(
            flags,
            numpatharrayelements as _,
            patharray as _,
            nummodeinfoarrayelements as _,
            modeinfoarray as _,
            currenttopologyid as _,
        )
    }
}
#[inline]
pub unsafe fn QueryPerformanceCounter(lpperformancecount: *mut i64) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn QueryPerformanceCounter(lpperformancecount : *mut i64) -> windows_core::BOOL);
    unsafe { QueryPerformanceCounter(lpperformancecount as _) }
}
#[inline]
pub unsafe fn QueryPerformanceFrequency(lpfrequency: *mut i64) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn QueryPerformanceFrequency(lpfrequency : *mut i64) -> windows_core::BOOL);
    unsafe { QueryPerformanceFrequency(lpfrequency as _) }
}
#[inline]
pub unsafe fn RegisterClassExW(param0: *const WNDCLASSEXW) -> ATOM {
    windows_core::link!("user32.dll" "system" fn RegisterClassExW(param0 : *const WNDCLASSEXW) -> ATOM);
    unsafe { RegisterClassExW(param0) }
}
#[inline]
pub unsafe fn ReleaseCapture() -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ReleaseCapture() -> windows_core::BOOL);
    unsafe { ReleaseCapture() }
}
#[inline]
pub unsafe fn ReleaseDC(hwnd: Option<HWND>, hdc: HDC) -> i32 {
    windows_core::link!("user32.dll" "system" fn ReleaseDC(hwnd : HWND, hdc : HDC) -> i32);
    unsafe { ReleaseDC(hwnd.unwrap_or(core::mem::zeroed()) as _, hdc) }
}
#[inline]
pub unsafe fn ScreenToClient(hwnd: HWND, lppoint: *mut POINT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ScreenToClient(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
    unsafe { ScreenToClient(hwnd, lppoint as _) }
}
#[inline]
pub unsafe fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ {
    windows_core::link!("gdi32.dll" "system" fn SelectObject(hdc : HDC, h : HGDIOBJ) -> HGDIOBJ);
    unsafe { SelectObject(hdc, h) }
}
#[inline]
pub unsafe fn SendMessageW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn SendMessageW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { SendMessageW(hwnd, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn SetCapture(hwnd: HWND) -> HWND {
    windows_core::link!("user32.dll" "system" fn SetCapture(hwnd : HWND) -> HWND);
    unsafe { SetCapture(hwnd) }
}
#[inline]
pub unsafe fn SetCursor(hcursor: Option<HCURSOR>) -> HCURSOR {
    windows_core::link!("user32.dll" "system" fn SetCursor(hcursor : HCURSOR) -> HCURSOR);
    unsafe { SetCursor(hcursor.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn SetDCBrushColor(hdc: HDC, color: COLORREF) -> COLORREF {
    windows_core::link!("gdi32.dll" "system" fn SetDCBrushColor(hdc : HDC, color : COLORREF) -> COLORREF);
    unsafe { SetDCBrushColor(hdc, color) }
}
#[inline]
pub unsafe fn SetProcessDpiAwarenessContext(value: DPI_AWARENESS_CONTEXT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetProcessDpiAwarenessContext(value : DPI_AWARENESS_CONTEXT) -> windows_core::BOOL);
    unsafe { SetProcessDpiAwarenessContext(value) }
}
#[inline]
pub unsafe fn SetProcessInformation(
    hprocess: HANDLE,
    processinformationclass: PROCESS_INFORMATION_CLASS,
    processinformation: *const core::ffi::c_void,
    processinformationsize: u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn SetProcessInformation(hprocess : HANDLE, processinformationclass : PROCESS_INFORMATION_CLASS, processinformation : *const core::ffi::c_void, processinformationsize : u32) -> windows_core::BOOL);
    unsafe {
        SetProcessInformation(
            hprocess,
            processinformationclass,
            processinformation,
            processinformationsize,
        )
    }
}
#[inline]
pub unsafe fn SetThreadDpiAwarenessContext(
    dpicontext: DPI_AWARENESS_CONTEXT,
) -> DPI_AWARENESS_CONTEXT {
    windows_core::link!("user32.dll" "system" fn SetThreadDpiAwarenessContext(dpicontext : DPI_AWARENESS_CONTEXT) -> DPI_AWARENESS_CONTEXT);
    unsafe { SetThreadDpiAwarenessContext(dpicontext) }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[inline]
pub unsafe fn SetWindowLongPtrW(hwnd: HWND, nindex: i32, dwnewlong: isize) -> isize {
    windows_core::link!("user32.dll" "system" fn SetWindowLongPtrW(hwnd : HWND, nindex : i32, dwnewlong : isize) -> isize);
    unsafe { SetWindowLongPtrW(hwnd, nindex, dwnewlong) }
}
#[cfg(target_pointer_width = "32")]
pub use SetWindowLongW as SetWindowLongPtrW;
#[inline]
pub unsafe fn SetWindowLongW(hwnd: HWND, nindex: i32, dwnewlong: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn SetWindowLongW(hwnd : HWND, nindex : i32, dwnewlong : i32) -> i32);
    unsafe { SetWindowLongW(hwnd, nindex, dwnewlong) }
}
#[inline]
pub unsafe fn SetWindowPos(
    hwnd: HWND,
    hwndinsertafter: Option<HWND>,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    uflags: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetWindowPos(hwnd : HWND, hwndinsertafter : HWND, x : i32, y : i32, cx : i32, cy : i32, uflags : u32) -> windows_core::BOOL);
    unsafe {
        SetWindowPos(
            hwnd,
            hwndinsertafter.unwrap_or(core::mem::zeroed()) as _,
            x,
            y,
            cx,
            cy,
            uflags,
        )
    }
}
#[inline]
pub unsafe fn SetWindowTextW<P1>(hwnd: HWND, lpstring: P1) -> windows_core::BOOL
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn SetWindowTextW(hwnd : HWND, lpstring : windows_core::PCWSTR) -> windows_core::BOOL);
    unsafe { SetWindowTextW(hwnd, lpstring.param().abi()) }
}
#[inline]
pub unsafe fn ShowWindow(hwnd: HWND, ncmdshow: i32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ShowWindow(hwnd : HWND, ncmdshow : i32) -> windows_core::BOOL);
    unsafe { ShowWindow(hwnd, ncmdshow) }
}
#[inline]
pub unsafe fn StartTraceW<P1>(
    traceid: *mut CONTROLTRACE_ID,
    instancename: P1,
    properties: *mut EVENT_TRACE_PROPERTIES,
) -> u32
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn StartTraceW(traceid : *mut CONTROLTRACE_ID, instancename : windows_core::PCWSTR, properties : *mut EVENT_TRACE_PROPERTIES) -> u32);
    unsafe { StartTraceW(traceid as _, instancename.param().abi(), properties as _) }
}
#[inline]
pub unsafe fn SystemParametersInfoW(
    uiaction: u32,
    uiparam: u32,
    pvparam: *mut core::ffi::c_void,
    fwinini: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SystemParametersInfoW(uiaction : u32, uiparam : u32, pvparam : *mut core::ffi::c_void, fwinini : u32) -> windows_core::BOOL);
    unsafe { SystemParametersInfoW(uiaction, uiparam, pvparam as _, fwinini) }
}
#[inline]
pub unsafe fn TdhGetEventInformation(
    event: *const EVENT_RECORD,
    tdhcontext: Option<&[TDH_CONTEXT]>,
    buffer: Option<*mut TRACE_EVENT_INFO>,
    buffersize: *mut u32,
) -> TDHSTATUS {
    windows_core::link!("tdh.dll" "system" fn TdhGetEventInformation(event : *const EVENT_RECORD, tdhcontextcount : u32, tdhcontext : *const TDH_CONTEXT, buffer : *mut TRACE_EVENT_INFO, buffersize : *mut u32) -> TDHSTATUS);
    unsafe {
        TdhGetEventInformation(
            event,
            tdhcontext.map_or(0, |slice| slice.len().try_into().unwrap()),
            tdhcontext.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            buffer.unwrap_or(core::mem::zeroed()) as _,
            buffersize as _,
        )
    }
}
#[inline]
pub unsafe fn ToUnicodeEx(
    wvirtkey: u32,
    wscancode: u32,
    lpkeystate: *const u8,
    pwszbuff: windows_core::PWSTR,
    cchbuff: i32,
    wflags: u32,
    dwhkl: Option<HKL>,
) -> i32 {
    windows_core::link!("user32.dll" "system" fn ToUnicodeEx(wvirtkey : u32, wscancode : u32, lpkeystate : *const u8, pwszbuff : windows_core::PWSTR, cchbuff : i32, wflags : u32, dwhkl : HKL) -> i32);
    unsafe {
        ToUnicodeEx(
            wvirtkey,
            wscancode,
            lpkeystate,
            pwszbuff,
            cchbuff,
            wflags,
            dwhkl.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn UpdateWindow(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn UpdateWindow(hwnd : HWND) -> windows_core::BOOL);
    unsafe { UpdateWindow(hwnd) }
}
#[inline]
pub unsafe fn ValidateRect(hwnd: Option<HWND>, lprect: Option<*const RECT>) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ValidateRect(hwnd : HWND, lprect : *const RECT) -> windows_core::BOOL);
    unsafe {
        ValidateRect(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            lprect.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn WaitForSingleObject(hhandle: HANDLE, dwmilliseconds: u32) -> u32 {
    windows_core::link!("kernel32.dll" "system" fn WaitForSingleObject(hhandle : HANDLE, dwmilliseconds : u32) -> u32);
    unsafe { WaitForSingleObject(hhandle, dwmilliseconds) }
}
#[inline]
pub unsafe fn WaitMessage() -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn WaitMessage() -> windows_core::BOOL);
    unsafe { WaitMessage() }
}
#[inline]
pub unsafe fn WindowFromPoint(point: POINT) -> HWND {
    windows_core::link!("user32.dll" "system" fn WindowFromPoint(point : POINT) -> HWND);
    unsafe { WindowFromPoint(point) }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ATOM(pub u16);
pub type CLSCTX = u32;
pub const CLSCTX_ACTIVATE_32_BIT_SERVER: CLSCTX = 262144;
pub const CLSCTX_ACTIVATE_64_BIT_SERVER: CLSCTX = 524288;
pub const CLSCTX_ACTIVATE_AAA_AS_IU: CLSCTX = 8388608;
pub const CLSCTX_ACTIVATE_ARM32_SERVER: CLSCTX = 33554432;
pub const CLSCTX_ACTIVATE_X86_SERVER: CLSCTX = 262144;
pub const CLSCTX_ALLOW_LOWER_TRUST_REGISTRATION: CLSCTX = 67108864;
pub const CLSCTX_APPCONTAINER: CLSCTX = 4194304;
pub const CLSCTX_DISABLE_AAA: CLSCTX = 32768;
pub const CLSCTX_DO_NOT_ELEVATE_SERVER: CLSCTX = 268435456;
pub const CLSCTX_ENABLE_AAA: CLSCTX = 65536;
pub const CLSCTX_ENABLE_CLOAKING: CLSCTX = 1048576;
pub const CLSCTX_ENABLE_CODE_DOWNLOAD: CLSCTX = 8192;
pub const CLSCTX_FROM_DEFAULT_CONTEXT: CLSCTX = 131072;
pub const CLSCTX_INPROC_HANDLER: CLSCTX = 2;
pub const CLSCTX_INPROC_HANDLER16: CLSCTX = 32;
pub const CLSCTX_INPROC_SERVER: CLSCTX = 1;
pub const CLSCTX_INPROC_SERVER16: CLSCTX = 8;
pub const CLSCTX_LOCAL_SERVER: CLSCTX = 4;
pub const CLSCTX_NO_CODE_DOWNLOAD: CLSCTX = 1024;
pub const CLSCTX_NO_CUSTOM_MARSHAL: CLSCTX = 4096;
pub const CLSCTX_NO_FAILURE_LOG: CLSCTX = 16384;
pub const CLSCTX_PS_DLL: CLSCTX = 2147483648;
pub const CLSCTX_REMOTE_SERVER: CLSCTX = 16;
pub const CLSCTX_RESERVED1: CLSCTX = 64;
pub const CLSCTX_RESERVED2: CLSCTX = 128;
pub const CLSCTX_RESERVED3: CLSCTX = 256;
pub const CLSCTX_RESERVED4: CLSCTX = 512;
pub const CLSCTX_RESERVED5: CLSCTX = 2048;
pub const CLSCTX_RESERVED6: CLSCTX = 16777216;
pub const CLSCTX_SERVER_MUST_BE_EQUAL_OR_GREATER_PRIVILEGE: CLSCTX = 134217728;
pub const CLSID_WICImagingFactory: windows_core::GUID =
    windows_core::GUID::from_u128(0xcacaf262_9370_4615_a13b_9f5539da4c0a);
pub const CM_LOCATE_DEVNODE_NORMAL: i32 = 0;
pub type COINIT = i32;
pub const COINIT_APARTMENTTHREADED: COINIT = 2;
pub const COINIT_DISABLE_OLE1DDE: COINIT = 4;
pub const COINIT_MULTITHREADED: COINIT = 0;
pub const COINIT_SPEED_OVER_MEMORY: COINIT = 8;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct COLORREF(pub u32);
pub type CONFIGRET = RETURN_TYPE;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CONTROLTRACE_ID(pub u64);
pub const CR_BUFFER_SMALL: i32 = 26;
pub const CR_SUCCESS: i32 = 0;
pub const D3D10_PRIMITIVE_TOPOLOGY_LINELIST: D3D_PRIMITIVE_TOPOLOGY = 2;
pub const D3D10_PRIMITIVE_TOPOLOGY_LINELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 10;
pub const D3D10_PRIMITIVE_TOPOLOGY_LINESTRIP: D3D_PRIMITIVE_TOPOLOGY = 3;
pub const D3D10_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 11;
pub const D3D10_PRIMITIVE_TOPOLOGY_POINTLIST: D3D_PRIMITIVE_TOPOLOGY = 1;
pub const D3D10_PRIMITIVE_TOPOLOGY_TRIANGLELIST: D3D_PRIMITIVE_TOPOLOGY = 4;
pub const D3D10_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 12;
pub const D3D10_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP: D3D_PRIMITIVE_TOPOLOGY = 5;
pub const D3D10_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 13;
pub const D3D10_PRIMITIVE_TOPOLOGY_UNDEFINED: D3D_PRIMITIVE_TOPOLOGY = 0;
pub const D3D11_PRIMITIVE_TOPOLOGY_10_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 42;
pub const D3D11_PRIMITIVE_TOPOLOGY_11_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 43;
pub const D3D11_PRIMITIVE_TOPOLOGY_12_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 44;
pub const D3D11_PRIMITIVE_TOPOLOGY_13_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 45;
pub const D3D11_PRIMITIVE_TOPOLOGY_14_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 46;
pub const D3D11_PRIMITIVE_TOPOLOGY_15_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 47;
pub const D3D11_PRIMITIVE_TOPOLOGY_16_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 48;
pub const D3D11_PRIMITIVE_TOPOLOGY_17_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 49;
pub const D3D11_PRIMITIVE_TOPOLOGY_18_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 50;
pub const D3D11_PRIMITIVE_TOPOLOGY_19_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 51;
pub const D3D11_PRIMITIVE_TOPOLOGY_1_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 33;
pub const D3D11_PRIMITIVE_TOPOLOGY_20_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 52;
pub const D3D11_PRIMITIVE_TOPOLOGY_21_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 53;
pub const D3D11_PRIMITIVE_TOPOLOGY_22_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 54;
pub const D3D11_PRIMITIVE_TOPOLOGY_23_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 55;
pub const D3D11_PRIMITIVE_TOPOLOGY_24_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 56;
pub const D3D11_PRIMITIVE_TOPOLOGY_25_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 57;
pub const D3D11_PRIMITIVE_TOPOLOGY_26_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 58;
pub const D3D11_PRIMITIVE_TOPOLOGY_27_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 59;
pub const D3D11_PRIMITIVE_TOPOLOGY_28_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 60;
pub const D3D11_PRIMITIVE_TOPOLOGY_29_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 61;
pub const D3D11_PRIMITIVE_TOPOLOGY_2_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 34;
pub const D3D11_PRIMITIVE_TOPOLOGY_30_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 62;
pub const D3D11_PRIMITIVE_TOPOLOGY_31_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 63;
pub const D3D11_PRIMITIVE_TOPOLOGY_32_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 64;
pub const D3D11_PRIMITIVE_TOPOLOGY_3_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 35;
pub const D3D11_PRIMITIVE_TOPOLOGY_4_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 36;
pub const D3D11_PRIMITIVE_TOPOLOGY_5_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 37;
pub const D3D11_PRIMITIVE_TOPOLOGY_6_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 38;
pub const D3D11_PRIMITIVE_TOPOLOGY_7_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 39;
pub const D3D11_PRIMITIVE_TOPOLOGY_8_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 40;
pub const D3D11_PRIMITIVE_TOPOLOGY_9_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 41;
pub const D3D11_PRIMITIVE_TOPOLOGY_LINELIST: D3D_PRIMITIVE_TOPOLOGY = 2;
pub const D3D11_PRIMITIVE_TOPOLOGY_LINELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 10;
pub const D3D11_PRIMITIVE_TOPOLOGY_LINESTRIP: D3D_PRIMITIVE_TOPOLOGY = 3;
pub const D3D11_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 11;
pub const D3D11_PRIMITIVE_TOPOLOGY_POINTLIST: D3D_PRIMITIVE_TOPOLOGY = 1;
pub const D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST: D3D_PRIMITIVE_TOPOLOGY = 4;
pub const D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 12;
pub const D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP: D3D_PRIMITIVE_TOPOLOGY = 5;
pub const D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 13;
pub const D3D11_PRIMITIVE_TOPOLOGY_UNDEFINED: D3D_PRIMITIVE_TOPOLOGY = 0;
pub type D3D12_BLEND = i32;
pub const D3D12_BLEND_ALPHA_FACTOR: D3D12_BLEND = 20;
pub const D3D12_BLEND_BLEND_FACTOR: D3D12_BLEND = 14;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D3D12_BLEND_DESC {
    pub AlphaToCoverageEnable: windows_core::BOOL,
    pub IndependentBlendEnable: windows_core::BOOL,
    pub RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC; 8],
}
impl Default for D3D12_BLEND_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const D3D12_BLEND_DEST_ALPHA: D3D12_BLEND = 7;
pub const D3D12_BLEND_DEST_COLOR: D3D12_BLEND = 9;
pub const D3D12_BLEND_INV_ALPHA_FACTOR: D3D12_BLEND = 21;
pub const D3D12_BLEND_INV_BLEND_FACTOR: D3D12_BLEND = 15;
pub const D3D12_BLEND_INV_DEST_ALPHA: D3D12_BLEND = 8;
pub const D3D12_BLEND_INV_DEST_COLOR: D3D12_BLEND = 10;
pub const D3D12_BLEND_INV_SRC1_ALPHA: D3D12_BLEND = 19;
pub const D3D12_BLEND_INV_SRC1_COLOR: D3D12_BLEND = 17;
pub const D3D12_BLEND_INV_SRC_ALPHA: D3D12_BLEND = 6;
pub const D3D12_BLEND_INV_SRC_COLOR: D3D12_BLEND = 4;
pub const D3D12_BLEND_ONE: D3D12_BLEND = 2;
pub type D3D12_BLEND_OP = i32;
pub const D3D12_BLEND_OP_ADD: D3D12_BLEND_OP = 1;
pub const D3D12_BLEND_OP_MAX: D3D12_BLEND_OP = 5;
pub const D3D12_BLEND_OP_MIN: D3D12_BLEND_OP = 4;
pub const D3D12_BLEND_OP_REV_SUBTRACT: D3D12_BLEND_OP = 3;
pub const D3D12_BLEND_OP_SUBTRACT: D3D12_BLEND_OP = 2;
pub const D3D12_BLEND_SRC1_ALPHA: D3D12_BLEND = 18;
pub const D3D12_BLEND_SRC1_COLOR: D3D12_BLEND = 16;
pub const D3D12_BLEND_SRC_ALPHA: D3D12_BLEND = 5;
pub const D3D12_BLEND_SRC_ALPHA_SAT: D3D12_BLEND = 11;
pub const D3D12_BLEND_SRC_COLOR: D3D12_BLEND = 3;
pub const D3D12_BLEND_ZERO: D3D12_BLEND = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_BOX {
    pub left: u32,
    pub top: u32,
    pub front: u32,
    pub right: u32,
    pub bottom: u32,
    pub back: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_BUFFER_RTV {
    pub FirstElement: u64,
    pub NumElements: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_BUFFER_SRV {
    pub FirstElement: u64,
    pub NumElements: u32,
    pub StructureByteStride: u32,
    pub Flags: D3D12_BUFFER_SRV_FLAGS,
}
pub type D3D12_BUFFER_SRV_FLAGS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_BUFFER_UAV {
    pub FirstElement: u64,
    pub NumElements: u32,
    pub StructureByteStride: u32,
    pub CounterOffsetInBytes: u64,
    pub Flags: D3D12_BUFFER_UAV_FLAGS,
}
pub type D3D12_BUFFER_UAV_FLAGS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_CACHED_PIPELINE_STATE {
    pub pCachedBlob: *const core::ffi::c_void,
    pub CachedBlobSizeInBytes: usize,
}
pub type D3D12_CLEAR_FLAGS = u32;
pub const D3D12_CLEAR_FLAG_DEPTH: D3D12_CLEAR_FLAGS = 1;
pub const D3D12_CLEAR_FLAG_STENCIL: D3D12_CLEAR_FLAGS = 2;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_CLEAR_VALUE {
    pub Format: DXGI_FORMAT,
    pub Anonymous: D3D12_CLEAR_VALUE_0,
}
impl Default for D3D12_CLEAR_VALUE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_CLEAR_VALUE_0 {
    pub Color: [f32; 4],
    pub DepthStencil: D3D12_DEPTH_STENCIL_VALUE,
}
impl Default for D3D12_CLEAR_VALUE_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type D3D12_COLOR_WRITE_ENABLE = i32;
pub const D3D12_COLOR_WRITE_ENABLE_ALL: D3D12_COLOR_WRITE_ENABLE = 15;
pub const D3D12_COLOR_WRITE_ENABLE_ALPHA: D3D12_COLOR_WRITE_ENABLE = 8;
pub const D3D12_COLOR_WRITE_ENABLE_BLUE: D3D12_COLOR_WRITE_ENABLE = 4;
pub const D3D12_COLOR_WRITE_ENABLE_GREEN: D3D12_COLOR_WRITE_ENABLE = 2;
pub const D3D12_COLOR_WRITE_ENABLE_RED: D3D12_COLOR_WRITE_ENABLE = 1;
pub type D3D12_COMMAND_LIST_TYPE = i32;
pub const D3D12_COMMAND_LIST_TYPE_BUNDLE: D3D12_COMMAND_LIST_TYPE = 1;
pub const D3D12_COMMAND_LIST_TYPE_COMPUTE: D3D12_COMMAND_LIST_TYPE = 2;
pub const D3D12_COMMAND_LIST_TYPE_COPY: D3D12_COMMAND_LIST_TYPE = 3;
pub const D3D12_COMMAND_LIST_TYPE_DIRECT: D3D12_COMMAND_LIST_TYPE = 0;
pub const D3D12_COMMAND_LIST_TYPE_NONE: D3D12_COMMAND_LIST_TYPE = -1;
pub const D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE: D3D12_COMMAND_LIST_TYPE = 4;
pub const D3D12_COMMAND_LIST_TYPE_VIDEO_ENCODE: D3D12_COMMAND_LIST_TYPE = 6;
pub const D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS: D3D12_COMMAND_LIST_TYPE = 5;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_COMMAND_QUEUE_DESC {
    pub Type: D3D12_COMMAND_LIST_TYPE,
    pub Priority: i32,
    pub Flags: D3D12_COMMAND_QUEUE_FLAGS,
    pub NodeMask: u32,
}
pub type D3D12_COMMAND_QUEUE_FLAGS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_COMMAND_SIGNATURE_DESC {
    pub ByteStride: u32,
    pub NumArgumentDescs: u32,
    pub pArgumentDescs: *const D3D12_INDIRECT_ARGUMENT_DESC,
    pub NodeMask: u32,
}
pub type D3D12_COMPARISON_FUNC = i32;
pub const D3D12_COMPARISON_FUNC_ALWAYS: D3D12_COMPARISON_FUNC = 8;
pub const D3D12_COMPARISON_FUNC_EQUAL: D3D12_COMPARISON_FUNC = 3;
pub const D3D12_COMPARISON_FUNC_GREATER: D3D12_COMPARISON_FUNC = 5;
pub const D3D12_COMPARISON_FUNC_GREATER_EQUAL: D3D12_COMPARISON_FUNC = 7;
pub const D3D12_COMPARISON_FUNC_LESS: D3D12_COMPARISON_FUNC = 2;
pub const D3D12_COMPARISON_FUNC_LESS_EQUAL: D3D12_COMPARISON_FUNC = 4;
pub const D3D12_COMPARISON_FUNC_NEVER: D3D12_COMPARISON_FUNC = 1;
pub const D3D12_COMPARISON_FUNC_NONE: D3D12_COMPARISON_FUNC = 0;
pub const D3D12_COMPARISON_FUNC_NOT_EQUAL: D3D12_COMPARISON_FUNC = 6;
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct D3D12_COMPUTE_PIPELINE_STATE_DESC {
    pub pRootSignature: core::mem::ManuallyDrop<Option<ID3D12RootSignature>>,
    pub CS: D3D12_SHADER_BYTECODE,
    pub NodeMask: u32,
    pub CachedPSO: D3D12_CACHED_PIPELINE_STATE,
    pub Flags: D3D12_PIPELINE_STATE_FLAGS,
}
pub type D3D12_CONSERVATIVE_RASTERIZATION_MODE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_CONSTANT_BUFFER_VIEW_DESC {
    pub BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS,
    pub SizeInBytes: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_CPU_DESCRIPTOR_HANDLE {
    pub ptr: usize,
}
pub type D3D12_CPU_PAGE_PROPERTY = i32;
pub type D3D12_CULL_MODE = i32;
pub const D3D12_CULL_MODE_BACK: D3D12_CULL_MODE = 3;
pub const D3D12_CULL_MODE_FRONT: D3D12_CULL_MODE = 2;
pub const D3D12_CULL_MODE_NONE: D3D12_CULL_MODE = 1;
pub const D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING: i32 = 5768;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_DEPTH_STENCILOP_DESC {
    pub StencilFailOp: D3D12_STENCIL_OP,
    pub StencilDepthFailOp: D3D12_STENCIL_OP,
    pub StencilPassOp: D3D12_STENCIL_OP,
    pub StencilFunc: D3D12_COMPARISON_FUNC,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_DEPTH_STENCIL_DESC {
    pub DepthEnable: windows_core::BOOL,
    pub DepthWriteMask: D3D12_DEPTH_WRITE_MASK,
    pub DepthFunc: D3D12_COMPARISON_FUNC,
    pub StencilEnable: windows_core::BOOL,
    pub StencilReadMask: u8,
    pub StencilWriteMask: u8,
    pub FrontFace: D3D12_DEPTH_STENCILOP_DESC,
    pub BackFace: D3D12_DEPTH_STENCILOP_DESC,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_DEPTH_STENCIL_VALUE {
    pub Depth: f32,
    pub Stencil: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_DEPTH_STENCIL_VIEW_DESC {
    pub Format: DXGI_FORMAT,
    pub ViewDimension: D3D12_DSV_DIMENSION,
    pub Flags: D3D12_DSV_FLAGS,
    pub Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0,
}
impl Default for D3D12_DEPTH_STENCIL_VIEW_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
    pub Texture1D: D3D12_TEX1D_DSV,
    pub Texture1DArray: D3D12_TEX1D_ARRAY_DSV,
    pub Texture2D: D3D12_TEX2D_DSV,
    pub Texture2DArray: D3D12_TEX2D_ARRAY_DSV,
    pub Texture2DMS: D3D12_TEX2DMS_DSV,
    pub Texture2DMSArray: D3D12_TEX2DMS_ARRAY_DSV,
}
impl Default for D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type D3D12_DEPTH_WRITE_MASK = i32;
pub const D3D12_DEPTH_WRITE_MASK_ALL: D3D12_DEPTH_WRITE_MASK = 1;
pub const D3D12_DEPTH_WRITE_MASK_ZERO: D3D12_DEPTH_WRITE_MASK = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_DESCRIPTOR_HEAP_DESC {
    pub Type: D3D12_DESCRIPTOR_HEAP_TYPE,
    pub NumDescriptors: u32,
    pub Flags: D3D12_DESCRIPTOR_HEAP_FLAGS,
    pub NodeMask: u32,
}
pub type D3D12_DESCRIPTOR_HEAP_FLAGS = u32;
pub const D3D12_DESCRIPTOR_HEAP_FLAG_NONE: D3D12_DESCRIPTOR_HEAP_FLAGS = 0;
pub const D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE: D3D12_DESCRIPTOR_HEAP_FLAGS = 1;
pub type D3D12_DESCRIPTOR_HEAP_TYPE = i32;
pub const D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV: D3D12_DESCRIPTOR_HEAP_TYPE = 0;
pub const D3D12_DESCRIPTOR_HEAP_TYPE_DSV: D3D12_DESCRIPTOR_HEAP_TYPE = 3;
pub const D3D12_DESCRIPTOR_HEAP_TYPE_NUM_TYPES: D3D12_DESCRIPTOR_HEAP_TYPE = 4;
pub const D3D12_DESCRIPTOR_HEAP_TYPE_RTV: D3D12_DESCRIPTOR_HEAP_TYPE = 2;
pub const D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER: D3D12_DESCRIPTOR_HEAP_TYPE = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_DISCARD_REGION {
    pub NumRects: u32,
    pub pRects: *const D3D12_RECT,
    pub FirstSubresource: u32,
    pub NumSubresources: u32,
}
pub type D3D12_DSV_DIMENSION = i32;
pub const D3D12_DSV_DIMENSION_TEXTURE1D: D3D12_DSV_DIMENSION = 1;
pub const D3D12_DSV_DIMENSION_TEXTURE1DARRAY: D3D12_DSV_DIMENSION = 2;
pub const D3D12_DSV_DIMENSION_TEXTURE2D: D3D12_DSV_DIMENSION = 3;
pub const D3D12_DSV_DIMENSION_TEXTURE2DARRAY: D3D12_DSV_DIMENSION = 4;
pub const D3D12_DSV_DIMENSION_TEXTURE2DMS: D3D12_DSV_DIMENSION = 5;
pub const D3D12_DSV_DIMENSION_TEXTURE2DMSARRAY: D3D12_DSV_DIMENSION = 6;
pub const D3D12_DSV_DIMENSION_UNKNOWN: D3D12_DSV_DIMENSION = 0;
pub type D3D12_DSV_FLAGS = u32;
pub const D3D12_DSV_FLAG_NONE: D3D12_DSV_FLAGS = 0;
pub const D3D12_DSV_FLAG_READ_ONLY_DEPTH: D3D12_DSV_FLAGS = 1;
pub const D3D12_DSV_FLAG_READ_ONLY_STENCIL: D3D12_DSV_FLAGS = 2;
pub type D3D12_FEATURE = i32;
pub type D3D12_FENCE_FLAGS = u32;
pub const D3D12_FENCE_FLAG_NONE: D3D12_FENCE_FLAGS = 0;
pub const D3D12_FENCE_FLAG_NON_MONITORED: D3D12_FENCE_FLAGS = 4;
pub const D3D12_FENCE_FLAG_SHARED: D3D12_FENCE_FLAGS = 1;
pub const D3D12_FENCE_FLAG_SHARED_CROSS_ADAPTER: D3D12_FENCE_FLAGS = 2;
pub type D3D12_FILL_MODE = i32;
pub const D3D12_FILL_MODE_SOLID: D3D12_FILL_MODE = 3;
pub const D3D12_FILL_MODE_WIREFRAME: D3D12_FILL_MODE = 2;
pub type D3D12_FILTER = i32;
pub const D3D12_FILTER_ANISOTROPIC: D3D12_FILTER = 85;
pub const D3D12_FILTER_COMPARISON_ANISOTROPIC: D3D12_FILTER = 213;
pub const D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT: D3D12_FILTER = 144;
pub const D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 145;
pub const D3D12_FILTER_COMPARISON_MIN_MAG_ANISOTROPIC_MIP_POINT: D3D12_FILTER = 212;
pub const D3D12_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 148;
pub const D3D12_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR: D3D12_FILTER = 149;
pub const D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT: D3D12_FILTER = 128;
pub const D3D12_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 129;
pub const D3D12_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 132;
pub const D3D12_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR: D3D12_FILTER = 133;
pub const D3D12_FILTER_MAXIMUM_ANISOTROPIC: D3D12_FILTER = 469;
pub const D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_MIP_POINT: D3D12_FILTER = 400;
pub const D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 401;
pub const D3D12_FILTER_MAXIMUM_MIN_MAG_ANISOTROPIC_MIP_POINT: D3D12_FILTER = 468;
pub const D3D12_FILTER_MAXIMUM_MIN_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 404;
pub const D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_LINEAR: D3D12_FILTER = 405;
pub const D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT: D3D12_FILTER = 384;
pub const D3D12_FILTER_MAXIMUM_MIN_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 385;
pub const D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 388;
pub const D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_MIP_LINEAR: D3D12_FILTER = 389;
pub const D3D12_FILTER_MINIMUM_ANISOTROPIC: D3D12_FILTER = 341;
pub const D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_MIP_POINT: D3D12_FILTER = 272;
pub const D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 273;
pub const D3D12_FILTER_MINIMUM_MIN_MAG_ANISOTROPIC_MIP_POINT: D3D12_FILTER = 340;
pub const D3D12_FILTER_MINIMUM_MIN_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 276;
pub const D3D12_FILTER_MINIMUM_MIN_MAG_MIP_LINEAR: D3D12_FILTER = 277;
pub const D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT: D3D12_FILTER = 256;
pub const D3D12_FILTER_MINIMUM_MIN_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 257;
pub const D3D12_FILTER_MINIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 260;
pub const D3D12_FILTER_MINIMUM_MIN_POINT_MAG_MIP_LINEAR: D3D12_FILTER = 261;
pub const D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT: D3D12_FILTER = 16;
pub const D3D12_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 17;
pub const D3D12_FILTER_MIN_MAG_ANISOTROPIC_MIP_POINT: D3D12_FILTER = 84;
pub const D3D12_FILTER_MIN_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 20;
pub const D3D12_FILTER_MIN_MAG_MIP_LINEAR: D3D12_FILTER = 21;
pub const D3D12_FILTER_MIN_MAG_MIP_POINT: D3D12_FILTER = 0;
pub const D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR: D3D12_FILTER = 1;
pub const D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D12_FILTER = 4;
pub const D3D12_FILTER_MIN_POINT_MAG_MIP_LINEAR: D3D12_FILTER = 5;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_GPU_DESCRIPTOR_HANDLE {
    pub ptr: u64,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct D3D12_GPU_VIRTUAL_ADDRESS(pub u64);
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct D3D12_GRAPHICS_PIPELINE_STATE_DESC {
    pub pRootSignature: core::mem::ManuallyDrop<Option<ID3D12RootSignature>>,
    pub VS: D3D12_SHADER_BYTECODE,
    pub PS: D3D12_SHADER_BYTECODE,
    pub DS: D3D12_SHADER_BYTECODE,
    pub HS: D3D12_SHADER_BYTECODE,
    pub GS: D3D12_SHADER_BYTECODE,
    pub StreamOutput: D3D12_STREAM_OUTPUT_DESC,
    pub BlendState: D3D12_BLEND_DESC,
    pub SampleMask: u32,
    pub RasterizerState: D3D12_RASTERIZER_DESC,
    pub DepthStencilState: D3D12_DEPTH_STENCIL_DESC,
    pub InputLayout: D3D12_INPUT_LAYOUT_DESC,
    pub IBStripCutValue: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE,
    pub PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE,
    pub NumRenderTargets: u32,
    pub RTVFormats: [DXGI_FORMAT; 8],
    pub DSVFormat: DXGI_FORMAT,
    pub SampleDesc: DXGI_SAMPLE_DESC,
    pub NodeMask: u32,
    pub CachedPSO: D3D12_CACHED_PIPELINE_STATE,
    pub Flags: D3D12_PIPELINE_STATE_FLAGS,
}
impl Default for D3D12_GRAPHICS_PIPELINE_STATE_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_HEAP_DESC {
    pub SizeInBytes: u64,
    pub Properties: D3D12_HEAP_PROPERTIES,
    pub Alignment: u64,
    pub Flags: D3D12_HEAP_FLAGS,
}
pub type D3D12_HEAP_FLAGS = u32;
pub const D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES: D3D12_HEAP_FLAGS = 0;
pub const D3D12_HEAP_FLAG_ALLOW_DISPLAY: D3D12_HEAP_FLAGS = 8;
pub const D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS: D3D12_HEAP_FLAGS = 192;
pub const D3D12_HEAP_FLAG_ALLOW_ONLY_NON_RT_DS_TEXTURES: D3D12_HEAP_FLAGS = 68;
pub const D3D12_HEAP_FLAG_ALLOW_ONLY_RT_DS_TEXTURES: D3D12_HEAP_FLAGS = 132;
pub const D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS: D3D12_HEAP_FLAGS = 1024;
pub const D3D12_HEAP_FLAG_ALLOW_WRITE_WATCH: D3D12_HEAP_FLAGS = 512;
pub const D3D12_HEAP_FLAG_CREATE_NOT_RESIDENT: D3D12_HEAP_FLAGS = 2048;
pub const D3D12_HEAP_FLAG_CREATE_NOT_ZEROED: D3D12_HEAP_FLAGS = 4096;
pub const D3D12_HEAP_FLAG_DENY_BUFFERS: D3D12_HEAP_FLAGS = 4;
pub const D3D12_HEAP_FLAG_DENY_NON_RT_DS_TEXTURES: D3D12_HEAP_FLAGS = 128;
pub const D3D12_HEAP_FLAG_DENY_RT_DS_TEXTURES: D3D12_HEAP_FLAGS = 64;
pub const D3D12_HEAP_FLAG_HARDWARE_PROTECTED: D3D12_HEAP_FLAGS = 256;
pub const D3D12_HEAP_FLAG_NONE: D3D12_HEAP_FLAGS = 0;
pub const D3D12_HEAP_FLAG_SHARED: D3D12_HEAP_FLAGS = 1;
pub const D3D12_HEAP_FLAG_SHARED_CROSS_ADAPTER: D3D12_HEAP_FLAGS = 32;
pub const D3D12_HEAP_FLAG_TOOLS_USE_MANUAL_WRITE_TRACKING: D3D12_HEAP_FLAGS = 8192;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_HEAP_PROPERTIES {
    pub Type: D3D12_HEAP_TYPE,
    pub CPUPageProperty: D3D12_CPU_PAGE_PROPERTY,
    pub MemoryPoolPreference: D3D12_MEMORY_POOL,
    pub CreationNodeMask: u32,
    pub VisibleNodeMask: u32,
}
pub type D3D12_HEAP_TYPE = i32;
pub const D3D12_HEAP_TYPE_CUSTOM: D3D12_HEAP_TYPE = 4;
pub const D3D12_HEAP_TYPE_DEFAULT: D3D12_HEAP_TYPE = 1;
pub const D3D12_HEAP_TYPE_GPU_UPLOAD: D3D12_HEAP_TYPE = 5;
pub const D3D12_HEAP_TYPE_READBACK: D3D12_HEAP_TYPE = 3;
pub const D3D12_HEAP_TYPE_UPLOAD: D3D12_HEAP_TYPE = 2;
pub type D3D12_INDEX_BUFFER_STRIP_CUT_VALUE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDEX_BUFFER_VIEW {
    pub BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS,
    pub SizeInBytes: u32,
    pub Format: DXGI_FORMAT,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC {
    pub Type: D3D12_INDIRECT_ARGUMENT_TYPE,
    pub Anonymous: D3D12_INDIRECT_ARGUMENT_DESC_0,
}
impl Default for D3D12_INDIRECT_ARGUMENT_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_INDIRECT_ARGUMENT_DESC_0 {
    pub VertexBuffer: D3D12_INDIRECT_ARGUMENT_DESC_0_0,
    pub Constant: D3D12_INDIRECT_ARGUMENT_DESC_0_1,
    pub ConstantBufferView: D3D12_INDIRECT_ARGUMENT_DESC_0_2,
    pub ShaderResourceView: D3D12_INDIRECT_ARGUMENT_DESC_0_3,
    pub UnorderedAccessView: D3D12_INDIRECT_ARGUMENT_DESC_0_4,
    pub IncrementingConstant: D3D12_INDIRECT_ARGUMENT_DESC_0_5,
}
impl Default for D3D12_INDIRECT_ARGUMENT_DESC_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_0 {
    pub Slot: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_1 {
    pub RootParameterIndex: u32,
    pub DestOffsetIn32BitValues: u32,
    pub Num32BitValuesToSet: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_2 {
    pub RootParameterIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_3 {
    pub RootParameterIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_4 {
    pub RootParameterIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INDIRECT_ARGUMENT_DESC_0_5 {
    pub RootParameterIndex: u32,
    pub DestOffsetIn32BitValues: u32,
}
pub type D3D12_INDIRECT_ARGUMENT_TYPE = i32;
pub type D3D12_INPUT_CLASSIFICATION = i32;
pub const D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA: D3D12_INPUT_CLASSIFICATION = 1;
pub const D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA: D3D12_INPUT_CLASSIFICATION = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INPUT_ELEMENT_DESC {
    pub SemanticName: windows_core::PCSTR,
    pub SemanticIndex: u32,
    pub Format: DXGI_FORMAT,
    pub InputSlot: u32,
    pub AlignedByteOffset: u32,
    pub InputSlotClass: D3D12_INPUT_CLASSIFICATION,
    pub InstanceDataStepRate: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_INPUT_LAYOUT_DESC {
    pub pInputElementDescs: *const D3D12_INPUT_ELEMENT_DESC,
    pub NumElements: u32,
}
pub type D3D12_LOGIC_OP = i32;
pub type D3D12_MEMORY_POOL = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_MESSAGE {
    pub Category: D3D12_MESSAGE_CATEGORY,
    pub Severity: D3D12_MESSAGE_SEVERITY,
    pub ID: D3D12_MESSAGE_ID,
    pub pDescription: *const i8,
    pub DescriptionByteLength: usize,
}
pub type D3D12_MESSAGE_CATEGORY = i32;
pub type D3D12_MESSAGE_ID = i32;
pub type D3D12_MESSAGE_SEVERITY = i32;
pub const D3D12_MESSAGE_SEVERITY_CORRUPTION: D3D12_MESSAGE_SEVERITY = 0;
pub const D3D12_MESSAGE_SEVERITY_ERROR: D3D12_MESSAGE_SEVERITY = 1;
pub const D3D12_MESSAGE_SEVERITY_INFO: D3D12_MESSAGE_SEVERITY = 3;
pub const D3D12_MESSAGE_SEVERITY_MESSAGE: D3D12_MESSAGE_SEVERITY = 4;
pub const D3D12_MESSAGE_SEVERITY_WARNING: D3D12_MESSAGE_SEVERITY = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_PACKED_MIP_INFO {
    pub NumStandardMips: u8,
    pub NumPackedMips: u8,
    pub NumTilesForPackedMips: u32,
    pub StartTileIndexInOverallResource: u32,
}
pub type D3D12_PIPELINE_STATE_FLAGS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
    pub Offset: u64,
    pub Footprint: D3D12_SUBRESOURCE_FOOTPRINT,
}
pub type D3D12_PREDICATION_OP = i32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct D3D12_PRIMITIVE_TOPOLOGY(pub D3D_PRIMITIVE_TOPOLOGY);
pub type D3D12_PRIMITIVE_TOPOLOGY_TYPE = i32;
pub const D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE: D3D12_PRIMITIVE_TOPOLOGY_TYPE = 2;
pub const D3D12_PRIMITIVE_TOPOLOGY_TYPE_PATCH: D3D12_PRIMITIVE_TOPOLOGY_TYPE = 4;
pub const D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT: D3D12_PRIMITIVE_TOPOLOGY_TYPE = 1;
pub const D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE: D3D12_PRIMITIVE_TOPOLOGY_TYPE = 3;
pub const D3D12_PRIMITIVE_TOPOLOGY_TYPE_UNDEFINED: D3D12_PRIMITIVE_TOPOLOGY_TYPE = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_QUERY_HEAP_DESC {
    pub Type: D3D12_QUERY_HEAP_TYPE,
    pub Count: u32,
    pub NodeMask: u32,
}
pub type D3D12_QUERY_HEAP_TYPE = i32;
pub const D3D12_QUERY_HEAP_TYPE_COPY_QUEUE_TIMESTAMP: D3D12_QUERY_HEAP_TYPE = 5;
pub const D3D12_QUERY_HEAP_TYPE_OCCLUSION: D3D12_QUERY_HEAP_TYPE = 0;
pub const D3D12_QUERY_HEAP_TYPE_PIPELINE_STATISTICS: D3D12_QUERY_HEAP_TYPE = 2;
pub const D3D12_QUERY_HEAP_TYPE_PIPELINE_STATISTICS1: D3D12_QUERY_HEAP_TYPE = 7;
pub const D3D12_QUERY_HEAP_TYPE_SO_STATISTICS: D3D12_QUERY_HEAP_TYPE = 3;
pub const D3D12_QUERY_HEAP_TYPE_TIMESTAMP: D3D12_QUERY_HEAP_TYPE = 1;
pub const D3D12_QUERY_HEAP_TYPE_VIDEO_DECODE_STATISTICS: D3D12_QUERY_HEAP_TYPE = 4;
pub type D3D12_QUERY_TYPE = i32;
pub const D3D12_QUERY_TYPE_BINARY_OCCLUSION: D3D12_QUERY_TYPE = 1;
pub const D3D12_QUERY_TYPE_OCCLUSION: D3D12_QUERY_TYPE = 0;
pub const D3D12_QUERY_TYPE_PIPELINE_STATISTICS: D3D12_QUERY_TYPE = 3;
pub const D3D12_QUERY_TYPE_PIPELINE_STATISTICS1: D3D12_QUERY_TYPE = 10;
pub const D3D12_QUERY_TYPE_SO_STATISTICS_STREAM0: D3D12_QUERY_TYPE = 4;
pub const D3D12_QUERY_TYPE_SO_STATISTICS_STREAM1: D3D12_QUERY_TYPE = 5;
pub const D3D12_QUERY_TYPE_SO_STATISTICS_STREAM2: D3D12_QUERY_TYPE = 6;
pub const D3D12_QUERY_TYPE_SO_STATISTICS_STREAM3: D3D12_QUERY_TYPE = 7;
pub const D3D12_QUERY_TYPE_TIMESTAMP: D3D12_QUERY_TYPE = 2;
pub const D3D12_QUERY_TYPE_VIDEO_DECODE_STATISTICS: D3D12_QUERY_TYPE = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RANGE {
    pub Begin: usize,
    pub End: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_RASTERIZER_DESC {
    pub FillMode: D3D12_FILL_MODE,
    pub CullMode: D3D12_CULL_MODE,
    pub FrontCounterClockwise: windows_core::BOOL,
    pub DepthBias: i32,
    pub DepthBiasClamp: f32,
    pub SlopeScaledDepthBias: f32,
    pub DepthClipEnable: windows_core::BOOL,
    pub MultisampleEnable: windows_core::BOOL,
    pub AntialiasedLineEnable: windows_core::BOOL,
    pub ForcedSampleCount: u32,
    pub ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RAYTRACING_ACCELERATION_STRUCTURE_SRV {
    pub Location: D3D12_GPU_VIRTUAL_ADDRESS,
}
pub type D3D12_RECT = RECT;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RENDER_TARGET_BLEND_DESC {
    pub BlendEnable: windows_core::BOOL,
    pub LogicOpEnable: windows_core::BOOL,
    pub SrcBlend: D3D12_BLEND,
    pub DestBlend: D3D12_BLEND,
    pub BlendOp: D3D12_BLEND_OP,
    pub SrcBlendAlpha: D3D12_BLEND,
    pub DestBlendAlpha: D3D12_BLEND,
    pub BlendOpAlpha: D3D12_BLEND_OP,
    pub LogicOp: D3D12_LOGIC_OP,
    pub RenderTargetWriteMask: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_RENDER_TARGET_VIEW_DESC {
    pub Format: DXGI_FORMAT,
    pub ViewDimension: D3D12_RTV_DIMENSION,
    pub Anonymous: D3D12_RENDER_TARGET_VIEW_DESC_0,
}
impl Default for D3D12_RENDER_TARGET_VIEW_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_RENDER_TARGET_VIEW_DESC_0 {
    pub Buffer: D3D12_BUFFER_RTV,
    pub Texture1D: D3D12_TEX1D_RTV,
    pub Texture1DArray: D3D12_TEX1D_ARRAY_RTV,
    pub Texture2D: D3D12_TEX2D_RTV,
    pub Texture2DArray: D3D12_TEX2D_ARRAY_RTV,
    pub Texture2DMS: D3D12_TEX2DMS_RTV,
    pub Texture2DMSArray: D3D12_TEX2DMS_ARRAY_RTV,
    pub Texture3D: D3D12_TEX3D_RTV,
}
impl Default for D3D12_RENDER_TARGET_VIEW_DESC_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RESOURCE_ALIASING_BARRIER {
    pub pResourceBefore: core::mem::ManuallyDrop<Option<ID3D12Resource>>,
    pub pResourceAfter: core::mem::ManuallyDrop<Option<ID3D12Resource>>,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RESOURCE_ALLOCATION_INFO {
    pub SizeInBytes: u64,
    pub Alignment: u64,
}
#[repr(C)]
pub struct D3D12_RESOURCE_BARRIER {
    pub Type: D3D12_RESOURCE_BARRIER_TYPE,
    pub Flags: D3D12_RESOURCE_BARRIER_FLAGS,
    pub Anonymous: D3D12_RESOURCE_BARRIER_0,
}
impl Clone for D3D12_RESOURCE_BARRIER {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for D3D12_RESOURCE_BARRIER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union D3D12_RESOURCE_BARRIER_0 {
    pub Transition: core::mem::ManuallyDrop<D3D12_RESOURCE_TRANSITION_BARRIER>,
    pub Aliasing: core::mem::ManuallyDrop<D3D12_RESOURCE_ALIASING_BARRIER>,
    pub UAV: core::mem::ManuallyDrop<D3D12_RESOURCE_UAV_BARRIER>,
}
impl Clone for D3D12_RESOURCE_BARRIER_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for D3D12_RESOURCE_BARRIER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES: u32 = 4294967295;
pub type D3D12_RESOURCE_BARRIER_FLAGS = u32;
pub const D3D12_RESOURCE_BARRIER_FLAG_BEGIN_ONLY: D3D12_RESOURCE_BARRIER_FLAGS = 1;
pub const D3D12_RESOURCE_BARRIER_FLAG_END_ONLY: D3D12_RESOURCE_BARRIER_FLAGS = 2;
pub const D3D12_RESOURCE_BARRIER_FLAG_NONE: D3D12_RESOURCE_BARRIER_FLAGS = 0;
pub type D3D12_RESOURCE_BARRIER_TYPE = i32;
pub const D3D12_RESOURCE_BARRIER_TYPE_ALIASING: D3D12_RESOURCE_BARRIER_TYPE = 1;
pub const D3D12_RESOURCE_BARRIER_TYPE_TRANSITION: D3D12_RESOURCE_BARRIER_TYPE = 0;
pub const D3D12_RESOURCE_BARRIER_TYPE_UAV: D3D12_RESOURCE_BARRIER_TYPE = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RESOURCE_DESC {
    pub Dimension: D3D12_RESOURCE_DIMENSION,
    pub Alignment: u64,
    pub Width: u64,
    pub Height: u32,
    pub DepthOrArraySize: u16,
    pub MipLevels: u16,
    pub Format: DXGI_FORMAT,
    pub SampleDesc: DXGI_SAMPLE_DESC,
    pub Layout: D3D12_TEXTURE_LAYOUT,
    pub Flags: D3D12_RESOURCE_FLAGS,
}
pub type D3D12_RESOURCE_DIMENSION = i32;
pub const D3D12_RESOURCE_DIMENSION_BUFFER: D3D12_RESOURCE_DIMENSION = 1;
pub const D3D12_RESOURCE_DIMENSION_TEXTURE1D: D3D12_RESOURCE_DIMENSION = 2;
pub const D3D12_RESOURCE_DIMENSION_TEXTURE2D: D3D12_RESOURCE_DIMENSION = 3;
pub const D3D12_RESOURCE_DIMENSION_TEXTURE3D: D3D12_RESOURCE_DIMENSION = 4;
pub const D3D12_RESOURCE_DIMENSION_UNKNOWN: D3D12_RESOURCE_DIMENSION = 0;
pub type D3D12_RESOURCE_FLAGS = u32;
pub const D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER: D3D12_RESOURCE_FLAGS = 16;
pub const D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL: D3D12_RESOURCE_FLAGS = 2;
pub const D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET: D3D12_RESOURCE_FLAGS = 1;
pub const D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS: D3D12_RESOURCE_FLAGS = 32;
pub const D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS: D3D12_RESOURCE_FLAGS = 4;
pub const D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE: D3D12_RESOURCE_FLAGS = 8;
pub const D3D12_RESOURCE_FLAG_NONE: D3D12_RESOURCE_FLAGS = 0;
pub const D3D12_RESOURCE_FLAG_RAYTRACING_ACCELERATION_STRUCTURE: D3D12_RESOURCE_FLAGS = 256;
pub const D3D12_RESOURCE_FLAG_USE_TIGHT_ALIGNMENT: D3D12_RESOURCE_FLAGS = 1024;
pub const D3D12_RESOURCE_FLAG_VIDEO_DECODE_REFERENCE_ONLY: D3D12_RESOURCE_FLAGS = 64;
pub const D3D12_RESOURCE_FLAG_VIDEO_ENCODE_REFERENCE_ONLY: D3D12_RESOURCE_FLAGS = 128;
pub type D3D12_RESOURCE_STATES = u32;
pub const D3D12_RESOURCE_STATE_ALL_SHADER_RESOURCE: D3D12_RESOURCE_STATES = 192;
pub const D3D12_RESOURCE_STATE_COMMON: D3D12_RESOURCE_STATES = 0;
pub const D3D12_RESOURCE_STATE_COPY_DEST: D3D12_RESOURCE_STATES = 1024;
pub const D3D12_RESOURCE_STATE_COPY_SOURCE: D3D12_RESOURCE_STATES = 2048;
pub const D3D12_RESOURCE_STATE_DEPTH_READ: D3D12_RESOURCE_STATES = 32;
pub const D3D12_RESOURCE_STATE_DEPTH_WRITE: D3D12_RESOURCE_STATES = 16;
pub const D3D12_RESOURCE_STATE_GENERIC_READ: D3D12_RESOURCE_STATES = 2755;
pub const D3D12_RESOURCE_STATE_INDEX_BUFFER: D3D12_RESOURCE_STATES = 2;
pub const D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT: D3D12_RESOURCE_STATES = 512;
pub const D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE: D3D12_RESOURCE_STATES = 64;
pub const D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE: D3D12_RESOURCE_STATES = 128;
pub const D3D12_RESOURCE_STATE_PREDICATION: D3D12_RESOURCE_STATES = 512;
pub const D3D12_RESOURCE_STATE_PRESENT: D3D12_RESOURCE_STATES = 0;
pub const D3D12_RESOURCE_STATE_RAYTRACING_ACCELERATION_STRUCTURE: D3D12_RESOURCE_STATES = 4194304;
pub const D3D12_RESOURCE_STATE_RENDER_TARGET: D3D12_RESOURCE_STATES = 4;
pub const D3D12_RESOURCE_STATE_RESERVED_INTERNAL_100000: D3D12_RESOURCE_STATES = 1048576;
pub const D3D12_RESOURCE_STATE_RESERVED_INTERNAL_4000: D3D12_RESOURCE_STATES = 16384;
pub const D3D12_RESOURCE_STATE_RESERVED_INTERNAL_40000000: D3D12_RESOURCE_STATES = 1073741824;
pub const D3D12_RESOURCE_STATE_RESERVED_INTERNAL_8000: D3D12_RESOURCE_STATES = 32768;
pub const D3D12_RESOURCE_STATE_RESERVED_INTERNAL_80000000: D3D12_RESOURCE_STATES = 2147483648;
pub const D3D12_RESOURCE_STATE_RESOLVE_DEST: D3D12_RESOURCE_STATES = 4096;
pub const D3D12_RESOURCE_STATE_RESOLVE_SOURCE: D3D12_RESOURCE_STATES = 8192;
pub const D3D12_RESOURCE_STATE_SHADING_RATE_SOURCE: D3D12_RESOURCE_STATES = 16777216;
pub const D3D12_RESOURCE_STATE_STREAM_OUT: D3D12_RESOURCE_STATES = 256;
pub const D3D12_RESOURCE_STATE_UNORDERED_ACCESS: D3D12_RESOURCE_STATES = 8;
pub const D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER: D3D12_RESOURCE_STATES = 1;
pub const D3D12_RESOURCE_STATE_VIDEO_DECODE_READ: D3D12_RESOURCE_STATES = 65536;
pub const D3D12_RESOURCE_STATE_VIDEO_DECODE_WRITE: D3D12_RESOURCE_STATES = 131072;
pub const D3D12_RESOURCE_STATE_VIDEO_ENCODE_READ: D3D12_RESOURCE_STATES = 2097152;
pub const D3D12_RESOURCE_STATE_VIDEO_ENCODE_WRITE: D3D12_RESOURCE_STATES = 8388608;
pub const D3D12_RESOURCE_STATE_VIDEO_PROCESS_READ: D3D12_RESOURCE_STATES = 262144;
pub const D3D12_RESOURCE_STATE_VIDEO_PROCESS_WRITE: D3D12_RESOURCE_STATES = 524288;
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RESOURCE_TRANSITION_BARRIER {
    pub pResource: core::mem::ManuallyDrop<Option<ID3D12Resource>>,
    pub Subresource: u32,
    pub StateBefore: D3D12_RESOURCE_STATES,
    pub StateAfter: D3D12_RESOURCE_STATES,
}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct D3D12_RESOURCE_UAV_BARRIER {
    pub pResource: core::mem::ManuallyDrop<Option<ID3D12Resource>>,
}
pub type D3D12_ROOT_SIGNATURE_FLAGS = u32;
pub const D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT: D3D12_ROOT_SIGNATURE_FLAGS =
    1;
pub const D3D12_ROOT_SIGNATURE_FLAG_ALLOW_STREAM_OUTPUT: D3D12_ROOT_SIGNATURE_FLAGS = 64;
pub const D3D12_ROOT_SIGNATURE_FLAG_CBV_SRV_UAV_HEAP_DIRECTLY_INDEXED: D3D12_ROOT_SIGNATURE_FLAGS =
    1024;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_AMPLIFICATION_SHADER_ROOT_ACCESS:
    D3D12_ROOT_SIGNATURE_FLAGS = 256;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS = 8;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS =
    16;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS = 4;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_MESH_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS = 512;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_PIXEL_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS = 32;
pub const D3D12_ROOT_SIGNATURE_FLAG_DENY_VERTEX_SHADER_ROOT_ACCESS: D3D12_ROOT_SIGNATURE_FLAGS = 2;
pub const D3D12_ROOT_SIGNATURE_FLAG_LOCAL_ROOT_SIGNATURE: D3D12_ROOT_SIGNATURE_FLAGS = 128;
pub const D3D12_ROOT_SIGNATURE_FLAG_NONE: D3D12_ROOT_SIGNATURE_FLAGS = 0;
pub const D3D12_ROOT_SIGNATURE_FLAG_SAMPLER_HEAP_DIRECTLY_INDEXED: D3D12_ROOT_SIGNATURE_FLAGS =
    2048;
pub type D3D12_RTV_DIMENSION = i32;
pub const D3D12_RTV_DIMENSION_BUFFER: D3D12_RTV_DIMENSION = 1;
pub const D3D12_RTV_DIMENSION_TEXTURE1D: D3D12_RTV_DIMENSION = 2;
pub const D3D12_RTV_DIMENSION_TEXTURE1DARRAY: D3D12_RTV_DIMENSION = 3;
pub const D3D12_RTV_DIMENSION_TEXTURE2D: D3D12_RTV_DIMENSION = 4;
pub const D3D12_RTV_DIMENSION_TEXTURE2DARRAY: D3D12_RTV_DIMENSION = 5;
pub const D3D12_RTV_DIMENSION_TEXTURE2DMS: D3D12_RTV_DIMENSION = 6;
pub const D3D12_RTV_DIMENSION_TEXTURE2DMSARRAY: D3D12_RTV_DIMENSION = 7;
pub const D3D12_RTV_DIMENSION_TEXTURE3D: D3D12_RTV_DIMENSION = 8;
pub const D3D12_RTV_DIMENSION_UNKNOWN: D3D12_RTV_DIMENSION = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct D3D12_SAMPLER_DESC {
    pub Filter: D3D12_FILTER,
    pub AddressU: D3D12_TEXTURE_ADDRESS_MODE,
    pub AddressV: D3D12_TEXTURE_ADDRESS_MODE,
    pub AddressW: D3D12_TEXTURE_ADDRESS_MODE,
    pub MipLODBias: f32,
    pub MaxAnisotropy: u32,
    pub ComparisonFunc: D3D12_COMPARISON_FUNC,
    pub BorderColor: [f32; 4],
    pub MinLOD: f32,
    pub MaxLOD: f32,
}
impl Default for D3D12_SAMPLER_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_SHADER_BYTECODE {
    pub pShaderBytecode: *const core::ffi::c_void,
    pub BytecodeLength: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_SHADER_RESOURCE_VIEW_DESC {
    pub Format: DXGI_FORMAT,
    pub ViewDimension: D3D12_SRV_DIMENSION,
    pub Shader4ComponentMapping: u32,
    pub Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0,
}
impl Default for D3D12_SHADER_RESOURCE_VIEW_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
    pub Buffer: D3D12_BUFFER_SRV,
    pub Texture1D: D3D12_TEX1D_SRV,
    pub Texture1DArray: D3D12_TEX1D_ARRAY_SRV,
    pub Texture2D: D3D12_TEX2D_SRV,
    pub Texture2DArray: D3D12_TEX2D_ARRAY_SRV,
    pub Texture2DMS: D3D12_TEX2DMS_SRV,
    pub Texture2DMSArray: D3D12_TEX2DMS_ARRAY_SRV,
    pub Texture3D: D3D12_TEX3D_SRV,
    pub TextureCube: D3D12_TEXCUBE_SRV,
    pub TextureCubeArray: D3D12_TEXCUBE_ARRAY_SRV,
    pub RaytracingAccelerationStructure: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_SRV,
}
impl Default for D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_SO_DECLARATION_ENTRY {
    pub Stream: u32,
    pub SemanticName: windows_core::PCSTR,
    pub SemanticIndex: u32,
    pub StartComponent: u8,
    pub ComponentCount: u8,
    pub OutputSlot: u8,
}
pub type D3D12_SRV_DIMENSION = i32;
pub const D3D12_SRV_DIMENSION_BUFFER: D3D12_SRV_DIMENSION = 1;
pub const D3D12_SRV_DIMENSION_RAYTRACING_ACCELERATION_STRUCTURE: D3D12_SRV_DIMENSION = 11;
pub const D3D12_SRV_DIMENSION_TEXTURE1D: D3D12_SRV_DIMENSION = 2;
pub const D3D12_SRV_DIMENSION_TEXTURE1DARRAY: D3D12_SRV_DIMENSION = 3;
pub const D3D12_SRV_DIMENSION_TEXTURE2D: D3D12_SRV_DIMENSION = 4;
pub const D3D12_SRV_DIMENSION_TEXTURE2DARRAY: D3D12_SRV_DIMENSION = 5;
pub const D3D12_SRV_DIMENSION_TEXTURE2DMS: D3D12_SRV_DIMENSION = 6;
pub const D3D12_SRV_DIMENSION_TEXTURE2DMSARRAY: D3D12_SRV_DIMENSION = 7;
pub const D3D12_SRV_DIMENSION_TEXTURE3D: D3D12_SRV_DIMENSION = 8;
pub const D3D12_SRV_DIMENSION_TEXTURECUBE: D3D12_SRV_DIMENSION = 9;
pub const D3D12_SRV_DIMENSION_TEXTURECUBEARRAY: D3D12_SRV_DIMENSION = 10;
pub const D3D12_SRV_DIMENSION_UNKNOWN: D3D12_SRV_DIMENSION = 0;
pub type D3D12_STENCIL_OP = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_STREAM_OUTPUT_BUFFER_VIEW {
    pub BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS,
    pub SizeInBytes: u64,
    pub BufferFilledSizeLocation: D3D12_GPU_VIRTUAL_ADDRESS,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_STREAM_OUTPUT_DESC {
    pub pSODeclaration: *const D3D12_SO_DECLARATION_ENTRY,
    pub NumEntries: u32,
    pub pBufferStrides: *const u32,
    pub NumStrides: u32,
    pub RasterizedStream: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_SUBRESOURCE_FOOTPRINT {
    pub Format: DXGI_FORMAT,
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
    pub RowPitch: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_SUBRESOURCE_TILING {
    pub WidthInTiles: u32,
    pub HeightInTiles: u16,
    pub DepthInTiles: u16,
    pub StartTileIndexInOverallResource: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEX1D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_ARRAY_UAV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_DSV {
    pub MipSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_RTV {
    pub MipSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEX1D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX1D_UAV {
    pub MipSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_ARRAY_DSV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_ARRAY_RTV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_ARRAY_SRV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_ARRAY_UAV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_DSV {
    pub UnusedField_NothingToDefine: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_RTV {
    pub UnusedField_NothingToDefine: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_SRV {
    pub UnusedField_NothingToDefine: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2DMS_UAV {
    pub UnusedField_NothingToDefine: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEX2D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_ARRAY_UAV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_DSV {
    pub MipSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_RTV {
    pub MipSlice: u32,
    pub PlaneSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEX2D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub PlaneSlice: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX2D_UAV {
    pub MipSlice: u32,
    pub PlaneSlice: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX3D_RTV {
    pub MipSlice: u32,
    pub FirstWSlice: u32,
    pub WSize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEX3D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TEX3D_UAV {
    pub MipSlice: u32,
    pub FirstWSlice: u32,
    pub WSize: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEXCUBE_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub First2DArrayFace: u32,
    pub NumCubes: u32,
    pub ResourceMinLODClamp: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_TEXCUBE_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub ResourceMinLODClamp: f32,
}
pub type D3D12_TEXTURE_ADDRESS_MODE = i32;
pub const D3D12_TEXTURE_ADDRESS_MODE_BORDER: D3D12_TEXTURE_ADDRESS_MODE = 4;
pub const D3D12_TEXTURE_ADDRESS_MODE_CLAMP: D3D12_TEXTURE_ADDRESS_MODE = 3;
pub const D3D12_TEXTURE_ADDRESS_MODE_MIRROR: D3D12_TEXTURE_ADDRESS_MODE = 2;
pub const D3D12_TEXTURE_ADDRESS_MODE_MIRROR_ONCE: D3D12_TEXTURE_ADDRESS_MODE = 5;
pub const D3D12_TEXTURE_ADDRESS_MODE_WRAP: D3D12_TEXTURE_ADDRESS_MODE = 1;
#[repr(C)]
pub struct D3D12_TEXTURE_COPY_LOCATION {
    pub pResource: core::mem::ManuallyDrop<Option<ID3D12Resource>>,
    pub Type: D3D12_TEXTURE_COPY_TYPE,
    pub Anonymous: D3D12_TEXTURE_COPY_LOCATION_0,
}
impl Clone for D3D12_TEXTURE_COPY_LOCATION {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for D3D12_TEXTURE_COPY_LOCATION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_TEXTURE_COPY_LOCATION_0 {
    pub PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    pub SubresourceIndex: u32,
}
impl Default for D3D12_TEXTURE_COPY_LOCATION_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type D3D12_TEXTURE_COPY_TYPE = i32;
pub const D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT: D3D12_TEXTURE_COPY_TYPE = 1;
pub const D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX: D3D12_TEXTURE_COPY_TYPE = 0;
pub const D3D12_TEXTURE_DATA_PITCH_ALIGNMENT: i32 = 256;
pub type D3D12_TEXTURE_LAYOUT = i32;
pub const D3D12_TEXTURE_LAYOUT_64KB_STANDARD_SWIZZLE: D3D12_TEXTURE_LAYOUT = 3;
pub const D3D12_TEXTURE_LAYOUT_64KB_UNDEFINED_SWIZZLE: D3D12_TEXTURE_LAYOUT = 2;
pub const D3D12_TEXTURE_LAYOUT_ROW_MAJOR: D3D12_TEXTURE_LAYOUT = 1;
pub const D3D12_TEXTURE_LAYOUT_UNKNOWN: D3D12_TEXTURE_LAYOUT = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TILED_RESOURCE_COORDINATE {
    pub X: u32,
    pub Y: u32,
    pub Z: u32,
    pub Subresource: u32,
}
pub type D3D12_TILE_COPY_FLAGS = u32;
pub type D3D12_TILE_MAPPING_FLAGS = u32;
pub type D3D12_TILE_RANGE_FLAGS = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TILE_REGION_SIZE {
    pub NumTiles: u32,
    pub UseBox: windows_core::BOOL,
    pub Width: u32,
    pub Height: u16,
    pub Depth: u16,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_TILE_SHAPE {
    pub WidthInTexels: u32,
    pub HeightInTexels: u32,
    pub DepthInTexels: u32,
}
pub type D3D12_UAV_DIMENSION = i32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_UNORDERED_ACCESS_VIEW_DESC {
    pub Format: DXGI_FORMAT,
    pub ViewDimension: D3D12_UAV_DIMENSION,
    pub Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0,
}
impl Default for D3D12_UNORDERED_ACCESS_VIEW_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
    pub Buffer: D3D12_BUFFER_UAV,
    pub Texture1D: D3D12_TEX1D_UAV,
    pub Texture1DArray: D3D12_TEX1D_ARRAY_UAV,
    pub Texture2D: D3D12_TEX2D_UAV,
    pub Texture2DArray: D3D12_TEX2D_ARRAY_UAV,
    pub Texture2DMS: D3D12_TEX2DMS_UAV,
    pub Texture2DMSArray: D3D12_TEX2DMS_ARRAY_UAV,
    pub Texture3D: D3D12_TEX3D_UAV,
}
impl Default for D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D12_VERTEX_BUFFER_VIEW {
    pub BufferLocation: D3D12_GPU_VIRTUAL_ADDRESS,
    pub SizeInBytes: u32,
    pub StrideInBytes: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3D12_VIEWPORT {
    pub TopLeftX: f32,
    pub TopLeftY: f32,
    pub Width: f32,
    pub Height: f32,
    pub MinDepth: f32,
    pub MaxDepth: f32,
}
pub type D3DKMT_SCHEDULINGPRIORITYCLASS = i32;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_ABOVE_NORMAL: D3DKMT_SCHEDULINGPRIORITYCLASS = 3;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_BELOW_NORMAL: D3DKMT_SCHEDULINGPRIORITYCLASS = 1;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_HIGH: D3DKMT_SCHEDULINGPRIORITYCLASS = 4;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_IDLE: D3DKMT_SCHEDULINGPRIORITYCLASS = 0;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_NORMAL: D3DKMT_SCHEDULINGPRIORITYCLASS = 2;
pub const D3DKMT_SCHEDULINGPRIORITYCLASS_REALTIME: D3DKMT_SCHEDULINGPRIORITYCLASS = 5;
pub type D3D_FEATURE_LEVEL = i32;
pub const D3D_FEATURE_LEVEL_10_0: D3D_FEATURE_LEVEL = 40960;
pub const D3D_FEATURE_LEVEL_10_1: D3D_FEATURE_LEVEL = 41216;
pub const D3D_FEATURE_LEVEL_11_0: D3D_FEATURE_LEVEL = 45056;
pub const D3D_FEATURE_LEVEL_11_1: D3D_FEATURE_LEVEL = 45312;
pub const D3D_FEATURE_LEVEL_12_0: D3D_FEATURE_LEVEL = 49152;
pub const D3D_FEATURE_LEVEL_12_1: D3D_FEATURE_LEVEL = 49408;
pub const D3D_FEATURE_LEVEL_12_2: D3D_FEATURE_LEVEL = 49664;
pub const D3D_FEATURE_LEVEL_1_0_CORE: D3D_FEATURE_LEVEL = 4096;
pub const D3D_FEATURE_LEVEL_1_0_GENERIC: D3D_FEATURE_LEVEL = 256;
pub const D3D_FEATURE_LEVEL_9_1: D3D_FEATURE_LEVEL = 37120;
pub const D3D_FEATURE_LEVEL_9_2: D3D_FEATURE_LEVEL = 37376;
pub const D3D_FEATURE_LEVEL_9_3: D3D_FEATURE_LEVEL = 37632;
pub type D3D_PRIMITIVE_TOPOLOGY = i32;
pub const D3D_PRIMITIVE_TOPOLOGY_10_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 42;
pub const D3D_PRIMITIVE_TOPOLOGY_11_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 43;
pub const D3D_PRIMITIVE_TOPOLOGY_12_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 44;
pub const D3D_PRIMITIVE_TOPOLOGY_13_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 45;
pub const D3D_PRIMITIVE_TOPOLOGY_14_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 46;
pub const D3D_PRIMITIVE_TOPOLOGY_15_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 47;
pub const D3D_PRIMITIVE_TOPOLOGY_16_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 48;
pub const D3D_PRIMITIVE_TOPOLOGY_17_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 49;
pub const D3D_PRIMITIVE_TOPOLOGY_18_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 50;
pub const D3D_PRIMITIVE_TOPOLOGY_19_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 51;
pub const D3D_PRIMITIVE_TOPOLOGY_1_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 33;
pub const D3D_PRIMITIVE_TOPOLOGY_20_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 52;
pub const D3D_PRIMITIVE_TOPOLOGY_21_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 53;
pub const D3D_PRIMITIVE_TOPOLOGY_22_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 54;
pub const D3D_PRIMITIVE_TOPOLOGY_23_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 55;
pub const D3D_PRIMITIVE_TOPOLOGY_24_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 56;
pub const D3D_PRIMITIVE_TOPOLOGY_25_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 57;
pub const D3D_PRIMITIVE_TOPOLOGY_26_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 58;
pub const D3D_PRIMITIVE_TOPOLOGY_27_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 59;
pub const D3D_PRIMITIVE_TOPOLOGY_28_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 60;
pub const D3D_PRIMITIVE_TOPOLOGY_29_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 61;
pub const D3D_PRIMITIVE_TOPOLOGY_2_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 34;
pub const D3D_PRIMITIVE_TOPOLOGY_30_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 62;
pub const D3D_PRIMITIVE_TOPOLOGY_31_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 63;
pub const D3D_PRIMITIVE_TOPOLOGY_32_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 64;
pub const D3D_PRIMITIVE_TOPOLOGY_3_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 35;
pub const D3D_PRIMITIVE_TOPOLOGY_4_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 36;
pub const D3D_PRIMITIVE_TOPOLOGY_5_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 37;
pub const D3D_PRIMITIVE_TOPOLOGY_6_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 38;
pub const D3D_PRIMITIVE_TOPOLOGY_7_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 39;
pub const D3D_PRIMITIVE_TOPOLOGY_8_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 40;
pub const D3D_PRIMITIVE_TOPOLOGY_9_CONTROL_POINT_PATCHLIST: D3D_PRIMITIVE_TOPOLOGY = 41;
pub const D3D_PRIMITIVE_TOPOLOGY_LINELIST: D3D_PRIMITIVE_TOPOLOGY = 2;
pub const D3D_PRIMITIVE_TOPOLOGY_LINELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 10;
pub const D3D_PRIMITIVE_TOPOLOGY_LINESTRIP: D3D_PRIMITIVE_TOPOLOGY = 3;
pub const D3D_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 11;
pub const D3D_PRIMITIVE_TOPOLOGY_POINTLIST: D3D_PRIMITIVE_TOPOLOGY = 1;
pub const D3D_PRIMITIVE_TOPOLOGY_TRIANGLEFAN: D3D_PRIMITIVE_TOPOLOGY = 6;
pub const D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST: D3D_PRIMITIVE_TOPOLOGY = 4;
pub const D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ: D3D_PRIMITIVE_TOPOLOGY = 12;
pub const D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP: D3D_PRIMITIVE_TOPOLOGY = 5;
pub const D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ: D3D_PRIMITIVE_TOPOLOGY = 13;
pub const D3D_PRIMITIVE_TOPOLOGY_UNDEFINED: D3D_PRIMITIVE_TOPOLOGY = 0;
pub const DC_BRUSH: i32 = 18;
pub type DECODING_SOURCE = i32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DEVINST(pub u32);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DEVNODE(pub u32);
pub const DEVPKEY_Device_BusReportedDeviceDesc: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0x540b947e_8b40_45bc_a8a2_6a0b894cbda2),
    pid: DEVPROPID(4),
};
pub const DEVPKEY_Device_Class: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(9),
};
pub const DEVPKEY_Device_DeviceDesc: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(2),
};
pub const DEVPKEY_Device_DriverInfPath: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa8b865dd_2e3d_4094_ad97_e593a70c75d6),
    pid: DEVPROPID(5),
};
pub const DEVPKEY_Device_FriendlyName: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(14),
};
pub const DEVPKEY_Device_HardwareIds: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(3),
};
pub const DEVPKEY_Device_InstanceId: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0x78c34fc8_104a_4aca_9ea4_524d52996e57),
    pid: DEVPROPID(256),
};
pub const DEVPKEY_Device_LocationInfo: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(15),
};
pub const DEVPKEY_Device_Manufacturer: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(13),
};
pub const DEVPKEY_Device_Service: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows_core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: DEVPROPID(6),
};
pub type DEVPROPGUID = windows_core::GUID;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DEVPROPID(pub u32);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DEVPROPKEY {
    pub fmtid: DEVPROPGUID,
    pub pid: DEVPROPID,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DEVPROPTYPE(pub u32);
pub const DEVPROP_TYPE_STRING: i32 = 18;
pub const DEVPROP_TYPE_STRING_LIST: i32 = 8210;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_2DREGION {
    pub cx: u32,
    pub cy: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_DESKTOP_IMAGE_INFO {
    pub PathSourceSize: POINTL,
    pub DesktopImageRegion: RECTL,
    pub DesktopImageClip: RECTL,
}
pub const DISPLAYCONFIG_DEVICE_INFO_FORCE_UINT32: DISPLAYCONFIG_DEVICE_INFO_TYPE = -1;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME: DISPLAYCONFIG_DEVICE_INFO_TYPE = 4;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO: DISPLAYCONFIG_DEVICE_INFO_TYPE = 9;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO_2: DISPLAYCONFIG_DEVICE_INFO_TYPE = 15;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_MONITOR_SPECIALIZATION: DISPLAYCONFIG_DEVICE_INFO_TYPE = 12;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL: DISPLAYCONFIG_DEVICE_INFO_TYPE = 11;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME: DISPLAYCONFIG_DEVICE_INFO_TYPE = 1;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_SUPPORT_VIRTUAL_RESOLUTION: DISPLAYCONFIG_DEVICE_INFO_TYPE =
    7;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_BASE_TYPE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 6;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME: DISPLAYCONFIG_DEVICE_INFO_TYPE = 2;
pub const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 3;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_DEVICE_INFO_HEADER {
    pub r#type: DISPLAYCONFIG_DEVICE_INFO_TYPE,
    pub size: u32,
    pub adapterId: LUID,
    pub id: u32,
}
pub const DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 10;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_HDR_STATE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 16;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_MONITOR_SPECIALIZATION: DISPLAYCONFIG_DEVICE_INFO_TYPE = 13;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_RESERVED1: DISPLAYCONFIG_DEVICE_INFO_TYPE = 14;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_SUPPORT_VIRTUAL_RESOLUTION: DISPLAYCONFIG_DEVICE_INFO_TYPE =
    8;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_TARGET_PERSISTENCE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 5;
pub const DISPLAYCONFIG_DEVICE_INFO_SET_WCG_STATE: DISPLAYCONFIG_DEVICE_INFO_TYPE = 17;
pub type DISPLAYCONFIG_DEVICE_INFO_TYPE = i32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_MODE_INFO {
    pub infoType: DISPLAYCONFIG_MODE_INFO_TYPE,
    pub id: u32,
    pub adapterId: LUID,
    pub Anonymous: DISPLAYCONFIG_MODE_INFO_0,
}
impl Default for DISPLAYCONFIG_MODE_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_MODE_INFO_0 {
    pub targetMode: DISPLAYCONFIG_TARGET_MODE,
    pub sourceMode: DISPLAYCONFIG_SOURCE_MODE,
    pub desktopImageInfo: DISPLAYCONFIG_DESKTOP_IMAGE_INFO,
}
impl Default for DISPLAYCONFIG_MODE_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type DISPLAYCONFIG_MODE_INFO_TYPE = i32;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPONENT_VIDEO: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY =
    3;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPOSITE_VIDEO: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY =
    2;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED:
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 11;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL:
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 10;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL:
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 18;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 4;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_D_JPN: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 8;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_FORCE_UINT32: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = -1;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 0;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 5;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY =
    17;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY =
    16;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY =
    -2147483648;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 6;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 15;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = -1;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDI: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 9;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDTVDONGLE: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 14;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SVIDEO: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 1;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 13;
pub const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EXTERNAL: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = 12;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_INFO {
    pub sourceInfo: DISPLAYCONFIG_PATH_SOURCE_INFO,
    pub targetInfo: DISPLAYCONFIG_PATH_TARGET_INFO,
    pub flags: u32,
}
impl Default for DISPLAYCONFIG_PATH_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_SOURCE_INFO {
    pub adapterId: LUID,
    pub id: u32,
    pub Anonymous: DISPLAYCONFIG_PATH_SOURCE_INFO_0,
    pub statusFlags: u32,
}
impl Default for DISPLAYCONFIG_PATH_SOURCE_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_PATH_SOURCE_INFO_0 {
    pub modeInfoIdx: u32,
    pub Anonymous: DISPLAYCONFIG_PATH_SOURCE_INFO_0_0,
}
impl Default for DISPLAYCONFIG_PATH_SOURCE_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_PATH_SOURCE_INFO_0_0 {
    pub _bitfield: u32,
}
impl DISPLAYCONFIG_PATH_SOURCE_INFO_0_0 {
    pub fn cloneGroupId(&self) -> u32 {
        (self._bitfield << 16) >> 16
    }
    pub fn set_cloneGroupId(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !65535) | (value & 65535);
    }
    pub fn sourceModeInfoIdx(&self) -> u32 {
        self._bitfield >> 16
    }
    pub fn set_sourceModeInfoIdx(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(65535 << 16)) | ((value & 65535) << 16);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_PATH_TARGET_INFO {
    pub adapterId: LUID,
    pub id: u32,
    pub Anonymous: DISPLAYCONFIG_PATH_TARGET_INFO_0,
    pub outputTechnology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    pub rotation: DISPLAYCONFIG_ROTATION,
    pub scaling: DISPLAYCONFIG_SCALING,
    pub refreshRate: DISPLAYCONFIG_RATIONAL,
    pub scanLineOrdering: DISPLAYCONFIG_SCANLINE_ORDERING,
    pub targetAvailable: windows_core::BOOL,
    pub statusFlags: u32,
}
impl Default for DISPLAYCONFIG_PATH_TARGET_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_PATH_TARGET_INFO_0 {
    pub modeInfoIdx: u32,
    pub Anonymous: DISPLAYCONFIG_PATH_TARGET_INFO_0_0,
}
impl Default for DISPLAYCONFIG_PATH_TARGET_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_PATH_TARGET_INFO_0_0 {
    pub _bitfield: u32,
}
impl DISPLAYCONFIG_PATH_TARGET_INFO_0_0 {
    pub fn desktopModeInfoIdx(&self) -> u32 {
        (self._bitfield << 16) >> 16
    }
    pub fn set_desktopModeInfoIdx(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !65535) | (value & 65535);
    }
    pub fn targetModeInfoIdx(&self) -> u32 {
        self._bitfield >> 16
    }
    pub fn set_targetModeInfoIdx(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(65535 << 16)) | ((value & 65535) << 16);
    }
}
pub type DISPLAYCONFIG_PIXELFORMAT = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_RATIONAL {
    pub Numerator: u32,
    pub Denominator: u32,
}
pub type DISPLAYCONFIG_ROTATION = i32;
pub type DISPLAYCONFIG_SCALING = i32;
pub type DISPLAYCONFIG_SCANLINE_ORDERING = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DISPLAYCONFIG_SOURCE_DEVICE_NAME {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub viewGdiDeviceName: [u16; 32],
}
impl Default for DISPLAYCONFIG_SOURCE_DEVICE_NAME {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_SOURCE_MODE {
    pub width: u32,
    pub height: u32,
    pub pixelFormat: DISPLAYCONFIG_PIXELFORMAT,
    pub position: POINTL,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_DEVICE_NAME {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub flags: DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS,
    pub outputTechnology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    pub edidManufactureId: u16,
    pub edidProductCodeId: u16,
    pub connectorInstance: u32,
    pub monitorFriendlyDeviceName: [u16; 64],
    pub monitorDevicePath: [u16; 128],
}
impl Default for DISPLAYCONFIG_TARGET_DEVICE_NAME {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS {
    pub Anonymous: DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0,
}
impl Default for DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0 {
    pub Anonymous: DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0_0,
    pub value: u32,
}
impl Default for DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0_0 {
    pub _bitfield: u32,
}
impl DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS_0_0 {
    pub fn friendlyNameFromEdid(&self) -> bool {
        self._bitfield & 1 != 0
    }
    pub fn set_friendlyNameFromEdid(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !1) | (value as u32);
    }
    pub fn friendlyNameForced(&self) -> bool {
        (self._bitfield >> 1) & 1 != 0
    }
    pub fn set_friendlyNameForced(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !(1 << 1)) | ((value as u32) << 1);
    }
    pub fn edidIdsValid(&self) -> bool {
        (self._bitfield >> 2) & 1 != 0
    }
    pub fn set_edidIdsValid(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !(1 << 2)) | ((value as u32) << 2);
    }
    pub fn reserved(&self) -> u32 {
        self._bitfield >> 3
    }
    pub fn set_reserved(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(536870911 << 3)) | ((value & 536870911) << 3);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_TARGET_MODE {
    pub targetVideoSignalInfo: DISPLAYCONFIG_VIDEO_SIGNAL_INFO,
}
impl Default for DISPLAYCONFIG_TARGET_MODE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type DISPLAYCONFIG_TOPOLOGY_ID = i32;
pub type DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY = i32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
    pub pixelRate: u64,
    pub hSyncFreq: DISPLAYCONFIG_RATIONAL,
    pub vSyncFreq: DISPLAYCONFIG_RATIONAL,
    pub activeSize: DISPLAYCONFIG_2DREGION,
    pub totalSize: DISPLAYCONFIG_2DREGION,
    pub Anonymous: DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0,
    pub scanLineOrdering: DISPLAYCONFIG_SCANLINE_ORDERING,
}
impl Default for DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0 {
    pub AdditionalSignalInfo: DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0_0,
    pub videoStandard: u32,
}
impl Default for DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0_0 {
    pub _bitfield: u32,
}
impl DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0_0 {
    pub fn videoStandard(&self) -> u32 {
        (self._bitfield << 16) >> 16
    }
    pub fn set_videoStandard(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !65535) | (value & 65535);
    }
    pub fn vSyncFreqDivider(&self) -> u32 {
        (self._bitfield << 10) >> 26
    }
    pub fn set_vSyncFreqDivider(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(63 << 16)) | ((value & 63) << 16);
    }
    pub fn reserved(&self) -> u32 {
        self._bitfield >> 22
    }
    pub fn set_reserved(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(1023 << 22)) | ((value & 1023) << 22);
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DPI_AWARENESS_CONTEXT(pub *mut core::ffi::c_void);
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: DPI_AWARENESS_CONTEXT =
    DPI_AWARENESS_CONTEXT(-4 as _);
pub const DWMWA_ALLOW_NCPAINT: DWMWINDOWATTRIBUTE = 4;
pub const DWMWA_BORDER_COLOR: DWMWINDOWATTRIBUTE = 34;
pub const DWMWA_BORDER_MARGINS: DWMWINDOWATTRIBUTE = 40;
pub const DWMWA_CAPTION_BUTTON_BOUNDS: DWMWINDOWATTRIBUTE = 5;
pub const DWMWA_CAPTION_COLOR: DWMWINDOWATTRIBUTE = 35;
pub const DWMWA_CLOAK: DWMWINDOWATTRIBUTE = 13;
pub const DWMWA_CLOAKED: DWMWINDOWATTRIBUTE = 14;
pub const DWMWA_DISALLOW_PEEK: DWMWINDOWATTRIBUTE = 11;
pub const DWMWA_EXCLUDED_FROM_PEEK: DWMWINDOWATTRIBUTE = 12;
pub const DWMWA_EXTENDED_FRAME_BOUNDS: DWMWINDOWATTRIBUTE = 9;
pub const DWMWA_FLIP3D_POLICY: DWMWINDOWATTRIBUTE = 8;
pub const DWMWA_FORCE_ICONIC_REPRESENTATION: DWMWINDOWATTRIBUTE = 7;
pub const DWMWA_FREEZE_REPRESENTATION: DWMWINDOWATTRIBUTE = 15;
pub const DWMWA_HAS_ICONIC_BITMAP: DWMWINDOWATTRIBUTE = 10;
pub const DWMWA_LAST: DWMWINDOWATTRIBUTE = 41;
pub const DWMWA_NCRENDERING_ENABLED: DWMWINDOWATTRIBUTE = 1;
pub const DWMWA_NCRENDERING_POLICY: DWMWINDOWATTRIBUTE = 2;
pub const DWMWA_NONCLIENT_RTL_LAYOUT: DWMWINDOWATTRIBUTE = 6;
pub const DWMWA_PASSIVE_UPDATE_MODE: DWMWINDOWATTRIBUTE = 16;
pub const DWMWA_REDIRECTIONBITMAP_ALPHA: DWMWINDOWATTRIBUTE = 39;
pub const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38;
pub const DWMWA_TEXT_COLOR: DWMWINDOWATTRIBUTE = 36;
pub const DWMWA_TRANSITIONS_FORCEDISABLED: DWMWINDOWATTRIBUTE = 3;
pub const DWMWA_USE_HOSTBACKDROPBRUSH: DWMWINDOWATTRIBUTE = 17;
pub const DWMWA_USE_IMMERSIVE_DARK_MODE: DWMWINDOWATTRIBUTE = 20;
pub const DWMWA_VISIBLE_FRAME_BORDER_THICKNESS: DWMWINDOWATTRIBUTE = 37;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = 33;
pub type DWMWINDOWATTRIBUTE = i32;
pub type DWRITE_FACTORY_TYPE = i32;
pub const DWRITE_FACTORY_TYPE_ISOLATED: DWRITE_FACTORY_TYPE = 1;
pub const DWRITE_FACTORY_TYPE_SHARED: DWRITE_FACTORY_TYPE = 0;
pub type DWRITE_FLOW_DIRECTION = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_FONT_METRICS {
    pub designUnitsPerEm: u16,
    pub ascent: u16,
    pub descent: u16,
    pub lineGap: i16,
    pub capHeight: u16,
    pub xHeight: u16,
    pub underlinePosition: i16,
    pub underlineThickness: u16,
    pub strikethroughPosition: i16,
    pub strikethroughThickness: u16,
}
pub type DWRITE_FONT_STRETCH = i32;
pub const DWRITE_FONT_STRETCH_CONDENSED: DWRITE_FONT_STRETCH = 3;
pub const DWRITE_FONT_STRETCH_EXPANDED: DWRITE_FONT_STRETCH = 7;
pub const DWRITE_FONT_STRETCH_EXTRA_CONDENSED: DWRITE_FONT_STRETCH = 2;
pub const DWRITE_FONT_STRETCH_EXTRA_EXPANDED: DWRITE_FONT_STRETCH = 8;
pub const DWRITE_FONT_STRETCH_MEDIUM: DWRITE_FONT_STRETCH = 5;
pub const DWRITE_FONT_STRETCH_NORMAL: DWRITE_FONT_STRETCH = 5;
pub const DWRITE_FONT_STRETCH_SEMI_CONDENSED: DWRITE_FONT_STRETCH = 4;
pub const DWRITE_FONT_STRETCH_SEMI_EXPANDED: DWRITE_FONT_STRETCH = 6;
pub const DWRITE_FONT_STRETCH_ULTRA_CONDENSED: DWRITE_FONT_STRETCH = 1;
pub const DWRITE_FONT_STRETCH_ULTRA_EXPANDED: DWRITE_FONT_STRETCH = 9;
pub const DWRITE_FONT_STRETCH_UNDEFINED: DWRITE_FONT_STRETCH = 0;
pub type DWRITE_FONT_STYLE = i32;
pub const DWRITE_FONT_STYLE_ITALIC: DWRITE_FONT_STYLE = 2;
pub const DWRITE_FONT_STYLE_NORMAL: DWRITE_FONT_STYLE = 0;
pub const DWRITE_FONT_STYLE_OBLIQUE: DWRITE_FONT_STYLE = 1;
pub type DWRITE_FONT_WEIGHT = i32;
pub const DWRITE_FONT_WEIGHT_BLACK: DWRITE_FONT_WEIGHT = 900;
pub const DWRITE_FONT_WEIGHT_BOLD: DWRITE_FONT_WEIGHT = 700;
pub const DWRITE_FONT_WEIGHT_DEMI_BOLD: DWRITE_FONT_WEIGHT = 600;
pub const DWRITE_FONT_WEIGHT_EXTRA_BLACK: DWRITE_FONT_WEIGHT = 950;
pub const DWRITE_FONT_WEIGHT_EXTRA_BOLD: DWRITE_FONT_WEIGHT = 800;
pub const DWRITE_FONT_WEIGHT_EXTRA_LIGHT: DWRITE_FONT_WEIGHT = 200;
pub const DWRITE_FONT_WEIGHT_HEAVY: DWRITE_FONT_WEIGHT = 900;
pub const DWRITE_FONT_WEIGHT_LIGHT: DWRITE_FONT_WEIGHT = 300;
pub const DWRITE_FONT_WEIGHT_MEDIUM: DWRITE_FONT_WEIGHT = 500;
pub const DWRITE_FONT_WEIGHT_NORMAL: DWRITE_FONT_WEIGHT = 400;
pub const DWRITE_FONT_WEIGHT_REGULAR: DWRITE_FONT_WEIGHT = 400;
pub const DWRITE_FONT_WEIGHT_SEMI_BOLD: DWRITE_FONT_WEIGHT = 600;
pub const DWRITE_FONT_WEIGHT_SEMI_LIGHT: DWRITE_FONT_WEIGHT = 350;
pub const DWRITE_FONT_WEIGHT_THIN: DWRITE_FONT_WEIGHT = 100;
pub const DWRITE_FONT_WEIGHT_ULTRA_BLACK: DWRITE_FONT_WEIGHT = 950;
pub const DWRITE_FONT_WEIGHT_ULTRA_BOLD: DWRITE_FONT_WEIGHT = 800;
pub const DWRITE_FONT_WEIGHT_ULTRA_LIGHT: DWRITE_FONT_WEIGHT = 200;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_GLYPH_METRICS {
    pub leftSideBearing: i32,
    pub advanceWidth: u32,
    pub rightSideBearing: i32,
    pub topSideBearing: i32,
    pub advanceHeight: u32,
    pub bottomSideBearing: i32,
    pub verticalOriginY: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_OFFSET {
    pub advanceOffset: f32,
    pub ascenderOffset: f32,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_RUN {
    pub fontFace: core::mem::ManuallyDrop<Option<IDWriteFontFace>>,
    pub fontEmSize: f32,
    pub glyphCount: u32,
    pub glyphIndices: *const u16,
    pub glyphAdvances: *const f32,
    pub glyphOffsets: *const DWRITE_GLYPH_OFFSET,
    pub isSideways: windows_core::BOOL,
    pub bidiLevel: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_GLYPH_RUN_DESCRIPTION {
    pub localeName: *const u16,
    pub string: *const u16,
    pub stringLength: u32,
    pub clusterMap: *const u16,
    pub textPosition: u32,
}
pub type DWRITE_GRID_FIT_MODE = i32;
pub const DWRITE_GRID_FIT_MODE_DEFAULT: DWRITE_GRID_FIT_MODE = 0;
pub const DWRITE_GRID_FIT_MODE_DISABLED: DWRITE_GRID_FIT_MODE = 1;
pub const DWRITE_GRID_FIT_MODE_ENABLED: DWRITE_GRID_FIT_MODE = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_HIT_TEST_METRICS {
    pub textPosition: u32,
    pub length: u32,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub bidiLevel: u32,
    pub isText: windows_core::BOOL,
    pub isTrimmed: windows_core::BOOL,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_LINE_METRICS {
    pub length: u32,
    pub trailingWhitespaceLength: u32,
    pub newlineLength: u32,
    pub height: f32,
    pub baseline: f32,
    pub isTrimmed: windows_core::BOOL,
}
pub type DWRITE_LINE_SPACING_METHOD = i32;
pub const DWRITE_LINE_SPACING_METHOD_DEFAULT: DWRITE_LINE_SPACING_METHOD = 0;
pub const DWRITE_LINE_SPACING_METHOD_PROPORTIONAL: DWRITE_LINE_SPACING_METHOD = 2;
pub const DWRITE_LINE_SPACING_METHOD_UNIFORM: DWRITE_LINE_SPACING_METHOD = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_MATRIX {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub dx: f32,
    pub dy: f32,
}
pub type DWRITE_MEASURING_MODE = i32;
pub const DWRITE_MEASURING_MODE_GDI_CLASSIC: DWRITE_MEASURING_MODE = 1;
pub const DWRITE_MEASURING_MODE_GDI_NATURAL: DWRITE_MEASURING_MODE = 2;
pub const DWRITE_MEASURING_MODE_NATURAL: DWRITE_MEASURING_MODE = 0;
pub type DWRITE_OUTLINE_THRESHOLD = i32;
pub const DWRITE_OUTLINE_THRESHOLD_ALIASED: DWRITE_OUTLINE_THRESHOLD = 1;
pub const DWRITE_OUTLINE_THRESHOLD_ANTIALIASED: DWRITE_OUTLINE_THRESHOLD = 0;
pub type DWRITE_PARAGRAPH_ALIGNMENT = i32;
pub const DWRITE_PARAGRAPH_ALIGNMENT_CENTER: DWRITE_PARAGRAPH_ALIGNMENT = 2;
pub const DWRITE_PARAGRAPH_ALIGNMENT_FAR: DWRITE_PARAGRAPH_ALIGNMENT = 1;
pub const DWRITE_PARAGRAPH_ALIGNMENT_NEAR: DWRITE_PARAGRAPH_ALIGNMENT = 0;
pub type DWRITE_READING_DIRECTION = i32;
pub type DWRITE_RENDERING_MODE = i32;
pub const DWRITE_RENDERING_MODE_ALIASED: DWRITE_RENDERING_MODE = 1;
pub const DWRITE_RENDERING_MODE_CLEARTYPE_GDI_CLASSIC: DWRITE_RENDERING_MODE = 2;
pub const DWRITE_RENDERING_MODE_CLEARTYPE_GDI_NATURAL: DWRITE_RENDERING_MODE = 3;
pub const DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL: DWRITE_RENDERING_MODE = 4;
pub const DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL_SYMMETRIC: DWRITE_RENDERING_MODE = 5;
pub const DWRITE_RENDERING_MODE_DEFAULT: DWRITE_RENDERING_MODE = 0;
pub const DWRITE_RENDERING_MODE_GDI_CLASSIC: DWRITE_RENDERING_MODE = 2;
pub const DWRITE_RENDERING_MODE_GDI_NATURAL: DWRITE_RENDERING_MODE = 3;
pub const DWRITE_RENDERING_MODE_NATURAL: DWRITE_RENDERING_MODE = 4;
pub const DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC: DWRITE_RENDERING_MODE = 5;
pub const DWRITE_RENDERING_MODE_OUTLINE: DWRITE_RENDERING_MODE = 6;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_STRIKETHROUGH {
    pub width: f32,
    pub thickness: f32,
    pub offset: f32,
    pub readingDirection: DWRITE_READING_DIRECTION,
    pub flowDirection: DWRITE_FLOW_DIRECTION,
    pub localeName: *const u16,
    pub measuringMode: DWRITE_MEASURING_MODE,
}
pub const DWRITE_TEXTURE_ALIASED_1x1: DWRITE_TEXTURE_TYPE = 0;
pub const DWRITE_TEXTURE_CLEARTYPE_3x1: DWRITE_TEXTURE_TYPE = 1;
pub type DWRITE_TEXTURE_TYPE = i32;
pub type DWRITE_TEXT_ALIGNMENT = i32;
pub const DWRITE_TEXT_ALIGNMENT_CENTER: DWRITE_TEXT_ALIGNMENT = 2;
pub const DWRITE_TEXT_ALIGNMENT_JUSTIFIED: DWRITE_TEXT_ALIGNMENT = 3;
pub const DWRITE_TEXT_ALIGNMENT_LEADING: DWRITE_TEXT_ALIGNMENT = 0;
pub const DWRITE_TEXT_ALIGNMENT_TRAILING: DWRITE_TEXT_ALIGNMENT = 1;
pub type DWRITE_TEXT_ANTIALIAS_MODE = i32;
pub const DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE: DWRITE_TEXT_ANTIALIAS_MODE = 0;
pub const DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE: DWRITE_TEXT_ANTIALIAS_MODE = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_TEXT_METRICS {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub widthIncludingTrailingWhitespace: f32,
    pub height: f32,
    pub layoutWidth: f32,
    pub layoutHeight: f32,
    pub maxBidiReorderingDepth: u32,
    pub lineCount: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_UNDERLINE {
    pub width: f32,
    pub thickness: f32,
    pub offset: f32,
    pub runHeight: f32,
    pub readingDirection: DWRITE_READING_DIRECTION,
    pub flowDirection: DWRITE_FLOW_DIRECTION,
    pub localeName: *const u16,
    pub measuringMode: DWRITE_MEASURING_MODE,
}
pub type DWRITE_WORD_WRAPPING = i32;
pub const DWRITE_WORD_WRAPPING_CHARACTER: DWRITE_WORD_WRAPPING = 4;
pub const DWRITE_WORD_WRAPPING_EMERGENCY_BREAK: DWRITE_WORD_WRAPPING = 2;
pub const DWRITE_WORD_WRAPPING_NO_WRAP: DWRITE_WORD_WRAPPING = 1;
pub const DWRITE_WORD_WRAPPING_WHOLE_WORD: DWRITE_WORD_WRAPPING = 3;
pub const DWRITE_WORD_WRAPPING_WRAP: DWRITE_WORD_WRAPPING = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DXGI_ADAPTER_DESC1 {
    pub Description: [u16; 128],
    pub VendorId: u32,
    pub DeviceId: u32,
    pub SubSysId: u32,
    pub Revision: u32,
    pub DedicatedVideoMemory: usize,
    pub DedicatedSystemMemory: usize,
    pub SharedSystemMemory: usize,
    pub AdapterLuid: LUID,
    pub Flags: u32,
}
impl Default for DXGI_ADAPTER_DESC1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type DXGI_ALPHA_MODE = i32;
pub type DXGI_COLOR_SPACE_TYPE = i32;
pub const DXGI_CREATE_FACTORY_DEBUG: i32 = 1;
pub type DXGI_FEATURE = i32;
pub const DXGI_FEATURE_PRESENT_ALLOW_TEARING: DXGI_FEATURE = 0;
pub type DXGI_FORMAT = i32;
pub const DXGI_FORMAT_420_OPAQUE: DXGI_FORMAT = 106;
pub const DXGI_FORMAT_A4B4G4R4_UNORM: DXGI_FORMAT = 191;
pub const DXGI_FORMAT_A8P8: DXGI_FORMAT = 114;
pub const DXGI_FORMAT_A8_UNORM: DXGI_FORMAT = 65;
pub const DXGI_FORMAT_AI44: DXGI_FORMAT = 111;
pub const DXGI_FORMAT_AYUV: DXGI_FORMAT = 100;
pub const DXGI_FORMAT_B4G4R4A4_UNORM: DXGI_FORMAT = 115;
pub const DXGI_FORMAT_B5G5R5A1_UNORM: DXGI_FORMAT = 86;
pub const DXGI_FORMAT_B5G6R5_UNORM: DXGI_FORMAT = 85;
pub const DXGI_FORMAT_B8G8R8A8_TYPELESS: DXGI_FORMAT = 90;
pub const DXGI_FORMAT_B8G8R8A8_UNORM: DXGI_FORMAT = 87;
pub const DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: DXGI_FORMAT = 91;
pub const DXGI_FORMAT_B8G8R8X8_TYPELESS: DXGI_FORMAT = 92;
pub const DXGI_FORMAT_B8G8R8X8_UNORM: DXGI_FORMAT = 88;
pub const DXGI_FORMAT_B8G8R8X8_UNORM_SRGB: DXGI_FORMAT = 93;
pub const DXGI_FORMAT_BC1_TYPELESS: DXGI_FORMAT = 70;
pub const DXGI_FORMAT_BC1_UNORM: DXGI_FORMAT = 71;
pub const DXGI_FORMAT_BC1_UNORM_SRGB: DXGI_FORMAT = 72;
pub const DXGI_FORMAT_BC2_TYPELESS: DXGI_FORMAT = 73;
pub const DXGI_FORMAT_BC2_UNORM: DXGI_FORMAT = 74;
pub const DXGI_FORMAT_BC2_UNORM_SRGB: DXGI_FORMAT = 75;
pub const DXGI_FORMAT_BC3_TYPELESS: DXGI_FORMAT = 76;
pub const DXGI_FORMAT_BC3_UNORM: DXGI_FORMAT = 77;
pub const DXGI_FORMAT_BC3_UNORM_SRGB: DXGI_FORMAT = 78;
pub const DXGI_FORMAT_BC4_SNORM: DXGI_FORMAT = 81;
pub const DXGI_FORMAT_BC4_TYPELESS: DXGI_FORMAT = 79;
pub const DXGI_FORMAT_BC4_UNORM: DXGI_FORMAT = 80;
pub const DXGI_FORMAT_BC5_SNORM: DXGI_FORMAT = 84;
pub const DXGI_FORMAT_BC5_TYPELESS: DXGI_FORMAT = 82;
pub const DXGI_FORMAT_BC5_UNORM: DXGI_FORMAT = 83;
pub const DXGI_FORMAT_BC6H_SF16: DXGI_FORMAT = 96;
pub const DXGI_FORMAT_BC6H_TYPELESS: DXGI_FORMAT = 94;
pub const DXGI_FORMAT_BC6H_UF16: DXGI_FORMAT = 95;
pub const DXGI_FORMAT_BC7_TYPELESS: DXGI_FORMAT = 97;
pub const DXGI_FORMAT_BC7_UNORM: DXGI_FORMAT = 98;
pub const DXGI_FORMAT_BC7_UNORM_SRGB: DXGI_FORMAT = 99;
pub const DXGI_FORMAT_D16_UNORM: DXGI_FORMAT = 55;
pub const DXGI_FORMAT_D24_UNORM_S8_UINT: DXGI_FORMAT = 45;
pub const DXGI_FORMAT_D32_FLOAT: DXGI_FORMAT = 40;
pub const DXGI_FORMAT_D32_FLOAT_S8X24_UINT: DXGI_FORMAT = 20;
pub const DXGI_FORMAT_FORCE_UINT: DXGI_FORMAT = -1;
pub const DXGI_FORMAT_G8R8_G8B8_UNORM: DXGI_FORMAT = 69;
pub const DXGI_FORMAT_IA44: DXGI_FORMAT = 112;
pub const DXGI_FORMAT_NV11: DXGI_FORMAT = 110;
pub const DXGI_FORMAT_NV12: DXGI_FORMAT = 103;
pub const DXGI_FORMAT_P010: DXGI_FORMAT = 104;
pub const DXGI_FORMAT_P016: DXGI_FORMAT = 105;
pub const DXGI_FORMAT_P208: DXGI_FORMAT = 130;
pub const DXGI_FORMAT_P8: DXGI_FORMAT = 113;
pub const DXGI_FORMAT_R10G10B10A2_TYPELESS: DXGI_FORMAT = 23;
pub const DXGI_FORMAT_R10G10B10A2_UINT: DXGI_FORMAT = 25;
pub const DXGI_FORMAT_R10G10B10A2_UNORM: DXGI_FORMAT = 24;
pub const DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM: DXGI_FORMAT = 89;
pub const DXGI_FORMAT_R11G11B10_FLOAT: DXGI_FORMAT = 26;
pub const DXGI_FORMAT_R16G16B16A16_FLOAT: DXGI_FORMAT = 10;
pub const DXGI_FORMAT_R16G16B16A16_SINT: DXGI_FORMAT = 14;
pub const DXGI_FORMAT_R16G16B16A16_SNORM: DXGI_FORMAT = 13;
pub const DXGI_FORMAT_R16G16B16A16_TYPELESS: DXGI_FORMAT = 9;
pub const DXGI_FORMAT_R16G16B16A16_UINT: DXGI_FORMAT = 12;
pub const DXGI_FORMAT_R16G16B16A16_UNORM: DXGI_FORMAT = 11;
pub const DXGI_FORMAT_R16G16_FLOAT: DXGI_FORMAT = 34;
pub const DXGI_FORMAT_R16G16_SINT: DXGI_FORMAT = 38;
pub const DXGI_FORMAT_R16G16_SNORM: DXGI_FORMAT = 37;
pub const DXGI_FORMAT_R16G16_TYPELESS: DXGI_FORMAT = 33;
pub const DXGI_FORMAT_R16G16_UINT: DXGI_FORMAT = 36;
pub const DXGI_FORMAT_R16G16_UNORM: DXGI_FORMAT = 35;
pub const DXGI_FORMAT_R16_FLOAT: DXGI_FORMAT = 54;
pub const DXGI_FORMAT_R16_SINT: DXGI_FORMAT = 59;
pub const DXGI_FORMAT_R16_SNORM: DXGI_FORMAT = 58;
pub const DXGI_FORMAT_R16_TYPELESS: DXGI_FORMAT = 53;
pub const DXGI_FORMAT_R16_UINT: DXGI_FORMAT = 57;
pub const DXGI_FORMAT_R16_UNORM: DXGI_FORMAT = 56;
pub const DXGI_FORMAT_R1_UNORM: DXGI_FORMAT = 66;
pub const DXGI_FORMAT_R24G8_TYPELESS: DXGI_FORMAT = 44;
pub const DXGI_FORMAT_R24_UNORM_X8_TYPELESS: DXGI_FORMAT = 46;
pub const DXGI_FORMAT_R32G32B32A32_FLOAT: DXGI_FORMAT = 2;
pub const DXGI_FORMAT_R32G32B32A32_SINT: DXGI_FORMAT = 4;
pub const DXGI_FORMAT_R32G32B32A32_TYPELESS: DXGI_FORMAT = 1;
pub const DXGI_FORMAT_R32G32B32A32_UINT: DXGI_FORMAT = 3;
pub const DXGI_FORMAT_R32G32B32_FLOAT: DXGI_FORMAT = 6;
pub const DXGI_FORMAT_R32G32B32_SINT: DXGI_FORMAT = 8;
pub const DXGI_FORMAT_R32G32B32_TYPELESS: DXGI_FORMAT = 5;
pub const DXGI_FORMAT_R32G32B32_UINT: DXGI_FORMAT = 7;
pub const DXGI_FORMAT_R32G32_FLOAT: DXGI_FORMAT = 16;
pub const DXGI_FORMAT_R32G32_SINT: DXGI_FORMAT = 18;
pub const DXGI_FORMAT_R32G32_TYPELESS: DXGI_FORMAT = 15;
pub const DXGI_FORMAT_R32G32_UINT: DXGI_FORMAT = 17;
pub const DXGI_FORMAT_R32G8X24_TYPELESS: DXGI_FORMAT = 19;
pub const DXGI_FORMAT_R32_FLOAT: DXGI_FORMAT = 41;
pub const DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS: DXGI_FORMAT = 21;
pub const DXGI_FORMAT_R32_SINT: DXGI_FORMAT = 43;
pub const DXGI_FORMAT_R32_TYPELESS: DXGI_FORMAT = 39;
pub const DXGI_FORMAT_R32_UINT: DXGI_FORMAT = 42;
pub const DXGI_FORMAT_R8G8B8A8_SINT: DXGI_FORMAT = 32;
pub const DXGI_FORMAT_R8G8B8A8_SNORM: DXGI_FORMAT = 31;
pub const DXGI_FORMAT_R8G8B8A8_TYPELESS: DXGI_FORMAT = 27;
pub const DXGI_FORMAT_R8G8B8A8_UINT: DXGI_FORMAT = 30;
pub const DXGI_FORMAT_R8G8B8A8_UNORM: DXGI_FORMAT = 28;
pub const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: DXGI_FORMAT = 29;
pub const DXGI_FORMAT_R8G8_B8G8_UNORM: DXGI_FORMAT = 68;
pub const DXGI_FORMAT_R8G8_SINT: DXGI_FORMAT = 52;
pub const DXGI_FORMAT_R8G8_SNORM: DXGI_FORMAT = 51;
pub const DXGI_FORMAT_R8G8_TYPELESS: DXGI_FORMAT = 48;
pub const DXGI_FORMAT_R8G8_UINT: DXGI_FORMAT = 50;
pub const DXGI_FORMAT_R8G8_UNORM: DXGI_FORMAT = 49;
pub const DXGI_FORMAT_R8_SINT: DXGI_FORMAT = 64;
pub const DXGI_FORMAT_R8_SNORM: DXGI_FORMAT = 63;
pub const DXGI_FORMAT_R8_TYPELESS: DXGI_FORMAT = 60;
pub const DXGI_FORMAT_R8_UINT: DXGI_FORMAT = 62;
pub const DXGI_FORMAT_R8_UNORM: DXGI_FORMAT = 61;
pub const DXGI_FORMAT_R9G9B9E5_SHAREDEXP: DXGI_FORMAT = 67;
pub const DXGI_FORMAT_SAMPLER_FEEDBACK_MIN_MIP_OPAQUE: DXGI_FORMAT = 189;
pub const DXGI_FORMAT_SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE: DXGI_FORMAT = 190;
pub const DXGI_FORMAT_UNKNOWN: DXGI_FORMAT = 0;
pub const DXGI_FORMAT_V208: DXGI_FORMAT = 131;
pub const DXGI_FORMAT_V408: DXGI_FORMAT = 132;
pub const DXGI_FORMAT_X24_TYPELESS_G8_UINT: DXGI_FORMAT = 47;
pub const DXGI_FORMAT_X32_TYPELESS_G8X24_UINT: DXGI_FORMAT = 22;
pub const DXGI_FORMAT_Y210: DXGI_FORMAT = 108;
pub const DXGI_FORMAT_Y216: DXGI_FORMAT = 109;
pub const DXGI_FORMAT_Y410: DXGI_FORMAT = 101;
pub const DXGI_FORMAT_Y416: DXGI_FORMAT = 102;
pub const DXGI_FORMAT_YUY2: DXGI_FORMAT = 107;
pub type DXGI_GPU_PREFERENCE = i32;
pub const DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE: DXGI_GPU_PREFERENCE = 2;
pub const DXGI_GPU_PREFERENCE_MINIMUM_POWER: DXGI_GPU_PREFERENCE = 1;
pub const DXGI_GPU_PREFERENCE_UNSPECIFIED: DXGI_GPU_PREFERENCE = 0;
pub type DXGI_MEMORY_SEGMENT_GROUP = i32;
pub const DXGI_MEMORY_SEGMENT_GROUP_LOCAL: DXGI_MEMORY_SEGMENT_GROUP = 0;
pub const DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL: DXGI_MEMORY_SEGMENT_GROUP = 1;
pub type DXGI_MODE_ROTATION = i32;
pub type DXGI_MODE_SCALING = i32;
pub type DXGI_MODE_SCANLINE_ORDER = i32;
pub const DXGI_MWA_NO_ALT_ENTER: i32 = 2;
pub const DXGI_MWA_NO_WINDOW_CHANGES: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DXGI_OUTPUT_DESC {
    pub DeviceName: [u16; 32],
    pub DesktopCoordinates: RECT,
    pub AttachedToDesktop: windows_core::BOOL,
    pub Rotation: DXGI_MODE_ROTATION,
    pub Monitor: HMONITOR,
}
impl Default for DXGI_OUTPUT_DESC {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const DXGI_PRESENT_ALLOW_TEARING: u32 = 512;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_QUERY_VIDEO_MEMORY_INFO {
    pub Budget: u64,
    pub CurrentUsage: u64,
    pub AvailableForReservation: u64,
    pub CurrentReservation: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_RATIONAL {
    pub Numerator: u32,
    pub Denominator: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SAMPLE_DESC {
    pub Count: u32,
    pub Quality: u32,
}
pub type DXGI_SCALING = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SWAP_CHAIN_DESC1 {
    pub Width: u32,
    pub Height: u32,
    pub Format: DXGI_FORMAT,
    pub Stereo: windows_core::BOOL,
    pub SampleDesc: DXGI_SAMPLE_DESC,
    pub BufferUsage: DXGI_USAGE,
    pub BufferCount: u32,
    pub Scaling: DXGI_SCALING,
    pub SwapEffect: DXGI_SWAP_EFFECT,
    pub AlphaMode: DXGI_ALPHA_MODE,
    pub Flags: u32,
}
pub type DXGI_SWAP_CHAIN_FLAG = i32;
pub const DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH: DXGI_SWAP_CHAIN_FLAG = 2;
pub const DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING: DXGI_SWAP_CHAIN_FLAG = 2048;
pub const DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY: DXGI_SWAP_CHAIN_FLAG = 32;
pub const DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER: DXGI_SWAP_CHAIN_FLAG = 128;
pub const DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT: DXGI_SWAP_CHAIN_FLAG = 64;
pub const DXGI_SWAP_CHAIN_FLAG_FULLSCREEN_VIDEO: DXGI_SWAP_CHAIN_FLAG = 256;
pub const DXGI_SWAP_CHAIN_FLAG_GDI_COMPATIBLE: DXGI_SWAP_CHAIN_FLAG = 4;
pub const DXGI_SWAP_CHAIN_FLAG_HW_PROTECTED: DXGI_SWAP_CHAIN_FLAG = 1024;
pub const DXGI_SWAP_CHAIN_FLAG_NONPREROTATED: DXGI_SWAP_CHAIN_FLAG = 1;
pub const DXGI_SWAP_CHAIN_FLAG_RESTRICTED_CONTENT: DXGI_SWAP_CHAIN_FLAG = 8;
pub const DXGI_SWAP_CHAIN_FLAG_RESTRICTED_TO_ALL_HOLOGRAPHIC_DISPLAYS: DXGI_SWAP_CHAIN_FLAG = 4096;
pub const DXGI_SWAP_CHAIN_FLAG_RESTRICT_SHARED_RESOURCE_DRIVER: DXGI_SWAP_CHAIN_FLAG = 16;
pub const DXGI_SWAP_CHAIN_FLAG_USE_DEFAULT_COLOR_SPACE: DXGI_SWAP_CHAIN_FLAG = 32768;
pub const DXGI_SWAP_CHAIN_FLAG_YUV_VIDEO: DXGI_SWAP_CHAIN_FLAG = 512;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SWAP_CHAIN_FULLSCREEN_DESC {
    pub RefreshRate: DXGI_RATIONAL,
    pub ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER,
    pub Scaling: DXGI_MODE_SCALING,
    pub Windowed: windows_core::BOOL,
}
pub type DXGI_SWAP_EFFECT = i32;
pub const DXGI_SWAP_EFFECT_DISCARD: DXGI_SWAP_EFFECT = 0;
pub const DXGI_SWAP_EFFECT_FLIP_DISCARD: DXGI_SWAP_EFFECT = 4;
pub const DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL: DXGI_SWAP_EFFECT = 3;
pub const DXGI_SWAP_EFFECT_SEQUENTIAL: DXGI_SWAP_EFFECT = 1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DXGI_USAGE(pub u32);
pub const DXGI_USAGE_RENDER_TARGET_OUTPUT: u32 = 32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ENABLE_TRACE_PARAMETERS {
    pub Version: u32,
    pub EnableProperty: u32,
    pub ControlFlags: u32,
    pub SourceId: windows_core::GUID,
    pub EnableFilterDesc: PEVENT_FILTER_DESCRIPTOR,
    pub FilterDescCount: u32,
}
pub const ENABLE_TRACE_PARAMETERS_VERSION_2: i32 = 2;
pub const ERROR_ACCESS_DENIED: i32 = 5;
pub const ERROR_ALREADY_EXISTS: i32 = 183;
pub const ERROR_INSUFFICIENT_BUFFER: i32 = 122;
pub const ERROR_MORE_DATA: i32 = 234;
pub const ERROR_SUCCESS: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ETW_BUFFER_CONTEXT {
    pub Anonymous: ETW_BUFFER_CONTEXT_0,
    pub LoggerId: u16,
}
impl Default for ETW_BUFFER_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ETW_BUFFER_CONTEXT_0 {
    pub Anonymous: ETW_BUFFER_CONTEXT_0_0,
    pub ProcessorIndex: u16,
}
impl Default for ETW_BUFFER_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ETW_BUFFER_CONTEXT_0_0 {
    pub ProcessorNumber: u8,
    pub Alignment: u8,
}
pub const EVENT_CONTROL_CODE_DISABLE_PROVIDER: i32 = 0;
pub const EVENT_CONTROL_CODE_ENABLE_PROVIDER: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_DESCRIPTOR {
    pub Id: u16,
    pub Version: u8,
    pub Channel: u8,
    pub Level: u8,
    pub Opcode: u8,
    pub Task: u16,
    pub Keyword: u64,
}
pub const EVENT_ENABLE_PROPERTY_IGNORE_KEYWORD_0: i32 = 16;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_FILTER_DESCRIPTOR {
    pub Ptr: u64,
    pub Size: u32,
    pub Type: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EVENT_FILTER_EVENT_ID {
    pub FilterIn: bool,
    pub Reserved: u8,
    pub Count: u16,
    pub Events: [u16; 1],
}
impl Default for EVENT_FILTER_EVENT_ID {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const EVENT_FILTER_TYPE_EVENT_ID: u32 = 2147484160;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_HEADER {
    pub Size: u16,
    pub HeaderType: u16,
    pub Flags: u16,
    pub EventProperty: u16,
    pub ThreadId: u32,
    pub ProcessId: u32,
    pub TimeStamp: i64,
    pub ProviderId: windows_core::GUID,
    pub EventDescriptor: EVENT_DESCRIPTOR,
    pub Anonymous: EVENT_HEADER_0,
    pub ActivityId: windows_core::GUID,
}
impl Default for EVENT_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_HEADER_0 {
    pub Anonymous: EVENT_HEADER_0_0,
    pub ProcessorTime: u64,
}
impl Default for EVENT_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_0_0 {
    pub KernelTime: u32,
    pub UserTime: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_EXTENDED_DATA_ITEM {
    pub Reserved1: u16,
    pub ExtType: u16,
    pub Anonymous: EVENT_HEADER_EXTENDED_DATA_ITEM_0,
    pub DataSize: u16,
    pub DataPtr: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_EXTENDED_DATA_ITEM_0 {
    pub _bitfield: u16,
}
impl EVENT_HEADER_EXTENDED_DATA_ITEM_0 {
    pub fn Linkage(&self) -> bool {
        self._bitfield & 1 != 0
    }
    pub fn set_Linkage(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !1) | (value as u16);
    }
    pub fn Reserved2(&self) -> u16 {
        self._bitfield >> 1
    }
    pub fn set_Reserved2(&mut self, value: u16) {
        self._bitfield = (self._bitfield & !(32767 << 1)) | ((value & 32767) << 1);
    }
}
pub const EVENT_HEADER_FLAG_64_BIT_HEADER: i32 = 64;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_PROPERTY_INFO {
    pub Flags: PROPERTY_FLAGS,
    pub NameOffset: u32,
    pub Anonymous: EVENT_PROPERTY_INFO_0,
    pub Anonymous2: EVENT_PROPERTY_INFO_1,
    pub Anonymous3: EVENT_PROPERTY_INFO_2,
    pub Anonymous4: EVENT_PROPERTY_INFO_3,
}
impl Default for EVENT_PROPERTY_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_0 {
    pub nonStructType: EVENT_PROPERTY_INFO_0_0,
    pub structType: EVENT_PROPERTY_INFO_0_1,
    pub customSchemaType: EVENT_PROPERTY_INFO_0_2,
}
impl Default for EVENT_PROPERTY_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_1 {
    pub count: u16,
    pub countPropertyIndex: u16,
}
impl Default for EVENT_PROPERTY_INFO_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_2 {
    pub length: u16,
    pub lengthPropertyIndex: u16,
}
impl Default for EVENT_PROPERTY_INFO_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_3 {
    pub Reserved: u32,
    pub Anonymous: EVENT_PROPERTY_INFO_3_0,
}
impl Default for EVENT_PROPERTY_INFO_3 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_3_0 {
    pub _bitfield: u32,
}
impl EVENT_PROPERTY_INFO_3_0 {
    pub fn Tags(&self) -> u32 {
        (self._bitfield << 4) >> 4
    }
    pub fn set_Tags(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !268435455) | (value & 268435455);
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_0 {
    pub InType: u16,
    pub OutType: u16,
    pub MapNameOffset: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_1 {
    pub StructStartIndex: u16,
    pub NumOfStructMembers: u16,
    pub padding: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_2 {
    pub InType: u16,
    pub OutType: u16,
    pub CustomSchemaOffset: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_RECORD {
    pub EventHeader: EVENT_HEADER,
    pub BufferContext: ETW_BUFFER_CONTEXT,
    pub ExtendedDataCount: u16,
    pub UserDataLength: u16,
    pub ExtendedData: PEVENT_HEADER_EXTENDED_DATA_ITEM,
    pub UserData: *mut core::ffi::c_void,
    pub UserContext: *mut core::ffi::c_void,
}
impl Default for EVENT_RECORD {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE {
    pub Header: EVENT_TRACE_HEADER,
    pub InstanceId: u32,
    pub ParentInstanceId: u32,
    pub ParentGuid: windows_core::GUID,
    pub MofData: *mut core::ffi::c_void,
    pub MofLength: u32,
    pub Anonymous: EVENT_TRACE_0,
}
impl Default for EVENT_TRACE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_0 {
    pub ClientContext: u32,
    pub BufferContext: ETW_BUFFER_CONTEXT,
}
impl Default for EVENT_TRACE_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const EVENT_TRACE_CONTROL_FLUSH: i32 = 3;
pub const EVENT_TRACE_CONTROL_STOP: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_HEADER {
    pub Size: u16,
    pub Anonymous: EVENT_TRACE_HEADER_0,
    pub Anonymous2: EVENT_TRACE_HEADER_1,
    pub ThreadId: u32,
    pub ProcessId: u32,
    pub TimeStamp: i64,
    pub Anonymous3: EVENT_TRACE_HEADER_2,
    pub Anonymous4: EVENT_TRACE_HEADER_3,
}
impl Default for EVENT_TRACE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_0 {
    pub FieldTypeFlags: u16,
    pub Anonymous: EVENT_TRACE_HEADER_0_0,
}
impl Default for EVENT_TRACE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_0_0 {
    pub HeaderType: u8,
    pub MarkerFlags: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_1 {
    pub Version: u32,
    pub Class: EVENT_TRACE_HEADER_1_0,
}
impl Default for EVENT_TRACE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_1_0 {
    pub Type: u8,
    pub Level: u8,
    pub Version: u16,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_2 {
    pub Guid: windows_core::GUID,
    pub GuidPtr: u64,
}
impl Default for EVENT_TRACE_HEADER_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_3 {
    pub Anonymous: EVENT_TRACE_HEADER_3_0,
    pub ProcessorTime: u64,
    pub Anonymous2: EVENT_TRACE_HEADER_3_1,
}
impl Default for EVENT_TRACE_HEADER_3 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_3_0 {
    pub KernelTime: u32,
    pub UserTime: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_3_1 {
    pub ClientContext: u32,
    pub Flags: u32,
}
pub const EVENT_TRACE_INDEPENDENT_SESSION_MODE: i32 = 134217728;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_LOGFILEW {
    pub LogFileName: windows_core::PWSTR,
    pub LoggerName: windows_core::PWSTR,
    pub CurrentTime: i64,
    pub BuffersRead: u32,
    pub Anonymous: EVENT_TRACE_LOGFILEW_0,
    pub CurrentEvent: EVENT_TRACE,
    pub LogfileHeader: TRACE_LOGFILE_HEADER,
    pub BufferCallback: PEVENT_TRACE_BUFFER_CALLBACKW,
    pub BufferSize: u32,
    pub Filled: u32,
    pub EventsLost: u32,
    pub Anonymous2: EVENT_TRACE_LOGFILEW_1,
    pub IsKernelTrace: u32,
    pub Context: *mut core::ffi::c_void,
}
impl Default for EVENT_TRACE_LOGFILEW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_LOGFILEW_0 {
    pub LogFileMode: u32,
    pub ProcessTraceMode: u32,
}
impl Default for EVENT_TRACE_LOGFILEW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_LOGFILEW_1 {
    pub EventCallback: PEVENT_CALLBACK,
    pub EventRecordCallback: PEVENT_RECORD_CALLBACK,
}
impl Default for EVENT_TRACE_LOGFILEW_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_PROPERTIES {
    pub Wnode: WNODE_HEADER,
    pub BufferSize: u32,
    pub MinimumBuffers: u32,
    pub MaximumBuffers: u32,
    pub MaximumFileSize: u32,
    pub LogFileMode: u32,
    pub FlushTimer: u32,
    pub EnableFlags: u32,
    pub Anonymous: EVENT_TRACE_PROPERTIES_0,
    pub NumberOfBuffers: u32,
    pub FreeBuffers: u32,
    pub EventsLost: u32,
    pub BuffersWritten: u32,
    pub LogBuffersLost: u32,
    pub RealTimeBuffersLost: u32,
    pub LoggerThreadId: HANDLE,
    pub LogFileNameOffset: u32,
    pub LoggerNameOffset: u32,
}
impl Default for EVENT_TRACE_PROPERTIES {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_PROPERTIES_0 {
    pub AgeLimit: i32,
    pub FlushThreshold: i32,
}
impl Default for EVENT_TRACE_PROPERTIES_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const EVENT_TRACE_REAL_TIME_MODE: i32 = 256;
pub const E_FAIL: windows_core::HRESULT = windows_core::HRESULT(0x80004005_u32 as _);
pub const E_INVALIDARG: windows_core::HRESULT = windows_core::HRESULT(0x80070057_u32 as _);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}
pub const GENERIC_WRITE: i32 = 1073741824;
pub const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: i32 = 4;
pub const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: i32 = 2;
pub const GUID_ContainerFormatPng: windows_core::GUID =
    windows_core::GUID::from_u128(0x1b7cfaf4_713f_473c_bbcd_6137425faeaf);
pub const GUID_WICPixelFormat24bppBGR: windows_core::GUID =
    windows_core::GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90c);
pub const GWLP_USERDATA: i32 = -21;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWL_STYLE: i32 = -16;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HBITMAP(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HBRUSH(pub *mut core::ffi::c_void);
pub type HCURSOR = HICON;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HDC(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HGDIOBJ(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HICON(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HINSTANCE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HKL(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMENU(pub *mut core::ffi::c_void);
pub type HMODULE = HINSTANCE;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMONITOR(pub *mut core::ffi::c_void);
pub const HTCLIENT: i32 = 1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
pub const HWND_BOTTOM: HWND = HWND(1 as _);
windows_core::imp::define_interface!(
    ID3D10Blob,
    ID3D10Blob_Vtbl,
    0x8ba5fb08_5195_40e2_ac58_0d989c3a0102
);
windows_core::imp::interface_hierarchy!(ID3D10Blob, windows_core::IUnknown);
#[repr(C)]
pub struct ID3D10Blob_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetBufferPointer: usize,
    GetBufferSize: usize,
}
impl windows_core::RuntimeName for ID3D10Blob {}
windows_core::imp::define_interface!(
    ID3D12CommandAllocator,
    ID3D12CommandAllocator_Vtbl,
    0x6102dee4_af59_4b09_b999_b44d73f09b24
);
impl core::ops::Deref for ID3D12CommandAllocator {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12CommandAllocator,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12CommandAllocator {
    pub unsafe fn Reset(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Reset)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct ID3D12CommandAllocator_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub Reset: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for ID3D12CommandAllocator {}
windows_core::imp::define_interface!(
    ID3D12CommandList,
    ID3D12CommandList_Vtbl,
    0x7116d91c_e7e4_47ce_b8c6_ec8168f437e5
);
impl core::ops::Deref for ID3D12CommandList {
    type Target = ID3D12DeviceChild;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12CommandList,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild
);
impl ID3D12CommandList {
    pub unsafe fn GetType(&self) -> D3D12_COMMAND_LIST_TYPE {
        unsafe {
            (windows_core::Interface::vtable(self).GetType)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct ID3D12CommandList_Vtbl {
    pub base__: ID3D12DeviceChild_Vtbl,
    pub GetType: unsafe extern "system" fn(*mut core::ffi::c_void) -> D3D12_COMMAND_LIST_TYPE,
}
impl windows_core::RuntimeName for ID3D12CommandList {}
windows_core::imp::define_interface!(
    ID3D12CommandQueue,
    ID3D12CommandQueue_Vtbl,
    0x0ec870a6_5d7e_4c22_8cfc_5baae07616ed
);
impl core::ops::Deref for ID3D12CommandQueue {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12CommandQueue,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12CommandQueue {
    pub unsafe fn UpdateTileMappings<P0, P4>(
        &self,
        presource: P0,
        numresourceregions: u32,
        presourceregionstartcoordinates: Option<*const D3D12_TILED_RESOURCE_COORDINATE>,
        presourceregionsizes: Option<*const D3D12_TILE_REGION_SIZE>,
        pheap: P4,
        numranges: u32,
        prangeflags: Option<*const D3D12_TILE_RANGE_FLAGS>,
        pheaprangestartoffsets: Option<*const u32>,
        prangetilecounts: Option<*const u32>,
        flags: D3D12_TILE_MAPPING_FLAGS,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P4: windows_core::Param<ID3D12Heap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).UpdateTileMappings)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                numresourceregions,
                presourceregionstartcoordinates.unwrap_or(core::mem::zeroed()) as _,
                presourceregionsizes.unwrap_or(core::mem::zeroed()) as _,
                pheap.param().abi(),
                numranges,
                prangeflags.unwrap_or(core::mem::zeroed()) as _,
                pheaprangestartoffsets.unwrap_or(core::mem::zeroed()) as _,
                prangetilecounts.unwrap_or(core::mem::zeroed()) as _,
                flags,
            );
        }
    }
    pub unsafe fn CopyTileMappings<P0, P2>(
        &self,
        pdstresource: P0,
        pdstregionstartcoordinate: *const D3D12_TILED_RESOURCE_COORDINATE,
        psrcresource: P2,
        psrcregionstartcoordinate: *const D3D12_TILED_RESOURCE_COORDINATE,
        pregionsize: *const D3D12_TILE_REGION_SIZE,
        flags: D3D12_TILE_MAPPING_FLAGS,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P2: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyTileMappings)(
                windows_core::Interface::as_raw(self),
                pdstresource.param().abi(),
                pdstregionstartcoordinate,
                psrcresource.param().abi(),
                psrcregionstartcoordinate,
                pregionsize,
                flags,
            );
        }
    }
    pub unsafe fn ExecuteCommandLists(&self, ppcommandlists: &[Option<ID3D12CommandList>]) {
        unsafe {
            (windows_core::Interface::vtable(self).ExecuteCommandLists)(
                windows_core::Interface::as_raw(self),
                ppcommandlists.len().try_into().unwrap(),
                core::mem::transmute(ppcommandlists.as_ptr()),
            );
        }
    }
    pub unsafe fn SetMarker(
        &self,
        metadata: u32,
        pdata: Option<*const core::ffi::c_void>,
        size: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetMarker)(
                windows_core::Interface::as_raw(self),
                metadata,
                pdata.unwrap_or(core::mem::zeroed()) as _,
                size,
            );
        }
    }
    pub unsafe fn BeginEvent(
        &self,
        metadata: u32,
        pdata: Option<*const core::ffi::c_void>,
        size: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).BeginEvent)(
                windows_core::Interface::as_raw(self),
                metadata,
                pdata.unwrap_or(core::mem::zeroed()) as _,
                size,
            );
        }
    }
    pub unsafe fn EndEvent(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).EndEvent)(windows_core::Interface::as_raw(self));
        }
    }
    pub unsafe fn Signal<P0>(&self, pfence: P0, value: u64) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID3D12Fence>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Signal)(
                windows_core::Interface::as_raw(self),
                pfence.param().abi(),
                value,
            )
        }
    }
    pub unsafe fn Wait<P0>(&self, pfence: P0, value: u64) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID3D12Fence>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Wait)(
                windows_core::Interface::as_raw(self),
                pfence.param().abi(),
                value,
            )
        }
    }
    pub unsafe fn GetTimestampFrequency(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTimestampFrequency)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn GetClockCalibration(
        &self,
        pgputimestamp: *mut u64,
        pcputimestamp: *mut u64,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetClockCalibration)(
                windows_core::Interface::as_raw(self),
                pgputimestamp as _,
                pcputimestamp as _,
            )
        }
    }
    pub unsafe fn GetDesc(&self) -> D3D12_COMMAND_QUEUE_DESC {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
}
#[repr(C)]
pub struct ID3D12CommandQueue_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub UpdateTileMappings: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        *const D3D12_TILED_RESOURCE_COORDINATE,
        *const D3D12_TILE_REGION_SIZE,
        *mut core::ffi::c_void,
        u32,
        *const D3D12_TILE_RANGE_FLAGS,
        *const u32,
        *const u32,
        D3D12_TILE_MAPPING_FLAGS,
    ),
    pub CopyTileMappings: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_TILED_RESOURCE_COORDINATE,
        *mut core::ffi::c_void,
        *const D3D12_TILED_RESOURCE_COORDINATE,
        *const D3D12_TILE_REGION_SIZE,
        D3D12_TILE_MAPPING_FLAGS,
    ),
    pub ExecuteCommandLists:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const *mut core::ffi::c_void),
    pub SetMarker:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const core::ffi::c_void, u32),
    pub BeginEvent:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const core::ffi::c_void, u32),
    pub EndEvent: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Signal: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u64,
    ) -> windows_core::HRESULT,
    pub Wait: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u64,
    ) -> windows_core::HRESULT,
    pub GetTimestampFrequency:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub GetClockCalibration: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u64,
        *mut u64,
    ) -> windows_core::HRESULT,
    pub GetDesc: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D12_COMMAND_QUEUE_DESC),
}
impl windows_core::RuntimeName for ID3D12CommandQueue {}
windows_core::imp::define_interface!(
    ID3D12CommandSignature,
    ID3D12CommandSignature_Vtbl,
    0xc36a797c_ec80_4f0a_8985_a7b2475082d1
);
impl core::ops::Deref for ID3D12CommandSignature {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12CommandSignature,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
#[repr(C)]
pub struct ID3D12CommandSignature_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
}
impl windows_core::RuntimeName for ID3D12CommandSignature {}
windows_core::imp::define_interface!(
    ID3D12Debug,
    ID3D12Debug_Vtbl,
    0x344488b7_6846_474b_b989_f027448245e0
);
windows_core::imp::interface_hierarchy!(ID3D12Debug, windows_core::IUnknown);
impl ID3D12Debug {
    pub unsafe fn EnableDebugLayer(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).EnableDebugLayer)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
}
#[repr(C)]
pub struct ID3D12Debug_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub EnableDebugLayer: unsafe extern "system" fn(*mut core::ffi::c_void),
}
pub trait ID3D12Debug_Impl: windows_core::IUnknownImpl {
    fn EnableDebugLayer(&self);
}
impl ID3D12Debug_Vtbl {
    pub const fn new<Identity: ID3D12Debug_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn EnableDebugLayer<
            Identity: ID3D12Debug_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID3D12Debug_Impl::EnableDebugLayer(this);
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            EnableDebugLayer: EnableDebugLayer::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ID3D12Debug as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for ID3D12Debug {}
windows_core::imp::define_interface!(
    ID3D12DescriptorHeap,
    ID3D12DescriptorHeap_Vtbl,
    0x8efb471d_616c_4f49_90f7_127bb763fa51
);
impl core::ops::Deref for ID3D12DescriptorHeap {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12DescriptorHeap,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12DescriptorHeap {
    pub unsafe fn GetDesc(&self) -> D3D12_DESCRIPTOR_HEAP_DESC {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetCPUDescriptorHandleForHeapStart(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCPUDescriptorHandleForHeapStart)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetGPUDescriptorHandleForHeapStart(&self) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetGPUDescriptorHandleForHeapStart)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
}
#[repr(C)]
pub struct ID3D12DescriptorHeap_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub GetDesc: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D12_DESCRIPTOR_HEAP_DESC),
    pub GetCPUDescriptorHandleForHeapStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D12_CPU_DESCRIPTOR_HANDLE),
    pub GetGPUDescriptorHandleForHeapStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D12_GPU_DESCRIPTOR_HANDLE),
}
impl windows_core::RuntimeName for ID3D12DescriptorHeap {}
windows_core::imp::define_interface!(
    ID3D12Device,
    ID3D12Device_Vtbl,
    0x189819f1_1db6_4b57_be54_1821339b85f7
);
impl core::ops::Deref for ID3D12Device {
    type Target = ID3D12Object;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID3D12Device, windows_core::IUnknown, ID3D12Object);
impl ID3D12Device {
    pub unsafe fn GetNodeCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetNodeCount)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn CreateCommandQueue<T>(
        &self,
        pdesc: *const D3D12_COMMAND_QUEUE_DESC,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateCommandQueue)(
                windows_core::Interface::as_raw(self),
                pdesc,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateCommandAllocator<T>(
        &self,
        r#type: D3D12_COMMAND_LIST_TYPE,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateCommandAllocator)(
                windows_core::Interface::as_raw(self),
                r#type,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateGraphicsPipelineState<T>(
        &self,
        pdesc: *const D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateGraphicsPipelineState)(
                windows_core::Interface::as_raw(self),
                pdesc,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateComputePipelineState<T>(
        &self,
        pdesc: *const D3D12_COMPUTE_PIPELINE_STATE_DESC,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateComputePipelineState)(
                windows_core::Interface::as_raw(self),
                pdesc,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateCommandList<P2, P3, T>(
        &self,
        nodemask: u32,
        r#type: D3D12_COMMAND_LIST_TYPE,
        pcommandallocator: P2,
        pinitialstate: P3,
    ) -> windows_core::Result<T>
    where
        P2: windows_core::Param<ID3D12CommandAllocator>,
        P3: windows_core::Param<ID3D12PipelineState>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateCommandList)(
                windows_core::Interface::as_raw(self),
                nodemask,
                r#type,
                pcommandallocator.param().abi(),
                pinitialstate.param().abi(),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CheckFeatureSupport(
        &self,
        feature: D3D12_FEATURE,
        pfeaturesupportdata: *mut core::ffi::c_void,
        featuresupportdatasize: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CheckFeatureSupport)(
                windows_core::Interface::as_raw(self),
                feature,
                pfeaturesupportdata as _,
                featuresupportdatasize,
            )
        }
    }
    pub unsafe fn CreateDescriptorHeap<T>(
        &self,
        pdescriptorheapdesc: *const D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateDescriptorHeap)(
                windows_core::Interface::as_raw(self),
                pdescriptorheapdesc,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetDescriptorHandleIncrementSize(
        &self,
        descriptorheaptype: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetDescriptorHandleIncrementSize)(
                windows_core::Interface::as_raw(self),
                descriptorheaptype,
            )
        }
    }
    pub unsafe fn CreateRootSignature<T>(
        &self,
        nodemask: u32,
        pblobwithrootsignature: &[u8],
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateRootSignature)(
                windows_core::Interface::as_raw(self),
                nodemask,
                core::mem::transmute(pblobwithrootsignature.as_ptr()),
                pblobwithrootsignature.len().try_into().unwrap(),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateConstantBufferView(
        &self,
        pdesc: Option<*const D3D12_CONSTANT_BUFFER_VIEW_DESC>,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).CreateConstantBufferView)(
                windows_core::Interface::as_raw(self),
                pdesc.unwrap_or(core::mem::zeroed()) as _,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CreateShaderResourceView<P0>(
        &self,
        presource: P0,
        pdesc: Option<*const D3D12_SHADER_RESOURCE_VIEW_DESC>,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateShaderResourceView)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                pdesc.unwrap_or(core::mem::zeroed()) as _,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CreateUnorderedAccessView<P0, P1>(
        &self,
        presource: P0,
        pcounterresource: P1,
        pdesc: Option<*const D3D12_UNORDERED_ACCESS_VIEW_DESC>,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P1: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateUnorderedAccessView)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                pcounterresource.param().abi(),
                pdesc.unwrap_or(core::mem::zeroed()) as _,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CreateRenderTargetView<P0>(
        &self,
        presource: P0,
        pdesc: Option<*const D3D12_RENDER_TARGET_VIEW_DESC>,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateRenderTargetView)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                pdesc.unwrap_or(core::mem::zeroed()) as _,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CreateDepthStencilView<P0>(
        &self,
        presource: P0,
        pdesc: Option<*const D3D12_DEPTH_STENCIL_VIEW_DESC>,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateDepthStencilView)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                pdesc.unwrap_or(core::mem::zeroed()) as _,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CreateSampler(
        &self,
        pdesc: *const D3D12_SAMPLER_DESC,
        destdescriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).CreateSampler)(
                windows_core::Interface::as_raw(self),
                pdesc,
                destdescriptor,
            );
        }
    }
    pub unsafe fn CopyDescriptors(
        &self,
        numdestdescriptorranges: u32,
        pdestdescriptorrangestarts: *const D3D12_CPU_DESCRIPTOR_HANDLE,
        pdestdescriptorrangesizes: Option<*const u32>,
        numsrcdescriptorranges: u32,
        psrcdescriptorrangestarts: *const D3D12_CPU_DESCRIPTOR_HANDLE,
        psrcdescriptorrangesizes: Option<*const u32>,
        descriptorheapstype: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).CopyDescriptors)(
                windows_core::Interface::as_raw(self),
                numdestdescriptorranges,
                pdestdescriptorrangestarts,
                pdestdescriptorrangesizes.unwrap_or(core::mem::zeroed()) as _,
                numsrcdescriptorranges,
                psrcdescriptorrangestarts,
                psrcdescriptorrangesizes.unwrap_or(core::mem::zeroed()) as _,
                descriptorheapstype,
            );
        }
    }
    pub unsafe fn CopyDescriptorsSimple(
        &self,
        numdescriptors: u32,
        destdescriptorrangestart: D3D12_CPU_DESCRIPTOR_HANDLE,
        srcdescriptorrangestart: D3D12_CPU_DESCRIPTOR_HANDLE,
        descriptorheapstype: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).CopyDescriptorsSimple)(
                windows_core::Interface::as_raw(self),
                numdescriptors,
                destdescriptorrangestart,
                srcdescriptorrangestart,
                descriptorheapstype,
            );
        }
    }
    pub unsafe fn GetResourceAllocationInfo(
        &self,
        visiblemask: u32,
        presourcedescs: &[D3D12_RESOURCE_DESC],
    ) -> D3D12_RESOURCE_ALLOCATION_INFO {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetResourceAllocationInfo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
                visiblemask,
                presourcedescs.len().try_into().unwrap(),
                presourcedescs.as_ptr(),
            );
            result__
        }
    }
    pub unsafe fn GetCustomHeapProperties(
        &self,
        nodemask: u32,
        heaptype: D3D12_HEAP_TYPE,
    ) -> D3D12_HEAP_PROPERTIES {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCustomHeapProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
                nodemask,
                heaptype,
            );
            result__
        }
    }
    pub unsafe fn CreateCommittedResource<T>(
        &self,
        pheapproperties: *const D3D12_HEAP_PROPERTIES,
        heapflags: D3D12_HEAP_FLAGS,
        pdesc: *const D3D12_RESOURCE_DESC,
        initialresourcestate: D3D12_RESOURCE_STATES,
        poptimizedclearvalue: Option<*const D3D12_CLEAR_VALUE>,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateCommittedResource)(
                windows_core::Interface::as_raw(self),
                pheapproperties,
                heapflags,
                pdesc,
                initialresourcestate,
                poptimizedclearvalue.unwrap_or(core::mem::zeroed()) as _,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn CreateHeap<T>(
        &self,
        pdesc: *const D3D12_HEAP_DESC,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateHeap)(
                windows_core::Interface::as_raw(self),
                pdesc,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn CreatePlacedResource<P0, T>(
        &self,
        pheap: P0,
        heapoffset: u64,
        pdesc: *const D3D12_RESOURCE_DESC,
        initialstate: D3D12_RESOURCE_STATES,
        poptimizedclearvalue: Option<*const D3D12_CLEAR_VALUE>,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ID3D12Heap>,
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreatePlacedResource)(
                windows_core::Interface::as_raw(self),
                pheap.param().abi(),
                heapoffset,
                pdesc,
                initialstate,
                poptimizedclearvalue.unwrap_or(core::mem::zeroed()) as _,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn CreateReservedResource<T>(
        &self,
        pdesc: *const D3D12_RESOURCE_DESC,
        initialstate: D3D12_RESOURCE_STATES,
        poptimizedclearvalue: Option<*const D3D12_CLEAR_VALUE>,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateReservedResource)(
                windows_core::Interface::as_raw(self),
                pdesc,
                initialstate,
                poptimizedclearvalue.unwrap_or(core::mem::zeroed()) as _,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn CreateSharedHandle<P0, P3>(
        &self,
        pobject: P0,
        pattributes: Option<*const SECURITY_ATTRIBUTES>,
        access: u32,
        name: P3,
    ) -> windows_core::Result<HANDLE>
    where
        P0: windows_core::Param<ID3D12DeviceChild>,
        P3: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSharedHandle)(
                windows_core::Interface::as_raw(self),
                pobject.param().abi(),
                pattributes.unwrap_or(core::mem::zeroed()) as _,
                access,
                name.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn OpenSharedHandle<T>(
        &self,
        nthandle: HANDLE,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OpenSharedHandle)(
                windows_core::Interface::as_raw(self),
                nthandle,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn OpenSharedHandleByName<P0>(
        &self,
        name: P0,
        access: u32,
    ) -> windows_core::Result<HANDLE>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).OpenSharedHandleByName)(
                windows_core::Interface::as_raw(self),
                name.param().abi(),
                access,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn MakeResident(
        &self,
        ppobjects: &[Option<ID3D12Pageable>],
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).MakeResident)(
                windows_core::Interface::as_raw(self),
                ppobjects.len().try_into().unwrap(),
                core::mem::transmute(ppobjects.as_ptr()),
            )
        }
    }
    pub unsafe fn Evict(&self, ppobjects: &[Option<ID3D12Pageable>]) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Evict)(
                windows_core::Interface::as_raw(self),
                ppobjects.len().try_into().unwrap(),
                core::mem::transmute(ppobjects.as_ptr()),
            )
        }
    }
    pub unsafe fn CreateFence<T>(
        &self,
        initialvalue: u64,
        flags: D3D12_FENCE_FLAGS,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateFence)(
                windows_core::Interface::as_raw(self),
                initialvalue,
                flags,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetDeviceRemovedReason(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDeviceRemovedReason)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetCopyableFootprints(
        &self,
        presourcedesc: *const D3D12_RESOURCE_DESC,
        firstsubresource: u32,
        numsubresources: u32,
        baseoffset: u64,
        playouts: Option<*mut D3D12_PLACED_SUBRESOURCE_FOOTPRINT>,
        pnumrows: Option<*mut u32>,
        prowsizeinbytes: Option<*mut u64>,
        ptotalbytes: Option<*mut u64>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).GetCopyableFootprints)(
                windows_core::Interface::as_raw(self),
                presourcedesc,
                firstsubresource,
                numsubresources,
                baseoffset,
                playouts.unwrap_or(core::mem::zeroed()) as _,
                pnumrows.unwrap_or(core::mem::zeroed()) as _,
                prowsizeinbytes.unwrap_or(core::mem::zeroed()) as _,
                ptotalbytes.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn CreateQueryHeap<T>(
        &self,
        pdesc: *const D3D12_QUERY_HEAP_DESC,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateQueryHeap)(
                windows_core::Interface::as_raw(self),
                pdesc,
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn SetStablePowerState(&self, enable: bool) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetStablePowerState)(
                windows_core::Interface::as_raw(self),
                enable.into(),
            )
        }
    }
    pub unsafe fn CreateCommandSignature<P1, T>(
        &self,
        pdesc: *const D3D12_COMMAND_SIGNATURE_DESC,
        prootsignature: P1,
        result__: *mut Option<T>,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<ID3D12RootSignature>,
        T: windows_core::Interface,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CreateCommandSignature)(
                windows_core::Interface::as_raw(self),
                pdesc,
                prootsignature.param().abi(),
                &T::IID,
                result__ as *mut _ as *mut _,
            )
            .ok()
        }
    }
    pub unsafe fn GetResourceTiling<P0>(
        &self,
        ptiledresource: P0,
        pnumtilesforentireresource: Option<*mut u32>,
        ppackedmipdesc: Option<*mut D3D12_PACKED_MIP_INFO>,
        pstandardtileshapefornonpackedmips: Option<*mut D3D12_TILE_SHAPE>,
        pnumsubresourcetilings: Option<*mut u32>,
        firstsubresourcetilingtoget: u32,
        psubresourcetilingsfornonpackedmips: *mut D3D12_SUBRESOURCE_TILING,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetResourceTiling)(
                windows_core::Interface::as_raw(self),
                ptiledresource.param().abi(),
                pnumtilesforentireresource.unwrap_or(core::mem::zeroed()) as _,
                ppackedmipdesc.unwrap_or(core::mem::zeroed()) as _,
                pstandardtileshapefornonpackedmips.unwrap_or(core::mem::zeroed()) as _,
                pnumsubresourcetilings.unwrap_or(core::mem::zeroed()) as _,
                firstsubresourcetilingtoget,
                psubresourcetilingsfornonpackedmips as _,
            );
        }
    }
    pub unsafe fn GetAdapterLuid(&self) -> LUID {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAdapterLuid)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
}
#[repr(C)]
pub struct ID3D12Device_Vtbl {
    pub base__: ID3D12Object_Vtbl,
    pub GetNodeCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub CreateCommandQueue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_COMMAND_QUEUE_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCommandAllocator: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_COMMAND_LIST_TYPE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGraphicsPipelineState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_GRAPHICS_PIPELINE_STATE_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateComputePipelineState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_COMPUTE_PIPELINE_STATE_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCommandList: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        D3D12_COMMAND_LIST_TYPE,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CheckFeatureSupport: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_FEATURE,
        *mut core::ffi::c_void,
        u32,
    ) -> windows_core::HRESULT,
    pub CreateDescriptorHeap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_DESCRIPTOR_HEAP_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetDescriptorHandleIncrementSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, D3D12_DESCRIPTOR_HEAP_TYPE) -> u32,
    pub CreateRootSignature: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const core::ffi::c_void,
        usize,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateConstantBufferView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_CONSTANT_BUFFER_VIEW_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CreateShaderResourceView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_SHADER_RESOURCE_VIEW_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CreateUnorderedAccessView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_UNORDERED_ACCESS_VIEW_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CreateRenderTargetView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_RENDER_TARGET_VIEW_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CreateDepthStencilView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_DEPTH_STENCIL_VIEW_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CreateSampler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_SAMPLER_DESC,
        D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub CopyDescriptors: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const D3D12_CPU_DESCRIPTOR_HANDLE,
        *const u32,
        u32,
        *const D3D12_CPU_DESCRIPTOR_HANDLE,
        *const u32,
        D3D12_DESCRIPTOR_HEAP_TYPE,
    ),
    pub CopyDescriptorsSimple: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        D3D12_DESCRIPTOR_HEAP_TYPE,
    ),
    pub GetResourceAllocationInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D3D12_RESOURCE_ALLOCATION_INFO,
        u32,
        u32,
        *const D3D12_RESOURCE_DESC,
    ),
    pub GetCustomHeapProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D3D12_HEAP_PROPERTIES,
        u32,
        D3D12_HEAP_TYPE,
    ),
    pub CreateCommittedResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_HEAP_PROPERTIES,
        D3D12_HEAP_FLAGS,
        *const D3D12_RESOURCE_DESC,
        D3D12_RESOURCE_STATES,
        *const D3D12_CLEAR_VALUE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateHeap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_HEAP_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePlacedResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u64,
        *const D3D12_RESOURCE_DESC,
        D3D12_RESOURCE_STATES,
        *const D3D12_CLEAR_VALUE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateReservedResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_RESOURCE_DESC,
        D3D12_RESOURCE_STATES,
        *const D3D12_CLEAR_VALUE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSharedHandle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const SECURITY_ATTRIBUTES,
        u32,
        windows_core::PCWSTR,
        *mut HANDLE,
    ) -> windows_core::HRESULT,
    pub OpenSharedHandle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HANDLE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub OpenSharedHandleByName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        u32,
        *mut HANDLE,
    ) -> windows_core::HRESULT,
    pub MakeResident: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Evict: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFence: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u64,
        D3D12_FENCE_FLAGS,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetDeviceRemovedReason:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetCopyableFootprints: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_RESOURCE_DESC,
        u32,
        u32,
        u64,
        *mut D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
        *mut u32,
        *mut u64,
        *mut u64,
    ),
    pub CreateQueryHeap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_QUERY_HEAP_DESC,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetStablePowerState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CreateCommandSignature: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_COMMAND_SIGNATURE_DESC,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetResourceTiling: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut u32,
        *mut D3D12_PACKED_MIP_INFO,
        *mut D3D12_TILE_SHAPE,
        *mut u32,
        u32,
        *mut D3D12_SUBRESOURCE_TILING,
    ),
    pub GetAdapterLuid: unsafe extern "system" fn(*mut core::ffi::c_void, *mut LUID),
}
impl windows_core::RuntimeName for ID3D12Device {}
windows_core::imp::define_interface!(
    ID3D12DeviceChild,
    ID3D12DeviceChild_Vtbl,
    0x905db94b_a00c_4140_9df5_2b64ca9ea357
);
impl core::ops::Deref for ID3D12DeviceChild {
    type Target = ID3D12Object;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID3D12DeviceChild, windows_core::IUnknown, ID3D12Object);
#[repr(C)]
pub struct ID3D12DeviceChild_Vtbl {
    pub base__: ID3D12Object_Vtbl,
    GetDevice: usize,
}
impl windows_core::RuntimeName for ID3D12DeviceChild {}
windows_core::imp::define_interface!(
    ID3D12Fence,
    ID3D12Fence_Vtbl,
    0x0a753dcf_c4d8_4b91_adf6_be5a60d95a76
);
impl core::ops::Deref for ID3D12Fence {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12Fence,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12Fence {
    pub unsafe fn GetCompletedValue(&self) -> u64 {
        unsafe {
            (windows_core::Interface::vtable(self).GetCompletedValue)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn SetEventOnCompletion(&self, value: u64, hevent: HANDLE) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetEventOnCompletion)(
                windows_core::Interface::as_raw(self),
                value,
                hevent,
            )
        }
    }
    pub unsafe fn Signal(&self, value: u64) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Signal)(
                windows_core::Interface::as_raw(self),
                value,
            )
        }
    }
}
#[repr(C)]
pub struct ID3D12Fence_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub GetCompletedValue: unsafe extern "system" fn(*mut core::ffi::c_void) -> u64,
    pub SetEventOnCompletion:
        unsafe extern "system" fn(*mut core::ffi::c_void, u64, HANDLE) -> windows_core::HRESULT,
    pub Signal: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for ID3D12Fence {}
windows_core::imp::define_interface!(
    ID3D12GraphicsCommandList,
    ID3D12GraphicsCommandList_Vtbl,
    0x5b160d0f_ac1b_4185_8ba8_b3ae42a5a455
);
impl core::ops::Deref for ID3D12GraphicsCommandList {
    type Target = ID3D12CommandList;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12GraphicsCommandList,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12CommandList
);
impl ID3D12GraphicsCommandList {
    pub unsafe fn Close(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Close)(windows_core::Interface::as_raw(self))
        }
    }
    pub unsafe fn Reset<P0, P1>(&self, pallocator: P0, pinitialstate: P1) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID3D12CommandAllocator>,
        P1: windows_core::Param<ID3D12PipelineState>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Reset)(
                windows_core::Interface::as_raw(self),
                pallocator.param().abi(),
                pinitialstate.param().abi(),
            )
        }
    }
    pub unsafe fn ClearState<P0>(&self, ppipelinestate: P0)
    where
        P0: windows_core::Param<ID3D12PipelineState>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ClearState)(
                windows_core::Interface::as_raw(self),
                ppipelinestate.param().abi(),
            );
        }
    }
    pub unsafe fn DrawInstanced(
        &self,
        vertexcountperinstance: u32,
        instancecount: u32,
        startvertexlocation: u32,
        startinstancelocation: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).DrawInstanced)(
                windows_core::Interface::as_raw(self),
                vertexcountperinstance,
                instancecount,
                startvertexlocation,
                startinstancelocation,
            );
        }
    }
    pub unsafe fn DrawIndexedInstanced(
        &self,
        indexcountperinstance: u32,
        instancecount: u32,
        startindexlocation: u32,
        basevertexlocation: i32,
        startinstancelocation: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).DrawIndexedInstanced)(
                windows_core::Interface::as_raw(self),
                indexcountperinstance,
                instancecount,
                startindexlocation,
                basevertexlocation,
                startinstancelocation,
            );
        }
    }
    pub unsafe fn Dispatch(
        &self,
        threadgroupcountx: u32,
        threadgroupcounty: u32,
        threadgroupcountz: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).Dispatch)(
                windows_core::Interface::as_raw(self),
                threadgroupcountx,
                threadgroupcounty,
                threadgroupcountz,
            );
        }
    }
    pub unsafe fn CopyBufferRegion<P0, P2>(
        &self,
        pdstbuffer: P0,
        dstoffset: u64,
        psrcbuffer: P2,
        srcoffset: u64,
        numbytes: u64,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P2: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyBufferRegion)(
                windows_core::Interface::as_raw(self),
                pdstbuffer.param().abi(),
                dstoffset,
                psrcbuffer.param().abi(),
                srcoffset,
                numbytes,
            );
        }
    }
    pub unsafe fn CopyTextureRegion(
        &self,
        pdst: *const D3D12_TEXTURE_COPY_LOCATION,
        dstx: u32,
        dsty: u32,
        dstz: u32,
        psrc: *const D3D12_TEXTURE_COPY_LOCATION,
        psrcbox: Option<*const D3D12_BOX>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).CopyTextureRegion)(
                windows_core::Interface::as_raw(self),
                pdst,
                dstx,
                dsty,
                dstz,
                psrc,
                psrcbox.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn CopyResource<P0, P1>(&self, pdstresource: P0, psrcresource: P1)
    where
        P0: windows_core::Param<ID3D12Resource>,
        P1: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyResource)(
                windows_core::Interface::as_raw(self),
                pdstresource.param().abi(),
                psrcresource.param().abi(),
            );
        }
    }
    pub unsafe fn CopyTiles<P0, P3>(
        &self,
        ptiledresource: P0,
        ptileregionstartcoordinate: *const D3D12_TILED_RESOURCE_COORDINATE,
        ptileregionsize: *const D3D12_TILE_REGION_SIZE,
        pbuffer: P3,
        bufferstartoffsetinbytes: u64,
        flags: D3D12_TILE_COPY_FLAGS,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P3: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyTiles)(
                windows_core::Interface::as_raw(self),
                ptiledresource.param().abi(),
                ptileregionstartcoordinate,
                ptileregionsize,
                pbuffer.param().abi(),
                bufferstartoffsetinbytes,
                flags,
            );
        }
    }
    pub unsafe fn ResolveSubresource<P0, P2>(
        &self,
        pdstresource: P0,
        dstsubresource: u32,
        psrcresource: P2,
        srcsubresource: u32,
        format: DXGI_FORMAT,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
        P2: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ResolveSubresource)(
                windows_core::Interface::as_raw(self),
                pdstresource.param().abi(),
                dstsubresource,
                psrcresource.param().abi(),
                srcsubresource,
                format,
            );
        }
    }
    pub unsafe fn IASetPrimitiveTopology(&self, primitivetopology: D3D12_PRIMITIVE_TOPOLOGY) {
        unsafe {
            (windows_core::Interface::vtable(self).IASetPrimitiveTopology)(
                windows_core::Interface::as_raw(self),
                primitivetopology,
            );
        }
    }
    pub unsafe fn RSSetViewports(&self, pviewports: &[D3D12_VIEWPORT]) {
        unsafe {
            (windows_core::Interface::vtable(self).RSSetViewports)(
                windows_core::Interface::as_raw(self),
                pviewports.len().try_into().unwrap(),
                pviewports.as_ptr(),
            );
        }
    }
    pub unsafe fn RSSetScissorRects(&self, prects: &[D3D12_RECT]) {
        unsafe {
            (windows_core::Interface::vtable(self).RSSetScissorRects)(
                windows_core::Interface::as_raw(self),
                prects.len().try_into().unwrap(),
                prects.as_ptr(),
            );
        }
    }
    pub unsafe fn OMSetBlendFactor(&self, blendfactor: Option<&[f32; 4]>) {
        unsafe {
            (windows_core::Interface::vtable(self).OMSetBlendFactor)(
                windows_core::Interface::as_raw(self),
                blendfactor.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            );
        }
    }
    pub unsafe fn OMSetStencilRef(&self, stencilref: u32) {
        unsafe {
            (windows_core::Interface::vtable(self).OMSetStencilRef)(
                windows_core::Interface::as_raw(self),
                stencilref,
            );
        }
    }
    pub unsafe fn SetPipelineState<P0>(&self, ppipelinestate: P0)
    where
        P0: windows_core::Param<ID3D12PipelineState>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetPipelineState)(
                windows_core::Interface::as_raw(self),
                ppipelinestate.param().abi(),
            );
        }
    }
    pub unsafe fn ResourceBarrier(&self, pbarriers: &[D3D12_RESOURCE_BARRIER]) {
        unsafe {
            (windows_core::Interface::vtable(self).ResourceBarrier)(
                windows_core::Interface::as_raw(self),
                pbarriers.len().try_into().unwrap(),
                pbarriers.as_ptr(),
            );
        }
    }
    pub unsafe fn ExecuteBundle<P0>(&self, pcommandlist: P0)
    where
        P0: windows_core::Param<Self>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ExecuteBundle)(
                windows_core::Interface::as_raw(self),
                pcommandlist.param().abi(),
            );
        }
    }
    pub unsafe fn SetDescriptorHeaps(&self, ppdescriptorheaps: &[Option<ID3D12DescriptorHeap>]) {
        unsafe {
            (windows_core::Interface::vtable(self).SetDescriptorHeaps)(
                windows_core::Interface::as_raw(self),
                ppdescriptorheaps.len().try_into().unwrap(),
                core::mem::transmute(ppdescriptorheaps.as_ptr()),
            );
        }
    }
    pub unsafe fn SetComputeRootSignature<P0>(&self, prootsignature: P0)
    where
        P0: windows_core::Param<ID3D12RootSignature>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRootSignature)(
                windows_core::Interface::as_raw(self),
                prootsignature.param().abi(),
            );
        }
    }
    pub unsafe fn SetGraphicsRootSignature<P0>(&self, prootsignature: P0)
    where
        P0: windows_core::Param<ID3D12RootSignature>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRootSignature)(
                windows_core::Interface::as_raw(self),
                prootsignature.param().abi(),
            );
        }
    }
    pub unsafe fn SetComputeRootDescriptorTable(
        &self,
        rootparameterindex: u32,
        basedescriptor: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRootDescriptorTable)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                basedescriptor,
            );
        }
    }
    pub unsafe fn SetGraphicsRootDescriptorTable(
        &self,
        rootparameterindex: u32,
        basedescriptor: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRootDescriptorTable)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                basedescriptor,
            );
        }
    }
    pub unsafe fn SetComputeRoot32BitConstant(
        &self,
        rootparameterindex: u32,
        srcdata: u32,
        destoffsetin32bitvalues: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRoot32BitConstant)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                srcdata,
                destoffsetin32bitvalues,
            );
        }
    }
    pub unsafe fn SetGraphicsRoot32BitConstant(
        &self,
        rootparameterindex: u32,
        srcdata: u32,
        destoffsetin32bitvalues: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRoot32BitConstant)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                srcdata,
                destoffsetin32bitvalues,
            );
        }
    }
    pub unsafe fn SetComputeRoot32BitConstants(
        &self,
        rootparameterindex: u32,
        num32bitvaluestoset: u32,
        psrcdata: *const core::ffi::c_void,
        destoffsetin32bitvalues: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRoot32BitConstants)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                num32bitvaluestoset,
                psrcdata,
                destoffsetin32bitvalues,
            );
        }
    }
    pub unsafe fn SetGraphicsRoot32BitConstants(
        &self,
        rootparameterindex: u32,
        num32bitvaluestoset: u32,
        psrcdata: *const core::ffi::c_void,
        destoffsetin32bitvalues: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRoot32BitConstants)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                num32bitvaluestoset,
                psrcdata,
                destoffsetin32bitvalues,
            );
        }
    }
    pub unsafe fn SetComputeRootConstantBufferView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRootConstantBufferView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn SetGraphicsRootConstantBufferView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRootConstantBufferView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn SetComputeRootShaderResourceView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRootShaderResourceView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn SetGraphicsRootShaderResourceView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRootShaderResourceView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn SetComputeRootUnorderedAccessView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetComputeRootUnorderedAccessView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn SetGraphicsRootUnorderedAccessView(
        &self,
        rootparameterindex: u32,
        bufferlocation: D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGraphicsRootUnorderedAccessView)(
                windows_core::Interface::as_raw(self),
                rootparameterindex,
                bufferlocation,
            );
        }
    }
    pub unsafe fn IASetIndexBuffer(&self, pview: Option<*const D3D12_INDEX_BUFFER_VIEW>) {
        unsafe {
            (windows_core::Interface::vtable(self).IASetIndexBuffer)(
                windows_core::Interface::as_raw(self),
                pview.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn IASetVertexBuffers(
        &self,
        startslot: u32,
        pviews: Option<&[D3D12_VERTEX_BUFFER_VIEW]>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).IASetVertexBuffers)(
                windows_core::Interface::as_raw(self),
                startslot,
                pviews.map_or(0, |slice| slice.len().try_into().unwrap()),
                pviews.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            );
        }
    }
    pub unsafe fn SOSetTargets(
        &self,
        startslot: u32,
        pviews: Option<&[D3D12_STREAM_OUTPUT_BUFFER_VIEW]>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SOSetTargets)(
                windows_core::Interface::as_raw(self),
                startslot,
                pviews.map_or(0, |slice| slice.len().try_into().unwrap()),
                pviews.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            );
        }
    }
    pub unsafe fn OMSetRenderTargets(
        &self,
        numrendertargetdescriptors: u32,
        prendertargetdescriptors: Option<*const D3D12_CPU_DESCRIPTOR_HANDLE>,
        rtssinglehandletodescriptorrange: bool,
        pdepthstencildescriptor: Option<*const D3D12_CPU_DESCRIPTOR_HANDLE>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).OMSetRenderTargets)(
                windows_core::Interface::as_raw(self),
                numrendertargetdescriptors,
                prendertargetdescriptors.unwrap_or(core::mem::zeroed()) as _,
                rtssinglehandletodescriptorrange.into(),
                pdepthstencildescriptor.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn ClearDepthStencilView(
        &self,
        depthstencilview: D3D12_CPU_DESCRIPTOR_HANDLE,
        clearflags: D3D12_CLEAR_FLAGS,
        depth: f32,
        stencil: u8,
        prects: &[D3D12_RECT],
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).ClearDepthStencilView)(
                windows_core::Interface::as_raw(self),
                depthstencilview,
                clearflags,
                depth,
                stencil,
                prects.len().try_into().unwrap(),
                prects.as_ptr(),
            );
        }
    }
    pub unsafe fn ClearRenderTargetView(
        &self,
        rendertargetview: D3D12_CPU_DESCRIPTOR_HANDLE,
        colorrgba: &[f32; 4],
        prects: &[D3D12_RECT],
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).ClearRenderTargetView)(
                windows_core::Interface::as_raw(self),
                rendertargetview,
                colorrgba.as_ptr(),
                prects.len().try_into().unwrap(),
                prects.as_ptr(),
            );
        }
    }
    pub unsafe fn ClearUnorderedAccessViewUint<P2>(
        &self,
        viewgpuhandleincurrentheap: D3D12_GPU_DESCRIPTOR_HANDLE,
        viewcpuhandle: D3D12_CPU_DESCRIPTOR_HANDLE,
        presource: P2,
        values: &[u32; 4],
        prects: &[D3D12_RECT],
    ) where
        P2: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ClearUnorderedAccessViewUint)(
                windows_core::Interface::as_raw(self),
                viewgpuhandleincurrentheap,
                viewcpuhandle,
                presource.param().abi(),
                values.as_ptr(),
                prects.len().try_into().unwrap(),
                prects.as_ptr(),
            );
        }
    }
    pub unsafe fn ClearUnorderedAccessViewFloat<P2>(
        &self,
        viewgpuhandleincurrentheap: D3D12_GPU_DESCRIPTOR_HANDLE,
        viewcpuhandle: D3D12_CPU_DESCRIPTOR_HANDLE,
        presource: P2,
        values: &[f32; 4],
        prects: &[D3D12_RECT],
    ) where
        P2: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ClearUnorderedAccessViewFloat)(
                windows_core::Interface::as_raw(self),
                viewgpuhandleincurrentheap,
                viewcpuhandle,
                presource.param().abi(),
                values.as_ptr(),
                prects.len().try_into().unwrap(),
                prects.as_ptr(),
            );
        }
    }
    pub unsafe fn DiscardResource<P0>(
        &self,
        presource: P0,
        pregion: Option<*const D3D12_DISCARD_REGION>,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DiscardResource)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                pregion.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn BeginQuery<P0>(&self, pqueryheap: P0, r#type: D3D12_QUERY_TYPE, index: u32)
    where
        P0: windows_core::Param<ID3D12QueryHeap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).BeginQuery)(
                windows_core::Interface::as_raw(self),
                pqueryheap.param().abi(),
                r#type,
                index,
            );
        }
    }
    pub unsafe fn EndQuery<P0>(&self, pqueryheap: P0, r#type: D3D12_QUERY_TYPE, index: u32)
    where
        P0: windows_core::Param<ID3D12QueryHeap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).EndQuery)(
                windows_core::Interface::as_raw(self),
                pqueryheap.param().abi(),
                r#type,
                index,
            );
        }
    }
    pub unsafe fn ResolveQueryData<P0, P4>(
        &self,
        pqueryheap: P0,
        r#type: D3D12_QUERY_TYPE,
        startindex: u32,
        numqueries: u32,
        pdestinationbuffer: P4,
        aligneddestinationbufferoffset: u64,
    ) where
        P0: windows_core::Param<ID3D12QueryHeap>,
        P4: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ResolveQueryData)(
                windows_core::Interface::as_raw(self),
                pqueryheap.param().abi(),
                r#type,
                startindex,
                numqueries,
                pdestinationbuffer.param().abi(),
                aligneddestinationbufferoffset,
            );
        }
    }
    pub unsafe fn SetPredication<P0>(
        &self,
        pbuffer: P0,
        alignedbufferoffset: u64,
        operation: D3D12_PREDICATION_OP,
    ) where
        P0: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetPredication)(
                windows_core::Interface::as_raw(self),
                pbuffer.param().abi(),
                alignedbufferoffset,
                operation,
            );
        }
    }
    pub unsafe fn SetMarker(
        &self,
        metadata: u32,
        pdata: Option<*const core::ffi::c_void>,
        size: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetMarker)(
                windows_core::Interface::as_raw(self),
                metadata,
                pdata.unwrap_or(core::mem::zeroed()) as _,
                size,
            );
        }
    }
    pub unsafe fn BeginEvent(
        &self,
        metadata: u32,
        pdata: Option<*const core::ffi::c_void>,
        size: u32,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).BeginEvent)(
                windows_core::Interface::as_raw(self),
                metadata,
                pdata.unwrap_or(core::mem::zeroed()) as _,
                size,
            );
        }
    }
    pub unsafe fn EndEvent(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).EndEvent)(windows_core::Interface::as_raw(self));
        }
    }
    pub unsafe fn ExecuteIndirect<P0, P2, P4>(
        &self,
        pcommandsignature: P0,
        maxcommandcount: u32,
        pargumentbuffer: P2,
        argumentbufferoffset: u64,
        pcountbuffer: P4,
        countbufferoffset: u64,
    ) where
        P0: windows_core::Param<ID3D12CommandSignature>,
        P2: windows_core::Param<ID3D12Resource>,
        P4: windows_core::Param<ID3D12Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ExecuteIndirect)(
                windows_core::Interface::as_raw(self),
                pcommandsignature.param().abi(),
                maxcommandcount,
                pargumentbuffer.param().abi(),
                argumentbufferoffset,
                pcountbuffer.param().abi(),
                countbufferoffset,
            );
        }
    }
}
#[repr(C)]
pub struct ID3D12GraphicsCommandList_Vtbl {
    pub base__: ID3D12CommandList_Vtbl,
    pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Reset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ClearState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub DrawInstanced: unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, u32, u32),
    pub DrawIndexedInstanced:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, u32, i32, u32),
    pub Dispatch: unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, u32),
    pub CopyBufferRegion: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u64,
        *mut core::ffi::c_void,
        u64,
        u64,
    ),
    pub CopyTextureRegion: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D12_TEXTURE_COPY_LOCATION,
        u32,
        u32,
        u32,
        *const D3D12_TEXTURE_COPY_LOCATION,
        *const D3D12_BOX,
    ),
    pub CopyResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    pub CopyTiles: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_TILED_RESOURCE_COORDINATE,
        *const D3D12_TILE_REGION_SIZE,
        *mut core::ffi::c_void,
        u64,
        D3D12_TILE_COPY_FLAGS,
    ),
    pub ResolveSubresource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        *mut core::ffi::c_void,
        u32,
        DXGI_FORMAT,
    ),
    pub IASetPrimitiveTopology:
        unsafe extern "system" fn(*mut core::ffi::c_void, D3D12_PRIMITIVE_TOPOLOGY),
    pub RSSetViewports:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const D3D12_VIEWPORT),
    pub RSSetScissorRects:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const D3D12_RECT),
    pub OMSetBlendFactor: unsafe extern "system" fn(*mut core::ffi::c_void, *const f32),
    pub OMSetStencilRef: unsafe extern "system" fn(*mut core::ffi::c_void, u32),
    pub SetPipelineState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub ResourceBarrier:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const D3D12_RESOURCE_BARRIER),
    pub ExecuteBundle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub SetDescriptorHeaps:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const *mut core::ffi::c_void),
    pub SetComputeRootSignature:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub SetGraphicsRootSignature:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub SetComputeRootDescriptorTable:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_DESCRIPTOR_HANDLE),
    pub SetGraphicsRootDescriptorTable:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_DESCRIPTOR_HANDLE),
    pub SetComputeRoot32BitConstant:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, u32),
    pub SetGraphicsRoot32BitConstant:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, u32),
    pub SetComputeRoot32BitConstants:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, *const core::ffi::c_void, u32),
    pub SetGraphicsRoot32BitConstants:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32, *const core::ffi::c_void, u32),
    pub SetComputeRootConstantBufferView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub SetGraphicsRootConstantBufferView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub SetComputeRootShaderResourceView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub SetGraphicsRootShaderResourceView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub SetComputeRootUnorderedAccessView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub SetGraphicsRootUnorderedAccessView:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, D3D12_GPU_VIRTUAL_ADDRESS),
    pub IASetIndexBuffer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const D3D12_INDEX_BUFFER_VIEW),
    pub IASetVertexBuffers: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        *const D3D12_VERTEX_BUFFER_VIEW,
    ),
    pub SOSetTargets: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        *const D3D12_STREAM_OUTPUT_BUFFER_VIEW,
    ),
    pub OMSetRenderTargets: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const D3D12_CPU_DESCRIPTOR_HANDLE,
        windows_core::BOOL,
        *const D3D12_CPU_DESCRIPTOR_HANDLE,
    ),
    pub ClearDepthStencilView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        D3D12_CLEAR_FLAGS,
        f32,
        u8,
        u32,
        *const D3D12_RECT,
    ),
    pub ClearRenderTargetView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        *const f32,
        u32,
        *const D3D12_RECT,
    ),
    pub ClearUnorderedAccessViewUint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_GPU_DESCRIPTOR_HANDLE,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        *mut core::ffi::c_void,
        *const u32,
        u32,
        *const D3D12_RECT,
    ),
    pub ClearUnorderedAccessViewFloat: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D3D12_GPU_DESCRIPTOR_HANDLE,
        D3D12_CPU_DESCRIPTOR_HANDLE,
        *mut core::ffi::c_void,
        *const f32,
        u32,
        *const D3D12_RECT,
    ),
    pub DiscardResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D3D12_DISCARD_REGION,
    ),
    pub BeginQuery: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D3D12_QUERY_TYPE,
        u32,
    ),
    pub EndQuery: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D3D12_QUERY_TYPE,
        u32,
    ),
    pub ResolveQueryData: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D3D12_QUERY_TYPE,
        u32,
        u32,
        *mut core::ffi::c_void,
        u64,
    ),
    pub SetPredication: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u64,
        D3D12_PREDICATION_OP,
    ),
    pub SetMarker:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const core::ffi::c_void, u32),
    pub BeginEvent:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const core::ffi::c_void, u32),
    pub EndEvent: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub ExecuteIndirect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        *mut core::ffi::c_void,
        u64,
        *mut core::ffi::c_void,
        u64,
    ),
}
impl windows_core::RuntimeName for ID3D12GraphicsCommandList {}
windows_core::imp::define_interface!(
    ID3D12Heap,
    ID3D12Heap_Vtbl,
    0x6b3b2502_6e51_45b3_90ee_9884265e8df3
);
impl core::ops::Deref for ID3D12Heap {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12Heap,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
#[repr(C)]
pub struct ID3D12Heap_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    GetDesc: usize,
}
impl windows_core::RuntimeName for ID3D12Heap {}
windows_core::imp::define_interface!(
    ID3D12InfoQueue,
    ID3D12InfoQueue_Vtbl,
    0x0742a90b_c387_483f_b946_30a7e4e61458
);
windows_core::imp::interface_hierarchy!(ID3D12InfoQueue, windows_core::IUnknown);
impl ID3D12InfoQueue {
    pub unsafe fn ClearStoredMessages(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).ClearStoredMessages)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
    pub unsafe fn GetMessage(
        &self,
        messageindex: u64,
        pmessage: Option<*mut D3D12_MESSAGE>,
        pmessagebytelength: *mut usize,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetMessage)(
                windows_core::Interface::as_raw(self),
                messageindex,
                pmessage.unwrap_or(core::mem::zeroed()) as _,
                pmessagebytelength as _,
            )
        }
    }
    pub unsafe fn GetNumStoredMessagesAllowedByRetrievalFilter(&self) -> u64 {
        unsafe {
            (windows_core::Interface::vtable(self).GetNumStoredMessagesAllowedByRetrievalFilter)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
}
#[repr(C)]
pub struct ID3D12InfoQueue_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetMessageCountLimit: usize,
    pub ClearStoredMessages: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub GetMessage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u64,
        *mut D3D12_MESSAGE,
        *mut usize,
    ) -> windows_core::HRESULT,
    GetNumMessagesAllowedByStorageFilter: usize,
    GetNumMessagesDeniedByStorageFilter: usize,
    GetNumStoredMessages: usize,
    pub GetNumStoredMessagesAllowedByRetrievalFilter:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> u64,
    GetNumMessagesDiscardedByMessageCountLimit: usize,
    GetMessageCountLimit: usize,
    AddStorageFilterEntries: usize,
    GetStorageFilter: usize,
    ClearStorageFilter: usize,
    PushEmptyStorageFilter: usize,
    PushCopyOfStorageFilter: usize,
    PushStorageFilter: usize,
    PopStorageFilter: usize,
    GetStorageFilterStackSize: usize,
    AddRetrievalFilterEntries: usize,
    GetRetrievalFilter: usize,
    ClearRetrievalFilter: usize,
    PushEmptyRetrievalFilter: usize,
    PushCopyOfRetrievalFilter: usize,
    PushRetrievalFilter: usize,
    PopRetrievalFilter: usize,
    GetRetrievalFilterStackSize: usize,
    AddMessage: usize,
    AddApplicationMessage: usize,
    SetBreakOnCategory: usize,
    SetBreakOnSeverity: usize,
    SetBreakOnID: usize,
    GetBreakOnCategory: usize,
    GetBreakOnSeverity: usize,
    GetBreakOnID: usize,
    SetMuteDebugOutput: usize,
    GetMuteDebugOutput: usize,
}
impl windows_core::RuntimeName for ID3D12InfoQueue {}
windows_core::imp::define_interface!(
    ID3D12Object,
    ID3D12Object_Vtbl,
    0xc4fec28f_7966_4e95_9f94_f431cb56c3b8
);
windows_core::imp::interface_hierarchy!(ID3D12Object, windows_core::IUnknown);
#[repr(C)]
pub struct ID3D12Object_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetPrivateData: usize,
    SetPrivateData: usize,
    SetPrivateDataInterface: usize,
    SetName: usize,
}
impl windows_core::RuntimeName for ID3D12Object {}
windows_core::imp::define_interface!(
    ID3D12Pageable,
    ID3D12Pageable_Vtbl,
    0x63ee58fb_1268_4835_86da_f008ce62f0d6
);
impl core::ops::Deref for ID3D12Pageable {
    type Target = ID3D12DeviceChild;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12Pageable,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild
);
#[repr(C)]
pub struct ID3D12Pageable_Vtbl {
    pub base__: ID3D12DeviceChild_Vtbl,
}
impl windows_core::RuntimeName for ID3D12Pageable {}
windows_core::imp::define_interface!(
    ID3D12PipelineState,
    ID3D12PipelineState_Vtbl,
    0x765a30f3_f624_4c6f_a828_ace948622445
);
impl core::ops::Deref for ID3D12PipelineState {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12PipelineState,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12PipelineState {
    pub unsafe fn GetCachedBlob(&self) -> windows_core::Result<ID3D10Blob> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedBlob)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct ID3D12PipelineState_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub GetCachedBlob: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for ID3D12PipelineState {}
windows_core::imp::define_interface!(
    ID3D12QueryHeap,
    ID3D12QueryHeap_Vtbl,
    0x0d9658ae_ed45_469e_a61d_970ec583cab4
);
impl core::ops::Deref for ID3D12QueryHeap {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12QueryHeap,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
#[repr(C)]
pub struct ID3D12QueryHeap_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
}
impl windows_core::RuntimeName for ID3D12QueryHeap {}
windows_core::imp::define_interface!(
    ID3D12Resource,
    ID3D12Resource_Vtbl,
    0x696442be_a72e_4059_bc79_5b5c98040fad
);
impl core::ops::Deref for ID3D12Resource {
    type Target = ID3D12Pageable;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12Resource,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild,
    ID3D12Pageable
);
impl ID3D12Resource {
    pub unsafe fn Map(
        &self,
        subresource: u32,
        preadrange: Option<*const D3D12_RANGE>,
        ppdata: *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Map)(
                windows_core::Interface::as_raw(self),
                subresource,
                preadrange.unwrap_or(core::mem::zeroed()) as _,
                ppdata as _,
            )
        }
    }
    pub unsafe fn Unmap(&self, subresource: u32, pwrittenrange: Option<*const D3D12_RANGE>) {
        unsafe {
            (windows_core::Interface::vtable(self).Unmap)(
                windows_core::Interface::as_raw(self),
                subresource,
                pwrittenrange.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn GetDesc(&self) -> D3D12_RESOURCE_DESC {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetGPUVirtualAddress(&self) -> D3D12_GPU_VIRTUAL_ADDRESS {
        unsafe {
            (windows_core::Interface::vtable(self).GetGPUVirtualAddress)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn WriteToSubresource(
        &self,
        dstsubresource: u32,
        pdstbox: Option<*const D3D12_BOX>,
        psrcdata: *const core::ffi::c_void,
        srcrowpitch: u32,
        srcdepthpitch: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).WriteToSubresource)(
                windows_core::Interface::as_raw(self),
                dstsubresource,
                pdstbox.unwrap_or(core::mem::zeroed()) as _,
                psrcdata,
                srcrowpitch,
                srcdepthpitch,
            )
        }
    }
    pub unsafe fn ReadFromSubresource(
        &self,
        pdstdata: *mut core::ffi::c_void,
        dstrowpitch: u32,
        dstdepthpitch: u32,
        srcsubresource: u32,
        psrcbox: Option<*const D3D12_BOX>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).ReadFromSubresource)(
                windows_core::Interface::as_raw(self),
                pdstdata as _,
                dstrowpitch,
                dstdepthpitch,
                srcsubresource,
                psrcbox.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetHeapProperties(
        &self,
        pheapproperties: Option<*mut D3D12_HEAP_PROPERTIES>,
        pheapflags: Option<*mut D3D12_HEAP_FLAGS>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetHeapProperties)(
                windows_core::Interface::as_raw(self),
                pheapproperties.unwrap_or(core::mem::zeroed()) as _,
                pheapflags.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
}
#[repr(C)]
pub struct ID3D12Resource_Vtbl {
    pub base__: ID3D12Pageable_Vtbl,
    pub Map: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const D3D12_RANGE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Unmap: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const D3D12_RANGE),
    pub GetDesc: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D12_RESOURCE_DESC),
    pub GetGPUVirtualAddress:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> D3D12_GPU_VIRTUAL_ADDRESS,
    pub WriteToSubresource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const D3D12_BOX,
        *const core::ffi::c_void,
        u32,
        u32,
    ) -> windows_core::HRESULT,
    pub ReadFromSubresource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        u32,
        u32,
        *const D3D12_BOX,
    ) -> windows_core::HRESULT,
    pub GetHeapProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D3D12_HEAP_PROPERTIES,
        *mut D3D12_HEAP_FLAGS,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for ID3D12Resource {}
windows_core::imp::define_interface!(
    ID3D12RootSignature,
    ID3D12RootSignature_Vtbl,
    0xc54a6b66_72df_4ee8_8be5_a946a1429214
);
impl core::ops::Deref for ID3D12RootSignature {
    type Target = ID3D12DeviceChild;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D12RootSignature,
    windows_core::IUnknown,
    ID3D12Object,
    ID3D12DeviceChild
);
#[repr(C)]
pub struct ID3D12RootSignature_Vtbl {
    pub base__: ID3D12DeviceChild_Vtbl,
}
impl windows_core::RuntimeName for ID3D12RootSignature {}
pub const IDC_ARROW: windows_core::PCWSTR = windows_core::PCWSTR(32512 as _);
pub const IDC_CROSS: windows_core::PCWSTR = windows_core::PCWSTR(32515 as _);
pub const IDC_HAND: windows_core::PCWSTR = windows_core::PCWSTR(32649 as _);
pub const IDC_IBEAM: windows_core::PCWSTR = windows_core::PCWSTR(32513 as _);
pub const IDC_NO: windows_core::PCWSTR = windows_core::PCWSTR(32648 as _);
pub const IDC_SIZEALL: windows_core::PCWSTR = windows_core::PCWSTR(32646 as _);
pub const IDC_SIZENESW: windows_core::PCWSTR = windows_core::PCWSTR(32643 as _);
pub const IDC_SIZENS: windows_core::PCWSTR = windows_core::PCWSTR(32645 as _);
pub const IDC_SIZENWSE: windows_core::PCWSTR = windows_core::PCWSTR(32642 as _);
pub const IDC_SIZEWE: windows_core::PCWSTR = windows_core::PCWSTR(32644 as _);
windows_core::imp::define_interface!(
    IDWriteFactory,
    IDWriteFactory_Vtbl,
    0xb859ee5a_d838_4b5b_a2e8_1adc7d93db48
);
windows_core::imp::interface_hierarchy!(IDWriteFactory, windows_core::IUnknown);
impl IDWriteFactory {
    pub unsafe fn GetSystemFontCollection(
        &self,
        fontcollection: *mut Option<IDWriteFontCollection>,
        checkforupdates: bool,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetSystemFontCollection)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(fontcollection),
                checkforupdates.into(),
            )
        }
    }
    pub unsafe fn CreateTextFormat<P0, P1, P6>(
        &self,
        fontfamilyname: P0,
        fontcollection: P1,
        fontweight: DWRITE_FONT_WEIGHT,
        fontstyle: DWRITE_FONT_STYLE,
        fontstretch: DWRITE_FONT_STRETCH,
        fontsize: f32,
        localename: P6,
    ) -> windows_core::Result<IDWriteTextFormat>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
        P1: windows_core::Param<IDWriteFontCollection>,
        P6: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTextFormat)(
                windows_core::Interface::as_raw(self),
                fontfamilyname.param().abi(),
                fontcollection.param().abi(),
                fontweight,
                fontstyle,
                fontstretch,
                fontsize,
                localename.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateTextLayout<P2>(
        &self,
        string: &[u16],
        textformat: P2,
        maxwidth: f32,
        maxheight: f32,
    ) -> windows_core::Result<IDWriteTextLayout>
    where
        P2: windows_core::Param<IDWriteTextFormat>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTextLayout)(
                windows_core::Interface::as_raw(self),
                string.as_ptr(),
                string.len().try_into().unwrap(),
                textformat.param().abi(),
                maxwidth,
                maxheight,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetSystemFontCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    CreateCustomFontCollection: usize,
    RegisterFontCollectionLoader: usize,
    UnregisterFontCollectionLoader: usize,
    CreateFontFileReference: usize,
    CreateCustomFontFileReference: usize,
    CreateFontFace: usize,
    CreateRenderingParams: usize,
    CreateMonitorRenderingParams: usize,
    CreateCustomRenderingParams: usize,
    RegisterFontFileLoader: usize,
    UnregisterFontFileLoader: usize,
    pub CreateTextFormat: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut core::ffi::c_void,
        DWRITE_FONT_WEIGHT,
        DWRITE_FONT_STYLE,
        DWRITE_FONT_STRETCH,
        f32,
        windows_core::PCWSTR,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateTypography: usize,
    GetGdiInterop: usize,
    pub CreateTextLayout: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u16,
        u32,
        *mut core::ffi::c_void,
        f32,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateGdiCompatibleTextLayout: usize,
    CreateEllipsisTrimmingSign: usize,
    CreateTextAnalyzer: usize,
    CreateNumberSubstitution: usize,
    CreateGlyphRunAnalysis: usize,
}
impl windows_core::RuntimeName for IDWriteFactory {}
windows_core::imp::define_interface!(
    IDWriteFactory1,
    IDWriteFactory1_Vtbl,
    0x30572f99_dac6_41db_a16e_0486307e606a
);
impl core::ops::Deref for IDWriteFactory1 {
    type Target = IDWriteFactory;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDWriteFactory1, windows_core::IUnknown, IDWriteFactory);
#[repr(C)]
pub struct IDWriteFactory1_Vtbl {
    pub base__: IDWriteFactory_Vtbl,
    GetEudcFontCollection: usize,
    CreateCustomRenderingParams: usize,
}
impl windows_core::RuntimeName for IDWriteFactory1 {}
windows_core::imp::define_interface!(
    IDWriteFactory2,
    IDWriteFactory2_Vtbl,
    0x0439fc60_ca44_4994_8dee_3a9af7b732ec
);
impl core::ops::Deref for IDWriteFactory2 {
    type Target = IDWriteFactory1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteFactory2,
    windows_core::IUnknown,
    IDWriteFactory,
    IDWriteFactory1
);
impl IDWriteFactory2 {
    pub unsafe fn CreateGlyphRunAnalysis(
        &self,
        glyphrun: *const DWRITE_GLYPH_RUN,
        transform: Option<*const DWRITE_MATRIX>,
        renderingmode: DWRITE_RENDERING_MODE,
        measuringmode: DWRITE_MEASURING_MODE,
        gridfitmode: DWRITE_GRID_FIT_MODE,
        antialiasmode: DWRITE_TEXT_ANTIALIAS_MODE,
        baselineoriginx: f32,
        baselineoriginy: f32,
    ) -> windows_core::Result<IDWriteGlyphRunAnalysis> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateGlyphRunAnalysis)(
                windows_core::Interface::as_raw(self),
                glyphrun,
                transform.unwrap_or(core::mem::zeroed()) as _,
                renderingmode,
                measuringmode,
                gridfitmode,
                antialiasmode,
                baselineoriginx,
                baselineoriginy,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFactory2_Vtbl {
    pub base__: IDWriteFactory1_Vtbl,
    GetSystemFontFallback: usize,
    CreateFontFallbackBuilder: usize,
    TranslateColorGlyphRun: usize,
    CreateCustomRenderingParams: usize,
    pub CreateGlyphRunAnalysis: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const DWRITE_GLYPH_RUN,
        *const DWRITE_MATRIX,
        DWRITE_RENDERING_MODE,
        DWRITE_MEASURING_MODE,
        DWRITE_GRID_FIT_MODE,
        DWRITE_TEXT_ANTIALIAS_MODE,
        f32,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDWriteFactory2 {}
windows_core::imp::define_interface!(
    IDWriteFont,
    IDWriteFont_Vtbl,
    0xacd16696_8c14_4f5d_877e_fe3fc1d32737
);
windows_core::imp::interface_hierarchy!(IDWriteFont, windows_core::IUnknown);
impl IDWriteFont {
    pub unsafe fn CreateFontFace(&self) -> windows_core::Result<IDWriteFontFace> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFontFace)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFont_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontFamily: usize,
    GetWeight: usize,
    GetStretch: usize,
    GetStyle: usize,
    IsSymbolFont: usize,
    GetFaceNames: usize,
    GetInformationalStrings: usize,
    GetSimulations: usize,
    GetMetrics: usize,
    HasCharacter: usize,
    pub CreateFontFace: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDWriteFont {}
windows_core::imp::define_interface!(
    IDWriteFontCollection,
    IDWriteFontCollection_Vtbl,
    0xa84cee02_3eea_4eee_a827_87c1a02a0fcc
);
windows_core::imp::interface_hierarchy!(IDWriteFontCollection, windows_core::IUnknown);
impl IDWriteFontCollection {
    pub unsafe fn GetFontFamily(&self, index: u32) -> windows_core::Result<IDWriteFontFamily> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFontFamily)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn FindFamilyName<P0>(
        &self,
        familyname: P0,
        index: *mut u32,
        exists: *mut windows_core::BOOL,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FindFamilyName)(
                windows_core::Interface::as_raw(self),
                familyname.param().abi(),
                index as _,
                exists as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteFontCollection_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontFamilyCount: usize,
    pub GetFontFamily: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindFamilyName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut u32,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    GetFontFromFontFace: usize,
}
impl windows_core::RuntimeName for IDWriteFontCollection {}
windows_core::imp::define_interface!(
    IDWriteFontFace,
    IDWriteFontFace_Vtbl,
    0x5f49804d_7024_4d43_bfa9_d25984f53849
);
windows_core::imp::interface_hierarchy!(IDWriteFontFace, windows_core::IUnknown);
impl IDWriteFontFace {
    pub unsafe fn GetMetrics(&self, fontfacemetrics: *mut DWRITE_FONT_METRICS) {
        unsafe {
            (windows_core::Interface::vtable(self).GetMetrics)(
                windows_core::Interface::as_raw(self),
                fontfacemetrics as _,
            );
        }
    }
    pub unsafe fn GetDesignGlyphMetrics(
        &self,
        glyphindices: *const u16,
        glyphcount: u32,
        glyphmetrics: *mut DWRITE_GLYPH_METRICS,
        issideways: bool,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesignGlyphMetrics)(
                windows_core::Interface::as_raw(self),
                glyphindices,
                glyphcount,
                glyphmetrics as _,
                issideways.into(),
            )
        }
    }
    pub unsafe fn GetGlyphIndices(
        &self,
        codepoints: *const u32,
        codepointcount: u32,
        glyphindices: *mut u16,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetGlyphIndices)(
                windows_core::Interface::as_raw(self),
                codepoints,
                codepointcount,
                glyphindices as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteFontFace_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetType: usize,
    GetFiles: usize,
    GetIndex: usize,
    GetSimulations: usize,
    IsSymbolFont: usize,
    pub GetMetrics: unsafe extern "system" fn(*mut core::ffi::c_void, *mut DWRITE_FONT_METRICS),
    GetGlyphCount: usize,
    pub GetDesignGlyphMetrics: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u16,
        u32,
        *mut DWRITE_GLYPH_METRICS,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetGlyphIndices: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u32,
        u32,
        *mut u16,
    ) -> windows_core::HRESULT,
    TryGetFontTable: usize,
    ReleaseFontTable: usize,
    GetGlyphRunOutline: usize,
    GetRecommendedRenderingMode: usize,
    GetGdiCompatibleMetrics: usize,
    GetGdiCompatibleGlyphMetrics: usize,
}
impl windows_core::RuntimeName for IDWriteFontFace {}
windows_core::imp::define_interface!(
    IDWriteFontFace1,
    IDWriteFontFace1_Vtbl,
    0xa71efdb4_9fdb_4838_ad90_cfc3be8c3daf
);
impl core::ops::Deref for IDWriteFontFace1 {
    type Target = IDWriteFontFace;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDWriteFontFace1, windows_core::IUnknown, IDWriteFontFace);
#[repr(C)]
pub struct IDWriteFontFace1_Vtbl {
    pub base__: IDWriteFontFace_Vtbl,
    GetMetrics: usize,
    GetGdiCompatibleMetrics: usize,
    GetCaretMetrics: usize,
    GetUnicodeRanges: usize,
    IsMonospacedFont: usize,
    GetDesignGlyphAdvances: usize,
    GetGdiCompatibleGlyphAdvances: usize,
    GetKerningPairAdjustments: usize,
    HasKerningPairs: usize,
    GetRecommendedRenderingMode: usize,
    GetVerticalGlyphVariants: usize,
    HasVerticalGlyphVariants: usize,
}
impl windows_core::RuntimeName for IDWriteFontFace1 {}
windows_core::imp::define_interface!(
    IDWriteFontFace2,
    IDWriteFontFace2_Vtbl,
    0xd8b768ff_64bc_4e66_982b_ec8e87f693f7
);
impl core::ops::Deref for IDWriteFontFace2 {
    type Target = IDWriteFontFace1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteFontFace2,
    windows_core::IUnknown,
    IDWriteFontFace,
    IDWriteFontFace1
);
impl IDWriteFontFace2 {
    pub unsafe fn GetRecommendedRenderingMode<P7>(
        &self,
        fontemsize: f32,
        dpix: f32,
        dpiy: f32,
        transform: Option<*const DWRITE_MATRIX>,
        issideways: bool,
        outlinethreshold: DWRITE_OUTLINE_THRESHOLD,
        measuringmode: DWRITE_MEASURING_MODE,
        renderingparams: P7,
        renderingmode: *mut DWRITE_RENDERING_MODE,
        gridfitmode: *mut DWRITE_GRID_FIT_MODE,
    ) -> windows_core::HRESULT
    where
        P7: windows_core::Param<IDWriteRenderingParams>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetRecommendedRenderingMode)(
                windows_core::Interface::as_raw(self),
                fontemsize,
                dpix,
                dpiy,
                transform.unwrap_or(core::mem::zeroed()) as _,
                issideways.into(),
                outlinethreshold,
                measuringmode,
                renderingparams.param().abi(),
                renderingmode as _,
                gridfitmode as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteFontFace2_Vtbl {
    pub base__: IDWriteFontFace1_Vtbl,
    IsColorFont: usize,
    GetColorPaletteCount: usize,
    GetPaletteEntryCount: usize,
    GetPaletteEntries: usize,
    pub GetRecommendedRenderingMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        f32,
        f32,
        *const DWRITE_MATRIX,
        windows_core::BOOL,
        DWRITE_OUTLINE_THRESHOLD,
        DWRITE_MEASURING_MODE,
        *mut core::ffi::c_void,
        *mut DWRITE_RENDERING_MODE,
        *mut DWRITE_GRID_FIT_MODE,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDWriteFontFace2 {}
windows_core::imp::define_interface!(
    IDWriteFontFamily,
    IDWriteFontFamily_Vtbl,
    0xda20d8ef_812a_4c43_9802_62ec4abd7add
);
impl core::ops::Deref for IDWriteFontFamily {
    type Target = IDWriteFontList;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDWriteFontFamily, windows_core::IUnknown, IDWriteFontList);
impl IDWriteFontFamily {
    pub unsafe fn GetFirstMatchingFont(
        &self,
        weight: DWRITE_FONT_WEIGHT,
        stretch: DWRITE_FONT_STRETCH,
        style: DWRITE_FONT_STYLE,
    ) -> windows_core::Result<IDWriteFont> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFirstMatchingFont)(
                windows_core::Interface::as_raw(self),
                weight,
                stretch,
                style,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDWriteFontFamily_Vtbl {
    pub base__: IDWriteFontList_Vtbl,
    GetFamilyNames: usize,
    pub GetFirstMatchingFont: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_FONT_WEIGHT,
        DWRITE_FONT_STRETCH,
        DWRITE_FONT_STYLE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    GetMatchingFonts: usize,
}
impl windows_core::RuntimeName for IDWriteFontFamily {}
windows_core::imp::define_interface!(
    IDWriteFontList,
    IDWriteFontList_Vtbl,
    0x1a0d8438_1d97_4ec1_aef9_a2fb86ed6acb
);
windows_core::imp::interface_hierarchy!(IDWriteFontList, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteFontList_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFontCollection: usize,
    GetFontCount: usize,
    GetFont: usize,
}
impl windows_core::RuntimeName for IDWriteFontList {}
windows_core::imp::define_interface!(
    IDWriteGlyphRunAnalysis,
    IDWriteGlyphRunAnalysis_Vtbl,
    0x7d97dbf7_e085_42d4_81e3_6a883bded118
);
windows_core::imp::interface_hierarchy!(IDWriteGlyphRunAnalysis, windows_core::IUnknown);
impl IDWriteGlyphRunAnalysis {
    pub unsafe fn GetAlphaTextureBounds(
        &self,
        texturetype: DWRITE_TEXTURE_TYPE,
    ) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAlphaTextureBounds)(
                windows_core::Interface::as_raw(self),
                texturetype,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn CreateAlphaTexture(
        &self,
        texturetype: DWRITE_TEXTURE_TYPE,
        texturebounds: *const RECT,
        alphavalues: *mut u8,
        buffersize: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateAlphaTexture)(
                windows_core::Interface::as_raw(self),
                texturetype,
                texturebounds,
                alphavalues as _,
                buffersize,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteGlyphRunAnalysis_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetAlphaTextureBounds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_TEXTURE_TYPE,
        *mut RECT,
    ) -> windows_core::HRESULT,
    pub CreateAlphaTexture: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_TEXTURE_TYPE,
        *const RECT,
        *mut u8,
        u32,
    ) -> windows_core::HRESULT,
    GetAlphaBlendParams: usize,
}
impl windows_core::RuntimeName for IDWriteGlyphRunAnalysis {}
windows_core::imp::define_interface!(
    IDWriteInlineObject,
    IDWriteInlineObject_Vtbl,
    0x8339fde3_106f_47ab_8373_1c6295eb10b3
);
windows_core::imp::interface_hierarchy!(IDWriteInlineObject, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteInlineObject_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Draw: usize,
    GetMetrics: usize,
    GetOverhangMetrics: usize,
    GetBreakConditions: usize,
}
impl windows_core::RuntimeName for IDWriteInlineObject {}
windows_core::imp::define_interface!(
    IDWritePixelSnapping,
    IDWritePixelSnapping_Vtbl,
    0xeaf3a2da_ecf4_4d24_b644_b34f6842024b
);
windows_core::imp::interface_hierarchy!(IDWritePixelSnapping, windows_core::IUnknown);
impl IDWritePixelSnapping {
    pub unsafe fn IsPixelSnappingDisabled(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsPixelSnappingDisabled)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn GetCurrentTransform(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        transform: *mut DWRITE_MATRIX,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetCurrentTransform)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                transform as _,
            )
        }
    }
    pub unsafe fn GetPixelsPerDip(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
    ) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelsPerDip)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IDWritePixelSnapping_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub IsPixelSnappingDisabled: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetCurrentTransform: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        *mut DWRITE_MATRIX,
    ) -> windows_core::HRESULT,
    pub GetPixelsPerDip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        *mut f32,
    ) -> windows_core::HRESULT,
}
pub trait IDWritePixelSnapping_Impl: windows_core::IUnknownImpl {
    fn IsPixelSnappingDisabled(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn GetCurrentTransform(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
        transform: *mut DWRITE_MATRIX,
    ) -> windows_core::Result<()>;
    fn GetPixelsPerDip(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
    ) -> windows_core::Result<f32>;
}
impl IDWritePixelSnapping_Vtbl {
    pub const fn new<Identity: IDWritePixelSnapping_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn IsPixelSnappingDisabled<
            Identity: IDWritePixelSnapping_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            isdisabled: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDWritePixelSnapping_Impl::IsPixelSnappingDisabled(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                ) {
                    Ok(ok__) => {
                        isdisabled.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentTransform<
            Identity: IDWritePixelSnapping_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            transform: *mut DWRITE_MATRIX,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDWritePixelSnapping_Impl::GetCurrentTransform(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                    core::mem::transmute_copy(&transform),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetPixelsPerDip<
            Identity: IDWritePixelSnapping_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            pixelsperdip: *mut f32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDWritePixelSnapping_Impl::GetPixelsPerDip(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                ) {
                    Ok(ok__) => {
                        pixelsperdip.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            IsPixelSnappingDisabled: IsPixelSnappingDisabled::<Identity, OFFSET>,
            GetCurrentTransform: GetCurrentTransform::<Identity, OFFSET>,
            GetPixelsPerDip: GetPixelsPerDip::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDWritePixelSnapping as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDWritePixelSnapping {}
windows_core::imp::define_interface!(
    IDWriteRenderingParams,
    IDWriteRenderingParams_Vtbl,
    0x2f0da53a_2add_47cd_82ee_d9ec34688e75
);
windows_core::imp::interface_hierarchy!(IDWriteRenderingParams, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteRenderingParams_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetGamma: usize,
    GetEnhancedContrast: usize,
    GetClearTypeLevel: usize,
    GetPixelGeometry: usize,
    GetRenderingMode: usize,
}
impl windows_core::RuntimeName for IDWriteRenderingParams {}
windows_core::imp::define_interface!(
    IDWriteTextFormat,
    IDWriteTextFormat_Vtbl,
    0x9c906818_31d7_4fd3_a151_7c5e225db55a
);
windows_core::imp::interface_hierarchy!(IDWriteTextFormat, windows_core::IUnknown);
impl IDWriteTextFormat {
    pub unsafe fn SetWordWrapping(
        &self,
        wordwrapping: DWRITE_WORD_WRAPPING,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetWordWrapping)(
                windows_core::Interface::as_raw(self),
                wordwrapping,
            )
        }
    }
    pub unsafe fn SetLineSpacing(
        &self,
        linespacingmethod: DWRITE_LINE_SPACING_METHOD,
        linespacing: f32,
        baseline: f32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetLineSpacing)(
                windows_core::Interface::as_raw(self),
                linespacingmethod,
                linespacing,
                baseline,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteTextFormat_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetTextAlignment: usize,
    SetParagraphAlignment: usize,
    pub SetWordWrapping: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_WORD_WRAPPING,
    ) -> windows_core::HRESULT,
    SetReadingDirection: usize,
    SetFlowDirection: usize,
    SetIncrementalTabStop: usize,
    SetTrimming: usize,
    pub SetLineSpacing: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DWRITE_LINE_SPACING_METHOD,
        f32,
        f32,
    ) -> windows_core::HRESULT,
    GetTextAlignment: usize,
    GetParagraphAlignment: usize,
    GetWordWrapping: usize,
    GetReadingDirection: usize,
    GetFlowDirection: usize,
    GetIncrementalTabStop: usize,
    GetTrimming: usize,
    GetLineSpacing: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
}
impl windows_core::RuntimeName for IDWriteTextFormat {}
windows_core::imp::define_interface!(
    IDWriteTextLayout,
    IDWriteTextLayout_Vtbl,
    0x53737037_6d14_410b_9bfe_0b182bb70961
);
impl core::ops::Deref for IDWriteTextLayout {
    type Target = IDWriteTextFormat;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteTextLayout,
    windows_core::IUnknown,
    IDWriteTextFormat
);
impl IDWriteTextLayout {
    pub unsafe fn Draw<P1>(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        renderer: P1,
        originx: f32,
        originy: f32,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IDWriteTextRenderer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Draw)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                renderer.param().abi(),
                originx,
                originy,
            )
        }
    }
    pub unsafe fn GetMetrics(
        &self,
        textmetrics: *mut DWRITE_TEXT_METRICS,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetMetrics)(
                windows_core::Interface::as_raw(self),
                textmetrics as _,
            )
        }
    }
    pub unsafe fn HitTestPoint(
        &self,
        pointx: f32,
        pointy: f32,
        istrailinghit: *mut windows_core::BOOL,
        isinside: *mut windows_core::BOOL,
        hittestmetrics: *mut DWRITE_HIT_TEST_METRICS,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).HitTestPoint)(
                windows_core::Interface::as_raw(self),
                pointx,
                pointy,
                istrailinghit as _,
                isinside as _,
                hittestmetrics as _,
            )
        }
    }
    pub unsafe fn HitTestTextPosition(
        &self,
        textposition: u32,
        istrailinghit: bool,
        pointx: *mut f32,
        pointy: *mut f32,
        hittestmetrics: *mut DWRITE_HIT_TEST_METRICS,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).HitTestTextPosition)(
                windows_core::Interface::as_raw(self),
                textposition,
                istrailinghit.into(),
                pointx as _,
                pointy as _,
                hittestmetrics as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteTextLayout_Vtbl {
    pub base__: IDWriteTextFormat_Vtbl,
    SetMaxWidth: usize,
    SetMaxHeight: usize,
    SetFontCollection: usize,
    SetFontFamilyName: usize,
    SetFontWeight: usize,
    SetFontStyle: usize,
    SetFontStretch: usize,
    SetFontSize: usize,
    SetUnderline: usize,
    SetStrikethrough: usize,
    SetDrawingEffect: usize,
    SetInlineObject: usize,
    SetTypography: usize,
    SetLocaleName: usize,
    GetMaxWidth: usize,
    GetMaxHeight: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetUnderline: usize,
    GetStrikethrough: usize,
    GetDrawingEffect: usize,
    GetInlineObject: usize,
    GetTypography: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
    pub Draw: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
        f32,
    ) -> windows_core::HRESULT,
    GetLineMetrics: usize,
    pub GetMetrics: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DWRITE_TEXT_METRICS,
    ) -> windows_core::HRESULT,
    GetOverhangMetrics: usize,
    GetClusterMetrics: usize,
    DetermineMinWidth: usize,
    pub HitTestPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        f32,
        *mut windows_core::BOOL,
        *mut windows_core::BOOL,
        *mut DWRITE_HIT_TEST_METRICS,
    ) -> windows_core::HRESULT,
    pub HitTestTextPosition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        windows_core::BOOL,
        *mut f32,
        *mut f32,
        *mut DWRITE_HIT_TEST_METRICS,
    ) -> windows_core::HRESULT,
    HitTestTextRange: usize,
}
impl windows_core::RuntimeName for IDWriteTextLayout {}
windows_core::imp::define_interface!(
    IDWriteTextRenderer,
    IDWriteTextRenderer_Vtbl,
    0xef8a8135_5cc6_45fe_8825_c5a0724eb819
);
impl core::ops::Deref for IDWriteTextRenderer {
    type Target = IDWritePixelSnapping;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteTextRenderer,
    windows_core::IUnknown,
    IDWritePixelSnapping
);
impl IDWriteTextRenderer {
    pub unsafe fn DrawGlyphRun<P6>(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        baselineoriginx: f32,
        baselineoriginy: f32,
        measuringmode: DWRITE_MEASURING_MODE,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        clientdrawingeffect: P6,
    ) -> windows_core::HRESULT
    where
        P6: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGlyphRun)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                baselineoriginx,
                baselineoriginy,
                measuringmode,
                glyphrun,
                glyphrundescription,
                clientdrawingeffect.param().abi(),
            )
        }
    }
    pub unsafe fn DrawUnderline<P4>(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        baselineoriginx: f32,
        baselineoriginy: f32,
        underline: *const DWRITE_UNDERLINE,
        clientdrawingeffect: P4,
    ) -> windows_core::HRESULT
    where
        P4: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawUnderline)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                baselineoriginx,
                baselineoriginy,
                underline,
                clientdrawingeffect.param().abi(),
            )
        }
    }
    pub unsafe fn DrawStrikethrough<P4>(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        baselineoriginx: f32,
        baselineoriginy: f32,
        strikethrough: *const DWRITE_STRIKETHROUGH,
        clientdrawingeffect: P4,
    ) -> windows_core::HRESULT
    where
        P4: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawStrikethrough)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                baselineoriginx,
                baselineoriginy,
                strikethrough,
                clientdrawingeffect.param().abi(),
            )
        }
    }
    pub unsafe fn DrawInlineObject<P3, P6>(
        &self,
        clientdrawingcontext: Option<*const core::ffi::c_void>,
        originx: f32,
        originy: f32,
        inlineobject: P3,
        issideways: bool,
        isrighttoleft: bool,
        clientdrawingeffect: P6,
    ) -> windows_core::HRESULT
    where
        P3: windows_core::Param<IDWriteInlineObject>,
        P6: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawInlineObject)(
                windows_core::Interface::as_raw(self),
                clientdrawingcontext.unwrap_or(core::mem::zeroed()) as _,
                originx,
                originy,
                inlineobject.param().abi(),
                issideways.into(),
                isrighttoleft.into(),
                clientdrawingeffect.param().abi(),
            )
        }
    }
}
#[repr(C)]
pub struct IDWriteTextRenderer_Vtbl {
    pub base__: IDWritePixelSnapping_Vtbl,
    pub DrawGlyphRun: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        f32,
        f32,
        DWRITE_MEASURING_MODE,
        *const DWRITE_GLYPH_RUN,
        *const DWRITE_GLYPH_RUN_DESCRIPTION,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawUnderline: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        f32,
        f32,
        *const DWRITE_UNDERLINE,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawStrikethrough: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        f32,
        f32,
        *const DWRITE_STRIKETHROUGH,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawInlineObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const core::ffi::c_void,
        f32,
        f32,
        *mut core::ffi::c_void,
        windows_core::BOOL,
        windows_core::BOOL,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IDWriteTextRenderer_Impl: IDWritePixelSnapping_Impl {
    fn DrawGlyphRun(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
        baselineoriginx: f32,
        baselineoriginy: f32,
        measuringmode: DWRITE_MEASURING_MODE,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        clientdrawingeffect: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn DrawUnderline(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
        baselineoriginx: f32,
        baselineoriginy: f32,
        underline: *const DWRITE_UNDERLINE,
        clientdrawingeffect: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn DrawStrikethrough(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
        baselineoriginx: f32,
        baselineoriginy: f32,
        strikethrough: *const DWRITE_STRIKETHROUGH,
        clientdrawingeffect: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn DrawInlineObject(
        &self,
        clientdrawingcontext: *const core::ffi::c_void,
        originx: f32,
        originy: f32,
        inlineobject: windows_core::Ref<IDWriteInlineObject>,
        issideways: windows_core::BOOL,
        isrighttoleft: windows_core::BOOL,
        clientdrawingeffect: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
}
impl IDWriteTextRenderer_Vtbl {
    pub const fn new<Identity: IDWriteTextRenderer_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn DrawGlyphRun<
            Identity: IDWriteTextRenderer_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            baselineoriginx: f32,
            baselineoriginy: f32,
            measuringmode: DWRITE_MEASURING_MODE,
            glyphrun: *const DWRITE_GLYPH_RUN,
            glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
            clientdrawingeffect: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDWriteTextRenderer_Impl::DrawGlyphRun(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                    core::mem::transmute_copy(&baselineoriginx),
                    core::mem::transmute_copy(&baselineoriginy),
                    core::mem::transmute_copy(&measuringmode),
                    core::mem::transmute_copy(&glyphrun),
                    core::mem::transmute_copy(&glyphrundescription),
                    core::mem::transmute_copy(&clientdrawingeffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DrawUnderline<
            Identity: IDWriteTextRenderer_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            baselineoriginx: f32,
            baselineoriginy: f32,
            underline: *const DWRITE_UNDERLINE,
            clientdrawingeffect: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDWriteTextRenderer_Impl::DrawUnderline(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                    core::mem::transmute_copy(&baselineoriginx),
                    core::mem::transmute_copy(&baselineoriginy),
                    core::mem::transmute_copy(&underline),
                    core::mem::transmute_copy(&clientdrawingeffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DrawStrikethrough<
            Identity: IDWriteTextRenderer_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            baselineoriginx: f32,
            baselineoriginy: f32,
            strikethrough: *const DWRITE_STRIKETHROUGH,
            clientdrawingeffect: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDWriteTextRenderer_Impl::DrawStrikethrough(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                    core::mem::transmute_copy(&baselineoriginx),
                    core::mem::transmute_copy(&baselineoriginy),
                    core::mem::transmute_copy(&strikethrough),
                    core::mem::transmute_copy(&clientdrawingeffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DrawInlineObject<
            Identity: IDWriteTextRenderer_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clientdrawingcontext: *const core::ffi::c_void,
            originx: f32,
            originy: f32,
            inlineobject: *mut core::ffi::c_void,
            issideways: windows_core::BOOL,
            isrighttoleft: windows_core::BOOL,
            clientdrawingeffect: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDWriteTextRenderer_Impl::DrawInlineObject(
                    this,
                    core::mem::transmute_copy(&clientdrawingcontext),
                    core::mem::transmute_copy(&originx),
                    core::mem::transmute_copy(&originy),
                    core::mem::transmute_copy(&inlineobject),
                    core::mem::transmute_copy(&issideways),
                    core::mem::transmute_copy(&isrighttoleft),
                    core::mem::transmute_copy(&clientdrawingeffect),
                )
                .into()
            }
        }
        Self {
            base__: IDWritePixelSnapping_Vtbl::new::<Identity, OFFSET>(),
            DrawGlyphRun: DrawGlyphRun::<Identity, OFFSET>,
            DrawUnderline: DrawUnderline::<Identity, OFFSET>,
            DrawStrikethrough: DrawStrikethrough::<Identity, OFFSET>,
            DrawInlineObject: DrawInlineObject::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDWriteTextRenderer as windows_core::Interface>::IID
            || iid == &<IDWritePixelSnapping as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDWriteTextRenderer {}
windows_core::imp::define_interface!(
    IDXGIAdapter,
    IDXGIAdapter_Vtbl,
    0x2411e7e1_12ac_4ccf_bd14_9798e8534dc0
);
impl core::ops::Deref for IDXGIAdapter {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIAdapter, windows_core::IUnknown, IDXGIObject);
impl IDXGIAdapter {
    pub unsafe fn EnumOutputs(
        &self,
        output: u32,
        ppoutput: *mut Option<IDXGIOutput>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).EnumOutputs)(
                windows_core::Interface::as_raw(self),
                output,
                core::mem::transmute(ppoutput),
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIAdapter_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    pub EnumOutputs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    GetDesc: usize,
    CheckInterfaceSupport: usize,
}
impl windows_core::RuntimeName for IDXGIAdapter {}
windows_core::imp::define_interface!(
    IDXGIAdapter1,
    IDXGIAdapter1_Vtbl,
    0x29038f61_3839_4626_91fd_086879011a05
);
impl core::ops::Deref for IDXGIAdapter1 {
    type Target = IDXGIAdapter;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIAdapter1,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIAdapter
);
impl IDXGIAdapter1 {
    pub unsafe fn GetDesc1(&self, pdesc: *mut DXGI_ADAPTER_DESC1) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesc1)(
                windows_core::Interface::as_raw(self),
                pdesc as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIAdapter1_Vtbl {
    pub base__: IDXGIAdapter_Vtbl,
    pub GetDesc1: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DXGI_ADAPTER_DESC1,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGIAdapter1 {}
windows_core::imp::define_interface!(
    IDXGIAdapter2,
    IDXGIAdapter2_Vtbl,
    0x0aa1ae0a_fa0e_4b84_8644_e05ff8e5acb5
);
impl core::ops::Deref for IDXGIAdapter2 {
    type Target = IDXGIAdapter1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIAdapter2,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIAdapter,
    IDXGIAdapter1
);
#[repr(C)]
pub struct IDXGIAdapter2_Vtbl {
    pub base__: IDXGIAdapter1_Vtbl,
    GetDesc2: usize,
}
impl windows_core::RuntimeName for IDXGIAdapter2 {}
windows_core::imp::define_interface!(
    IDXGIAdapter3,
    IDXGIAdapter3_Vtbl,
    0x645967a4_1392_4310_a798_8053ce3e93fd
);
impl core::ops::Deref for IDXGIAdapter3 {
    type Target = IDXGIAdapter2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIAdapter3,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIAdapter,
    IDXGIAdapter1,
    IDXGIAdapter2
);
impl IDXGIAdapter3 {
    pub unsafe fn QueryVideoMemoryInfo(
        &self,
        nodeindex: u32,
        memorysegmentgroup: DXGI_MEMORY_SEGMENT_GROUP,
        pvideomemoryinfo: *mut DXGI_QUERY_VIDEO_MEMORY_INFO,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).QueryVideoMemoryInfo)(
                windows_core::Interface::as_raw(self),
                nodeindex,
                memorysegmentgroup,
                pvideomemoryinfo as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIAdapter3_Vtbl {
    pub base__: IDXGIAdapter2_Vtbl,
    RegisterHardwareContentProtectionTeardownStatusEvent: usize,
    UnregisterHardwareContentProtectionTeardownStatus: usize,
    pub QueryVideoMemoryInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        DXGI_MEMORY_SEGMENT_GROUP,
        *mut DXGI_QUERY_VIDEO_MEMORY_INFO,
    ) -> windows_core::HRESULT,
    SetVideoMemoryReservation: usize,
    RegisterVideoMemoryBudgetChangeNotificationEvent: usize,
    UnregisterVideoMemoryBudgetChangeNotification: usize,
}
impl windows_core::RuntimeName for IDXGIAdapter3 {}
windows_core::imp::define_interface!(
    IDXGIDeviceSubObject,
    IDXGIDeviceSubObject_Vtbl,
    0x3d3e0379_f9de_4d58_bb6c_18d62992f1a6
);
impl core::ops::Deref for IDXGIDeviceSubObject {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIDeviceSubObject, windows_core::IUnknown, IDXGIObject);
#[repr(C)]
pub struct IDXGIDeviceSubObject_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    GetDevice: usize,
}
impl windows_core::RuntimeName for IDXGIDeviceSubObject {}
windows_core::imp::define_interface!(
    IDXGIFactory,
    IDXGIFactory_Vtbl,
    0x7b7166ec_21c7_44ae_b21a_c9ae321ae369
);
impl core::ops::Deref for IDXGIFactory {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIFactory, windows_core::IUnknown, IDXGIObject);
impl IDXGIFactory {
    pub unsafe fn MakeWindowAssociation(
        &self,
        windowhandle: HWND,
        flags: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).MakeWindowAssociation)(
                windows_core::Interface::as_raw(self),
                windowhandle,
                flags,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIFactory_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    EnumAdapters: usize,
    pub MakeWindowAssociation:
        unsafe extern "system" fn(*mut core::ffi::c_void, HWND, u32) -> windows_core::HRESULT,
    GetWindowAssociation: usize,
    CreateSwapChain: usize,
    CreateSoftwareAdapter: usize,
}
impl windows_core::RuntimeName for IDXGIFactory {}
windows_core::imp::define_interface!(
    IDXGIFactory1,
    IDXGIFactory1_Vtbl,
    0x770aae78_f26f_4dba_a829_253c83d1b387
);
impl core::ops::Deref for IDXGIFactory1 {
    type Target = IDXGIFactory;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory1,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory
);
#[repr(C)]
pub struct IDXGIFactory1_Vtbl {
    pub base__: IDXGIFactory_Vtbl,
    EnumAdapters1: usize,
    IsCurrent: usize,
}
impl windows_core::RuntimeName for IDXGIFactory1 {}
windows_core::imp::define_interface!(
    IDXGIFactory2,
    IDXGIFactory2_Vtbl,
    0x50c83a1c_e072_4c48_87b0_3630fa36a6d0
);
impl core::ops::Deref for IDXGIFactory2 {
    type Target = IDXGIFactory1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory2,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory,
    IDXGIFactory1
);
impl IDXGIFactory2 {
    pub unsafe fn CreateSwapChainForHwnd<P0, P4>(
        &self,
        pdevice: P0,
        hwnd: HWND,
        pdesc: *const DXGI_SWAP_CHAIN_DESC1,
        pfullscreendesc: Option<*const DXGI_SWAP_CHAIN_FULLSCREEN_DESC>,
        prestricttooutput: P4,
    ) -> windows_core::Result<IDXGISwapChain1>
    where
        P0: windows_core::Param<windows_core::IUnknown>,
        P4: windows_core::Param<IDXGIOutput>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSwapChainForHwnd)(
                windows_core::Interface::as_raw(self),
                pdevice.param().abi(),
                hwnd,
                pdesc,
                pfullscreendesc.unwrap_or(core::mem::zeroed()) as _,
                prestricttooutput.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDXGIFactory2_Vtbl {
    pub base__: IDXGIFactory1_Vtbl,
    IsWindowedStereoEnabled: usize,
    pub CreateSwapChainForHwnd: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        HWND,
        *const DXGI_SWAP_CHAIN_DESC1,
        *const DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateSwapChainForCoreWindow: usize,
    GetSharedResourceAdapterLuid: usize,
    RegisterStereoStatusWindow: usize,
    RegisterStereoStatusEvent: usize,
    UnregisterStereoStatus: usize,
    RegisterOcclusionStatusWindow: usize,
    RegisterOcclusionStatusEvent: usize,
    UnregisterOcclusionStatus: usize,
    CreateSwapChainForComposition: usize,
}
impl windows_core::RuntimeName for IDXGIFactory2 {}
windows_core::imp::define_interface!(
    IDXGIFactory3,
    IDXGIFactory3_Vtbl,
    0x25483823_cd46_4c7d_86ca_47aa95b837bd
);
impl core::ops::Deref for IDXGIFactory3 {
    type Target = IDXGIFactory2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory3,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory,
    IDXGIFactory1,
    IDXGIFactory2
);
#[repr(C)]
pub struct IDXGIFactory3_Vtbl {
    pub base__: IDXGIFactory2_Vtbl,
    GetCreationFlags: usize,
}
impl windows_core::RuntimeName for IDXGIFactory3 {}
windows_core::imp::define_interface!(
    IDXGIFactory4,
    IDXGIFactory4_Vtbl,
    0x1bc6ea02_ef36_464f_bf0c_21ca39e5168a
);
impl core::ops::Deref for IDXGIFactory4 {
    type Target = IDXGIFactory3;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory4,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory,
    IDXGIFactory1,
    IDXGIFactory2,
    IDXGIFactory3
);
impl IDXGIFactory4 {
    pub unsafe fn EnumWarpAdapter<T>(&self) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).EnumWarpAdapter)(
                windows_core::Interface::as_raw(self),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDXGIFactory4_Vtbl {
    pub base__: IDXGIFactory3_Vtbl,
    EnumAdapterByLuid: usize,
    pub EnumWarpAdapter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGIFactory4 {}
windows_core::imp::define_interface!(
    IDXGIFactory5,
    IDXGIFactory5_Vtbl,
    0x7632e1f5_ee65_4dca_87fd_84cd75f8838d
);
impl core::ops::Deref for IDXGIFactory5 {
    type Target = IDXGIFactory4;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory5,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory,
    IDXGIFactory1,
    IDXGIFactory2,
    IDXGIFactory3,
    IDXGIFactory4
);
impl IDXGIFactory5 {
    pub unsafe fn CheckFeatureSupport(
        &self,
        feature: DXGI_FEATURE,
        pfeaturesupportdata: *mut core::ffi::c_void,
        featuresupportdatasize: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CheckFeatureSupport)(
                windows_core::Interface::as_raw(self),
                feature,
                pfeaturesupportdata as _,
                featuresupportdatasize,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIFactory5_Vtbl {
    pub base__: IDXGIFactory4_Vtbl,
    pub CheckFeatureSupport: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DXGI_FEATURE,
        *mut core::ffi::c_void,
        u32,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGIFactory5 {}
windows_core::imp::define_interface!(
    IDXGIFactory6,
    IDXGIFactory6_Vtbl,
    0xc1b6694f_ff09_44a9_b03c_77900a0a1d17
);
impl core::ops::Deref for IDXGIFactory6 {
    type Target = IDXGIFactory5;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGIFactory6,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIFactory,
    IDXGIFactory1,
    IDXGIFactory2,
    IDXGIFactory3,
    IDXGIFactory4,
    IDXGIFactory5
);
impl IDXGIFactory6 {
    pub unsafe fn EnumAdapterByGpuPreference<T>(
        &self,
        adapter: u32,
        gpupreference: DXGI_GPU_PREFERENCE,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).EnumAdapterByGpuPreference)(
                windows_core::Interface::as_raw(self),
                adapter,
                gpupreference,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDXGIFactory6_Vtbl {
    pub base__: IDXGIFactory5_Vtbl,
    pub EnumAdapterByGpuPreference: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        DXGI_GPU_PREFERENCE,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGIFactory6 {}
windows_core::imp::define_interface!(
    IDXGIObject,
    IDXGIObject_Vtbl,
    0xaec22fb8_76f3_4639_9be0_28eb43a67a2e
);
windows_core::imp::interface_hierarchy!(IDXGIObject, windows_core::IUnknown);
#[repr(C)]
pub struct IDXGIObject_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetPrivateData: usize,
    SetPrivateDataInterface: usize,
    GetPrivateData: usize,
    GetParent: usize,
}
impl windows_core::RuntimeName for IDXGIObject {}
windows_core::imp::define_interface!(
    IDXGIOutput,
    IDXGIOutput_Vtbl,
    0xae02eedb_c735_4690_8d52_5a8dc20213aa
);
impl core::ops::Deref for IDXGIOutput {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIOutput, windows_core::IUnknown, IDXGIObject);
impl IDXGIOutput {
    pub unsafe fn GetDesc(&self, pdesc: *mut DXGI_OUTPUT_DESC) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                pdesc as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGIOutput_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    pub GetDesc: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DXGI_OUTPUT_DESC,
    ) -> windows_core::HRESULT,
    GetDisplayModeList: usize,
    FindClosestMatchingMode: usize,
    WaitForVBlank: usize,
    TakeOwnership: usize,
    ReleaseOwnership: usize,
    GetGammaControlCapabilities: usize,
    SetGammaControl: usize,
    GetGammaControl: usize,
    SetDisplaySurface: usize,
    GetDisplaySurfaceData: usize,
    GetFrameStatistics: usize,
}
impl windows_core::RuntimeName for IDXGIOutput {}
windows_core::imp::define_interface!(
    IDXGISwapChain,
    IDXGISwapChain_Vtbl,
    0x310d36a0_d2e7_4c0a_aa04_6a9d23b8886a
);
impl core::ops::Deref for IDXGISwapChain {
    type Target = IDXGIDeviceSubObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGISwapChain,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIDeviceSubObject
);
impl IDXGISwapChain {
    pub unsafe fn Present(&self, syncinterval: u32, flags: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Present)(
                windows_core::Interface::as_raw(self),
                syncinterval,
                flags,
            )
        }
    }
    pub unsafe fn GetBuffer<T>(&self, buffer: u32) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetBuffer)(
                windows_core::Interface::as_raw(self),
                buffer,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn ResizeBuffers(
        &self,
        buffercount: u32,
        width: u32,
        height: u32,
        newformat: DXGI_FORMAT,
        swapchainflags: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).ResizeBuffers)(
                windows_core::Interface::as_raw(self),
                buffercount,
                width,
                height,
                newformat,
                swapchainflags,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGISwapChain_Vtbl {
    pub base__: IDXGIDeviceSubObject_Vtbl,
    pub Present:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32) -> windows_core::HRESULT,
    pub GetBuffer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    SetFullscreenState: usize,
    GetFullscreenState: usize,
    GetDesc: usize,
    pub ResizeBuffers: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        u32,
        DXGI_FORMAT,
        u32,
    ) -> windows_core::HRESULT,
    ResizeTarget: usize,
    GetContainingOutput: usize,
    GetFrameStatistics: usize,
    GetLastPresentCount: usize,
}
impl windows_core::RuntimeName for IDXGISwapChain {}
windows_core::imp::define_interface!(
    IDXGISwapChain1,
    IDXGISwapChain1_Vtbl,
    0x790a45f7_0d42_4876_983a_0a55cfe6f4aa
);
impl core::ops::Deref for IDXGISwapChain1 {
    type Target = IDXGISwapChain;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGISwapChain1,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIDeviceSubObject,
    IDXGISwapChain
);
impl IDXGISwapChain1 {
    pub unsafe fn GetDesc1(&self, pdesc: *mut DXGI_SWAP_CHAIN_DESC1) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesc1)(
                windows_core::Interface::as_raw(self),
                pdesc as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDXGISwapChain1_Vtbl {
    pub base__: IDXGISwapChain_Vtbl,
    pub GetDesc1: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DXGI_SWAP_CHAIN_DESC1,
    ) -> windows_core::HRESULT,
    GetFullscreenDesc: usize,
    GetHwnd: usize,
    GetCoreWindow: usize,
    Present1: usize,
    IsTemporaryMonoSupported: usize,
    GetRestrictToOutput: usize,
    SetBackgroundColor: usize,
    GetBackgroundColor: usize,
    SetRotation: usize,
    GetRotation: usize,
}
impl windows_core::RuntimeName for IDXGISwapChain1 {}
windows_core::imp::define_interface!(
    IDXGISwapChain2,
    IDXGISwapChain2_Vtbl,
    0xa8be2ac4_199f_4946_b331_79599fb98de7
);
impl core::ops::Deref for IDXGISwapChain2 {
    type Target = IDXGISwapChain1;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGISwapChain2,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIDeviceSubObject,
    IDXGISwapChain,
    IDXGISwapChain1
);
impl IDXGISwapChain2 {
    pub unsafe fn SetMaximumFrameLatency(&self, maxlatency: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaximumFrameLatency)(
                windows_core::Interface::as_raw(self),
                maxlatency,
            )
        }
    }
    pub unsafe fn GetFrameLatencyWaitableObject(&self) -> HANDLE {
        unsafe {
            (windows_core::Interface::vtable(self).GetFrameLatencyWaitableObject)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
}
#[repr(C)]
pub struct IDXGISwapChain2_Vtbl {
    pub base__: IDXGISwapChain1_Vtbl,
    SetSourceSize: usize,
    GetSourceSize: usize,
    pub SetMaximumFrameLatency:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    GetMaximumFrameLatency: usize,
    pub GetFrameLatencyWaitableObject: unsafe extern "system" fn(*mut core::ffi::c_void) -> HANDLE,
    SetMatrixTransform: usize,
    GetMatrixTransform: usize,
}
impl windows_core::RuntimeName for IDXGISwapChain2 {}
windows_core::imp::define_interface!(
    IDXGISwapChain3,
    IDXGISwapChain3_Vtbl,
    0x94d99bdb_f1f8_4ab0_b236_7da0170edab1
);
impl core::ops::Deref for IDXGISwapChain3 {
    type Target = IDXGISwapChain2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGISwapChain3,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIDeviceSubObject,
    IDXGISwapChain,
    IDXGISwapChain1,
    IDXGISwapChain2
);
impl IDXGISwapChain3 {
    pub unsafe fn GetCurrentBackBufferIndex(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetCurrentBackBufferIndex)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
}
#[repr(C)]
pub struct IDXGISwapChain3_Vtbl {
    pub base__: IDXGISwapChain2_Vtbl,
    pub GetCurrentBackBufferIndex: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    CheckColorSpaceSupport: usize,
    SetColorSpace1: usize,
    ResizeBuffers1: usize,
}
impl windows_core::RuntimeName for IDXGISwapChain3 {}
pub const INFINITE: u32 = 4294967295;
windows_core::imp::define_interface!(
    IPropertyBag2,
    IPropertyBag2_Vtbl,
    0x22f55882_280b_11d0_a8a9_00a0c90c2004
);
windows_core::imp::interface_hierarchy!(IPropertyBag2, windows_core::IUnknown);
#[repr(C)]
pub struct IPropertyBag2_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Read: usize,
    Write: usize,
    CountProperties: usize,
    GetPropertyInfo: usize,
    LoadObject: usize,
}
impl windows_core::RuntimeName for IPropertyBag2 {}
windows_core::imp::define_interface!(
    ISequentialStream,
    ISequentialStream_Vtbl,
    0x0c733a30_2a1c_11ce_ade5_00aa0044773d
);
windows_core::imp::interface_hierarchy!(ISequentialStream, windows_core::IUnknown);
#[repr(C)]
pub struct ISequentialStream_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Read: usize,
    Write: usize,
}
impl windows_core::RuntimeName for ISequentialStream {}
windows_core::imp::define_interface!(
    IStream,
    IStream_Vtbl,
    0x0000000c_0000_0000_c000_000000000046
);
impl core::ops::Deref for IStream {
    type Target = ISequentialStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IStream, windows_core::IUnknown, ISequentialStream);
#[repr(C)]
pub struct IStream_Vtbl {
    pub base__: ISequentialStream_Vtbl,
    Seek: usize,
    SetSize: usize,
    CopyTo: usize,
    Commit: usize,
    Revert: usize,
    LockRegion: usize,
    UnlockRegion: usize,
    Stat: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IStream {}
windows_core::imp::define_interface!(
    IWICBitmapEncoder,
    IWICBitmapEncoder_Vtbl,
    0x00000103_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICBitmapEncoder, windows_core::IUnknown);
impl IWICBitmapEncoder {
    pub unsafe fn Initialize<P0>(
        &self,
        pistream: P0,
        cacheoption: WICBitmapEncoderCacheOption,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IStream>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                pistream.param().abi(),
                cacheoption,
            )
        }
    }
    pub unsafe fn CreateNewFrame(
        &self,
        ppiframeencode: *mut Option<IWICBitmapFrameEncode>,
        ppiencoderoptions: *mut Option<IPropertyBag2>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateNewFrame)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(ppiframeencode),
                core::mem::transmute(ppiencoderoptions),
            )
        }
    }
    pub unsafe fn Commit(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Commit)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct IWICBitmapEncoder_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Initialize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        WICBitmapEncoderCacheOption,
    ) -> windows_core::HRESULT,
    GetContainerFormat: usize,
    GetEncoderInfo: usize,
    SetColorContexts: usize,
    SetPalette: usize,
    SetThumbnail: usize,
    SetPreview: usize,
    pub CreateNewFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Commit: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    GetMetadataQueryWriter: usize,
}
impl windows_core::RuntimeName for IWICBitmapEncoder {}
windows_core::imp::define_interface!(
    IWICBitmapFrameEncode,
    IWICBitmapFrameEncode_Vtbl,
    0x00000105_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICBitmapFrameEncode, windows_core::IUnknown);
impl IWICBitmapFrameEncode {
    pub unsafe fn Initialize<P0>(&self, piencoderoptions: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IPropertyBag2>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                piencoderoptions.param().abi(),
            )
        }
    }
    pub unsafe fn SetSize(&self, uiwidth: u32, uiheight: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                uiwidth,
                uiheight,
            )
        }
    }
    pub unsafe fn SetPixelFormat(
        &self,
        ppixelformat: *mut WICPixelFormatGUID,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetPixelFormat)(
                windows_core::Interface::as_raw(self),
                ppixelformat as _,
            )
        }
    }
    pub unsafe fn WritePixels(
        &self,
        linecount: u32,
        cbstride: u32,
        cbbuffersize: u32,
        pbpixels: *const u8,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).WritePixels)(
                windows_core::Interface::as_raw(self),
                linecount,
                cbstride,
                cbbuffersize,
                pbpixels,
            )
        }
    }
    pub unsafe fn Commit(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Commit)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct IWICBitmapFrameEncode_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Initialize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, u32) -> windows_core::HRESULT,
    SetResolution: usize,
    pub SetPixelFormat: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut WICPixelFormatGUID,
    ) -> windows_core::HRESULT,
    SetColorContexts: usize,
    SetPalette: usize,
    SetThumbnail: usize,
    pub WritePixels: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        u32,
        *const u8,
    ) -> windows_core::HRESULT,
    WriteSource: usize,
    pub Commit: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    GetMetadataQueryWriter: usize,
}
impl windows_core::RuntimeName for IWICBitmapFrameEncode {}
windows_core::imp::define_interface!(
    IWICImagingFactory,
    IWICImagingFactory_Vtbl,
    0xec5ec8a9_c395_4314_9c77_54d7a935ff70
);
windows_core::imp::interface_hierarchy!(IWICImagingFactory, windows_core::IUnknown);
impl IWICImagingFactory {
    pub unsafe fn CreateEncoder(
        &self,
        guidcontainerformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICBitmapEncoder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEncoder)(
                windows_core::Interface::as_raw(self),
                guidcontainerformat,
                pguidvendor,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn CreateStream(&self) -> windows_core::Result<IWICStream> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateStream)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IWICImagingFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CreateDecoderFromFilename: usize,
    CreateDecoderFromStream: usize,
    CreateDecoderFromFileHandle: usize,
    CreateComponentInfo: usize,
    CreateDecoder: usize,
    pub CreateEncoder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreatePalette: usize,
    CreateFormatConverter: usize,
    CreateBitmapScaler: usize,
    CreateBitmapClipper: usize,
    CreateBitmapFlipRotator: usize,
    pub CreateStream: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateColorContext: usize,
    CreateColorTransformer: usize,
    CreateBitmap: usize,
    CreateBitmapFromSource: usize,
    CreateBitmapFromSourceRect: usize,
    CreateBitmapFromMemory: usize,
    CreateBitmapFromHBITMAP: usize,
    CreateBitmapFromHICON: usize,
    CreateComponentEnumerator: usize,
    CreateFastMetadataEncoderFromDecoder: usize,
    CreateFastMetadataEncoderFromFrameDecode: usize,
    CreateQueryWriter: usize,
    CreateQueryWriterFromReader: usize,
}
impl windows_core::RuntimeName for IWICImagingFactory {}
windows_core::imp::define_interface!(
    IWICStream,
    IWICStream_Vtbl,
    0x135ff860_22b7_4ddf_b0f6_218f4f299a43
);
impl core::ops::Deref for IWICStream {
    type Target = IStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICStream,
    windows_core::IUnknown,
    ISequentialStream,
    IStream
);
impl IWICStream {
    pub unsafe fn InitializeFromFilename<P0>(
        &self,
        wzfilename: P0,
        dwdesiredaccess: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InitializeFromFilename)(
                windows_core::Interface::as_raw(self),
                wzfilename.param().abi(),
                dwdesiredaccess,
            )
        }
    }
}
#[repr(C)]
pub struct IWICStream_Vtbl {
    pub base__: IStream_Vtbl,
    InitializeFromIStream: usize,
    pub InitializeFromFilename: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        u32,
    ) -> windows_core::HRESULT,
    InitializeFromMemory: usize,
    InitializeFromIStreamRegion: usize,
}
impl windows_core::RuntimeName for IWICStream {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LPARAM(pub isize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LRESULT(pub isize);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LUID {
    pub LowPart: u32,
    pub HighPart: i32,
}
pub const MAPVK_VK_TO_VSC_EX: i32 = 4;
pub const MAPVK_VSC_TO_VK_EX: i32 = 3;
pub const MDT_ANGULAR_DPI: MONITOR_DPI_TYPE = 1;
pub const MDT_DEFAULT: MONITOR_DPI_TYPE = 0;
pub const MDT_EFFECTIVE_DPI: MONITOR_DPI_TYPE = 0;
pub const MDT_RAW_DPI: MONITOR_DPI_TYPE = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MODULEENTRY32W {
    pub dwSize: u32,
    pub th32ModuleID: u32,
    pub th32ProcessID: u32,
    pub GlblcntUsage: u32,
    pub ProccntUsage: u32,
    pub modBaseAddr: *mut u8,
    pub modBaseSize: u32,
    pub hModule: HMODULE,
    pub szModule: [u16; 256],
    pub szExePath: [u16; 260],
}
impl Default for MODULEENTRY32W {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type MONITORENUMPROC = Option<
    unsafe extern "system" fn(
        param0: HMONITOR,
        param1: HDC,
        param2: *mut RECT,
        param3: LPARAM,
    ) -> windows_core::BOOL,
>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MONITORINFOEXW {
    pub Base: MONITORINFO,
    pub szDevice: [u16; 32],
}
impl Default for MONITORINFOEXW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const MONITORINFOF_PRIMARY: i32 = 1;
pub const MONITOR_DEFAULTTONEAREST: i32 = 2;
pub const MONITOR_DEFAULTTOPRIMARY: i32 = 1;
pub type MONITOR_DPI_TYPE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: windows_core::BOOL,
    pub rcPaint: RECT,
    pub fRestore: windows_core::BOOL,
    pub fIncUpdate: windows_core::BOOL,
    pub rgbReserved: [u8; 32],
}
impl Default for PAINTSTRUCT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type PEVENT_CALLBACK = Option<unsafe extern "system" fn(pevent: *mut EVENT_TRACE)>;
pub type PEVENT_FILTER_DESCRIPTOR = *mut EVENT_FILTER_DESCRIPTOR;
pub type PEVENT_HEADER_EXTENDED_DATA_ITEM = *mut EVENT_HEADER_EXTENDED_DATA_ITEM;
pub type PEVENT_RECORD_CALLBACK = Option<unsafe extern "system" fn(eventrecord: *mut EVENT_RECORD)>;
pub type PEVENT_TRACE_BUFFER_CALLBACKW =
    Option<unsafe extern "system" fn(logfile: *mut EVENT_TRACE_LOGFILEW) -> u32>;
pub type PEVENT_TRACE_PROPERTIES = *mut EVENT_TRACE_PROPERTIES;
pub const PM_REMOVE: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINTL {
    pub x: i32,
    pub y: i32,
}
pub const PRF_CLIENT: i32 = 4;
pub const PRF_ERASEBKGND: i32 = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PROCESSENTRY32W {
    pub dwSize: u32,
    pub cntUsage: u32,
    pub th32ProcessID: u32,
    pub th32DefaultHeapID: usize,
    pub th32ModuleID: u32,
    pub cntThreads: u32,
    pub th32ParentProcessID: u32,
    pub pcPriClassBase: i32,
    pub dwFlags: u32,
    pub szExeFile: [u16; 260],
}
impl Default for PROCESSENTRY32W {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PROCESSTRACE_HANDLE(pub u64);
pub type PROCESS_INFORMATION_CLASS = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_MEMORY_COUNTERS {
    pub cb: u32,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_MEMORY_COUNTERS_EX {
    pub cb: u32,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
    pub PrivateUsage: usize,
}
pub const PROCESS_POWER_THROTTLING_CURRENT_VERSION: i32 = 1;
pub const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_POWER_THROTTLING_STATE {
    pub Version: u32,
    pub ControlMask: u32,
    pub StateMask: u32,
}
pub const PROCESS_QUERY_LIMITED_INFORMATION: i32 = 4096;
pub const PROCESS_TRACE_MODE_EVENT_RECORD: i32 = 268435456;
pub const PROCESS_TRACE_MODE_RAW_TIMESTAMP: i32 = 4096;
pub const PROCESS_TRACE_MODE_REAL_TIME: i32 = 256;
pub type PROPERTY_FLAGS = i32;
pub const ProcessPowerThrottling: PROCESS_INFORMATION_CLASS = 4;
pub const PropertyHasCustomSchema: PROPERTY_FLAGS = 128;
pub const PropertyHasTags: PROPERTY_FLAGS = 64;
pub const PropertyParamCount: PROPERTY_FLAGS = 4;
pub const PropertyParamFixedCount: PROPERTY_FLAGS = 32;
pub const PropertyParamFixedLength: PROPERTY_FLAGS = 16;
pub const PropertyParamLength: PROPERTY_FLAGS = 2;
pub const PropertyStruct: PROPERTY_FLAGS = 1;
pub const PropertyWBEMXmlFragment: PROPERTY_FLAGS = 8;
pub const QDC_ONLY_ACTIVE_PATHS: i32 = 2;
pub const QS_ALLINPUT: i32 = 7423;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RECTL {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct RETURN_TYPE(pub u32);
pub const SC_KEYMENU: i32 = 61696;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: u32,
    pub lpSecurityDescriptor: *mut core::ffi::c_void,
    pub bInheritHandle: windows_core::BOOL,
}
pub const SIZE_MINIMIZED: i32 = 1;
pub const SPI_GETKEYBOARDDELAY: i32 = 22;
pub const SPI_GETKEYBOARDSPEED: i32 = 10;
pub const STILL_ACTIVE: i32 = 259;
pub const SWP_FRAMECHANGED: i32 = 32;
pub const SWP_NOACTIVATE: i32 = 16;
pub const SWP_NOMOVE: i32 = 2;
pub const SWP_NOOWNERZORDER: i32 = 512;
pub const SWP_NOSIZE: i32 = 1;
pub const SWP_NOZORDER: i32 = 4;
pub const SW_SHOW: i32 = 5;
pub const SW_SHOWNA: i32 = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TDHSTATUS(pub u32);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TDH_CONTEXT {
    pub ParameterValue: u64,
    pub ParameterType: TDH_CONTEXT_TYPE,
    pub ParameterSize: u32,
}
pub type TDH_CONTEXT_TYPE = i32;
pub const TDH_INTYPE_ANSICHAR: _TDH_IN_TYPE = 307;
pub const TDH_INTYPE_ANSISTRING: _TDH_IN_TYPE = 2;
pub const TDH_INTYPE_BINARY: _TDH_IN_TYPE = 14;
pub const TDH_INTYPE_BOOLEAN: _TDH_IN_TYPE = 13;
pub const TDH_INTYPE_COUNTEDANSISTRING: _TDH_IN_TYPE = 301;
pub const TDH_INTYPE_COUNTEDSTRING: _TDH_IN_TYPE = 300;
pub const TDH_INTYPE_DOUBLE: _TDH_IN_TYPE = 12;
pub const TDH_INTYPE_FILETIME: _TDH_IN_TYPE = 17;
pub const TDH_INTYPE_FLOAT: _TDH_IN_TYPE = 11;
pub const TDH_INTYPE_GUID: _TDH_IN_TYPE = 15;
pub const TDH_INTYPE_HEXDUMP: _TDH_IN_TYPE = 309;
pub const TDH_INTYPE_HEXINT32: _TDH_IN_TYPE = 20;
pub const TDH_INTYPE_HEXINT64: _TDH_IN_TYPE = 21;
pub const TDH_INTYPE_INT16: _TDH_IN_TYPE = 5;
pub const TDH_INTYPE_INT32: _TDH_IN_TYPE = 7;
pub const TDH_INTYPE_INT64: _TDH_IN_TYPE = 9;
pub const TDH_INTYPE_INT8: _TDH_IN_TYPE = 3;
pub const TDH_INTYPE_MANIFEST_COUNTEDANSISTRING: _TDH_IN_TYPE = 23;
pub const TDH_INTYPE_MANIFEST_COUNTEDBINARY: _TDH_IN_TYPE = 25;
pub const TDH_INTYPE_MANIFEST_COUNTEDSTRING: _TDH_IN_TYPE = 22;
pub const TDH_INTYPE_NONNULLTERMINATEDANSISTRING: _TDH_IN_TYPE = 305;
pub const TDH_INTYPE_NONNULLTERMINATEDSTRING: _TDH_IN_TYPE = 304;
pub const TDH_INTYPE_NULL: _TDH_IN_TYPE = 0;
pub const TDH_INTYPE_POINTER: _TDH_IN_TYPE = 16;
pub const TDH_INTYPE_RESERVED24: _TDH_IN_TYPE = 24;
pub const TDH_INTYPE_REVERSEDCOUNTEDANSISTRING: _TDH_IN_TYPE = 303;
pub const TDH_INTYPE_REVERSEDCOUNTEDSTRING: _TDH_IN_TYPE = 302;
pub const TDH_INTYPE_SID: _TDH_IN_TYPE = 19;
pub const TDH_INTYPE_SIZET: _TDH_IN_TYPE = 308;
pub const TDH_INTYPE_SYSTEMTIME: _TDH_IN_TYPE = 18;
pub const TDH_INTYPE_UINT16: _TDH_IN_TYPE = 6;
pub const TDH_INTYPE_UINT32: _TDH_IN_TYPE = 8;
pub const TDH_INTYPE_UINT64: _TDH_IN_TYPE = 10;
pub const TDH_INTYPE_UINT8: _TDH_IN_TYPE = 4;
pub const TDH_INTYPE_UNICODECHAR: _TDH_IN_TYPE = 306;
pub const TDH_INTYPE_UNICODESTRING: _TDH_IN_TYPE = 1;
pub const TDH_INTYPE_WBEMSID: _TDH_IN_TYPE = 310;
pub type TEMPLATE_FLAGS = i32;
pub const TH32CS_SNAPMODULE: i32 = 8;
pub const TH32CS_SNAPPROCESS: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TIME_ZONE_INFORMATION {
    pub Bias: i32,
    pub StandardName: [u16; 32],
    pub StandardDate: SYSTEMTIME,
    pub StandardBias: i32,
    pub DaylightName: [u16; 32],
    pub DaylightDate: SYSTEMTIME,
    pub DaylightBias: i32,
}
impl Default for TIME_ZONE_INFORMATION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TRACE_EVENT_INFO {
    pub ProviderGuid: windows_core::GUID,
    pub EventGuid: windows_core::GUID,
    pub EventDescriptor: EVENT_DESCRIPTOR,
    pub DecodingSource: DECODING_SOURCE,
    pub ProviderNameOffset: u32,
    pub LevelNameOffset: u32,
    pub ChannelNameOffset: u32,
    pub KeywordsNameOffset: u32,
    pub TaskNameOffset: u32,
    pub OpcodeNameOffset: u32,
    pub EventMessageOffset: u32,
    pub ProviderMessageOffset: u32,
    pub BinaryXMLOffset: u32,
    pub BinaryXMLSize: u32,
    pub Anonymous: TRACE_EVENT_INFO_0,
    pub Anonymous2: TRACE_EVENT_INFO_1,
    pub PropertyCount: u32,
    pub TopLevelPropertyCount: u32,
    pub Anonymous3: TRACE_EVENT_INFO_2,
    pub EventPropertyInfoArray: [EVENT_PROPERTY_INFO; 1],
}
impl Default for TRACE_EVENT_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_0 {
    pub EventNameOffset: u32,
    pub ActivityIDNameOffset: u32,
}
impl Default for TRACE_EVENT_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_1 {
    pub EventAttributesOffset: u32,
    pub RelatedActivityIDNameOffset: u32,
}
impl Default for TRACE_EVENT_INFO_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_2 {
    pub Flags: TEMPLATE_FLAGS,
    pub Anonymous: TRACE_EVENT_INFO_2_0,
}
impl Default for TRACE_EVENT_INFO_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_EVENT_INFO_2_0 {
    pub _bitfield: u32,
}
impl TRACE_EVENT_INFO_2_0 {
    pub fn Reserved(&self) -> u32 {
        (self._bitfield << 28) >> 28
    }
    pub fn set_Reserved(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !15) | (value & 15);
    }
    pub fn Tags(&self) -> u32 {
        self._bitfield >> 4
    }
    pub fn set_Tags(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(268435455 << 4)) | ((value & 268435455) << 4);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TRACE_LOGFILE_HEADER {
    pub BufferSize: u32,
    pub Anonymous: TRACE_LOGFILE_HEADER_0,
    pub ProviderVersion: u32,
    pub NumberOfProcessors: u32,
    pub EndTime: i64,
    pub TimerResolution: u32,
    pub MaximumFileSize: u32,
    pub LogFileMode: u32,
    pub BuffersWritten: u32,
    pub Anonymous2: TRACE_LOGFILE_HEADER_1,
    pub LoggerName: windows_core::PWSTR,
    pub LogFileName: windows_core::PWSTR,
    pub TimeZone: TIME_ZONE_INFORMATION,
    pub BootTime: i64,
    pub PerfFreq: i64,
    pub StartTime: i64,
    pub ReservedFlags: u32,
    pub BuffersLost: u32,
}
impl Default for TRACE_LOGFILE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_LOGFILE_HEADER_0 {
    pub Version: u32,
    pub VersionDetail: TRACE_LOGFILE_HEADER_0_0,
}
impl Default for TRACE_LOGFILE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_LOGFILE_HEADER_0_0 {
    pub MajorVersion: u8,
    pub MinorVersion: u8,
    pub SubVersion: u8,
    pub SubMinorVersion: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_LOGFILE_HEADER_1 {
    pub LogInstanceGuid: windows_core::GUID,
    pub Anonymous: TRACE_LOGFILE_HEADER_1_0,
}
impl Default for TRACE_LOGFILE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_LOGFILE_HEADER_1_0 {
    pub StartBuffers: u32,
    pub PointerSize: u32,
    pub EventsLost: u32,
    pub CpuSpeedInMHz: u32,
}
pub const USER_DEFAULT_SCREEN_DPI: i32 = 96;
pub const WAIT_OBJECT_0: i32 = 0;
pub const WHEEL_DELTA: i32 = 120;
pub const WICBITMAPENCODERCACHEOPTION_FORCE_DWORD: WICBitmapEncoderCacheOption = 2147483647;
pub const WICBitmapEncoderCacheInMemory: WICBitmapEncoderCacheOption = 0;
pub type WICBitmapEncoderCacheOption = i32;
pub const WICBitmapEncoderCacheTempFile: WICBitmapEncoderCacheOption = 1;
pub const WICBitmapEncoderNoCache: WICBitmapEncoderCacheOption = 2;
pub type WICPixelFormatGUID = windows_core::GUID;
pub const WM_CLOSE: i32 = 16;
pub const WM_DISPLAYCHANGE: i32 = 126;
pub const WM_DPICHANGED: i32 = 736;
pub const WM_ERASEBKGND: i32 = 20;
pub const WM_KILLFOCUS: i32 = 8;
pub const WM_MOVE: i32 = 3;
pub const WM_PAINT: i32 = 15;
pub const WM_PRINT: i32 = 791;
pub const WM_QUIT: i32 = 18;
pub const WM_SETCURSOR: i32 = 32;
pub const WM_SETFOCUS: i32 = 7;
pub const WM_SETTINGCHANGE: i32 = 26;
pub const WM_SIZE: i32 = 5;
pub const WM_SYSCOMMAND: i32 = 274;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: windows_core::PCWSTR,
    pub lpszClassName: windows_core::PCWSTR,
    pub hIconSm: HICON,
}
pub type WNDPROC = Option<
    unsafe extern "system" fn(param0: HWND, param1: u32, param2: WPARAM, param3: LPARAM) -> LRESULT,
>;
pub const WNODE_FLAG_TRACED_GUID: i32 = 131072;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WNODE_HEADER {
    pub BufferSize: u32,
    pub ProviderId: u32,
    pub Anonymous: WNODE_HEADER_0,
    pub Anonymous2: WNODE_HEADER_1,
    pub Guid: windows_core::GUID,
    pub ClientContext: u32,
    pub Flags: u32,
}
impl Default for WNODE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union WNODE_HEADER_0 {
    pub HistoricalContext: u64,
    pub Anonymous: WNODE_HEADER_0_0,
}
impl Default for WNODE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WNODE_HEADER_0_0 {
    pub Version: u32,
    pub Linkage: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union WNODE_HEADER_1 {
    pub CountLost: u32,
    pub KernelHandle: HANDLE,
    pub TimeStamp: i64,
}
impl Default for WNODE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WPARAM(pub usize);
pub const WS_EX_APPWINDOW: i32 = 262144;
pub const WS_EX_NOACTIVATE: i32 = 134217728;
pub const WS_MAXIMIZE: i32 = 16777216;
pub const WS_OVERLAPPEDWINDOW: i32 = 13565952;
pub const WS_POPUP: u32 = 2147483648;
pub const WS_VISIBLE: i32 = 268435456;
pub type _TDH_IN_TYPE = i32;
