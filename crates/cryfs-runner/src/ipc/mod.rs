mod pipe;
mod rpc;
mod spawn;

pub use rpc::{RpcClient, RpcConnection, RpcServer};
pub use spawn::{
    rpc_server_from_inherited_fds, send_handshake, start_background_process,
    start_background_process_with_exe,
};
