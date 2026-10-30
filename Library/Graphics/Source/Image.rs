// Decodes image files, like PNGs, with the Windows Imaging Component, which
// Windows has built in, so that no crate is needed for them.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_api::*;
use windows_core::{Error, Interface, PCWSTR, Result};

// Rows of pixels, top to bottom, 4 bytes each: red, green, blue and alpha.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn load_image(path: &Path) -> Result<Image> {
    unsafe {
        // WIC is COM. S_FALSE means COM was already initialized on this
        // thread, which still needs a CoUninitialize. If it was initialized
        // in another mode, WIC works in that one, and this leaves it alone.
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED as u32).is_ok();
        // Every WIC object is released before COM is uninitialized.
        let image = decode(path);
        if initialized {
            CoUninitialize();
        }
        image
    }
}

unsafe fn decode(path: &Path) -> Result<Image> {
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let decoder = factory
            .CreateDecoderFromFilename(
                PCWSTR(wide.as_ptr()),
                std::ptr::null(),
                GENERIC_READ,
                WICDecodeMetadataCacheOnLoad,
            )
            .map_err(|error| {
                let message = format!("couldn't decode {}: {}", path.display(), error.message());
                Error::new(error.code(), message)
            })?;
        let frame = decoder.GetFrame(0)?;
        let converter = factory.CreateFormatConverter()?;
        converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppRGBA,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeCustom,
        )
        .ok()?;
        let (mut width, mut height) = (0, 0);
        converter.GetSize(&mut width, &mut height).ok()?;
        let stride = width as usize * 4;
        let mut pixels = vec![0u8; stride * height as usize];
        // The generated CopyPixels takes no buffer, as the metadata doesn't
        // say how big it is, so this calls the method through the vtable.
        let source: IWICBitmapSource = converter.cast()?;
        (source.vtable().CopyPixels)(
            source.as_raw(),
            std::ptr::null(),
            stride as u32,
            pixels.len() as u32,
            pixels.as_mut_ptr(),
        )
        .ok()?;
        Ok(Image {
            width,
            height,
            pixels,
        })
    }
}
