#include "GameInputDemo/DeviceTree.h"

#include "Base/Text.h"
#include "Base/Windows.h"

// Before devpkey.h, which then defines the property keys that it names,
// instead of only declaring them.
#include <initguid.h>

#include <cfgmgr32.h>
#include <devpkey.h>

#include <algorithm>
#include <array>
#include <atomic>
#include <cstddef>
#include <format>
#include <optional>
#include <span>
#include <string_view>
#include <thread>
#include <utility>

#pragma comment(lib, "cfgmgr32.lib")

namespace game_input_demo {

// What OnceLock is in Rust: a value that's set once, by one thread, and read
// by others once it's there.
struct Tree::Lookup {
    std::optional<std::expected<std::vector<Node>, std::string>> result;
    // Whether `result` is there, which it then stays.
    std::atomic<bool> set = false;
};

namespace {

// Nodes in these classes are the computer and its buses, which every device
// is connected through, so the walk up the tree stops at the first.
constexpr std::array<std::string_view, 2> SystemClasses{"System", "Computer"};

std::string describe(CONFIGRET code) {
    return std::format("CONFIGRET {}", code);
}

// UTF-16 that ends in a zero, as a std::string.
std::string text(std::span<const wchar_t> units) {
    size_t end = static_cast<size_t>(std::ranges::find(units, L'\0') - units.begin());
    return base::narrow(std::wstring_view(units.data(), end));
}

// Reads a property of a string type into `buffer`, as UTF-16 that ends in a
// zero, or a list of such strings that ends in another. `get` is the
// configuration manager's function for it, which first says how big it is.
template <class Get> std::expected<void, std::string> read_property(std::vector<wchar_t>& buffer, Get&& get) {
    DEVPROPTYPE kind = 0;
    ULONG size = 0;
    CONFIGRET code = get(&kind, nullptr, &size);
    if (code != CR_BUFFER_SMALL) {
        return std::unexpected(describe(code));
    }
    buffer.resize((static_cast<size_t>(size) + 1) / 2, 0);
    code = get(&kind, reinterpret_cast<PBYTE>(buffer.data()), &size);
    if (code != CR_SUCCESS) {
        return std::unexpected(describe(code));
    }
    if (kind != DEVPROP_TYPE_STRING && kind != DEVPROP_TYPE_STRING_LIST) {
        return std::unexpected(std::format("property of type {}", kind));
    }
    return {};
}

// A node's property that's a string or a list of them, or none if it
// doesn't have it.
std::vector<std::string> strings(DEVINST node, const DEVPROPKEY& key) {
    std::vector<wchar_t> buffer;
    std::expected<void, std::string> read =
        read_property(buffer, [&](DEVPROPTYPE* kind, PBYTE data, ULONG* size) { return CM_Get_DevNode_PropertyW(node, &key, kind, data, size, 0); });
    std::vector<std::string> items;
    if (!read) {
        return items;
    }
    size_t start = 0;
    for (size_t index = 0; index <= buffer.size(); index++) {
        if (index == buffer.size() || buffer[index] == 0) {
            if (index > start) {
                items.push_back(base::narrow(std::wstring_view(buffer.data() + start, index - start)));
            }
            start = index + 1;
        }
    }
    return items;
}

// A node's property that's a string, or "" if it doesn't have it.
std::string string(DEVINST node, const DEVPROPKEY& key) {
    std::vector<std::string> items = strings(node, key);
    return items.empty() ? std::string() : std::move(items.front());
}

// The device's node, then the nodes above it, nearest first, up to the first
// one in the System or Computer class. Or why Windows doesn't know it.
std::expected<std::vector<Node>, std::string> nodes(std::string_view pnp_path) {
    if (pnp_path.empty()) {
        return std::unexpected(std::string("GameInput gives no PnP path"));
    }
    std::wstring path = base::widen(pnp_path);
    std::vector<wchar_t> instance_id;
    std::expected<void, std::string> read = read_property(instance_id, [&](DEVPROPTYPE* kind, PBYTE buffer, ULONG* size) {
        return CM_Get_Device_Interface_PropertyW(path.c_str(), &DEVPKEY_Device_InstanceId, kind, buffer, size, 0);
    });
    if (!read) {
        return std::unexpected(std::format("Windows has no device interface at the PnP path ({})", read.error()));
    }
    DEVINST node = 0;
    CONFIGRET code = CM_Locate_DevNodeW(&node, instance_id.data(), CM_LOCATE_DEVNODE_NORMAL);
    if (code != CR_SUCCESS) {
        return std::unexpected(std::format("Windows has no device {} ({})", text(instance_id), describe(code)));
    }

    std::vector<Node> found;
    DEVINST current = node;
    for (;;) {
        std::string class_ = string(current, DEVPKEY_Device_Class);
        if (!found.empty() && std::ranges::contains(SystemClasses, std::string_view(class_))) {
            break;
        }
        std::string friendly_name = string(current, DEVPKEY_Device_FriendlyName);
        found.push_back(Node{
            .instance_id = string(current, DEVPKEY_Device_InstanceId),
            .name = friendly_name.empty() ? string(current, DEVPKEY_Device_DeviceDesc) : std::move(friendly_name),
            .bus_name = string(current, DEVPKEY_Device_BusReportedDeviceDesc),
            .manufacturer = string(current, DEVPKEY_Device_Manufacturer),
            .class_ = std::move(class_),
            .service = string(current, DEVPKEY_Device_Service),
            .inf = string(current, DEVPKEY_Device_DriverInfPath),
            .location = string(current, DEVPKEY_Device_LocationInfo),
            .hardware_ids = strings(current, DEVPKEY_Device_HardwareIds),
        });
        // The root of the tree has no parent.
        DEVINST parent = 0;
        if (CM_Get_Parent(&parent, current, 0) != CR_SUCCESS) {
            break;
        }
        current = parent;
    }
    return found;
}

} // namespace

Tree::Tree() : lookup(std::make_shared<Lookup>()) {}

Tree Tree::look_up(std::string pnp_path) {
    Tree tree;
    std::thread([shared = tree.lookup, pnp_path = std::move(pnp_path)] {
        shared->result.emplace(nodes(pnp_path));
        shared->set.store(true, std::memory_order_release);
    }).detach();
    return tree;
}

Tree Tree::known(std::expected<std::vector<Node>, std::string> result) {
    Tree tree;
    tree.lookup->result.emplace(std::move(result));
    tree.lookup->set.store(true, std::memory_order_release);
    return tree;
}

const std::expected<std::vector<Node>, std::string>* Tree::get() const {
    return lookup->set.load(std::memory_order_acquire) ? &*lookup->result : nullptr;
}

bool Tree::operator==(const Tree& other) const {
    if (lookup == other.lookup) {
        return true;
    }
    const std::expected<std::vector<Node>, std::string>* mine = get();
    const std::expected<std::vector<Node>, std::string>* theirs = other.get();
    if (mine == nullptr || theirs == nullptr) {
        return mine == theirs;
    }
    return *mine == *theirs;
}

} // namespace game_input_demo
