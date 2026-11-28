# Subsystem Ownership & Responsible AI Disclosure
## CA-6: Operating Systems – Systems Construction

---

## 1. Subsystem Ownership Matrix

In compliance with Section 31 of the CA-6 specification, individual contributions across subsystems, experiments, and deliverables are delineated below:

| Subsystem / Deliverable | Primary Owner | Code Contributed | Experimental Evidence |
|---|---|---|---|
| **Kernel Core & Boot (Layers 1, 2)** | Systems Engineer 1 | `src/kernel/cpu.rs`, `src/kernel/logger.rs`, `src/kernel/boot.rs` | Boot logs, startup state machine verification |
| **Task & Scheduler (Layers 3, 4, 5, 6)** | Systems Engineer 2 | `src/task/tcb.rs`, `src/scheduler/queue.rs`, `src/scheduler/round_robin.rs` | TC1, TC2, TC6 scheduling traces & turnaround analysis |
| **Concurrency Track C (Layer 7 Specialization)** | Systems Engineer 3 | `src/sync/spinlock.rs`, `src/sync/mutex.rs`, `src/sync/deadlock.rs` | TC4 (10-task stress), TC5 (deadlock prevention), contention benchmarks |
| **Memory Management (Layer 8)** | Systems Engineer 1 & 2 | `src/memory/bump.rs`, `src/memory/block.rs`, `src/memory/tracker.rs` | TC3 (boundary allocation), TC5 (OOM and double-free rejection) |
| **Telemetry & Evaluation Layer** | Systems Engineer 3 | `scripts/trace_parser.py`, `scripts/contention_analyzer.py` | CSV telemetry export, ASCII charts, turnaround time metrics |
| **Documentation & Defense** | All Team Members | `docs/architecture.md`, `docs/design.md`, `docs/report.md`, `docs/viva_prep.md` | Full 20-section report, viva defense package |

---

## 2. Responsible AI Usage Disclosure

In compliance with Section 25 of the CA-6 specification:
> "Generative AI may assist with: Rust syntax, explaining compiler errors, test scaffolding, debugging hypotheses, architecture brainstorming, documentation, Python trace analysis. However: Students remain responsible for every line of kernel code submitted."

### Detailed Disclosure Statement:
1. **Scaffolding and Syntax**: Generative AI was consulted to structure initial Rust module layouts and to generate Python telemetry parsing scripts for processing CSV scheduler traces.
2. **Algorithmic Validation**: AI was used to brainstorm edge-case scenarios for TC3 (zero-sized allocations) and TC5 (deadlock rank inversion).
3. **Student Comprehension & Code Integrity**: Every line of kernel source code (`src/`), synchronization primitive, allocator algorithm, and test case was manually reviewed, verified, and audited by team members.
4. **Theoretical Understanding**: All team members thoroughly understand Rust ownership, borrowing rules, atomic memory orderings (`Acquire`, `Release`, `SeqCst`), and Dijkstra's resource hierarchy invariants used throughout the Micro-OS.
