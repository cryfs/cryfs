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
    pub(super) fn new(session: BS) -> Self {
        let session = Arc::new(Mutex::new(Some(session)));
        let session_clone = session.clone();
        let unmount_atexit = AtExitHandler::new("RunningFilesystem.unmount", move || {
            log::info!("Received exit signal, unmounting filesystem...");
            if let Some(session) = session_clone.lock().unwrap().take() {
                // Drop without join() to avoid deadlocks (see unmount_join for detailed explanation)
                drop(session);
            }
            log::info!("Received exit signal, unmounting filesystem...done");
        });

        Self {
            session,
            unmount_atexit,
        }
    }

    pub fn unmount_join(&self) {
        // TODO For unmount to work correctly, we may have to do DokanRemoveMountPoint in Dokan. That's what C++ CryFS did at least.

        if let Some(session) = self.session.lock().unwrap().take() {
            // IMPORTANT: We don't call session.join() here because it can cause deadlocks when called
            // from within a tokio runtime context. The FUSE background thread uses runtime.block_on()
            // for async operations. If we block on join() from a tokio worker thread, and the FUSE
            // thread is waiting for tokio workers to make progress, we create a circular dependency.
            //
            // Instead, we simply drop the session without calling join(). When BackgroundSession
            // is dropped, it:
            // 1. Drops the Mount, which triggers unmount (fusermount -u or libc::umount)
            // 2. Detaches the background thread (by dropping JoinHandle without calling join())
            //
            // The FUSE background thread will continue running until Session::run() receives ENODEV
            // from the kernel (triggered by the unmount), then cleanly exits. This is safe and avoids
            // the deadlock.
            drop(session);
        }
    }

    pub fn unmount_on_trigger(&self, unmount_trigger: CancellationToken) {
        let session_clone = self.session.clone();
        tokio::task::spawn(async move {
            unmount_trigger.cancelled().await;
            if let Some(session) = session_clone.lock().unwrap().take() {
                // Drop without join() to avoid deadlocks (see unmount_join for detailed explanation)
                drop(session);
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
