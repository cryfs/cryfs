use std::fmt::Debug;

use crate::{
    OpenInFlags,
    common::{DirEntry, FsResult, Gid, Mode, NodeAttrs, Uid},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    path::PathComponent,
};

pub trait Dir: AsyncDrop + Debug + Sized {
    type Device: super::Device;

    fn into_node(
        this: AsyncDropGuard<Self>,
    ) -> AsyncDropGuard<<Self::Device as super::Device>::Node>;

    fn entries(&self) -> impl Future<Output = FsResult<Vec<DirEntry>>> + Send;

    /// If the child doesn't exist, this must fail with [crate::FsError::NodeDoesNotExist] rather than returning a [super::Node]
    /// object that throws [crate::FsError::NodeDoesNotExist] when any of its members that require existence are called.
    fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<AsyncDropGuard<<Self::Device as super::Device>::Node>>> + Send;

    fn rename_child(
        &self,
        oldname: &PathComponent,
        newname: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: AsyncDropGuard<Self>,
        newname: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    fn create_child_dir(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> impl Future<
        Output = FsResult<(
            NodeAttrs,
            AsyncDropGuard<<Self::Device as super::Device>::Dir<'_>>,
        )>,
    > + Send;

    fn remove_child_dir(&self, name: &PathComponent) -> impl Future<Output = FsResult<()>> + Send;

    fn create_child_symlink(
        &self,
        name: &PathComponent,
        // TODO Use custom type for target that can wrap an absolute-or-relative path
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> impl Future<
        Output = FsResult<(
            NodeAttrs,
            AsyncDropGuard<<Self::Device as super::Device>::Symlink<'_>>,
        )>,
    > + Send;

    fn remove_child_file_or_symlink(
        &self,
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        flags: OpenInFlags,
    ) -> impl Future<
        Output = FsResult<(
            NodeAttrs,
            // TODO Should we return `File` instead of `Node`?
            AsyncDropGuard<<Self::Device as super::Device>::Node>,
            AsyncDropGuard<<Self::Device as super::Device>::OpenFile>,
        )>,
    > + Send;

    fn fsync(&self, datasync: bool) -> impl Future<Output = FsResult<()>> + Send;
}
