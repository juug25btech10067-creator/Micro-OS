//! CA-6 Micro-OS Main Demonstration Binary.

use micro_os_kernel::kernel::{boot::SystemConfig, BootManager};
use micro_os_kernel::memory::BlockAllocator;
use micro_os_kernel::net::{Ipv4Address, MacAddress, NetworkStack};
use micro_os_kernel::scheduler::KernelEngine;
use micro_os_kernel::sync::{ContentionReport, Spinlock};
use micro_os_kernel::task::{TaskControlBlock, TaskId, TaskInstruction};
use std::fs::File;
use std::io::Write;

fn main() {
    println!("\n============================================================");
    println!("     STARTING CA-6 RUST MICRO-OS DEMONSTRATION SUITE        ");
    println!("============================================================");

    // 1. Boot Phase
    let config = SystemConfig::default();
    let mut boot_mgr = BootManager::new(config.clone());
    boot_mgr.boot().expect("Kernel boot failure");

    // 2. Demonstration 1: Integrated Round Robin Scheduling (TC1 & TC2)
    demo_round_robin_multitasking(&config);

    // 3. Demonstration 2: Track C - Synchronization & Mutex Blocking
    demo_concurrency_synchronization(&config);

    // 4. Demonstration 3: Track C - Contention & Scalability Benchmark
    demo_contention_benchmark(&config);

    // 5. Demonstration 4: TC5 - Deadlock Prevention & Invariant Verification
    demo_deadlock_prevention(&config);

    // 6. Demonstration 5: TC3 - Memory Allocation Boundaries & Free-List
    demo_memory_boundaries_and_freelist();

    // 7. Demonstration 6: Internet & Networking Subsystem (DHCP, Ping, HTTP)
    demo_internet_networking();

    println!("\n============================================================");
    println!("     ALL CA-6 DEMONSTRATIONS COMPLETED SUCCESSFULLY!        ");
    println!("============================================================\n");
}

/// Demo 1: Core Multitasking & Round Robin Scheduling with 4 tasks.
fn demo_round_robin_multitasking(config: &SystemConfig) {
    println!("\n>>> DEMO 1: ROUND ROBIN MULTITASKING (TC1 & TC2) <<<");
    let mut engine = KernelEngine::new(config.quantum_ticks, config.heap_size_bytes);

    let t1_instr = vec![
        TaskInstruction::Compute(10),
        TaskInstruction::Allocate(128),
        TaskInstruction::Compute(15),
        TaskInstruction::Exit,
    ];
    let t2_instr = vec![
        TaskInstruction::Compute(20),
        TaskInstruction::Compute(20),
        TaskInstruction::Exit,
    ];
    let t3_instr = vec![
        TaskInstruction::Compute(5),
        TaskInstruction::Yield,
        TaskInstruction::Compute(10),
        TaskInstruction::Exit,
    ];
    let t4_instr = vec![
        TaskInstruction::Allocate(64),
        TaskInstruction::Compute(15),
        TaskInstruction::Exit,
    ];

    engine.spawn_task(TaskControlBlock::new(TaskId(1), "Worker-Alpha", 10, t1_instr, 0x1000, 0x1000, 0));
    engine.spawn_task(TaskControlBlock::new(TaskId(2), "Worker-Beta", 10, t2_instr, 0x2000, 0x1000, 0));
    engine.spawn_task(TaskControlBlock::new(TaskId(3), "Worker-Gamma", 15, t3_instr, 0x3000, 0x1000, 0));
    engine.spawn_task(TaskControlBlock::new(TaskId(4), "Worker-Delta", 5, t4_instr, 0x4000, 0x1000, 0));

    engine.run_until_complete(100);
    engine.print_metrics_summary();

    export_traces(&engine, "traces/demo1_rr_trace.csv");
}

/// Demo 2: Concurrency Specialization - Mutex Blocking & Sleep/Wakeup
fn demo_concurrency_synchronization(config: &SystemConfig) {
    println!("\n>>> DEMO 2: TRACK C - SLEEPING MUTEX & BLOCKING <<<");
    let mut engine = KernelEngine::new(config.quantum_ticks, config.heap_size_bytes);

    // Mutex 1 with rank 10
    engine.register_mutex(1, "SharedDatabaseMutex", 10);

    // Task 1 acquires mutex, does long computation, then releases
    let t1_instr = vec![
        TaskInstruction::Compute(5),
        TaskInstruction::MutexLock(1),
        TaskInstruction::Compute(30), // Holds lock across multiple quantum slices
        TaskInstruction::MutexUnlock(1),
        TaskInstruction::Exit,
    ];

    // Task 2 attempts to acquire contested mutex, gets BLOCKED, then wakes up
    let t2_instr = vec![
        TaskInstruction::Compute(2),
        TaskInstruction::MutexLock(1), // Will find it locked by Task 1!
        TaskInstruction::Compute(10),
        TaskInstruction::MutexUnlock(1),
        TaskInstruction::Exit,
    ];

    engine.spawn_task(TaskControlBlock::new(TaskId(1), "LockHolder", 10, t1_instr, 0x1000, 0x1000, 0));
    engine.spawn_task(TaskControlBlock::new(TaskId(2), "Contender", 10, t2_instr, 0x2000, 0x1000, 0));

    engine.run_until_complete(100);
    engine.print_metrics_summary();

    if let Some(m) = engine.mutexes.get(&1) {
        let rep = ContentionReport::from_mutex_stats(m.name(), 2, m.stats(), engine.current_tick as u64);
        rep.display_summary();
    }

    export_traces(&engine, "traces/demo2_mutex_trace.csv");
}

/// Demo 3: Lock Contention & Scalability Benchmark under varying task loads.
fn demo_contention_benchmark(config: &SystemConfig) {
    println!("\n>>> DEMO 3: TRACK C - CONTENTION & SCALABILITY EVALUATION <<<");

    for num_tasks in [2, 4, 8] {
        println!("\n--- Benchmarking Contention with {} Tasks ---", num_tasks);
        let mut engine = KernelEngine::new(1, config.heap_size_bytes);
        engine.register_mutex(10, "HighContentionResource", 100);

        for i in 1..=num_tasks {
            let task_instructions = vec![
                TaskInstruction::Compute(2),
                TaskInstruction::MutexLock(10),
                TaskInstruction::Compute(4),
                TaskInstruction::MutexUnlock(10),
                TaskInstruction::Exit,
            ];
            engine.spawn_task(TaskControlBlock::new(
                TaskId(i),
                &format!("Contender-{}", i),
                10,
                task_instructions,
                0x1000 * i,
                0x1000,
                0,
            ));
        }

        engine.run_until_complete(200);
        if let Some(m) = engine.mutexes.get(&10) {
            let rep = ContentionReport::from_mutex_stats(m.name(), num_tasks, m.stats(), engine.current_tick as u64);
            rep.display_summary();
        }
    }
}

/// Demo 4: TC5 - Controlled Deadlock Prevention via Hierarchical Lock Ranking.
fn demo_deadlock_prevention(config: &SystemConfig) {
    println!("\n>>> DEMO 4: TC5 - DEADLOCK PREVENTION INVARIANT CHECK <<<");
    let mut engine = KernelEngine::new(config.quantum_ticks, config.heap_size_bytes);

    // Register Lock A (rank 10) and Lock B (rank 20)
    engine.register_spinlock(1, "LockA_LowRank", 10);
    engine.register_spinlock(2, "LockB_HighRank", 20);

    // Illegal Task: Tries to acquire Lock B (rank 20) and then Lock A (rank 10) -> Rank Inversion!
    let illegal_instructions = vec![
        TaskInstruction::SpinlockAcquire(2), // Rank 20 acquired
        TaskInstruction::SpinlockAcquire(1), // Rank 10 requested -> VIOLATION!
        TaskInstruction::SpinlockRelease(2),
        TaskInstruction::Exit,
    ];

    engine.spawn_task(TaskControlBlock::new(
        TaskId(99),
        "InvertingTask",
        10,
        illegal_instructions,
        0x9000,
        0x1000,
        0,
    ));

    engine.run_until_complete(50);
    println!(
        "Result: Deadlock Prevention correctly caught {} circular wait violation(s)!",
        engine.deadlock_mgr.violations_count()
    );
    assert!(engine.deadlock_mgr.violations_count() >= 1, "Invariant check failed: Violation was not caught!");
}

/// Demo 5: TC3 - Memory Allocator Boundaries & Free-List.
fn demo_memory_boundaries_and_freelist() {
    println!("\n>>> DEMO 5: TC3 - MEMORY BOUNDARIES & FREE-LIST RECYCLING <<<");
    let mut pool = BlockAllocator::new(1024); // 1 KB pool (16 blocks of 64 bytes)
    println!("Initial free blocks: {}", pool.free_blocks_count());

    // 1. Allocate blocks
    let b1 = pool.allocate_block().expect("Alloc 1");
    let b2 = pool.allocate_block().expect("Alloc 2");
    println!("Allocated Block 1 at offset 0x{:04x}, Block 2 at offset 0x{:04x}", b1, b2);
    println!("Free blocks remaining: {}", pool.free_blocks_count());

    // 2. Deallocate block 1 and verify recycling
    pool.deallocate_block(b1).expect("Free block 1");
    println!("Deallocated Block 1. Free blocks now: {}", pool.free_blocks_count());

    // 3. Reallocate: should reuse block 1
    let b3 = pool.allocate_block().expect("Alloc 3");
    println!("Reallocated block offset: 0x{:04x} (Recycled from free list!)", b3);
    assert_eq!(b1, b3, "Block recycling failed: did not reuse freed block!");

    println!("Block Allocator Stats: {}", pool.stats());
}

/// Demo 6: Internet & Networking Subsystem (Intel e1000, DHCP, Ping & HTTP).
fn demo_internet_networking() {
    println!("\n>>> DEMO 6: INTERNET & NETWORKING SUBSYSTEM (DHCP, PING, HTTP) <<<");

    // Initialize Intel e1000 NIC with MAC 52:54:00:12:34:56 (standard QEMU/KVM OUI)
    let mac = MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let mut net = NetworkStack::new(mac);

    // 1. Negotiate DHCP configuration
    let cfg = net.perform_dhcp_negotiation().expect("DHCP configuration failed");
    println!("Acquired IP via DHCP: {}", cfg.ip_address);

    // 2. Transmit ICMP Ping to Internet Gateway (10.0.2.2) and Public DNS (8.8.8.8)
    let gateway_rtt = net.ping(cfg.gateway).expect("Ping to gateway failed");
    println!("Gateway Ping Verified: {} ms RTT", gateway_rtt * 10);

    let public_dns = Ipv4Address::new(8, 8, 8, 8);
    let dns_rtt = net.ping(public_dns).expect("Ping to 8.8.8.8 failed");
    println!("Public Internet Ping (8.8.8.8) Verified: {} ms RTT", dns_rtt * 10);

    // 3. Live HTTP 1.1 Web Client Request
    let response = net.http_get("micro-os.kernel.org", "/status").expect("HTTP GET failed");
    println!("HTTP Client Connection Status: {} {}", response.status_code, response.status_text);
    println!("HTTP Body Received: {}", response.body.trim());

    // Print summary
    net.print_network_summary();
}

/// Export scheduler events to CSV for Python trace analysis.
fn export_traces(engine: &KernelEngine, path: &str) {
    if let Ok(mut file) = File::create(path) {
        writeln!(file, "tick,event_type,task_id,details").unwrap();
        for ev in &engine.scheduler_events {
            let tid = ev.task_id.map(|t| t.0.to_string()).unwrap_or_else(|| "NONE".to_string());
            writeln!(file, "{},{},{},\"{}\"", ev.tick, ev.event_type, tid, ev.details).unwrap();
        }
        println!("Exported scheduler trace to: {}", path);
    }
}
