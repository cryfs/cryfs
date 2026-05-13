use cryfs_utils::at_exit::AtExitHandler;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub trait BackgroundSession {
    /// Unmount the filesystem and wait for the session's background thread (which runs `destroy()`) to
    /// finish. Returns any unmount error or background-thread failure to the caller, which decides how
    /// to handle it (production Drop/at-exit/trigger paths log; the test harness asserts).
    fn join(self) -> io::Result<()>;
    fn is_finished(&self) -> bool;
}

// The fuse_mt backend mounts via fuser 0.16 (`fuser_fusemt`), a distinct type from the fuser-0.17
// `BackgroundSession` below, so both impls coexist when both features are enabled. fuser 0.16's
// `join()` actively unmounts (it has no `umount_and_join`), so unlike the 0.17 impl we just call it.
#[cfg(feature = "fuse_mt")]
impl BackgroundSession for fuser_fusemt::BackgroundSession {
    fn join(self) -> io::Result<()> {
        // fuser 0.16's `join()` unmounts + joins but `.unwrap()`s internally, so it can't report an
        // error — it panics on failure. Nothing better is possible with 0.16; just call it.
        self.join();
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.guard.is_finished()
    }
}

#[cfg(feature = "fuser")]
impl BackgroundSession for fuser::BackgroundSession {
    fn join(self) -> io::Result<()> {
        // In fuser 0.17, plain `join()` only waits for the session thread to finish; `umount_and_join()`
        // is what actively unmounts. It returns `Err` if the unmount failed OR the background thread
        // (which runs `destroy()`) panicked — fuser converts that thread panic into an `io::Error`. We
        // surface that to the caller rather than deciding here, so each call site picks its own policy.
        //
        // NOTE: when unprivileged, `umount_and_join()` unmounts via a lazy `umount2(MNT_DETACH)`, which
        // does not abort the FUSE connection until the kernel destroys the superblock. If kernel-side
        // references linger (e.g. dcache entries pinned by earlier lookups), the background thread can
        // block in `read()` on `/dev/fuse` and this join can hang. The test harness force-aborts the
        // connection on teardown to avoid exactly this; production relies on the kernel tearing the
        // connection down on its own (production doesn't unmount mid-activity, so it isn't seen there).
        // TODO If a production hang is ever observed here, escalate to "join with a timeout, abort the
        // connection only if it doesn't complete" — never an unconditional abort, which would fail any
        // in-flight request on every unmount.
        self.umount_and_join()
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
                // We're in a signal handler with nowhere to propagate, so log any failure.
                if let Err(err) = session.join() {
                    log::error!("Error unmounting filesystem on exit signal: {err}");
                }
            }
            log::info!("Received exit signal, unmounting filesystem...done");
        });

        Self {
            session,
            unmount_atexit,
        }
    }

    pub fn unmount_join(&self) -> io::Result<()> {
        // TODO For unmount to work correctly, we may have to do DokanRemoveMountPoint in Dokan. That's what C++ CryFS did at least.

        match self.session.lock().unwrap().take() {
            Some(session) => session.join(),
            None => Ok(()),
        }
    }

    pub fn unmount_on_trigger(&self, unmount_trigger: CancellationToken) {
        let session_clone = self.session.clone();
        tokio::task::spawn(async move {
            unmount_trigger.cancelled().await;
            if let Some(session) = session_clone.lock().unwrap().take() {
                // Detached task with nowhere to propagate, so log any unmount failure.
                if let Err(err) = session.join() {
                    log::error!("Error unmounting filesystem on trigger: {err}");
                }
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
        // Log rather than `safe_panic!` here (unlike the test `Runner`, which asserts): a failed unmount
        // at shutdown is usually an expected operational error (mountpoint busy, or already unmounted
        // externally) rather than a bug, and `unmount_join`'s error can't be distinguished from a genuine
        // `destroy()` panic. Aborting a user's process on a benign unmount hiccup is worse than logging.
        // (`safe_panic!` *would* be double-panic-safe — that's not why we avoid it here.)
        if let Err(err) = self.unmount_join() {
            log::error!("Error unmounting filesystem: {err}");
        }
    }
}
