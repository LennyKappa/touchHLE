use core::future::Future;
use core::task::{RawWaker, RawWakerVTable, Waker};
use std::ops::{Deref, DerefMut};
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
    been_exec: bool,
}

impl YieldFuture {
    pub fn new() -> Self {
        Self { been_exec: false }
    }
}

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        let self_in = self.get_mut();
        if !self_in.been_exec {
            self_in.been_exec = true;
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}

// IMM: blehhh this is hacky
pub struct NullableBox<T> {
    inner: Option<Box<T>>,
}

impl<T> NullableBox<T> {
    #[allow(unused)]
    pub fn new(obj: T) -> NullableBox<T> {
        NullableBox {
            inner: Some(Box::new(obj)),
        }
    }

    #[allow(unused)]
    pub fn null() -> NullableBox<T> {
        NullableBox { inner: None }
    }

    #[allow(unused)]
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T> Deref for NullableBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap().deref()
    }
}

impl<T> DerefMut for NullableBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap().deref_mut()
    }
}
impl<T> AsRef<T> for NullableBox<T> {
    fn as_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}
impl<T> AsMut<T> for NullableBox<T> {
    fn as_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}
