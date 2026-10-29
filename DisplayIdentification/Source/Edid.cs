using System.Buffers.Binary;
using System.Text;
using Microsoft.Win32;

namespace DisplayIdentification;

// Identity information from a monitor's EDID (VESA Extended Display Identification Data) block.
internal sealed record Edid(string ManufacturerId, ushort ProductCode, uint SerialNumber, string SerialText, string MonitorName) {
    private const int BaseBlockLength = 128;
    private const byte SerialTextTag = 0xFF;
    private const byte MonitorNameTag = 0xFC;

    private static ReadOnlySpan<byte> Header => [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

    // Reads the EDID that Windows keeps in the registry for a monitor device path such as
    // \\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8450#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}.
    internal static Edid? FromDevicePath(string devicePath) {
        string[] parts = devicePath.Split('#');
        if (parts.Length < 3 || !parts[0].StartsWith(@"\\?\", StringComparison.Ordinal)) {
            return null;
        }

        string keyPath = $@"SYSTEM\CurrentControlSet\Enum\{parts[0][4..]}\{parts[1]}\{parts[2]}\Device Parameters";
        using RegistryKey? key = Registry.LocalMachine.OpenSubKey(keyPath);
        return key?.GetValue("EDID") is byte[] data ? Parse(data) : null;
    }

    internal static Edid? Parse(byte[] data) {
        if (data.Length < BaseBlockLength || !data.AsSpan(0, Header.Length).SequenceEqual(Header)) {
            return null;
        }

        // Three letters of five bits each, where 1 is A.
        int manufacturer = (data[8] << 8) | data[9];
        string manufacturerId = $"{Letter(manufacturer >> 10)}{Letter(manufacturer >> 5)}{Letter(manufacturer)}";
        ushort productCode = BinaryPrimitives.ReadUInt16LittleEndian(data.AsSpan(10));
        uint serialNumber = BinaryPrimitives.ReadUInt32LittleEndian(data.AsSpan(12));

        // Four 18-byte descriptors. Display descriptors start with three zero bytes, followed by a tag.
        string serialText = "";
        string monitorName = "";
        for (int offset = 54; offset <= 108; offset += 18) {
            if (data[offset] != 0 || data[offset + 1] != 0 || data[offset + 2] != 0) {
                continue;
            }

            string text = DescriptorText(data.AsSpan(offset + 5, 13));
            if (data[offset + 3] == SerialTextTag) {
                serialText = text;
            } else if (data[offset + 3] == MonitorNameTag) {
                monitorName = text;
            }
        }

        return new Edid(manufacturerId, productCode, serialNumber, serialText, monitorName);
    }

    private static char Letter(int bits) {
        return (char)('A' - 1 + (bits & 0x1F));
    }

    // Descriptor text ends with a line feed and is padded with spaces.
    private static string DescriptorText(ReadOnlySpan<byte> bytes) {
        int end = bytes.IndexOf((byte)'\n');
        return Encoding.ASCII.GetString(end >= 0 ? bytes[..end] : bytes).Trim();
    }
}
