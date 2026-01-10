use cryfs_utils::at_exit::AtExitHandler;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub trait BackgroundSession {
    fn join(self);
    fn is_finished(&self) -> bool;
}

#[cfg(all(feature = "fuse_mt", not(feature = "fuser")))]
impl BackgroundSession for fuser::BackgroundSession {
    fn join(self) {
        self.join();
    }
    fn is_finished(&self) -> bool {
        self.guard.is_finished()
    }
}

#[cfg(feature = "fuser")]
impl BackgroundSession for fuser::BackgroundSession {
    fn join(self) {
        self.join();
    }
    fn is_finished(&self) -> bool {
        self.guard.is_finished()
    }
}

pub struct RunningFilesystem<BS>
where
    BS: BackgroundSession + Send + 'static,
{
    session: Arc<Mutex<Option<BS>>>,

    /// The tokio runtime handle used by the FUSE session. We store this so we can properly
    /// join the session in a way that avoids deadlocks when called from tokio contexts.
    runtime: tokio::runtime::Handle,

    /// This holds the `AtExitHandler` instance which makes sure the filesystem is unmounted if the process receives a SIGTERM, SIGINT, or SIGQUIT signal.
    /// We need to keep this alive as a RAII guard, when [RunningFilesystem] is destructed, the exit handler will be dropped as well.
    #[allow(dead_code)]
    unmount_atexit: AtExitHandler,
}

impl<BS: BackgroundSession> RunningFilesystem<BS>
where
    BS: BackgroundSession + Send + 'static,
{
    #[cfg(any(feature = "fuser", feature = "fuse_mt"))]
    pub(super) fn new(session: BS, runtime: tokio::runtime::Handle) -> Self {
        let session = Arc::new(Mutex::new(Some(session)));

        // IMPORTANT: In test builds, skip creating signal handlers to avoid cross-test interference.
        // When tests run concurrently, signal handler tests in cryfs-utils send process-wide signals
        // via libc::raise(SIGTERM/SIGINT/SIGQUIT), which trigger ALL registered signal handlers
        // including FUSE cleanup handlers. This causes unexpected unmount attempts mid-test leading
        // to deadlocks. In tests, cleanup happens through Drop, not through signals.
        #[cfg(not(test))]
        let unmount_atexit = {
            let session_clone = session.clone();
            let runtime_clone = runtime.clone();
            AtExitHandler::new("RunningFilesystem.unmount", move || {
                log::info!("Received exit signal, unmounting filesystem...");
                if let Some(session) = session_clone.lock().unwrap().take() {
                    Self::join_session_blocking(session, &runtime_clone);
                }
                log::info!("Received exit signal, unmounting filesystem...done");
            })
        };

        // In tests, create a no-op handler that doesn't register signal handlers
        #[cfg(test)]
        let unmount_atexit = {
            AtExitHandler::new("RunningFilesystem.unmount.test_noop", || {
                // No-op: tests handle cleanup through Drop, not signals
            })
        };

        Self {
            session,
            runtime,
            unmount_atexit,
        }
    }

    /// Join the FUSE session without blocking tokio worker threads.
    ///
    /// ## The Deadlock Problem
    ///
    /// Calling `session.join()` directly from a tokio worker can cause deadlocks:
    /// 1. The FUSE background thread uses `runtime.block_on()` for async filesystem operations
    /// 2. If all tokio workers are blocked waiting for the FUSE thread (via `join()`), there
    ///    are no workers available to execute the async operations
    /// 3. Deadlock: FUSE thread waits for tokio workers, tokio workers wait for FUSE thread
    ///
    /// ## The Solution
    ///
    /// We spawn the join operation in a dedicated OS thread and do NOT wait for it to complete.
    /// This sacrifices structured concurrency (the method returns before cleanup completes) but
    /// avoids the deadlock. The FUSE thread will still clean up properly:
    /// 1. Dropping `BackgroundSession` drops the Mount, triggering unmount
    /// 2. The FUSE thread receives ENODEV from the kernel
    /// 3. The thread exits cleanly on its own
    ///
    /// For production usage (non-test), callers should use `block_until_unmounted()` to wait
    /// for the filesystem to fully unmount before proceeding.
    fn join_session_blocking(session: BS, _runtime: &tokio::runtime::Handle) {
        // Spawn join in a dedicated thread to avoid tying up tokio workers
        thread::spawn(move || {
            session.join();
        });
        // Note: We intentionally don't join this thread. The FUSE session will clean up
        // asynchronously, which is necessary to avoid deadlocks with the tokio runtime.
    }

    pub fn unmount_join(&self) {
        // TODO For unmount to work correctly, we may have to do DokanRemoveMountPoint in Dokan. That's what C++ CryFS did at least.

        if let Some(session) = self.session.lock().unwrap().take() {
            Self::join_session_blocking(session, &self.runtime);
        }
    }

    pub fn unmount_on_trigger(&self, unmount_trigger: CancellationToken) {
        let session_clone = self.session.clone();
        let runtime = self.runtime.clone();
        tokio::task::spawn(async move {
            unmount_trigger.cancelled().await;
            if let Some(session) = session_clone.lock().unwrap().take() {
                Self::join_session_blocking(session, &runtime);
            }
        });
    }

    pub fn block_until_unmounted(&self) {
        loop {
            let session = self.session.lock().unwrap();
            if let Some(session) = &*session {
                if session.is_finished() {
                    return;
                }
            } else {
                // Session was dropped, so we're unmounted
                return;
            }
            std::mem::drop(session);
            // TODO Use condition variable instead of busy waiting
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl<BS> Drop for RunningFilesystem<BS>
where
    BS: BackgroundSession + Send + 'static,
{
    fn drop(&mut self) {
        self.unmount_join();
    }
}
