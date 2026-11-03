# SceneTest

What [MinimalSceneTest](../../Program/MinimalSceneTest/README.md) and
[SpinningCarTest](../../Program/SpinningCarTest/README.md) share: a 3D scene that fills the
window at either supported height, drawn with the lowest latency, and a text box over its
bottom left corner with the test's name, its frame rate, the GPU, the display, and whether the
window has focus. `F1` hides the text box, and `F11` switches between maximized and borderless
fullscreen.

`scene_test::run` opens the window, with the test's settings, and draws frames until it closes,
or a script ends. A test gives it its scene, as a `Scene`, which says what the scene shows, adds
lines and shortcuts to the text box, handles the test's own keys, moves to each frame's time,
draws, and gives its part of the state that scripts save and check. `Test/SceneTest` tests the
keys.
