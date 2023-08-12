use async_trait::async_trait;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use super::dir::Dir;
use super::node::Node;
use crate::common::{AbsolutePath, FsError, FsResult, Statfs};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    // TODO Do we need those Send bounds on Node and Dir?
    type Node: super::Node<Device = Self> + AsyncDrop<Error = FsError> + Debug + Send + Sync;
    type Dir<'a>: super::Dir<Device = Self> + Send + Sync
    where
        Self: 'a;
    type Symlink<'a>: super::Symlink<Device = Self> + Send
    where
        Self: 'a;
    type File<'a>: super::File<Device = Self>
    where
        Self: 'a;
    type OpenFile: super::OpenFile + AsyncDrop<Error = FsError>;

    async fn rootdir(&self) -> FsResult<Self::Dir<'_>>;
    async fn rename(&self, from: &AbsolutePath, to: &AbsolutePath) -> FsResult<()>;
    async fn statfs(&self) -> FsResult<Statfs>;
    async fn destroy(self);

    // If the node at `path` doesn't exist, it's ok to either immediately fail with [FsError::NodeDoesNotExist]
    // or to return a [Node] object that throws [FsError::NodeDoesNotExist] when any of its members that
    // require existence are called.
    async fn lookup(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<Self::Node>>
    where
        // TODO Why is Self: 'static needed?
        Self: 'static,
    {
        let rootdir = self
            .rootdir()
            .await?
            // TODO Can we do this without first converting `rootdir` to `Node` by calling `.as_node()`, and then immediately calling `.as_dir()` in the loop below?
            .as_node();

        match path.split_last() {
            None => {
                // We're being asked to load the root dir
                Ok(rootdir)
            }
            // TODO Simplify code
            Some((parent_path, node_name)) => {
                let mut currentnode = rootdir;
                for component in parent_path {
                    let dir = currentnode.as_dir();
                    let child = {
                        let dir = dir.await;
                        match dir {
                            Ok(dir) => {
                                let child = dir.lookup_child(component);
                                child.await
                            }
                            Err(err) => Err(err),
                        }
                    };
                    currentnode.async_drop().await?;
                    currentnode = child?;
                }
                let dir = currentnode.as_dir();
                let child = {
                    let dir = dir.await;
                    match dir {
                        Ok(dir) => {
                            let child = dir.lookup_child(node_name);
                            child.await
                        }
                        Err(err) => Err(err),
                    }
                };
                currentnode.async_drop().await?;
                child
            }
        }
    }
}
