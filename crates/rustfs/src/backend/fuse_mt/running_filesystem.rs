use cryfs_utils::at_exit::AtExitHandler;
use fuse_mt_fuser::BackgroundSession;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct RunningFilesystem {
    session: Arc<Mutex<Option<BackgroundSession>>>,

    /// This holds the `AtExitHandler` instance which makes sure the filesystem is unmounted if the process receives a SIGTERM, SIGINT, or SIGQUIT signal.
    /// We need to keep this alive as a RAII guard, when [RunningFilesystem] is destructed, the exit handler will be dropped as well.
    #[allow(dead_code)]
    unmount_atexit: AtExitHandler,
}

impl RunningFilesystem {
    pub(super) fn new(session: Arc<Mutex<Option<BackgroundSession>>>) -> Self {
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

    pub fn unmount_join(self) {
        if let Some(session) = self.session.lock().unwrap().take() {
            session.join();
        }
    }

    pub fn block_until_unmounted(&self) {
        loop {
            let session = self.session.lock().unwrap();
            if let Some(session) = &*session {
                if session.guard.is_finished() {
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

// No need to implement Drop for RunningFilesystem because `BackgroundSession` already unmounts on Drop.
