// The GameInput v4 API, from Microsoft's own header, native/include/GameInput.h
// in the Microsoft.GameInput NuGet package that NuGetPackages.proj declares,
// which is the package's v4/GameInput.h, and create, which starts GameInput
// with the package's GameInput.lib.
//
// The header puts the API in the namespace GameInput::v4, and this one names
// all of it too, so game_input::IGameInput is GameInput::v4::IGameInput, and
// game_input::GameInputKindKeyboard is GameInput::v4::GameInputKindKeyboard.
// Its interfaces are COM's, held in a Microsoft::WRL::ComPtr. What the header
// doesn't say about the methods that KGames calls:
//
// - IGameInputReading::GetDevice hands the device back with a reference
//   already added, so it's owned.
// - IGameInputDevice::GetDisplayName returns the size in bytes of the UTF-8
//   name, including its terminating zero. Pass a null buffer and a size of 0
//   to ask for the size. GetPnpPath is like it, for the device interface's
//   path, which Windows' configuration manager functions take.
// - GameInput's objects are free-threaded: device callbacks hand devices
//   over from GameInput's own threads, and the main thread then calls into
//   them.
//
// What the header leaves out for the structs that KGames compares is here
// too: ==, field by field. Test/GameInput pins that none is left out. A
// struct of GameInput's made from nothing, like GameInputKeyState{}, is all
// zeros.

#pragma once

#include "Base/Windows.h"

#include "GameInput/Installation.h"

#include <GameInput.h>
#include <wrl/client.h>

#include <array>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <string>
#include <utility>

// These are in GameInput's namespace, which is where a comparison of its
// structs looks for them, whatever namespace the comparison is in.
namespace GameInput::v4 {

inline bool operator==(const GameInputKeyState& left, const GameInputKeyState& right) {
    return left.scanCode == right.scanCode && left.codePoint == right.codePoint && left.virtualKey == right.virtualKey && left.isDeadKey == right.isDeadKey;
}

inline bool operator==(const GameInputMouseState& left, const GameInputMouseState& right) {
    return left.buttons == right.buttons && left.positions == right.positions && left.positionX == right.positionX && left.positionY == right.positionY &&
           left.absolutePositionX == right.absolutePositionX && left.absolutePositionY == right.absolutePositionY && left.wheelX == right.wheelX && left.wheelY == right.wheelY;
}

inline bool operator==(const GameInputGamepadState& left, const GameInputGamepadState& right) {
    return left.buttons == right.buttons && left.leftTrigger == right.leftTrigger && left.rightTrigger == right.rightTrigger && left.leftThumbstickX == right.leftThumbstickX &&
           left.leftThumbstickY == right.leftThumbstickY && left.rightThumbstickX == right.rightThumbstickX && left.rightThumbstickY == right.rightThumbstickY;
}

inline bool operator==(const GameInputVersion& left, const GameInputVersion& right) {
    return left.major == right.major && left.minor == right.minor && left.build == right.build && left.revision == right.revision;
}

inline bool operator==(const GameInputUsage& left, const GameInputUsage& right) {
    return left.page == right.page && left.id == right.id;
}

inline bool operator==(const GameInputKeyboardInfo& left, const GameInputKeyboardInfo& right) {
    return left.kind == right.kind && left.layout == right.layout && left.keyCount == right.keyCount && left.functionKeyCount == right.functionKeyCount &&
           left.maxSimultaneousKeys == right.maxSimultaneousKeys && left.platformType == right.platformType && left.platformSubtype == right.platformSubtype;
}

inline bool operator==(const GameInputMouseInfo& left, const GameInputMouseInfo& right) {
    return left.supportedButtons == right.supportedButtons && left.sampleRate == right.sampleRate && left.hasWheelX == right.hasWheelX && left.hasWheelY == right.hasWheelY;
}

inline bool operator==(const GameInputGamepadInfo& left, const GameInputGamepadInfo& right) {
    return left.supportedLayout == right.supportedLayout && left.menuButtonLabel == right.menuButtonLabel && left.viewButtonLabel == right.viewButtonLabel &&
           left.aButtonLabel == right.aButtonLabel && left.bButtonLabel == right.bButtonLabel && left.cButtonLabel == right.cButtonLabel &&
           left.xButtonLabel == right.xButtonLabel && left.yButtonLabel == right.yButtonLabel && left.zButtonLabel == right.zButtonLabel && left.dpadUpLabel == right.dpadUpLabel &&
           left.dpadDownLabel == right.dpadDownLabel && left.dpadLeftLabel == right.dpadLeftLabel && left.dpadRightLabel == right.dpadRightLabel &&
           left.leftShoulderButtonLabel == right.leftShoulderButtonLabel && left.rightShoulderButtonLabel == right.rightShoulderButtonLabel &&
           left.leftThumbstickButtonLabel == right.leftThumbstickButtonLabel && left.rightThumbstickButtonLabel == right.rightThumbstickButtonLabel &&
           left.paddleLeft1Label == right.paddleLeft1Label && left.paddleLeft2Label == right.paddleLeft2Label && left.paddleRight1Label == right.paddleRight1Label &&
           left.paddleRight2Label == right.paddleRight2Label && left.leftTriggerLabel == right.leftTriggerLabel && left.rightTriggerLabel == right.rightTriggerLabel &&
           left.leftThumbstickXLabel == right.leftThumbstickXLabel && left.leftThumbstickYLabel == right.leftThumbstickYLabel &&
           left.rightThumbstickXLabel == right.rightThumbstickXLabel && left.rightThumbstickYLabel == right.rightThumbstickYLabel &&
           left.extraButtonCount == right.extraButtonCount && left.extraAxisCount == right.extraAxisCount;
}

inline bool operator==(const GameInputRumbleParams& left, const GameInputRumbleParams& right) {
    return left.lowFrequency == right.lowFrequency && left.highFrequency == right.highFrequency && left.leftTrigger == right.leftTrigger && left.rightTrigger == right.rightTrigger;
}

} // namespace GameInput::v4

namespace game_input {

// Everything in Microsoft's GameInput.h, by the names it gives them.
using namespace GameInput::v4;

// The GameInput runtime that GameInput.lib's loader loaded, for display.
struct Runtime {
    std::filesystem::path path;
    std::array<std::uint16_t, 4> version;
};

// Starts GameInput the way Microsoft's v4 header does for C++ games: with
// GameInputInitialize from GameInput.lib, and the IID of v4's IGameInput. It
// returns the runtime that the loader loaded, or says what's wrong, and how to
// install GameInput 4.0.306.
//
// The loader takes the runtime with the newest file version of Windows'
// GameInput.dll, the installed GameInputRedist.dll, from System32, and a
// GameInputRedist.dll beside the exe, and the one beside the exe on a tie,
// then the installed one. With 4.0.306 installed and nothing beside the exe,
// it's System32's, which COM also loads to carry calls to GameInput's service,
// so the process runs one copy of the runtime. It loads the runtime with
// LoadLibraryW, so the DLLs that the runtime imports, like HID.DLL, are looked
// for beside the exe first. Test/GameInput pins all three.
//
// It checks that GameInput 4.0.306 is installed first, so that a build or a
// program without it says how to install it, instead of failing to start
// GameInput.
//
// The loader keeps the runtime it loaded in globals that it doesn't guard, so
// one thread at a time calls it. GameInput 4.0.306's runtime crashes when one
// thread releases the last IGameInput while another creates one, so the first
// one that this creates is kept for the life of the process, like the runtime,
// and the last one is never released. Test/GameInput pins it.
std::expected<std::pair<Microsoft::WRL::ComPtr<IGameInput>, Runtime>, std::string> create();

} // namespace game_input
