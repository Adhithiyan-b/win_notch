# Progress: Dynamic Notch

## Current Status
- Milestone A (Remove upstream dependencies): DONE (Commit `97b74c7`)
- Milestone B (Memory metric): DONE (Private WS ~36-40 MB, Commit ~75-79 MB, Total WS ~105 MB)
- Milestone C (Frame-time verification baseline): DONE (0 missed vblanks @ 240 Hz, p95 0.8 ms)
- Milestone D.1 (Debug overlay): DONE in code (Ctrl+Alt+F hotkey, frame stats graph, spring inspection, memory stats)
- Milestone D.2 (Reduce Motion): DONE in code (SPI_GETCLIENTAREAANIMATION query, full/reduced settings & tray toggle, critically damped transitions)
- Next immediate task: Fix remaining unit tests in `pages/mod.rs` & `pill/lyric.rs` following upstream tab removal, then commit D.1 & D.2. Then proceed to D.3 (Output device quick switcher).

## Checklist
- [x] A. Remove upstream dependencies
- [x] B. Memory metric (private working set <= 60 MB idle verified)
- [x] C. Frame-time verification (baseline 240 Hz verified)
- [x] D.1. Ctrl+Alt+F debug overlay (implemented, testing/clean up)
- [x] D.2. Reduce Motion (implemented, testing/clean up)
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
- Created `AGENTS.md` with session protocol, rules, and checklist verbatim.
- Created `PROGRESS.md` tracking detailed progress.
- Implemented `HOTKEY_DEBUG` (Ctrl+Alt+F) in `app.rs` with `draw_debug_overlay` (sparkline, p50/p95/p99, missed vblanks, spring state, memory metrics).
- Implemented `MotionMode` in `settings.rs`, `island.rs`, `app.rs`, and tray menu.
- Fixed branding in `version.rs` and `tray.rs`.

## Measured Numbers
- Display: 240 Hz (4.17 ms budget).
- Baseline frame time: 0.3 - 0.7 ms draw, 0.1 ms present (p95 = 0.8 ms).
- Missed vblanks: 0.
- Memory: Private WS = 36-40 MB (budget: <= 60 MB), Commit = 75-79 MB, Total WS = 105 MB.
- Idle CPU: 0.0%.
- Outbound sockets: 0 during 5-minute capture on default launch.

## Known Issues
- Unit tests: Some tests in `pages/mod.rs` expect tabs that were removed in milestone A (`Oyenza`, `Ai`, `Health`, `Ielts`). Needs updating.
- Network tests in `http_cache` fail in offline sandbox environment.

## Exact Next Step
Fix remaining test failures in `crates/poni/src/pages/mod.rs` and commit D.1/D.2. Then start D.3 (Output device quick switcher via `IPolicyConfig`).

## Decisions
- D.1: Avoided borrow conflict on mutable canvas by capturing snapshot struct `DebugSnapshot` before `gfx.begin()`.
- D.2: Configured reduced motion spring params as critically damped (k=750, c=54.8, zeta=1.0) with instant content target (no staggered fade) for quick yet smooth transitions.
- D.3: Will isolate `IPolicyConfig` behind a Rust trait `AudioOutputSwitcher` with fallback on failure.

## Questions for User
(None at this time; defaults selected according to AGENTS.md rules.)
