use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

pub struct EntryStateDropping {
    future: Shared<BoxFuture<'static, ()>>,
}

impl EntryStateDropping {
    pub fn new(future: Shared<BoxFuture<'static, ()>>) -> Self {
        Self { future }
    }

    pub fn new_dummy() -> Self {
        Self::new(futures::future::ready(()).boxed().shared())
    }

    pub fn future(&self) -> &Shared<BoxFuture<'static, ()>> {
        &self.future
    }

    pub fn into_future(self) -> Shared<BoxFuture<'static, ()>> {
        self.future
    }
}
