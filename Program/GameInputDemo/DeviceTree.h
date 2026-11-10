// Where a device is in Windows' device tree, from the PnP path that GameInput
// gives: its node, and the nodes above it, which say what it's connected
// through, like a USB hub, or a driver that makes it up from another device.
// Windows' configuration manager answers from what Plug and Play already
// knows, without asking the device, but it still takes a few milliseconds
// for a device, so a thread of its own looks each one up.

#pragma once

#include <expected>
#include <memory>
#include <string>
#include <vector>

namespace game_input_demo {

struct Node;

// A device's nodes, nearest first, or why there are none, once they're
// looked up. Copies share the lookup.
class Tree {
public:
    // Nodes that aren't known, and that nothing looks up.
    Tree();

    // Starts looking up the nodes for a PnP path, on another thread.
    static Tree look_up(std::string pnp_path);

    // Nodes that are already known, or why there are none.
    static Tree known(std::expected<std::vector<Node>, std::string> result);

    // The nodes, or why there are none, or null while they're looked up.
    const std::expected<std::vector<Node>, std::string>* get() const;

    bool operator==(const Tree& other) const;

private:
    // The result, once it's there, which the thread that looks it up sets.
    struct Lookup;

    std::shared_ptr<Lookup> lookup;
};

// A node in the device tree, with what Device Manager shows about it. Each
// is empty when Windows doesn't have it.
struct Node {
    // Like "HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000".
    std::string instance_id;
    // Its friendly name, or else its description, like "USB Input Device".
    std::string name;
    // The name that the device itself reports on its bus, like a USB
    // device's product string.
    std::string bus_name;
    std::string manufacturer;
    // Its setup class, like "Keyboard" or "HIDClass".
    std::string class_;
    // The driver's service, like "kbdhid", and the INF file that installed it.
    std::string service;
    std::string inf;
    // Where it is on its bus, like "Port_#0001.Hub_#0002".
    std::string location;
    std::vector<std::string> hardware_ids;

    bool operator==(const Node&) const = default;
};

} // namespace game_input_demo
