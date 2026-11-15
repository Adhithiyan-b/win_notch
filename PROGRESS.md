# Progress: Dynamic Notch

## HANDOFF
- Current Item: Step A (Remove upstream dependencies)
- Last Completed Step: Background poni-core test terminated; files deleted from git
- WIP Files: crates/poni (app.rs, services, pages, tray, state), Cargo.toml, Cargo.lock
- Compile State: HEAD may not compile: run cargo check first
- Exact Next Step: Run `cargo check --bin dynamic-notch` and resolve remaining references in poni
- Tried / Do NOT Repeat: Do NOT run full poni-core test suite during step A; do NOT wait >5 min on tasks
- Waiting On: Nothing

## Current Status
- Milestone A (Remove upstream dependencies): PARTIAL (Reopened: removing leftover oyenza, usage, studio-usage, update, pairdrop, applemusic modules)
- Milestone B (Memory metric): DONE (Private WS 36–40 MB idle, commit 75–79 MB, no page trimming)
- Milestone C (Frame-time verification & optimization): PARTIAL (Reopened for per-phase stats, verification on 4K multi-mon load, and post-bench idle WS verification)
- Milestone D.1 (Debug overlay): DONE (Commit `32f5e92`, Ctrl+Alt+F overlay)
- Milestone D.2 (Reduce Motion): DONE (Commits `32f5e92` & `42cac51`, critically damped ~140–154ms geometry, ~137ms crossfade)
- Milestone D.3 (Output device switcher): DONE (Commit `f9d0067`, IPolicyConfig via AudioOutputSwitcher trait + fallback, tray quick switcher)
- Next immediate task: Finish Milestone A removal of remaining upstream infrastructure

## Checklist
- [ ] A. Remove upstream dependencies (reopened to complete full removal)
- [x] B. Memory metric (private working set <= 60 MB idle verified)
- [ ] C. Frame-time verification & optimization (reopened for per-phase benchmark)
- [x] D.1. Ctrl+Alt+F debug overlay (graph, missed vblanks, spring state, memory)
- [x] D.2. Reduce Motion (critically damped 120–150ms transition + quick crossfade)
- [x] D.3. Output device quick switcher (IPolicyConfig + IMMDeviceEnumerator)
- [ ] D.4. Volume HUD polish + Brightness HUD
- [ ] D.5. Split mode
- [ ] E. Finish (installer, README, PERFORMANCE.md, branding, auto-start)
- [ ] D.6. Quick actions
- [ ] D.7. Clipboard history
- [ ] D.8. Calendar
- [ ] D.9. Weather (Open-Meteo)
- [ ] D.10. Themes (album-art-tinted and Light)

## What Changed
- Step C Frame-Time Optimization:
  1. GDI Hit Mask Decoupling: In `crates/poni/src/app.rs`, eliminated 30 Hz `UpdateLayeredWindowIndirect` stalls during animation. Hit mask now updates only at transition start and upon settling.
  2. Registry I/O Elimination: In `crates/poni/src/services/recorder.rs`, cached `installed()` status with an `AtomicU8` so `expand()` never invokes synchronous registry queries on the UI thread.
  3. Swapchain Pre-allocation: In `layout_surface()`, pre-allocated window surface width (`expanded_width.max(520.0)`) and extra height for overlays, preventing mid-animation `ResizeBuffers` stalls.
  4. Non-blocking Asynchronous Logging: In `crates/poni/src/plog.rs`, replaced synchronous file opens and per-frame thread spawning with a background worker thread driven by a bounded `mpsc::sync_channel`.
- Milestone D.3: Implemented `AudioOutputSwitcher` trait with `ComPolicyConfig` (calling undocumented `IPolicyConfig::SetDefaultEndpoint` across console, multimedia, communications roles) and `FallbackAudioOutputSwitcher`. Enumerated active audio endpoints via `IMMDeviceEnumerator`, mapped form factors to icons, added "Sound Output" submenu to tray, and added switch toast notifications.
- Item 2: Implemented 60-second automated `--bench` stress test loop (`Ctrl+Alt+B`), recording present-to-present intervals, GPU present times, missed vblanks, and slow frames, saving to `PERFORMANCE.md`.
- Item 1: Configured Reduce Motion to critically damped transitions (~140–154ms) and quick content crossfade (~137ms) with passing tests.

## Measured Numbers
- Display: 240 Hz (4.17 ms budget).
- Baseline frame time: 0.3–0.7 ms draw, 0.1 ms present (p95 = 0.8 ms).
- Missed vblanks: 0 (baseline).
- Memory: Private WS = 36–40 MB (budget: <= 60 MB), Commit = 75–79 MB, Total WS = 105 MB.
- Idle CPU: 0.0%.
- Outbound sockets: 0 during 5-minute capture on default launch.
- Test suite: 351 passed, 0 failed (offline), 6 ignored (network).

## Known Issues
- A and C reopened; Step C fixes in c093e63 are not yet benchmarked.

## Exact Next Step
Finish A, then C, then D.4 (do NOT replace/suppress the Windows OSD).

## Decisions
- D.1: Captured immutable snapshot `DebugSnapshot` before `gfx.begin()` to avoid canvas borrow conflict.
- D.2: Configured reduced motion spring params as critically damped (k=20000, c=282.8, zeta=1.0) for geometry settling in ~140–154ms and (k=4000, c=126.5) for content crossfade in ~137ms.
- D.3: Isolated `IPolicyConfig::SetDefaultEndpoint` behind `AudioOutputSwitcher` trait with `ComPolicyConfig` and `FallbackAudioOutputSwitcher`. Added active audio endpoints with type icons to tray menu with checkmark on default.
- Sequence: Following updated user order (D.3 -> D.4 -> D.5 -> E -> D.6 to D.10).

## Questions for User
(None at this time; defaults selected according to AGENTS.md rules.)
