This repo will contain some of the games I make.

I plan to develop the latest versions of Veehiicuul and Automatou in here.

# TODO

- Implement in-app and logged measurement for these
  - Input latency
  - Frame time
  - Jitters
- Implement in-app frame rate limiter
- Implement vsync option
- Implement a scene that displays nothing or almost nothing to measure base fps
- Figure out how to design or constrain UI
- Specify a policy for handling user input when the window isn't focused
  - How does this affect non-disruptive agent dev?

# Motivation

This repo aims to minimize dependencies on things like game engines and UI frameworks like Unity,
Godot, WPF, Dear ImGui, Electron, etc.

This decision was primarily motivated by my experience working with those dependencies, but here are a summary of the reasons:

- Agents are now good enough to handle the significantly increased development burden of using low-level APIs.
- Maintain precise control over window behavior, input latency, vsync behavior.
- Avoid memory performance issues and frametime spikes caused by garbage collection.
- Avoid annoying bugs and quirks caused by dependencies.
- Avoid opinionated workflows and fundamental design decisions that I don't like that are imposed by dependencies.
- Avoid maintenance burden, bugs, churn, etc introduced by new versions of dependencies that don't benefit my specific usage.
- Make it easier to configure a non-disruptive agent workflow.

# Design

This is a work-in-progress and shouldn't be applied to the repo yet.

- Folders
  - Program: Contains things that are runnable.
  - Library: Contains things that aren't runnable and are shared.
  - Test: Contains all tests.
- GUI apps
  - If the window isn't focused, the app must ignore all input.
  - Use Win32, Rust, Direct3D 12, DirectWrite
  - Use a light theme by default
