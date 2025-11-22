use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

pub struct EntryStateDropping {
    // TODO Would an Event be enough here or do we actually benefit from the shared future?
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
}
