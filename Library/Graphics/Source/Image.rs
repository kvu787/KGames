// Encodes PNG files with the Windows Imaging Component, which Windows has
// built in, so that no crate is needed for them.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_api::*;
use windows_core::{Error, Interface, PCWSTR, Result};

// Writes rows of pixels, top to bottom, 3 bytes each, blue, green and red,
// to a PNG file, like a captured frame.
pub fn save_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<()> {
    assert_eq!(pixels.len(), width as usize * height as usize * 3);
    unsafe {
        // WIC is COM. S_FALSE means COM was already initialized on this
        // thread, which still needs a CoUninitialize. If it was initialized
        // in another mode, WIC works in that one, and this leaves it alone.
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED as u32).is_ok();
        // Every WIC object is released before COM is uninitialized.
        let result = encode(path, width, height, pixels);
        if initialized {
            CoUninitialize();
        }
        result.map_err(|error| {
            let message = format!("couldn't save {}: {}", path.display(), error.message());
            Error::new(error.code(), message)
        })
    }
}

unsafe fn encode(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<()> {
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let stream = factory.CreateStream()?;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        stream
            .InitializeFromFilename(PCWSTR(wide.as_ptr()), GENERIC_WRITE as u32)
            .ok()?;
        let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
        encoder
            .Initialize(&stream.cast::<IStream>()?, WICBitmapEncoderNoCache)
            .ok()?;
        let mut frame = None;
        let mut options = None;
        encoder.CreateNewFrame(&mut frame, &mut options).ok()?;
        let frame = frame.ok_or_else(|| Error::new(E_FAIL, "no frame to encode into"))?;
        frame.Initialize(options.as_ref()).ok()?;
        frame.SetSize(width, height).ok()?;
        let mut format = GUID_WICPixelFormat24bppBGR;
        frame.SetPixelFormat(&mut format).ok()?;
        if format != GUID_WICPixelFormat24bppBGR {
            return Err(Error::new(E_FAIL, "the PNG encoder doesn't take 24-bit BGR"));
        }
        let stride = width * 3;
        frame
            .WritePixels(height, stride, pixels.len() as u32, pixels.as_ptr())
            .ok()?;
        frame.Commit().ok()?;
        encoder.Commit().ok()
    }
}
