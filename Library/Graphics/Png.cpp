#include "Graphics/Png.h"

#include "Base/Windows.h"

#include <cstdint>
#include <objbase.h>
#include <vector>
#include <wincodec.h>
#include <wrl/client.h>

#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "windowscodecs.lib")

namespace graphics {

void write_png(const std::filesystem::path& path, const Image& image) {
    // Fails harmlessly if the thread already chose its apartment.
    (void)CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    ComPtr<IWICImagingFactory> factory;
    // The factory's first version, by its own name: from Windows 8 on, the
    // SDK's CLSID_WICImagingFactory is the second's, which this doesn't need.
    base::check(CoCreateInstance(CLSID_WICImagingFactory1, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&factory)));
    ComPtr<IWICStream> stream;
    base::check(factory->CreateStream(&stream));
    base::check(stream->InitializeFromFilename(path.c_str(), GENERIC_WRITE));
    ComPtr<IWICBitmapEncoder> encoder;
    base::check(factory->CreateEncoder(GUID_ContainerFormatPng, nullptr, &encoder));
    base::check(encoder->Initialize(stream.Get(), WICBitmapEncoderNoCache));
    ComPtr<IWICBitmapFrameEncode> frame;
    ComPtr<IPropertyBag2> options;
    base::check(encoder->CreateNewFrame(&frame, &options));
    base::check(frame->Initialize(options.Get()));
    base::check(frame->SetSize(image.width, image.height));
    WICPixelFormatGUID format = GUID_WICPixelFormat24bppBGR;
    base::check(frame->SetPixelFormat(&format));
    std::uint32_t stride = image.width * 3;
    std::vector<std::uint8_t> rows;
    rows.reserve(static_cast<size_t>(stride * image.height));
    const std::uint8_t* pixels = image.pixels.data();
    for (size_t pixel = 0; pixel + 4 <= image.pixels.size(); pixel += 4) {
        rows.insert(rows.end(), pixels + pixel, pixels + pixel + 3);
    }
    base::check(frame->WritePixels(image.height, stride, static_cast<UINT>(rows.size()), rows.data()));
    base::check(frame->Commit());
    base::check(encoder->Commit());
}

} // namespace graphics
