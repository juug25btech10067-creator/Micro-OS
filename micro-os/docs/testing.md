# Micro-OS Verification and Test Strategy
## CA-6: Operating Systems – Systems Construction

---

## 1. Testing Hierarchy

Our verification plan implements the 3-level testing strategy mandated by Section 15 of the CA-6 specification:

```text
Level 3: Full Kernel Integration Tests (Boot -> Dispatch -> Sync -> Mem -> Exit)
   ▲
   │
Level 2: Mechanism Verification (Round Robin Preemption, Mutex Sleep/Wakeup, Deadlock Guard)
   ▲
   │
Level 1: Subsystem Unit Tests (ReadyQueue, CpuContext, BumpAllocator, BlockAllocator)
```

---

## 2. Required Test Cases (TC1 – TC6) Implementation Matrix

| Test Case | Objective | Input / Workload | Expected Observable Behavior | Status |
|---|---|---|---|---|
| **TC1 – Minimum Configuration** | Validate basic 2-task Round Robin scheduling | 2 tasks with 10 CPU compute cycles each; Quantum = 2 | Alternating timeslices, $\ge 2$ context switches, both tasks terminate cleanly | **PASS** |
| **TC2 – Multiple Tasks** | Validate queue management with 4 mixed workloads | 4 tasks with mixed compute, memory allocations, and voluntary yields | Fair timeslice allocation, all 4 tasks reach `Terminated`, queue empties | **PASS** |
| **TC3 – Boundary Case** | Verify edge-case parameters | Quantum = 1 tick; 0-byte memory alloc; 1-byte memory alloc | Quantum=1 forces preemption every tick; 0-byte alloc returns safely; 1-byte aligns to 8B | **PASS** |
| **TC4 – Stress Case** | High concurrency lock contention | 10 tasks contending on a single Sleeping Mutex | Tasks block on contention, FIFO unblocking handoffs, zero data corruption | **PASS** |
| **TC5 – Failure Case** | Controlled rejection of illegal operations | 1) Deadlock rank inversion; 2) Double-free on block pool; 3) OOM on 128B heap | 1) `DeadlockRiskError` caught; 2) Double-free rejected; 3) OOM logged without kernel panic | **PASS** |
| **TC6 – Repeatability** | Verify deterministic scheduler execution | 3 identical runs with identical 3-task mutex workloads | Identical ticks (e.g. 100), identical switch count, identical lock grant order | **PASS** |
| **TC7 – Ethernet Framing** | Validate Layer 2 data link frame packing | 14-byte header, MAC addresses, EtherType 0x0800 | Full serialization and deserialization matching IEEE 802.3 format | **PASS** |
| **TC8 – IPv4 Routing & Checksum** | Verify RFC 791 network layer checksum | IPv4 header with variable payload | 16-bit one's complement sum equals stored checksum (0x0000 on wire) | **PASS** |
| **TC9 – DHCP & Internet Ping** | Dynamic IP assignment & ICMP Echo | Broadcast DHCP discover -> Ack (`10.0.2.15`), Ping `10.0.2.2` & `8.8.8.8` | IP acquired, gateway ARP resolved, ping RTT measured | **PASS** |
| **TC10 – HTTP Web Client** | L7 Application Layer Web request | `GET /status HTTP/1.1` to remote host | Status line parsed (`200 OK`), payload length verified | **PASS** |

---

## 3. How to Run the Tests

### Automated Cargo Integration Suite
To execute all test cases (TC1 through TC6) via Rust's test harness:
```bash
cargo test --test integration_tests
```

### Verbose Test Output with Kernel Logging
To observe internal kernel diagnostics and subsystem traces during test execution:
```bash
cargo test --test integration_tests -- --nocapture
```

### Trace Parsing and Performance Validation
Generate the CSV traces by running the main demonstration, then parse the telemetry:
```bash
cargo run --bin micro_os
python scripts/trace_parser.py traces/demo1_rr_trace.csv
python scripts/contention_analyzer.py
```
