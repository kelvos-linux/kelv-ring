//! Fixed x86-64 bootstrap and virtual-memory layout.
//! RAM and MMIO discovered from firmware are handled by the kernel at boot.

pub const PAGE_SIZE: u64 = 4096;
pub const BOOT_IDENTITY_END: u64 = 1 << 32;
pub const BOOT_DIRECTORY_COUNT: u64 = BOOT_IDENTITY_END >> 30;
pub const BOOT_LARGE_PAGE_COUNT: u64 = BOOT_IDENTITY_END >> 21;

pub const LOW_RESERVED_END: u64 = 0x10_0000;
pub const KERNEL_BASE: u64 = 0x10_0000;
pub const AP_TRAMPOLINE_BASE: u64 = 0x8000;
pub const AP_STACK_TOP: u64 = AP_TRAMPOLINE_BASE + PAGE_SIZE - 16;
pub const AP_TRAMPOLINE_MAX_SIZE: u64 = PAGE_SIZE - 0x100;
pub const LAPIC_DEFAULT: u64 = 0xfee0_0000;

pub const USER_LIMIT: u64 = 1 << 47;
pub const USER_IMAGE_ADDRESS: u64 = 0x1_0000_0000;
pub const USER_IMAGE_CAPACITY: usize = 0x1_0000;
pub const USER_IMAGE_END: u64 = USER_IMAGE_ADDRESS + USER_IMAGE_CAPACITY as u64;
pub const USER_RING_ADDRESS: u64 = USER_IMAGE_ADDRESS + 0x1_0000;
pub const USER_STACK_TOP: u64 = USER_IMAGE_ADDRESS + 0x2_2000;
pub const SPAWN_STACK_TOP: u64 = USER_IMAGE_ADDRESS + 0x2_6000;
pub const USER_STACK_SIZE: usize = 2 * PAGE_SIZE as usize;

pub const HEAP_START: u64 = 0xffff_ffff_c000_0000;
pub const HEAP_SIZE: usize = 0x100_0000;

const fn aligned(value: u64) -> bool {
    value % PAGE_SIZE == 0
}

const _: () = {
    assert!(PAGE_SIZE == 4096 && BOOT_DIRECTORY_COUNT == 4);
    assert!(aligned(LOW_RESERVED_END) && aligned(KERNEL_BASE) && aligned(LAPIC_DEFAULT));
    assert!(LOW_RESERVED_END >= 0x10_0000 && LOW_RESERVED_END <= KERNEL_BASE);
    assert!(KERNEL_BASE < LAPIC_DEFAULT && LAPIC_DEFAULT + PAGE_SIZE <= BOOT_IDENTITY_END);
    assert!(aligned(AP_TRAMPOLINE_BASE) && AP_TRAMPOLINE_BASE >= PAGE_SIZE);
    assert!(AP_STACK_TOP <= u16::MAX as u64 && AP_TRAMPOLINE_BASE + PAGE_SIZE <= LOW_RESERVED_END);

    assert!(aligned(USER_IMAGE_ADDRESS) && aligned(USER_IMAGE_END) && aligned(USER_RING_ADDRESS));
    assert!(aligned(USER_STACK_TOP) && aligned(SPAWN_STACK_TOP) && aligned(USER_STACK_SIZE as u64));
    assert!(BOOT_IDENTITY_END <= USER_IMAGE_ADDRESS && USER_IMAGE_ADDRESS < USER_IMAGE_END);
    assert!(USER_IMAGE_END <= USER_RING_ADDRESS);
    assert!(USER_RING_ADDRESS + 2 * PAGE_SIZE <= USER_STACK_TOP - USER_STACK_SIZE as u64);
    assert!(
        USER_STACK_TOP <= SPAWN_STACK_TOP - USER_STACK_SIZE as u64 && SPAWN_STACK_TOP <= USER_LIMIT
    );

    assert!(aligned(HEAP_START) && aligned(HEAP_SIZE as u64) && HEAP_SIZE > 0);
    assert!(HEAP_START >= u64::MAX - USER_LIMIT + 1);
    assert!(HEAP_START.checked_add(HEAP_SIZE as u64 - 1).is_some());
};
