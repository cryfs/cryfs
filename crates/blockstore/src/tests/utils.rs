use cryfs_utils::{data::Data, testutils::data_fixture::DataFixture};

use crate::BlockId;

pub fn blockid(seed: u64) -> BlockId {
    BlockId::from_slice(data(16, seed).as_ref()).unwrap()
}

pub fn data(size: usize, seed: u64) -> Data {
    DataFixture::new(seed).get(size).into()
}
