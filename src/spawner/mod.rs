// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Once,
    },
    task::{Context, Poll, Waker},
};

mod thread_pool;

use thread_pool::ThreadPool;

struct ThreadFuture<R> {
    shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)>,
    handle: Option<thread_pool::ThreadHandle>,
    ret: Arc<Mutex<Option<R>>>,
}

impl<R> Future for ThreadFuture<R> {
    type Output = R;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if !self.shared_state.1.load(Ordering::Relaxed) {
            let mut shared_state = self.shared_state.0.lock().unwrap();

            *shared_state = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready({
                self.handle.take().unwrap().join();
                self.ret.lock().unwrap().take().unwrap()
            })
        }
    }
}

impl<R> ThreadFuture<R> {
    fn new<F>(function: F) -> Self
    where
        F: FnOnce() -> R,
        F: Send + 'static,
        R: Send + 'static,
    {
        let shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)> =
            Arc::new((Mutex::new(None), AtomicBool::new(false)));

        let thread_shared_state = shared_state.clone();

        let ret = Arc::new(Mutex::new(None));
        let thread_ret = ret.clone();

        let handle = Some(thread_pool().spawn(move || {
            *thread_ret.lock().unwrap() = Some(function());
            let mut shared_state = thread_shared_state.0.lock().unwrap();
            thread_shared_state.1.store(true, Ordering::Relaxed);
            if let Some(waker) = shared_state.take() {
                waker.wake()
            }
        }));

        ThreadFuture {
            shared_state,
            handle,
            ret,
        }
    }
}

static mut THREAD_POOL: MaybeUninit<Arc<ThreadPool>> = MaybeUninit::uninit();
static START: Once = Once::new();

// Return the global thread pool.
#[allow(unsafe_code)]
fn thread_pool() -> Arc<ThreadPool> {
    // unsafe: initialize thread pool only on first call
    START.call_once(|| unsafe {
        ptr::write(THREAD_POOL.as_mut_ptr(), ThreadPool::new());
    });
    // unsafe: already initialized so dereference won't be UB.
    unsafe { (*THREAD_POOL.as_ptr()).clone() }
}

/// **std** feature required.  Construct a future from a blocking function to
/// be run on a dynamically sized thread pool.
pub fn spawn_blocking<F, R>(function: F) -> impl Future<Output = R>
where
    F: FnOnce() -> R,
    F: Send + 'static,
    R: Send + 'static,
{
    ThreadFuture::new(function)
}
