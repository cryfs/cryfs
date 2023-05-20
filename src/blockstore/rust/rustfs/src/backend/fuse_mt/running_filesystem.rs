pub struct RunningFilesystem {
    handle: fuser::BackgroundSession,
}

impl RunningFilesystem {
    pub(super) fn new(handle: fuser::BackgroundSession) -> Self {
        Self { handle }
    }

    pub fn unmount_join(self) {
        self.handle.join();
    }
}
