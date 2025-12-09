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
    pub(super) fn new(session: Arc<Mutex<Option<BS>>>) -> Self {
        let session_clone = session.clone();
        let unmount_atexit = AtExitHandler::new(move || {
            log::info!("Received exit signal, unmounting filesystem...");
            if let Some(session) = session_clone.lock().unwrap().take() {
                session.join();
            }
            log::info!("Received exit signal, unmounting filesystem...done");
        });

        let fs = Self {
            session,
            unmount_atexit,
        };

        fs
    }

    pub fn unmount_join(&self) {
        // TODO For unmount to work correctly, we may have to do DokanRemoveMountPoint in Dokan. That's what C++ CryFS did at least.

        if let Some(session) = self.session.lock().unwrap().take() {
            session.join();
        }
    }

    pub fn unmount_on_trigger(&self, unmount_trigger: CancellationToken) {
        let session_clone = self.session.clone();
        tokio::task::spawn(async move {
            unmount_trigger.cancelled().await;
            if let Some(session) = session_clone.lock().unwrap().take() {
                session.join();
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
