use pretty_assertions::assert_eq;
use std::cell::RefCell;

use crate::{
    filesystem_driver::FilesystemDriver,
    fixture::{ActionCounts, FilesystemFixture},
    rstest::{FixtureFactory, FixtureType},
};
use cryfs_blobstore::BlobStore as _;
use cryfs_blockstore::{LLBlockStore, OptimizedBlockStoreWriter};
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[must_use]
pub struct TestDriver<B, CreateBlockstoreFn, FS, FF>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    CreateBlockstoreFn: Fn() -> AsyncDropGuard<B>,
    FS: FilesystemDriver,
    FF: FixtureFactory<Driver = FS>,
{
    blockstore: CreateBlockstoreFn,
    fixture_factory: FF,
    atime_behavior: AtimeUpdateBehavior,
}

impl<B, CreateBlockstoreFn, FS, FF> TestDriver<B, CreateBlockstoreFn, FS, FF>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    CreateBlockstoreFn: Fn() -> AsyncDropGuard<B>,
    FS: FilesystemDriver,
    FF: FixtureFactory<Driver = FS>,
{
    #[must_use]
    pub fn new(
        blockstore: CreateBlockstoreFn,
        fixture_factory: FF,
        atime_behavior: AtimeUpdateBehavior,
    ) -> Self {
        Self {
            blockstore,
            fixture_factory,
            atime_behavior,
        }
    }

    #[must_use]
    pub fn create_filesystem(
        self,
    ) -> TestDriverWithFs<B, FS, impl AsyncFn() -> FilesystemFixture<B, FS>> {
        let fixture_type = self.fixture_factory.fixture_type();
        TestDriverWithFs {
            filesystem: async move || {
                self.fixture_factory
                    .create_filesystem((self.blockstore)(), self.atime_behavior)
                    .await
            },
            fixture_type,
        }
    }

    #[must_use]
    pub fn create_uninitialized_filesystem(
        self,
    ) -> TestDriverWithFs<B, FS, impl AsyncFn() -> FilesystemFixture<B, FS>> {
        let fixture_type = self.fixture_factory.fixture_type();
        TestDriverWithFs {
            filesystem: async move || {
                self.fixture_factory
                    .create_uninitialized_filesystem((self.blockstore)(), self.atime_behavior)
                    .await
            },
            fixture_type,
        }
    }
}

#[must_use]
pub struct TestDriverWithFs<B, FS, CreateFsFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
{
    filesystem: CreateFsFn,
    fixture_type: FixtureType,
}

impl<B, FS, CreateFsFn> TestDriverWithFs<B, FS, CreateFsFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
{
    #[must_use]
    pub fn setup<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<
        B,
        FS,
        CreateFsFn,
        impl AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
        SetupResult,
    >
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    {
        TestDriverWithFsAndSetupOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn: async move |fs| {
                let setup_result = setup_fn(fs).await;
                fs.blobstore.clear_cache_slow().await.unwrap();
                setup_result
            },
        }
    }

    #[must_use]
    pub fn setup_noflush<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    {
        TestDriverWithFsAndSetupOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
{
    fixture_type: FixtureType,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult>
    TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
{
    #[must_use]
    pub fn test<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<
        B,
        FS,
        CreateFsFn,
        SetupFn,
        SetupResult,
        impl AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    >
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        self.test_noflush(async move |fs, setup_result| {
            let test_result = (test_fn)(fs, setup_result).await;
            fs.blobstore.clear_cache_slow().await.unwrap();
            test_result
        })
    }

    #[must_use]
    pub fn test_noflush<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        TestDriverWithFsAndSetupOpAndTestOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    fixture_type: FixtureType,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    #[must_use]
    pub fn expect_op_counts(
        self,
        expected: impl FnOnce(FixtureType) -> ActionCounts,
    ) -> impl TestReady {
        TestReadyImpl {
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn: self.test_fn,
            expected_op_counts: expected(self.fixture_type),
        }
    }
}

#[must_use]
pub trait TestReady {
    /// Run the test and assert that the operation counts match the expected values.
    fn assert_op_counts(&self);

    #[cfg(feature = "benchmark")]
    fn run_benchmark(&self, b: &mut criterion::Bencher);
}

#[must_use]
pub struct TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
    expected_op_counts: ActionCounts,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    async fn _execute_test(&self) -> ActionCounts {
        let mut filesystem = (self.filesystem)().await;
        let setup_result = (self.setup_fn)(&mut filesystem).await;

        filesystem.blobstore.get_and_reset_counts();
        filesystem.hl_blockstore.get_and_reset_counts();
        filesystem.ll_blockstore.get_and_reset_counts();
        (self.test_fn)(&mut filesystem, setup_result).await;
        let counts = ActionCounts {
            blobstore: filesystem.blobstore.get_and_reset_counts(),
            high_level: filesystem.hl_blockstore.get_and_reset_counts(),
            low_level: filesystem.ll_blockstore.get_and_reset_counts(),
        };

        counts
    }

    fn _new_runtime() -> tokio::runtime::Runtime {
        // Runtime setup copied from #[tokio::test] code
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    }
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn> TestReady
    for TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    fn assert_op_counts(&self) {
        let expected = self.expected_op_counts;
        let actual = Self::_new_runtime().block_on(async { self._execute_test().await });
        assert_eq!(expected, actual, "Action counts mismatch expectations");
    }

    #[cfg(feature = "benchmark")]
    fn run_benchmark(&self, b: &mut criterion::Bencher) {
        let setup_fn = || {
            // Need to use futures instead of tokio because tokio can't start a runtime inside a runtime and criterion calls this from within a runtime (but doesn't allow us to pass in an async function).
            futures::executor::block_on(async {
                let mut filesystem = (self.filesystem)().await;
                let setup_result = (self.setup_fn)(&mut filesystem).await;
                RefCell::new(Some((filesystem, setup_result)))
            })
        };
        let test_fn = |input: &mut RefCell<Option<(FilesystemFixture<B, FS>, SetupResult)>>| {
            let (mut filesystem, setup_result) = input.replace(None).expect(
                // The RefCell is a hack to make this FnMut (required by the criterion API) but we actually have a FnOnce.
                "Tried to run benchmark function multiple times without re-running setup",
            );
            async move {
                (self.test_fn)(&mut filesystem, setup_result).await;
            }
        };
        b.to_async(Self::_new_runtime()).iter_batched_ref(
            setup_fn,
            test_fn,
            criterion::BatchSize::PerIteration,
        );
    }
}
