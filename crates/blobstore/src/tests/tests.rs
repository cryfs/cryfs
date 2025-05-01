use super::fixture::Fixture;
use crate::BlobId;
use crate::BlobStore as _;

/// This macro instantiates all blobstore tests for a given blobstore.
/// See [Fixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_blobstore_specific_tests {
    ($target: ty) => {
        $crate::instantiate_blobstore_specific_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        $crate::_instantiate_blobstore_specific_tests!(@module load, $target, $tokio_test_args,
            test_givenEmptyBlobstore_whenLoadingNonexistingBlob_thenReturnsNone,
            test_givenNonEmptyBlobstore_whenLoadingNonexistingBlob_thenReturnsNone,
        );
    };
}

#[macro_export]
macro_rules! _instantiate_blobstore_specific_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_blobstore_specific_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::tests::fixture::Fixture>::new();
            $crate::tests::tests::$module_name::$head_test_case(fixture).await
        }
        $crate::_instantiate_blobstore_specific_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

pub mod load {
    use super::*;

    pub async fn test_givenEmptyBlobstore_whenLoadingNonexistingBlob_thenReturnsNone(
        mut f: impl Fixture,
    ) {
        let mut store = f.store().await;

        let loaded = store
            .load(&BlobId::from_hex("1491BB4932A389EE14BC7090AC772972").unwrap())
            .await
            .unwrap();
        assert!(loaded.is_none());

        drop(loaded);
        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonEmptyBlobstore_whenLoadingNonexistingBlob_thenReturnsNone(
        mut f: impl Fixture,
    ) {
        let mut store = f.store().await;

        store
            .try_create(&BlobId::from_hex("AB0DC45269804AC6B1CF95391895DDF1").unwrap())
            .await
            .unwrap();

        let loaded = store
            .load(&BlobId::from_hex("1491BB4932A389EE14BC7090AC772972").unwrap())
            .await
            .unwrap();
        assert!(loaded.is_none());

        drop(loaded);
        store.async_drop().await.unwrap();
    }

    // TODO More `load` tests
}

// TODO More tests
