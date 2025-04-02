use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
};

// Simple reimplementation of Map from futures-util to avoid pulling that in
pub struct MapFuture<F, M> {
    future: F,
    mapper: M,
}

impl<F, M> MapFuture<F, M> {
    pub fn new(future: F, mapper: M) -> Self {
        Self { future, mapper }
    }
}

impl<F: Future, M: Fn(F::Output) -> O, O> Future for MapFuture<F, M> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let output = ready!(unsafe { Pin::new_unchecked(&mut this.future) }.poll(cx));
        let mapped_output = (this.mapper)(output);

        Poll::Ready(mapped_output)
    }
}
