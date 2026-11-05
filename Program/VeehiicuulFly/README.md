# Description

Veehiicuul Fly is first/third-person vehicle flying game that I am currently prototyping.

I was inspired to investigate this genre after playing and enjoying Ace Combat 8. Interestingly,
I played Ace Combat 7 several years ago, and I played AC7 again after playing AC8, and I didn't
get the same inspiration from AC7. I think there is something about physics or control feel of
AC7 that isn't as enjoyable, but I'm not sure. Investigating and getting this "feel" right is
crucial to making my own enjoyable flight game.

I have also played Everspace 1 and 2, and I don't want that control feel. One must-have from AC8
is using a yaw/pitch/roll control model with a weak yaw - basically jet fighter control physics.
If I remember correctly, Everspace has stronger yaw. Having strong yaw and pitch leads to a
"point to where you want to fly"-style control approach, which I find less interesting and
enjoyable.

I did a direct comparison between AC7 and AC8 using the base (no part upgrades) using the F/A-18F
and F-22. I definitely liked AC8 and didn't like AC7. Also, I used these settings:

- G-SYNC = on
- NVCP frame rate limit = 100
- Low latency mode = Ultra

From what I read online, AC7 uses lighter feeling physics while AC8 uses heavier, which corresponds
to what I felt.

There are many flying games or games that have flight in them that could be worth checking out.

# Design

I call this a 4 degrees of freedom (4DOF) flying game because it has yaw, pitch, roll, and
forward/backward movement.

For the camera, I considered using a model in which you are flying a volume-less point around
instead of a typical 3d object to avoid issues with the object blocking your view in third-person
or not being able to see the extents of your vehicle in first-person. However, I think using a
point feels too abstract.

To work around the camera issues with a vehicle model, I will experiment with using "first class"
camera controls. This means instead of just a single button to cycle between different camera
perspectives like in Ace Combat, I will try these things:

- Player-configurable arbitrary camera positions you can place around the vehicle.
- "Security camera" style views that you can persist on the HUD and assign to different cameras.
- Hold down the d-pad or face buttons to instantly change the main view to different cameras.

Ace Combat also provides a free-look camera via the right stick, but I don't think this is enough.
For example, I'm not sure how this would work if you're piloting a very large ship with strange
dimensions or "interior" sections. Also, I don't want an analog stick to be reserved for free-look.
Also, I find the free-look to be too slow, which is why I will experiment with instant camera looks
and persistent camera views.

The camera projection will be perspective instead of orthographic, so this will require changes
in the existing Veehiicuul code.

The only analog controls will be analog left and right stick for these reasons:

- Nintendo Switch controllers don't have analog triggers.
- This is more ergonomic for mobile controls.
- More engineering goes into improving and perfecting the analog sticks of gamepads.

Yaw and roll map naturally to the left/right stick axes and pitch and forward/backward map
naturally to up/down stick axes.

Like Veehiicuul, Veehiicuul Fly will be flying-only and will not have player-initiated combat.
This means that the player won't be shooting or ramming anything, but things may be shot at the
player that must be evaded. The reason I don't want player combat in Veehiicuul is that it
distracts the player from optimal driving/flying, which I personally don't like. When a game has
movement and combat, the player has to track, analyze, and predict many more things at once than
when a game only has movement.

## Terrain

- Since there isn't really an "up" direction in this world, there isn't a terrain ground plane
- All surfaces should be solids
- No zero-thickness surfaces to avoid issues
- Kind of like a collection of asteroids

## Shaders

- Need to change Simple Paint so that it covers the whole sphere and not do any warping
