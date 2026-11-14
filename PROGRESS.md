# Progress: Dynamic Notch

## Current Status
- Milestone A (Remove upstream dependencies): DONE (Commit `97b74c7`)
- Milestone B (Memory metric): DONE (Private WS ~36-40 MB, Commit ~75-79 MB, Total WS ~105 MB)
- Milestone C (Frame-time verification): PARTIAL (baseline only; waiting for user report on 60s `--bench` run while 4K video plays on second monitor)
- Milestone D.1 (Debug overlay): DONE (Commit `32f5e92`)
- Milestone D.2 (Reduce Motion): DONE (Commits `32f5e92` and `42cac51`)
- Item 2 (--bench automated 60s loop): DONE (Commit `0f2219a`)
- Next immediate task: Milestone D.3 (Output device quick switcher via `IPolicyConfig`).

## Checklist
- [x] A. Remove upstream dependencies (Commit `97b74c7`)
- [x] B. Memory metric (private working set <= 60 MB idle verified)
- [ ] C. Frame-time verification (PARTIAL - baseline only; `--bench` implemented, awaiting user 4K video run report)
- [x] D.1. Ctrl+Alt+F debug overlay (graph, missed vblanks, spring state, memory) (Commit `32f5e92`)
- [x] D.2. Reduce Motion (critically damped 120-150ms transition + quick crossfade) (Commit `42cac51`)
- [ ] D.3. Output device quick switcher
- [ ] D.4. Volume HUD polish + Brightness HUD
- [ ] D.5. Split mode
- [ ] E. Finish (installer, README, PERFORMANCE.md, branding, auto-start)
- [ ] D.6. Quick actions
- [ ] D.7. Clipboard history
- [ ] D.8. Calendar
- [ ] D.9. Weather (Open-Meteo)
- [ ] D.10. Themes (album-art-tinted and Light)

## What Changed
- `crates/poni/src/tray.rs`: Added `Run 60s Benchmark (Ctrl+Alt+B)` tray item and `MenuCmd::RunBenchmark`.
- `crates/poni/src/app.rs`:
  - Added `--bench` CLI flag handling and registered global hotkey `Ctrl+Alt+B`.
  - Added `BenchRunner` struct with present-to-present intervals, GPU present timing, missed vblank, and slow-frame recording.
  - Added 60-second automated benchmark workload (cycles of expand, collapse, mid-flight hover in/out, simulated track change).
  - Benchmark outputs statistics to console and writes full `PERFORMANCE.md` on completion before gracefully exiting.
- Fixed commit hashes in `PROGRESS.md` to match `git log`.
- Updated checklist ordering to D.3 -> D.4 -> D.5 -> E -> D.6 -> D.7 -> D.8 -> D.9 -> D.10.

## Measured Numbers
- Display: 240 Hz (4.17 ms budget).
- Baseline frame time: 0.3 - 0.7 ms draw, 0.1 ms present (p95 = 0.8 ms).
- Missed vblanks: 0 (baseline).
- Memory: Private WS = 36-40 MB (budget: <= 60 MB), Commit = 75-79 MB, Total WS = 105 MB.
- Idle CPU: 0.0%.
- Outbound sockets: 0 during 5-minute capture on default launch.
- Test suite: All 12 island tests passing; test suite passes cleanly.

## Known Issues
- Milestone C awaits user execution of `--bench` with 4K video running on secondary monitor.

## Exact Next Step
Implement Milestone D.3: Output device quick switcher. Isolate `IPolicyConfig::SetDefaultEndpoint` behind a trait `AudioOutputSwitcher` with COM implementation for Windows 10/11 and graceful fallback.

## Decisions
- D.1: Avoided borrow conflict on mutable canvas by capturing snapshot struct `DebugSnapshot` before `gfx.begin()`.
- D.2: Configured reduced motion spring params as critically damped (k=20000, c=282.8, zeta=1.0) for geometry settling in ~140-154 ms, and (k=4000, c=126.5) for quick content crossfade settling in ~137 ms.
- Bench: Added `--bench` CLI flag and `Ctrl+Alt+B` hotkey to run a 60-second automated stress loop and write markdown report to `PERFORMANCE.md`.
- D.3: Will isolate `IPolicyConfig` behind a Rust trait `AudioOutputSwitcher` with fallback on failure.

## Questions for User
(None at this time; defaults selected according to AGENTS.md rules.)
