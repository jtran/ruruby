use super::*;
use region::{protect, Protection};
use std::alloc::{alloc, Layout};

const DEFAULT_STACK_SIZE: usize = 1024 * 128;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct Stack(*mut u8);

impl Stack {
    pub fn allocate() -> Self {
        let stack =
            unsafe { alloc(Layout::from_size_align(DEFAULT_STACK_SIZE, 16).expect("Bad Layout.")) };
        unsafe {
            protect(stack, DEFAULT_STACK_SIZE, Protection::READ_WRITE).expect("Mprotect failed.");
        }
        Self(stack)
    }

    pub fn deallocate(self) {
        unsafe {
            std::alloc::dealloc(
                self.0,
                Layout::from_size_align(DEFAULT_STACK_SIZE, 16).expect("Bad Layout."),
            )
        };
    }

    pub fn init(
        &mut self,
        fiber: *const FiberContext,
        f: extern "C" fn(FiberHandle, Value) -> *mut VMResult,
    ) -> u64 {
        unsafe {
            let s_ptr = self.0.offset(DEFAULT_STACK_SIZE as isize);
            (s_ptr.offset(-8) as *mut u64).write(fiber as u64);
            (s_ptr.offset(-16) as *mut u64).write(guard as u64);
            // this is a dummy function for 16bytes-align.
            (s_ptr.offset(-24) as *mut u64).write(asm::skip as u64);
            (s_ptr.offset(-32) as *mut u64).write(f as u64);
            // 48 bytes to store registers.
            s_ptr.offset(-80) as u64
        }
    }
}

extern "C" fn guard(fiber: *mut FiberContext, val: *mut VMResult) {
    unsafe {
        (*fiber).state = FiberState::Dead;
    }
    asm::yield_context(fiber, val);
}
