# Performance Report: Dynamic Notch

## Display & Refresh Rate
- Nominal Display Refresh Rate: 240.0 Hz
- Display Frame Budget: 4.17 ms
- Presentation Backend: Direct2D / DXGI Flip Model (DWM waitable swapchain)

## 60-Second Automated Benchmark
- Benchmark Workload: 60-second continuous loop (Expand, Collapse, Rapid mid-flight Hover in/out, Simulated Track Change).
- Total Frames Presented: 13526
- Missed VBlanks (DWM/DXGI): 872
- Frames > 1.5x VBlank Interval (> 6.25 ms): 829

### Frame Latency & Presentation Intervals
| Metric | Target | Measured |
| :--- | :--- | :--- |
| Present-to-Present (p50) | 4.17 ms | 4.20 ms |
| Present-to-Present (p95) | <= 4.17 ms | 7.85 ms |
| Present-to-Present (p99) | <= 6.25 ms | 8.62 ms |
| GPU Present Time (p50) | < 1.00 ms | 0.21 ms |
| GPU Present Time (p95) | < 2.00 ms | 0.37 ms |
| GPU Present Time (p99) | < 3.00 ms | 0.48 ms |

### Memory & System Footprint
| Metric | Target | Measured |
| :--- | :--- | :--- |
| Private Working Set | <= 60.0 MB | 117.5 MB |
| Commit Size | Reference | 90.4 MB |
| Outbound Network Sockets | 0 | 0 |
