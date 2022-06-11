use super::{BlockId, BlockStore, BlockStoreReader, BlockStoreWriter};

/// By writing a [Fixture] implementation and using the [instantiate_blockstore_                                                                                         ]
pub trait Fixture {
    type ConcreteBlockStore: BlockStore;

    fn new() -> Self;
    fn setup(&self) -> Self::ConcreteBlockStore;
}

pub async fn test_store_load<F: Fixture>(f: F) {
    let store = f.setup();

    let blockid = BlockId::from_hex("00000000000000000000000000000000").unwrap();

    store.store(&blockid, &[1, 2, 3]).await.unwrap();
    let loaded = store.load(&blockid).await.unwrap().unwrap();
    assert_eq!([1, 2, 3], *loaded);
}

#[macro_export]
macro_rules! _instantiate_blockstore_test {
    ($fixture: ty, $name: ident) => {
        #[tokio::test]
        async fn $name() {
            let fixture = <$fixture as $crate::blockstore::tests::Fixture>::new();
            $crate::blockstore::tests::$name(fixture).await
        }
    };
}

/// This macro instantiates all blockstore tests for a given blockstore.
/// See [Fixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_blockstore_tests {
    ($target: ty) => {
        $crate::_instantiate_blockstore_test!($target, test_store_load);
    };
}
