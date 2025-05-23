use crate::mem::{PhysAddr, VirtAddr, virt_to_phys};

/// Hart number of rk3588 board
pub const MAX_HARTS: usize = 8;
/// CPU HWID from cpu device tree nodes with "reg" property
pub const CPU_HWID: [usize; MAX_HARTS] = [0x00, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700];

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: PhysAddr) {
    assert!(cpu_id < MAX_HARTS, "No support for rk3588 core {}", cpu_id);
    unsafe extern "C" {
        fn _start_secondary();
    }
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    crate::platform::aarch64_common::psci::cpu_on(
        CPU_HWID[cpu_id],
        entry.as_usize(),
        stack_top.as_usize(),
    );
}
