# Micro-OS Design Decisions & Low-Level Systems Engineering
## CA-6: Operating Systems – Systems Construction

---

## 1. Core OS Design Decisions

### 1.1 CPU Scheduling: Round Robin with Time-Slice Preemption
- **Rationale**: Round Robin guarantees bounded waiting time and proportional CPU fairness across equal-priority tasks without starvation.
- **Quantum Sizing Trade-Off**:
  - *Large Quantum*: Minimizes context switch overhead, but degrades interactive responsiveness (approaches FCFS).
  - *Small Quantum (e.g. 1-2 ticks)*: Maximizes fairness and responsiveness, but amplifies context-switching CPU overhead. Micro-OS defaults to 2 ticks, allowing observable preemption while maintaining high execution efficiency.

### 1.2 Concurrency Architecture: Spinlock vs. Sleeping Mutex
- **Spinlock Evaluation**:
  - Ideal for short, low-latency critical sections where thread-sleeping overhead exceeds spin latency.
  - *Deficiency*: On a single CPU core, a spinning thread burns its entire time-slice quantum in a loop without allowing the lock holder to progress, causing severe priority inversion and CPU waste.
- **Sleeping Mutex Evaluation**:
  - When contended, the requesting task is moved to `TaskState::Blocked` and enqueued onto the mutex's private wait queue.
  - Voluntarily yields the CPU via `context_switch()`, allowing the lock holder (or other runnable tasks) to execute immediately.
  - Upon release, ownership is transferred directly to the head of the wait queue, moving it to `TaskState::Ready`.

### 1.3 Deadlock Prevention: Hierarchical Resource Ordering (Lock Ranking)
- **Theoretical Basis**: Dijkstra's resource ordering condition breaks the *Circular Wait* requirement (one of Coffman's four deadlock conditions).
- **Implementation**:
  - Every lock instance is tagged with an immutable rank: $R(L) \in \mathbb{N}$.
  - A task holding locks with maximum rank $R_{\max}$ is strictly forbidden from requesting any lock $L'$ where $R(L') \le R_{\max}$.
  - *Complexity*: $O(1)$ time and $O(1)$ space check on lock acquisition, completely avoiding heavy dynamic cycle detection algorithms (e.g. Tarjan's SCC on resource allocation graphs).

### 1.4 Memory Management: Dual Allocation Strategy
- **Bump Allocator**:
  - Extremely fast pointer bump ($O(1)$).
  - Used for boot-time static structures and scratch allocations.
  - Limitations: Cannot reclaim individual allocations; only bulk reset.
- **Fixed-Size Block Allocator with Free-List**:
  - Partitions memory into uniform 64-byte blocks.
  - Deallocated blocks are pushed onto a free list ($O(1)$ alloc and free).
  - Guarantees zero external fragmentation and protects against heap exhaustion.

---

## 2. Safe vs. `unsafe` Rust Engineering Policy

In accordance with Section 26 of the CA-6 specification:
> "Every significant `unsafe` block should have: 1. a clear reason; 2. documented assumptions; 3. stated invariants; 4. limited scope; 5. validation/testing evidence."

### Strict Rust Safety Auditing

| Subsystem | Safety Level | Justification / Invariants |
|---|---|---|
| **Kernel Core & Scheduler** | Safe Rust (`#![deny(unsafe_code)]` compatible) | State encapsulation through standard Rust ownership and interior mutability (`AtomicUsize`, `AtomicBool`, `Mutex`). |
| **Virtual Stack Inspection** | Safe Boundary Checks | Explicit bounds checking (`sp >= base && sp <= base + size`) prevents stack corruption without raw pointer dereferencing. |
| **Lock Operations** | Atomic Primitives | Utilizes atomic memory orderings (`Acquire` on lock entry, `Release` on unlock, `SeqCst` on ticket distribution) to prevent instruction reordering. |
| **Memory Buffer Slicing** | Safe Bounds Validated | Block offsets strictly calculated through modulo arithmetic and checked vector indexing. |

### Documented Invariant Rules:
1. **Mutex Ownership Invariant**: A mutex can only ever be released by the task currently designated as its `owner`. Any release attempt by a non-owner results in an explicit error return and kernel log warning.
2. **Lock Order Invariant**: $R(L_{\text{requested}}) > \max_{L \in \text{Held}(T)} R(L)$.
3. **Queue Invariant**: A task can reside in at most one queue at any given instant (`ReadyQueue`, `MutexWaitQueue`, or running on CPU).
