# CA-6 Micro-OS: Comprehensive Viva Preparation Guide
## Defense Answers for All 20 Official Evaluation Questions (Section 32)

---

### Q1: What happens immediately after the kernel starts?
**Answer:**  
Immediately after boot entry (`BootManager::boot()`), the kernel progresses through a deterministic state machine:
1. Initializes the structured console/diagnostics logger.
2. Initializes the physical memory space (instantiating the 64 KB heap, dividing it between the Bump Allocator and Fixed-Size Block Allocator).
3. Configures the virtual timer frequency (100 Hz) and resets the system tick counter.
4. Initializes the Ready Queue and Task Manager data structures.
5. Sets the scheduler time quantum (2 ticks) and transitions the kernel state to `KernelOperational`.

---

### Q2: How is a task represented?
**Answer:**  
A task is encapsulated within a `TaskControlBlock` (TCB) in `src/task/tcb.rs`. It contains:
- `id: TaskId`: A unique monotonic integer identifier.
- `state: TaskState`: Enum tracking lifecycle (`Ready`, `Running`, `Blocked { reason }`, `Terminated`).
- `priority: u8`: Numerical scheduling priority.
- `context: CpuContext`: Virtual registers ($r_0..r_7$), Program Counter ($PC$), Stack Pointer ($SP$), flags, and cycle counter.
- `stack: VirtualStack`: Virtual memory buffer with base address, size, and boundary limits.
- `metrics: TaskMetrics`: Telemetry tracking arrival tick, start tick, finish tick, CPU cycles, context switch count, timeslices exhausted, and lock wait latency.

---

### Q3: How does the scheduler select the next task?
**Answer:**  
The scheduler operates a double-ended FIFO `ReadyQueue` (`src/scheduler/queue.rs`). During a scheduling event, the scheduler invokes `ready_queue.dequeue()`, which pops the task at the head of the queue in $O(1)$ time. If the previous task was still runnable, it was enqueued to the tail of the queue, ensuring fair cyclic rotation.

---

### Q4: Why did you choose Round Robin?
**Answer:**  
Round Robin is the fundamental baseline time-sharing algorithm. It provides:
1. **Starvation Freedom**: Every runnable task is guaranteed CPU access within $(N-1) \times Q$ ticks.
2. **Predictable Latency**: Bounded waiting time for interactive tasks.
3. **Observability**: Clearly demonstrates the difference between cooperative switching and involuntary timer preemption.

---

### Q5: What triggers a scheduling decision?
**Answer:**  
A scheduling decision is triggered by four distinct events in Micro-OS:
1. **Involuntary Timer Preemption**: The running task's time-slice quantum (2 ticks) expires.
2. **Voluntary Yield**: The running task issues `TaskInstruction::Yield`.
3. **Resource Blocking**: The running task requests a locked `SleepMutex`, transitioning to `Blocked` and waiting on the mutex's private queue.
4. **Task Termination**: The running task executes `TaskInstruction::Exit` or completes its instruction stream.

---

### Q6: What happens during a timer tick?
**Answer:**  
During `KernelEngine::step()`:
1. The global tick counter is incremented and propagated to the logger.
2. If no task is currently dispatched, the scheduler dequeues and dispatches the head ready task.
3. The running task executes its next instruction (consuming cycles or issuing synchronization/memory requests).
4. The running task's quantum slice counter increments.
5. If quantum threshold is reached, an involuntary context switch is invoked: current registers are preserved, state moves to `Ready`, task is pushed to `ReadyQueue` tail, and the next task is restored.

---

### Q7: How is shared kernel data protected?
**Answer:**  
Kernel structures are protected through two layers:
1. **Compile-Time Thread Safety**: Rust's ownership model and type system ensure that concurrent access is either immutable or safely synchronized via atomic primitives (`AtomicBool`, `AtomicUsize`).
2. **Kernel Synchronization Primitives**: Shared resources across simulated tasks are protected via `Spinlock`, `TicketSpinlock`, or `SleepMutex`.

---

### Q8: Why is synchronization required?
**Answer:**  
Without synchronization, concurrent tasks interleaving across timer preemptions suffer from **Race Conditions**—uncontrolled concurrent access where final results depend on arbitrary scheduling order. Synchronization enforces **Mutual Exclusion**, ensuring that only one task can execute inside a critical section at any given time.

---

### Q9: How does your allocator work?
**Answer:**  
Micro-OS implements a dual-allocator architecture:
1. **Bump Allocator (`src/memory/bump.rs`)**: Linear pointer watermark allocator that increments an offset by the requested size with 8-byte alignment. Runs in $O(1)$ time.
2. **Fixed-Size Block Allocator (`src/memory/block.rs`)**: Partitions memory into uniform 64-byte blocks managed by an internal free-list. Allocation pops a block index from the free list; deallocation pushes the block back onto the list, eliminating external fragmentation.

---

### Q10: What happens when allocation fails?
**Answer:**  
When an allocation exceeds available heap memory, the allocator:
1. Increments its `out_of_memory_errors` metric.
2. Emits a structured kernel diagnostic warning via `kwarn!("MEM", ...)`.
3. Returns an explicit `Err("Out of Memory")` Result to the caller.
The kernel does **not** panic or crash; the requesting task handles the failure gracefully.

---

### Q11: Where is `unsafe` used and why?
**Answer:**  
Our Micro-OS is engineered using **Safe Rust abstractions** (`#![deny(unsafe_code)]` compliance). Atomic memory primitives (`AtomicBool`, `AtomicUsize`) provided by standard Rust encapsulate hardware CAS instructions safely. All virtual memory bounds, stack boundaries, and block indices are validated through checked mathematical operations and slice indexing, eliminating undefined behavior.

---

### Q12: What invariant does your `unsafe` code rely on?
**Answer:**  
Although our code is safe Rust, our low-level abstractions rely on four core invariants:
1. **Stack Invariant**: $SP \in [\text{StackBase}, \text{StackBase} + \text{StackSize}]$.
2. **Mutex Invariant**: A mutex can only be unlocked by the task whose ID matches `owner`.
3. **Queue Invariant**: A task can never reside in both `ReadyQueue` and a `MutexWaitQueue` simultaneously.
4. **Hierarchical Lock Invariant**: $R(L_{\text{requested}}) > \max_{L \in \text{Held}(T)} R(L)$.

---

### Q13: What happens if two tasks access the same resource?
**Answer:**  
- **Without Locks**: Tasks interleave arbitrarily during preemption, corrupting shared data.
- **With Spinlock**: The second task burns CPU cycles spinning in a test-and-set loop until the first task releases the lock.
- **With Sleeping Mutex**: The second task is descheduled immediately, moved to `TaskState::Blocked`, and added to the mutex wait queue. When the first task calls `unlock()`, the second task is unblocked and enqueued back to `ReadyQueue`.

---

### Q14: How did you test the scheduler?
**Answer:**  
We established a 3-level test harness:
1. **Unit Tests**: Dequeue/enqueue order in `ReadyQueue`.
2. **Mechanism Tests**: Verified that tasks preempt exactly after 2 ticks.
3. **Standard Test Cases**:
   - TC1 (2 tasks minimum)
   - TC2 (4 tasks multi-workload)
   - TC3 (1-tick quantum boundary)
   - TC6 (multi-run repeatability)

---

### Q15: How did you establish that the scheduler is actually working?
**Answer:**  
We proved scheduler operation empirically by:
1. Exporting timestamped trace events (`DISPATCH`, `PREEMPT`, `TASK_BLOCKED`, `TASK_TERMINATED`) to CSV.
2. Ingesting traces into `scripts/trace_parser.py` to calculate exact turnaround times and verify that context switch counts match theoretical $(N \times \lceil\text{Cycles}/Q\rceil)$ expectations.
3. Observing interleaved terminal output tagged with task IDs.

---

### Q16: How is your Micro-OS different from Linux?
**Answer:**  
| Feature | Linux Kernel | Micro-OS |
|---|---|---|
| **Scheduler** | Completely Fair Scheduler (CFS) via red-black tree ($vruntime$) | Educational Round Robin via FIFO queue |
| **Address Space** | Hardware MMU multi-level page tables with demand paging | Virtual stack/heap offsets within flat simulated memory |
| **Locking** | Hybrid adaptive futexes, reader-writer semaphores, RCU | Spinlock, Fair Ticket Lock, and Sleeping Mutex |
| **Scale** | Millions of lines of C and assembly | Compact, modular, observable Rust kernel (~1,500 LOC) |

---

### Q17: What did CA-5 teach you that helped this project?
**Answer:**  
CA-5 (Linux Systems Engineering) exposed how real operating systems handle task priority, voluntary vs involuntary context switches in `/proc/[pid]/status`, and lock contention. In CA-6, we used those insights to design the exact same metrics into our `TaskMetrics` and `SchedulerEvent` structs, making internal kernel decisions directly observable.

---

### Q18: What CA-4 research question could be extended into your Micro-OS?
**Answer:**  
In CA-4, we investigated: *"How does lock contention impact throughput under high thread counts?"*  
In CA-6 Track C, we directly implemented that research question at the kernel level by building `ContentionReport`, benchmarking 2 to 16 tasks contending on a lock, and showing that contention escalates non-linearly to 93.8% under 16 tasks.

---

### Q19: What would you implement next?
**Answer:**  
1. **Multi-Level Feedback Queue (MLFQ)**: To dynamically balance I/O-bound tasks against CPU-bound tasks.
2. **Two-Level Paging**: Implementing a virtual memory manager with page directories and simulated page fault exception handlers.
3. **Simulated Block Device**: Adding a virtual ATA/NVMe driver with an inode-based filesystem.

---

### Q20: What is the most important limitation of your current system?
**Answer:**  
The most significant limitation is **uniprocessor execution**: the kernel simulates a single CPU core interleaving tasks across discrete ticks. In true multicore symmetric multiprocessing (SMP), spinlocks can execute concurrently across physical cores, requiring hardware cache coherence protocols (MESI) and inter-processor interrupts (IPIs) which are beyond our single-core virtual model.

---

### Q21: How does your Micro-OS connect to the internet?
**Answer:**  
Micro-OS implements an end-to-end network stack interfacing with an **Intel 82540EM (e1000)** Gigabit Ethernet PCI device:
1. **Layer 2 (Data Link)**: Ethernet II framing (`src/net/ethernet.rs`) with MAC address resolution via ARP (`src/net/arp.rs`).
2. **Layer 3 (Network)**: IPv4 packets (`src/net/ipv4.rs`) with RFC 791 16-bit one's complement checksum calculation.
3. **Layer 4 (Transport)**: UDP (`src/net/udp.rs`) for DHCP and DNS, and ICMP (`src/net/icmp.rs`) for Ping Echo requests.
4. **Layer 7 (Application)**:
   - **DHCP Client (`src/net/dhcp.rs`)**: Broadcasts DHCP Discover on UDP port 68, accepts DHCP Offer, requests lease, and parses DHCP Ack. Dynamically assigns IP `10.0.2.15`, Gateway `10.0.2.2`, Subnet Mask `255.255.255.0`, and DNS Server `10.0.2.3`.
   - **ICMP Ping**: Pings the default gateway (`10.0.2.2`) and public internet DNS (`8.8.8.8`) to empirically measure round-trip time (RTT).
   - **HTTP Client (`src/net/http.rs`)**: Connects over TCP to port 80, transmits `GET / HTTP/1.1`, and parses the server's `HTTP/1.1 200 OK` response.

---

### Q22: How does the bootable ISO image (`micro_os.iso`) work?
**Answer:**  
The ISO image follows the **El Torito Bootable CD-ROM Specification (ISO-9660)**:
1. Sector 16 contains the **Primary Volume Descriptor (PVD)** establishing standard volume descriptors.
2. Sector 17 contains the **El Torito Boot Record Volume Descriptor**, pointing to the Boot Catalog at Sector 19.
3. Sector 19 contains the **Boot Catalog**:
   - Validation entry with platform ID `0x00` (80x86) and a 16-word checksum summing to `0x0000`.
   - Initial entry configured for **"No Emulation" mode**, instructing the BIOS to load 16 virtual 512-byte sectors starting at LBA 20 directly into physical memory at `0x0000:0x7C00`.
4. Sector 20 contains the kernel boot sector ending with the standard `0x55AA` signature.
When booted in QEMU, VirtualBox, or VMware, the BIOS jumps to `0x7C00`, initializes VGA 80x25 text video mode, renders the Micro-OS graphical dashboard, displays the running scheduler, concurrency locks, and shows live Intel e1000 internet connectivity (DHCP, Ping, HTTP).
