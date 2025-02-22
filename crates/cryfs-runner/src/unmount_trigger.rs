use atomic_time::AtomicInstant;
use cryfs_blockstore::IntegrityViolationError;
use std::{
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub enum TriggerReason {
    UnmountIdle,
    IntegrityViolation(IntegrityViolationError),
}

#[derive(Clone)]
pub struct UnmountTrigger {
    trigger: CancellationToken,
    trigger_reason: Arc<Mutex<Option<TriggerReason>>>,
}

impl UnmountTrigger {
    pub fn new() -> Self {
        Self {
            trigger: CancellationToken::new(),
            trigger_reason: Arc::new(Mutex::new(None)),
        }
    }

    pub fn trigger_after_idle_timeout(
        &self,
        last_filesystem_access_time: Arc<AtomicInstant>,
        unmount_after_idle_for: Duration,
    ) {
        let this = self.clone();
        tokio::task::spawn(async move {
            loop {
                if last_filesystem_access_time
                    .load(Ordering::Relaxed)
                    .elapsed()
                    > unmount_after_idle_for
                {
                    this.trigger_now(TriggerReason::UnmountIdle);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub fn trigger_now(&self, reason: TriggerReason) {
        // Concurrency: trigger_reason needs to be set before we actually cancel the trigger
        // because the cancellation triggers shutdown the file system, which will trigger
        // code to read the trigger_reason. So this order prevents a race condition.
        *self.trigger_reason.lock().unwrap() = Some(reason);
        self.trigger.cancel();
    }

    pub fn waiter(&self) -> CancellationToken {
        self.trigger.clone()
    }

    pub fn trigger_reason(&self) -> &Arc<Mutex<Option<TriggerReason>>> {
        &self.trigger_reason
    }
}

// TODO Test
