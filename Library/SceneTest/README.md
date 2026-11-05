# SceneTest

What [MinimalSceneTest](../../Program/MinimalSceneTest/README.md) and
[SpinningCarTest](../../Program/SpinningCarTest/README.md) share: a 3D scene that fills the
window at either supported height, drawn with the lowest latency, and a card that floats over
its bottom left corner with the test's name, its frame rate, the GPU, the display, a badge for
whether the window has focus, and the shortcuts on keycaps. `F1` hides the card, and `F11`
switches between maximized and borderless fullscreen.

`scene_test::run::<T>` opens the window, with the test's settings, and draws frames until it
closes, or a script ends, with [Library/UserInterface](../UserInterface/README.md)'s main loop. A
test gives it its scene, as a `Scene`, which has the test's name and settings, makes the scene
with the renderer, says what the scene shows, adds rows and shortcuts to the card, handles
the test's own keys, moves to each frame's time, draws, and gives its part of the state that
scripts save and check. `Test/SceneTest` tests the keys.
