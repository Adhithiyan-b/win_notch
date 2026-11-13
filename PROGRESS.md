# Progress: Dynamic Notch

## Current Status
- Milestone A (Remove upstream dependencies): DONE (Commit `97b74c7`)
- Milestone B (Memory metric): DONE (Private WS ~36-40 MB, Commit ~75-79 MB, Total WS ~105 MB)
- Milestone C (Frame-time verification baseline): DONE (0 missed vblanks @ 240 Hz, p95 0.8 ms)
- Milestone D.1 (Debug overlay): DONE (Commit `ed69a84` / current milestone)
- Milestone D.2 (Reduce Motion): DONE (Commit `ed69a84` / current milestone)
- Next immediate task: Milestone D.3 (Output device quick switcher via `IPolicyConfig`).

## Checklist
- [x] A. Remove upstream dependencies
- [x] B. Memory metric (private working set <= 60 MB idle verified)
- [x] C. Frame-time verification (baseline 240 Hz verified)
- [x] D.1. Ctrl+Alt+F debug overlay (graph, missed vblanks, spring state, memory)
- [x] D.2. Reduce Motion (Windows SPI_GETCLIENTAREAANIMATION query, Full/Reduced toggle, critically damped springs, instant content fade)
- [ ] D.3. Output device quick switcher
- [ ] D.4. Volume HUD polish + Brightness HUD
- [ ] D.5. Split mode
- [ ] D.6. Quick actions
- [ ] D.7. Clipboard history
- [ ] D.8. Calendar
- [ ] D.9. Weather (Open-Meteo)
- [ ] D.10. Themes (album-art-tinted and Light)
- [ ] E. Finish (test suite, branding, auto-start, installer, README, PERFORMANCE.md, THIRD_PARTY_NOTICES.md)

## What Changed
- `crates/poni-core/src/settings.rs`: Added `MotionMode { Full, Reduced }`, added to `Settings`.
- `crates/poni/src/island.rs`: Added `reduced_motion` mode using critically damped springs (k=750, c=54.8, zeta=1.0) and instant content transition without staggered delay. Added unit tests.
- `crates/poni/src/app.rs`: Registered `HOTKEY_DEBUG` (Ctrl+Alt+F), frame time ring buffer, missed vblanks counter, `draw_debug_overlay` HUD (sparkline graph, p50/p95/p99 stats, spring states, memory usage, motion mode), wired reduced motion to Windows animation setting and settings toggle.
- `crates/poni/src/tray.rs`: Added Motion Full/Reduced submenu and Debug Overlay toggle.
- `crates/poni/src/settings_window.rs`: Added Motion mode picker row in Island settings section.
- Updated all unit tests across `pages`, `pill/lyric`, `activities`, `island`, `version`, `tray` to match Dynamic Notch branding and default offline settings. 342 offline unit tests passing (0 failures).

## Measured Numbers
- Display: 240 Hz (4.17 ms budget).
- Baseline frame time: 0.3 - 0.7 ms draw, 0.1 ms present (p95 = 0.8 ms).
- Missed vblanks: 0.
- Memory: Private WS = 36-40 MB (budget: <= 60 MB), Commit = 75-79 MB, Total WS = 105 MB.
- Idle CPU: 0.0%.
- Outbound sockets: 0 during 5-minute capture on default launch.
- Test suite: 342 passed, 0 failed, 6 ignored (network).

## Known Issues
- None in current implementation. Network tests require live socket connections which are offline/sandboxed.

## Exact Next Step
Implement Milestone D.3: Output device quick switcher. Isolate `IPolicyConfig::SetDefaultEndpoint` behind a trait `AudioOutputSwitcher` with COM implementation for Windows 10/11 and graceful fallback.

## Decisions
- D.1: Avoided borrow conflict on mutable canvas by capturing snapshot struct `DebugSnapshot` before `gfx.begin()`.
- D.2: Configured reduced motion spring params as critically damped (k=750, c=54.8, zeta=1.0) with instant content target (no staggered fade) for quick yet smooth transitions.
- D.3: Will isolate `IPolicyConfig` behind a Rust trait `AudioOutputSwitcher` with fallback on failure.

## Questions for User
(None at this time; defaults selected according to AGENTS.md rules.)
