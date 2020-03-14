//! Minimal and simpler alternative to the futures crate.
//!
//! - No required std
//! - No allocations
//! - No procedural macros (for faster compile times)
//! - No dependencies
//! - No cost (True zero-cost abstractions!)
//! - No pain (API super easy to learn & use!)
//! - No unsafe code in pinning macro (allowing you to `forbid(unsafe_code)`)

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[doc(hidden)]
pub mod _pasts_hide {
    #[cfg(feature = "std")]
    pub extern crate std;

    #[cfg(feature = "std")]
    pub use std as stn;

    #[cfg(not(feature = "std"))]
    pub use core as stn;

    /// Not actually safe task pinning only for use in macros.
    #[inline(always)]
    pub fn new_task<F, O>(
        future: &mut F,
    ) -> (crate::Task<F, O>, stn::mem::MaybeUninit<O>)
    where
        F: self::stn::future::Future<Output = O>,
    {
        (
            crate::Task::new(self::new_pin(future)),
            stn::mem::MaybeUninit::uninit(),
        )
    }

    /// Not actually safe pinning only for use in macros.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn new_pin<P>(pointer: P) -> stn::pin::Pin<P>
    where
        P: stn::ops::Deref,
    {
        unsafe { stn::pin::Pin::new_unchecked(pointer) }
    }

    /// Not actually safe: This is needed to initialize task queue.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn transmute_slice<A: Sized, B: Sized>(a: &mut [A]) -> &mut [B] {
        unsafe {
            stn::mem::transmute::<&mut [A], &mut [B]>(a)
        }
    }

    /// Not actually safe: This is needed for join to return a tuple.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn join<O>(output: stn::mem::MaybeUninit<O>) -> O {
        unsafe { output.assume_init() }
    }

    /// Not actually safe: This is needed to create a single-threaded "Mutex" to
    /// satisfy the borrow checker in `run!()`.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn ref_from_ptr<'a, T>(ptr: *mut T) -> &'a mut T {
        // Make clippy not complain
        fn deref<'a, T>(ptr: *mut T) -> &'a mut T {
            unsafe { ptr.as_mut().unwrap() }
        }

        deref(ptr)
    }
}

mod execute;
mod join;
mod run;
mod select;
mod tasks;
mod task_queue;
mod pin_mut;

pub use execute::Interrupt;
pub use tasks::Task;
pub use task_queue::TaskQueue;

#[cfg(feature = "std")]
mod spawner;
#[cfg(feature = "std")]
mod thread_interrupt;

#[cfg(feature = "std")]
pub use spawner::spawn_blocking;
#[cfg(feature = "std")]
pub use thread_interrupt::ThreadInterrupt;
