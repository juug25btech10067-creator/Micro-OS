//! Comprehensive Integration Tests covering TC1 through TC6 as required by CA-6,
//! plus TC7 through TC10 for the Internet & Networking Subsystem.

use micro_os_kernel::memory::{BlockAllocator, BumpAllocator};
use micro_os_kernel::net::{
    EtherType, EthernetFrame, HttpRequest, HttpResponse, IcmpEchoPacket,
    Ipv4Address, Ipv4Packet, IpProtocol, MacAddress, NetworkStack,
};
use micro_os_kernel::scheduler::KernelEngine;
use micro_os_kernel::sync::{DeadlockManager, SleepMutex, Spinlock};
use micro_os_kernel::task::{TaskControlBlock, TaskId, TaskInstruction, TaskState};

/// TC1 - Minimum Configuration: Two tasks executing Round Robin.
#[test]
fn test_tc1_minimum_configuration() {
    let mut engine = KernelEngine::new(2, 4096);

    let t1 = TaskControlBlock::new(
        TaskId(1),
        "Task-1",
        1,
        vec![TaskInstruction::Compute(10), TaskInstruction::Exit],
        0x1000,
        0x1000,
        0,
    );
    let t2 = TaskControlBlock::new(
        TaskId(2),
        "Task-2",
        1,
        vec![TaskInstruction::Compute(10), TaskInstruction::Exit],
        0x2000,
        0x1000,
        0,
    );

    engine.spawn_task(t1);
    engine.spawn_task(t2);

    engine.run_until_complete(50);

    assert_eq!(engine.tasks.get(&TaskId(1)).unwrap().state, TaskState::Terminated);
    assert_eq!(engine.tasks.get(&TaskId(2)).unwrap().state, TaskState::Terminated);
    assert!(engine.total_context_switches >= 2, "Expected at least 2 context switches");
}

/// TC2 - Multiple Tasks: 4 tasks with mixed CPU compute, memory, and voluntary yields.
#[test]
fn test_tc2_multiple_tasks() {
    let mut engine = KernelEngine::new(2, 8192);

    for i in 1..=4 {
        let instructions = vec![
            TaskInstruction::Compute(5),
            TaskInstruction::Yield,
            TaskInstruction::Compute(5),
            TaskInstruction::Exit,
        ];
        engine.spawn_task(TaskControlBlock::new(
            TaskId(i),
            &format!("Task-{}", i),
            5,
            instructions,
            0x1000 * i,
            0x1000,
            0,
        ));
    }

    engine.run_until_complete(100);

    for i in 1..=4 {
        assert_eq!(
            engine.tasks.get(&TaskId(i)).unwrap().state,
            TaskState::Terminated,
            "Task-{} did not terminate",
            i
        );
    }
    assert!(engine.total_context_switches >= 8);
}

/// TC3 - Boundary Case: Smallest meaningful parameters (quantum = 1 tick, 0-byte allocation).
#[test]
fn test_tc3_boundary_cases() {
    // Boundary 1: Minimum quantum = 1
    let mut engine = KernelEngine::new(1, 4096);
    let t = TaskControlBlock::new(
        TaskId(1),
        "BoundaryTask",
        1,
        vec![TaskInstruction::Compute(5), TaskInstruction::Exit],
        0x1000,
        0x1000,
        0,
    );
    engine.spawn_task(t);
    engine.run_until_complete(20);
    assert_eq!(engine.tasks.get(&TaskId(1)).unwrap().state, TaskState::Terminated);

    // Boundary 2: 0-byte memory allocation
    let mut bump = BumpAllocator::new(1024);
    let zero_alloc = bump.allocate(0);
    assert!(zero_alloc.is_ok(), "0-byte allocation must be supported as a no-op");

    // Boundary 3: 1-byte memory allocation (aligned to 8 bytes)
    let one_byte_alloc = bump.allocate(1);
    assert!(one_byte_alloc.is_ok());
}

/// TC4 - Stress Case: 10 concurrent tasks contending on a single sleeping mutex.
#[test]
fn test_tc4_stress_concurrency() {
    let mut engine = KernelEngine::new(2, 16384);
    engine.register_mutex(1, "HotLock", 10);

    let num_tasks = 10;
    for i in 1..=num_tasks {
        let instructions = vec![
            TaskInstruction::MutexLock(1),
            TaskInstruction::Compute(3),
            TaskInstruction::MutexUnlock(1),
            TaskInstruction::Exit,
        ];
        engine.spawn_task(TaskControlBlock::new(
            TaskId(i),
            &format!("StressTask-{}", i),
            1,
            instructions,
            0x1000 * i,
            0x1000,
            0,
        ));
    }

    engine.run_until_complete(300);

    for i in 1..=num_tasks {
        assert_eq!(
            engine.tasks.get(&TaskId(i)).unwrap().state,
            TaskState::Terminated,
            "StressTask-{} did not finish",
            i
        );
    }

    let m = engine.mutexes.get(&1).unwrap();
    assert_eq!(m.stats().total_acquisitions, num_tasks);
    assert!(m.stats().contended_acquisitions > 0, "Expected lock contention");
}

/// TC5 - Failure Case: Controlled invalid operations (deadlock violation, double-free, OOM).
#[test]
fn test_tc5_failure_cases() {
    // Failure 1: Deadlock hierarchy violation (acquiring lower rank after higher rank)
    let mut deadlock_mgr = DeadlockManager::new();
    let task = TaskId(42);
    // Acquire Rank 50
    assert!(deadlock_mgr.validate_acquisition(task, 1, 50).is_ok());
    deadlock_mgr.record_acquired(task, 1, 50);

    // Try to acquire Rank 20 (Lower rank -> Circular Wait Risk!)
    let violation = deadlock_mgr.validate_acquisition(task, 2, 20);
    assert!(violation.is_err(), "Must reject lock acquisition violating rank ordering");
    assert_eq!(deadlock_mgr.violations_count(), 1);

    // Failure 2: Out of Memory (OOM) rejection
    let mut bump = BumpAllocator::new(128);
    let oom_err = bump.allocate(256);
    assert!(oom_err.is_err(), "Must return OOM error when heap is exhausted");
    assert_eq!(bump.stats().out_of_memory_errors, 1);

    // Failure 3: Double-Free rejection in Block Allocator
    let mut pool = BlockAllocator::new(256); // 4 blocks
    let addr = pool.allocate_block().expect("alloc");
    assert!(pool.deallocate_block(addr).is_ok());
    let double_free = pool.deallocate_block(addr);
    assert!(double_free.is_err(), "Must detect and reject double free");
}

/// TC6 - Repeatability: Run identical workloads and assert deterministic outputs.
#[test]
fn test_tc6_repeatability() {
    fn run_sim() -> (usize, usize, usize) {
        let mut engine = KernelEngine::new(2, 4096);
        engine.register_mutex(1, "TestMutex", 10);

        for i in 1..=3 {
            let instructions = vec![
                TaskInstruction::Compute(2),
                TaskInstruction::MutexLock(1),
                TaskInstruction::Compute(4),
                TaskInstruction::MutexUnlock(1),
                TaskInstruction::Exit,
            ];
            engine.spawn_task(TaskControlBlock::new(
                TaskId(i),
                &format!("RepeatTask-{}", i),
                1,
                instructions,
                0x1000 * i,
                0x1000,
                0,
            ));
        }

        engine.run_until_complete(100);
        (
            engine.current_tick,
            engine.total_context_switches,
            engine.mutexes.get(&1).unwrap().stats().total_acquisitions,
        )
    }

    let run1 = run_sim();
    let run2 = run_sim();
    let run3 = run_sim();

    assert_eq!(run1, run2, "Execution 1 and 2 must produce deterministic results");
    assert_eq!(run2, run3, "Execution 2 and 3 must produce deterministic results");
}

/// TC7 - Internet & Networking: Ethernet II Frame Packing & Parsing.
#[test]
fn test_tc7_ethernet_framing() {
    let src = MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let dest = MacAddress::BROADCAST;
    let payload = b"Hello Micro-OS Internet".to_vec();

    let frame = EthernetFrame::new(dest, src, EtherType::Ipv4, payload.clone());
    let bytes = frame.to_bytes();
    assert_eq!(bytes.len(), 14 + payload.len());

    let parsed = EthernetFrame::parse(&bytes).expect("Failed to parse Ethernet frame");
    assert_eq!(parsed.src_mac, src);
    assert_eq!(parsed.dest_mac, dest);
    assert_eq!(parsed.ethertype, EtherType::Ipv4);
    assert_eq!(parsed.payload, payload);
}

/// TC8 - Internet & Networking: IPv4 Header Checksum & Validation.
#[test]
fn test_tc8_ipv4_checksum_and_validation() {
    let src = Ipv4Address::new(10, 0, 2, 15);
    let dest = Ipv4Address::new(8, 8, 8, 8);
    let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];

    let packet = Ipv4Packet::new(src, dest, IpProtocol::Udp, payload.clone());
    let bytes = packet.to_bytes();

    let parsed = Ipv4Packet::parse(&bytes).expect("Failed to parse IPv4 packet");
    assert_eq!(parsed.src_ip, src);
    assert_eq!(parsed.dest_ip, dest);
    assert_eq!(parsed.protocol, IpProtocol::Udp);
    assert_eq!(parsed.payload, payload);

    // Verify RFC 791 header checksum
    let header_csum = Ipv4Packet::calculate_checksum(&bytes[0..20]);
    let stored_csum = ((bytes[10] as u16) << 8) | (bytes[11] as u16);
    assert_eq!(header_csum, stored_csum);
}

/// TC9 - Internet & Networking: Live DHCP Negotiation & ICMP Ping.
#[test]
fn test_tc9_dhcp_and_ping() {
    let mac = MacAddress([0x52, 0x54, 0x00, 0xAA, 0xBB, 0xCC]);
    let mut net = NetworkStack::new(mac);

    let cfg = net.perform_dhcp_negotiation().expect("DHCP failed");
    assert_eq!(cfg.ip_address, Ipv4Address::new(10, 0, 2, 15));
    assert_eq!(cfg.gateway, Ipv4Address::new(10, 0, 2, 2));
    assert!(net.internet_connected);

    let rtt = net.ping(cfg.gateway).expect("Ping failed");
    assert!(rtt > 0, "RTT must be positive");
}

/// TC10 - Internet & Networking: HTTP/1.1 Client Request & Status Parsing.
#[test]
fn test_tc10_http_client() {
    let req = HttpRequest::new_get("micro-os.org", "/api/status");
    let wire = req.to_http_wire();
    assert!(wire.starts_with("GET /api/status HTTP/1.1\r\nHost: micro-os.org\r\n"));

    let raw_response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nMicro-OS Online";
    let parsed = HttpResponse::parse(raw_response).expect("Failed to parse HTTP response");
    assert_eq!(parsed.status_code, 200);
    assert_eq!(parsed.status_text, "OK");
    assert_eq!(parsed.body, "Micro-OS Online");
}
