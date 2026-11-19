# Dynamic Notch: agent instructions (read at the start of EVERY session)

Project: Windows Dynamic Island-style notch overlay in Rust, forked from oyenza-studio (MIT) at D:\projects\win notch\dynamic-notch. Goal: finish the checklist below, verified and polished. Customization comes AFTER this and is out of scope now. Do not add features that aren't on the list.

## Session protocol
- Start of session: read AGENTS.md and PROGRESS.md, run `git log -5`, continue with the first unchecked item. Never ask me to repeat anything that is in these files.
- After EVERY completed item, or every ~20 minutes of work: update PROGRESS.md (status, what changed, measured numbers, known issues, exact next step) and `git commit`. Never leave a half-finished edit without a note in PROGRESS.md.
- Quota economy (I have small limits and switch accounts): iterate with `cargo check`; run full `cargo build --release` only for measurements and at the end of an item. Don't re-print long files or repeat earlier reports. Chat replies: max ~5 lines.
- TOKEN RULES: No plans, narration, summaries or confirmations. Tool calls, then max 5 lines of reply. Never print compiler/test/bench output raw. Always filter: `cargo check --bin dynamic-notch 2>&1 | grep -E "^(error|warning: unused)" -A4 | head -50`. Test/bench output: `| tail -n 30`. Edit with targeted str_replace; never re-write or re-print whole files; read only lines needed (grep -n, view small range). Prefer deleting off-list code over repairing it. Don't add anything not on checklist. Never search /research or /target. Keep HANDOFF block current; commit before long commands; WIP commits allowed. On "switching": update HANDOFF, commit, reply "HANDOFF READY". Don't ask questions unless admin rights, API key, or licence change needed.
- If I write "switching" or "handoff": bring the current file to a compiling state, update PROGRESS.md, commit, reply "HANDOFF READY".
- Don't stop for approval between items; go to the next one. Ask me ONLY for admin rights, API keys, or licensing changes. For any other ambiguity choose the safest default, record it in PROGRESS.md under "Decisions", and continue. Batch questions at the end of PROGRESS.md.
- An item may be marked DONE only with an evidence line in PROGRESS.md (command output, test count, or numbers from PERFORMANCE.md). No evidence means PARTIAL. Never re-analyze the whole codebase; use the file map below.
- HANDOFF PROTOCOL: Keep a "## HANDOFF" block at the top of PROGRESS.md (max 10 lines, always current: current item, last completed step, uncommitted/WIP files, compile state, exact next command/edit, things tried not to repeat, anything waiting on user). Update HANDOFF block and commit BEFORE starting any long command (cargo build --release, cargo test, --bench, network capture) and after every finished step. WIP commits allowed if noted. If user writes "switching" or on quota warning: stop immediately, use at most 2-3 tool calls to update HANDOFF block and commit, reply "HANDOFF READY". At session start read only AGENTS.md, PROGRESS.md and git status; resume from HANDOFF block without re-analyzing. Never wait on any background command for more than 5 minutes (terminate on expiry).

## File Map
(paths under crates/poni/src unless noted):
- app.rs: main loop, window messages, hotkeys, debug overlay, --bench / BenchRunner
- island.rs: island size/morph springs, reduced motion. poni-core/src/spring.rs: spring solver
- pill/: drawing of pill states (expanded, hud, idle, live, media, music, notice, shape, status, motion)
- activities.rs: live activities and priority
- services/: audio.rs (volume, output devices), bluetooth.rs, media.rs (GSMTC), power.rs, notifications.rs, clipboard.rs, fullscreen.rs, sysmon.rs, mic.rs, recorder.rs, location.rs, cities.rs
- tray.rs, settings_window.rs, settings_host.rs; poni-core/src/settings.rs (settings model)
- commands.rs, events.rs, state.rs (command/event/state plumbing)
- gfx.rs, glass.rs, window.rs (Direct2D/DirectComposition, layered window)
- poni-core/src: agenda.rs (calendar), pomodoro.rs, shelf.rs, frametime.rs

## Rules
- No telemetry. No network without a feature that needs it, off by default. No calls to oyenza servers.
- `cargo build --release` must pass at every commit (toolchain stable-x86_64-pc-windows-gnullvm + LLVM-MinGW).
- Keep LICENSE, copyright headers and THIRD_PARTY_NOTICES.md. Only load add-ons from a local folder I choose; never download code.
- If a feature can't work reliably on this machine, degrade gracefully and note it in PROGRESS.md.
- Auto-hide & Fullscreen: in fullscreen, hide the notch by default, but add a "Show over games" setting that shows only a compact strip (FPS + mic-muted indicator) while a game is in front, and costs zero CPU when the game isn't running.
- Animation: springs stay interruptible (velocity-preserving retarget). Before/after any animation change, measure real presentation on the 240 Hz display (DXGI frame stats / DwmGetCompositionTimingInfo: missed vblanks, present-to-present intervals, GPU time) during expand, collapse, rapid hover in/out, track change, and 4K video on another monitor. Target: no missed vblanks.
- Memory: private working set ≤ 60 MB idle. Never use EmptyWorkingSet or page trimming. Cap thumbnail cache; pre-warm album-art textures when media starts.

## Order and Execution Rules
- Execution order is strict: A, C, D.4, D.5, D.6, D.7, D.8, D.9, D.10, then E LAST. Write "PROJECT COMPLETE" only after E.
- Remove off-list leftovers instead of fixing them (settings and code): oyenza_show_amounts, usage/AI tab, prayer/sholat, caffeine, ja add-on, and anything else not on the checklist that blocks the build or touches the network.
- D.11 NotchPixel parity: OPTIONAL, do last, only after E. We are NOT cloning NotchPixel; it was only a reference. Keep the "Show over games" setting; skip anything else from NotchPixel.

## Checklist (in order)
A. Remove upstream dependencies: oyenza.rs, usage.rs (AI/LLM tab), update.rs/updates.rs, pairdrop.rs signaling, Apple Music token flow, related UI/settings; remove Oyenza/Poni branding (strings, resources, tray tooltip); disable remote add-on downloads; phonedrop socket open only while its panel is open. Then capture 5 minutes of real network traffic on a fresh default launch: expect zero connections. Re-check after any milestone that adds networking.
B. Memory metric: report private working set + commit (total working set for reference). Apply only smoothness-safe cache limits.
C. Frame-time verification as described in Rules; fix any problem before new features.
D. Features:
  1. Ctrl+Alt+F debug overlay (frame-time graph, missed vblanks, spring state), hidden by default
  2. Reduce Motion: honor the Windows setting + in-app Full/Reduced toggle; Reduced = short critically damped transitions/crossfades, not instant snaps
  3. Output device quick switcher (IPolicyConfig isolated behind a trait, graceful failure)
  4. Volume HUD polish + Brightness HUD (laptop: WMI; external monitor: DDC/CI via Dxva2; degrade gracefully; skip OSD suppression if no clean method)
  5. Split mode: detached secondary mini-pill that separates/merges with the same spring/morph quality
  6. Quick actions: screenshot, Focus mode, configurable favorite app launcher
  7. Clipboard history: in-memory by default; optional DPAPI-encrypted persistence; honor password-manager opt-out formats; per-app exclusion; pinning; auto-clear timer; clear-all
  8. Calendar: local .ics + ICS subscription URL, no OAuth/API keys; countdown pill under 15 min; remove oyenza.app dependency
  9. Weather (Open-Meteo): manual city via Open-Meteo geocoding, no IP geolocation, off by default, animated icon + compact temperature
  10. Themes: album-art-tinted and Light, with live preview in the settings window
E. Finish:
  - Run the test suite; fix failures
  - Rename remaining branding to "Dynamic Notch"
  - Auto-start option verified
  - Installer (Inno Setup or similar, unsigned is fine)
  - README (features, settings, hotkeys, build steps)
  - PERFORMANCE.md (frame stats, private memory, idle CPU)
  - Final THIRD_PARTY_NOTICES.md check
  Then write "PROJECT COMPLETE" in PROGRESS.md.
D.11. [OPTIONAL, after E] NotchPixel parity: check base FPS strip, global mic-mute hotkey with visible muted state, file shelf drag in/out. Fix only what is missing or broken. Don't add anything else.
