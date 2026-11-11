#![cfg_attr(not(test), no_std)]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU32, Ordering};

pub const RING_CAPACITY: u32 = 8;
pub const OP_SPAWN: u32 = 1;

pub const USER_IMAGE_ADDRESS: u64 = 0x1_0000_0000;
pub const USER_IMAGE_CAPACITY: usize = 64 * 1024;
pub const USER_RING_ADDRESS: u64 = USER_IMAGE_ADDRESS + 0x1_0000;
pub const USER_STACK_TOP: u64 = USER_IMAGE_ADDRESS + 0x2_2000;
pub const SPAWN_STACK_TOP: u64 = USER_IMAGE_ADDRESS + 0x2_6000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const MAX_SPAWNED_TASKS: u32 = RING_CAPACITY;
pub const MAX_TASKS: usize = MAX_SPAWNED_TASKS as usize + 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Submission {
    pub opcode: u32,
    pub flags: u32,
    pub entry_point: u64,
    pub stack_top: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Completion {
    pub user_data: u64,
    pub result: i64,
}

#[repr(C, align(64))]
pub struct SharedRing {
    pub sq_head: AtomicU32,
    pub sq_tail: AtomicU32,
    pub cq_head: AtomicU32,
    pub cq_tail: AtomicU32,
    /// Bit N is set after user task N reaches its entry point.
    pub child_started: AtomicU32,
    submissions: UnsafeCell<[Submission; RING_CAPACITY as usize]>,
    completions: UnsafeCell<[Completion; RING_CAPACITY as usize]>,
}

unsafe impl Sync for SharedRing {}

impl SharedRing {
    pub const fn new() -> Self {
        Self {
            sq_head: AtomicU32::new(0),
            sq_tail: AtomicU32::new(0),
            cq_head: AtomicU32::new(0),
            cq_tail: AtomicU32::new(0),
            child_started: AtomicU32::new(0),
            submissions: UnsafeCell::new(
                [Submission {
                    opcode: 0,
                    flags: 0,
                    entry_point: 0,
                    stack_top: 0,
                }; RING_CAPACITY as usize],
            ),
            completions: UnsafeCell::new(
                [Completion {
                    user_data: 0,
                    result: 0,
                }; RING_CAPACITY as usize],
            ),
        }
    }

    pub unsafe fn read_submission(&self, sequence: u32) -> Submission {
        unsafe { (*self.submissions.get())[(sequence % RING_CAPACITY) as usize] }
    }

    pub unsafe fn write_completion(&self, sequence: u32, completion: Completion) {
        unsafe {
            (*self.completions.get())[(sequence % RING_CAPACITY) as usize] = completion;
        }
    }

    pub unsafe fn read_completion(&self, sequence: u32) -> Completion {
        unsafe { (*self.completions.get())[(sequence % RING_CAPACITY) as usize] }
    }

    pub unsafe fn write_submission(&self, sequence: u32, submission: Submission) {
        unsafe {
            (*self.submissions.get())[(sequence % RING_CAPACITY) as usize] = submission;
        }
    }

    pub fn producer_has_space(&self, tail: u32) -> bool {
        tail.wrapping_sub(self.sq_head.load(Ordering::Acquire)) < RING_CAPACITY
    }

    pub fn consumer_has_entries(&self, head: u32) -> bool {
        head != self.cq_tail.load(Ordering::Acquire)
    }
}

impl Default for SharedRing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_and_consumer_exchange_one_spawn_entry() {
        let ring = SharedRing::new();
        let submission = Submission {
            opcode: OP_SPAWN,
            flags: 0,
            entry_point: USER_IMAGE_ADDRESS,
            stack_top: SPAWN_STACK_TOP,
        };

        assert!(ring.producer_has_space(0));
        unsafe { ring.write_submission(0, submission) };
        ring.sq_tail.store(1, Ordering::Release);

        let tail = ring.sq_tail.load(Ordering::Acquire);
        assert_eq!(tail, 1);
        assert_eq!(unsafe { ring.read_submission(0) }, submission);
        ring.sq_head.store(1, Ordering::Release);

        unsafe {
            ring.write_completion(
                0,
                Completion {
                    user_data: 7,
                    result: 1,
                },
            )
        };
        ring.cq_tail.store(1, Ordering::Release);
        assert!(ring.consumer_has_entries(0));
        assert_eq!(
            unsafe { ring.read_completion(0) },
            Completion {
                user_data: 7,
                result: 1
            }
        );
    }
}
