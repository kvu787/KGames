// Writes images as PNGs, with the Windows Imaging Component.

use std::path::Path;

use windows_api::*;
use windows_core::{HSTRING, Result};

use crate::Image;

// Without alpha, like a screenshot of a window.
pub fn write_png(path: &Path, image: &Image) -> Result<()> {
    unsafe {
        // Fails harmlessly if the thread already chose its apartment.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED as u32);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let stream = factory.CreateStream()?;
        stream
            .InitializeFromFilename(&HSTRING::from(path.as_os_str()), GENERIC_WRITE as u32)
            .ok()?;
        let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache).ok()?;
        let mut frame = None;
        let mut options = None;
        encoder.CreateNewFrame(&mut frame, &mut options).ok()?;
        let frame = frame.unwrap();
        frame.Initialize(options.as_ref()).ok()?;
        frame.SetSize(image.width, image.height).ok()?;
        let mut format = GUID_WICPixelFormat24bppBGR;
        frame.SetPixelFormat(&mut format).ok()?;
        let stride = image.width * 3;
        let mut rows = Vec::with_capacity((stride * image.height) as usize);
        for pixel in image.pixels.as_chunks::<4>().0 {
            rows.extend_from_slice(&pixel[..3]);
        }
        frame.WritePixels(image.height, stride, rows.len() as u32, rows.as_ptr()).ok()?;
        frame.Commit().ok()?;
        encoder.Commit().ok()?;
        Ok(())
    }
}

