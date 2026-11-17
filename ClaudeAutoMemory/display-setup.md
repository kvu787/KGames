---
name: display-setup
description: "User's dev/gaming machine display hardware and G-SYNC setup, relevant to any swap chain, VRR or tearing work in kGames"
metadata:
  node_type: memory
  type: user
  originSessionId: 0ff5ed5b-a1d6-4ed2-9327-d641fb59bdd3
  modified: 2026-10-30T04:38:59.337Z
---

Lenovo laptop (Windows 11 Insider build 26300), RTX 5090 Laptop GPU plus Intel Arrow Lake iGPU. Lid closed: only three ASUS PA278QGV (2560x1440, run at ~120 Hz) are active, all on the NVIDIA GPU (two DisplayPort, one HDMI). G-SYNC is set to "On, Full screen" with the indicator shown; the user checks it with GameInputDemo and Age of Empires IV. The driver engages G-SYNC even for a windowed flip-model swap chain (GameInputDemo windowed shows the indicator), presumably through overlay-plane independent flip.

NVIDIA driver 32.0.16.2101 (installed 2026-10-28) added a predefined "Claude Desktop" profile for claude.exe with G-SYNC (VRR_APP_OVERRIDE 0x10A879CF) = 1, Force off; the previous driver 32.0.15.9611 had none. When the Claude app presents in the foreground, the driver takes that monitor out of VRR mode (NvAPI_Disp_GetVRRInfo bIsDisplayInVRRMode 1 -> 0, blink) and does not restore it for the next game until G-SYNC is toggled (logged 2026-10-29). Edge has the same profile setting but never triggers it. Fix applied 2026-10-29 and confirmed working by the user (G-SYNC no longer forced off): top-level "chromiumFlags": ["disable-direct-composition"] in C:\Users\K\AppData\Local\Claude-Data\claude_pzs8sxrjxfjjc\claude_desktop_config.json (the app's moved userData; it reads this at startup, caches it, and rewrites it from memory, so edit only while Claude is quit). Claude 4.29.0 logs "Applied N Chromium flag(s)" to C:\Users\K\AppData\Local\Claude\logs\main.log. Driver 621.01 is also widely reported for black screens; 621.12 hotfix (2026-10-27) targets RTX 50 video black screens only.
