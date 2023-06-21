use core::future::Future;
use core::task::{RawWaker, RawWakerVTable, Waker};
use std::pin::Pin;
use std::task::{Context, Poll};

// Based on https://os.phil-opp.com/async-await/#simple-executor
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}
pub fn dummy_waker() -> Waker {
    // SAFETY: All RawWakerVTable functions in [dummy_raw_waker] are valid and thread safe.
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

/// "Future" that returns [Poll::Pending] on the first call, and [Poll::Ready] afterwards.
pub struct YieldFuture {
    first_exec: bool,
}

impl YieldFuture {
    pub fn new() -> Self {
        Self { first_exec: false }
    }
}

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        if !Pin::into_inner(self).first_exec {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}
