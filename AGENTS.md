# Dynamic Notch (Rust, Windows overlay; fork of oyenza-studio, MIT). Finish checklist only. No extras, no customization.

## Protocol
- Start: read this + PROGRESS.md + `git status -s`. Resume from HANDOFF. Never ask for info in these files.
- Replies max 3 lines. No plans, narration, summaries. Don't ask questions except admin rights / API key / licence change; else pick the safe default, log under Decisions.
- Build via `scripts\chk.ps1` (iterate) and `scripts\rel.ps1` (end of item/measurements only). Never run raw cargo output. Tests/bench: `| Select-Object -Last 25`.
- Files: `Select-String -n` to locate, then view <=60 lines. Never print whole files. Targeted edits only. Don't touch /target or the archive.
- Prefer deleting off-list code over repairing it.
- After each item: PROGRESS.md (status + evidence line + next step) and `git commit`. DONE needs an evidence line, else PARTIAL.
- HANDOFF block (<=10 lines) at top of PROGRESS.md: item, last step, WIP files, compile state, exact next step, do-not-repeat. Update + commit before any long command. On "switching" or quota warning: update HANDOFF, commit, reply "HANDOFF READY".
- Timebox: any command >5 min gets killed. Max 2 fix attempts per problem, then log it in Known Issues and move on.

## Rules
- No telemetry. No network unless a feature needs it (off by default). No oyenza servers. Add-ons load from local folder only.
- `cargo build --release` passes at every commit (toolchain stable-x86_64-pc-windows-gnullvm).
- Keep LICENSE, copyright headers, THIRD_PARTY_NOTICES.md.
- Springs stay interruptible. Measure on the 240 Hz display only after D.5 and at the end: missed vblanks, present intervals. No EmptyWorkingSet. Private WS <= 60 MB idle.
- Fullscreen: hide by default; "Show over games" setting = compact strip, zero CPU when no game.
- Unreliable feature: degrade gracefully, log it.

## Map (crates/poni/src)
app.rs loop/hotkeys/overlay/bench | island.rs morph+reduced motion | poni-core/src/spring.rs | pill/* drawing | activities.rs priority | services/ audio, bluetooth, media, power, notifications, clipboard, fullscreen, sysmon, recorder | tray.rs, settings_window.rs, poni-core/src/settings.rs | commands/events/state.rs | gfx/glass/window.rs | poni-core: agenda.rs pomodoro.rs shelf.rs frametime.rs

## Checklist (strict order; E last, then write PROJECT COMPLETE)
A. Done = no code path to oyenza/z.ai/Apple Music; no user-visible Oyenza/Poni text; phonedrop socket only while its panel is open; one 5-min capture = 0 connections. Don't rename crates, identifiers or comments. Remove off-list leftovers (oyenza_show_amounts, usage/AI tab, prayer/sholat, caffeine, ja add-on) instead of fixing them.
C. One bench pass; private WS measured at true idle (not during bench; PERFORMANCE.md says 117.5 MB but PROGRESS.md says 36-40 MB, find out why); max 2 fixes.
D.4 Brightness HUD (WMI laptop / DDC-CI external, degrade gracefully) + volume polish. No OSD suppression.
D.5 Split mode: detached mini-pill, spring separate/merge.
D.6 Quick actions: screenshot, Focus mode, favorite app launcher.
D.7 Clipboard history: memory only by default; optional DPAPI persistence; honor password-manager opt-out formats; per-app exclude; pin; auto-clear; clear-all.
D.8 Calendar: local .ics + ICS URL, countdown pill <15 min, no OAuth.
D.9 Weather: Open-Meteo, manual city geocoding, no IP geolocation, off by default, animated icon + compact temp.
D.10 Themes: album-tinted + Light, live preview in settings.
E. Test suite, branding "Dynamic Notch", auto-start check, Inno Setup installer, README, PERFORMANCE.md, THIRD_PARTY_NOTICES check.
Optional after E: NotchPixel-style extras (reference only, not a clone).

## Done: B, D.1, D.2, D.3
