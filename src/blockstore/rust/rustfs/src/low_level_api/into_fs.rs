use super::AsyncFilesystem;

pub trait IntoFs<Fs: AsyncFilesystem> {
    fn into_fs(self) -> Fs;
}

impl<Fs> IntoFs<Fs> for Fs
where
    Fs: AsyncFilesystem,
{
    fn into_fs(self) -> Fs {
        self
    }
}
