use async_trait::async_trait;
use std::path::Path;

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    type Node: super::Node;
    type Dir: super::Dir;

    // TODO Here and elsewhere in the interface, std::io::Result is probably the wrong error handling strategy
    async fn load_node(&self, path: &Path) -> std::io::Result<Self::Node>;

    async fn load_dir(&self, path: &Path) -> std::io::Result<Self::Dir>;
}
