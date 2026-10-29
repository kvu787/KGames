# Non-disruptive workflow

While working:

- The agent must not launch the app on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.
